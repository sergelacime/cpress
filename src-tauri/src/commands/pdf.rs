use super::sidecar::resolve_binary;
use crate::progress::ProgressEmitter;
use lopdf::Document;
use std::path::Path;
use std::process::{Command, Stdio};
use tauri::AppHandle;

pub fn compress(
    app: &AppHandle,
    input: &Path,
    output: &Path,
    quality: u8,
    progress: &ProgressEmitter,
) -> Result<(), String> {
    progress.emit(10.0, "Compression PDF…");

    let jpeg_quality = effective_jpeg_quality(quality);

    if let Ok(qpdf) = resolve_binary(app, "qpdf") {
        progress.emit(25.0, "Compression avec qpdf…");
        let args = qpdf_args(input, output, jpeg_quality);
        match run_qpdf(&qpdf, &args) {
            Ok(()) => {
                progress.emit(95.0, "PDF compressé");
                return Ok(());
            }
            Err(e) => {
                progress.emit(30.0, "Repli compression Rust…");
                let _ = std::fs::remove_file(output);
                eprintln!("qpdf indisponible ({e}), utilisation de lopdf");
            }
        }
    } else {
        progress.emit(25.0, "Compression Rust (lopdf)…");
    }

    compress_lopdf(input, output, progress)
}

fn effective_jpeg_quality(quality: u8) -> u8 {
    let q = quality.clamp(1, 100) as f32;
    (30.0 + (q / 100.0) * 65.0).round() as u8
}

fn qpdf_args(input: &Path, output: &Path, jpeg_quality: u8) -> Vec<String> {
    vec![
        "--compress-streams=y".to_string(),
        "--object-streams=generate".to_string(),
        "--stream-data=compress".to_string(),
        "--optimize-images".to_string(),
        format!("--jpeg-quality={jpeg_quality}"),
        "--remove-unreferenced-resources".to_string(),
        input.to_string_lossy().into_owned(),
        output.to_string_lossy().into_owned(),
    ]
}

fn run_qpdf(qpdf: &Path, args: &[String]) -> Result<(), String> {
    let output = Command::new(qpdf)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Impossible d'exécuter qpdf : {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("qpdf a échoué : {stderr}"));
    }

    Ok(())
}

/// Compression sûre via lopdf : recompression des flux uniquement, sans toucher aux images.
/// Modifier les XObject image (CMYK, masques, Indexed…) corrompait les PDF.
fn compress_lopdf(input: &Path, output: &Path, progress: &ProgressEmitter) -> Result<(), String> {
    progress.emit(45.0, "Lecture du PDF…");

    let mut doc = Document::load(input).map_err(|e| format!("PDF illisible : {e}"))?;

    progress.emit(70.0, "Recompression des flux…");
    doc.compress();

    progress.emit(85.0, "Écriture du fichier…");
    doc.save(output)
        .map_err(|e| format!("Écriture PDF : {e}"))?;

    progress.emit(95.0, "PDF compressé");
    Ok(())
}
