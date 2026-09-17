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
export const MAX_FIELD_TEXT_LENGTH = 16 * 1024;
export const MAX_ERROR_MESSAGE_LENGTH = 2_000;

export type UploadProgressHandlers = {
  onUploadProgress?: (loaded: number, total: number) => void;
  onUploadComplete?: () => void;
};

function validateFormData(formData: FormData): void {
  for (const [, value] of formData.entries()) {
    if (!(value instanceof Blob) && value.length > MAX_FIELD_TEXT_LENGTH) {
      throw new ApiError("Nilai field multipart terlalu panjang.", {
        status: 0,
        code: "field_too_large",
      });
    }
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

function apiErrorFromPayload(
  status: number,
  payload: ApiErrorPayload,
): ApiError {
  const error = payload.error;
  const message =
    typeof error?.message === "string"
      ? error.message.slice(0, MAX_ERROR_MESSAGE_LENGTH)
      : `Request gagal (${status}).`;

  return new ApiError(message, {
    code: error?.code,
    status,
  });
}

async function parseError(response: Response): Promise<ApiError> {
  const contentType = response.headers.get("content-type") ?? "";

  if (contentType.toLowerCase().includes("application/json")) {
    try {
      const payload = (await response.json()) as ApiErrorPayload;
      return apiErrorFromPayload(response.status, payload);
    } catch {
      // Fallback ke pesan status HTTP.
    }
  }

  return new ApiError(`Request gagal (${response.status}).`, {
    status: response.status,
  });
}

async function parseXhrError(xhr: XMLHttpRequest): Promise<ApiError> {
  const contentType = xhr.getResponseHeader("content-type") ?? "";

  if (contentType.toLowerCase().includes("application/json")) {
    try {
      const response = xhr.response;
      const text =
        response instanceof Blob
          ? await response.text()
          : typeof xhr.responseText === "string"
            ? xhr.responseText
            : "";
      const payload = JSON.parse(text) as ApiErrorPayload;
      return apiErrorFromPayload(xhr.status, payload);
    } catch {
      // Fallback ke pesan status HTTP.
    }
  }

  return new ApiError(`Request gagal (${xhr.status}).`, {
    status: xhr.status,
  });
}

function postMultipartWithProgress(
  endpoint: string,
  formData: FormData,
  timeoutMs: number,
  progress: UploadProgressHandlers,
): Promise<Blob> {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();

    const rejectNetwork = () => {
      reject(new ApiError("Tidak dapat terhubung ke server.", {
        status: 0,
        code: "network_error",
      }));
    };

    try {
      xhr.open("POST", endpoint, true);
      xhr.responseType = "blob";
      xhr.timeout = timeoutMs;

      xhr.upload.onprogress = (event) => {
        if (event.lengthComputable && event.total > 0) {
          progress.onUploadProgress?.(event.loaded, event.total);
        }
      };

      xhr.upload.onload = () => {
        progress.onUploadComplete?.();
      };

      xhr.onload = async () => {
        if (xhr.status < 200 || xhr.status >= 300) {
          reject(await parseXhrError(xhr));
          return;
        }

        const blob = xhr.response;

        if (!(blob instanceof Blob) || blob.size === 0) {
          reject(new ApiError("Server mengembalikan file hasil yang kosong.", {
            status: xhr.status,
            code: "empty_response",
          }));
          return;
        }

        resolve(blob);
      };

      xhr.ontimeout = () => {
        reject(new ApiError("Request terlalu lama dan dihentikan.", {
          status: 0,
          code: "request_timeout",
        }));
      };

      xhr.onerror = rejectNetwork;
      xhr.onabort = rejectNetwork;
      xhr.send(formData);
    } catch {
      rejectNetwork();
    }
  });
}

export async function postMultipart(
  endpoint: string,
  formData: FormData,
  timeoutMs = DEFAULT_API_TIMEOUT_MS,
  progress?: UploadProgressHandlers,
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

  if (progress) {
    return postMultipartWithProgress(
      endpoint,
      formData,
      timeoutMs,
      progress,
    );
  }

  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);

  try {
    const response = await fetch(endpoint, {
      method: "POST",
      body: formData,
      signal: controller.signal,
    });

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
  } catch (error) {
    if (error instanceof ApiError) {
      throw error;
    }

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
}

export function postFile(
  endpoint: string,
  file: File | Blob,
  timeoutMs = DEFAULT_API_TIMEOUT_MS,
  progress?: UploadProgressHandlers,
): Promise<Blob> {
  const formData = new FormData();
  formData.append("file", file);
  return postMultipart(endpoint, formData, timeoutMs, progress);
}

export function postFileWithText(
  endpoint: string,
  file: File | Blob,
  fieldName: string,
  value: string,
  timeoutMs = DEFAULT_API_TIMEOUT_MS,
  progress?: UploadProgressHandlers,
): Promise<Blob> {
  validateTextField(fieldName, value);
  const formData = new FormData();
  formData.append("file", file);
  formData.append(fieldName, value);
  return postMultipart(endpoint, formData, timeoutMs, progress);
}
