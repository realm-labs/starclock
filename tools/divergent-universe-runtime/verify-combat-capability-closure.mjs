#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const relative =
  "content-manifests/divergent-universe-runtime-v1/combat-capability-closure.json";
execFileSync("node", [
  "tools/divergent-universe-runtime/generate-combat-capability-closure.mjs", "--check",
], { cwd: root, stdio: "inherit" });

const closure = json(relative);
const inventory = json(
  "content-manifests/divergent-universe-runtime-v1/capability-inventory.json",
);
assert(closure.batch === "G22-P2-B3"
  && closure.status === "CompleteNoSharedRustDelta",
"combat capability closure is not terminal");
assert(closure.summary.combat_programs === 0
  && closure.summary.combat_operation_occurrences === 0
  && closure.summary.combat_shapes === 0,
"combat capability denominator drift");
assert(equal(closure.summary.combat_shape_dispositions, {}),
  "excluded combat source left an actionable shape");
assert(closure.summary.shared_combat_missing_capabilities === 0
  && closure.summary.shared_combat_primitive_additions === 0,
"a shared combat gap or unreviewed primitive addition remains");
assert(closure.summary.mode_owned_capabilities === 0
  && closure.summary.mode_owned_shapes === 0
  && closure.summary.unresolved_shared_expression_shapes === 0,
"mode-owned or unresolved expression closure drift");
assert(inventory.missing_capabilities.every(({ capability }) =>
  !capability.startsWith("combat.")),
"shared combat gap survived G22-P2-B3");
assert(equal(
  closure.mode_owned_capabilities.map(({ capability }) => capability),
  inventory.missing_capabilities.filter(({ capability }) =>
    capability.startsWith("mode.divergent-universe.combat."))
    .map(({ capability }) => capability),
), "mode-owned combat capability assignment drift");

for (const item of [...closure.existing_shared_surfaces, ...closure.capability_probes]) {
  const source = fs.readFileSync(path.join(root, item.file), "utf8");
  for (const fragment of item.required_fragments)
    assert(source.includes(fragment), `${item.id} lost required fragment: ${fragment}`);
}

console.log(
  "Divergent Universe combat capability closure verified "
    + "(zero reachable programs/shapes/gaps; zero shared additions).",
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
