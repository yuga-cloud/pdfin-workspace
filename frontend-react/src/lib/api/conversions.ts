import { postFileWithText } from "./client";

const CONVERSIONS_API = "/api/v1/conversions";

export async function convertFile(
  file: File,
  format: string,
): Promise<Blob> {
  return postFileWithText(
    `${CONVERSIONS_API}/convert`,
    file,
    "format",
    format,
  );
}

export async function optimizeFile(file: File): Promise<Blob> {
  return postFileWithText(
    `${CONVERSIONS_API}/optimize`,
    file,
    "mode",
    "balanced",
  );
}
