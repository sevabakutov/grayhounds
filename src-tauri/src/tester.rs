use anyhow::{
    anyhow, 
    Result
};
use chrono::{
    TimeZone, 
    Utc
};
use mongodb::bson::{
    self, 
    doc, 
    Bson, 
    Document
};
use futures::stream::TryStreamExt;

use crate::{
    client::OpenAIClient, 
    constants::{
        DOG_INFO_COLLECTION, 
        MAX_REQUEST_DEFENCE
    }, 
    models::{
        OddsRange, 
        RequestsInfo, 
        Settings, 
        TestDateTime, 
        TestResults
    }, 
    utils::build_requests
};

#[allow(unused)]
pub struct Tester {
    db_client: mongodb::Client,
    config: Settings,
    date_time: TestDateTime,
    distances: Vec<i32>
}

impl Tester {
    pub fn new(
        config: Settings, 
        db_client: mongodb::Client,
        date_time: TestDateTime,
        distances: Vec<i32>
    ) -> Self {
        Self { 
            db_client, 
            config,
            date_time,
            distances
        }
    }

    async fn generate_races(&self) -> Result<RequestsInfo> {
        let database = self.db_client
            .default_database()
            .ok_or_else(|| anyhow!("No default database"))?;
        
        let collection = database.collection::<Document>(DOG_INFO_COLLECTION);

        let distances_bson = self.distances
            .iter()
            .map(|&d| Bson::Int32(d))
            .collect::<Vec<Bson>>();

        let filter = match &self.date_time {
            TestDateTime::FixedDateTime(naive_dt) => {
                let utc_dt = Utc.from_utc_datetime(naive_dt);
                let fixed = bson::DateTime::from_millis(utc_dt.timestamp_millis());

                doc! {
                    "raceDateTime": fixed,
                    "distance": { "$in": Bson::Array(distances_bson) }
                }
            }

            TestDateTime::RangeDateTime(range) => {
                let start_utc = Utc.from_utc_datetime(&range.start_date_time);
                let end_utc   = Utc.from_utc_datetime(&range.end_date_time);

                let start_bson = bson::DateTime::from_millis(start_utc.timestamp_millis());
                let end_bson   = bson::DateTime::from_millis(end_utc.timestamp_millis());

                doc! {
                    "raceDateTime": {
                        "$gte": start_bson,
                        "$lte": end_bson
                    },
                    "distance": { "$in": Bson::Array(distances_bson) }
                }
            }
        };

        let pipeline = vec![
            doc! { "$match": filter.clone() },
            doc! { "$sort": { "raceDateTime": 1_i32 } },
            doc! { 
                "$group": { 
                    "_id": "$raceId",
                    "raceDateTime": { "$first": "$raceDateTime" }
                } 
            },
            doc! { "$sort": { "raceDateTime": 1_i32 } },
        ];

        let race_ids = collection
            .aggregate(pipeline)
            .await?
            .try_collect::<Vec<Document>>()
            .await?
            .into_iter()
            .map(|d| match d.get("_id").expect("no _id in doc") {
                Bson::Int64(v) => *v,
                Bson::Int32(v) => *v as i64,
                _ => unreachable!(),
            })
            .collect::<Vec<i64>>();

        let total = race_ids.len();
        log::info!("Found: {total} records");

        let mut races = Vec::new();
        for (idx, race_id) in race_ids.into_iter().enumerate() {
            log::info!("> [{}/{}] raceId={}", idx + 1, total, race_id);
            
            let filter = doc! { "raceId": Bson::Int64(race_id) };
            let race_docs: Vec<Document> = collection
                .find(filter)
                .await?
                .try_collect()
                .await?;

            if race_docs.is_empty() {
                log::info!("  - no docs, skip");
                continue;
            }

            let meta = &race_docs[0];
            let race_date_time = meta.get_datetime("raceDateTime")?.to_owned();

            // "2025-06-02T14:18:00Z"
            let rfc3339 = race_date_time.try_to_rfc3339_string()?;
            let mut parts = rfc3339.split('T');
            let race_date = parts.next().unwrap().to_string(); // => "2024-02-08"
            let time_with_z = parts.next().unwrap();          
            let race_time = time_with_z.trim_end_matches('Z').to_string();

            let dist = meta.get_i32("distance")?;
            let track_name = meta.get_str("trackName")?;

            let mut min_rt = f64::INFINITY;
            for doc in &race_docs {
                if let Some(Bson::Double(val)) = doc.get("resultRunTime") {
                    min_rt = min_rt.min(*val);
                }
            }
            let winners_time = if min_rt.is_finite() {
                min_rt
            } else {
                0.0
            };

            let mut dogs = Vec::with_capacity(race_docs.len());
            for dog_result in &race_docs {
                let dog_name = dog_result.get_str("dogName")?;
                let trap_number = match dog_result
                    .get_i32("trapNumber")
                {
                    Ok(val) => val,
                    Err(_) => match dog_result.get_i64("trapNumber") {
                        Ok(val) => val as i32,
                        Err(err) => {
                            log::error!("Error: {err}");
                            unreachable!()
                        }
                    }
                };

                let dog_id = match dog_result
                    .get_i32("dogId") 
                {
                    Ok(val) => val,
                    Err(_) => match dog_result.get_i64("dogId") {
                        Ok(val) => val as i32,
                        Err(err) => {
                            log::error!("Error: {err}");
                            unreachable!() 
                        }
                    }
                };

                let mut form_docs: Vec<_> = collection
                    .find(doc! { "dogId": dog_id })
                    .await?
                    .try_collect()
                    .await?;

                form_docs.sort_by(|a, b| b.get_datetime("raceDateTime").unwrap().cmp(a.get_datetime("raceDateTime").unwrap()));
                form_docs.truncate(5);

                let mut forms_array = Vec::with_capacity(form_docs.len());
                for form in form_docs {
                    let distance = match form
                        .get_i32("distance")
                    {
                        Ok(val) => val,
                        Err(_) => match dog_result.get_i64("distance") {
                            Ok(val) => val as i32,
                            Err(err) => {
                                log::error!("Error: {err}");
                                unreachable!() 
                            }
                        }
                    };

                    let sectional = form.get_f64("resultSectionalTime");
                    let trap = form.get_i32("trapNumber")?;
                    let weight = form.get_f64("resultDogWeight");
                    let by = form.get_str("resultBtnDistance")?;
                    let grade = form.get_str("raceClass")?;
                    let comm = form.get_str("resultComment")?;
                    let calc = form.get_f64("resultRunTime")?;
                    let outcome = form.get_i32("resultPosition")?;

                    let going_type = match dog_result
                        .get_i32("raceGoing")
                    {
                        Ok(val) => val,
                        Err(_) => match dog_result.get_i64("raceGoing") {
                            Ok(val) => val as i32,
                            Err(err) => {
                                log::error!("{err}");
                                panic!()
                            }
                        }
                    };

                    let form_doc = doc! {
                        "btnDistance": by,
                        "resultRunTime": calc,
                        "resultDogWeight": weight.map(Bson::from).unwrap_or(Bson::Null),
                        "raceComment": comm,
                        "raceWinnersTime": winners_time,
                        "goingType": going_type,
                        "raceClass": grade,
                        "trap": trap,
                        "sectionalTime": sectional.map(Bson::from).unwrap_or(Bson::Null),
                        "resultPosition": outcome,
                        "distance": distance,
                    };
                    forms_array.push(Bson::Document(form_doc));
                }

                let dog_doc = doc!{
                    "trackName": track_name,
                    "trapNumber": trap_number,
                    "dogName": dog_name,
                    "forms": Bson::Array(forms_array),
                };
                dogs.push(Bson::Document(dog_doc));
            }

            let n = dogs.len();
            if !(5..=6).contains(&n) {
                log::warn!("Skip race, participants amount: {}", n);
                continue;
            }

            let race_doc = doc! {
                "race_date": race_date,
                "race_time": race_time,
                "race_id": race_id,
                "distance": dist,
                "dogs": Bson::Array(dogs),
            };

            // println!("Race\n:{:?}", race_doc.clone());

            races.push(race_doc);

            if self.config.max_races <= races.len() {
                break;
            }
        }

        // println!("races: {:?}", races.clone());

        if MAX_REQUEST_DEFENCE < races.len() {
            races.truncate(self.config.max_races);
            log::info!("Defenced to {} requests", races.len());
        }

        let total_races = races.len();
        let requests = build_requests(
            races,
            database.clone(),
            self.config.clone()
        ).await?;

        Ok(RequestsInfo { requests, total_races })
    }

    pub async fn run(
        &self,
        initial_balance: f64,
        initial_stake: f64,
        odds_range: OddsRange,
        is_favorite_protected: bool
    ) -> Result<TestResults> {
        let requests_info = self.generate_races().await?;

        log::info!(
            "{} requests, {} races in total", 
            requests_info.requests.len(), 
            requests_info.total_races
        );

        let database = self.db_client
            .default_database()
            .ok_or_else(|| anyhow!("Failed to get default database"))?;

        let client = OpenAIClient::new(self.config.clone());
    
        client
            .test(
                requests_info, 
                database,
                initial_balance,
                initial_stake,
                odds_range,
                is_favorite_protected
            )
            .await
    }
}
