mod commands;
mod detect;
mod history;
mod progress;

use commands::{CompressOptions, CompressResult};
use history::HistoryRecord;

#[tauri::command]
fn inspect_file(path: String) -> Result<detect::FileInfo, String> {
    commands::inspect_file(&path)
}

#[tauri::command]
async fn compress_file(
    app: tauri::AppHandle,
    input: String,
    output: Option<String>,
    options: CompressOptions,
    job_id: String,
) -> Result<CompressResult, String> {
    commands::compress_file(app, input, output, options, job_id).await
}

#[tauri::command]
fn list_compression_history(app: tauri::AppHandle) -> Result<Vec<HistoryRecord>, String> {
    history::list_history(&app)
}

#[tauri::command]
fn clear_compression_history(app: tauri::AppHandle) -> Result<(), String> {
    history::clear_history(&app)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            inspect_file,
            compress_file,
            list_compression_history,
            clear_compression_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
