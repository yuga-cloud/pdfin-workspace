import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdirSync, mkdtempSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { promisify } from "node:util";
import { loadEnv } from "vite";
import {
  APP_ENV_REL_PATH,
  mergeAppEnv,
  parseAppEnv,
  projectRoot,
  readAppEnv,
} from "./with-app-env.ts";

const execFileAsync = promisify(execFile);
const WRAPPER = join(projectRoot(), "scripts/with-app-env.ts");
const TSX_BIN = join(
  projectRoot(),
  "node_modules",
  ".bin",
  process.platform === "win32" ? "tsx.cmd" : "tsx",
);
const PRINT_FLAG = "process.stdout.write(String(process.env.VITE_AUTH_ENABLED));";

type ProcessEnvJson = string | undefined;

type ProcessError = {
  code?: number | string;
  signal?: string;
};

function makeWorkspace(appEnvJson: ProcessEnvJson): string {
  const root = mkdtempSync(join(tmpdir(), "app-env-"));
  if (appEnvJson !== undefined) {
    mkdirSync(join(root, ".app"), { recursive: true });
    writeFileSync(join(root, APP_ENV_REL_PATH), appEnvJson);
  }
  return root;
}

test("keeps VITE_-prefixed string entries", () => {
  assert.deepEqual(parseAppEnv('{"VITE_AUTH_ENABLED":"false"}'), {
    VITE_AUTH_ENABLED: "false",
  });
});

test("drops non-VITE keys, non-string values and malformed documents", () => {
  assert.deepEqual(parseAppEnv('{"DATABASE_URL":"postgres://x","VITE_N":1,"VITE_OK":"y"}'), {
    VITE_OK: "y",
  });
  assert.deepEqual(parseAppEnv("not json"), {});
  assert.deepEqual(parseAppEnv('["VITE_AUTH_ENABLED"]'), {});
  assert.deepEqual(parseAppEnv("null"), {});
});

test("a missing app-env.json is a clean no-op", () => {
  assert.deepEqual(readAppEnv(makeWorkspace(undefined)), {});
});

test("reads the app env from a workspace", () => {
  const root = makeWorkspace('{"VITE_AUTH_ENABLED":"false"}');
  assert.deepEqual(readAppEnv(root), { VITE_AUTH_ENABLED: "false" });
});

test("an explicit process-env override wins over the file", () => {
  const merged = mergeAppEnv(
    { VITE_AUTH_ENABLED: "false" },
    { VITE_AUTH_ENABLED: "true", PATH: "/usr/bin" },
  );
  assert.equal(merged.VITE_AUTH_ENABLED, "true");
  assert.equal(merged.PATH, "/usr/bin");
});

test("the template ships auth off", () => {
  assert.deepEqual(readAppEnv(projectRoot()), { VITE_AUTH_ENABLED: "false" });
});

test("vite loadEnv resolves the wrapped value", () => {
  const root = makeWorkspace('{"VITE_AUTH_ENABLED":"false"}');
  const previousEnv = process.env;
  try {
    process.env = mergeAppEnv(readAppEnv(root), { ...process.env });
    assert.equal(loadEnv("production", root, "VITE_").VITE_AUTH_ENABLED, "false");
  } finally {
    process.env = previousEnv;
  }
});

test("the wrapped command runs with the app env applied", async () => {
  const { stdout } = await execFileAsync(TSX_BIN, [WRAPPER, process.execPath, "-e", PRINT_FLAG]);
  assert.equal(stdout, "false");
});

test("the wrapped command sees an explicit override, not the file value", async () => {
  const { stdout } = await execFileAsync(
    TSX_BIN,
    [WRAPPER, process.execPath, "-e", PRINT_FLAG],
    { env: { ...process.env, VITE_AUTH_ENABLED: "true" } },
  );
  assert.equal(stdout, "true");
});

test("the wrapper propagates the command's exit code", async () => {
  await assert.rejects(
    execFileAsync(TSX_BIN, [WRAPPER, process.execPath, "-e", "process.exit(3)"]),
    (err: ProcessError) => Number(err.code) === 3,
  );
});

test("a signal-killed command is never reported as success", async () => {
  await assert.rejects(
    execFileAsync(TSX_BIN, [
      WRAPPER,
      process.execPath,
      "-e",
      "process.kill(process.pid, 'SIGTERM');setTimeout(() => {}, 1000);",
    ]),
    (err: ProcessError) =>
      err.signal === "SIGTERM" || (err.code !== undefined && Number(err.code) !== 0),
  );
});

test("the CLI still runs when invoked through a symlinked path", async () => {
  const link = join(mkdtempSync(join(tmpdir(), "app-env-link-")), "scripts");
  symlinkSync(join(projectRoot(), "scripts"), link);
  const { stdout } = await execFileAsync(TSX_BIN, [
    join(link, "with-app-env.ts"),
    process.execPath,
    "-e",
    PRINT_FLAG,
  ]);
  assert.equal(stdout, "false");
});
