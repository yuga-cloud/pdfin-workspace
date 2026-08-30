export type ApiErrorPayload = {
  error?: {
    code?: string;
    message?: string;
  };
};

export class ApiError extends Error {
  readonly code: string;
  readonly status: number;

  constructor(
    message: string,
    options: { code?: string; status: number },
  ) {
    super(message);
    this.name = "ApiError";
    this.code = options.code ?? "api_error";
    this.status = options.status;
  }
}

export const DEFAULT_API_TIMEOUT_MS = 130_000;
export const MAX_REQUEST_BODY_BYTES = 50 * 1024 * 1024;
export const MAX_FIELD_TEXT_LENGTH = 16 * 1024;
export const MAX_ERROR_MESSAGE_LENGTH = 2_000;

function validateFormData(formData: FormData): void {
  let totalBytes = 0;

  for (const [, value] of formData.entries()) {
    if (value instanceof Blob) {
      totalBytes += value.size;
    } else if (value.length > MAX_FIELD_TEXT_LENGTH) {
      throw new ApiError("Nilai field multipart terlalu panjang.", {
        status: 0,
        code: "field_too_large",
      });
    }

    if (totalBytes > MAX_REQUEST_BODY_BYTES) {
      throw new ApiError(
        `Total ukuran upload melebihi batas ${MAX_REQUEST_BODY_BYTES / 1024 / 1024} MiB.`,
        {
          status: 0,
          code: "request_too_large",
        },
      );
    }
  }
}

function validateUploadSize(file: File | Blob): void {
  if (file.size > MAX_REQUEST_BODY_BYTES) {
    throw new ApiError(
      `Ukuran file melebihi batas upload ${MAX_REQUEST_BODY_BYTES / 1024 / 1024} MiB.`,
      { status: 0, code: "file_too_large" },
    );
  }
}

function validateTextField(fieldName: string, value: string): void {
  if (value.length > MAX_FIELD_TEXT_LENGTH) {
    throw new ApiError(`Nilai field ${fieldName} terlalu panjang.`, {
      status: 0,
      code: "field_too_large",
    });
  }
}

async function parseError(response: Response): Promise<ApiError> {
  const contentType = response.headers.get("content-type") ?? "";

  if (contentType.toLowerCase().includes("application/json")) {
    try {
      const payload = (await response.json()) as ApiErrorPayload;
      const error = payload.error;
      const message =
        typeof error?.message === "string"
          ? error.message.slice(0, MAX_ERROR_MESSAGE_LENGTH)
          : `Request gagal (${response.status}).`;

      return new ApiError(message, {
        code: error?.code,
        status: response.status,
      });
    } catch {
      // Fallback ke pesan status HTTP.
    }
  }

  return new ApiError(`Request gagal (${response.status}).`, {
    status: response.status,
  });
}

export async function postMultipart(
  endpoint: string,
  formData: FormData,
  timeoutMs = DEFAULT_API_TIMEOUT_MS,
): Promise<Blob> {
  if (!endpoint.trim()) {
    throw new ApiError("Endpoint API tidak valid.", {
      status: 0,
      code: "invalid_endpoint",
    });
  }

  if (!Number.isFinite(timeoutMs) || timeoutMs <= 0) {
    throw new ApiError("Timeout API tidak valid.", {
      status: 0,
      code: "invalid_timeout",
    });
  }

  validateFormData(formData);

  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  let response: Response;

  try {
    response = await fetch(endpoint, {
      method: "POST",
      body: formData,
      signal: controller.signal,
    });
  } catch (error) {
    if (error instanceof DOMException && error.name === "AbortError") {
      throw new ApiError("Request terlalu lama dan dihentikan.", {
        status: 0,
        code: "request_timeout",
      });
    }

    throw new ApiError("Tidak dapat terhubung ke server.", {
      status: 0,
      code: "network_error",
    });
  } finally {
    clearTimeout(timer);
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
  timeoutMs = DEFAULT_API_TIMEOUT_MS,
): Promise<Blob> {
  validateUploadSize(file);
  const formData = new FormData();
  formData.append("file", file);
  return postMultipart(endpoint, formData, timeoutMs);
}

export async function postFileWithText(
  endpoint: string,
  file: File | Blob,
  fieldName: string,
  value: string,
  timeoutMs = DEFAULT_API_TIMEOUT_MS,
): Promise<Blob> {
  validateUploadSize(file);
  validateTextField(fieldName, value);
  const formData = new FormData();
  formData.append("file", file);
  formData.append(fieldName, value);
  return postMultipart(endpoint, formData, timeoutMs);
}
