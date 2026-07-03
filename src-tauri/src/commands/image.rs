use crate::progress::ProgressEmitter;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::webp::WebPEncoder;
use image::{ExtendedColorType, ImageEncoder, ImageFormat, ImageReader};
use std::io::Cursor;
use std::path::Path;

pub fn compress(
    input: &Path,
    output: &Path,
    quality: u8,
    progress: &ProgressEmitter,
) -> Result<(), String> {
    progress.emit(10.0, "Lecture de l'image…");

    let data = std::fs::read(input).map_err(|e| e.to_string())?;
    let format = ImageReader::new(Cursor::new(&data))
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .format()
        .ok_or_else(|| "Format d'image non reconnu".to_string())?;

    progress.emit(40.0, "Compression en cours…");

    let compressed = match format {
        ImageFormat::Png => compress_png(&data)?,
        ImageFormat::Jpeg => compress_jpeg(&data, quality)?,
        ImageFormat::WebP => compress_webp(&data, quality)?,
        _ => {
            let img = image::load_from_memory(&data).map_err(|e| e.to_string())?;
            compress_raster(&img.into_rgba8())?
        }
    };

    let final_data = keep_smaller(&data, compressed);

    progress.emit(90.0, "Écriture du fichier…");
    std::fs::write(output, final_data).map_err(|e| e.to_string())?;
    Ok(())
}

fn keep_smaller(original: &[u8], compressed: Vec<u8>) -> Vec<u8> {
    if compressed.len() < original.len() {
        compressed
    } else {
        original.to_vec()
    }
}

use std::time::Duration;

fn compress_png(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut options = oxipng::Options::from_preset(2);
    options.strip = oxipng::StripChunks::Safe;
    options.fix_errors = true;
    options.timeout = Some(Duration::from_secs(30));

    match oxipng::optimize_from_memory(data, &options) {
        Ok(result) => Ok(result),
        Err(e) => {
            compress_png_fallback(data)
                .map_err(|fallback| format!("PNG : {e} (repli : {fallback})"))
        }
    }
}

fn compress_png_fallback(data: &[u8]) -> Result<Vec<u8>, String> {
    use image::codecs::png::{CompressionType, FilterType, PngEncoder};

    let img = image::load_from_memory(data).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let mut output = Vec::new();
    PngEncoder::new_with_quality(&mut output, CompressionType::Best, FilterType::Adaptive)
        .write_image(
            rgba.as_raw(),
            rgba.width(),
            rgba.height(),
            ExtendedColorType::Rgba8,
        )
        .map_err(|e| e.to_string())?;
    Ok(output)
}

fn compress_jpeg(data: &[u8], quality: u8) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(data).map_err(|e| e.to_string())?;
    let rgb = img.to_rgb8();
    let (w, h) = (rgb.width(), rgb.height());
    let pixels = rgb.as_raw();

    let mut best = data.to_vec();
    let mut q = quality;

    while q >= 20 {
        let mut output = Vec::new();
        JpegEncoder::new_with_quality(&mut output, q)
            .encode(pixels, w, h, ExtendedColorType::Rgb8)
            .map_err(|e| e.to_string())?;

        if output.len() < best.len() {
            best = output;
            break;
        }

        if q <= 20 {
            break;
        }
        q = q.saturating_sub(15);
    }

    Ok(best)
}

fn compress_webp(data: &[u8], _quality: u8) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(data).map_err(|e| e.to_string())?;
    let rgba = img.into_rgba8();
    let mut output = Vec::new();
    WebPEncoder::new_lossless(&mut output)
        .encode(
            rgba.as_raw(),
            rgba.width(),
            rgba.height(),
            ExtendedColorType::Rgba8,
        )
        .map_err(|e| e.to_string())?;
    Ok(output)
}

fn compress_raster(img: &image::RgbaImage) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    WebPEncoder::new_lossless(&mut output)
        .encode(
            img.as_raw(),
            img.width(),
            img.height(),
            ExtendedColorType::Rgba8,
        )
        .map_err(|e| e.to_string())?;
    Ok(output)
}

pub fn compress_image_bytes(data: &[u8], quality: u8) -> Result<Vec<u8>, String> {
    let format = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .format()
        .ok_or_else(|| "Format d'image non reconnu".to_string())?;

    let compressed = match format {
        ImageFormat::Png => compress_png(data)?,
        ImageFormat::Jpeg => compress_jpeg(data, quality)?,
        ImageFormat::WebP => compress_webp(data, quality)?,
        _ => {
            let img = image::load_from_memory(data).map_err(|e| e.to_string())?;
            compress_raster(&img.into_rgba8())?
        }
    };

    Ok(keep_smaller(data, compressed))
}

fn is_image_entry(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".png")
        || lower.ends_with(".webp")
}

pub fn is_image_filename(name: &str) -> bool {
    is_image_entry(name)
}
