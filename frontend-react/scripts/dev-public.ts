#!/usr/bin/env node
import { spawn, type ChildProcess } from "node:child_process";
import process from "node:process";

const QUICK_TUNNEL_URL = /https:\/\/([a-z0-9-]+\.trycloudflare\.com)/i;
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
        reject(
          new Error(`Production build exited via ${signal}.`),
        );
        return;
      }

      resolve(code ?? 1);
    });
  });
}

function logTunnelOutput(chunk: Buffer | string): void {
  const text = chunk.toString();
  process.stdout.write(text);

  const match = text.match(QUICK_TUNNEL_URL);
  if (!match || preview) return;

  const publicHost = match[1];

  console.log(`\n[pdfin] Public URL: ${match[0]}`);
  console.log(
    "[pdfin] Serving the production build on loopback with an exact Host allowlist entry.\n",
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
      console.error(
        `[pdfin] Preview exited with code ${code ?? 1}.`,
      );
    }

    shutdown(code ?? 1);
  });
}

async function main(): Promise<void> {
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
      console.error(
        `[pdfin] cloudflared exited with code ${code ?? 1}.`,
      );
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
