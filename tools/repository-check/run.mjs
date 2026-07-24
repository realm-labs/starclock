import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const allowed = new Set(["--full", "--with-source-cache", "--all-rust", "--no-cache", "--no-budget"]);
const unknown = args.find((argument) => !allowed.has(argument));
if (unknown) throw new Error(`unsupported repository-check argument: ${unknown}`);

const includeSourceCache = args.includes("--with-source-cache");
const explicitFull = args.includes("--full") || includeSourceCache;
const environmentFull = process.env.CI === "true"
  || process.env.STARCLOCK_REPOSITORY_PROFILE === "full";
const full = explicitFull || environmentFull;
if (full && args.some((argument) => ["--all-rust", "--no-cache", "--no-budget"].includes(argument))) {
  throw new Error("--all-rust, --no-cache and --no-budget are quick-profile options");
}

const started = Date.now();
const steps = [];
if (full) runFull();
else runQuick();

function runQuick() {
  const budgetMs = args.includes("--no-budget")
    ? Number.POSITIVE_INFINITY
    : Number(process.env.STARCLOCK_QUICK_BUDGET_SECONDS ?? "180") * 1_000;
  assert(Number.isFinite(budgetMs) || budgetMs === Number.POSITIVE_INFINITY, "quick budget must be numeric");
  assert(budgetMs === Number.POSITIVE_INFINITY || budgetMs >= 30_000, "quick budget must be at least 30 seconds");

  const changes = changedPaths();
  const scope = rustScope(changes.paths, args.includes("--all-rust"));
  const fingerprint = rustFingerprint();
  const receiptPath = path.join(root, ".cache", "repository-check", "quick-rust-receipt.json");
  const receipt = readOptionalJson(receiptPath);
  const priorDirect = new Set(receipt?.direct_packages ?? []);
  const priorChecked = new Set([...(receipt?.direct_packages ?? []), ...(receipt?.checked_downstream_packages ?? [])]);
  const cacheHit = !args.includes("--no-cache")
    && scope.direct.length > 0
    && receipt?.schema_revision === "starclock.repository-quick-rust.v1"
    && receipt.result === "pass"
    && receipt.fingerprint === fingerprint
    && scope.direct.every((entry) => priorDirect.has(entry))
    && scope.downstream.every((entry) => priorChecked.has(entry));

  for (const command of [
    ["node", "tools/repository-check/verify-runner.mjs"],
    ["node", "tools/repository-check/verify-extension-contract.mjs"],
    ["node", "tools/dependency-policy/verify.mjs"],
    ["node", "tools/workspace/verify-dependencies.mjs"],
    ["node", "tools/repository-check/verify-source-policy.mjs"],
    ["node", "tools/repository-check/verify-native-handlers.mjs"],
    ["cargo", "fmt", "--all", "--", "--check"],
  ]) run(command, budgetMs);

  if (scope.direct.length === 0) {
    console.log("\nSKIP Rust compile/test: no workspace Rust input changed.");
  } else if (cacheHit) {
    console.log(`\nHIT  Rust quick receipt ${short(fingerprint)} (${scope.direct.length} direct package${scope.direct.length === 1 ? "" : "s"}).`);
  } else {
    const directFlags = packageFlags(scope.direct);
    const directTargets = scope.directHasLibrary ? ["--lib", "--bins", "--tests"] : ["--bins", "--tests"];
    run(["cargo", "clippy", ...directFlags, ...directTargets, "--all-features", "--", "-D", "warnings"], budgetMs);
    run(["cargo", "test", ...directFlags, ...directTargets, "--all-features"], budgetMs);
    if (scope.downstream.length > 0) {
      const downstreamTargets = scope.downstreamHasLibrary ? ["--lib", "--bins", "--tests"] : ["--bins", "--tests"];
      run(["cargo", "check", ...packageFlags(scope.downstream), ...downstreamTargets, "--all-features"], budgetMs);
    }
    writeJson(receiptPath, {
      schema_revision: "starclock.repository-quick-rust.v1",
      result: "pass",
      fingerprint,
      rustc: capture("rustc", ["-vV"]),
      direct_packages: scope.direct,
      checked_downstream_packages: scope.downstream,
      recorded_on: new Date().toISOString(),
    });
  }

  const deferred = changes.paths.filter(requiresFullGate);
  const elapsedMs = Date.now() - started;
  const report = {
    schema_revision: "starclock.repository-check-report.v1",
    profile: "quick",
    result: elapsedMs <= budgetMs ? "pass" : "budget-exceeded",
    basis: changes.basis,
    changed_paths: changes.paths,
    rust: { direct_packages: scope.direct, checked_downstream_packages: scope.downstream, cache_hit: cacheHit },
    deferred_full_gate_paths: deferred,
    elapsed_ms: elapsedMs,
    budget_ms: Number.isFinite(budgetMs) ? budgetMs : null,
    steps,
  };
  writeJson(path.join(root, ".cache", "repository-check", "last-quick-report.json"), report);
  if (deferred.length > 0) {
    console.log(`\nDEFER ${deferred.length} generated/release/CI input${deferred.length === 1 ? "" : "s"} to \`node tools/repository-check/run.mjs --full\`.`);
  }
  assert(elapsedMs <= budgetMs, `quick gate exceeded ${Math.round(budgetMs / 1_000)}s budget (${(elapsedMs / 1_000).toFixed(1)}s); inspect .cache/repository-check/last-quick-report.json`);
  console.log(`\nQuick repository checks passed in ${(elapsedMs / 1_000).toFixed(1)}s (${scope.direct.length} direct, ${scope.downstream.length} downstream, Rust cache ${cacheHit ? "hit" : "miss"}).`);
}

