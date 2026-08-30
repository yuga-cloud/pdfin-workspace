#!/usr/bin/env node
import { spawn } from "node:child_process";
import { existsSync, readFileSync, realpathSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export const APP_ENV_REL_PATH = ".app/app-env.json";

const VITE_PREFIX = "VITE_";

type AppEnv = Record<string, string>;
type ProcessEnv = NodeJS.ProcessEnv;

export function parseAppEnv(text: string): AppEnv {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch {
    return {};
  }

  if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
    return {};
  }

  const env: AppEnv = {};
  for (const [key, value] of Object.entries(parsed)) {
    if (!key.startsWith(VITE_PREFIX) || typeof value !== "string") {
      continue;
    }
    env[key] = value;
  }
  return env;
}

export function readAppEnv(root: string): AppEnv {
  try {
    return parseAppEnv(readFileSync(join(root, APP_ENV_REL_PATH), "utf8"));
  } catch {
    return {};
  }
}

export function mergeAppEnv(appEnv: AppEnv, processEnv: ProcessEnv): ProcessEnv {
  return { ...appEnv, ...processEnv };
}

export function projectRoot(): string {
  return dirname(dirname(fileURLToPath(import.meta.url)));
}

function resolveRuntimeRoot(): string {
  const cwd = process.cwd();
  if (existsSync(join(cwd, "package.json")) || existsSync(join(cwd, APP_ENV_REL_PATH))) {
    return cwd;
  }
  return projectRoot();
}

function resolveLocalBin(command: string, root: string): string {
  const binDir = join(root, "node_modules", ".bin");
  if (process.platform === "win32") {
    const cmdPath = join(binDir, `${command}.cmd`);
    if (existsSync(cmdPath)) return cmdPath;
    const exePath = join(binDir, `${command}.exe`);
    if (existsSync(exePath)) return exePath;
  }

  const unixPath = join(binDir, command);
  return existsSync(unixPath) ? unixPath : command;
}

export function isMainModule(moduleUrl: string): boolean {
  const entry = process.argv[1];
  if (!entry) return false;

  try {
    return realpathSync(entry) === fileURLToPath(moduleUrl);
  } catch {
    return false;
  }
}

function main(argv: string[]): void {
  const [command, ...args] = argv;
  if (!command) {
    console.error("usage: tsx scripts/with-app-env.ts <command> [args…]");
    process.exit(2);
  }

  const root = resolveRuntimeRoot();
  const env = mergeAppEnv(readAppEnv(root), process.env);
  const resolved = resolveLocalBin(command, root);
  const isWin = process.platform === "win32";

  const child = spawn(resolved, args, {
    stdio: "inherit",
    env,
    shell: isWin,
    cwd: root,
  });

  for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"] as const) {
    process.on(signal, () => child.kill(signal));
  }

  child.on("error", (err: Error) => {
    console.error(`[with-app-env] failed to run ${command}:`, err.message);
    process.exit(127);
  });

  child.on("exit", (code, signal) => {
    if (signal) {
      process.removeAllListeners(signal);
      process.kill(process.pid, signal);
      return;
    }
    process.exit(code ?? 1);
  });
}

if (isMainModule(import.meta.url)) {
  main(process.argv.slice(2));
}
