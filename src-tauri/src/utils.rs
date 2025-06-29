use std::collections::HashMap;
use anyhow::{
    bail, 
    Context, 
    Result
};
use async_openai::types::{
    CreateChatCompletionResponse, 
    ResponseFormatJsonSchema
};
// use chrono::{
//     NaiveDateTime, 
//     // TimeZone, 
//     // Utc
// };
use mongodb::{
    bson::{
        doc, 
        // DateTime, 
        Document
    }, 
    Database
};
use serde_json::{
    json, 
    Value
};

use crate::{
    constants::{
        BETFAIR_PERCENTAGE, 
        INSTRUCTION_COLLECTION
    }, 
    models::{
        Balance, 
        InstructionDoc, 
        OddsRange, 
        PosOdds, 
        PositionInfo, 
        PredictResponse, 
        RaceCount, 
        Settings, 
        SkipInfo, 
        TestErrors, 
        TestResultsDog, 
        TestResultsMeta, 
        TestResultsRace, 
        TestResultsRaceMeta, 
        TestResultsRealResults
    }, 
    DogInfoRepo
};

pub async fn build_requests(
    races: Vec<Document>,
    database: Database,
    config: Settings
) -> Result<Vec<HashMap<String, Value>>> {
    let instruction = database
        .collection::<InstructionDoc>(INSTRUCTION_COLLECTION)
        .find_one(doc! { "name": config.instruction_name.as_str() })
        .await?
        .with_context(|| format!("Not instruction with such name: {}", config.instruction_name.as_str()))?;

    let mut requests = Vec::new();
    for chunk in races.chunks(config.races_per_request) {
        let meta = json!({ "model": config.model });
        let mut map = HashMap::new();

        map.insert("meta".to_string(), meta);
        
        let system = json!({ "role": "system", "content": instruction.content.as_str() });
        let user = json!({ "role": "user", "content": json!({ "races": chunk }).to_string()});
        
        map.insert("messages".to_string(), json!([ system, user ]));
        
        requests.push(map);
    }

    Ok(requests)
}

pub fn get_response_format_json_schema() -> ResponseFormatJsonSchema {
    let description = None;
    let name = "PredictionResponse".to_string();
    let strict = Some(true);
    let schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "PredictionResponse",
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "meta": {
                "type": "object",
                "description": "Метаинформация о гонке",
                "additionalProperties": false,
                "properties": {
                    "date": {
                        "type": ["string"],
                        "description": "Дата гонки (YYYY-MM-DD). Я передаю ее в поле raceDateTime, будь внимателен!! Это очень важно!"
                    },
                    "time": {
                        "type": ["string"],
                        "description": "Время гонки (HH:MM:SS)"
                    },
                    "distance": {
                        "type": "integer",
                        "description": "Дистанция гонки (например, 480)"
                    },
                    "track": {
                        "type": "string",
                        "description": "Название трека. Очень важно! Выдавай всегда"
                    },
                    "grade": {
                        "type": ["string"],
                        "description": "Класс гонки (A3, A4, D3 и т.д.)"
                    }
                },
                "required": ["date", "time", "distance", "track", "grade"]
            },
            "predictions": {
                "type": "array",
                "description": "Рейтинг собак по шансам на победу",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "name": { "type": "string", "description": "Имя собаки" },
                        "rawScore": { "type": "number", "description": "Суммарный Raw Score" },
                        "percentage": { "type": "number", "min": 0, "max": 100, "description": "Шанс победы в процентах" },
                        "rank": { "type": "integer", "description": "Позиция в прогнозе (1 — фаворит)" },
                        "comment": { "type": ["string", "null"], "description": "Краткий комментарий по результату. Выдавай его всегда" }
                    },
                    "required": ["name", "rawScore", "percentage", "rank", "comment"]
                }
            },
            "summary": {
                "type": ["string", "null"],
                "description": "Краткое заключение по прогнозу. Выдавай его всегда!"
            }
        },
        "required": ["meta", "predictions", "summary"]
    });

    ResponseFormatJsonSchema {
        description,
        name,
        schema: Some(schema),
        strict,
    }
}

