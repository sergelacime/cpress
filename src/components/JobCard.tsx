import { revealItemInDir } from "@tauri-apps/plugin-opener";
import {
  CATEGORY_LABELS,
  formatBytes,
  formatSavings,
  type CompressionJob,
} from "../types";

interface JobCardProps {
  job: CompressionJob;
  selected: boolean;
  isActive: boolean;
  onSelect: () => void;
  onCompress: () => void;
  onRemove: () => void;
}

export function JobCard({
  job,
  selected,
  isActive,
  onSelect,
  onCompress,
  onRemove,
}: JobCardProps) {
  const { info, status, progress, message, result, error } = job;

  async function handleReveal(path: string) {
    try {
      await revealItemInDir(path);
    } catch (err) {
      console.error(err);
    }
  }

  return (
    <div
      className={`job-card status-${status}${selected ? " selected" : ""}`}
      onClick={onSelect}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onSelect();
        }
      }}
    >
      <div className="job-header">
        <div className="job-info">
          <span className="job-name">{info.name}</span>
          <span className="job-badge">{CATEGORY_LABELS[info.category]}</span>
        </div>
        <span className="job-size">{formatBytes(info.size)}</span>
      </div>

      {status === "compressing" && (
        <div className="progress-block">
          <div className="progress-bar">
            <div className="progress-fill" style={{ width: `${progress}%` }} />
          </div>
          <span className="progress-text">{message}</span>
        </div>
      )}

      {result && (
        <button
          type="button"
          className="job-result success"
          title={`Afficher dans le Finder : ${result.output_path}`}
          onClick={(e) => {
            e.stopPropagation();
            handleReveal(result.output_path);
          }}
        >
          <span className="job-result-icon">📁</span>
          <span>
            {formatBytes(result.original_size)} → {formatBytes(result.compressed_size)}
          </span>
          <strong>{formatSavings(result.savings_percent)}</strong>
          <span className="output-path">{result.output_path.split("/").pop()}</span>
          <span className="job-result-action">Afficher dans le dossier</span>
        </button>
      )}

      {error && <p className="job-error">{error}</p>}

      <div className="job-actions" onClick={(e) => e.stopPropagation()}>
        {status === "pending" && info.category !== "unknown" && (
          <button type="button" onClick={onCompress} disabled={isActive}>
            Compresser
          </button>
        )}
        {(status === "done" || status === "error" || status === "pending") && (
          <button type="button" className="btn-ghost" onClick={onRemove}>
            Retirer
          </button>
        )}
      </div>
    </div>
  );
}
