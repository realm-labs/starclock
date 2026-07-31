import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const quick = args.includes("--quick");
const packages = [];
for (let index = 0; index < args.length; index += 1) {
  const argument = args[index];
  if (argument === "--quick") continue;
  assert(argument === "--package" && nonEmpty(args[index + 1]),
    `unsupported run-workspace-tests argument: ${argument}`);
  packages.push(args[index + 1]);
  index += 1;
}
assert(quick === (packages.length > 0),
  "--quick requires at least one --package and package selection requires --quick");
assert(new Set(packages).size === packages.length, "duplicate selected test package");
const available = os.availableParallelism?.() ?? os.cpus().length;
// The workspace has five coarse integration suites. Favor test-level parallelism
// inside those suites and retain a second process for unit/CLI harness overlap.
const defaultJobs = Math.max(1, Math.min(2, Math.ceil(available / 4)));
const jobs = Number(process.env.STARCLOCK_TEST_JOBS ?? defaultJobs);
const threads = Number(process.env.STARCLOCK_TEST_THREADS ?? Math.max(1, Math.floor(available / jobs)));
const automaticScheduling = process.env.STARCLOCK_TEST_JOBS === undefined
  && process.env.STARCLOCK_TEST_THREADS === undefined;
const exclusiveThreads = Math.max(1, Math.min(16, available));
assert(Number.isInteger(jobs) && jobs >= 1 && jobs <= 16, "STARCLOCK_TEST_JOBS must be from 1 through 16");
assert(Number.isInteger(threads) && threads >= 1 && threads <= 16, "STARCLOCK_TEST_THREADS must be from 1 through 16");

const started = Date.now();
const buildStarted = Date.now();
const quickSuites = quick ? suitesFor(packages) : [];
const selectedPackages = quickSuites.length > 0
  ? [...new Set([...packages, "starclock-test-kit"])]
  : packages;
const selection = quick
  ? selectedPackages.flatMap((entry) => ["--package", entry])
  : ["--workspace"];
const cliTests = packages.includes("starclock-cli")
  ? ["cli_contract", "mcp_stdio", "standard_replay_smoke", "universe_cli", "workspace_boundaries"]
  : [];
const targets = quick
  ? [
      "--lib",
      "--bins",
      ...quickSuites.flatMap((entry) => ["--test", entry]),
      ...cliTests.flatMap((entry) => ["--test", entry]),
    ]
  : ["--all-targets"];
const build = spawnSync("cargo", [
  "test", ...selection, ...targets, "--all-features", "--no-run", "--message-format=json",
], {
  cwd: root,
  encoding: "utf8",
  maxBuffer: 64 * 1024 * 1024,
  stdio: ["ignore", "pipe", "inherit"],
});
if (build.error) throw build.error;
assert(build.status === 0, `${quick ? "selected" : "workspace"} test build exited ${build.status}`);
const buildMs = Date.now() - buildStarted;
const executables = [...new Set(build.stdout
  .split(/\r?\n/)
  .filter(Boolean)
  .map(parseJson)
  .filter((entry) => entry?.reason === "compiler-artifact" && entry.profile?.test && entry.executable)
  .map((entry) => path.resolve(entry.executable)))]
  .sort((left, right) => harnessWeight(right) - harnessWeight(left) || left.localeCompare(right));
assert(
  quick ? executables.length >= packages.length : executables.length >= 20,
  `expected ${quick ? "at least one harness per selected package" : "at least 20 workspace test harnesses"}, found ${executables.length}`,
);

const exclusiveExecutables = automaticScheduling
  ? executables.filter((entry) => harnessWeight(entry) >= 80)
  : [];
const sharedExecutables = executables.filter((entry) => !exclusiveExecutables.includes(entry));
console.log(`Built ${executables.length} ${quick ? "selected" : "workspace"} test harnesses in ${(buildMs / 1_000).toFixed(1)}s; executing ${exclusiveExecutables.length} memory-heavy harness${exclusiveExecutables.length === 1 ? "" : "es"} exclusively with ${exclusiveThreads} threads, then ${sharedExecutables.length} harnesses with ${jobs} processes x ${threads} threads.`);
const executionStarted = Date.now();
const results = [];
for (const executable of exclusiveExecutables)
  results.push(await execute(executable, exclusiveThreads));
