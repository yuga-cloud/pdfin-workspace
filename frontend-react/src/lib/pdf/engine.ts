import type { ToolSlug } from "@/lib/tools-catalog";

import {
  jpgToPdf as jpgToPdfApi,
  excelToPdf as excelToPdfApi,
  pdfToExcel as pdfToExcelApi,
} from "@/lib/api/convert";

import {
  compressPdf as compressPdfApi,
} from "@/lib/api/optimize";

import {
  mergePdfs as mergePdfsApi,
  splitPdf as splitPdfApi,
  managePages as managePagesApi,
  rotatePdf as rotatePdfApi,
} from "@/lib/api/pdf";

import {
  blobFromBytes,
  stem,
  yieldToUi,
} from "@/lib/pdf/bytes";

export type Quality = "high" | "medium" | "low";

export type ToolOptions = {
  rangeText: string;
  splitEach: boolean;
  quality: Quality;
  rotateDegrees: 90 | 180 | 270;
  watermarkText: string;
  watermarkOpacity: number;
  addPageNumbers: boolean;
  pageOrder: number[] | null;
};

export type ProgressFn = (
  done: number,
  total: number,
  label: string,
) => void;

export type ProcessOutput = {
  blob: Blob;
  filename: string;
  mime: string;
};

function fail(message: string): never {
  throw new Error(message);
}

function throwIfAborted(signal?: AbortSignal): void {
  if (!signal?.aborted) {
    return;
  }

  const error = new Error("Pembuatan pratinjau dibatalkan.");
  error.name = "AbortError";
  throw error;
}

export function revokeObjectUrls(urls: readonly string[]): void {
  for (const url of urls) {
    URL.revokeObjectURL(url);
  }
}

async function loadPdfLib() {
  return import("pdf-lib");
}

let pdfjsMod: typeof import("pdfjs-dist") | null = null;

export async function loadPdfjs() {
  if (pdfjsMod) {
    return pdfjsMod;
  }

  const pdfjs = await import("pdfjs-dist");

  const worker = await import("pdfjs-dist/build/pdf.worker.min.mjs?url");

  pdfjs.GlobalWorkerOptions.workerSrc = worker.default;

  pdfjsMod = pdfjs;

  return pdfjs;
}

async function normalizeImageToJpeg(file: File): Promise<Blob> {
  if (file.type === "image/jpeg" || /\.jpe?g$/i.test(file.name)) {
    return file;
  }

  if (typeof createImageBitmap !== "function") {
    fail("Browser tidak mendukung konversi format gambar ini.");
  }

  const bitmap = await createImageBitmap(file);
  try {
    const canvas = document.createElement("canvas");
    canvas.width = bitmap.width;
    canvas.height = bitmap.height;

    const context = canvas.getContext("2d");
    if (!context) {
      fail("Browser tidak mendukung canvas.");
    }

    context.drawImage(bitmap, 0, 0);

    return await new Promise<Blob>((resolve, reject) => {
      canvas.toBlob(
        (blob) =>
          blob
            ? resolve(blob)
            : reject(new Error("Gagal mengonversi gambar ke JPEG.")),
        "image/jpeg",
        0.92,
      );
    });
  } finally {
    bitmap.close();
  }
}

async function fileBytes(file: File): Promise<Uint8Array> {
  return new Uint8Array(await file.arrayBuffer());
}

async function loadPdfDoc(bytes: Uint8Array) {
  const { PDFDocument } = await loadPdfLib();

  try {
    return await PDFDocument.load(bytes);
  } catch {
    fail("PDF tidak bisa dibaca. Mungkin rusak atau terkunci password.");
  }
}

