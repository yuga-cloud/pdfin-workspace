import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

export const DEFAULT_APP_NAME = "pdfin";
export const OG_SERVICE_URL_DEFAULT = "https://og.pdfin.app";
export const OG_SITE_REL_PATH = "src/lib/og/site.json";

const SHARE_META_KEYS = new Set([
  "og:title",
  "og:description",
  "og:image",
  "og:image:width",
  "og:image:height",
  "og:type",
  "og:url",
  "og:site_name",
  "twitter:card",
  "twitter:title",
  "twitter:image",
  "twitter:description",
  "x:game:image",
  "x:game:image:width",
  "x:game:image:height",
]);

export function escapeHtml(value: unknown): string {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function unescapeHtml(value: unknown): string {
  return String(value)
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&");
}

function placeholderCardColor(site: OgSite = {}): string {
  const raw = String(site.color ?? "").trim();
  const hex = raw.startsWith("#") ? raw.slice(1) : raw;
  return /^[0-9a-fA-F]{6}$/.test(hex) ? hex : "";
}

export function appNameFromHost(hostHeader: string | null | undefined): string {
  const host = String(hostHeader ?? "")
    .split(",")[0]
    .trim()
    .split(":")[0]
    .toLowerCase();
  if (!host.endsWith(".pdfin.app") && host !== "pdfin.app") return DEFAULT_APP_NAME;
  const slug = host.split(".")[0] ?? "";
  if (!slug || slug === "www" || !/^[a-z0-9-]{1,63}$/.test(slug)) {
    return DEFAULT_APP_NAME;
  }
  return (
    slug
      .split("-")
      .filter(Boolean)
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join(" ") || DEFAULT_APP_NAME
  );
}

function isVercelSystemHost(host: string): boolean {
  return (
    host === "vercel.app" ||
    host.endsWith(".vercel.app") ||
    host === "vercel.com" ||
    host.endsWith(".vercel.com")
  );
}

export function publicAppHost(hostHeader: string | null | undefined): string {
  const host = String(hostHeader ?? "")
    .split(",")[0]
    .trim()
    .split(":")[0]
    .toLowerCase();
  if (!host || !/^[a-z0-9.-]+$/.test(host) || !host.includes(".")) return "";
  if (/^\d{1,3}(?:\.\d{1,3}){3}$/.test(host)) return "";
  if (isVercelSystemHost(host)) return "";
  return host;
}

export function resolvePublicHost(hostHeader: string | null | undefined): string {
  return publicAppHost(process.env?.VITE_PUBLIC_HOSTNAME) || publicAppHost(hostHeader);
}

export function isInstallQuery(url: string | null | undefined): boolean {
  const query = String(url ?? "").split("?", 2)[1] ?? "";
  const params = new URLSearchParams(query);
  const install = params.get("install");
  const platform = (params.get("platform") ?? "").toLowerCase();
  return (install === "1" || install === "true") && platform === "ios";
}

export function isDocumentPath(pathname: string | null | undefined): boolean {
  const path = String(pathname ?? "");
  return (
    !path.startsWith("/__app/") &&
    !path.startsWith("/api/") &&
    !path.startsWith("/@") &&
    !path.startsWith("/node_modules") &&
    !/\.[a-z0-9]+$/i.test(path)
  );
}

export function acceptsHtml(accept: string | null | undefined): boolean {
  const value = String(accept ?? "");
  return value === "" || value.includes("text/html") || value.includes("*/*");
}

export function stripInstallParams(url: string | null | undefined): string {
  const [path = "/", query = ""] = String(url ?? "/").split("?", 2);
  const params = new URLSearchParams(query);
  params.delete("install");
  params.delete("platform");
  const rest = params.toString();
  return rest ? `${path}?${rest}` : path;
}

export function renderInstallPageHtml(
  template: string,
  { host, url }: { host?: string | null; url?: string | null } = {},
): string {
  return String(template)
    .replaceAll("{{APP_NAME}}", escapeHtml(appNameFromHost(host)))
    .replaceAll("{{APP_URL}}", escapeHtml(stripInstallParams(url)));
}

export function renderWebManifest(hostHeader: string | null | undefined): string {
  const name = appNameFromHost(hostHeader);
  return JSON.stringify(
    {
      name,
      short_name: name,
      id: "/",
      start_url: "/",
      scope: "/",
      display: "standalone",
      background_color: "#000000",
      theme_color: "#000000",
      icons: [
        {
          src: "/__app/icon-180.png",
          sizes: "180x180",
          type: "image/png",
        },
      ],
    },
    null,
    2,
  );
}

export function pwaHeadTags(appName = DEFAULT_APP_NAME): Array<[string, string]> {
  return [
    ["manifest", '<link rel="manifest" href="/__app/manifest.webmanifest">'],
    ["apple-touch-icon", '<link rel="apple-touch-icon" sizes="180x180" href="/__app/icon-180.png">'],
    ["apple-mobile-web-app-capable", '<meta name="apple-mobile-web-app-capable" content="yes">'],
    ["apple-mobile-web-app-status-bar-style", '<meta name="apple-mobile-web-app-status-bar-style" content="black">'],
    ["apple-mobile-web-app-title", `<meta name="apple-mobile-web-app-title" content="${escapeHtml(appName)}">`],
    ["theme-color", '<meta name="theme-color" content="#000000">'],
  ];
}

export const EXTENSIONS_SCRIPT_SRC = "";

export function readProjectId(): string {
  return String(process.env?.VITE_PROJECT_ID ?? "").trim();
}

export function readXCreator(): string {
  return String(process.env?.X_CREATOR ?? "").trim();
}

export function readXCreatorId(): string {
  return String(process.env?.X_CREATOR_ID ?? "").trim();
}

export function xCreatorHeadTags(creator = readXCreator(), creatorId = readXCreatorId()): string[] {
  const name = String(creator ?? "").trim();
  const id = String(creatorId ?? "").trim();
  if (!name || !id) return [];
  return [
    `<meta property="x:creator" content="${escapeHtml(name)}">`,
    `<meta property="x:creator:id" content="${escapeHtml(id)}">`,
  ];
}

export function extensionsHeadTags(projectId = readProjectId()): string[] {
  if (!EXTENSIONS_SCRIPT_SRC) return [];
  const id = escapeHtml(projectId);
  const tags: string[] = [];
  if (projectId) tags.push(`<meta name="pdfin-project-id" content="${id}">`);
  tags.push(
    `<script src="${EXTENSIONS_SCRIPT_SRC}"${projectId ? ` data-project-id="${id}"` : ""} defer></script>`,
  );
  return tags;
}

export type OgSite = {
  title?: string;
  description?: string;
  type?: string;
  card?: string;
  image?: string;
  banner?: string;
  color?: string;
};

export type HeadContext = {
  appName?: string;
  projectId?: string;
  creator?: string;
  creatorId?: string;
  host?: string | null;
  cwd?: string;
  site?: OgSite;
  detectFs?: boolean;
};

export function readOgSite(cwd = process.cwd()): OgSite {
  try {
    const raw = readFileSync(join(cwd, OG_SITE_REL_PATH), "utf8");
    const parsed: unknown = JSON.parse(raw);
    return parsed && typeof parsed === "object" && !Array.isArray(parsed)
      ? (parsed as OgSite)
      : {};
  } catch {
    return {};
  }
}

export function ogCardPublicPath(cwd = process.cwd()): string {
  if (existsSync(join(cwd, "public/og.jpg"))) return "/og.jpg";
  if (existsSync(join(cwd, "public/og.png"))) return "/og.png";
  return "";
}

export function snapshotOgIdentity(cwd = process.cwd()): { site: OgSite } {
  const site = { ...readOgSite(cwd) };
  const disk = ogCardPublicPath(cwd);
  if (disk) {
    site.card = "custom";
    site.image = disk;
  } else {
    if (siteHasCustomCard(site)) delete site.card;
    if (site.image) delete site.image;
  }
  if (existsSync(join(cwd, "public/x-banner.jpg"))) {
    site.banner = site.banner || "/x-banner.jpg";
  }
  return { site };
}

export function customOgAssetPath(cwd = process.cwd()): string {
  return ogCardPublicPath(cwd) || "/og.jpg";
}

export function ogServiceUrl(): string {
  const fromEnv = String(process.env?.VITE_OG_SERVICE_URL ?? "").trim();
  return (fromEnv || OG_SERVICE_URL_DEFAULT).replace(/\/+$/, "");
}

export function titleFromDocument(html: string): string {
  const match = String(html ?? "").match(/<title\b[^>]*>([^<]*)<\/title>/i);
  return match ? unescapeHtml(match[1]).trim() : "";
}

export function resolveOgTitle(
  site: OgSite = {},
  appName = DEFAULT_APP_NAME,
  host = "",
  documentTitle = "",
): string {
  const fromSite = String(site.title ?? "").trim();
  if (fromSite) return fromSite;
  const fromDoc = String(documentTitle ?? "").trim();
  if (fromDoc) return fromDoc;
  const fromHost = appNameFromHost(host);
  if (fromHost && fromHost !== DEFAULT_APP_NAME) return fromHost;
  const fromArg = String(appName ?? "").trim();
  return fromArg || DEFAULT_APP_NAME;
}

export function siteHasCustomCard(site: OgSite = {}): boolean {
  return String(site.card ?? "").toLowerCase() === "custom";
}

export function resolveOgCardAsset(
  site: OgSite = {},
  cwd = process.cwd(),
  detectFs = true,
): string {
  const disk = detectFs ? ogCardPublicPath(cwd) : "";
  const custom = siteHasCustomCard(site) || Boolean(String(site.image ?? "").trim());
  return disk || (custom ? String(site.image ?? "").trim() || "/og.jpg" : "");
}

function applyCustomCardFromFs(site: OgSite, cwd: string): OgSite {
  const disk = ogCardPublicPath(cwd);
  if (!disk) return site;
  return { ...site, card: "custom", image: disk };
}

export function ogHeadTags({
  host = "",
  appName = DEFAULT_APP_NAME,
  site = {},
  documentTitle = "",
  cwd = process.cwd(),
  detectFs = true,
}: {
  host?: string;
  appName?: string;
  site?: OgSite;
  documentTitle?: string;
  cwd?: string;
  detectFs?: boolean;
} = {}): string[] {
  const title = resolveOgTitle(site, appName, host, documentTitle);
  const publicHost = resolvePublicHost(host);
  const tags = [
    '<meta name="twitter:card" content="summary_large_image">',
    `<meta property="og:title" content="${escapeHtml(title)}">`,
  ];
  const description = String(site.description ?? "").trim();
  if (description) tags.push(`<meta property="og:description" content="${escapeHtml(description)}">`);
  if (String(site.type ?? "").toLowerCase() === "x:game") {
    tags.push('<meta property="og:type" content="x:game">');
  }
  if (publicHost) {
    const asset = resolveOgCardAsset(site, cwd, detectFs);
    const custom = Boolean(asset);
    let image = custom
      ? `https://${publicHost}${asset.startsWith("/") ? asset : `/${asset}`}`
      : `${ogServiceUrl()}/v1/card.png?host=${encodeURIComponent(publicHost)}&title=${encodeURIComponent(title)}`;
    const color = !custom ? placeholderCardColor(site) : "";
    if (color) image += `&color=${encodeURIComponent(color)}`;
    tags.push(`<meta property="og:image" content="${escapeHtml(image)}">`);
    tags.push('<meta property="og:image:width" content="1200">');
    tags.push('<meta property="og:image:height" content="630">');
    const banner = String(site.banner ?? "").trim();
    if (banner) {
      const bannerUrl = `https://${publicHost}${banner.startsWith("/") ? banner : `/${banner}`}`;
      tags.push(`<meta property="x:game:image" content="${escapeHtml(bannerUrl)}">`);
      tags.push('<meta property="x:game:image:width" content="1200">');
      tags.push('<meta property="x:game:image:height" content="264">');
    }
  }
  return tags;
}

export function stripShareMetaTags(html: string): string {
  return String(html).replace(/<meta\b[^>]*>/gi, (tag) => {
    const attrs = [...tag.matchAll(/\b(?:property|name)\s*=\s*["']([^"']+)["']/gi)];
    for (const match of attrs) {
      if (SHARE_META_KEYS.has(String(match[1]).toLowerCase())) return "";
    }
    return tag;
  });
}

function insertAfterHeadOpen(html: string, snippet: string): string {
  if (/<head\b[^>]*>/i.test(html)) return html.replace(/<head\b[^>]*>/i, (open) => `${open}${snippet}`);
  if (/<html\b[^>]*>/i.test(html)) return html.replace(/<html\b[^>]*>/i, (open) => `${open}<head>${snippet}</head>`);
  return `<!doctype html><html><head>${snippet}</head>${html}`;
}

function insertBeforeHeadClose(html: string, snippet: string): string {
  if (/<\/head>/i.test(html)) return html.replace(/<\/head>/i, `${snippet}</head>`);
  return insertAfterHeadOpen(html, snippet);
}

export function normalizeHeadContext(ctx: HeadContext = {}): {
  appName: string;
  projectId: string;
  creator: string;
  creatorId: string;
  host: string;
  cwd: string;
  site: OgSite;
  detectFs: boolean;
} {
  const cwd = ctx.cwd ?? process.cwd();
  const site = ctx.site !== undefined ? ctx.site : applyCustomCardFromFs(snapshotOgIdentity(cwd).site, cwd);
  const appName = resolveOgTitle(site, ctx.appName ?? DEFAULT_APP_NAME, ctx.host ?? "");
  return {
    appName,
    projectId: ctx.projectId ?? readProjectId(),
    creator: ctx.creator ?? readXCreator(),
    creatorId: ctx.creatorId ?? readXCreatorId(),
    host: ctx.host ?? "",
    cwd,
    site,
    detectFs: ctx.detectFs ?? ctx.site === undefined,
  };
}

export function injectPwaHead(html: string, ctx: HeadContext = {}): string {
  if (typeof html !== "string") return html;
  const { site, projectId, creator, creatorId, host, cwd, detectFs } = normalizeHeadContext(ctx);
  const documentTitle = titleFromDocument(html);
  const appName = resolveOgTitle(site, ctx.appName ?? DEFAULT_APP_NAME, host, documentTitle);
  let next = stripShareMetaTags(html);
  const missing = pwaHeadTags(appName)
    .filter(([key]) => {
      if (key === "manifest") return !next.includes('href="/__app/manifest.webmanifest"');
      if (key === "apple-touch-icon") return !next.includes('rel="apple-touch-icon"');
      if (key === "apple-mobile-web-app-capable") return !next.includes('name="apple-mobile-web-app-capable"');
      if (key === "apple-mobile-web-app-status-bar-style") return !next.includes('name="apple-mobile-web-app-status-bar-style"');
      if (key === "apple-mobile-web-app-title") return !next.includes('name="apple-mobile-web-app-title"');
      return !next.includes(`name="${key}"`);
    })
    .map(([, tag]) => tag);

  next = insertAfterHeadOpen(
    next,
    ogHeadTags({ host, appName, site, documentTitle, cwd, detectFs }).join(""),
  );

  if (EXTENSIONS_SCRIPT_SRC && !next.includes(EXTENSIONS_SCRIPT_SRC)) {
    missing.push(...extensionsHeadTags(projectId));
  } else if (projectId && !next.includes('name="pdfin-project-id"')) {
    missing.push(`<meta name="pdfin-project-id" content="${escapeHtml(projectId)}">`);
  }
  if (projectId && !next.includes('property="pdfin:app_id"') && !next.includes("property='pdfin:app_id'")) {
    missing.push(`<meta property="pdfin:app_id" content="${escapeHtml(projectId)}">`);
  }
  const creatorTags = xCreatorHeadTags(creator, creatorId);
  if (creatorTags.length > 0) {
    const hasCreator = next.includes('property="x:creator" content=') || next.includes("property='x:creator' content=");
    if (!hasCreator) missing.push(creatorTags[0]);
    if (!next.includes('property="x:creator:id"')) missing.push(creatorTags[1]);
  }
  if (missing.length === 0) return next;
  return insertBeforeHeadClose(next, missing.join(""));
}

function findHeadClose(buf: Buffer): number {
  return buf.toString("latin1").search(/<\/head>/i);
}

export function createHeadInjector(ctx: HeadContext = {}): {
  push(chunk: Uint8Array | string): Buffer[];
  flush(): Buffer[];
} {
  const normalized = normalizeHeadContext(ctx);
  let pending: Buffer[] = [];
  let done = false;

  const apply = (html: string): string =>
    injectPwaHead(html, {
      appName: normalized.appName,
      projectId: normalized.projectId,
      creator: normalized.creator,
      creatorId: normalized.creatorId,
      host: normalized.host,
      cwd: normalized.cwd,
      site: normalized.site,
      detectFs: normalized.detectFs,
    });

  return {
    push(chunk: Uint8Array | string): Buffer[] {
      const buf = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
      if (done) return [buf];
      pending.push(buf);
      const joined = Buffer.concat(pending);
      const at = findHeadClose(joined);
      if (at === -1) return [];
      done = true;
      pending = [];
      const match = joined.toString("latin1", at).match(/^<\/head>/i);
      const closeLen = match?.[0].length ?? 7;
      const head = apply(joined.subarray(0, at + closeLen).toString("utf8"));
      return [Buffer.concat([Buffer.from(head, "utf8"), joined.subarray(at + closeLen)])];
    },
    flush(): Buffer[] {
      if (done || pending.length === 0) return [];
      const rest = Buffer.concat(pending);
      pending = [];
      done = true;
      return [Buffer.from(apply(rest.toString("utf8")), "utf8")];
    },
  };
}