let cursor = 0;
await Promise.all(Array.from({ length: jobs }, async () => {
  while (cursor < sharedExecutables.length) {
    const executable = sharedExecutables[cursor];
    cursor += 1;
    results.push(await execute(executable, threads));
  }
}));

const failures = results.filter((entry) => entry.status !== 0);
for (const failure of failures) {
  console.error(`\nFAIL ${failure.name} (${(failure.elapsed_ms / 1_000).toFixed(1)}s)`);
  if (failure.stdout) console.error(failure.stdout);
  if (failure.stderr) console.error(failure.stderr);
}
assert(failures.length === 0, `${failures.length} ${quick ? "selected" : "workspace"} test harness${failures.length === 1 ? "" : "es"} failed`);

const docsStarted = Date.now();
if (!quick) {
  const docs = spawnSync("cargo", ["test", "--workspace", "--doc", "--all-features"], {
    cwd: root,
    stdio: "inherit",
  });
  if (docs.error) throw docs.error;
  assert(docs.status === 0, `workspace doctests exited ${docs.status}`);
}
const docsMs = Date.now() - docsStarted;
const executionMs = docsStarted - executionStarted;
const elapsedMs = Date.now() - started;
const slowest = [...results].sort((left, right) => right.elapsed_ms - left.elapsed_ms).slice(0, 12);
const report = {
  schema_revision: "starclock.workspace-test-run.v1",
  result: "pass",
  scope: quick ? "selected-packages" : "workspace",
  packages,
  jobs,
  test_threads_per_process: threads,
  exclusive_harnesses: exclusiveExecutables.length,
  exclusive_test_threads: exclusiveThreads,
  harnesses: results.length,
  build_ms: buildMs,
  execution_ms: executionMs,
  doctest_ms: docsMs,
  elapsed_ms: elapsedMs,
  slowest_harnesses: slowest,
};
const reportPath = path.join(
  root,
  ".cache",
  "repository-check",
  quick ? "quick-test-timings.json" : "workspace-test-timings.json",
);
fs.mkdirSync(path.dirname(reportPath), { recursive: true });
fs.writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
console.log(`${quick ? "Selected package" : "Workspace"} tests passed: ${results.length} harnesses in ${(elapsedMs / 1_000).toFixed(1)}s; timings written to ${path.relative(root, reportPath).replaceAll("\\", "/")}.`);
for (const entry of slowest.slice(0, 5)) {
  console.log(`  ${(entry.elapsed_ms / 1_000).toFixed(1)}s  ${entry.name}`);
}

function execute(executable, testThreads) {
  return new Promise((resolve, reject) => {
    const began = Date.now();
    const child = spawn(executable, ["--quiet", "--test-threads", String(testThreads)], {
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

function harnessWeight(executable) {
  const name = path.basename(executable);
  if (name.startsWith("universe_suite-")) return 100;
  if (name.startsWith("adapter_suite-")) return 90;
  if (name.startsWith("starclock_mode_universe-")) return 80;
  if (name.startsWith("exhaustive_suite-")) return 70;
  if (name.startsWith("combat_suite-")) return 60;
  if (name.startsWith("activity_suite-")) return 50;
  return 0;
}

function suitesFor(selectedPackages) {
  const suites = new Set();
  for (const packageName of selectedPackages) {
    for (const suite of ({
      "starclock-activity": ["activity_suite"],
      "starclock-agent-api": ["adapter_suite"],
      "starclock-build": ["combat_suite"],
      "starclock-combat": ["combat_suite"],
      "starclock-data": ["combat_suite"],
      "starclock-mcp": ["adapter_suite"],
      "starclock-mode-standard": ["combat_suite"],
      "starclock-mode-universe": ["universe_suite"],
      "starclock-replay": ["activity_suite"],
      "starclock-rules": ["combat_suite"],
      "starclock-test-kit": [
        "activity_suite",
        "adapter_suite",
        "combat_suite",
        "exhaustive_suite",
        "universe_suite",
      ],
    })[packageName] ?? []) suites.add(suite);
  }
  return [...suites].sort();
}

function nonEmpty(value) {
  return typeof value === "string" && value.length > 0;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
