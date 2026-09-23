import {
  postFile,
  postMultipart,
} from "./client";

/**
 * Konversi JPG ke PDF.
 */
export function jpgToPdf(
  files: File | Blob | Array<File | Blob>,
): Promise<Blob> {
  const values = Array.isArray(files) ? files : [files];
  const formData = new FormData();

  for (const file of values) {
    formData.append("file", file);
  }

  return postMultipart("/api/v1/convert/jpg-to-pdf", formData);
}

/**
 * Konversi PDF ke JPG.
 */
export function pdfToJpg(
  file: File,
): Promise<Blob> {
  return postFile(
    "/api/v1/convert/pdf-to-jpg",
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
    "/api/v1/convert/word-to-pdf",
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
    "/api/v1/convert/excel-to-pdf",
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
    "/api/v1/convert/powerpoint-to-pdf",
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
    "/api/v1/convert/pdf-to-word",
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
    "/api/v1/convert/pdf-to-excel",
    file,
  );
}
