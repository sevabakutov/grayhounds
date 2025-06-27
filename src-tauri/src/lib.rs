pub mod commands;
pub mod constants;
pub mod models;
pub mod predictor;
pub mod scrapper;
pub mod client;
pub mod utils;
pub mod tester;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{
    NaiveDate, 
    NaiveTime, 
    TimeZone, 
    Utc
};
use futures::TryStreamExt;
use mongodb::{
    bson::{
        doc, 
        from_document, 
        DateTime, 
        Document
    }, 
    Collection
};
use crate::models::DogRaceInfo;


#[async_trait]
pub trait DogInfoRepo: Send + Sync {
    /// All recorrds about participants
    async fn race_participants(
        &self,
        date: NaiveDate,
        time: NaiveTime,
    ) -> Result<Vec<DogRaceInfo>>;

    /// Dog record
    async fn dog_record(
        &self,
        date: NaiveDate,
        time: NaiveTime,
        distance: u32,
        dog_name: &str,
    ) -> Result<Option<DogRaceInfo>>;
}

pub struct MongoDogInfoRepo {
    col: Collection<Document>,
}

impl MongoDogInfoRepo {
    pub fn new(col: Collection<Document>) -> Self {
       Self { col }
    }
}

#[async_trait::async_trait]
impl DogInfoRepo for MongoDogInfoRepo {
    async fn race_participants(
        &self,
        date: NaiveDate,
        time: NaiveTime,
    ) -> Result<Vec<DogRaceInfo>> {
        let dt_utc = Utc.from_utc_datetime(&date.and_time(time));
        let date_time = DateTime::from_millis(dt_utc.timestamp_millis());
        let filter = doc! { "raceDateTime": date_time };

        println!("{:?}", filter.clone());

        let cur = self.col.find(filter).await?;
        let vec = cur
            .try_collect::<Vec<Document>>()
            .await?
            .into_iter()
            .filter_map(|d| from_document::<DogRaceInfo>(d).ok())
            .collect();

        Ok(vec)
    }

    async fn dog_record(
        &self,
        date: NaiveDate,
        time: NaiveTime,
        distance: u32,
        dog_name: &str,
    ) -> Result<Option<DogRaceInfo>> {
        let dt_utc = Utc.from_utc_datetime(&date.and_time(time));
        let bson_date = DateTime::from_millis(dt_utc.timestamp_millis());
        let filter = doc! {
            "raceDateTime": bson_date,
            "distance": distance,
            "dogName": dog_name,
        };
        let doc = self.col.find_one(filter).await?;
        Ok(doc.and_then(|d| from_document(d).ok()))
    }
}