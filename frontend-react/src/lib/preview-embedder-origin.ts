export function isAllowedEmbedderOrigin(origin: string): boolean {
  try {
    const url = new URL(origin);
    if (url.protocol !== "https:" && url.protocol !== "http:") return false;
    const host = url.hostname.toLowerCase();
    // Allow same-host or loopback (local dev)
    if (host === "localhost" || host === "127.0.0.1" || host === "[::1]") return true;
    // Allow your own domain  -  update this when deploying to production
    if (host === "pdfin.app" || host.endsWith(".pdfin.app")) return true;
    return false;
  } catch {
    return false;
  }
}

export function isSandboxPreviewGuestHost(hostname: string): boolean {
  const host = hostname.toLowerCase();
  return host === "localhost" || host === "127.0.0.1";
}

function isPreviewPair(guestHost: string, parentHost: string): boolean {
  const guest = guestHost.toLowerCase();
  const parent = parentHost.toLowerCase();
  const sep = ".preview.";
  const i = guest.indexOf(sep);
  if (i <= 0) return false;
  const label = guest.slice(0, i);
  const rest = guest.slice(i + sep.length);
  if (label.includes(".") || !rest.includes(".")) return false;
  return parent === rest;
}

export function resolveParentEmbedderOrigin(
  parentIsSelf: boolean,
  referrer: string,
  ancestorOrigin?: string | null,
  guestHostname: string = "",
): string | null {
  if (parentIsSelf) return null;
  for (const candidate of [referrer, ancestorOrigin ?? ""].filter(Boolean)) {
    try {
      const url = new URL(
        candidate.includes("://") ? candidate : `https://${candidate}`,
      );
      if (url.protocol !== "https:" && url.protocol !== "http:") continue;
      if (isAllowedEmbedderOrigin(url.origin)) return url.origin;
      if (
        isSandboxPreviewGuestHost(guestHostname) ||
        isPreviewPair(guestHostname, url.hostname)
      ) {
        return url.origin;
      }
    } catch {
      // try next candidate
    }
  }
  return null;
}

// Legacy alias  -  keep so existing imports of `isGrokEmbedderOrigin` still resolve
// without a broad find-and-replace across generated/compiled files.
export const isGrokEmbedderOrigin = isAllowedEmbedderOrigin;
