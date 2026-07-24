#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync("node", ["tools/goal07/generate-partitions.mjs", "--check"], {
  cwd: root,
  stdio: "inherit",
});
const policy = json("policy/goal07-partitions.json");
const manifest = json(
  "content-manifests/standard-universe-mechanics-complete-v1/content-partitions.json",
);
const evidence = json(
  "evidence/standard-universe-mechanics-complete-v1/phase0/partition-summary.json",
);
assert(manifest.schema_revision === "starclock.goal07-content-partitions.v1"
  && evidence.schema_revision === "starclock.goal07-partition-evidence.v1",
"Goal 07 partition revision drift");
assert(manifest.partitions.length === policy.expected_generated_batches
  && manifest.summary.total_batches === policy.expected_total_batches,
"Goal 07 partition denominator drift");
assert(JSON.stringify(manifest.summary.by_milestone)
  === JSON.stringify(policy.expected_by_milestone),
"Goal 07 milestone expansion drift");
assert(manifest.summary.assigned.records === 2201
  && manifest.summary.assigned.rules === 786
  && manifest.summary.assigned.fixtures === 78
  && manifest.summary.assigned.enemy_variants === 86
  && manifest.summary.assigned.encounter_members === 173,
"Goal 07 partition assignment drift");
assert(manifest.summary.admitted_native_handlers === 0,
  "P0 partitioning admitted a native handler");
assert(manifest.partitions.every((entry, index) =>
  entry.ordinal === index
  && (index === 0
    ? entry.dependencies.length === 0
    : JSON.stringify(entry.dependencies) === JSON.stringify([manifest.partitions[index - 1].id]))),
"Goal 07 partition order/dependency drift");
assert(Object.values(policy.contracts).every((value) => value === true),
  "Goal 07 partition policy contains an unaccepted contract");
const status = text("docs/goals/07-standard-universe-mechanics-completion-status.md");
assert(status.includes("| `G07-P0-B3` | `Complete` |"), "G07-P0-B3 is incomplete");
assert(status.includes("| Next unblocked batch | `G07-P0-B4` |"),
  "Goal 07 next batch drift");
assert(status.includes("| Concrete content sub-batches | 104 frozen"),
  "Goal 07 status omits expanded batch denominator");
console.log(
  `Goal 07 partition manifest verified (${manifest.partitions.length} generated, ` +
  `${manifest.summary.total_batches} total batches).`,
);

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
