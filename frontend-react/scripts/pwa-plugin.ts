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
} from "./pwa-shared.js";

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
    const originalEnd = response.end.bind(response);
    const host = requestHost(request);
    const injector = createHeadInjector({ host, cwd });
    let mode: "inject" | "passthrough" | null = null;

    const decideMode = (): "inject" | "passthrough" => {
      if (mode) return mode;
      const contentType = String(response.getHeader("content-type") ?? "");
      const isHtml = contentType.includes("text/html");
      const encoded = Boolean(response.getHeader("content-encoding"));
      mode = isHtml && !encoded ? "inject" : "passthrough";
      if (mode === "inject" && !response.headersSent) response.removeHeader("content-length");
      return mode;
    };

    const toBuffer = (chunk: Uint8Array | string, encoding?: BufferEncoding): Buffer => {
      if (Buffer.isBuffer(chunk)) return chunk;
      if (typeof chunk === "string") return Buffer.from(chunk, encoding ?? "utf8");
      return Buffer.from(chunk);
    };

    response.write = (chunk, encoding, callback) => {
      if (decideMode() === "passthrough") return originalWrite(chunk, encoding as BufferEncoding, callback);
      const done = typeof encoding === "function" ? encoding : callback;
      if (chunk) {
        for (const output of injector.push(toBuffer(chunk, typeof encoding === "string" ? encoding : undefined))) {
          originalWrite(output);
        }
      }
      if (typeof done === "function") done();
      return true;
    };

    response.end = (chunk, encoding, callback) => {
      const done = typeof encoding === "function" ? encoding : callback;
      if (decideMode() === "passthrough") return originalEnd(chunk, encoding as BufferEncoding, callback);
      if (chunk) {
        for (const output of injector.push(toBuffer(chunk, typeof encoding === "string" ? encoding : undefined))) {
          originalWrite(output);
        }
      }
      for (const output of injector.flush()) originalWrite(output);
      return originalEnd(undefined, undefined, done);
    };

    next();
  }) as Middleware);
}

export function pwaPlugin(): Plugin {
  let root = process.cwd();

  return {
    name: "app-builder:pwa",
    configResolved(config) {
      root = config.root;
    },
    resolveId(id) {
      if (id === OG_IDENTITY_ID) return `\0${OG_IDENTITY_ID}`;
      return null;
    },
    load(id) {
      if (id !== `\0${OG_IDENTITY_ID}`) return null;
      return `export const ogIdentity = ${JSON.stringify(snapshotOgIdentity(root))};`;
    },
    transformIndexHtml(html) {
      return injectPwaHead(html, {
        host: process.env.VITE_PUBLIC_HOSTNAME ?? "",
        cwd: root,
      });
    },
    configureServer(server) {
      servePwa(server.middlewares);
      wrapHtmlResponses(server.middlewares, root);
    },
    configurePreviewServer(server) {
      servePwa(server.middlewares);
      return () => {
        wrapHtmlResponses(server.middlewares, root);
      };
    },
  };
}