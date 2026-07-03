use crate::commands::image;
use crate::progress::ProgressEmitter;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub fn compress(
    input: &Path,
    output: &Path,
    quality: u8,
    progress: &ProgressEmitter,
) -> Result<(), String> {
    progress.emit(10.0, "Ouverture de l'archive…");

    let data = std::fs::read(input).map_err(|e| e.to_string())?;
    let reader = Cursor::new(data);
    let mut archive = ZipArchive::new(reader).map_err(|e| e.to_string())?;
    let total = archive.len();

    let output_buffer = Vec::new();
    let mut writer = ZipWriter::new(Cursor::new(output_buffer));
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(9));

    for i in 0..total {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        let is_dir = entry.is_dir() || name.ends_with('/');

        let mut contents = Vec::new();
        entry.read_to_end(&mut contents).map_err(|e| e.to_string())?;

        let final_data = if !is_dir && image::is_image_filename(&name) {
            progress.emit(
                10.0 + (i as f32 / total as f32) * 70.0,
                format!("Compression de l'image interne : {name}"),
            );
            match image::compress_image_bytes(&contents, quality) {
                Ok(compressed) if compressed.len() < contents.len() => compressed,
                Ok(compressed) => compressed,
                Err(_) => contents,
            }
        } else {
            contents
        };

        if is_dir {
            writer.add_directory(&name, options).map_err(|e| e.to_string())?;
        } else {
            writer
                .start_file(&name, options)
                .map_err(|e| e.to_string())?;
            writer.write_all(&final_data).map_err(|e| e.to_string())?;
        }
    }

    progress.emit(90.0, "Finalisation de l'archive…");
    let finished = writer.finish().map_err(|e| e.to_string())?;
    std::fs::write(output, finished.into_inner()).map_err(|e| e.to_string())?;
    Ok(())
}
