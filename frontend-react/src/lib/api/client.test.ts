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
    ).rejects.toMatchObject({
      name: "ApiError",
      code: "invalid_multipart",
      status: 400,
      message: "File tidak valid.",
    } satisfies Partial<ApiError>);
  });

  it("preserves backend rate-limit errors", async () => {
    globalThis.fetch = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          error: {
            code: "rate_limited",
            message: "Terlalu banyak request.",
          },
        }),
        {
          status: 429,
          headers: {
            "content-type": "application/json",
            "retry-after": "1",
          },
        },
      ),
    );

    await expect(
      postMultipart("/rust-api/test", new FormData()),
    ).rejects.toMatchObject({
      code: "rate_limited",
      status: 429,
      message: "Terlalu banyak request.",
    } satisfies Partial<ApiError>);
  });

  it("maps a transport failure to network_error", async () => {
    globalThis.fetch = vi.fn().mockRejectedValue(new TypeError("offline"));

    await expect(
      postMultipart("/rust-api/test", new FormData(), 1000),
    ).rejects.toMatchObject({
      code: "network_error",
      status: 0,
    } satisfies Partial<ApiError>);
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
    ).rejects.toMatchObject({
      code: "request_timeout",
      status: 0,
    } satisfies Partial<ApiError>);
  });

  it("keeps the timeout active while reading a slow response body", async () => {
    let abortHandler: (() => void) | undefined;
    const bodyPromise = new Promise<Response>((_resolve, reject) => {
      abortHandler = () => {
        reject(new DOMException("The operation was aborted.", "AbortError"));
      };
    });

    globalThis.fetch = vi.fn((_input: RequestInfo | URL, init?: RequestInit) => {
      init?.signal?.addEventListener("abort", () => abortHandler?.());
      return bodyPromise;
    });

    await expect(
      postMultipart("/rust-api/test", new FormData(), 10),
    ).rejects.toMatchObject({
      code: "request_timeout",
      status: 0,
    } satisfies Partial<ApiError>);
  });

  it("rejects an oversized text field before fetch", async () => {
    const fetchMock = vi.fn();
    globalThis.fetch = fetchMock;

    const file = new Blob(["pdf"]);
    const oversized = "x".repeat(16 * 1024 + 1);

    await expect(
      postFileWithText("/rust-api/test", file, "text", oversized),
    ).rejects.toMatchObject({
      code: "field_too_large",
      status: 0,
    } satisfies Partial<ApiError>);

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
    ).rejects.toMatchObject({
      code: "empty_response",
      status: 200,
    } satisfies Partial<ApiError>);
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
