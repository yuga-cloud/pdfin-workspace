export type JobStatus =
  | "queued"
  | "running"
  | "completed"
  | "failed";

export interface JobItem {
  id: string;
  status: JobStatus;
}
