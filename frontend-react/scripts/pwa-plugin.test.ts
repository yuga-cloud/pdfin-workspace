import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import {
  appNameFromHost,
  createHeadInjector,
  injectPwaHead,
  isDocumentPath,
  isInstallQuery,
  renderWebManifest,
  snapshotOgIdentity,
  stripInstallParams,
} from "./pwa-shared";
import { renderInstallPage } from "./pwa-plugin";

const PROJECT_ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
process.chdir(mkdtempSync(join(tmpdir(), "pdfin-pwa-test-cwd-")));

test("injects PWA metadata before </head>", () => {
  const out = injectPwaHead("<html><head><title>x</title></head><body></body></html>");
  assert.match(out, /rel="manifest"/);
  assert.match(out, /apple-touch-icon/);
  assert.match(out, /property="og:type" content="website"/);
  assert.ok(out.indexOf("manifest") < out.indexOf("</head>"));
});

test("does not duplicate injected metadata", () => {
  const once = injectPwaHead("<html><head></head></html>");
  const twice = injectPwaHead(once);
  assert.equal(once, twice);
  assert.equal(twice.split('name="twitter:card"').length - 1, 1);
  assert.equal(twice.split('property="og:type"').length - 1, 1);
});

test("custom Open Graph card is preferred", () => {
  const out = injectPwaHead("<html><head></head></html>", {
    host: "docs.pdfin.app",
    appName: "Docs",
    site: { title: "Docs", card: "custom", image: "/og.jpg" },
    detectFs: false,
  });
  assert.match(out, /https:\/\/docs\.pdfin\.app\/og\.jpg/);
});

test("placeholder Open Graph card uses the PDFin card service", () => {
  const out = injectPwaHead("<html><head></head></html>", {
    host: "docs.pdfin.app",
    appName: "Docs",
    site: { title: "Docs", color: "#FF4D2E" },
    detectFs: false,
  });
  assert.match(out, /og\.pdfin\.app\/v1\/card\.png/);
  assert.match(out, /color=FF4D2E/);
});

test("baked identity can use a public card without runtime filesystem access", () => {
  const empty = mkdtempSync(join(tmpdir(), "pdfin-og-empty-"));
  const out = injectPwaHead("<html><head></head></html>", {
    host: "docs.pdfin.app",
    cwd: empty,
    site: { title: "Docs", card: "custom", image: "/og.jpg" },
  });
  assert.match(out, /https:\/\/docs\.pdfin\.app\/og\.jpg/);
});

test("snapshotOgIdentity stamps a custom card from public assets", () => {
  const root = mkdtempSync(join(tmpdir(), "pdfin-og-snap-"));
  mkdirSync(join(root, "public"));
  writeFileSync(join(root, "public/og.jpg"), "x");
  const { site } = snapshotOgIdentity(root);
  assert.equal(site.card, "custom");
  assert.equal(site.image, "/og.jpg");
});

test("document title is escaped once", () => {
  const out = injectPwaHead("<html><head><title>Cats &amp; Dogs</title></head></html>");
  assert.match(out, /property="og:title" content="Cats &amp; Dogs"/);
  assert.doesNotMatch(out, /Cats &amp;amp; Dogs/);
});

test("injects into documents without a head element", () => {
  const out = injectPwaHead("<html><body>hi</body></html>", { appName: "Docs" });
  assert.match(out, /<head>/);
  assert.match(out, /property="og:title" content="Docs"/);
  assert.match(out, /<\/head>/);
});

test("streaming injector handles </head> split across chunks", () => {
  const injector = createHeadInjector({ appName: "Docs" });
  const chunks = [
    ...injector.push("<html><head><title>x</title></he"),
    ...injector.push("ad><body>hello</body></html>"),
  ];
  const out = Buffer.concat(chunks).toString("utf8");
  assert.match(out, /property="og:title" content="x"/);
  assert.match(out, /<body>hello<\/body>/);
});

test("streaming injector passes post-head chunks through untouched", () => {
  const injector = createHeadInjector();
  injector.push("<html><head></head>");
  const [tail] = injector.push("<body>tail</body>");
  assert.equal(tail.toString("utf8"), "<body>tail</body>");
});

test("streaming injector falls back when no </head> is seen", () => {
  const injector = createHeadInjector();
  injector.push("<html><head>");
  const out = Buffer.concat(injector.flush()).toString("utf8");
  assert.match(out, /rel="manifest"/);
});

test("detects the iOS install query", () => {
  assert.equal(isInstallQuery("/?install=1&platform=ios"), true);
  assert.equal(isInstallQuery("/app?foo=1&install=true&platform=ios"), true);
  assert.equal(isInstallQuery("/?install=1"), false);
  assert.equal(isInstallQuery("/?install=1&platform=android"), false);
  assert.equal(isInstallQuery("/"), false);
});

test("filters non-document paths", () => {
  assert.equal(isDocumentPath("/"), true);
  assert.equal(isDocumentPath("/app"), true);
  assert.equal(isDocumentPath("/api/thing"), false);
  assert.equal(isDocumentPath("/__app/install/styles.css"), false);
  assert.equal(isDocumentPath("/logo.png"), false);
});

test("strips install params from the app link", () => {
  assert.equal(stripInstallParams("/?install=1&platform=ios"), "/");
  assert.equal(stripInstallParams("/app?install=1&platform=ios&tab=2"), "/app?tab=2");
});

test("host slugs produce safe application names", () => {
  assert.equal(appNameFromHost("localhost:8080"), "pdfin");
  assert.equal(appNameFromHost("172.17.154.217:8080"), "pdfin");
  assert.equal(appNameFromHost("docs.pdfin.app"), "Docs");
  assert.equal(appNameFromHost("<script>alert(1)</script>"), "pdfin");
});

test("renders the install page with escaped host-derived values", () => {
  const html = renderInstallPage("docs.pdfin.app", "/?install=1&platform=ios");
  assert.match(html, /Add Docs to your/);
  assert.match(html, /\/__app\/install\/styles\.css/);
  assert.equal(html.includes("Powered by"), false);
  assert.equal(html.includes("{{APP_NAME}}"), false);
  assert.equal(html.includes("{{APP_URL}}"), false);

  const escaped = renderInstallPage("<script>alert(1)</script>", "/?install=1&platform=ios");
  assert.equal(escaped.includes("<script>alert(1)</script>"), false);
});

test("renders the manifest with the per-host name", () => {
  const manifest = JSON.parse(renderWebManifest("docs.pdfin.app"));
  assert.equal(manifest.name, "Docs");
  assert.equal(manifest.short_name, "Docs");
  assert.equal(manifest.icons[0].src, "/__app/icon-180.png");
});

test("Vite keeps the Nitro/PWA server wiring", () => {
  const viteConfig = readFileSync(join(PROJECT_ROOT, "vite.config.ts"), "utf8");
  assert.equal(viteConfig.includes('serverDir: "./server"'), true);
  assert.match(viteConfig, /pwaPlugin\(\)/);
});

test("PWA middleware assets are present", () => {
  const middleware = readFileSync(join(PROJECT_ROOT, "server/middleware/pwa.ts"), "utf8");
  assert.match(middleware, /install-page\.html\?raw/);
  assert.match(middleware, /virtual:og-identity/);
  readFileSync(join(PROJECT_ROOT, "scripts/install-page.html"));
  readFileSync(join(PROJECT_ROOT, "public/__app/icon-180.png"));
  readFileSync(join(PROJECT_ROOT, "public/__app/install/styles.css"));
});