pub async fn process_test_results<R: DogInfoRepo>(
    responses: Vec<CreateChatCompletionResponse>,
    repo: &R,
    total_races: usize,
    initial_balance: f64,
    initial_stake: f64,
    odds_range: OddsRange,
    is_favorite_protected: bool,
) -> Result<(TestResultsMeta, Vec<TestResultsRace>)> {
    if responses.is_empty() {
        bail!("Пустой ответ от LLM модели, выход из функции. Выход изз функции тестирования.");
    }
    if initial_balance <= 2.0 || !initial_balance.is_normal() {
        bail!("Неверный стартовый баланс: {initial_balance}. Выход из функции тестирования.");
    }
    if initial_stake <= 0.0 || !initial_stake.is_normal() {
        bail!("Неверный стартовая, неизменяемая ставка: {initial_stake}. Выход из функции.");
    }
    if odds_range.low <= 0.0
        || !odds_range.low.is_normal()
        || !odds_range.high.is_normal()
        || odds_range.high <= 0.0
        || odds_range.low > odds_range.high
    {
        bail!(
            "Неверный диапозон коэффициентов: low: {}, high: {}. Выход из функции.",
            odds_range.low,
            odds_range.high
        );
    }

    let mut tracked_races: usize = 0;
    let mut total_empty_content = 0;
    let mut total_race_parse_error = 0;
    let mut total_mongo_db_error = 0;
    let mut bad_hit_4_pos = 0;
    let mut bad_hit_5_pos = 0;
    let mut bad_hit_6_pos = 0;
    let mut current_balance = initial_balance;
    let mut skipped_races_lt5 = 0;
    let mut skipped_races_gt6 = 0;
    let mut skipped_odds_range = 0;
    let mut skipped_favorite = 0;
    let mut races = Vec::with_capacity(total_races);

    log::info!(
        "Начало тестирования: total races: {}; initial balance: {}; initial stake: {}; odds range: (low - {} high - {});",
        total_races,
        initial_balance,
        initial_stake,
        odds_range.low,
        odds_range.high
    );

    'responses: for resp in responses {
        for choice in &resp.choices {
            let json = match &choice.message.content {
                Some(j) => j,
                None => {
                    log::warn!("Пустой ответ от LLM модели.");
                    total_empty_content += 1;
                    continue;
                }
            };

            let mut predict = match serde_json::from_str::<PredictResponse>(json) {
                Ok(p) => p,
                Err(error) => {
                    total_race_parse_error += 1;
                    log::error!("{error}\n{json}");
                    continue;
                }
            };
            predict.sort_predictions();

            let meta_pred = &predict.meta;

            let dogs = match repo.race_participants(meta_pred.date, meta_pred.time).await {
                Ok(v) => v,
                Err(error) => {
                    total_mongo_db_error += 1;
                    log::error!("{error}");
                    continue;
                }
            };

            let n_participants = dogs.len();
            if !(5..=6).contains(&n_participants) {
                if n_participants < 5 {
                    skipped_races_lt5 += 1;
                } else {
                    skipped_races_gt6 += 1;
                }
                log::warn!(
                    "Пропущенна гонка. Кол-во участников: {}; skipped_races_lt5 == {}; skipped_races_gt6 == {}",
                    n_participants,
                    skipped_races_lt5,
                    skipped_races_gt6
                );
                continue;
            }

            let favorite_odds = dogs
                .iter()
                .map(|d| d.bf_odds_1_minute)
                .fold(f64::INFINITY, f64::min);

            let mut odds_info = Vec::new();
            for p in &predict.predictions[predict.predictions.len().saturating_sub(2)..] {
                let rec = match repo
                    .dog_record(meta_pred.date, meta_pred.time, meta_pred.distance, &p.name)
                    .await
                {
                    Ok(Some(r)) => r,
                    Ok(None) => continue,
                    Err(e) => {
                        total_mongo_db_error += 1;
                        log::error!("{e}");
                        continue;
                    }
                };

                match (p.rank as i32, rec.result_position) {
                    (4, 1) => bad_hit_4_pos += 1,
                    (5, 1) => bad_hit_5_pos += 1,
                    (6, 1) => bad_hit_6_pos += 1,
                    _ => {}
                }

                if !(odds_range.low..=odds_range.high).contains(&rec.bf_odds_1_minute) {
                    // skipped_odds_range += 1;
                    log::info!(
                        "Коэффициент не входит в указанный диапозон: {}; skipped_odds_range: {}",
                        rec.bf_odds_1_minute,
                        skipped_odds_range
                    );
                    continue;
                }

                odds_info.push(PosOdds {
                    real_position: rec.result_position,
                    odds: rec.bf_odds_1_minute,
                });
            }

            let mut test_dogs = Vec::with_capacity(n_participants);
            for dog in dogs.iter() {
                let record_opt = repo
                    .dog_record(meta_pred.date, meta_pred.time, meta_pred.distance, &dog.dog_name)
                    .await
                    .ok()
                    .flatten();

                let (rank, odds_res) = if let Some(record) = &record_opt {
                    (record.result_position as u8, record.bf_odds_1_minute as f32)
                } else {
                    (0, 0.0)
                };

                let model_pred = predict
                    .predictions
                    .iter()
                    .find(|p| p.name.eq(&dog.dog_name))
                    .cloned()
                    .unwrap_or_default();

                let real_results = TestResultsRealResults::new(rank, odds_res);

                test_dogs.push(TestResultsDog::new(
                    dog.dog_name.clone(),
                    model_pred,
                    real_results,
                ));
            }

            let bet_target_opt: Option<PosOdds> = if odds_info.is_empty() {
                skipped_odds_range += 1;
                None
            } else {
                odds_info.sort_by(|a, b| a.odds.partial_cmp(&b.odds).unwrap());
                if is_favorite_protected {
                    match odds_info.len() {
                        1 => {
                            if odds_info[0].odds == favorite_odds {
                                skipped_favorite += 1;
                                None
                            } else {
                                Some(odds_info[0])
                            }
                        }
                        _ => {
                            if odds_info[0].odds == favorite_odds {
                                skipped_favorite += 1;
                                if odds_info[1].odds == favorite_odds {
                                    // skipped_favorite += 1;
                                    None
                                } else {
                                    Some(odds_info[1])
                                }
                            } else {
                                Some(odds_info[0])
                            }
                        }
                    }
                } else {
                    Some(odds_info[0])
                }
            };

            if let Some(bet_target) = bet_target_opt {
                let obligation = initial_stake * (bet_target.odds - 1.0);
                if current_balance < obligation {
                    log::warn!(
                        "Недостаточно баланса ({}) для обязательства ставки {}. Прерываем.",
                        current_balance,
                        obligation
                    );
                    break 'responses;
                }

                if bet_target.real_position == 1 {
                    current_balance -= obligation;
                } else {
                    current_balance += initial_stake * BETFAIR_PERCENTAGE;
                }
                tracked_races += 1;
            }

            let profit = ((current_balance - initial_balance) / initial_balance) * 100.0;
            let race_meta = TestResultsRaceMeta::new(
                meta_pred.date,
                meta_pred.distance,
                meta_pred.grade.clone(),
                meta_pred.time,
                meta_pred.track.clone(),
                current_balance,
                profit,
            );

            let race_summary = predict.summary.clone().unwrap_or_default();
            let race_id = dogs.first().unwrap().race_id;
            let race_struct = TestResultsRace::new(race_id, race_meta, test_dogs, race_summary);
            races.push(race_struct);
        }
    }

    let percentage = ((current_balance - initial_balance) / initial_balance) * 100.0;
    let meta = TestResultsMeta::new(
        RaceCount::new(total_races, tracked_races),
        odds_range,
        PositionInfo::new(bad_hit_4_pos, bad_hit_5_pos, bad_hit_6_pos),
        SkipInfo::new(
            skipped_races_lt5,
            skipped_races_gt6,
            skipped_odds_range,
            skipped_favorite,
        ),
        Balance::new(initial_balance, current_balance),
        TestErrors::new(total_empty_content, total_race_parse_error, total_mongo_db_error),
        initial_stake,
        percentage,
    );

    Ok((meta, races))
}