function runFull() {
  const commands = [
    ["node", "tools/repository-check/verify-runner.mjs"],
    ["node", "tools/repository-check/verify-extension-contract.mjs"],
    ["node", "tools/dependency-policy/verify.mjs"],
    ["node", "tools/workspace/verify-dependencies.mjs"],
    ["node", "tools/ci/verify-workflow.mjs"],
    ["node", "tools/repository-check/verify-release-snapshots.mjs"],
    ["node", "tools/goal05/verify-release-contract.mjs", ".", "--release"],
    ["node", "tools/repository-check/verify-source-policy.mjs"],
    ["node", "tools/repository-check/verify-native-handlers.mjs"],
    ["node", "tools/goal-hardening/verify-content-audits.mjs"],
    ["node", "tools/benchmark/review-phase8.mjs", "--check"],
    ["node", "tools/repository-check/verify-generated-drift.mjs", ...(includeSourceCache ? ["--with-source-cache"] : [])],
    ["node", "tools/core-kernel/verify-phase4.mjs", "--artifacts-only"],
    ["cargo", "fmt", "--all", "--", "--check"],
    ["cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings"],
    ["node", "tools/repository-check/run-workspace-tests.mjs"],
  ];
  for (const command of commands) run(command, Number.POSITIVE_INFINITY);
  const elapsedMs = Date.now() - started;
  writeJson(path.join(root, ".cache", "repository-check", "last-full-report.json"), {
    schema_revision: "starclock.repository-check-report.v1",
    profile: includeSourceCache ? "full-with-source-cache" : "full",
    result: "pass",
    elapsed_ms: elapsedMs,
    steps,
  });
  console.log(`\nFull repository checks passed in ${(elapsedMs / 1_000).toFixed(1)}s.`);
}

function changedPaths() {
  const dirty = new Set([
    ...lines(capture("git", ["diff", "--name-only", "HEAD"])),
    ...lines(capture("git", ["ls-files", "--others", "--exclude-standard"])),
  ]);
  if (dirty.size > 0) return { basis: "working-tree-vs-head", paths: [...dirty].map(normalize).sort() };
  return {
    basis: "head-vs-first-parent",
    paths: lines(capture("git", ["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"]))
      .map(normalize)
      .sort(),
  };
}

