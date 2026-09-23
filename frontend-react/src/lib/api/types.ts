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

export type FileStatus = 'uploaded' | 'ready' | 'processing' | 'failed';

export type JobStatus = 'queued' | 'running' | 'completed' | 'failed';

export interface FileMetadata {
  id: string;
  name: string;
  size: number;
  status: FileStatus;
}

export interface UploadResponse {
  file: FileMetadata;
}

export interface JobResponse {
  id: string;
  status: JobStatus;
  progress?: number;
}
