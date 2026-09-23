import { requestJson } from "./client";
import type {
  ApiResponse,
  ConvertRequest,
  CreateConversionResponse,
} from "./types";

const CONVERSIONS_API = "/api/v1/conversions";

export async function createConversion(
  request: ConvertRequest,
): Promise<CreateConversionResponse> {
  const payload = await requestJson<ApiResponse<CreateConversionResponse>>(
    CONVERSIONS_API,
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
      },
      body: JSON.stringify(request),
    },
  );

  return payload.data;
}

export const conversionsApi = {
  create: createConversion,
};
