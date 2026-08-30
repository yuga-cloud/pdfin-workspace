export type ApiErrorPayload = {
  error?: {
    code?: string;
    message?: string;
  };
};

export const API_REQUEST_TIMEOUT_MS = 130_000;

export class ApiError extends Error {
  readonly code: string;
  readonly status: number;

  constructor(
    message: string,
    options: {
      code?: string;
      status: number;
    },
  ) {
    super(message);

    this.name = "ApiError";
    this.code = options.code ?? "api_error";
    this.status = options.status;
  }
}

async function parseError(response: Response): Promise<ApiError> {
  const contentType = response.headers.get("content-type") ?? "";

  if (contentType.toLowerCase().includes("application/json")) {
    try {
      const payload = (await response.json()) as ApiErrorPayload;
      const error = payload.error;

      return new ApiError(
        error?.message ?? `Request gagal (${response.status}).`,
        {
          code: error?.code,
          status: response.status,
        },
      );
    } catch {
      // Lanjut ke error fallback.
    }
  }

  return new ApiError(`Request gagal (${response.status}).`, {
    status: response.status,
  });
}

export async function postMultipart(
  endpoint: string,
  formData: FormData,
  timeoutMs: number = API_REQUEST_TIMEOUT_MS,
): Promise<Blob> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), timeoutMs);

  let response: Response;

  try {
    response = await fetch(endpoint, {
      method: "POST",
      body: formData,
      signal: controller.signal,
    });
  } catch (error) {
    if (error instanceof DOMException && error.name === "AbortError") {
      throw new ApiError("Permintaan ke server melebihi batas waktu.", {
        status: 408,
        code: "request_timeout",
      });
    }

    throw new ApiError("Tidak dapat terhubung ke server.", {
      status: 0,
      code: "network_error",
    });
  } finally {
    clearTimeout(timeoutId);
  }

  if (!response.ok) {
    throw await parseError(response);
  }

  const blob = await response.blob();

  if (blob.size === 0) {
    throw new ApiError("Server mengembalikan file hasil yang kosong.", {
      status: response.status,
      code: "empty_response",
    });
  }

  return blob;
}

export async function postFile(
  endpoint: string,
  file: File | Blob,
  timeoutMs?: number,
): Promise<Blob> {
  const formData = new FormData();

  formData.append("file", file);

  return postMultipart(endpoint, formData, timeoutMs);
}

export async function postFileWithText(
  endpoint: string,
  file: File | Blob,
  fieldName: string,
  value: string,
  timeoutMs?: number,
): Promise<Blob> {
  const formData = new FormData();

  formData.append("file", file);
  formData.append(fieldName, value);

  return postMultipart(endpoint, formData, timeoutMs);
}
