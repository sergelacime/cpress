import { revealItemInDir } from "@tauri-apps/plugin-opener";
import {
  CATEGORY_LABELS,
  formatBytes,
  formatSavings,
  type HistoryRecord,
} from "../types";

interface HistoryPanelProps {
  records: HistoryRecord[];
  loading: boolean;
  onClear: () => void;
}

function formatDate(iso: string): string {
  try {
    return new Intl.DateTimeFormat("fr-FR", {
      dateStyle: "short",
      timeStyle: "short",
    }).format(new Date(iso));
  } catch {
    return iso;
  }
}

export function HistoryPanel({ records, loading, onClear }: HistoryPanelProps) {
  if (loading) {
    return (
      <section className="history-panel">
        <div className="history-header">
          <h2>Historique</h2>
        </div>
        <p className="history-empty">Chargement…</p>
      </section>
    );
  }

  return (
    <section className="history-panel">
      <div className="history-header">
        <h2>Historique</h2>
        {records.length > 0 && (
          <button type="button" className="btn-ghost btn-sm" onClick={onClear}>
            Effacer
          </button>
        )}
      </div>

      {records.length === 0 ? (
        <p className="history-empty">Aucune compression enregistrée.</p>
      ) : (
        <ul className="history-list">
          {records.map((record) => (
            <li key={record.id}>
              <button
                type="button"
                className="history-item"
                title={record.output_path}
                onClick={() => revealItemInDir(record.output_path)}
              >
                <div className="history-item-top">
                  <span className="history-name">{record.file_name}</span>
                  <span className="history-badge">
                    {CATEGORY_LABELS[record.category]}
                  </span>
                </div>
                <div className="history-item-meta">
                  <span>{formatDate(record.timestamp)}</span>
                  <span>Qualité {record.quality}</span>
                </div>
                <div className="history-item-savings">
                  <span>
                    {formatBytes(record.original_size)} →{" "}
                    {formatBytes(record.compressed_size)}
                  </span>
                  <strong>{formatSavings(record.savings_percent)}</strong>
                </div>
                <span className="history-item-action">Afficher dans le dossier</span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
