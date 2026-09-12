#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const relative =
  "content-manifests/divergent-universe-runtime-v1/build-capability-closure.json";
execFileSync("node", [
  "tools/divergent-universe-runtime/generate-build-capability-closure.mjs", "--check",
], { cwd: root, stdio: "inherit" });

const closure = json(relative);
assert(closure.batch === "G22-P2-B4"
  && closure.status === "CompleteNoSharedRustDelta",
"Build capability closure is not terminal");
assert(closure.summary.mapping_eligibility === 84
  && closure.summary.mapping_builds === 95
  && closure.summary.mapping_lifecycle_rules === 7
  && closure.summary.mapping_source_obligations === 258,
"Arithmetic Mapping denominator drift");
assert(closure.summary.resolved_avatar_builds === 91
  && closure.summary.unresolved_avatar_builds === 4
  && closure.summary.special_avatar_builds === 79,
"Arithmetic Mapping identity boundary drift");
assert(equal(closure.summary.role_buff_parameter_arities, {
  4: 41, 5: 34, 6: 18, 7: 2,
}), "role-buff parameter arity drift");
assert(closure.summary.build_domain_mechanic_programs === 0
  && closure.summary.build_domain_shapes === 0
  && closure.summary.runtime_lowered_rows === 0,
"Build closure claimed source-program or runtime execution credit");
assert(closure.summary.shared_build_primitive_additions === 0
  && closure.decision.shared_build_primitive_additions.length === 0,
"unreviewed shared Build primitive addition remains");
assert(closure.unresolved_public_identity_build_ids.length === 4,
  "unreleased avatar identity boundary drift");
assert(equal(closure.reference_field_values, {
  level: ["EquilibriumLevelCapWhenBelow"],
  trace_state: ["ActivateOrRaiseWhenInactiveOrBelowRequirement"],
  light_cone: ["UnspecifiedConditionAndTemporaryIdentity"],
  relics: ["ReplaceWhenTotalEnhancementBelowRequirement"],
  exact_temporary_loadout: ["Unspecified"],
}), "Mapping field policy drift");

for (const item of [...closure.existing_shared_surfaces, ...closure.capability_probes]) {
  const source = fs.readFileSync(path.join(root, item.file), "utf8");
  for (const fragment of item.required_fragments)
    assert(source.includes(fragment), `${item.id} lost required fragment: ${fragment}`);
}

console.log(
  "Divergent Universe Build capability closure verified "
    + "(95 mappings; 258 obligations; zero shared additions or execution credit).",
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
