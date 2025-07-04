use std::{collections::HashMap, time::Duration};

use anyhow::{bail, Context, Result};
use chrono::{NaiveDate, TimeZone, Utc};
use log::{error, info};
use mongodb::bson::{self, Bson, Document};
use serde_json::json;

use crate::constants::BASE_GRAYHOUND_URL;

pub struct Scrapper {
    client: reqwest::Client,
}

impl Scrapper {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .context("Failed to build reqwest::Cient")?;

        Ok(Self { client })
    }

    pub async fn get_daily_races(
        &self,
        date: &NaiveDate,
    ) -> Result<Vec<Vec<HashMap<String, String>>>> {
        let url = format!(
            "{}/meeting/blocks.sd?r_date={}&view=meetings&blocks=header%2Clist",
            BASE_GRAYHOUND_URL, date
        );
        info!("Fetching list of races for date '{}' from {}", date, url);

        let response = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("Request error when fetching races for date {}", date))?;

        let data: serde_json::Value = response
            .json()
            .await
            .context("JSON decoding error for daily races")?;

        let mut races_ids: Vec<Vec<HashMap<String, String>>> = Vec::new();

        let meetings = data
            .get("list")
            .and_then(|v| v.get("items"))
            .and_then(|v| v.as_array());

        if meetings.is_none() {
            bail!("No meetings for date: {}", date);
        }

        for meeting in meetings.unwrap() {
            // For each meeting, get array "races"
            let races = meeting.get("races").and_then(|v| v.as_array());
            if races.is_none() {
                continue;
            }

            let mut meeting_race_ids: Vec<HashMap<String, String>> = Vec::new();
            for race in races.unwrap() {
                let race_date_time = race
                    .get("raceDate")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .split_whitespace()
                    .collect::<Vec<&str>>();

                if race_date_time.len() < 2 {
                    continue;
                }

                let race_date = race_date_time[0].to_string();
                let race_time = race_date_time[1].to_string();

                let race_id = race
                    .get("raceId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let distance = race
                    .get("distance")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let mut race_map = HashMap::new();
                race_map.insert("raceId".to_string(), race_id);
                race_map.insert("raceDate".to_string(), race_date);
                race_map.insert("raceTime".to_string(), race_time);
                race_map.insert("distance".to_string(), distance);

                meeting_race_ids.push(race_map);
            }

            if !meeting_race_ids.is_empty() {
                races_ids.push(meeting_race_ids);
            }
        }

        Ok(races_ids)
    }

    /// Retrieves all dog data for all race IDs on a given date.
    pub async fn get_all_dogs_data(&self, date: &NaiveDate) -> Result<Vec<bson::Document>> {
        let all_meetings_races = self.get_daily_races(date).await?;
        let mut all_dogs_data: Vec<bson::Document> = Vec::new();

        if all_meetings_races.is_empty() {
            bail!("No race IDs found for date {}", date);
        }

        for meeting_races in all_meetings_races {
            for race in meeting_races {
                let race_id_str = race.get("raceId").cloned().unwrap_or_default(); // String
                let race_time_str = race.get("raceTime").cloned().unwrap_or_default(); // String, "14:36"
                let distance_str = race.get("distance").cloned().unwrap_or_default(); // String, "277m"
                let race_date_str = race.get("raceDate").cloned().unwrap_or_default(); // String, "2025-06-02"

                // 1.1) "277m" -> 277i32
                let distance_i32: i32 = distance_str
                    .trim_end_matches('m')
                    .parse::<i32>()
                    .unwrap_or_else(|_| {
                        error!(
                            "Cannot parse distance '{}' as i32, default to 0",
                            distance_str
                        );
                        0
                    });

                // 1.2) race_id -> u64
                let race_id_u64: u64 = race_id_str.parse::<u64>().unwrap_or_else(|_| {
                    error!("Cannot parse raceId '{}' as u64, default to 0", race_id_str);
                    0
                });

                // 1.3) date + time -> bson::DateTime (ISODate)
                //     date: "2025-06-02", time: "14:36"
                let dt_utc = {
                    let naive_date = NaiveDate::parse_from_str(&race_date_str, "%Y-%m-%d")
                        .unwrap_or_else(|_| {
                            error!(
                                "Cannot parse raceDate '{}' as NaiveDate, default to today",
                                race_date_str
                            );
                            Utc::now().date_naive()
                        });
                    let parts: Vec<&str> = race_time_str.split(':').collect();
                    let (h, m) = if parts.len() == 2 {
                        (
                            parts[0].parse::<u32>().unwrap_or(0),
                            parts[1].parse::<u32>().unwrap_or(0),
                        )
                    } else {
                        error!(
                            "Cannot split raceTime '{}' into HH:MM, default to 00:00",
                            race_time_str
                        );
                        (0, 0)
                    };
                    let naive_dt = naive_date
                        .and_hms_opt(h, m, 0)
                        .unwrap_or_else(|| naive_date.and_hms_opt(0, 0, 0).unwrap());
                    let dt_utc = Utc.from_utc_datetime(&naive_dt);
                    bson::DateTime::from_millis(dt_utc.timestamp_millis())
                };

                let url = format!(
                    "{}/card/blocks.sd?race_id={}&blocks=card,form",
                    BASE_GRAYHOUND_URL, race_id_str
                );
                info!("Fetching dog data for race_id={} from {}", race_id_str, url);

                let resp = match self.client.get(&url).send().await {
                    Ok(res) => res,
                    Err(error) => {
                        error!("Request error for race_id={}: {:?}", race_id_str, error);
                        continue;
                    }
                };

                let data: serde_json::Value = match resp.json().await {
                    Ok(d) => d,
                    Err(error) => {
                        error!(
                            "JSON decoding error for race_id={}: {:?}",
                            race_id_str, error
                        );
                        continue;
                    }
                };

                let dogs_data = data
                    .get("card")
                    .and_then(|v| v.get("dogs"))
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_else(Vec::new);

                let dogs_form = data
                    .get("form")
                    .and_then(|v| v.get("dogs"))
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_else(Vec::new);

                let mut merged_dogs = dogs_data;
                for (i, form_entry) in dogs_form.into_iter().enumerate() {
                    if let Some(dog) = merged_dogs.get_mut(i) {
                        if dog.is_object() {
                            let dog_obj = dog.as_object_mut().unwrap();
                            dog_obj.insert("form".to_string(), form_entry);
                        }
                    }
                }

                let filtered_dogs = self.filter_dogs_data(merged_dogs);

                let converted_dogs: Vec<Document> = filtered_dogs
                    .into_iter()
                    .filter_map(|dog_val| self.convert_data_types(dog_val))
                    .collect();

                let mut doc = Document::new();
                doc.insert("distance", Bson::Int32(distance_i32));
                doc.insert("race_date_time", dt_utc);
                doc.insert("race_time", race_time_str);
                doc.insert("race_id", Bson::Int64(race_id_u64 as i64));
                doc.insert("createdAt", bson::DateTime::now());

                let bson_array_of_dogs = converted_dogs
                    .into_iter()
                    .map(Bson::Document)
                    .collect::<Vec<Bson>>();

                doc.insert("dogs", Bson::Array(bson_array_of_dogs));

                all_dogs_data.push(doc);
            }
        }

        Ok(all_dogs_data)
    }

    /// Removes unneeded fields from each dog's data record.
    fn filter_dogs_data(&self, dogs_data: Vec<serde_json::Value>) -> Vec<serde_json::Value> {
        let main_dog_keys_to_remove = vec![
            "dogId",
            "dogSex",
            "trainerName",
            "spotlightComment",
            "nonRunner",
            "reserved",
            "isVacant",
            "dateOfBirth",
            "bestTimeGrade",
            "bestTimeGradeDate",
            "sire",
            "dam",
            "dateOfSeason",
            "shortForm",
            "videoid",
            "clipId",
            "diffusionName",
            "birthMonYY",
            "wideYn",
            "handicapMetre",
            "shortTracName",
            "raceTitle",
            "trainerLocation",
            "typeCde",
            "dogColor",
        ];

        let form_entry_keys_to_remove = vec![
            "dogId",
            "videoid",
            "clipId",
            "diffusionName",
            "birthMonYY",
            "wideYn",
            "handicapMetre",
            "otherDogId",
            "otherDHandicapMetre",
            "shortTracName",
            "raceTitle",
            "trialFlag",
            "typeCde",
            "otherDogName",
            "otherDTxt",
            "raceGradeId",
            "raceTime",
            "resultDate",
            "resultsAvailable",
            "trackId",
            "trackName",
        ];

        let mut filtered = Vec::new();

        for dog_value in dogs_data {
            if let Some(mut dog_obj) = dog_value.as_object().cloned() {
                for key in &main_dog_keys_to_remove {
                    dog_obj.remove(*key);
                }

                if let Some(form_val) = dog_obj.get("form").cloned() {
                    if let Some(mut form_obj) = form_val.as_object().cloned() {
                        for key in &main_dog_keys_to_remove {
                            form_obj.remove(*key);
                        }

                        if let Some(forms_arr) = form_obj.get("forms").and_then(|v| v.as_array()) {
                            let mut new_forms = Vec::new();
                            for form_entry in forms_arr {
                                if let Some(mut entry_obj) = form_entry.as_object().cloned() {
                                    for k in &form_entry_keys_to_remove {
                                        entry_obj.remove(*k);
                                    }
                                    new_forms.push(json!(entry_obj));
                                } else {
                                    new_forms.push(form_entry.clone());
                                }
                            }
                            form_obj.insert("forms".to_string(), json!(new_forms));
                        }

                        dog_obj.insert("form".to_string(), json!(form_obj));
                    }
                }

                filtered.push(json!(dog_obj));
            } else {
                filtered.push(dog_value);
            }
        }

        info!("Filtered out unused fields from dogs_data.");
        filtered
    }

    fn convert_data_types(&self, dog_val: serde_json::Value) -> Option<Document> {
        let dog_map = dog_val.as_object()?;
        let mut doc = Document::new();

        // 1) trackId -> trackName
        if let Some(serde_json::Value::String(track_id_str)) = dog_map.get("trackId") {
            let track_name = self.convert_track_id(track_id_str);
            
            if track_name.eq(&"Limerick") || track_name.eq(&"Youghal") || track_name.eq(&"Clonmel") {
                return None;
            }
            
            doc.insert("trackName", track_name);
        }

        // 2) trapNum -> u32
        if let Some(serde_json::Value::String(trap_num_str)) = dog_map.get("trapNum") {
            if let Ok(tn) = trap_num_str.parse::<u32>() {
                doc.insert("trapNumber", Bson::Int32(tn as i32));
            } else {
                doc.insert("trapNumber", Bson::Null);
            }
        }

        // 3) dogName -> don't change (String)
        if let Some(serde_json::Value::String(name)) = dog_map.get("dogName") {
            doc.insert("dogName", name.clone());
        }

        // 4) forecast -> skip
        // 5) topSpeed -> skip
        // 6) forecastComment -> skip
        // 7) chanceOfWin -> skip

        // 9) form (object) → convert recursevly
        if let Some(serde_json::Value::Object(form_map)) = dog_map.get("form") {
            let mut form_doc = Document::new();

            // 9.a) trackId -> trackName
            if let Some(serde_json::Value::String(track_id_str)) = form_map.get("trackId") {
                let track_name = self.convert_track_id(track_id_str);
                form_doc.insert("trackName", track_name);
            }

            // 9.b) dogName -> String
            if let Some(serde_json::Value::String(name)) = form_map.get("dogName") {
                form_doc.insert("dogName", name);
            }

            // 9.c) forecast -> skip
            // 9.e) chanceOfWin -> skip

            // 9.f) forms → array of documents
            if let Some(serde_json::Value::Array(forms_arr)) = form_map.get("forms") {
                let mut bson_forms = Vec::new();
                for entry in forms_arr {
                    if let Some(entry_map) = entry.as_object() {
                        let mut entry_doc = Document::new();

                        // calcRTimes -> f32
                        if let Some(serde_json::Value::String(calc_r_str)) =
                            entry_map.get("calcRTimes")
                        {
                            if let Ok(val) = calc_r_str.parse::<f32>() {
                                entry_doc.insert("resultRunTime", Bson::Double(val as f64));
                            }
                        }

                        // weight -> f32
                        if let Some(serde_json::Value::String(weight_str)) = entry_map.get("weight")
                        {
                            if let Ok(val) = weight_str.parse::<f32>() {
                                entry_doc.insert("resultDogWeight", Bson::Double(val as f64));
                            }
                        }

                        // winnersTimeS -> f32
                        if let Some(serde_json::Value::String(win_s_str)) =
                            entry_map.get("winnersTimeS")
                        {
                            if let Ok(val) = win_s_str.parse::<f32>() {
                                entry_doc.insert("raceWinnersTime", Bson::Double(val as f64));
                            }
                        }

                        // distanceTitle: "277m" -> i32
                        if let Some(serde_json::Value::String(dist_t_str)) =
                            entry_map.get("distanceTitle")
                        {
                            let dist_i =
                                dist_t_str.trim_end_matches('m').parse::<i32>().unwrap_or(0);
                            entry_doc.insert("distance", Bson::Int32(dist_i));
                        }

                        // goingType: "-10" -> i32
                        if let Some(serde_json::Value::String(going_str)) =
                            entry_map.get("goingType")
                        {
                            if let Ok(val) = going_str.parse::<i32>() {
                                entry_doc.insert("goingType", Bson::Int32(val));
                            }
                        }

                        // oddsDesc -> skip

                        // rOutcomeId -> u32
                        if let Some(serde_json::Value::String(r_out_str)) =
                            entry_map.get("rOutcomeId")
                        {
                            if let Ok(val) = r_out_str.parse::<u32>() {
                                entry_doc.insert("resultPosition", Bson::Int32(val as i32));
                            }
                        }

                        // raceId -> u64
                        if let Some(serde_json::Value::Number(num)) = entry_map.get("raceId") {
                            if let Some(rid) = num.as_u64() {
                                entry_doc.insert("raceId", Bson::Int64(rid as i64));
                            }
                        }

                        // trap -> u32
                        if let Some(serde_json::Value::String(trap_str)) = entry_map.get("trap") {
                            if let Ok(val) = trap_str.parse::<u32>() {
                                entry_doc.insert("trapNumber", Bson::Int32(val as i32));
                            }
                        }

                        // sectionalTime -> Option<f32>
                        if let Some(serde_json::Value::String(sec_str)) =
                            entry_map.get("sectionalTime")
                        {
                            if let Ok(val) = sec_str.parse::<f32>() {
                                entry_doc.insert("sectionalTime", Bson::Double(val as f64));
                            }
                        }

                        // bndPos, by, closeUpCmnt, gradeCde → оставляем как есть (String), если нужно
                        if let Some(serde_json::Value::String(bp)) = entry_map.get("bndPos") {
                            entry_doc.insert("bndPos", bp);
                        }
                        if let Some(serde_json::Value::String(by)) = entry_map.get("by") {
                            entry_doc.insert("btnDistance", by);
                        }
                        if let Some(serde_json::Value::String(cuc)) = entry_map.get("closeUpCmnt") {
                            entry_doc.insert("raceComment", cuc);
                        }
                        if let Some(serde_json::Value::String(gc)) = entry_map.get("gradeCde") {
                            entry_doc.insert("raceClass", gc);
                        }

                        // Ading to the array
                        bson_forms.push(Bson::Document(entry_doc));
                    }
                }
                form_doc.insert("forms", Bson::Array(bson_forms));
            }

            doc.insert("form", Bson::Document(form_doc));
        }

        Some(doc)
    }

    fn convert_track_id(&self, track_id_str: &str) -> String {
        match track_id_str {
            "4" => "Monmore".to_string(),
            "5" => "Hove".to_string(),
            "6" => "Newcastle".to_string(),
            "7" => "Oxford".to_string(),
            "11" => "Romford".to_string(),
            "16" => "Yarmouth".to_string(),
            "21" => "Shelbourne Park".to_string(),
            "33" => "Nottingham".to_string(),
            "34" => "Sheffield".to_string(),
            "39" => "Swindon".to_string(),
            "40" => "Limerick".to_string(),
            "41" => "Clonmel".to_string(),
            "42" => "Cork".to_string(),
            "45" => "Dundalk".to_string(),
            "48" => "Enniscorthy".to_string(),
            "49" => "Galway".to_string(),
            "50" => "Kilkenny".to_string(),
            "51" => "Lifford".to_string(),
            "53" => "Mullingar".to_string(),
            "55" => "Newbridge".to_string(),
            "56" => "Thurles".to_string(),
            "57" => "Tralee".to_string(),
            "58" => "Waterford".to_string(),
            "59" => "Youghal".to_string(),
            "61" => "Sunderland".to_string(),
            "62" => "Perry Bar".to_string(),
            "66" => "Doncaster".to_string(),
            "69" => "Harlow".to_string(),
            "70" => "Central Park".to_string(),
            "73" => "Valley".to_string(),
            "76" => "Kingslay".to_string(),
            "86" => "Star Pelaw".to_string(),
            "88" => "Drumbo Park".to_string(),
            "98" => "Towcester".to_string(),
            _ => unreachable!(),
        }
    }
}
