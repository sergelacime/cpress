use crate::detect::FileCategory;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecord {
    pub id: String,
    pub timestamp: String,
    pub input_path: String,
    pub output_path: String,
    pub file_name: String,
    pub original_size: u64,
    pub compressed_size: u64,
    pub savings_percent: f64,
    pub category: FileCategory,
    pub quality: u8,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct HistoryStore {
    records: Vec<HistoryRecord>,
}

const MAX_RECORDS: usize = 500;

fn history_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("compression_history.json"))
}

fn read_store(app: &AppHandle) -> Result<HistoryStore, String> {
    let path = history_path(app)?;
    if !path.exists() {
        return Ok(HistoryStore::default());
    }
    let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| format!("Historique corrompu : {e}"))
}

fn write_store(app: &AppHandle, store: &HistoryStore) -> Result<(), String> {
    let path = history_path(app)?;
    let json = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn list_history(app: &AppHandle) -> Result<Vec<HistoryRecord>, String> {
    Ok(read_store(app)?.records)
}

pub fn append_record(app: &AppHandle, record: HistoryRecord) -> Result<(), String> {
    let mut store = read_store(app)?;
    store.records.insert(0, record);
    store.records.truncate(MAX_RECORDS);
    write_store(app, &store)
}

pub fn clear_history(app: &AppHandle) -> Result<(), String> {
    write_store(app, &HistoryStore::default())
}

pub fn record_from_result(
    result: &crate::commands::CompressResult,
    quality: u8,
) -> HistoryRecord {
    let file_name = PathBuf::from(&result.input_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("fichier")
        .to_string();

    HistoryRecord {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        input_path: result.input_path.clone(),
        output_path: result.output_path.clone(),
        file_name,
        original_size: result.original_size,
        compressed_size: result.compressed_size,
        savings_percent: result.savings_percent,
        category: result.category,
        quality,
    }
}
