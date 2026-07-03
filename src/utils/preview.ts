import type { FileCategory } from "../types";

export type PreviewKind = "image" | "video" | "audio" | "pdf" | "document" | "none";

export function getPreviewKind(category: FileCategory): PreviewKind {
  switch (category) {
    case "image":
      return "image";
    case "video":
      return "video";
    case "audio":
      return "audio";
    case "pdf":
      return "pdf";
    case "document":
    case "archive":
      return "document";
    default:
      return "none";
  }
}
