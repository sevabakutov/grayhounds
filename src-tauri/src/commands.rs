use anyhow::Result;
use chrono::{TimeZone, Utc};
use futures::TryStreamExt;
use mongodb::{
    bson::{
        self, doc, to_document, DateTime, Document
    }, 
    Client
};
use tauri::State;
use crate::{
    constants::{
        INSTRUCTION_COLLECTION, PREDICTIONS_COLLECTION, RACES_COLLECTION, SETTINGS_COLLECTION, TIME_RANGES_COLLECTION
    }, 
    models::{
        AddInstructionInput, LoadPredictionsInput, LoadSettingsInput, LoadSettingsOutput, OddsRange, PredictInput, PredictResponse, SaveSettingsInput, Settings, TestDateTime, TestResults, Time, TimeRange
    }, 
    predictor::Predictor, 
    tester::Tester
};

#[tauri::command]
pub async fn load_settings(
    input: LoadSettingsInput,
    client_state: State<'_, Client>
) -> Result<LoadSettingsOutput, String> {
    let db = client_state
        .default_database()
        .ok_or("No default DB")?;
    let collection = db
        .collection::<LoadSettingsOutput>(SETTINGS_COLLECTION);

    let filter = doc! { "model": &input.model };

    let found = collection
        .find_one(filter)
        .await
        .map_err(|e| format!("Find error: {}", e))?
        .unwrap_or_default();

    Ok(found)
}

#[tauri::command]
pub async fn save_settings(
    input: SaveSettingsInput,
    client_state: State<'_, Client>
) -> Result<String, String> {
    let db = client_state
        .default_database()
        .ok_or("No default DB")?;
    let collection = db.collection::<Document>(SETTINGS_COLLECTION);

    collection.update_many(
        doc! { "selected": true },
        doc! { "$set": { "selected": false } },
    )
    .await
    .map_err(|e| format!("Clear selected error: {}", e))?;

    let filter = doc! { "model": &input.model };

    let mut update_doc = to_document(&input)
        .map_err(|e| format!("Serialization error: {}", e))?;
    update_doc.remove("model");

    let update = doc! { "$set": update_doc };

    collection
        .update_one(filter, update)
        .await
        .map_err(|e| format!("Update error: {}", e))?;

    Ok("Settings were successfully saved!".to_string())
}

#[tauri::command]
pub async fn add_instruction(
    input: AddInstructionInput,
    client_state: State<'_, Client>,
) -> Result<String, String> {
    println!("add_instruction called with name: {}, content length: {}", input.name, input.content.len());
    
    let collection = client_state
        .default_database()
        .ok_or("No default DB")?
        .collection::<Document>(INSTRUCTION_COLLECTION);

    collection
        .insert_one(doc! {
            "name": &input.name,
            "content": &input.content,
            "created_at": DateTime::now()
        })
        .await
        .map_err(|e| format!("Insert error: {}", e))?;

    println!("Instruction '{}' added successfully", input.name);
    Ok("Added instruction!".into())
}

#[tauri::command]
pub async fn read_instruction_names(
    client_state: State<'_, Client>,
) -> Result<Vec<String>, String> {
    let collection = client_state
        .default_database()
        .ok_or("No default DB")?
        .collection::<Document>(INSTRUCTION_COLLECTION);

    let mut cursor = collection
        .find(doc! {})
        .await
        .map_err(|e| format!("Find error: {}", e))?;

    let mut names = Vec::new();
    while let Some(doc) = cursor.try_next().await.map_err(|e| format!("Cursor error: {}", e))? {
        if let Ok(name) = doc.get_str("name") {
            names.push(name.to_string());
        }
    }

    Ok(names)
}

#[tauri::command]
pub async fn load_time_ranges(
    client_state: State<'_, Client>,
) -> Result<Vec<TimeRange>, String> {
    let db = client_state
        .default_database()
        .ok_or("No default database")?;

    let cursor = db
        .collection::<TimeRange>(TIME_RANGES_COLLECTION)
        .find(doc! {})
        .await
        .map_err(|e| format!("{e}"))?;

    cursor
        .try_collect::<Vec<TimeRange>>()
        .await
        .map_err(|e| format!("{e}"))
}

#[tauri::command]
pub async fn load_predictions(
    client_state: State<'_, Client>,
    input: LoadPredictionsInput,
) -> Result<Vec<PredictResponse>, String> {
    let db = client_state
        .default_database()
        .ok_or_else(|| "Not default database".to_string())?;

    let filter = match &input.time_range.end_time {
        Some(end) => doc! { "meta.time": { "$gte": &input.time_range.start_time, "$lte": end } },
        None               => doc! { "meta.time": &input.time_range.start_time },
    };

    let mut predictions: Vec<PredictResponse> = db.collection::<PredictResponse>(PREDICTIONS_COLLECTION)
        .find(filter)
        .await
        .map_err(|e| e.to_string())?
        .try_collect()
        .await
        .map_err(|e| e.to_string())?;
    predictions.sort_unstable_by(|a, b| a.meta.time.cmp(&b.meta.time));

    Ok(predictions)
}

#[tauri::command]
pub async fn run_predict(
    client_state: State<'_, Client>,
    input: PredictInput,
) -> Result<Vec<PredictResponse>, String> {
    let db_client = client_state.inner().clone();
    let config = db_client
        .default_database()
        .ok_or("No default database")?
        .collection::<Settings>(SETTINGS_COLLECTION)
        .find_one(doc! { "selected": true })
        .await
        .map_err(|e| e.to_string())?
        .ok_or("No settings for selected model")?;

    let predictor = Predictor::new(config, db_client.clone(), input.clone()).await;
    
    let mut result = predictor.run()
        .await
        .map_err(|e| e.to_string())?;
    result.sort_unstable_by(|a, b| a.meta.time.cmp(&b.meta.time));
    
    Ok(result)
}

#[tauri::command]
pub async fn run_test(
    client_state: State<'_, Client>,
    date_time: TestDateTime,
    distances: Vec<i32>,
    initial_stake: f64,
    initial_balance: f64,
    is_favorite_protected: bool,
    odds_range: OddsRange
) -> Result<TestResults, String> {
    let db_client = client_state.inner().clone();
    let config = db_client
        .default_database()
        .ok_or("No default database")?
        .collection::<Settings>(SETTINGS_COLLECTION)
        .find_one(doc! { "selected": true })
        .await
        .map_err(|e| e.to_string())?
        .ok_or("No settings for selected model")?;

    let tester = Tester::new(config, db_client, date_time, distances);
    
    let result = tester
        .run(initial_balance, initial_stake, odds_range, is_favorite_protected)
        .await
        .map_err(|err| err.to_string())?;

    Ok(result)
}

#[tauri::command]
pub async fn copy_predict_request(
    client_state: State<'_, Client>,
    input: PredictInput,
) -> Result<String, String> {
    let db = client_state
        .default_database()
        .ok_or_else(|| "No default database".to_string())?;
    
    let today = Utc::now().date_naive();
    let distances = &input.distances;
    
    let filter = match &input.time {
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

    let races: Vec<Document> = db
        .collection::<Document>(RACES_COLLECTION)
        .aggregate(pipeline)
        .await
        .map_err(|e| e.to_string())?
        .try_collect()
        .await
        .map_err(|e| e.to_string())?;

    let json = serde_json::to_string_pretty(&races).map_err(|e| e.to_string())?;
    Ok(json)
}