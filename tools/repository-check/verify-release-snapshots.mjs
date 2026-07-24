import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = JSON.parse(fs.readFileSync(path.join(root, "policy/release-snapshots.json"), "utf8"));
assert(policy.schema_revision === "starclock.release-snapshots.v1", "unsupported release snapshot policy");
assert(Array.isArray(policy.goals) && policy.goals.length > 0, "release snapshot policy is empty");

const ids = new Set();
for (const goal of policy.goals) {
  assert(!ids.has(goal.goal_id), `duplicate release snapshot ${goal.goal_id}`);
  ids.add(goal.goal_id);
  for (const field of ["completion_commit", "completion_tree"])
    assert(/^[0-9a-f]{40}$/u.test(goal[field]), `${goal.goal_id}: invalid ${field}`);

  runGit(["cat-file", "-e", `${goal.completion_commit}^{commit}`]);
  const tree = captureGit(["show", "-s", "--format=%T", goal.completion_commit]).trim();
  assert(tree === goal.completion_tree, `${goal.goal_id}: completion tree differs`);

  const status = snapshotText(goal, goal.status_path);
  const releasePolicy = JSON.parse(snapshotText(goal, goal.release_policy_path));
  const evidence = JSON.parse(snapshotText(goal, goal.release_evidence_path));
  assert(status.includes("| State | `Complete` |") || status.includes("| Final state | `Complete` |"),
    `${goal.goal_id}: completion snapshot is not Complete`);
  assert(releasePolicy.goal_id === goal.goal_id, `${goal.goal_id}: release policy goal differs`);
  assert(evidence.goal_id === goal.goal_id, `${goal.goal_id}: release evidence goal differs`);
  assert(evidence.schema_revision === goal.evidence_schema_revision,
    `${goal.goal_id}: release evidence schema differs`);
  if (Object.hasOwn(evidence, "result"))
    assert(evidence.result === "complete", `${goal.goal_id}: release evidence is not complete`);
}

console.log(`Immutable release snapshots verified (${policy.goals.length} goals; current source remains evolvable).`);

function snapshotText(goal, relative) {
  assert(typeof relative === "string" && relative.length > 0 && !path.isAbsolute(relative),
    `${goal.goal_id}: invalid snapshot path`);
  return captureGit(["show", `${goal.completion_commit}:${relative}`]);
}

function runGit(args) {
  execFileSync("git", args, { cwd: root, stdio: "ignore" });
}

function captureGit(args) {
  return execFileSync("git", args, { cwd: root, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 });
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
