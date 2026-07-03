use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

const MIN_BINARY_BYTES: u64 = 4096;

/// Chemin vers un binaire réel (sidecar embarqué, dev ou système).
pub fn resolve_binary(app: &AppHandle, name: &str) -> Result<PathBuf, String> {
    find_bundled_sidecar(app, name)
        .or_else(|| find_dev_sidecar(name))
        .or_else(|| which::which(name).ok())
        .ok_or_else(|| {
            format!(
                "« {name} » introuvable. \
                 Exécutez « bash scripts/download-sidecars.sh » avant « pnpm tauri build »."
            )
        })
}

fn is_real_binary(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|meta| meta.is_file() && meta.len() >= MIN_BINARY_BYTES)
        .unwrap_or(false)
}

fn find_dev_sidecar(name: &str) -> Option<PathBuf> {
    let bin_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries");
    let entries = std::fs::read_dir(&bin_dir).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with(name) && is_real_binary(&path))
        {
            return Some(path);
        }
    }

    None
}

fn find_bundled_sidecar(app: &AppHandle, name: &str) -> Option<PathBuf> {
    use tauri::path::BaseDirectory;

    let resolved = app
        .path()
        .resolve(format!("binaries/{name}"), BaseDirectory::Resource)
        .ok()?;

    is_real_binary(&resolved).then_some(resolved)
}
