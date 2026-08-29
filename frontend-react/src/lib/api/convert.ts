import {
  postFile,
} from "./client";

/**
 * Konversi JPG ke PDF.
 */
export function jpgToPdf(
  file: File,
): Promise<Blob> {
  return postFile(
    "/rust-api/jpg-to-pdf",
    file,
  );
}

/**
 * Konversi PDF ke JPG.
 */
export function pdfToJpg(
  file: File,
): Promise<Blob> {
  return postFile(
    "/rust-api/pdf-to-jpg",
    file,
  );
}

/**
 * Konversi Word ke PDF.
 */
export function wordToPdf(
  file: File,
): Promise<Blob> {
  return postFile(
    "/rust-api/word-to-pdf",
    file,
  );
}

/**
 * Konversi Excel ke PDF.
 */
export function excelToPdf(
  file: File,
): Promise<Blob> {
  return postFile(
    "/rust-api/excel-to-pdf",
    file,
  );
}

/**
 * Konversi PowerPoint ke PDF.
 */
export function powerpointToPdf(
  file: File,
): Promise<Blob> {
  return postFile(
    "/rust-api/powerpoint-to-pdf",
    file,
  );
}

/**
 * Konversi PDF ke Word.
 */
export function pdfToWord(
  file: File,
): Promise<Blob> {
  return postFile(
    "/rust-api/pdf-to-word",
    file,
  );
}

/**
 * Konversi PDF ke Excel.
 */
export function pdfToExcel(
  file: File,
): Promise<Blob> {
  return postFile(
    "/rust-api/pdf-to-excel",
    file,
  );
}

/**
 * Konversi PDF ke PowerPoint.
 */
export function pdfToPowerpoint(
  file: File,
): Promise<Blob> {
  return postFile(
    "/rust-api/pdf-to-powerpoint",
    file,
  );
}
