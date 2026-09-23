import {
  postFile,
  postFileWithText,
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
): Promise<Blob> {
  return postFileWithText(
    "/api/v1/pdf/compress",
    file,
    "quality",
    quality,
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
    "/api/v1/pdf/watermark",
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
    "/api/v1/pdf/page-numbers",
    file,
  );
}