export function parsePageRange(text: string, pageCount: number): number[] {
  const value = text.trim();

  if (!value) {
    return Array.from({ length: pageCount }, (_, index) => index);
  }

  const indexes = new Set<number>();

  for (const raw of value.split(/[,;]+/)) {
    const part = raw.trim();

    if (!part) {
      continue;
    }

    const match = part.match(/^(\d+)\s*-\s*(\d+)$/);

    if (match) {
      let start = Number(match[1]);
      let end = Number(match[2]);

      if (start > end) {
        [start, end] = [end, start];
      }

      for (let page = start; page <= end; page += 1) {
        if (page >= 1 && page <= pageCount) {
          indexes.add(page - 1);
        }
      }

      continue;
    }

    if (/^\d+$/.test(part)) {
      const page = Number(part);

      if (page >= 1 && page <= pageCount) {
        indexes.add(page - 1);
        continue;
      }
    }

    fail(`Rentang tidak valid: "${part}". Contoh: 1-3, 5, 8-10`);
  }

  if (indexes.size === 0) {
    fail("Tidak ada halaman yang cocok dengan rentang itu.");
  }

  return [...indexes].sort((a, b) => a - b);
}

export async function countPages(file: File): Promise<number> {
  const doc = await loadPdfDoc(await fileBytes(file));

  return doc.getPageCount();
}

export async function renderThumbs(
  file: File,
  onProgress?: ProgressFn,
  signal?: AbortSignal,
): Promise<string[]> {
  const pdfjs = await loadPdfjs();
  throwIfAborted(signal);

  const data = await fileBytes(file);
  throwIfAborted(signal);

  const loadingTask = pdfjs.getDocument({
    data,
    disableAutoFetch: true,
  });

  const pdf = await loadingTask.promise;
  const urls: string[] = [];
  const total = pdf.numPages;

  try {
    for (let pageNumber = 1; pageNumber <= total; pageNumber += 1) {
      throwIfAborted(signal);

      onProgress?.(
        pageNumber - 1,
        total,
        `Pratinjau halaman ${pageNumber}/${total}`,
      );

      const page = await pdf.getPage(pageNumber);
      const base = page.getViewport({ scale: 1 });
      const scale = 160 / base.width;
      const viewport = page.getViewport({ scale });
      const canvas = document.createElement("canvas");

      canvas.width = Math.max(1, Math.floor(viewport.width));
      canvas.height = Math.max(1, Math.floor(viewport.height));

      const context = canvas.getContext("2d");
      if (!context) {
        fail("Browser tidak mendukung canvas.");
      }

      context.fillStyle = "#fff";
      context.fillRect(0, 0, canvas.width, canvas.height);

      await page.render({ canvas, canvasContext: context, viewport }).promise;

      const blob = await new Promise<Blob>((resolve, reject) => {
        canvas.toBlob(
          (value) => {
            if (value) {
              resolve(value);
            } else {
              reject(new Error("Gagal membuat pratinjau JPG."));
            }
          },
          "image/jpeg",
          0.72,
        );
      });

      const url = URL.createObjectURL(blob);
      canvas.width = 0;
      canvas.height = 0;

      if (signal?.aborted) {
        URL.revokeObjectURL(url);
        throwIfAborted(signal);
      }

      urls.push(url);
      page.cleanup();
      await yieldToUi();
    }
  } catch (error) {
    revokeObjectUrls(urls);
    throw error;
  } finally {
    await pdf.cleanup();
  }

  onProgress?.(total, total, "Pratinjau siap");

  return urls;
}

async function splitEachPage(
  file: File,
  onProgress?: ProgressFn,
): Promise<Blob> {
  const { PDFDocument } = await loadPdfLib();
  const { default: JSZip } = await import("jszip");
  const source = await loadPdfDoc(await fileBytes(file));
  const zip = new JSZip();
  const total = source.getPageCount();
  const base = stem(file.name);

  for (let index = 0; index < total; index += 1) {
    onProgress?.(index, total, `Halaman ${index + 1}/${total}`);

    const output = await PDFDocument.create();
    const [page] = await output.copyPages(source, [index]);
    output.addPage(page);
    const bytes = await output.save();

    zip.file(
      `${base}-halaman-${String(index + 1).padStart(3, "0")}.pdf`,
      bytes,
    );

    await yieldToUi();
  }

  onProgress?.(total, total, "Mengemas ZIP");

  return zip.generateAsync({ type: "blob" });
}

async function mergeAll(files: File[], onProgress?: ProgressFn): Promise<Blob> {
  onProgress?.(0, 1, "Menggabungkan PDF di server Rust...");
  const blob = await mergePdfsApi(files);
  onProgress?.(1, 1, "Selesai");
  return blob;
}

