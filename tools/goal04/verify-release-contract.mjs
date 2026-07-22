import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const hasRoot = Boolean(process.argv[2] && !process.argv[2].startsWith("--"));
const root = path.resolve(hasRoot ? process.argv[2] : ".");
const args = process.argv.slice(hasRoot ? 3 : 2);
const scaffold = args.includes("--scaffold");
const release = args.includes("--release");
const requireClean = args.includes("--require-clean");
assert(scaffold !== release, "select exactly one of --scaffold or --release");
assert(args.every((arg) => ["--scaffold", "--release", "--require-clean"].includes(arg)), "unknown release verifier argument");
assert(!requireClean || release, "--require-clean is release-only");

const policy = json("policy/goal04-release-contract.json");
assert(policy.schema_revision === "starclock.goal04-release-contract-scaffold.v1", "unsupported Goal 04 release scaffold");
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
  "content-manifests/standard-universe-runtime-v1/partition-manifest.json"
]) assert(fs.statSync(path.join(root, file), { throwIfNoEntry: false })?.isFile(), `release scaffold omits ${file}`);

const dispositions = json("content-manifests/standard-universe-runtime-v1/runtime-dispositions.json");
const partitions = json("content-manifests/standard-universe-runtime-v1/partition-manifest.json");
assert(dispositions.records.length === policy.terminal_denominators.content_records, "content denominator differs");
assert(dispositions.rules.length === policy.terminal_denominators.rule_bindings, "rule denominator differs");
assert(dispositions.fixtures.length === policy.terminal_denominators.semantic_fixtures, "fixture denominator differs");
assert(partitions.partitions.length === policy.terminal_denominators.mechanic_partitions, "partition denominator differs");

run("node", ["tools/goal-hardening/verify-release-contract.mjs"]);
run("node", ["tools/agent-control/verify-goal02-release-contract.mjs"]);
run("node", ["tools/universe-reference/verify-release.mjs", "."]);

if (scaffold) {
  assert(policy.state === "Scaffold", "foundation expects a scaffold policy");
  assert(status.includes("| State | `InProgress` |"), "Goal 04 implementation is not active");
  console.log(`Goal 04 release scaffold verified (${policy.planned_phases} phases, ${policy.planned_batches} batches; no release claim).`);
} else {
  assert(policy.state === "Released", "Goal 04 release contract has not been promoted by G04-P6-B4");
  assert(status.includes("| State | `Complete` |"), "Goal 04 state is not Complete");
  assert((status.match(/^\| `G04-P[0-6]-(?:B\d+|M\d+)` \| `Complete` \|/gm) ?? []).length === policy.planned_batches, "not every Goal 04 batch is Complete");
  assert(!status.includes("- [ ]"), "Goal 04 terminal checklist is incomplete");
  assert(dispositions.records.every((record) => record.implementation_state === "Verified"), "content runtime coverage is incomplete");
  assert(dispositions.rules.every((record) => record.implementation_state === "Verified"), "rule runtime coverage is incomplete");
  assert(dispositions.fixtures.every((record) => record.implementation_state === "Verified"), "fixture runtime coverage is incomplete");
  assert(fs.statSync(path.join(root, policy.release_evidence), { throwIfNoEntry: false })?.isFile(), "release evidence is missing");
  if (requireClean) assert(capture("git", ["status", "--porcelain"]).trim() === "", "Goal 04 worktree is not clean");
  console.log(`Goal 04 release verified (${policy.planned_batches} batches${requireClean ? ", clean" : ""}).`);
}

function run(command, args) { execFileSync(command, args, { cwd: root, stdio: "ignore" }); }
function capture(command, args) { return execFileSync(command, args, { cwd: root, encoding: "utf8" }); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function assert(condition, message) { if (!condition) throw new Error(message); }
