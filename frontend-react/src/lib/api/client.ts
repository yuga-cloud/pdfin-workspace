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

async function parseError(
  response: Response,
): Promise<ApiError> {
  const contentType =
    response.headers.get("content-type") ?? "";

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

  return new ApiError(
    `Request gagal (${response.status}).`,
    {
      status: response.status,
    },
  );
}

export async function postMultipart(
  endpoint: string,
  formData: FormData,
): Promise<Blob> {
  let response: Response;

  try {
    response = await fetch(endpoint, {
      method: "POST",
      body: formData,
    });
  } catch {
    throw new ApiError("Tidak dapat terhubung ke server.", {
      status: 0,
      code: "network_error",
    });
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
): Promise<Blob> {
  const formData = new FormData();

  formData.append("file", file);

  return postMultipart(endpoint, formData);
}

export async function postFileWithText(
  endpoint: string,
  file: File | Blob,
  fieldName: string,
  value: string,
): Promise<Blob> {
  const formData = new FormData();

  formData.append("file", file);
  formData.append(fieldName, value);

  return postMultipart(endpoint, formData);
}
