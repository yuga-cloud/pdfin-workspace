import type { ToolOptions } from "@/lib/pdf/engine";
import type { ProcessingLocation, ToolDef } from "@/shared/tools/catalog";

/**
 * Browser-side processing is deliberately bounded to avoid turning large PDF
 * jobs into excessive renderer/heap pressure.
 */
export const MAX_DEVICE_PROCESSING_BYTES = 100 * 1024 * 1024;
export const MAX_REORDER_UI_PAGES = 500;
export const MAX_SUBMISSION_FILES = 50;
export const MAX_SUBMISSION_BYTES = 500 * 1024 * 1024;
export const MAX_SERVER_CONVERSION_FILE_BYTES = 100 * 1024 * 1024;
export const MAX_SERVER_JPG_TOTAL_BYTES = 256 * 1024 * 1024;

export function effectiveProcessingLocation(
  tool: ToolDef,
  options: Pick<ToolOptions, "splitEach">,
): ProcessingLocation {
  if (tool.slug === "pisah" && options.splitEach) {
    return "device";
  }

  return tool.processing;
}

export function validateDeviceProcessingSize(
  tool: ToolDef,
  files: readonly File[],
  options: Pick<ToolOptions, "splitEach">,
): void {
  if (effectiveProcessingLocation(tool, options) !== "device") {
    return;
  }

  const oversized = files.find((file) => file.size > MAX_DEVICE_PROCESSING_BYTES);
  if (!oversized) {
    return;
  }

  throw new Error(
    `File ${oversized.name} terlalu besar untuk diproses. ` +
      `Gunakan file maksimal ${MAX_DEVICE_PROCESSING_BYTES / 1024 / 1024} MiB.`,
  );
}

export function validateSubmissionLimits(
  tool: ToolDef,
  files: readonly File[],
): void {
  if (files.length > MAX_SUBMISSION_FILES) {
    throw new Error(
      "Jumlah file dalam satu proses dibatasi " +
        MAX_SUBMISSION_FILES +
        " file.",
    );
  }

  const totalBytes = files.reduce(
    (total, file) => total + file.size,
    0,
  );

  if (totalBytes > MAX_SUBMISSION_BYTES) {
    throw new Error(
      "Total ukuran file dalam satu proses dibatasi " +
        MAX_SUBMISSION_BYTES / 1024 / 1024 +
        " MiB.",
    );
  }

  if (tool.processing !== "server" || tool.group !== "konversi") {
    return;
  }

  const oversized = files.find(
    (file) => file.size > MAX_SERVER_CONVERSION_FILE_BYTES,
  );
  if (oversized) {
    throw new Error(
      `File ${oversized.name} terlalu besar untuk konversi server-side. Gunakan file maksimal 100 MiB.`,
    );
  }

  if (tool.slug === "jpg-ke-pdf" && totalBytes > MAX_SERVER_JPG_TOTAL_BYTES) {
    throw new Error(
      "Total gambar untuk JPG ke PDF dibatasi 256 MiB.",
    );
  }
}
