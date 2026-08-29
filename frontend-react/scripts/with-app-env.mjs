#!/usr/bin/env node
/**
 * Run a command with optional app-env.json merged into its environment.
 *
 * `dev`, `build` and `preview` all route through this wrapper so the dev
 * server, the built bundle and the preview server agree on VITE_* flags.
 *
 * Only VITE_-prefixed keys are honored. A real process.env entry always wins.
 */
import { spawn } from "node:child_process";
import { existsSync, readFileSync, realpathSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export const APP_ENV_REL_PATH = ".app/app-env.json";

const VITE_PREFIX = "VITE_";

/**
 * Parse an app-env document, keeping only VITE_-prefixed string entries.
 */
export function parseAppEnv(text) {
  let parsed;
  try {
    parsed = JSON.parse(text);
  } catch {
    return {};
  }
  if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) return {};
  const env = {};
  for (const [key, value] of Object.entries(parsed)) {
    if (!key.startsWith(VITE_PREFIX)) continue;
    if (typeof value !== "string") continue;
    env[key] = value;
  }
  return env;
}

/** The app env recorded under `root`, or `{}` when the file is absent. */
export function readAppEnv(root) {
  try {
    return parseAppEnv(readFileSync(join(root, APP_ENV_REL_PATH), "utf8"));
  } catch {
    return {};
  }
}

/** File values under the process environment: an explicit override wins. */
export function mergeAppEnv(appEnv, processEnv) {
  return { ...appEnv, ...processEnv };
}

/** The workspace root (this file lives in `<root>/scripts/`). */
export function projectRoot() {
  return dirname(dirname(fileURLToPath(import.meta.url)));
}

/**
 * Resolve a local node_modules/.bin binary so spawn works on Windows
 * (where `vite` alone is not on PATH and the file is `vite.cmd`).
 */
function resolveLocalBin(command, root) {
  const binDir = join(root, "node_modules", ".bin");
  if (process.platform === "win32") {
    const cmdPath = join(binDir, `${command}.cmd`);
    if (existsSync(cmdPath)) return cmdPath;
    const exePath = join(binDir, `${command}.exe`);
    if (existsSync(exePath)) return exePath;
  }
  const unixPath = join(binDir, command);
  if (existsSync(unixPath)) return unixPath;
  return command; // fall back to PATH
}

/**
 * Whether `moduleUrl` is the script node was asked to run.
 */
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
  // On Windows, .cmd shims need shell:true (or spawn via cmd.exe).
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
    console.error(`[with-app-env] failed to run ${command}:`, err?.message || err);
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
