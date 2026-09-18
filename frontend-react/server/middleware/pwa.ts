/**
 * Deployed-app (Nitro) half of the platform PWA chrome. Auto-registered as
 * global h3 middleware because vite.config.ts sets `serverDir: "./server"` -
 * without that option Nitro v3 never scans this directory.
 */
import installPageTemplate from "../../scripts/install-page.html?raw";
import { ogIdentity } from "virtual:og-identity";
import {
  acceptsHtml,
  createHeadInjector,
  isDocumentPath,
  isInstallQuery,
  renderInstallPageHtml,
  renderWebManifest,
} from "../../scripts/pwa-shared";

interface PwaEvent {
  url: URL;
  req: { method: string; headers: Headers };
}

function requestHost(event: PwaEvent): string {
  return (
    event.req.headers.get("x-forwarded-host") ?? event.req.headers.get("host") ?? event.url.host
  );
}

function applySecurityHeaders(response: Response): Response {
  const headers = new Headers(response.headers);

  headers.set("X-Content-Type-Options", "nosniff");
  headers.set("X-Frame-Options", "DENY");
  headers.set("Referrer-Policy", "no-referrer");
  headers.set(
    "Permissions-Policy",
    "camera=(), microphone=(), geolocation=(), payment=(), usb=()",
  );
  headers.set("Cross-Origin-Opener-Policy", "same-origin");
  headers.set("Cross-Origin-Resource-Policy", "same-origin");
  headers.set("X-Permitted-Cross-Domain-Policies", "none");
  headers.set("X-DNS-Prefetch-Control", "off");
  headers.set("Origin-Agent-Cluster", "?1");
  headers.set(
    "Content-Security-Policy",
    "default-src 'self'; base-uri 'self'; frame-ancestors 'none'; object-src 'none'; form-action 'self'; img-src 'self' data: blob:; font-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline'; worker-src 'self' blob:; connect-src 'self';",
  );

  return new Response(response.body, {
    status: response.status,
    statusText: response.statusText,
    headers,
  });
}

function injectHeadStreaming(response: Response, host: string): Response {
  const injector = createHeadInjector({
    host,
    site: ogIdentity.site,
  });
  const transformed = response.body!.pipeThrough(
    new TransformStream<Uint8Array, Uint8Array>({
      transform(chunk, controller) {
        for (const out of injector.push(chunk)) controller.enqueue(out);
      },
      flush(controller) {
        for (const out of injector.flush()) controller.enqueue(out);
      },
    }),
  );
  const headers = new Headers(response.headers);
  headers.delete("content-length");
  return new Response(transformed, {
    status: response.status,
    statusText: response.statusText,
    headers,
  });
}

export default async function pwsMiddleware(
  event: PwaEvent,
  next: () => unknown | Promise<unknown>,
): Promise<unknown> {
  const method = (event.req.method ?? "GET").toUpperCase();
  if (method !== "GET") {
    const result = await next();
    return result instanceof Response ? applySecurityHeaders(result) : result;
  }

  const path = event.url.pathname;
  const urlWithQuery = path + event.url.search;

  if (path === "/__app/manifest.webmanifest" || path === "/__app/manifest.json") {
    return applySecurityHeaders(
      new Response(renderWebManifest(requestHost(event)), {
        headers: {
          "content-type": "application/manifest+json; charset=utf-8",
          "cache-control": "no-cache",
        },
      }),
    );
  }

  if (
    isInstallQuery(urlWithQuery) &&
    isDocumentPath(path) &&
    acceptsHtml(event.req.headers.get("accept"))
  ) {
    const html = renderInstallPageHtml(installPageTemplate, {
      host: requestHost(event),
      url: urlWithQuery,
    });
    return applySecurityHeaders(
      new Response(html, {
        headers: {
          "content-type": "text/html; charset=utf-8",
          "cache-control": "no-cache",
        },
      }),
    );
  }

  if (!isDocumentPath(path)) {
    const result = await next();
    return result instanceof Response ? applySecurityHeaders(result) : result;
  }

  const result = await next();
  if (
    result instanceof Response &&
    result.body &&
    String(result.headers.get("content-type") ?? "").includes("text/html") &&
    !result.headers.get("content-encoding")
  ) {
    return applySecurityHeaders(injectHeadStreaming(result, requestHost(event)));
  }
  return result instanceof Response ? applySecurityHeaders(result) : result;
}
