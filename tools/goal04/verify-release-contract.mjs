import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const hasRoot = Boolean(process.argv[2] && !process.argv[2].startsWith("--"));
const root = path.resolve(hasRoot ? process.argv[2] : ".");
const args = process.argv.slice(hasRoot ? 3 : 2);
const scaffold = args.includes("--scaffold");
const release = args.includes("--release");
const bless = args.includes("--bless");
const requireClean = args.includes("--require-clean");
const artifactOnly = process.env.STARCLOCK_ARTIFACT_CHECK_ONLY === "1";
assert(scaffold !== release, "select exactly one of --scaffold or --release");
assert(args.every((arg) => ["--scaffold", "--release", "--bless", "--require-clean"].includes(arg)), "unknown release verifier argument");
assert(!bless || release, "--bless is release-only");
assert(!requireClean || release, "--require-clean is release-only");

const policy = json("policy/goal04-release-contract.json");
assert(["starclock.goal04-release-contract-scaffold.v1", "starclock.goal04-release-contract.v1"].includes(policy.schema_revision), "unsupported Goal 04 release contract");
const status = text("docs/goals/04-standard-universe-runtime-status.md");
const batches = status.match(/^\| `G04-P[0-6]-(?:B\d+|M\d+)` \|/gm) ?? [];
assert(batches.length === policy.planned_batches, "Goal 04 batch denominator differs");
assert((status.match(/^\| Phase [0-6].*\|/gm) ?? []).length === policy.planned_phases, "Goal 04 phase denominator differs");

for (const file of [
  "docs/goals/04-standard-universe-runtime.md",
  "docs/goals/04-standard-universe-runtime-prompt.md",
  "docs/25-standard-universe-runtime-design.md",
  "policy/goal04-surface-audit.json",
  "policy/goal04-interface-contract.json",
  "policy/goal04-runtime-dispositions.json",
  "policy/goal04-foundation.json",
  "content-manifests/standard-universe-runtime-v1/runtime-dispositions.json",
  "content-manifests/standard-universe-runtime-v1/partition-manifest.json",
]) assert(fileExists(file), `release contract omits ${file}`);

const dispositions = json("content-manifests/standard-universe-runtime-v1/runtime-dispositions.json");
const partitions = json("content-manifests/standard-universe-runtime-v1/partition-manifest.json");
assert(dispositions.records.length === policy.terminal_denominators.content_records, "content denominator differs");
assert(dispositions.rules.length === policy.terminal_denominators.rule_bindings, "rule denominator differs");
assert(dispositions.fixtures.length === policy.terminal_denominators.semantic_fixtures, "fixture denominator differs");
assert(partitions.partitions.length === policy.terminal_denominators.mechanic_partitions, "partition denominator differs");

if (!artifactOnly) {
  run("node", ["tools/goal-hardening/verify-release-contract.mjs"]);
  run("node", ["tools/agent-control/verify-goal02-release-contract.mjs"]);
  run("node", ["tools/universe-reference/verify-release.mjs", "."]);
}

if (scaffold) {
  if (policy.state === "Scaffold") {
    assert(status.includes("| State | `InProgress` |"), "Goal 04 implementation is not active");
    console.log(`Goal 04 release scaffold verified (${policy.planned_phases} phases, ${policy.planned_batches} batches; no release claim).`);
  } else {
    assert(policy.state === "Released", "unknown Goal 04 release state");
    console.log(`Goal 04 released structure verified (${policy.planned_phases} phases, ${policy.planned_batches} batches).`);
  }
  process.exit(0);
}

assert(policy.schema_revision === "starclock.goal04-release-contract.v1" && policy.state === "Released", "Goal 04 release contract has not been promoted");
assert(status.includes("| State | `Complete` |"), "Goal 04 state is not Complete");
assert((status.match(/^\| `G04-P[0-6]-(?:B\d+|M\d+)` \| `Complete` \|/gm) ?? []).length === policy.planned_batches, "not every Goal 04 batch is Complete");
assert(!status.includes("- [ ]"), "Goal 04 terminal checklist is incomplete");
assert(dispositions.records.every((record) => record.implementation_state === "Executable"), "content runtime coverage is incomplete");
assert(dispositions.rules.every((record) => record.implementation_state === "Executable"), "rule runtime coverage is incomplete");
assert(dispositions.fixtures.every((record) => record.implementation_state === "Executable"), "fixture runtime coverage is incomplete");
for (const collection of [policy.evidence_files, policy.documentation_files]) {
  assert(new Set(collection).size === collection.length, "release reference inventory contains duplicates");
  for (const file of collection) assert(fileExists(file), `release reference is missing ${file}`);
}