async function splitSelectedPages(
  file: File,
  rangeText: string,
  pageCount: number,
  onProgress?: ProgressFn,
): Promise<Blob> {
  const indexes = parsePageRange(rangeText, pageCount);
  const ranges: Array<{ start: number; end: number }> = [];
  let start = indexes[0];
  let previous = indexes[0];

  for (let index = 1; index < indexes.length; index += 1) {
    const current = indexes[index];

    if (current === previous + 1) {
      previous = current;
      continue;
    }

    ranges.push({ start: start + 1, end: previous + 1 });
    start = current;
    previous = current;
  }

  ranges.push({ start: start + 1, end: previous + 1 });

  const rangeValue = ranges
    .map((range) =>
      range.start === range.end ? String(range.start) : `${range.start}-${range.end}`,
    )
    .join(",");

  onProgress?.(0, 1, "Memisahkan PDF di server Rust...");
  const blob = await splitPdfApi(file, rangeValue);
  onProgress?.(1, 1, "Selesai");
  return blob;
}

async function manageAllPages(
  file: File,
  pageOrder: number[],
  onProgress?: ProgressFn,
): Promise<Blob> {
  const backendPageOrder = pageOrder.map((index) => index + 1);
  onProgress?.(0, 1, "Mengatur halaman di server Rust...");
  const blob = await managePagesApi(file, backendPageOrder);
  onProgress?.(1, 1, "Selesai");
  return blob;
}

async function rotateAll(
  file: File,
  degrees: 90 | 180 | 270,
  onProgress?: ProgressFn,
): Promise<Blob> {
  onProgress?.(0, 1, "Memutar PDF di server Rust...");
  const blob = await rotatePdfApi(file, degrees);
  onProgress?.(1, 1, "Selesai");
  return blob;
}

async function stampPdf(
  file: File,
  options: { text: string; opacity: number; numbers: boolean },
  onProgress?: ProgressFn,
): Promise<Uint8Array> {
  const { rgb, degrees, StandardFonts } = await loadPdfLib();
  const document = await loadPdfDoc(await fileBytes(file));
  const font = await document.embedFont(StandardFonts.HelveticaBold);
  const total = document.getPageCount();
  const text = options.text.trim().slice(0, 48);

  for (let index = 0; index < total; index += 1) {
    onProgress?.(index, total, `Stempel halaman ${index + 1}/${total}`);
    const page = document.getPage(index);
    const { width, height } = page.getSize();

    if (text) {
      const size = Math.min(64, Math.max(28, width * 0.12));
      const textWidth = font.widthOfTextAtSize(text, size);

      page.drawText(text, {
        x: width / 2 - textWidth / 2,
        y: height / 2 - size / 3,
        size,
        font,
        color: rgb(0.55, 0.12, 0.12),
        rotate: degrees(32),
        opacity: Math.min(0.45, Math.max(0.08, options.opacity)),
      });
    }

    if (options.numbers) {
      const label = String(index + 1);
      page.drawText(label, {
        x: width / 2 - font.widthOfTextAtSize(label, 10) / 2,
        y: 20,
        size: 10,
        font,
        color: rgb(0.25, 0.25, 0.25),
      });
    }

    await yieldToUi();
  }

  onProgress?.(total, total, "Menyimpan PDF...");
  return document.save();
}

