export type ApiResponse<T> = {
  data: T;
  error?: never;
};

export type ApiErrorResponse = {
  data?: never;
  error: {
    code: string;
    message: string;
  };
};

export type FileStatus =
  | "uploaded"
  | "processing"
  | "ready"
  | "expired";

export type JobStatus =
  | "queued"
  | "processing"
  | "completed"
  | "failed";

export type PdfOperation =
  | "compress"
  | "merge"
  | "split"
  | "rotate"
  | "watermark"
  | "page-numbers"
  | "pdf-to-word"
  | "pdf-to-excel"
  | "pdf-to-power-point"
  | "word-to-pdf"
  | "excel-to-pdf"
  | "power-point-to-pdf"
  | "jpg-to-pdf"
  | "pdf-to-jpg";

export interface FileMetadata {
  id: string;
  filename: string;
  size: number;
  mime: string;
  status: FileStatus;
}

export interface UploadResponse {
  files: FileMetadata[];
}

export interface DownloadResponse {
  file_id: string;
  filename: string;
}

export interface JobResponse {
  job_id: string;
  status: JobStatus;
  progress: number;
}

export interface ConvertRequest {
  file_ids: string[];
  operation: PdfOperation;
}

export type CreateConversionResponse = JobResponse;
