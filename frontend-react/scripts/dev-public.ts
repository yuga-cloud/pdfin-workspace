#!/usr/bin/env node
import { spawn, type ChildProcess } from "node:child_process";
import process from "node:process";

const QUICK_TUNNEL_URL = /https:\/\/([a-z0-9-]+\.trycloudflare\.com)/i;
const tunnelArgs = ["tunnel", "--protocol", "http2", "--url", "http://localhost:8080"];
const npmCommand = process.platform === "win32" ? "npm.cmd" : "npm";

let tunnel: ChildProcess | undefined;
let vite: ChildProcess | undefined;
let shuttingDown = false;

function stopProcess(child: ChildProcess | undefined): void {
  if (!child || child.killed || child.exitCode !== null) return;
  child.kill("SIGTERM");
}

function shutdown(code = 0): void {
  if (shuttingDown) return;
  shuttingDown = true;
  stopProcess(vite);
  stopProcess(tunnel);
  setTimeout(() => process.exit(code), 250);
}

function logTunnelOutput(chunk: Buffer | string): void {
  const text = chunk.toString();
  process.stdout.write(text);

  const match = text.match(QUICK_TUNNEL_URL);
  if (!match || vite) return;

  const publicHost = match[1];
  console.log(`\n[pdfin] Public URL: ${match[0]}`);
  console.log("[pdfin] Starting Vite with an exact Host allowlist entry.\n");

  vite = spawn(npmCommand, ["run", "dev"], {
    cwd: process.cwd(),
    env: {
      ...process.env,
      PDFIN_DEV_PUBLIC_HOST: publicHost,
    },
    stdio: "inherit",
    windowsHide: true,
  });

  vite.once("error", (error) => {
    console.error("[pdfin] Failed to start Vite:", error.message);
    shutdown(1);
  });

  vite.once("exit", (code, signal) => {
    if (shuttingDown) return;
    if (signal) {
      console.error(`[pdfin] Vite exited via ${signal}.`);
    } else if (code !== 0) {
      console.error(`[pdfin] Vite exited with code ${code ?? 1}.`);
    }
    shutdown(code ?? 1);
  });
}

console.log("[pdfin] Starting Cloudflare Quick Tunnel (HTTP/2)…");
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
  console.error("[pdfin] Make sure `cloudflared` is installed and available on PATH.");
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

for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"] as const) {
  process.on(signal, () => shutdown(0));
}
