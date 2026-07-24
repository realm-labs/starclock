import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = json("policy/goal06-foundation.json");
assert(policy.schema_revision === "starclock.goal06-foundation.v1",
  "unsupported Goal 06 foundation revision");
assert(policy.goal_id === "combat-identity-dynamic-assembly-v1", "Goal 06 ID differs");
assert(policy.planned_phases === 5 && policy.planned_batches === 18,
  "Goal 06 execution denominator differs");

const snapshots = json("policy/release-snapshots.json");
const required = snapshots.goals.find((goal) => goal.goal_id === policy.required_snapshot.goal_id);
assert(required, "Goal 05 immutable snapshot is missing");
for (const field of ["completion_commit", "completion_tree"])
  assert(required[field] === policy.required_snapshot[field], `Goal 05 ${field} differs`);
runGit(["cat-file", "-e", `${required.completion_commit}^{commit}`]);
assert(captureGit(["show", "-s", "--format=%T", required.completion_commit]).trim()
  === required.completion_tree, "Goal 05 completion tree differs");

const status = text("docs/goals/06-combat-identity-and-dynamic-assembly-status.md");
const batches = status.match(/^\| `G06-P[0-4]-B\d+` \|/gmu) ?? [];
assert(batches.length === policy.planned_batches, "Goal 06 status batch count differs");
assert(status.includes("| `G06-P0-B1` | `Complete` |"), "G06-P0-B1 is not Complete");
assert(status.includes("| State | `InProgress` |"), "Goal 06 is not active");

for (const file of policy.documents)
  assert(fs.statSync(path.join(root, file), { throwIfNoEntry: false })?.isFile(),
    `Goal 06 document is missing ${file}`);
const terms = Object.values(policy.identity_terms);
assert(new Set(terms).size === terms.length, "Goal 06 identity terms are not distinct");
assert(policy.starting_denominators.rule_bindings
  === policy.starting_denominators.integrated_rules
    + policy.starting_denominators.retained_rule_approximations,
"Goal 06 rule starting denominator differs");
assert(Object.values(policy.contracts).every((value) => value === true),
  "Goal 06 foundation contains an unaccepted contract");

const combatManifest = text("crates/starclock-combat/Cargo.toml")
  .split("[dependencies]")[1]
  .split("[dev-dependencies]")[0];
for (const forbidden of ["starclock-", "serde", "serde_json"])
  assert(!combatManifest.includes(forbidden), `combat-core dependency boundary contains ${forbidden}`);

console.log("Goal 06 foundation verified (5 phases, 18 batches, Goal 05 snapshot bound).");

function runGit(args) {
  execFileSync("git", args, { cwd: root, stdio: "ignore" });
}
function captureGit(args) {
  return execFileSync("git", args, { cwd: root, encoding: "utf8" });
}
function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
