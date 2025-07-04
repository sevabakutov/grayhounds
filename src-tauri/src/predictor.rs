use std::collections::HashMap;

use anyhow::{
    anyhow, 
    Result
};
use chrono::{
    NaiveDate, 
    NaiveTime, 
    TimeZone, 
    Utc
};
use futures::stream::TryStreamExt;
use log::{
    error, 
    info
};
use mongodb::bson::{
    self, 
    doc, 
    to_document, 
    DateTime, 
    Document
};
use serde_json::json;
use chrono_tz::Europe::London;

use crate::{
    client::OpenAIClient,
    constants::{
        MAX_REQUEST_DEFENCE, 
        PREDICTIONS_COLLECTION, 
        RACES_COLLECTION, TIME_RANGES_COLLECTION
    },
    models::{
        PredictInput, 
        PredictResponse, 
        Settings, 
        Time
    },
    scrapper::Scrapper,
    utils::build_requests,
};

#[allow(unused)]
pub struct Predictor {
    fixed_date: NaiveDate,
    db_client: mongodb::Client, 
    config: Settings,
    distances: Vec<i32>,
    time: Time,
}

impl Predictor {
    pub async fn new(
        config: Settings,
        db_client: mongodb::Client,
        input: PredictInput,
    ) -> Self {
        let fixed_date = chrono::Utc::now().date_naive();
        let distances = input.distances;
        let time = input.time;

        Self {
            fixed_date,
            db_client,
            config,
            distances,
            time
        }
    }

    pub async fn create_request(&self) -> Result<Vec<HashMap<String, serde_json::Value>>> {
        let database = self.db_client
            .default_database()
            .ok_or_else(|| anyhow!("Not default DB"))?;

        let distances = &self.distances;
        let today = Utc::now().date_naive();
        
        let filter = match &self.time {
            Time::FixedTime(naive_time) => {
                let dt = Utc.from_utc_datetime(&today.and_time(*naive_time));
                let fixed = bson::DateTime::from_millis(dt.timestamp_millis());
                doc! {
                    "race_date_time": fixed,
                    "distance": { "$in": distances }
                }
            }
            Time::RangeTime(range) => {
                let start_dt = {
                    let dt = Utc.from_utc_datetime(&today.and_time(range.start_time));
                    bson::DateTime::from_millis(dt.timestamp_millis())
                };
                let end_dt = {
                    let dt = Utc.from_utc_datetime(&today.and_time(range.end_time));
                    bson::DateTime::from_millis(dt.timestamp_millis())
                };
                doc! {
                    "race_date_time": {
                        "$gte": start_dt,
                        "$lte": end_dt
                    },
                    "distance": { "$in": distances }
                }
            }
        };

        let pipeline = vec![
            doc! { "$match": filter.clone() },
            doc! { "$sort": { "race_date_time": 1_i32 } },
        ];

        let mut races: Vec<Document> = database
            .collection::<Document>(RACES_COLLECTION)
            .aggregate(pipeline)
            .await?
            .try_collect()
            .await?;

        log::info!("Found {} races", races.len());
        // log::info!("Races: {:?}", races.clone());

        if self.config.max_races < races.len() {
            races.truncate(self.config.max_races);
            log::info!("Truncated to {} requests", races.len());
        }

        if MAX_REQUEST_DEFENCE < races.len() {
            races.truncate(self.config.max_races);
            log::info!("Defenced to {} requests", races.len());
        }

        let requests = build_requests(races, database.clone(), self.config.clone()).await?;
        log::info!("{} requests", requests.len());

        Ok(requests)
    }

    pub async fn scrape_races(&self) -> Result<()> {
        let database = self
            .db_client
            .default_database()
            .expect("Failed to connect to default database");

        const DAILY_RACES_MIN_AMOUNT: u64 = 50;

        let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
        let today_end = Utc::now().date_naive().and_hms_opt(23, 59, 59).unwrap();

        let bson_start = bson::DateTime::from_millis(today_start.and_utc().timestamp_millis());
        let bson_end = bson::DateTime::from_millis(today_end.and_utc().timestamp_millis());

        let filter = doc! {
            "race_date_time": {
                "$gte": bson_start,
                "$lte": bson_end
            }
        };
        let daily_races_amount = database
            .collection::<Document>(RACES_COLLECTION)
            .count_documents(filter)
            .await?;

        info!("Found {} documments for today", daily_races_amount);

        if daily_races_amount > DAILY_RACES_MIN_AMOUNT {
            info!("Day was already scrapped!");
            Ok(())
        } else {
            info!("Races scrapping");
            let scrapper = Scrapper::new()?;
            
            let data = scrapper.get_all_dogs_data(&self.fixed_date).await?;

            let docs: Vec<Document> = data
                .into_iter()
                .filter_map(|item| {
                    let json_val = json!(item);
                    match mongodb::bson::to_document(&json_val) {
                        Ok(doc) => Some(doc),
                        Err(e) => {
                            error!("Error converting JSON to BSON: {:?}", e);
                            None
                        }
                    }
                })
                .collect();

            database
                .collection::<Document>(RACES_COLLECTION)
                .insert_many(docs)
                .await?;

            info!("Saved scrapped races to database!");
            Ok(())
        }
    }

    pub async fn save_predictions(&self, preds: &[PredictResponse]) -> Result<()> {
        if preds.is_empty() {
            return Ok(());
        }

        let collection = self
            .db_client
            .default_database()
            .ok_or_else(|| anyhow!("Not default DB"))?
            .collection::<Document>(PREDICTIONS_COLLECTION);

        let mut writes = Vec::with_capacity(preds.len());

        let today = Utc::now()
            .with_timezone(&London)
            .date_naive();
        let midnight  = London
            .from_local_datetime(&today
                .succ_opt()
                .unwrap()
                .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            )
            .single()
            .unwrap()
            .with_timezone(&Utc);
        let expire_at = DateTime::from_millis(midnight.timestamp_millis());

        for p in preds {
            let mut doc = to_document(&p)?;
            doc.insert("expireAt", expire_at);

            writes.push(doc);
        }

        collection
            .insert_many(writes)
            .await?;

        Ok(())
    }

    pub async fn save_time_ranges(&self) -> Result<()> {
        let today = Utc::now()
            .with_timezone(&London)
            .date_naive();
        let midnight  = London
            .from_local_datetime(&today
                .succ_opt()
                .unwrap()
                .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            )
            .single()
            .unwrap()
            .with_timezone(&Utc);
        let expire_at = DateTime::from_millis(midnight.timestamp_millis());
        
        let time = match &self.time {
            Time::FixedTime(time) => doc! {
                "startTime": time.format("%H:%M").to_string(),
                "endTime"  : null,
                "expireAt" : expire_at
            },
            Time::RangeTime(range) => doc! {
                "startTime": range.start_time.format("%H:%M").to_string(),
                "endTime"  : range.end_time.format("%H:%M").to_string(),
                "expireAt" : expire_at
            },
        };

        self.db_client
            .default_database()
            .ok_or_else(|| anyhow!("No default DB"))?
            .collection(TIME_RANGES_COLLECTION)
            .insert_one(time)
            .await?;

        Ok(())
    }

    pub async fn run(&self) -> Result<Vec<PredictResponse>> {
        self.scrape_races().await?;

        let requests = self.create_request().await?;

        let client = OpenAIClient::new(self.config.clone());
        let responses = client.send_multiple(requests).await?;

        self.save_predictions(&responses).await?;
        self.save_time_ranges().await?;

        Ok(responses)
    }
}
