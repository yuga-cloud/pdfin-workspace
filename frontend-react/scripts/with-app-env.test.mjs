import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import {
  mkdirSync,
  mkdtempSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
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
const PROJECT_ROOT = projectRoot();
const WRAPPER = join(PROJECT_ROOT, "scripts/with-app-env.ts");
const TSX_BIN = join(
  PROJECT_ROOT,
  "node_modules",
  ".bin",
  process.platform === "win32" ? "tsx.cmd" : "tsx",
);
const PRINT_FLAG = "process.stdout.write(String(process.env.VITE_AUTH_ENABLED));";
const DEFAULT_APP_ENV = '{"VITE_AUTH_ENABLED":"false"}';

function makeWorkspace(appEnvJson) {
  const root = mkdtempSync(join(tmpdir(), "app-env-"));
  if (appEnvJson !== undefined) {
    mkdirSync(join(root, ".app"), { recursive: true });
    writeFileSync(join(root, APP_ENV_REL_PATH), appEnvJson);
  }
  return root;
}

function testEnv(overrides = {}) {
  const env = { ...process.env };
  delete env.VITE_AUTH_ENABLED;
  return { ...env, ...overrides };
}

test("keeps VITE_-prefixed string entries", () => {
  assert.deepEqual(parseAppEnv('{"VITE_AUTH_ENABLED":"false"}'), {
    VITE_AUTH_ENABLED: "false",
  });
});

test("drops non-VITE keys, non-string values and malformed documents", () => {
  assert.deepEqual(
    parseAppEnv('{"DATABASE_URL":"postgres://x","VITE_N":1,"VITE_OK":"y"}'),
    { VITE_OK: "y" },
  );
  assert.deepEqual(parseAppEnv("not json"), {});
  assert.deepEqual(parseAppEnv('["VITE_AUTH_ENABLED"]'), {});
  assert.deepEqual(parseAppEnv("null"), {});
});

test("a missing app-env.json is a clean no-op", () => {
  assert.deepEqual(readAppEnv(makeWorkspace(undefined)), {});
});

test("reads the app env from a workspace", () => {
  const root = makeWorkspace(DEFAULT_APP_ENV);
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

test("the template default is explicit and reproducible", () => {
  assert.deepEqual(parseAppEnv(DEFAULT_APP_ENV), { VITE_AUTH_ENABLED: "false" });
});

test("vite loadEnv resolves the wrapped value", () => {
  const root = makeWorkspace(DEFAULT_APP_ENV);
  const previousEnv = process.env;
  try {
    process.env = mergeAppEnv(readAppEnv(root), testEnv());
    assert.equal(loadEnv("production", root, "VITE_").VITE_AUTH_ENABLED, "false");
  } finally {
    process.env = previousEnv;
  }
});

test("the wrapped command runs with the app env applied", async () => {
  const root = makeWorkspace(DEFAULT_APP_ENV);
  const { stdout } = await execFileAsync(
    TSX_BIN,
    [WRAPPER, process.execPath, "-e", PRINT_FLAG],
    { cwd: root, env: testEnv() },
  );
  assert.equal(stdout, "false");
});

test("the wrapped command sees an explicit override, not the fixture value", async () => {
  const root = makeWorkspace(DEFAULT_APP_ENV);
  const { stdout } = await execFileAsync(
    TSX_BIN,
    [WRAPPER, process.execPath, "-e", PRINT_FLAG],
    {
      cwd: root,
      env: testEnv({ VITE_AUTH_ENABLED: "true" }),
    },
  );
  assert.equal(stdout, "true");
});

test("the wrapper propagates the command's exit code", async () => {
  const root = makeWorkspace(DEFAULT_APP_ENV);
  await assert.rejects(
    execFileAsync(TSX_BIN, [WRAPPER, process.execPath, "-e", "process.exit(3)"], {
      cwd: root,
      env: testEnv(),
    }),
    (err) => Number(err.code) === 3,
  );
});

test("a signal-killed command is never reported as success", async () => {
  const root = makeWorkspace(DEFAULT_APP_ENV);
  await assert.rejects(
    execFileAsync(
      TSX_BIN,
      [
        WRAPPER,
        process.execPath,
        "-e",
        "process.kill(process.pid, 'SIGTERM');setTimeout(() => {}, 1000);",
      ],
      {
        cwd: root,
        env: testEnv(),
      },
    ),
    (err) =>
      err.signal === "SIGTERM" ||
      (err.code !== undefined && Number(err.code) !== 0),
  );
});

test("the CLI still runs when invoked through a symlinked path", async () => {
  const root = makeWorkspace(DEFAULT_APP_ENV);
  const linkRoot = mkdtempSync(join(tmpdir(), "app-env-link-"));
  const link = join(linkRoot, "scripts");
  symlinkSync(join(PROJECT_ROOT, "scripts"), link);
  const { stdout } = await execFileAsync(
    TSX_BIN,
    [join(link, "with-app-env.ts"), process.execPath, "-e", PRINT_FLAG],
    { cwd: root, env: testEnv() },
  );
  assert.equal(stdout, "false");
});
