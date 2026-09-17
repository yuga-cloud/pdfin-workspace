import {
  postFileWithText,
  postMultipart,
} from "./client";

export const PDF_MIME =
  "application/pdf";

export const EXCEL_MIME =
  "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

/**
 * Menggabungkan beberapa file PDF.
 */
export function mergePdfs(
  files: File[]
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
  );
}

/**
 * Memisahkan PDF berdasarkan rentang halaman.
 */
export function splitPdf(
  file: File,
  rangeText: string,
): Promise<Blob> {
  return postFileWithText(
    "/rust-api/pisah",
    file,
    "ranges",
    rangeText,
  );
}

/**
 * Mengatur urutan halaman PDF.
 */
export function managePages(
  file: File,
  pageOrder: number[],
): Promise<Blob> {
  return postFileWithText(
    "/rust-api/atur-halaman",
    file,
    "pages",
    pageOrder.join(","),
  );
}

/**
 * Memutar PDF.
 */
export function rotatePdf(
  file: File,
  degrees: 90 | 180 | 270,
): Promise<Blob> {
  return postFileWithText(
    "/rust-api/putar",
    file,
    "degrees",
    String(degrees),
  );
}
