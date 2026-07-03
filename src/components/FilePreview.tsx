import { convertFileSrc } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import type { FileCategory } from "../types";
import { formatBytes } from "../types";
import { getPreviewKind } from "../utils/preview";

interface FilePreviewProps {
  path: string | null;
  name: string | null;
  category: FileCategory | null;
  size?: number;
  label?: string;
}

export function FilePreview({
  path,
  name,
  category,
  size,
  label,
}: FilePreviewProps) {
  if (!path || !name || !category) {
    return (
      <aside className="preview-panel preview-empty">
        <p>Sélectionnez un fichier pour l&apos;aperçu</p>
      </aside>
    );
  }

  const kind = getPreviewKind(category);
  const src = convertFileSrc(path);

  async function handleOpenExternal() {
    try {
      await openPath(path!);
    } catch (err) {
      console.error(err);
    }
  }

  return (
    <aside className="preview-panel">
      <div className="preview-header">
        <div>
          {label && <span className="preview-label">{label}</span>}
          <h2 className="preview-title" title={name}>
            {name}
          </h2>
          {size !== undefined && size > 0 && (
            <span className="preview-meta">{formatBytes(size)}</span>
          )}
        </div>
        <button type="button" className="btn-ghost btn-sm" onClick={handleOpenExternal}>
          Ouvrir
        </button>
      </div>

      <div className="preview-body">
        {kind === "image" && (
          <img src={src} alt={name} className="preview-media preview-image" />
        )}

        {kind === "video" && (
          <video
            key={path}
            src={src}
            controls
            className="preview-media preview-video"
            preload="metadata"
          />
        )}

        {kind === "audio" && (
          <div className="preview-audio-wrap">
            <div className="preview-audio-icon">♫</div>
            <audio key={path} src={src} controls className="preview-audio" preload="metadata" />
          </div>
        )}

        {kind === "pdf" && (
          <iframe
            key={path}
            src={src}
            title={name}
            className="preview-pdf"
          />
        )}

        {kind === "document" && (
          <div className="preview-doc">
            <div className="preview-doc-icon">
              {category === "archive" ? "📦" : "📄"}
            </div>
            <p className="preview-doc-title">{name}</p>
            <p className="preview-doc-hint">
              {category === "archive"
                ? "Archive ZIP — ouvrez avec votre gestionnaire de fichiers."
                : "Document Office — ouvrez avec l'application par défaut."}
            </p>
            <button type="button" onClick={handleOpenExternal}>
              Ouvrir le document
            </button>
          </div>
        )}

        {kind === "none" && (
          <div className="preview-doc">
            <p className="preview-doc-hint">Aperçu non disponible pour ce type de fichier.</p>
            <button type="button" onClick={handleOpenExternal}>
              Ouvrir le fichier
            </button>
          </div>
        )}
      </div>
    </aside>
  );
}
