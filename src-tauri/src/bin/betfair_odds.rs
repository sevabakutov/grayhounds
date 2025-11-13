use bzip2::read::BzDecoder;
use mongodb::{
    bson::{doc, Document},
    options::ClientOptions,
    Client as MongoClient,
};
use serde_json::Value;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct RunnerInfo {
    race_id: i64,
    dog_name: String,
    bsp: f64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to MongoDB
    let mut opts = ClientOptions::parse("mongodb+srv://qwertyrom:qwertyrom@dogs.wkobavq.mongodb.net/main?retryWrites=true&w=majority&appName=DOGS").await?;
    opts.app_name = Some("GreyhoundScraper".to_string());
    let mongo = MongoClient::with_options(opts)?;
    let db = match mongo.default_database() {
        Some(database) => database,
        None => {
            eprintln!("Error: No default database configured in MongoDB connection string");
            return Ok(());
        }
    };
    let collection = db.collection::<Document>("dog_race_info_3");

    // Path relative to src-tauri directory (parent of src/bin)
    let bf_odds_dir = "BF_ODDS";

    println!(
        "Starting recursive search for .bz2 files in: {}",
        bf_odds_dir
    );
    println!("Scanning all subdirectories...\n");

    let bz2_files = find_bz2_files(bf_odds_dir)?;
    println!("\nFound {} .bz2 files to process\n", bz2_files.len());

    if bz2_files.is_empty() {
        println!("No .bz2 files found. Please check:");
        println!("  1. Directory 'BF_ODDS' exists in current path");
        println!("  2. .bz2 files are present in subdirectories");
        return Ok(());
    }

    let mut total_runners_found = 0;
    let mut total_updated = 0;

    for (index, file_path) in bz2_files.iter().enumerate() {
        println!(
            "[{}/{}] Processing: {:?}",
            index + 1,
            bz2_files.len(),
            file_path
        );
        match process_bz2_file(file_path) {
            Ok(runners) => {
                if runners.is_empty() {
                    println!("  No runner data found in this file");
                } else {
                    println!("  Found {} runners:", runners.len());
                    for runner in &runners {
                        let formatted_name = format_dog_name(&runner.dog_name);
                        println!(
                            "    Race ID: {} | Dog: {} | BSP: {:.2}",
                            runner.race_id, formatted_name, runner.bsp
                        );

                        // Update MongoDB document
                        match update_runner_in_db(&collection, runner).await {
                            Ok(updated) => {
                                if updated {
                                    println!("      ≠ Updated in MongoDB");
                                    total_updated += 1;
                                } else {
                                    // println!("      ✗ Not found in MongoDB");
                                }
                            }
                            Err(e) => {
                                eprintln!("      ✗ MongoDB error: {}", e);
                            }
                        }
                    }
                    total_runners_found += runners.len();
                }
            }
            Err(e) => {
                eprintln!("  ERROR: {}", e);
            }
        }
        println!();
    }

    println!("======================");
    println!("Total files processed: {}", bz2_files.len());
    println!("Total runners found: {}", total_runners_found);
    println!("Total documents updated: {}", total_updated);
    println!("======================");

    Ok(())
}

fn find_bz2_files(dir: &str) -> anyhow::Result<Vec<PathBuf>> {
    let mut bz2_files = Vec::new();
    visit_dirs(Path::new(dir), &mut bz2_files)?;
    Ok(bz2_files)
}

fn visit_dirs(dir: &Path, bz2_files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, bz2_files)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("bz2") {
                bz2_files.push(path);
            }
        }
    }
    Ok(())
}

fn format_dog_name(name: &str) -> String {
    // Remove pattern like "1. ", "2. ", etc. from the beginning
    if let Some(idx) = name.find(". ") {
        // Check if everything before ". " is a number
        let prefix = &name[..idx];
        if prefix.chars().all(|c| c.is_numeric()) {
            return name[idx + 2..].to_string();
        }
    }
    name.to_string()
}

async fn update_runner_in_db(
    collection: &mongodb::Collection<mongodb::bson::Document>,
    runner: &RunnerInfo,
) -> anyhow::Result<bool> {
    // Format dog name: remove "1. ", "2. ", etc.
    let formatted_dog_name = format_dog_name(&runner.dog_name);

    // Create filter to find document by dog_name and race_id
    let filter = doc! {
        "dog_name": &formatted_dog_name,
        "race_id": runner.race_id
    };

    // Create update to set bf_odds_1_minute field with bsp value
    let update = doc! {
        "$set": {
            "bf_odds_1_minute": runner.bsp
        }
    };

    // Execute update
    let result = collection.update_one(filter, update).await?;

    // Return true if document was found and updated
    Ok(result.matched_count > 0)
}

fn process_bz2_file(file_path: &Path) -> anyhow::Result<Vec<RunnerInfo>> {
    let file = File::open(file_path)?;
    let decoder = BzDecoder::new(file);
    let reader = BufReader::new(decoder);

    let mut results = Vec::new();
    let mut last_json: Option<Value> = None;

    // Читаем все строки и сохраняем последний валидный JSON
    for line in reader.lines() {
        let line = line?;

        // Пропускаем пустые строки
        if line.trim().is_empty() {
            continue;
        }

        // Пытаемся распарсить JSON
        if let Ok(data) = serde_json::from_str::<Value>(&line) {
            last_json = Some(data);
        }
    }

    // Обрабатываем последний JSON
    if let Some(data) = last_json {
        // Достаем основной блок данных mc[0]
        if let Some(mc) = data.get("mc").and_then(|v| v.as_array()) {
            if let Some(market_change) = mc.first() {
                // Получаем id (market id) из mc[0].id
                let market_id_str = match market_change.get("id").and_then(|v| v.as_str()) {
                    Some(id) if !id.is_empty() => id,
                    _ => {
                        println!("Warning: Missing market id, skipping file");
                        return Ok(results);
                    }
                };

                // Форматируем race_id: убираем префикс "1." -> получаем числовую часть
                let race_id = if let Some(dot_pos) = market_id_str.find('.') {
                    match market_id_str[dot_pos + 1..].parse::<i64>() {
                        Ok(id) if id > 0 => id,
                        _ => {
                            println!(
                                "Warning: Invalid race_id format after dot: {}",
                                market_id_str
                            );
                            return Ok(results);
                        }
                    }
                } else {
                    match market_id_str.parse::<i64>() {
                        Ok(id) if id > 0 => id,
                        _ => {
                            println!("Warning: Invalid race_id format: {}", market_id_str);
                            return Ok(results);
                        }
                    }
                };

                // Ищем marketDefinition с информацией о участниках
                if let Some(market_def) = market_change.get("marketDefinition") {
                    // Извлекаем информацию о runners
                    if let Some(runners) = market_def.get("runners").and_then(|r| r.as_array()) {
                        for runner in runners {
                            let dog_name = match runner.get("name").and_then(|v| v.as_str()) {
                                Some(name) if !name.is_empty() => name.to_string(),
                                _ => {
                                    println!(
                                        "Warning: Missing dog name for race_id {}, skipping runner",
                                        race_id
                                    );
                                    continue;
                                }
                            };

                            let bsp = match runner.get("bsp").and_then(|v| v.as_f64()) {
                                Some(value) if value > 0.0 => value,
                                _ => {
                                    println!("Warning: Missing or invalid BSP for dog '{}' in race_id {}, skipping runner", dog_name, race_id);
                                    continue;
                                }
                            };

                            results.push(RunnerInfo {
                                race_id,
                                dog_name,
                                bsp,
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(results)
}