export async function processTool(
  slug: ToolSlug,
  files: File[],
  options: ToolOptions,
  onProgress?: ProgressFn,
): Promise<ProcessOutput> {
  const primary = files[0];
  if (!primary) {
    fail("Pilih file dulu.");
  }

  switch (slug) {
    case "gabung": {
      const blob = await mergeAll(files, onProgress);
      return { blob, filename: "gabungan.pdf", mime: "application/pdf" };
    }

    case "pisah": {
      if (options.splitEach) {
        const blob = await splitEachPage(primary, onProgress);
        return { blob, filename: `${stem(primary.name)}-halaman.zip`, mime: "application/zip" };
      }

      const pageCount = await countPages(primary);
      const blob = await splitSelectedPages(primary, options.rangeText, pageCount, onProgress);
      return { blob, filename: `${stem(primary.name)}-pisah.pdf`, mime: "application/pdf" };
    }

    case "halaman": {
      const pageCount = await countPages(primary);
      const pageOrder = options.pageOrder ?? Array.from({ length: pageCount }, (_, index) => index);
      const blob = await manageAllPages(primary, pageOrder, onProgress);
      return { blob, filename: `${stem(primary.name)}-halaman.pdf`, mime: "application/pdf" };
    }

    case "putar": {
      const blob = await rotateAll(primary, options.rotateDegrees, onProgress);
      return { blob, filename: `${stem(primary.name)}-diputar.pdf`, mime: "application/pdf" };
    }

    case "kompres": {
      const blob = await compressPdfApi(primary, options.quality);
      return { blob, filename: `${stem(primary.name)}-kompres.pdf`, mime: "application/pdf" };
    }

    case "jpg-ke-pdf": {
      const normalized = await Promise.all(files.map(normalizeImageToJpeg));
      const blob = await jpgToPdfApi(normalized);
      return { blob, filename: "gambar.pdf", mime: "application/pdf" };
    }

    case "pdf-ke-jpg": {
      const pdfjs = await loadPdfjs();
      const data = await fileBytes(primary);
      const task = pdfjs.getDocument({ data });
      const pdf = await task.promise;
      const zip = await (async () => {
        const { default: JSZip } = await import("jszip");
        const result = new JSZip();

        try {
          for (let pageNumber = 1; pageNumber <= pdf.numPages; pageNumber += 1) {
            onProgress?.(pageNumber - 1, pdf.numPages, `Mengekspor halaman ${pageNumber}/${pdf.numPages}`);
            const page = await pdf.getPage(pageNumber);
            const viewport = page.getViewport({ scale: 1.5 });
            const canvas = document.createElement("canvas");
            canvas.width = Math.max(1, Math.floor(viewport.width));
            canvas.height = Math.max(1, Math.floor(viewport.height));
            const context = canvas.getContext("2d");
            if (!context) {
              fail("Browser tidak mendukung canvas.");
            }
            await page.render({ canvas, canvasContext: context, viewport }).promise;
            const blob = await new Promise<Blob>((resolve, reject) => {
              canvas.toBlob((value) => value ? resolve(value) : reject(new Error("Gagal membuat JPG.")), "image/jpeg", 0.88);
            });
            result.file(`${stem(primary.name)}-${String(pageNumber).padStart(3, "0")}.jpg`, blob);
            canvas.width = 0;
            canvas.height = 0;
            page.cleanup();
            await yieldToUi();
          }
          onProgress?.(pdf.numPages, pdf.numPages, "Mengemas ZIP");
          return result.generateAsync({ type: "blob" });
        } finally {
          await pdf.cleanup();
        }
      })();

      return { blob: await zip, filename: `${stem(primary.name)}-jpg.zip`, mime: "application/zip" };
    }

    case "watermark": {
      const bytes = await stampPdf(
        primary,
        {
          text: options.watermarkText,
          opacity: options.watermarkOpacity,
          numbers: options.addPageNumbers,
        },
        onProgress,
      );
      return { blob: blobFromBytes(bytes, "application/pdf"), filename: `${stem(primary.name)}-watermark.pdf`, mime: "application/pdf" };
    }

    case "word-ke-pdf": {
      const { wordToPdf } = await import("./office-convert");
      return wordToPdf(primary, onProgress);
    }

    case "excel-ke-pdf": {
      return excelToPdfApi(primary);
    }

    case "powerpoint-ke-pdf": {
      const { powerpointToPdf } = await import("./office-convert");
      return powerpointToPdf(primary, onProgress);
    }

    case "pdf-ke-word": {
      const { pdfToWord } = await import("./office-convert");
      return pdfToWord(primary, onProgress);
    }

    case "pdf-ke-excel": {
      return pdfToExcelApi(primary);
    }

    case "pdf-ke-powerpoint": {
      const { pdfToPowerpoint } = await import("./office-convert");
      return pdfToPowerpoint(primary, onProgress);
    }
  }
}
