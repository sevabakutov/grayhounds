use anyhow::anyhow;
use chrono::{Duration, NaiveDate, NaiveDateTime};
use mongodb::{
    bson::{doc, DateTime as BsonDateTime, Document},
    options::ClientOptions,
    Client as MongoClient,
};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashSet;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1) Диапазон дат
    let mut current_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2025, 1, 20).unwrap();

    // HTTP-клиент
    let http = Client::new();

    // MongoDB
    let mut opts = ClientOptions::parse("mongodb+srv://qwertyrom:qwertyrom@dogs.wkobavq.mongodb.net/main?retryWrites=true&w=majority&appName=DOGS").await?;
    opts.app_name = Some("GreyhoundScraper".to_string());
    let mongo = MongoClient::with_options(opts)?;
    let db = mongo.default_database().unwrap();
    let coll = db.collection::<Document>("dog_race_info_3");

    let mut seen: HashSet<(String, String)> = HashSet::new();

    println!("Conected!");

    while current_date <= end_date {
        let date_str = current_date.format("%Y-%m-%d").to_string();

        println!("Current date: {}", date_str.as_str());

        // Сначала получаем список meetings
        let url = format!(
            "https://greyhoundbet.racingpost.com/results/blocks.sd?blocks=header%2Cmeetings&r_date={}",
            date_str
        );
        let txt = http.get(&url).send().await?.text().await?;
        let root: Value =
            serde_json::from_str(&txt).map_err(|e| anyhow!("Invalid JSON from {}: {}", url, e))?;

        if let Some(tracks) = root.pointer("/meetings/tracks").and_then(Value::as_object) {
            for (_track_key, track_val) in tracks {
                if let Some(meetings) = track_val.get("races").and_then(Value::as_array) {
                    for meeting in meetings {
                        // track_id как строка
                        let track_id = meeting
                            .get("track_id")
                            .and_then(Value::as_str)
                            .ok_or_else(|| anyhow!("Missing track_id in meeting {:?}", meeting))?;

                        if let Some(races_ids) = meeting.get("racesIds").and_then(Value::as_array) {
                            for race_id_val in races_ids {
                                let race_id = match race_id_val.as_str() {
                                    Some(id) => id,
                                    None => {
                                        println!("Warning: Missing race_id, skipping");
                                        continue;
                                    }
                                };

                                let url = format!(
                                    "https://greyhoundbet.racingpost.com/results/blocks.sd?race_id={race_id}&track_id={track_id}&r_date={date}&blocks=meetingHeader%2Cresults-meeting-pager%2Clist",
                                    race_id = race_id,
                                    track_id = track_id,
                                    date = date_str
                                );
                                let text = match http.get(&url).send().await {
                                    Ok(response) => match response.text().await {
                                        Ok(t) => t,
                                        Err(err) => {
                                            println!("Warning: Failed to get response text for race {}: {}, skipping", race_id, err);
                                            continue;
                                        }
                                    },
                                    Err(err) => {
                                        println!("Warning: Failed to fetch race data for {}: {}, skipping", race_id, err);
                                        continue;
                                    }
                                };
                                let val = match serde_json::from_str::<Value>(&text) {
                                    Ok(val) => val,
                                    Err(err) => {
                                        println!("Error: {err}");
                                        continue;
                                    }
                                };
                                let results_opt = val
                                    .get("list")
                                    .and_then(|l| l.get("track"))
                                    .and_then(|t| t.get("results"))
                                    .and_then(|r| r.as_object());

                                if let Some(results) = results_opt {
                                    for (race_id, dogs) in results {
                                        let dogs = match dogs.as_array() {
                                            Some(arr) => arr,
                                            None => {
                                                println!("Warning: Dogs data is not an array for race {}, skipping", race_id);
                                                continue;
                                            }
                                        };
                                        for dog_result in dogs {
                                            let dog_id = match dog_result
                                                .get("dogId")
                                                .and_then(Value::as_str)
                                            {
                                                Some(id) => id,
                                                None => {
                                                    println!("Warning: Missing dogId in race {}, skipping", race_id);
                                                    continue;
                                                }
                                            };
                                            let detail_url = format!(
                                                "https://greyhoundbet.racingpost.com/results/blocks.sd\
                                                ?race_id={race}&track_id={track}&r_date={date}\
                                                &dog_id={dog_id}&blocks=results-dog-header%2Cresults-dog-details",
                                                race = race_id,
                                                track = track_id,
                                                date = date_str,
                                                dog_id = dog_id
                                            );

                                            let dtxt = match http.get(&detail_url).send().await {
                                                Ok(response) => match response.text().await {
                                                    Ok(t) => t,
                                                    Err(err) => {
                                                        println!("Warning: Failed to get detail response text for dog {}: {}, skipping", dog_id, err);
                                                        continue;
                                                    }
                                                },
                                                Err(err) => {
                                                    println!("Warning: Failed to fetch dog details for {}: {}, skipping", dog_id, err);
                                                    continue;
                                                }
                                            };
                                            let droot = match serde_json::from_str::<Value>(&dtxt) {
                                                Ok(val) => val,
                                                Err(err) => {
                                                    println!("Error: {err}");
                                                    continue;
                                                }
                                            };

                                            // println!("Result: {:?}", droot);

                                            // Извлекаем dogInfo (одно) и forms (массив)
                                            let dog_info = &droot["results-dog-details"]["dogInfo"];
                                            let forms = match droot["results-dog-details"]["forms"]
                                                .as_array()
                                            {
                                                Some(arr) => arr,
                                                None => {
                                                    println!("Warning: Forms data is not an array for dog {}, skipping", dog_id);
                                                    continue;
                                                }
                                            };

                                            let dog_name = match dog_info
                                                .get("dogName")
                                                .and_then(Value::as_str)
                                            {
                                                Some(name) => name,
                                                None => {
                                                    println!("Warning: Missing dog name for dog {}, skipping", dog_id);
                                                    continue;
                                                }
                                            };

                                            let dog_id_str = match dog_info
                                                .get("dogId")
                                                .and_then(Value::as_str)
                                            {
                                                Some(id) => id,
                                                None => {
                                                    println!("Warning: Missing dogId in dog info, skipping");
                                                    continue;
                                                }
                                            };

                                            let dog_id = match dog_id_str.parse::<i64>() {
                                                Ok(val) => val,
                                                Err(err) => {
                                                    println!("{err}");
                                                    continue;
                                                }
                                            };

                                            let key = (race_id.to_string(), dog_name.to_string());
                                            if seen.contains(&key) {
                                                continue;
                                            }
                                            seen.insert(key);

                                            println!("Dog name: {}", dog_name);
                                            for form in forms {
                                                // Parse race date to filter by date range
                                                let race_time_str = match form
                                                    .get("rFormDatetime")
                                                    .and_then(Value::as_str)
                                                {
                                                    Some(s) if !s.is_empty() => s,
                                                    _ => {
                                                        println!("Warning: Missing race datetime for dog {}, skipping form", dog_name);
                                                        continue;
                                                    }
                                                };
                                                let race_dt = match NaiveDateTime::parse_from_str(
                                                    race_time_str,
                                                    "%Y-%m-%d %H:%M",
                                                ) {
                                                    Ok(dt) => dt,
                                                    Err(e) => {
                                                        println!("Warning: Bad date format '{}' for dog {}: {}, skipping form", race_time_str, dog_name, e);
                                                        continue;
                                                    }
                                                };
                                                let race_date = race_dt.date();

                                                if race_date
                                                    < NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
                                                    || race_date
                                                        > NaiveDate::from_ymd_opt(2024, 6, 30)
                                                            .unwrap()
                                                {
                                                    continue;
                                                }

                                                let race_bson_dt = BsonDateTime::from_millis(
                                                    race_dt.and_utc().timestamp_millis(),
                                                );

                                                // Extract raceId from form before converting
                                                let race_inst_id_str = match form
                                                    .get("rInstId")
                                                    .and_then(Value::as_str)
                                                {
                                                    Some(val) => val,
                                                    None => {
                                                        println!("Warning: Missing or invalid rInstId for dog {} at {}, skipping", dog_name, race_time_str);
                                                        continue;
                                                    }
                                                };

                                                let race_id = match race_inst_id_str.parse::<i64>()
                                                {
                                                    Ok(val) => val,
                                                    Err(err) => {
                                                        println!("Warning: Failed to parse rInstId '{}' for dog {} at {}: {}, skipping", race_inst_id_str, dog_name, race_time_str, err);
                                                        continue;
                                                    }
                                                };

                                                // Convert form JSON to BSON Document and add dogName
                                                let mut doc_to_insert = mongodb::bson::to_document(
                                                    &form,
                                                )
                                                .map_err(|e| {
                                                    anyhow!("Failed to convert form to BSON: {}", e)
                                                })?;

                                                // Remove rInstId and insert raceId as integer
                                                doc_to_insert.remove("rInstId");
                                                doc_to_insert.insert("raceId", race_id);
                                                // Add dogName and parsed raceDateTime
                                                doc_to_insert.insert("dogName", dog_name);
                                                doc_to_insert.insert("raceDateTime", race_bson_dt);
                                                doc_to_insert.insert("dogId", dog_id);

                                                // Insert new document, skip if duplicate exists (enforced by unique index)
                                                match coll.insert_one(&doc_to_insert).await {
                                                    Ok(_) => {
                                                        // Successfully inserted
                                                    }
                                                    Err(e) => {
                                                        // Check if it's a duplicate key error (code 11000)
                                                        if e.to_string().contains("E11000") || e.to_string().contains("duplicate key") {
                                                            // Document already exists, skip silently
                                                            continue;
                                                        }
                                                        // Re-throw other errors
                                                        return Err(e.into());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        current_date += Duration::days(1);
    }

    Ok(())
}
