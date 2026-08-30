import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { Plugin, PreviewServer, ViteDevServer } from "vite";
import {
  acceptsHtml,
  createHeadInjector,
  injectPwaHead,
  isDocumentPath,
  isInstallQuery,
  renderInstallPageHtml,
  renderWebManifest,
  snapshotOgIdentity,
} from "./pwa-shared.ts";

export const OG_IDENTITY_ID = "virtual:og-identity";

const INSTALL_PAGE_PATH = join(dirname(fileURLToPath(import.meta.url)), "install-page.html");

type RequestLike = {
  url?: string;
  method?: string;
  headers: Record<string, string | string[] | undefined>;
};

type ResponseLike = {
  statusCode: number;
  setHeader(name: string, value: string): void;
  getHeader(name: string): string | number | string[] | undefined;
  removeHeader(name: string): void;
  headersSent: boolean;
  write(chunk: Uint8Array | string, encoding?: BufferEncoding | ((error?: Error | null) => void), callback?: (error?: Error | null) => void): boolean;
  end(chunk?: Uint8Array | string, encoding?: BufferEncoding | ((error?: Error | null) => void), callback?: (error?: Error | null) => void): ResponseLike;
};

type Next = () => void;

type MiddlewareServer = ViteDevServer | PreviewServer;

type Middleware = (req: RequestLike, res: ResponseLike, next: Next) => void;

function requestHost(req: RequestLike): string | undefined {
  const forwarded = req.headers["x-forwarded-host"];
  const host = forwarded ?? req.headers.host ?? req.headers[":authority"];
  return Array.isArray(host) ? host[0] : host;
}

export function renderInstallPage(hostHeader: string | undefined, url = "/"): string {
  const template = readFileSync(INSTALL_PAGE_PATH, "utf8");
  return renderInstallPageHtml(template, { host: hostHeader, url });
}

function sendHtml(res: ResponseLike, html: string): void {
  const body = Buffer.from(html, "utf8");
  res.statusCode = 200;
  res.setHeader("content-type", "text/html; charset=utf-8");
  res.setHeader("cache-control", "no-cache");
  res.setHeader("content-length", String(body.byteLength));
  res.end(body);
}

function servePwa(middlewares: MiddlewareServer["middlewares"]): void {
  middlewares.use(((req, res, next) => {
    const request = req as RequestLike;
    const response = res as unknown as ResponseLike;
    const rawUrl = request.url ?? "";
    const pathOnly = rawUrl.split("?", 1)[0] ?? "";
    const method = (request.method ?? "GET").toUpperCase();
    if (method !== "GET") {
      next();
      return;
    }

    if (pathOnly === "/__app/manifest.webmanifest" || pathOnly === "/__app/manifest.json") {
      const body = Buffer.from(renderWebManifest(requestHost(request)), "utf8");
      response.statusCode = 200;
      response.setHeader("content-type", "application/manifest+json; charset=utf-8");
      response.setHeader("cache-control", "no-cache");
      response.setHeader("content-length", String(body.byteLength));
      response.end(body);
      return;
    }

    if (isInstallQuery(rawUrl) && isDocumentPath(pathOnly) && acceptsHtml(headerString(request.headers.accept))) {
      try {
        sendHtml(response, renderInstallPage(requestHost(request), rawUrl));
      } catch (error) {
        console.error("[app-builder] install page missing:", error);
        response.statusCode = 500;
        response.end("install page unavailable");
      }
      return;
    }

    next();
  }) as Middleware);
}

function headerString(value: string | string[] | undefined): string | undefined {
  return Array.isArray(value) ? value[0] : value;
}

function wrapHtmlResponses(middlewares: MiddlewareServer["middlewares"], cwd: string): void {
  middlewares.use(((req, res, next) => {
    const request = req as RequestLike;
    const response = res as unknown as ResponseLike;
    const rawUrl = request.url ?? "";
    const pathOnly = rawUrl.split("?", 1)[0] ?? "";
    const method = (request.method ?? "GET").toUpperCase();
    const looksLikeDocument =
      method === "GET" &&
      headerString(request.headers.accept)?.includes("text/html") === true &&
      !isInstallQuery(rawUrl) &&
      isDocumentPath(pathOnly);
    if (!looksLikeDocument) {
      next();
      return;
    }

    const originalWrite = response.write.bind(response);