function rustScope(changes, allRust) {
  const metadata = JSON.parse(capture("cargo", ["metadata", "--format-version", "1"]));
  const members = new Set(metadata.workspace_members);
  const packages = metadata.packages.filter((entry) => members.has(entry.id));
  const byId = new Map(packages.map((entry) => [entry.id, entry]));
  const packageByPrefix = packages
    .map((entry) => ({
      id: entry.id,
      name: entry.name,
      prefix: `${normalize(path.relative(root, path.dirname(entry.manifest_path)))}/`,
    }))
    .sort((left, right) => right.prefix.length - left.prefix.length);
  const globalRust = changes.some((entry) => ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml"].includes(entry));
  const directIds = new Set();
  if (allRust || globalRust) {
    for (const entry of packages) directIds.add(entry.id);
  } else {
    for (const changed of changes) {
      if (!changed.endsWith(".rs") && !changed.endsWith("Cargo.toml")) continue;
      const owner = packageByPrefix.find((entry) => changed.startsWith(entry.prefix));
      if (owner) directIds.add(owner.id);
    }
  }

  const reverse = new Map(packages.map((entry) => [entry.id, new Set()]));
  for (const node of metadata.resolve?.nodes ?? []) {
    if (!members.has(node.id)) continue;
    for (const dependency of node.dependencies) {
      if (members.has(dependency)) reverse.get(dependency).add(node.id);
    }
  }
  const downstreamIds = new Set();
  const queue = [...directIds];
  while (queue.length > 0) {
    const dependency = queue.shift();
    for (const dependent of reverse.get(dependency) ?? []) {
      if (directIds.has(dependent) || downstreamIds.has(dependent)) continue;
      downstreamIds.add(dependent);
      queue.push(dependent);
    }
  }
  const names = (ids) => [...ids].map((id) => byId.get(id).name).sort();
  const hasLibrary = (ids) => [...ids].some((id) =>
    byId.get(id).targets.some((target) => target.kind.includes("lib"))
  );
  return {
    direct: names(directIds),
    downstream: names(downstreamIds),
    directHasLibrary: hasLibrary(directIds),
    downstreamHasLibrary: hasLibrary(downstreamIds),
  };
}

function rustFingerprint() {
  const files = [
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "tools/repository-check/run.mjs",
    ...lines(capture("git", ["ls-files", "--cached", "--others", "--exclude-standard", "--", "crates"]))
      .filter((relative) => relative.endsWith(".rs") || relative.endsWith("Cargo.toml")),
  ].map(normalize).sort();
  const hash = crypto.createHash("sha256");
  hash.update(capture("rustc", ["-vV"]));
  hash.update(capture("cargo", ["-V"]));
  for (const relative of files) {
    const absolute = path.join(root, relative);
    if (!fs.existsSync(absolute)) continue;
    hash.update(Buffer.from(relative));
    hash.update(Buffer.from([0]));
    hash.update(fs.readFileSync(absolute));
    hash.update(Buffer.from([0]));
  }
  return hash.digest("hex");
}

function requiresFullGate(relative) {
  return [
    ".github/",
    "config/",
    "content-manifests/",
    "content-reference/",
    "evidence/",
    "policy/",
    "schemas/",
    "tools/config-",
    "tools/content-reference/",
    "tools/goal",
    "tools/sora/",
    "tools/universe-",
  ].some((prefix) => relative.startsWith(prefix))
    || /\.(?:sora|xlsx)$/i.test(relative);
}

function run(command, budgetMs) {
  const elapsed = Date.now() - started;
  const remaining = Number.isFinite(budgetMs) ? Math.max(1, budgetMs - elapsed) : undefined;
  console.log(`\n==> ${command.join(" ")}`);
  const stepStarted = Date.now();
  const result = spawnSync(command[0], command.slice(1), {
    cwd: root,
    env: full ? process.env : { ...process.env, CARGO_INCREMENTAL: process.env.CARGO_INCREMENTAL ?? "1" },
    stdio: "inherit",
    timeout: remaining,
  });
  const step = { command, elapsed_ms: Date.now() - stepStarted, status: result.status };
  steps.push(step);
  if (result.error?.code === "ETIMEDOUT") {
    throw new Error(`${command[0]} exceeded the quick gate's remaining time budget`);
  }
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}

function capture(command, commandArgs) {
  const result = spawnSync(command, commandArgs, { cwd: root, encoding: "utf8" });
  if (result.error) throw result.error;
  assert(result.status === 0, `${command} ${commandArgs.join(" ")} exited ${result.status}: ${result.stderr}`);
  return result.stdout;
}

function packageFlags(packages) {
  return packages.flatMap((entry) => ["-p", entry]);
}

function lines(value) {
  return value.split(/\r?\n/).map((entry) => entry.trim()).filter(Boolean);
}

function normalize(value) {
  return value.replaceAll("\\", "/");
}

function short(value) {
  return value.slice(0, 12);
}

function readOptionalJson(file) {
  try {
    return JSON.parse(fs.readFileSync(file, "utf8"));
  } catch (error) {
    if (error.code === "ENOENT") return undefined;
    throw error;
  }
}

function writeJson(file, value) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