if (!artifactOnly) {
  run("node", ["tools/goal04/verify-mechanic-fixtures.mjs", "."]);
  run("node", ["tools/goal04/verify-runtime-completeness.mjs", "."]);
  run("node", ["tools/goal04/verify-seeded-matrix.mjs", "."]);
  run("node", ["tools/goal04/verify-determinism-hardening.mjs", "."]);
  run("node", ["tools/goal04/verify-universe-performance.mjs", "."]);
  run("node", ["tools/goal04/verify-universe-audits.mjs", "."]);
}

const seeded = json("evidence/standard-universe-runtime-v1/hardening/seeded-matrix.json");
const performance = json("evidence/standard-universe-runtime-v1/performance/stable-runner.json");
const mcp = json("evidence/standard-universe-runtime-v1/interfaces/activity-mcp.json");
const catalog = json("evidence/standard-universe-runtime-v1/catalog/bootstrap.json");
assert(seeded.matrix.coverage.complete_runs === policy.terminal_denominators.complete_seeded_runs, "seeded run denominator differs");
assert(mcp.activity_tools.length === policy.terminal_denominators.mcp_activity_tools, "Activity MCP tool denominator differs");
assert(performance.samples.length === 3 && performance.medians.length === 6, "performance release evidence differs");

const report = {
  schema_revision: "starclock.goal04-release-evidence.v1",
  goal_id: policy.goal_id,
  released_on: policy.released_on,
  result: "complete",
  policy_sha256: sha256("policy/goal04-release-contract.json"),
  completion: { phases: policy.planned_phases, batches: policy.planned_batches, release_batch: policy.release_batch },
  runtime: {
    revisions: policy.runtime_revisions,
    composed_configuration_sha256: catalog.digests.composed_configuration,
    content_records: dispositions.records.length,
    rule_bindings: dispositions.rules.length,
    semantic_fixtures: dispositions.fixtures.length,
    mechanic_partitions: partitions.partitions.length,
  },
  seeded_matrix: {
    worlds: seeded.matrix.coverage.worlds,
    paths: seeded.matrix.coverage.distinct_path_options,
    difficulties: seeded.matrix.coverage.difficulties,
    complete_runs: seeded.matrix.coverage.complete_runs,
    evidence_sha256: sha256("evidence/standard-universe-runtime-v1/hardening/seeded-matrix.json"),
  },
  interfaces: {
    activity_mcp_tools: mcp.activity_tools.length,
    activity_mcp_resources: mcp.activity_resources.length,
    replay: "complete-fresh-verification",
    cli: ["universe config validate", "universe coverage", "universe run", "replay verify"],
  },
  performance: {
    samples: performance.samples.length,
    workloads: performance.medians.length,
    stable_runner: performance.runner.id,
    evidence_sha256: sha256("evidence/standard-universe-runtime-v1/performance/stable-runner.json"),
  },
  evidence_sha256: Object.fromEntries(policy.evidence_files.map((file) => [file, sha256(file)])),
  documentation_sha256: Object.fromEntries(policy.documentation_files.map((file) => [file, sha256(file)])),
  prior_contracts: policy.required_prior_contracts,
  clean_checkout_command: policy.clean_checkout_command,
};
const output = `${JSON.stringify(report, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, policy.release_evidence)), { recursive: true });
  fs.writeFileSync(path.join(root, policy.release_evidence), output);
} else {
  assert(fileExists(policy.release_evidence), "release evidence is missing; run with --bless");
  assert(text(policy.release_evidence).replaceAll("\r\n", "\n") === output, "Goal 04 release evidence is stale; run with --bless");
}
if (requireClean) assert(capture("git", ["status", "--porcelain"]) === "", "Goal 04 worktree is not clean");
console.log(`Goal 04 release verified (${policy.planned_batches} batches${requireClean ? ", clean" : ""}).`);

function run(command, args) { execFileSync(command, args, { cwd: root, stdio: "ignore" }); }
function capture(command, args) { return execFileSync(command, args, { cwd: root, encoding: "utf8" }).trim(); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function fileExists(relative) { return fs.statSync(path.join(root, relative), { throwIfNoEntry: false })?.isFile(); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
