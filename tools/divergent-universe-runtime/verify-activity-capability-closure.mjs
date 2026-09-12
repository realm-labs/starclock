#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const relative =
  "content-manifests/divergent-universe-runtime-v1/activity-capability-closure.json";
execFileSync("node", [
  "tools/divergent-universe-runtime/generate-activity-capability-closure.mjs", "--check",
], { cwd: root, stdio: "inherit" });

const closure = json(relative);
const inventory = json(
  "content-manifests/divergent-universe-runtime-v1/capability-inventory.json",
);
assert(closure.batch === "G22-P2-B2"
  && closure.status === "CompleteNoSharedRustDelta",
"Activity capability closure is not terminal");
assert(closure.summary.activity_programs === 663
  && closure.summary.activity_operation_occurrences === 4_879
  && closure.summary.activity_shapes === 212,
"Activity capability denominator drift");
assert(equal(closure.summary.activity_shape_dispositions, {
  ExistingPrimitive: 56,
  MissingCapability: 63,
  NonAuthoritative: 93,
}), "Activity capability disposition drift");
assert(closure.summary.shared_activity_missing_capabilities === 0
  && closure.summary.shared_activity_primitive_additions === 0,
"a shared Activity gap or unreviewed primitive addition remains");
assert(closure.summary.mode_owned_capabilities === 42
  && closure.summary.mode_owned_shapes === 63,
"mode-owned Activity gap closure drift");
assert(inventory.missing_capabilities.every(({ capability }) =>
  !capability.startsWith("activity.")),
"shared Activity gap survived G22-P2-B2");
assert(equal(
  closure.mode_owned_capabilities.map(({ capability }) => capability),
  inventory.missing_capabilities.filter(({ capability }) =>
    capability.startsWith("mode.divergent-universe.activity."))
    .map(({ capability }) => capability),
), "mode-owned Activity capability assignment drift");

for (const item of [...closure.existing_shared_surfaces, ...closure.capability_probes]) {
  const source = fs.readFileSync(path.join(root, item.file), "utf8");
  for (const fragment of item.required_fragments)
    assert(source.includes(fragment), `${item.id} lost required fragment: ${fragment}`);
}

console.log(
  "Divergent Universe Activity capability closure verified "
    + "(663 programs; 212 shapes; 42 mode-owned gaps; zero shared additions).",
);

function json(file) {
  return JSON.parse(fs.readFileSync(path.join(root, file), "utf8"));
}

function equal(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
