import { postFile } from "./client";
import type { UploadResponse } from "./types";

const FILES_API = "/api/v1/files";

export async function uploadFile(file: File): Promise<Blob> {
  return postFile(`${FILES_API}/upload`, file);
}

export async function downloadFile(fileId: string): Promise<Blob> {
  const response = await fetch(`${FILES_API}/${encodeURIComponent(fileId)}`);

  if (!response.ok) {
    throw new Error(`Failed to fetch file (${response.status})`);
  }

  return response.blob();
}

export async function getUploadMetadata(file: File): Promise<UploadResponse> {
  return {
    fileId: crypto.randomUUID(),
    filename: file.name,
    size: file.size,
    status: "uploaded",
  };
}
