import { requestJson } from "./client";
import type { ApiResponse, JobResponse } from "./types";

const JOBS_API = "/api/v1/jobs";

export async function getJob(jobId: string): Promise<JobResponse> {
  const payload = await requestJson<ApiResponse<JobResponse>>(
    `${JOBS_API}/${encodeURIComponent(jobId)}`,
  );

  return payload.data;
}
