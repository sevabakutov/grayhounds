use std::{collections::HashMap, fmt};

use async_openai::types::ReasoningEffort;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RangeTime {
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RangeDateTime {
    pub start_date_time: NaiveDateTime,
    pub end_date_time: NaiveDateTime
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Time {
    FixedTime(NaiveTime),
    RangeTime(RangeTime),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum TestDateTime {
    FixedDateTime(NaiveDateTime),
    RangeDateTime(RangeDateTime)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Model {
    O3Mini,
    O4Mini,
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Model::O3Mini => "o3-mini",
            Model::O4Mini => "o4-mini",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct PredictInput {
    pub time: Time,
    pub distances: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DogRaceInfo {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub dog_id: u32,
    pub dog_name: String,
    pub race_class: Option<String>,
    pub race_going: Option<i32>,
    pub race_handicap: Option<bool>,
    pub race_id: u64,
    pub race_type: Option<String>,
    pub result_adjusted_time: Option<f32>,
    pub result_btn_distance: Option<String>,
    pub result_comment: Option<String>,
    pub result_dog_weight: Option<f32>,
    pub result_market_cnt: Option<u32>,
    pub result_market_pos: Option<u32>,
    pub result_position: u32,
    pub result_run_time: Option<f32>,
    pub result_sectional_time: Option<f32>,
    pub track_name: Option<String>,
    pub trap_handicap: Option<String>,
    pub trap_number: Option<u32>,
    pub race_date_time: DateTime,
    pub distance: u32,
    pub bf_odds_1_minute: f64,
}

impl Default for DogRaceInfo {
    fn default() -> Self {
        DogRaceInfo {
            id: ObjectId::default(),
            dog_id: 0,
            dog_name: String::default(),
            race_class: None,
            race_going: None,
            race_handicap: None,
            race_id: 0,
            race_type: None,
            result_adjusted_time: None,
            result_btn_distance: None,
            result_comment: None,
            result_dog_weight: None,
            result_market_cnt: None,
            result_market_pos: None,
            result_position: 0,
            result_run_time: None,
            result_sectional_time: None,
            track_name: None,
            trap_handicap: None,
            trap_number: None,
            race_date_time: DateTime::now(),
            distance: 0,
            bf_odds_1_minute: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PosOdds {
    pub real_position: u32,
    pub odds: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RaceCount {
    total_races: usize,
    races_tracked: usize,
}

impl RaceCount {
    pub fn new(
        total_races: usize,
        races_tracked: usize
    ) -> Self {
        Self {
            total_races,
            races_tracked
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OddsRange {
    pub low: f64,
    pub high: f64
}

impl OddsRange {
    pub fn new(low: f64, high: f64) -> Self {
        Self {
            low,
            high
        }
    }
}

// pub struct Metrics {
//     pub roi: f64,
//     pub profit_factor: f64,
//     pub expectancy: f64,
// }

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionInfo {
    bad_hit_4_pos: i32,
    bad_hit_5_pos: i32,
    bad_hit_6_pos: i32
}

impl PositionInfo {
    pub fn new(
        bad_hit_4_pos: i32,
        bad_hit_5_pos: i32,
        bad_hit_6_pos: i32,
    ) -> Self {
        Self {
            bad_hit_4_pos,
            bad_hit_5_pos,
            bad_hit_6_pos
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestErrors {
    total_empty_content: usize,
    total_race_parse_error: usize,
    total_mongo_db_error: usize,
}

impl TestErrors {
    pub fn new(
        total_empty_content: usize,
        total_race_parse_error: usize,
        total_mongo_db_error: usize,
    ) -> Self {
        Self {
            total_empty_content,
            total_race_parse_error,
            total_mongo_db_error
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkipInfo {
    skipped_races_lt5: i32,
    skipped_races_gt6: i32,
    skipped_odds_range: i32,
    skipped_favorite: i32
}

impl SkipInfo {
    pub fn new(
        skipped_races_lt5: i32,
        skipped_races_gt6: i32,
        skipped_odds_range: i32,
        skipped_favorite: i32
    ) -> Self {
        Self {
            skipped_races_lt5,
            skipped_races_gt6,
            skipped_odds_range,
            skipped_favorite
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    initial_balance: f64,
    final_balance: f64
}

impl Balance {
    pub fn new(
        initial_balance: f64,
        final_balance: f64
    ) -> Self {
        Self {
            initial_balance,
            final_balance
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResultsMeta {
    race_count: RaceCount,
    odds_range: OddsRange, 
    position_info: PositionInfo,
    skip_info: SkipInfo,
    balance: Balance,
    errors: TestErrors,
    initial_stake: f64,
    percentage: f64,
}

impl TestResultsMeta {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        race_count: RaceCount,
        odds_range: OddsRange,
        position_info: PositionInfo,
        skip_info: SkipInfo,
        balance: Balance,
        errors: TestErrors,
        initial_stake: f64,
        percentage: f64
    ) -> Self {
        Self {
            race_count,
            odds_range,
            position_info,
            skip_info,
            balance,
            errors,
            initial_stake,
            percentage
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResultsRaceMeta {
    date: NaiveDate,
    distance: u32,
    grade: Option<String>,
    time: NaiveTime,
    track: String,
    current_balance: f64,
    profit: f64
}

impl TestResultsRaceMeta {
    pub fn new(
        date: NaiveDate,
        distance: u32,
        grade: Option<String>,
        time: NaiveTime,
        track: String,
        current_balance: f64,
        profit: f64,
    ) -> Self {
        Self {
            date,
            distance,
            grade,
            time,
            track,
            current_balance,
            profit
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResultsRealResults {
    rank: u8,
    betfair_odds: f32
}

impl TestResultsRealResults {
    pub fn new(rank: u8, betfair_odds: f32) -> Self {
        Self {
            rank,
            betfair_odds
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResultsDog {
    dog_name: String,
    model_prediction: Prediction,
    real_results: TestResultsRealResults,
}

impl TestResultsDog {
    pub fn new(
        dog_name: String,
        model_prediction: Prediction,
        real_results: TestResultsRealResults,
    ) -> Self {
        Self {
            dog_name,
            model_prediction,
            real_results,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TestResultsRace {
    #[serde(rename = "raceId")]
    race_id: u64,
    meta: TestResultsRaceMeta,
    dogs: Vec<TestResultsDog>,
    summary: String
}

impl TestResultsRace {
    pub fn new(
        race_id: u64,
        meta: TestResultsRaceMeta,
        dogs: Vec<TestResultsDog>,
        summary: String
    ) -> Self {
        Self {
            race_id,
            meta,
            dogs,
            summary
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TestResults {
    meta: TestResultsMeta,
    races: Vec<TestResultsRace>
}

impl TestResults {
    pub fn new(
        meta: TestResultsMeta,
        races: Vec<TestResultsRace>
    ) -> Self {
        Self {
            meta,
            races
        }        
    }
}

pub struct RequestsInfo {
    pub requests: Vec<HashMap<String, serde_json::Value>>,
    pub total_races: usize
}

#[derive(Debug, Deserialize)]
pub struct AddInstructionInput {
    pub name: String,
    pub content: String
}

#[derive(Debug, Deserialize)]
pub struct LoadSettingsInput {
    pub model: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoadSettingsOutput {
    pub model: String,
    pub max_completion_tokens: Option<u32>,
    pub frequency_penalty: Option<f32>,
    pub logprobs: Option<bool>,
    pub presence_penalty: Option<f32>,
    pub reasoning_effort: Option<String>,
    pub seed: Option<f64>,
    pub store: Option<bool>,
    pub temperature: Option<f32>,
    pub max_races: usize,
    pub races_per_request: usize,
    pub instruction_name: String
}

impl Default for LoadSettingsOutput {
    fn default() -> Self {
        Self {
            model: "o3-mini".to_string(),
            max_completion_tokens: None,
            frequency_penalty: None,
            logprobs: None,
            presence_penalty: None,
            reasoning_effort: None,
            seed: None,
            store: None,
            temperature: None,
            max_races: 50,
            races_per_request: 1,
            instruction_name: String::new()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveSettingsInput {
    pub model: String,
    pub selected: bool,
    pub max_completion_tokens: Option<u32>,
    pub frequency_penalty: Option<f32>,
    pub logprobs: Option<bool>,
    pub presence_penalty: Option<f32>,
    pub reasoning_effort: Option<String>,
    pub seed: Option<f64>,
    pub store: Option<bool>,
    pub temperature: Option<f32>,
    pub max_races: usize,
    pub races_per_request: usize,
    pub instruction_name: String
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub model: Model,
    pub instruction_name: String,
    pub max_completion_tokens: Option<u32>,
    pub frequency_penalty: Option<f32>,
    pub logprobs: Option<bool>,
    pub presence_penalty: Option<f32>,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub seed: Option<i64>,
    pub store: Option<bool>,
    pub temperature: Option<f32>,
    pub max_races: usize,
    pub races_per_request: usize,
    pub selected: bool,
}

#[derive(Debug, Deserialize)]
pub struct InstructionDoc {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Meta {
    // #[serde(skip)]
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub distance: u32,
    pub track: String,
    pub grade: Option<String>
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Prediction {
    pub name: String,
    pub raw_score: f32,
    pub percentage: f32,
    pub rank: u8,
    pub comment: Option<String>
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PredictResponse {
    pub meta: Meta,
    pub predictions: Vec<Prediction>,
    pub summary: Option<String>
}

impl PredictResponse {
    pub fn sort_predictions(&mut self) {
        self.predictions.sort_by(|a, b| a.rank.cmp(&b.rank));
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimeRange {
    pub start_time: String,
    pub end_time: Option<String>
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoadPredictionsInput {
    #[serde(rename = "timeRange")]
    pub time_range: TimeRange
}