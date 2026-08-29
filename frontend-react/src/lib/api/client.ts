export type ApiErrorPayload = {
  code?: string;
  message?: string;
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

  if (
    contentType.includes("application/json")
  ) {
    try {
      const payload =
        (await response.json()) as ApiErrorPayload;

      return new ApiError(
        payload.message ??
          `Request gagal (${response.status}).`,
        {
          code: payload.code,
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
  const response = await fetch(
    endpoint,
    {
      method: "POST",
      body: formData,
    },
  );

  if (!response.ok) {
    throw await parseError(response);
  }

  return response.blob();
}

export async function postFile(
  endpoint: string,
  file: File | Blob,
): Promise<Blob> {
  const formData =
    new FormData();

  formData.append(
    "file",
    file,
  );

  return postMultipart(
    endpoint,
    formData,
  );
}

export async function postFileWithText(
  endpoint: string,
  file: File | Blob,
  fieldName: string,
  value: string,
): Promise<Blob> {
  const formData =
    new FormData();

  formData.append(
    "file",
    file,
  );

  formData.append(
    fieldName,
    value,
  );

  return postMultipart(
    endpoint,
    formData,
  );
}
