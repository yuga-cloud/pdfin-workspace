import { ApiError } from "./client";
import type { ConvertRequest, CreateConversionResponse } from "./types";

const CONVERSIONS_API = "/api/v1/conversions";

export async function createConversion(
  request: ConvertRequest,
): Promise<CreateConversionResponse> {
  const response = await fetch(CONVERSIONS_API, {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    throw new ApiError(
      `Gagal membuat conversion (${response.status}).`,
      { status: response.status },
    );
  }

  const payload = (await response.json()) as
    | { data?: CreateConversionResponse }
    | CreateConversionResponse;

  return "data" in payload && payload.data ? payload.data : payload;
}

export const conversionsApi = {
  create: createConversion,
};
