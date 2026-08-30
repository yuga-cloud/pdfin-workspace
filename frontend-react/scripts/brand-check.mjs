/**
 * Brand-asset gate shared by browser-smoke.mjs (and unit-testable without a
 * browser): a canvas app is almost always a game / visually rich app, and
 * those must ship a custom share card  -  the default og.example.com placeholder is
 * not acceptable for them (see .app/skills/og/SKILL.md).
 *
 * Games must also set type=x:game in src/lib/og/site.json so the platform
 * injector emits og:type for X game-card unfurls, and public/x-banner.jpg for
 * the 50:11 X feed card. A card file is enough for bake to emit /og.jpg, but
 * brand-check still requires site.json `"card": "custom"` so the agent-facing
 * contract stays explicit.
 *
 * Checked on the filesystem (not the served head) so preview and mid-scaffold
 * workspaces are judged the same way.
 */
import { existsSync, statSync } from "node:fs";
import { join } from "node:path";
import { OG_SITE_REL_PATH, readOgSite, siteHasCustomCard } from "./pwa-shared.ts";

// Over this, link scrapers (X card previews included) time out or skip the
// image, so the guard applies to any generated custom card.
export const MAX_CARD_BYTES = 2 * 1024 * 1024;

function fileBytes(root, relativePath) {
  const path = join(root, relativePath);
  try {
    return existsSync(path) ? statSync(path).size : 0;
  } catch {
    return 0;
  }
}

function customCardPath(root) {
  const candidates = ["public/og.jpg", "public/og.png", "public/og.webp"];
  return candidates.find((candidate) => fileBytes(root, candidate) > 0) ?? null;
}

export function siteDeclaresOgTypeGame(site) {
  return String(site?.type ?? "").trim().toLowerCase() === "x:game";
}

export function computeBrandWarnings({ hasCanvas, workspaceRoot }) {
  const warnings = [];
  const sitePath = join(workspaceRoot, OG_SITE_REL_PATH);
  const site = readOgSite(workspaceRoot);
  const cardPath = customCardPath(workspaceRoot);
  const customSiteCard = siteHasCustomCard(site);

  if (hasCanvas && !customSiteCard) {
    warnings.push('BRAND WARNING: canvas apps must declare `"card": "custom"` in src/lib/og/site.json.');
  } else if (!hasCanvas && cardPath && !customSiteCard) {
    warnings.push('BRAND WARNING: custom card file found without `"card": "custom"` in src/lib/og/site.json.');
  } else if (!hasCanvas && !cardPath && !customSiteCard) {
    warnings.push("BRAND NOTE: plain utility app; no custom share card configured.");
  }

  if (cardPath && fileBytes(workspaceRoot, cardPath) > MAX_CARD_BYTES) {
    warnings.push(`BRAND WARNING: custom card exceeds ${MAX_CARD_BYTES} bytes.`);
  }

  if (siteDeclaresOgTypeGame(site) && !customSiteCard) {
    warnings.push('BRAND WARNING: `type=x:game` requires `"card": "custom"`.');
  }

  if (siteDeclaresOgTypeGame(site)) {
    const banner = fileBytes(workspaceRoot, "public/x-banner.jpg");
    if (!banner) {
      warnings.push("BRAND WARNING: x:game requires public/x-banner.jpg.");
    } else if (banner > MAX_CARD_BYTES) {
      warnings.push(`BRAND WARNING: public/x-banner.jpg exceeds ${MAX_CARD_BYTES} bytes.`);
    }
  }

  if (hasCanvas && !existsSync(sitePath)) {
    warnings.push("BRAND WARNING: canvas apps should provide src/lib/og/site.json.");
  }

  return warnings;
}
