import { ApiError } from "./client";
import type { FileMetadata, UploadResponse } from "./types";

const FILES_API = "/api/v1/files";

export async function uploadFile(file: File): Promise<UploadResponse> {
  const formData = new FormData();
  formData.append("file", file);

  const response = await fetch(FILES_API, {
    method: "POST",
    body: formData,
  });

  if (!response.ok) {
    throw new ApiError(`Upload file gagal (${response.status}).`, {
      status: response.status,
    });
  }

  return response.json() as Promise<UploadResponse>;
}

export async function getFile(fileId: string): Promise<FileMetadata> {
  const response = await fetch(
    `${FILES_API}/${encodeURIComponent(fileId)}`,
  );

  if (!response.ok) {
    throw new ApiError(
      `Gagal mengambil metadata file (${response.status}).`,
      { status: response.status },
    );
  }

  const payload = (await response.json()) as {
    data?: FileMetadata;
  } & FileMetadata;

  return payload.data ?? payload;
}
