#!/usr/bin/env bash
# Telecharge les binaires sidecar ffmpeg et qpdf pour la plateforme courante.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN_DIR="$ROOT/src-tauri/binaries"
mkdir -p "$BIN_DIR"

TARGET=$(rustc -vV | grep '^host:' | awk '{print $2}')
MIN_SIZE=4096
echo "Target triple: ${TARGET}"

file_size() {
  stat -f%z "$1" 2>/dev/null || stat -c%s "$1"
}

remove_if_stub() {
  local dest="$1"
  if [[ -f "$dest" ]] && [[ $(file_size "$dest") -lt $MIN_SIZE ]]; then
    echo "Stub detecte, suppression: $dest"
    rm -f "$dest"
  fi
}

download_ffmpeg() {
  local dest="$BIN_DIR/ffmpeg-${TARGET}"
  [[ "$TARGET" == *"windows"* ]] && dest="${dest}.exe"

  remove_if_stub "$dest"

  if [[ -f "$dest" ]] && [[ $(file_size "$dest") -ge $MIN_SIZE ]]; then
    echo "ffmpeg deja present: $dest ($(file_size "$dest") octets)"
    return
  fi

  echo "Telechargement ffmpeg pour ${TARGET}..."
  case "$TARGET" in
    aarch64-apple-darwin)
      url="https://www.osxexperts.net/ffmpeg80arm.zip"
      archive_type="zip"
      binary_name="ffmpeg"
      ;;
    x86_64-apple-darwin)
      url="https://www.osxexperts.net/ffmpeg80intel.zip"
      archive_type="zip"
      binary_name="ffmpeg"
      ;;
    x86_64-unknown-linux-gnu)
      url="https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl.tar.xz"
      archive_type="tar.xz"
      binary_name="ffmpeg"
      ;;
    x86_64-pc-windows-msvc)
      url="https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
      archive_type="zip"
      binary_name="ffmpeg.exe"
      ;;
    *)
      echo "ERREUR: plateforme non supportee: ${TARGET}"
      exit 1
      ;;
  esac

  tmp=$(mktemp -d)
  cleanup() { rm -rf "$tmp"; }
  trap cleanup EXIT

  if [[ "$archive_type" == "zip" ]]; then
    curl -fsSL "$url" -o "$tmp/archive.zip"
    unzip -q "$tmp/archive.zip" -d "$tmp"
    find "$tmp" -name "$binary_name" -type f -exec cp {} "$dest" \;
  else
    curl -fsSL "$url" -o "$tmp/archive.tar.xz"
    tar -xf "$tmp/archive.tar.xz" -C "$tmp"
    find "$tmp" -name "$binary_name" -type f -exec cp {} "$dest" \;
  fi

  cleanup
  trap - EXIT

  chmod +x "$dest" 2>/dev/null || true

  if [[ ! -f "$dest" ]] || [[ $(file_size "$dest") -lt $MIN_SIZE ]]; then
    echo "ERREUR: ffmpeg non installe correctement dans $dest"
    exit 1
  fi

  echo "ffmpeg installe: $dest ($(file_size "$dest") octets)"
}

download_qpdf() {
  local dest="$BIN_DIR/qpdf-${TARGET}"
  [[ "$TARGET" == *"windows"* ]] && dest="${dest}.exe"

  remove_if_stub "$dest"

  if [[ -f "$dest" ]] && [[ $(file_size "$dest") -ge $MIN_SIZE ]]; then
    echo "qpdf deja present: $dest"
    return
  fi

  if command -v qpdf &>/dev/null; then
    cp "$(command -v qpdf)" "$dest"
    chmod +x "$dest"
    echo "qpdf copie depuis le systeme: $dest"
    return
  fi

  echo "INFO: qpdf optionnel absent - PDF via moteur Rust (installez qpdf avec brew pour plus de compression)"
}

download_ffmpeg
download_qpdf
echo "Sidecars prets."
