import {
  postFile,
  postFileWithText,
  type UploadProgressHandlers,
} from "./client";

export type CompressionQuality =
  | "high"
  | "medium"
  | "low";

/**
 * Kompres PDF.
 */
export function compressPdf(
  file: File,
  quality: CompressionQuality,
  progress?: UploadProgressHandlers,
): Promise<Blob> {
  return postFileWithText(
    "/rust-api/kompres",
    file,
    "quality",
    quality,
    undefined,
    progress,
  );
}

/**
 * Menambahkan watermark ke PDF.
 */
export function addWatermark(
  file: File | Blob,
  text: string,
): Promise<Blob> {
  return postFileWithText(
    "/rust-api/watermark",
    file,
    "text",
    text,
  );
}

/**
 * Menambahkan nomor halaman ke PDF.
 */
export function addPageNumbers(
  file: File | Blob,
): Promise<Blob> {
  return postFile(
    "/rust-api/page-numbers",
    file,
  );
}
