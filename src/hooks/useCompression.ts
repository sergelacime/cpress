import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type {
  CompressionJob,
  CompressOptions,
  CompressResult,
  FileInfo,
  ProgressPayload,
} from "../types";

function createJob(info: FileInfo): CompressionJob {
  return {
    id: crypto.randomUUID(),
    info,
    status: "pending",
    progress: 0,
    message: "En attente…",
  };
}

export function useCompression(quality: number, onCompressed?: () => void) {
  const [jobs, setJobs] = useState<CompressionJob[]>([]);
  const [activeJobId, setActiveJobId] = useState<string | null>(null);

  useEffect(() => {
    const unlisten = listen<ProgressPayload>("compress-progress", (event) => {
      const { job_id, percent, message } = event.payload;
      setJobs((prev) =>
        prev.map((job) =>
          job.id === job_id
            ? { ...job, progress: percent, message, status: "compressing" }
            : job,
        ),
      );
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const addFiles = useCallback(async (paths: string[]) => {
    const newJobs: CompressionJob[] = [];
    for (const path of paths) {
      try {
        const info = await invoke<FileInfo>("inspect_file", { path });
        newJobs.push(createJob(info));
      } catch (err) {
        newJobs.push({
          id: crypto.randomUUID(),
          info: {
            path,
            name: path.split("/").pop() ?? path,
            size: 0,
            mime_type: "unknown",
            category: "unknown",
          },
          status: "error",
          progress: 0,
          message: String(err),
          error: String(err),
        });
      }
    }
    setJobs((prev) => [...prev, ...newJobs]);
  }, []);

  const pickFiles = useCallback(async () => {
    const selected = await open({
      multiple: true,
      directory: false,
    });
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      await addFiles(paths);
    }
  }, [addFiles]);

  useEffect(() => {
    const win = getCurrentWindow();
    const unlisten = win.onDragDropEvent(async (event) => {
      if (event.payload.type === "drop") {
        await addFiles(event.payload.paths);
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [addFiles]);

  const compressJob = useCallback(
    async (jobId: string) => {
      const job = jobs.find((j) => j.id === jobId);
      if (!job || job.status === "compressing") return;

      setActiveJobId(jobId);
      setJobs((prev) =>
        prev.map((j) =>
          j.id === jobId
            ? { ...j, status: "compressing", progress: 0, message: "Démarrage…" }
            : j,
        ),
      );

      try {
        const options: CompressOptions = { quality };
        const result = await invoke<CompressResult>("compress_file", {
          input: job.info.path,
          output: null,
          options,
          jobId: job.id,
        });
        setJobs((prev) =>
          prev.map((j) =>
            j.id === jobId
              ? {
                  ...j,
                  status: "done",
                  progress: 100,
                  message: "Terminé",
                  result,
                }
              : j,
          ),
        );
        onCompressed?.();
      } catch (err) {
        setJobs((prev) =>
          prev.map((j) =>
            j.id === jobId
              ? {
                  ...j,
                  status: "error",
                  message: String(err),
                  error: String(err),
                }
              : j,
          ),
        );
      } finally {
        setActiveJobId(null);
      }
    },
    [jobs, quality, onCompressed],
  );

  const compressAll = useCallback(async () => {
    const pending = jobs.filter(
      (j) => j.status === "pending" && j.info.category !== "unknown",
    );
    for (const job of pending) {
      await compressJob(job.id);
    }
  }, [jobs, compressJob]);

  const removeJob = useCallback((jobId: string) => {
    setJobs((prev) => prev.filter((j) => j.id !== jobId));
  }, []);

  const clearDone = useCallback(() => {
    setJobs((prev) => prev.filter((j) => j.status !== "done"));
  }, []);

  return {
    jobs,
    activeJobId,
    addFiles,
    pickFiles,
    compressJob,
    compressAll,
    removeJob,
    clearDone,
  };
}
