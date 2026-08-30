import { afterEach, describe, expect, it, vi } from "vitest";

import { ApiError, postMultipart } from "./client";

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
