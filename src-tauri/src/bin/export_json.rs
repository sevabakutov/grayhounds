use anyhow::{anyhow, Result};
use dogs_lib::{constants::{ALL_DISTANCES, DOG_INFO_COLLECTION}, models::{RangeDateTime, TestDateTime}};
use futures::TryStreamExt;
use mongodb::{bson::{self, doc, Bson, Document}, options::{ClientOptions, ServerApi, ServerApiVersion}, Client};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use mongodb::Cursor;
use serde_json::to_string;
use std::fs::File;
use std::io::Write;
use dotenv::dotenv;

// Define a struct to hold the state
struct RaceGenerator {
    date_time: TestDateTime,
    db_client: Client,
}

impl RaceGenerator {
    async fn generate_races(&self) -> Result<()> {
        let database = self.db_client
            .default_database()
            .ok_or_else(|| anyhow!("No default database"))?;
        
        let collection = database.collection::<Document>(DOG_INFO_COLLECTION);

        let distances_bson = ALL_DISTANCES
            .iter()
            .map(|&d| Bson::Int32(d as i32))
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

        let mut cursor: Cursor<Document> = collection.aggregate(pipeline).await?;
        let mut race_ids: Vec<i64> = Vec::new();
        while let Some(doc) = cursor.try_next().await? {
            let id = match doc.get("_id").expect("no _id in doc") {
                Bson::Int64(v) => *v,
                Bson::Int32(v) => *v as i64,
                _ => unreachable!(),
            };
            race_ids.push(id);
        }

        let total = race_ids.len();
        println!("Found: {} records", total);

        const BATCH_SIZE: usize = 500;
        let mut batch_index: usize = 0;
        let mut current_batch: Vec<Document> = Vec::new();
        for (idx, race_id) in race_ids.into_iter().enumerate() {
            println!("> [{}/{}] raceId={}", idx + 1, total, race_id);

            if idx <= 6500 {
                continue;
            }
            
            let filter = doc! { "raceId": Bson::Int64(race_id) };
            let mut cursor: Cursor<Document> = collection.find(filter).await?;
            let mut race_docs: Vec<Document> = Vec::new();
            while let Some(doc) = cursor.try_next().await? {
                race_docs.push(doc);
            }

            if race_docs.is_empty() {
                println!("  - no docs, skip");
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

            let mut dogs = Vec::with_capacity(race_docs.len());
            for dog_result in &race_docs {
                let dog_name = dog_result.get_str("dogName")?;
                let trap_number = match dog_result.get_i32("trapNumber") {
                    Ok(val) => val,
                    Err(_) => match dog_result.get_i64("trapNumber") {
                        Ok(val) => val as i32,
                        Err(err) => {
                            println!("Error: {}", err);
                            unreachable!()
                        }
                    }
                };

                let dog_id = match dog_result.get_i32("dogId") {
                    Ok(val) => val,
                    Err(_) => match dog_result.get_i64("dogId") {
                        Ok(val) => val as i32,
                        Err(err) => {
                            println!("Error: {}", err);
                            unreachable!() 
                        }
                    }
                };

                let mut cursor: Cursor<Document> = collection
                    .find(doc! { "dogId": Bson::Int32(dog_id) })
                    .await?;
                let mut form_docs: Vec<Document> = Vec::new();
                while let Some(doc) = cursor.try_next().await? {
                    form_docs.push(doc);
                }

                form_docs.sort_by(|a, b| b.get_datetime("raceDateTime").unwrap().cmp(a.get_datetime("raceDateTime").unwrap()));
                form_docs.truncate(5);

                let mut forms_array = Vec::with_capacity(form_docs.len());
                for form in form_docs {

                    let form_race_id = match form.get_i64("raceId") {
                        Ok(val) => val,
                        Err(_) => match form.get_i32("raceId") {
                            Ok(val) => val as i64,
                            Err(err) => {
                                println!("Error: {}", err);
                                unreachable!()
                            }
                        }
                    };

                    let form_filter = doc! { "raceId": Bson::Int64(form_race_id) };
                    let mut form_cursor: Cursor<Document> = collection.find(form_filter).await?;
                    let mut form_race_docs: Vec<Document> = Vec::new();
                    while let Some(doc) = form_cursor.try_next().await? {
                        form_race_docs.push(doc);
                    }

                    let mut form_min_rt = f64::INFINITY;
                    for doc in &form_race_docs {
                        if let Some(Bson::Double(val)) = doc.get("resultRunTime") {
                            form_min_rt = form_min_rt.min(*val);
                        }
                    }
                    let form_winners_time = if form_min_rt.is_finite() {
                        form_min_rt
                    } else {
                        0.0
                    };
                    let distance = match form.get_i32("distance") {
                        Ok(val) => val,
                        Err(_) => match dog_result.get_i64("distance") {
                            Ok(val) => val as i32,
                            Err(err) => {
                                println!("Error: {}", err);
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

                    let going_type = match dog_result.get_i32("raceGoing") {
                        Ok(val) => val,
                        Err(_) => match dog_result.get_i64("raceGoing") {
                            Ok(val) => val as i32,
                            Err(err) => {
                                println!("{}", err);
                                panic!()
                            }
                        }
                    };

                    let form_doc = doc! {
                        "btnDistance": by,
                        "resultRunTime": calc,
                        "resultDogWeight": weight.map(Bson::from).unwrap_or(Bson::Null),
                        "raceComment": comm,
                        "raceWinnersTime": form_winners_time,
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
                println!("Skip race, participants amount: {}", n);
                continue;
            }

            let race_doc = doc! {
                "race_date": race_date,
                "race_time": race_time,
                "race_id": race_id,
                "distance": dist,
                "dogs": Bson::Array(dogs),
            };

            current_batch.push(race_doc);

            if current_batch.len() == BATCH_SIZE {
                let start_label = if batch_index == 0 { 0 } else { batch_index * BATCH_SIZE + 1 };
                let end_label = (batch_index + 1) * BATCH_SIZE;
                let filename = format!("{}-{}.jsonl", start_label, end_label);
                let mut file = File::create(&filename)?;
                for doc in &current_batch {
                    let json_data = to_string(doc)?;
                    file.write_all(json_data.as_bytes())?;
                    file.write_all(b"\n")?;
                }
                println!("Exported batch to {}", filename);
                current_batch.clear();
                batch_index += 1;
            }
        }

        if !current_batch.is_empty() {
            let start_label = if batch_index == 0 { 0 } else { batch_index * BATCH_SIZE + 1 };
            let end_label = batch_index * BATCH_SIZE + current_batch.len();
            let filename = format!("jsonl/races/{}-{}.jsonl", start_label, end_label);
            let mut file = File::create(&filename)?;
            for doc in &current_batch {
                let json_data = to_string(doc)?;
                file.write_all(json_data.as_bytes())?;
                file.write_all(b"\n")?;
            }
            println!("Exported batch to {}", filename);
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok().unwrap();

    let conn_str = std::env::var("DB_CONNECTION_STRING").expect("Failed to read db connection string.");
    let mut opts = ClientOptions::parse(&conn_str).await?;
    opts.server_api = Some(ServerApi::builder().version(ServerApiVersion::V1).build());
    let client = Client::with_options(opts)?;

    // Set up dates: 2024-01-01 to 2024-06-30
    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let start_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let start_date_time = NaiveDateTime::new(start_date, start_time);

    let end_date = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
    let end_time = NaiveTime::from_hms_opt(23, 59, 59).unwrap();
    let end_date_time = NaiveDateTime::new(end_date, end_time);

    let range = RangeDateTime {
        start_date_time,
        end_date_time
    };

    let generator = RaceGenerator {
        date_time: TestDateTime::RangeDateTime(range),
        db_client: client,
    };

    generator.generate_races().await?;

    Ok(())
}