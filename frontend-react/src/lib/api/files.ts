import { requestJson } from "./client";
import type { ApiResponse, FileMetadata, UploadResponse } from "./types";

const FILES_API = "/api/v1/files";

export async function uploadFile(file: File): Promise<UploadResponse> {
  const formData = new FormData();
  formData.append("file", file);

  const payload = await requestJson<ApiResponse<UploadResponse>>(FILES_API, {
    method: "POST",
    body: formData,
  });

  return payload.data;
}

export async function getFile(fileId: string): Promise<FileMetadata> {
  const payload = await requestJson<ApiResponse<FileMetadata>>(
    `${FILES_API}/${encodeURIComponent(fileId)}`,
  );

  return payload.data;
}
