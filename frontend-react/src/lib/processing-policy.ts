import type { ToolOptions } from "@/lib/pdf/engine";
import type { ProcessingLocation, ToolDef } from "@/lib/tools-catalog";

/**
 * Browser-side processing is deliberately bounded to avoid turning large PDF
 * jobs into excessive renderer/heap pressure.
 */
export const MAX_DEVICE_PROCESSING_BYTES = 100 * 1024 * 1024;

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
    `File ${oversized.name} terlalu besar untuk pemrosesan di perangkat. ` +
      `Gunakan file maksimal ${MAX_DEVICE_PROCESSING_BYTES / 1024 / 1024} MiB atau pilih mode server jika tersedia.`,
  );
}
