import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
assert(process.argv.length === 2, "run-workspace-tests.mjs takes no arguments");
const available = os.availableParallelism?.() ?? os.cpus().length;
const jobs = Number(process.env.STARCLOCK_TEST_JOBS ?? Math.max(2, Math.min(8, Math.floor(available / 2))));
const threads = Number(process.env.STARCLOCK_TEST_THREADS ?? "1");
assert(Number.isInteger(jobs) && jobs >= 1 && jobs <= 16, "STARCLOCK_TEST_JOBS must be from 1 through 16");
assert(Number.isInteger(threads) && threads >= 1 && threads <= 16, "STARCLOCK_TEST_THREADS must be from 1 through 16");

const started = Date.now();
const buildStarted = Date.now();
const build = spawnSync("cargo", [
  "test", "--workspace", "--all-targets", "--all-features", "--no-run", "--message-format=json",
], {
  cwd: root,
  encoding: "utf8",
  maxBuffer: 64 * 1024 * 1024,
  stdio: ["ignore", "pipe", "inherit"],
});
if (build.error) throw build.error;
assert(build.status === 0, `workspace test build exited ${build.status}`);
const buildMs = Date.now() - buildStarted;
const executables = [...new Set(build.stdout
  .split(/\r?\n/)
  .filter(Boolean)
  .map(parseJson)
  .filter((entry) => entry?.reason === "compiler-artifact" && entry.profile?.test && entry.executable)
  .map((entry) => path.resolve(entry.executable)))]
  .sort();
assert(executables.length >= 80, `expected at least 80 workspace test harnesses, found ${executables.length}`);

console.log(`Built ${executables.length} test harnesses in ${(buildMs / 1_000).toFixed(1)}s; executing with ${jobs} processes x ${threads} test threads.`);
const executionStarted = Date.now();
let cursor = 0;
const results = [];
await Promise.all(Array.from({ length: jobs }, async () => {
  while (cursor < executables.length) {
    const executable = executables[cursor];
    cursor += 1;
    results.push(await execute(executable));
  }
}));

const failures = results.filter((entry) => entry.status !== 0);
for (const failure of failures) {
  console.error(`\nFAIL ${failure.name} (${(failure.elapsed_ms / 1_000).toFixed(1)}s)`);
  if (failure.stdout) console.error(failure.stdout);
  if (failure.stderr) console.error(failure.stderr);
}
assert(failures.length === 0, `${failures.length} workspace test harness${failures.length === 1 ? "" : "es"} failed`);

const docsStarted = Date.now();
const docs = spawnSync("cargo", ["test", "--workspace", "--doc", "--all-features"], {
  cwd: root,
  stdio: "inherit",
});
if (docs.error) throw docs.error;
assert(docs.status === 0, `workspace doctests exited ${docs.status}`);
const docsMs = Date.now() - docsStarted;
const executionMs = docsStarted - executionStarted;
const elapsedMs = Date.now() - started;
const slowest = [...results].sort((left, right) => right.elapsed_ms - left.elapsed_ms).slice(0, 12);
const report = {
  schema_revision: "starclock.workspace-test-run.v1",
  result: "pass",
  jobs,
  test_threads_per_process: threads,
  harnesses: results.length,
  build_ms: buildMs,
  execution_ms: executionMs,
  doctest_ms: docsMs,
  elapsed_ms: elapsedMs,
  slowest_harnesses: slowest,
};
const reportPath = path.join(root, ".cache", "repository-check", "workspace-test-timings.json");
fs.mkdirSync(path.dirname(reportPath), { recursive: true });
fs.writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
console.log(`Workspace tests passed: ${results.length} harnesses in ${(elapsedMs / 1_000).toFixed(1)}s; timings written to ${path.relative(root, reportPath).replaceAll("\\", "/")}.`);
for (const entry of slowest.slice(0, 5)) {
  console.log(`  ${(entry.elapsed_ms / 1_000).toFixed(1)}s  ${entry.name}`);
}

function execute(executable) {
  return new Promise((resolve, reject) => {
    const began = Date.now();
    const child = spawn(executable, ["--quiet", "--test-threads", String(threads)], {
      cwd: root,
      env: { ...process.env, RUST_BACKTRACE: process.env.RUST_BACKTRACE ?? "1" },
      windowsHide: true,
    });
    const stdout = [];
    const stderr = [];
    child.stdout.on("data", (chunk) => stdout.push(chunk));
    child.stderr.on("data", (chunk) => stderr.push(chunk));
    child.on("error", reject);
    child.on("close", (status) => {
      const now = Date.now();
      resolve({
        name: path.basename(executable),
        executable: path.relative(root, executable).replaceAll("\\", "/"),
        status,
        elapsed_ms: now - began,
        stdout: Buffer.concat(stdout).toString("utf8").trim(),
        stderr: Buffer.concat(stderr).toString("utf8").trim(),
      });
    });
  });
}

function parseJson(line) {
  try {
    return JSON.parse(line);
  } catch {
    return undefined;
  }
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
