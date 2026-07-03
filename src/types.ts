export type FileCategory =
  | "image"
  | "document"
  | "pdf"
  | "video"
  | "audio"
  | "archive"
  | "unknown";

export interface FileInfo {
  path: string;
  name: string;
  size: number;
  mime_type: string;
  category: FileCategory;
}

export interface CompressOptions {
  quality: number;
}

export interface CompressResult {
  input_path: string;
  output_path: string;
  original_size: number;
  compressed_size: number;
  savings_percent: number;
  category: FileCategory;
}

export interface ProgressPayload {
  job_id: string;
  percent: number;
  message: string;
}

export type JobStatus = "pending" | "compressing" | "done" | "error";

export interface CompressionJob {
  id: string;
  info: FileInfo;
  status: JobStatus;
  progress: number;
  message: string;
  result?: CompressResult;
  error?: string;
}

export interface HistoryRecord {
  id: string;
  timestamp: string;
  input_path: string;
  output_path: string;
  file_name: string;
  original_size: number;
  compressed_size: number;
  savings_percent: number;
  category: FileCategory;
  quality: number;
}

export const CATEGORY_LABELS: Record<FileCategory, string> = {
  image: "Image",
  document: "Document Office",
  pdf: "PDF",
  video: "Vidéo",
  audio: "Audio",
  archive: "Archive ZIP",
  unknown: "Non supporté",
};

export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 o";
  const units = ["o", "Ko", "Mo", "Go"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const value = bytes / Math.pow(1024, i);
  return `${value.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

export function formatSavings(percent: number): string {
  if (percent <= 0) return "Aucun gain";
  return `−${percent.toFixed(1)} %`;
}
