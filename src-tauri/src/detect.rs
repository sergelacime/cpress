use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileCategory {
    Image,
    Document,
    Pdf,
    Video,
    Audio,
    Archive,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub mime_type: String,
    pub category: FileCategory,
}

const OFFICE_EXTENSIONS: &[&str] = &["docx", "xlsx", "pptx", "odt", "ods", "odp"];

pub fn detect_file(path: &Path) -> Result<FileInfo, String> {
    let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("fichier")
        .to_string();

    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let mime_type = infer::get(&bytes)
        .map(|t| t.mime_type().to_string())
        .unwrap_or_else(|| mime_from_extension(path));

    let category = categorize(&mime_type, path);

    Ok(FileInfo {
        path: path.to_string_lossy().into_owned(),
        name,
        size: metadata.len(),
        mime_type,
        category,
    })
}

fn mime_from_extension(path: &Path) -> String {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("jpg" | "jpeg") => "image/jpeg".into(),
        Some("png") => "image/png".into(),
        Some("webp") => "image/webp".into(),
        Some("pdf") => "application/pdf".into(),
        Some("mp4") => "video/mp4".into(),
        Some("mov") => "video/quicktime".into(),
        Some("mkv") => "video/x-matroska".into(),
        Some("mp3") => "audio/mpeg".into(),
        Some("wav") => "audio/wav".into(),
        Some("zip") => "application/zip".into(),
        Some(ext) if OFFICE_EXTENSIONS.contains(&ext) => {
            format!("application/vnd.office.{ext}")
        }
        _ => "application/octet-stream".into(),
    }
}

fn categorize(mime: &str, path: &Path) -> FileCategory {
    if mime.starts_with("image/") {
        return FileCategory::Image;
    }
    if mime == "application/pdf" {
        return FileCategory::Pdf;
    }
    if mime.starts_with("video/") {
        return FileCategory::Video;
    }
    if mime.starts_with("audio/") {
        return FileCategory::Audio;
    }
    if is_office_document(mime, path) {
        return FileCategory::Document;
    }
    if mime == "application/zip" || mime == "application/x-zip-compressed" {
        return FileCategory::Archive;
    }
    FileCategory::Unknown
}

fn is_office_document(mime: &str, path: &Path) -> bool {
    if mime.contains("openxmlformats") || mime.contains("oasis.opendocument") {
        return true;
    }
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| OFFICE_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn default_output_path(input: &Path) -> std::path::PathBuf {
    let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let ext = input
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!("{stem}_compressed{ext}"))
}
