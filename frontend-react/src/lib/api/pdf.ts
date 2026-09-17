import {
  postFileWithText,
  postMultipart,
  type UploadProgressHandlers,
} from "./client";

export const PDF_MIME =
  "application/pdf";

export const EXCEL_MIME =
  "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

/**
 * Menggabungkan beberapa file PDF.
 */
export function mergePdfs(
  files: File[],
  progress?: UploadProgressHandlers,
): Promise<Blob> {
  const formData =
    new FormData();

  for (const file of files) {
    formData.append(
      "file",
      file,
    );
  }

  return postMultipart(
    "/rust-api/gabung",
    formData,
    undefined,
    progress,
  );
}

/**
 * Memisahkan PDF berdasarkan rentang halaman.
 */
export function splitPdf(
  file: File,
  rangeText: string,
  progress?: UploadProgressHandlers,
): Promise<Blob> {
  return postFileWithText(
    "/rust-api/pisah",
    file,
    "ranges",
    rangeText,
    undefined,
    progress,
  );
}

/**
 * Mengatur urutan halaman PDF.
 */
export function managePages(
  file: File,
  pageOrder: number[],
  progress?: UploadProgressHandlers,
): Promise<Blob> {
  return postFileWithText(
    "/rust-api/atur-halaman",
    file,
    "pages",
    pageOrder.join(","),
    undefined,
    progress,
  );
}

/**
 * Memutar PDF.
 */
export function rotatePdf(
  file: File,
  degrees: 90 | 180 | 270,
  progress?: UploadProgressHandlers,
): Promise<Blob> {
  return postFileWithText(
    "/rust-api/putar",
    file,
    "degrees",
    String(degrees),
    undefined,
    progress,
  );
}
