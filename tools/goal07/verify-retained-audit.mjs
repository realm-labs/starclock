#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync("node", ["tools/goal07/audit-retained.mjs", "--check"], {
  cwd: root,
  stdio: "inherit",
});
const policy = json("policy/goal07-retained-audit.json");
const audit = json(
  "content-manifests/standard-universe-mechanics-complete-v1/retained-audit.json",
);
const evidence = json(
  "evidence/standard-universe-mechanics-complete-v1/phase0/retained-audit-summary.json",
);
assert(audit.schema_revision === "starclock.goal07-retained-audit.v1"
  && evidence.schema_revision === "starclock.goal07-retained-audit-summary.v1",
"Goal 07 retained audit revision drift");
for (const [field, expected] of Object.entries(policy.denominators)) {
  if (field === "retained_records")
    assert(audit.summary.records.inherited_states.RetainedApproximation === expected,
      `${field} drift`);
  else if (field === "retained_rules")
    assert(audit.summary.rules.inherited_states.RetainedApproximation === expected,
      `${field} drift`);
  else if (field === "approximate_enemy_proxies")
    assert(audit.summary.enemy_variants.inherited_states.ApproximateProxy === expected,
      `${field} drift`);
  else
    assert(evidence.audited[field] === expected, `${field} drift`);
}
for (const entry of audit.rules)
  assert(entry.intended_runtime_disposition === "ExecutableRuleIr",
    `${entry.id}: rule target is not executable Rule IR`);
for (const entry of [...audit.records, ...audit.rules, ...audit.fixtures]) {
  assert(entry.evidence_gaps.length > 0, `${entry.id}: audit has no closure proof`);
  assert(policy.runtime_targets.includes(entry.intended_runtime_disposition),
    `${entry.id}: unsupported runtime target`);
  assert(policy.accuracy_targets.includes(entry.intended_accuracy_disposition),
    `${entry.id}: unsupported accuracy target`);
}
assert(audit.enemy_variants.every((entry) =>
  entry.mechanism_target === "ExactPublic"), "an enemy permits mechanic approximation");
assert(audit.enemy_variants.filter((entry) =>
  entry.intended_accuracy_disposition === "ApprovedNumericApproximation").length
  === policy.denominators.approximate_enemy_proxies,
"enemy numeric approximation target drift");
assert(audit.summary.native_review_candidate_rules > 0
  && audit.summary.native_review_candidate_entries
    === audit.summary.native_review_candidate_rules * 2,
  "legacy static-handler candidates were silently discarded");
assert(Object.values(policy.contracts).every((value) => value === true),
  "Goal 07 audit contains an unaccepted contract");
const status = text("docs/goals/07-standard-universe-mechanics-completion-status.md");
assert(status.includes("| `G07-P0-B2` | `Complete` |"), "G07-P0-B2 is incomplete");
const nextBatch = status.match(/^\| Next unblocked batch \| (.+) \|$/mu)?.[1];
assert(nextBatch === "None"
  || /^`G07-(?:P0-B[34]|P1-B[1-6]|P[2-5]-M\d+-S\d+|P[67]-B\d+)`$/u
    .test(nextBatch ?? ""), "Goal 07 next batch regressed before G07-P0-B3");
console.log(
  `Goal 07 retained audit verified (${audit.records.length + audit.rules.length
    + audit.fixtures.length} inherited rows, ${audit.enemy_variants.length} enemies, ` +
  `${audit.encounter_members.length} encounter members).`,
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
