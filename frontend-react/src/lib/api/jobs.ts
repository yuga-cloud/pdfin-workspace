import { ApiError } from "./client";
import type { JobResponse } from "./types";

const JOBS_API = "/api/v1/jobs";

export async function getJob(jobId: string): Promise<JobResponse> {
  const response = await fetch(
    `${JOBS_API}/${encodeURIComponent(jobId)}`,
  );

  if (!response.ok) {
    throw new ApiError(
      `Gagal mengambil job (${response.status}).`,
      { status: response.status },
    );
  }

  const payload = (await response.json()) as {
    data?: JobResponse;
  } & JobResponse;

  return payload.data ?? payload;
}
