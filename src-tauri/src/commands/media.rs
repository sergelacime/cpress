use super::sidecar::resolve_binary;
use crate::detect::{FileCategory, FileInfo};
use crate::progress::ProgressEmitter;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use tauri::AppHandle;

pub async fn compress(
    app: &AppHandle,
    input: &Path,
    output: &Path,
    info: &FileInfo,
    quality: u8,
    progress: &ProgressEmitter,
) -> Result<(), String> {
    progress.emit(5.0, "Analyse du média…");

    let ffmpeg = resolve_binary(app, "ffmpeg")?;
    let duration_ms = probe_duration(&ffmpeg, input).unwrap_or(0);
    let crf = quality_to_crf(quality);
    let audio_bitrate = quality_to_audio_bitrate(quality);

    let mut args = vec![
        "-y".to_string(),
        "-i".to_string(),
        input.to_string_lossy().into_owned(),
        "-progress".to_string(),
        "pipe:1".to_string(),
        "-nostats".to_string(),
    ];

    match info.category {
        FileCategory::Video => {
            args.extend([
                "-c:v".to_string(),
                "libx264".to_string(),
                "-crf".to_string(),
                crf.to_string(),
                "-preset".to_string(),
                "medium".to_string(),
                "-c:a".to_string(),
                "aac".to_string(),
                "-b:a".to_string(),
                format!("{audio_bitrate}k"),
            ]);
        }
        FileCategory::Audio => {
            args.extend([
                "-vn".to_string(),
                "-c:a".to_string(),
                "aac".to_string(),
                "-b:a".to_string(),
                format!("{audio_bitrate}k"),
            ]);
        }
        _ => return Err("Catégorie média invalide".into()),
    }

    args.push(output.to_string_lossy().into_owned());

    progress.emit(10.0, "Compression média en cours…");
    run_ffmpeg_with_progress(&ffmpeg, &args, duration_ms, progress)
}

fn quality_to_crf(quality: u8) -> u8 {
    let q = quality.clamp(1, 100) as f32;
    (40.0 - (q / 100.0) * 22.0).round() as u8
}

fn quality_to_audio_bitrate(quality: u8) -> u32 {
    let q = quality.clamp(1, 100) as f32;
    (64.0 + (q / 100.0) * 192.0).round() as u32
}

fn probe_duration(ffmpeg: &Path, input: &Path) -> Option<u64> {
    let args = vec![
        "-i".to_string(),
        input.to_string_lossy().into_owned(),
        "-f".to_string(),
        "null".to_string(),
        "-".to_string(),
    ];

    let output = Command::new(ffmpeg)
        .args(&args)
        .stderr(Stdio::piped())
        .output()
        .ok()?;

    parse_duration_ms(&String::from_utf8_lossy(&output.stderr))
}

fn parse_duration_ms(text: &str) -> Option<u64> {
    for line in text.lines() {
        if let Some(rest) = line.trim().strip_prefix("Duration:") {
            let time_part = rest.trim().split(',').next()?.trim();
            return parse_time_to_ms(time_part);
        }
    }
    None
}

fn parse_time_to_ms(time: &str) -> Option<u64> {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let hours: f64 = parts[0].parse().ok()?;
    let minutes: f64 = parts[1].parse().ok()?;
    let seconds: f64 = parts[2].parse().ok()?;
    Some(((hours * 3600.0 + minutes * 60.0 + seconds) * 1000.0) as u64)
}

fn run_ffmpeg_with_progress(
    ffmpeg: &Path,
    args: &[String],
    duration_ms: u64,
    progress: &ProgressEmitter,
) -> Result<(), String> {
    let mut child = Command::new(ffmpeg)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Impossible de lancer ffmpeg : {e}"))?;

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            handle_progress_line(line.as_bytes(), duration_ms, progress);
        }
    }

    let status = child.wait().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("ffmpeg terminé avec le code {status}"));
    }

    Ok(())
}

fn handle_progress_line(line: &[u8], duration_ms: u64, progress: &ProgressEmitter) {
    let text = String::from_utf8_lossy(line);
    for part in text.split_whitespace() {
        if let Some(ms_str) = part.strip_prefix("out_time_ms=") {
            if let Ok(out_ms) = ms_str.parse::<u64>() {
                let percent = if duration_ms > 0 {
                    (out_ms as f32 / duration_ms as f32) * 90.0 + 10.0
                } else {
                    50.0
                };
                progress.emit(percent, "Compression média…");
            }
        }
    }
}
