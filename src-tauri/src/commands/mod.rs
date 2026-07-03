pub mod document;
pub mod image;
pub mod media;
pub mod pdf;
mod sidecar;

use crate::detect::{self, FileCategory, FileInfo};
use crate::progress::ProgressEmitter;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::AppHandle;

#[derive(Debug, Clone, Deserialize)]
pub struct CompressOptions {
    #[serde(default = "default_quality")]
    pub quality: u8,
}

fn default_quality() -> u8 {
    80
}

#[derive(Debug, Clone, Serialize)]
pub struct CompressResult {
    pub input_path: String,
    pub output_path: String,
    pub original_size: u64,
    pub compressed_size: u64,
    pub savings_percent: f64,
    pub category: FileCategory,
}

pub fn inspect_file(path: &str) -> Result<FileInfo, String> {
    detect::detect_file(Path::new(path))
}

pub async fn compress_file(
    app: AppHandle,
    input: String,
    output: Option<String>,
    options: CompressOptions,
    job_id: String,
) -> Result<CompressResult, String> {
    let input_path = PathBuf::from(&input);
    let info = detect::detect_file(&input_path)?;
    let output_path = output
        .map(PathBuf::from)
        .unwrap_or_else(|| detect::default_output_path(&input_path));

    if output_path == input_path {
        return Err("Le fichier de sortie doit être différent de l'original.".into());
    }

    let progress = ProgressEmitter::new(app.clone(), job_id.clone());
    progress.emit(0.0, "Démarrage de la compression…");

    let original_size = info.size;

    match info.category {
        FileCategory::Image => {
            let app = app.clone();
            let job_id = job_id.clone();
            let input_path = input_path.clone();
            let output_path = output_path.clone();
            let quality = options.quality;
            tauri::async_runtime::spawn_blocking(move || {
                let progress = ProgressEmitter::new(app, job_id);
                image::compress(&input_path, &output_path, quality, &progress)
            })
            .await
            .map_err(|e| format!("Compression interrompue : {e}"))??;
        }
        FileCategory::Document | FileCategory::Archive => {
            let app = app.clone();
            let job_id = job_id.clone();
            let input_path = input_path.clone();
            let output_path = output_path.clone();
            let quality = options.quality;
            tauri::async_runtime::spawn_blocking(move || {
                let progress = ProgressEmitter::new(app, job_id);
                document::compress(&input_path, &output_path, quality, &progress)
            })
            .await
            .map_err(|e| format!("Compression interrompue : {e}"))??;
        }
        FileCategory::Pdf => {
            let app = app.clone();
            let job_id = job_id.clone();
            let input_path = input_path.clone();
            let output_path = output_path.clone();
            let quality = options.quality;
            tauri::async_runtime::spawn_blocking(move || {
                let progress = ProgressEmitter::new(app.clone(), job_id);
                pdf::compress(&app, &input_path, &output_path, quality, &progress)
            })
            .await
            .map_err(|e| format!("Compression interrompue : {e}"))??;
        }
        FileCategory::Video | FileCategory::Audio => {
            media::compress(&app, &input_path, &output_path, &info, options.quality, &progress)
                .await?;
        }
        FileCategory::Unknown => {
            return Err(format!(
                "Type de fichier non pris en charge : {}",
                info.mime_type
            ));
        }
    }

    let compressed_size = std::fs::metadata(&output_path)
        .map_err(|e| e.to_string())?
        .len();

    let savings_percent = if original_size > 0 {
        ((original_size as f64 - compressed_size as f64) / original_size as f64) * 100.0
    } else {
        0.0
    };

    progress.emit(100.0, "Compression terminée");

    let result = CompressResult {
        input_path: input,
        output_path: output_path.to_string_lossy().into_owned(),
        original_size,
        compressed_size,
        savings_percent,
        category: info.category,
    };

    let record = crate::history::record_from_result(&result, options.quality);
    crate::history::append_record(&app, record)?;

    Ok(result)
}
