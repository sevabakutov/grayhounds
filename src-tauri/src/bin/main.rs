#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dotenv::dotenv;
use anyhow::Result;
use mongodb::{
    options::{
        ClientOptions, 
        ServerApi, 
        ServerApiVersion
    },
    Client,
};
use tauri::Manager;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    
    let conn_str = std::env::var("DB_CONNECTION_STRING").expect("Failed to read db connection string.");
    let mut opts = ClientOptions::parse(&conn_str).await?;
    opts.server_api = Some(ServerApi::builder().version(ServerApiVersion::V1).build());
    let client = Client::with_options(opts)?;

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(move |app| {
            app.manage(client);
            Ok(())
        })
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Folder {
                        path: "log/".into(),
                        file_name: Some("app_logs".into()),
                    }),
                ])
                .level(log::LevelFilter::Debug)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
                .max_file_size(10_000_000)
                .format(|out, msg, rec| out.finish(format_args!("[{}] {}", rec.level(), msg)))
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            dogs_lib::commands::run_predict,
            dogs_lib::commands::add_instruction,
            dogs_lib::commands::read_instruction_names,
            dogs_lib::commands::load_settings,
            dogs_lib::commands::save_settings,
            dogs_lib::commands::load_time_ranges,
            dogs_lib::commands::load_predictions,
            dogs_lib::commands::run_test,
            dogs_lib::commands::copy_predict_request
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}