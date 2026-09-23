import type { JobResponse } from "./types";

const JOBS_API = "/api/v1/jobs";

export async function getJob(jobId: string): Promise<JobResponse> {
  const response = await fetch(`${JOBS_API}/${encodeURIComponent(jobId)}`);

  if (!response.ok) {
    throw new Error(`Failed to fetch job (${response.status})`);
  }

  return response.json() as Promise<JobResponse>;
}

export async function cancelJob(jobId: string): Promise<void> {
  const response = await fetch(`${JOBS_API}/${encodeURIComponent(jobId)}`, {
    method: "DELETE",
  });

  if (!response.ok) {
    throw new Error(`Failed to cancel job (${response.status})`);
  }
}
