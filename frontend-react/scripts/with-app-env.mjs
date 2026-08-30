#!/usr/bin/env node
import { spawn } from "node:child_process";
import { existsSync, readFileSync, realpathSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export const APP_ENV_REL_PATH = ".app/app-env.json";
const VITE_PREFIX = "VITE_";

export function parseAppEnv(text) {
  let parsed;
  try {
    parsed = JSON.parse(text);
  } catch {
    return {};
  }

  if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
    return {};
  }

  const env = {};
  for (const [key, value] of Object.entries(parsed)) {
    if (!key.startsWith(VITE_PREFIX) || typeof value !== "string") continue;
    env[key] = value;
  }
  return env;
}

export function readAppEnv(root) {
  try {
    return parseAppEnv(readFileSync(join(root, APP_ENV_REL_PATH), "utf8"));
  } catch {
    return {};
  }
}

export function mergeAppEnv(appEnv, processEnv) {
  return { ...appEnv, ...processEnv };
}

export function projectRoot() {
  return dirname(dirname(fileURLToPath(import.meta.url)));
}

function resolveLocalBin(command, root) {
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

export function isMainModule(moduleUrl) {
  const entry = process.argv[1];
  if (!entry) return false;

  try {
    return realpathSync(entry) === fileURLToPath(moduleUrl);
  } catch {
    return false;
  }
}

function main(argv) {
  const [command, ...args] = argv;
  if (!command) {
    console.error("usage: node scripts/with-app-env.mjs <command> [args…]");
    process.exit(2);
  }

  const root = projectRoot();
  const env = mergeAppEnv(readAppEnv(root), process.env);
  const resolved = resolveLocalBin(command, root);
  const isWin = process.platform === "win32";

  const child = spawn(resolved, args, {
    stdio: "inherit",
    env,
    shell: isWin,
    cwd: root,
  });

  for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) {
    process.on(signal, () => child.kill(signal));
  }

  child.on("error", (err) => {
    console.error(`[with-app-env] failed to run ${command}:`, err.message);
    process.exit(127);
  });

  child.on("exit", (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
      return;
    }
    process.exit(code ?? 1);
  });
}

if (isMainModule(import.meta.url)) {
  main(process.argv.slice(2));
}
