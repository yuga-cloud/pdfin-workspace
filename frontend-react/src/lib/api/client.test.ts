import { afterEach, describe, expect, it, vi } from "vitest";

import { ApiError, postFileWithText, postMultipart } from "./client";

describe("postMultipart", () => {
  const originalFetch = globalThis.fetch;

  afterEach(() => {
    globalThis.fetch = originalFetch;
    vi.restoreAllMocks();
  });

  it("parses the backend error envelope", async () => {
    globalThis.fetch = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          error: {
            code: "invalid_multipart",
            message: "File tidak valid.",
          },
        }),
        {
          status: 400,
          headers: { "content-type": "application/json" },
        },
      ),
    );

    await expect(
      postMultipart("/rust-api/test", new FormData()),
    ).rejects.toMatchObject<ApiError>({
      name: "ApiError",
      code: "invalid_multipart",
      status: 400,
      message: "File tidak valid.",
    });
  });

  it("maps a transport failure to network_error", async () => {
    globalThis.fetch = vi.fn().mockRejectedValue(new TypeError("offline"));

    await expect(
      postMultipart("/rust-api/test", new FormData(), 1000),
    ).rejects.toMatchObject<ApiError>({
      code: "network_error",
      status: 0,
    });
  });

  it("maps an aborted request to request_timeout", async () => {
    globalThis.fetch = vi.fn(
      (_input: RequestInfo | URL, init?: RequestInit) =>
        new Promise<Response>((_, reject) => {
          init?.signal?.addEventListener("abort", () => {
            reject(new DOMException("The operation was aborted.", "AbortError"));
          });
        }),
    );

    await expect(
      postMultipart("/rust-api/test", new FormData(), 10),
    ).rejects.toMatchObject<ApiError>({
      code: "request_timeout",
      status: 0,
    });
  });

  it("rejects an oversized multipart payload before fetch", async () => {
    const fetchMock = vi.fn();
    globalThis.fetch = fetchMock;

    const formData = new FormData();
    formData.append("file", new Blob([new Uint8Array(51 * 1024 * 1024)]));

    await expect(postMultipart("/rust-api/test", formData)).rejects.toMatchObject<ApiError>({
      code: "request_too_large",
      status: 0,
    });

    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("rejects an oversized text field before fetch", async () => {
    const fetchMock = vi.fn();
    globalThis.fetch = fetchMock;

    const file = new Blob(["pdf"]);
    const oversized = "x".repeat(16 * 1024 + 1);

    await expect(
      postFileWithText("/rust-api/test", file, "text", oversized),
    ).rejects.toMatchObject<ApiError>({
      code: "field_too_large",
      status: 0,
    });

    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("rejects empty successful responses", async () => {
    globalThis.fetch = vi.fn().mockResolvedValue(
      new Response(new Blob([]), {
        status: 200,
        headers: { "content-type": "application/pdf" },
      }),
    );

    await expect(
      postMultipart("/rust-api/test", new FormData()),
    ).rejects.toMatchObject<ApiError>({
      code: "empty_response",
      status: 200,
    });
  });

  it("returns a non-empty successful response", async () => {
    const payload = new Blob(["pdf-bytes"], {
      type: "application/pdf",
    });

    globalThis.fetch = vi.fn().mockResolvedValue(
      new Response(payload, {
        status: 200,
        headers: { "content-type": "application/pdf" },
      }),
    );

    const result = await postMultipart(
      "/rust-api/test",
      new FormData(),
    );

    expect(result.type).toBe("application/pdf");
    expect(await result.text()).toBe("pdf-bytes");
  });
});
