import { useEffect, useMemo, useState } from "react";
import { useCompression } from "./hooks/useCompression";
import { useTheme } from "./hooks/useTheme";
import { FilePreview } from "./components/FilePreview";
import { JobCard } from "./components/JobCard";
import { HistoryPanel } from "./components/HistoryPanel";
import { ThemeToggle } from "./components/ThemeToggle";
import { useHistory } from "./hooks/useHistory";
import type { CompressionJob } from "./types";
import "./App.css";

type PreviewSource = "original" | "compressed";

function getPreviewForJob(
  job: CompressionJob | undefined,
  source: PreviewSource,
) {
  if (!job) return { path: null, name: null, category: null, size: undefined, label: undefined };

  if (source === "compressed" && job.result) {
    return {
      path: job.result.output_path,
      name: job.result.output_path.split("/").pop() ?? job.info.name,
      category: job.info.category,
      size: job.result.compressed_size,
      label: "Fichier compressé",
    };
  }

  return {
    path: job.info.path,
    name: job.info.name,
    category: job.info.category,
    size: job.info.size,
    label: job.status === "done" ? "Fichier original" : "Aperçu",
  };
}

function App() {
  const [quality, setQuality] = useState(80);
  const [dragOver, setDragOver] = useState(false);
  const [selectedJobId, setSelectedJobId] = useState<string | null>(null);
  const [previewSource, setPreviewSource] = useState<PreviewSource>("original");
  const { theme, toggle: toggleTheme } = useTheme();
  const { records, loading, refresh, clearHistory } = useHistory();

  const {
    jobs,
    activeJobId,
    pickFiles,
    compressJob,
    compressAll,
    removeJob,
    clearDone,
  } = useCompression(quality, refresh);

  const selectedJob = jobs.find((j) => j.id === selectedJobId);

  useEffect(() => {
    if (selectedJobId && !jobs.some((j) => j.id === selectedJobId)) {
      setSelectedJobId(null);
    }
  }, [jobs, selectedJobId]);

  useEffect(() => {
    if (selectedJob?.status === "done" && selectedJob.result) {
      setPreviewSource("compressed");
    }
  }, [selectedJob?.id, selectedJob?.status]);

  const preview = useMemo(
    () => getPreviewForJob(selectedJob, previewSource),
    [selectedJob, previewSource],
  );

  const pendingCount = jobs.filter(
    (j) => j.status === "pending" && j.info.category !== "unknown",
  ).length;
  const doneCount = jobs.filter((j) => j.status === "done").length;

  function handleSelectJob(jobId: string) {
    setSelectedJobId(jobId);
    const job = jobs.find((j) => j.id === jobId);
    if (job?.status === "done" && job.result) {
      setPreviewSource("compressed");
    } else {
      setPreviewSource("original");
    }
  }

  return (
    <div className="app">
      <header className="header">
        <div className="header-brand">
          <img src="/logo-header.png" alt="cPress" className="app-logo" />
          <p className="subtitle">
            Compression locale — images, documents, PDF, audio & vidéo
          </p>
        </div>
        <ThemeToggle theme={theme} onToggle={toggleTheme} />
      </header>

      <div className="app-layout">
        <div className="main-column">
          <section className="controls">
            <label className="quality-control">
              <span>Qualité</span>
              <input
                type="range"
                min={1}
                max={100}
                value={quality}
                onChange={(e) => setQuality(Number(e.target.value))}
              />
              <span className="quality-value">{quality}</span>
            </label>

            <div className="control-buttons">
              <button type="button" onClick={pickFiles}>
                Choisir des fichiers
              </button>
              {pendingCount > 0 && (
                <button
                  type="button"
                  className="btn-primary"
                  onClick={compressAll}
                  disabled={!!activeJobId}
                >
                  Tout compresser ({pendingCount})
                </button>
              )}
              {doneCount > 0 && (
                <button type="button" className="btn-ghost" onClick={clearDone}>
                  Effacer terminés
                </button>
              )}
            </div>
          </section>

          <section
            className={`drop-zone ${dragOver ? "drag-over" : ""} ${jobs.length === 0 ? "empty" : ""}`}
            onDragEnter={() => setDragOver(true)}
            onDragLeave={() => setDragOver(false)}
            onClick={pickFiles}
          >
            <div className="drop-content">
              <span className="drop-icon">↓</span>
              <p>Glissez-déposez vos fichiers ici</p>
              <p className="drop-hint">JPEG, PNG, WebP, DOCX, XLSX, PDF, MP4, MP3…</p>
            </div>
          </section>

          {jobs.length > 0 && (
            <section className="job-list">
              {jobs.map((job) => (
                <JobCard
                  key={job.id}
                  job={job}
                  selected={selectedJobId === job.id}
                  isActive={!!activeJobId && activeJobId !== job.id}
                  onSelect={() => handleSelectJob(job.id)}
                  onCompress={() => compressJob(job.id)}
                  onRemove={() => removeJob(job.id)}
                />
              ))}
            </section>
          )}

          <footer className="footer">
            <p>
              Les fichiers compressés sont enregistrés à côté de l&apos;original avec le
              suffixe <code>_compressed</code>.
            </p>
            <p className="footer-copyright">
              © {new Date().getFullYear()} sergelacime — cPress
            </p>
          </footer>

          <HistoryPanel
            records={records}
            loading={loading}
            onClear={clearHistory}
          />
        </div>

        <div className="preview-column">
          {selectedJob?.status === "done" && selectedJob.result && (
            <div className="preview-source-toggle">
              <button
                type="button"
                className={previewSource === "original" ? "active" : ""}
                onClick={() => setPreviewSource("original")}
              >
                Original
              </button>
              <button
                type="button"
                className={previewSource === "compressed" ? "active" : ""}
                onClick={() => setPreviewSource("compressed")}
              >
                Compressé
              </button>
            </div>
          )}
          <FilePreview
            path={preview.path}
            name={preview.name}
            category={preview.category}
            size={preview.size}
            label={preview.label}
          />
        </div>
      </div>
    </div>
  );
}

export default App;
