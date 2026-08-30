/**
 * Brand-asset gate shared by browser-smoke.mjs (and unit-testable without a
 * browser): a canvas app is almost always a game / visually rich app, and
 * those must ship a custom share card. Games must also declare x:game in
 * src/lib/og/site.json and provide the X feed card.
 */
import { existsSync, statSync } from "node:fs";
import { join } from "node:path";
import { OG_SITE_REL_PATH, readOgSite, siteHasCustomCard } from "./pwa-shared.ts";

export const MAX_CARD_BYTES = 600 * 1024;

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
  const gameType = siteDeclaresOgTypeGame(site);

  if (hasCanvas && !cardPath) {
    warnings.push(
      'BRAND WARNING: public/og.jpg is missing; canvas apps are not done without a custom share card.',
    );
  } else if (hasCanvas && !customSiteCard) {
    warnings.push(
      'BRAND WARNING: canvas apps must declare `"card": "custom"` in src/lib/og/site.json.',
    );
  } else if (!hasCanvas && cardPath && !customSiteCard) {
    warnings.push(
      'BRAND WARNING: custom card file found without `"card": "custom"` in src/lib/og/site.json.',
    );
  } else if (!hasCanvas && !cardPath && !customSiteCard) {
    warnings.push("BRAND NOTE: plain utility app; no custom share card configured.");
  }

  if (cardPath && fileBytes(workspaceRoot, cardPath) > MAX_CARD_BYTES) {
    warnings.push(`BRAND WARNING: custom card is over 600 KB (${MAX_CARD_BYTES} bytes).`);
  }

  if (hasCanvas && !gameType) {
    warnings.push(
      'BRAND WARNING: canvas apps must declare `type": "x:game"` in src/lib/og/site.json.',
    );
  }

  if (gameType) {
    const banner = fileBytes(workspaceRoot, "public/x-banner.jpg");
    if (!banner) {
      warnings.push("BRAND WARNING: x:game requires public/x-banner.jpg.");
    } else if (banner > MAX_CARD_BYTES) {
      warnings.push("BRAND WARNING: x-banner.jpg is over 600 KB.");
    }
  }

  if (hasCanvas && !existsSync(sitePath)) {
    warnings.push("BRAND WARNING: canvas apps should provide src/lib/og/site.json.");
  }

  return warnings;
}
