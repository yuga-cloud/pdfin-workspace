#!/usr/bin/env node
import { spawn, type ChildProcess } from "node:child_process";
import process from "node:process";

const QUICK_TUNNEL_URL = /https:\/\/([a-z0-9-]+\.trycloudflare\.com)/i;
const LOCAL_PREVIEW_URL = "http://127.0.0.1:8081/";
const LOCAL_BACKEND_HEALTH_URL = "http://127.0.0.1:3000/health";
const PREVIEW_READY_TIMEOUT_MS = 15_000;
const PREVIEW_POLL_INTERVAL_MS = 100;
const tunnelArgs = [
  "tunnel",
  "--protocol",
  "auto",
  "--url",
  "http://127.0.0.1:8081",
];
const npmCommand = process.platform === "win32" ? "npm.cmd" : "npm";

let build: ChildProcess | undefined;
let tunnel: ChildProcess | undefined;
let preview: ChildProcess | undefined;
let tunnelOutputBuffer = "";
let previewStarting = false;
let shuttingDown = false;

function stopProcess(child: ChildProcess | undefined): void {
  if (!child || child.killed || child.exitCode !== null) return;
  child.kill("SIGTERM");
}

function shutdown(code = 0): void {
  if (shuttingDown) return;
  shuttingDown = true;

  stopProcess(preview);
  stopProcess(tunnel);
  stopProcess(build);

  setTimeout(() => process.exit(code), 250);
}

async function waitForHttp(url: string, timeoutMs: number): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  let lastError = "unknown error";

  while (Date.now() < deadline) {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), 2_000);

    try {
      const response = await fetch(url, {
        method: "HEAD",
        signal: controller.signal,
      });

      if (response.ok || response.status === 404) {
        return;
      }

      lastError = `HTTP ${response.status}`;
    } catch (error) {
      lastError = error instanceof Error ? error.message : String(error);
    } finally {
      clearTimeout(timer);
    }

    await new Promise((resolve) =>
      setTimeout(resolve, PREVIEW_POLL_INTERVAL_MS),
    );
  }

  throw new Error(`Timeout menunggu ${url}: ${lastError}`);
}

function spawnBuild(): Promise<number> {
  return new Promise((resolve, reject) => {
    build = spawn(npmCommand, ["run", "build"], {
      cwd: process.cwd(),
      env: process.env,
      stdio: "inherit",
      windowsHide: true,
    });

    build.once("error", reject);
    build.once("exit", (code, signal) => {
      build = undefined;

      if (signal) {
        reject(new Error(`Production build exited via ${signal}.`));
        return;
      }

      resolve(code ?? 1);
    });
  });
}

async function startPreview(publicHost: string): Promise<void> {
  if (preview || previewStarting || shuttingDown) return;

  previewStarting = true;

  console.log(
    "[pdfin] Starting production preview on loopback with an exact Host allowlist entry…",
  );

  preview = spawn(npmCommand, ["run", "preview"], {
    cwd: process.cwd(),
    env: {
      ...process.env,
      PDFIN_DEV_PUBLIC_HOST: publicHost,
    },
    stdio: "inherit",
    windowsHide: true,
  });

  preview.once("error", (error) => {
    console.error(
      "[pdfin] Failed to start production preview:",
      error.message,
    );
    shutdown(1);
  });

  preview.once("exit", (code, signal) => {
    if (shuttingDown) return;

    if (signal) {
      console.error(`[pdfin] Preview exited via ${signal}.`);
    } else if (code !== 0) {
      console.error(`[pdfin] Preview exited with code ${code ?? 1}.`);
    }

    shutdown(code ?? 1);
  });

  try {
    await waitForHttp(LOCAL_PREVIEW_URL, PREVIEW_READY_TIMEOUT_MS);
    console.log(`\n[pdfin] Public preview ready: https://${publicHost}\n`);
  } catch (error) {
    console.error(
      "[pdfin] Production preview did not become ready:",
      error instanceof Error ? error.message : String(error),
    );
    shutdown(1);
  } finally {
    previewStarting = false;
  }
}

function logTunnelOutput(chunk: Buffer | string): void {
  const text = chunk.toString();
  process.stdout.write(text);

  tunnelOutputBuffer = (tunnelOutputBuffer + text).slice(-8_192);
  const match = tunnelOutputBuffer.match(QUICK_TUNNEL_URL);

  if (!match || preview || previewStarting) return;
  void startPreview(match[1]);
}

async function main(): Promise<void> {
  console.log("[pdfin] Checking the local backend…");

  try {
    await waitForHttp(LOCAL_BACKEND_HEALTH_URL, 5_000);
  } catch (error) {
    console.error(
      "[pdfin] Backend is not ready. Start Axum on 127.0.0.1:3000 first.",
    );
    console.error(
      error instanceof Error ? error.message : String(error),
    );
    shutdown(1);
    return;
  }

  console.log("[pdfin] Building production preview…");

  try {
    const buildCode = await spawnBuild();

    if (buildCode !== 0) {
      console.error(
        `[pdfin] Production build failed with code ${buildCode}.`,
      );
      shutdown(buildCode);
      return;
    }
  } catch (error) {
    console.error(
      "[pdfin] Failed to build production preview:",
      error instanceof Error ? error.message : String(error),
    );
    shutdown(1);
    return;
  }

  console.log(
    "\n[pdfin] Starting Cloudflare Quick Tunnel (auto protocol)…",
  );
  console.log("[pdfin] This is for development/testing only.\n");

  tunnel = spawn("cloudflared", tunnelArgs, {
    cwd: process.cwd(),
    env: process.env,
    stdio: ["ignore", "pipe", "pipe"],
    windowsHide: true,
  });

  tunnel.stdout?.on("data", logTunnelOutput);
  tunnel.stderr?.on("data", logTunnelOutput);

  tunnel.once("error", (error) => {
    console.error("[pdfin] Failed to start cloudflared:", error.message);
    console.error(
      "[pdfin] Make sure `cloudflared` is installed and available on PATH.",
    );
    shutdown(1);
  });

  tunnel.once("exit", (code, signal) => {
    if (shuttingDown) return;

    if (signal) {
      console.error(`[pdfin] cloudflared exited via ${signal}.`);
    } else {
      console.error(`[pdfin] cloudflared exited with code ${code ?? 1}.`);
    }

    shutdown(code ?? 1);
  });
}

for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"] as const) {
  process.on(signal, () => shutdown(0));
}

void main().catch((error) => {
  console.error(
    "[pdfin] Public preview failed:",
    error instanceof Error ? error.message : String(error),
  );
  shutdown(1);
});
