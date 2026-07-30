#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = json("policy/goal14-coverage-and-release.json");
const ci = json(policy.inputs.ci_matrix.path);
assert(policy.schema_revision === "starclock.goal14-coverage-and-release.v1",
  "unsupported Goal 14 coverage/release contract");
assert(policy.goal_id === "gold-and-gears-runtime-v1"
  && policy.batch === "G14-P0-B4", "Goal 14 phase-0 identity drift");

assert(policy.first_vertical_slice.default_faces.length === 6,
  "first vertical slice does not fill six dice slots");
assert(policy.first_vertical_slice.required_boundaries.length === 10,
  "first vertical slice boundary count drift");
assert(allTrue(policy.first_vertical_slice.contracts),
  "first vertical slice contract is incomplete");

assert(policy.policy_gaps.length === 16, "policy owner denominator drift");
assert(unique(policy.policy_gaps, "register_id") === 16
  && unique(policy.policy_gaps, "source_id") === 16,
"policy ownership is not exact");
assert(policy.policy_gaps.every(({ owner_batches: owners }) =>
  owners.length > 0 && owners.every((batch) => /^G14-P[2-6]-(?:B|M)\d+$/u.test(batch))),
"policy gap has an invalid owner batch");
assert(policy.policy_terminal_states.length === 4,
  "policy terminal-state vocabulary drift");

const performance = policy.performance;
assert(performance.workloads.length === 7, "performance workload count drift");
assert(unique(performance.workloads, "id") === performance.workloads.length,
  "duplicate performance workload");
assert(performance.structural_budgets.catalog_clones_per_run === 0
  && performance.structural_budgets.catalog_compositions_per_factory === 1
  && performance.structural_budgets.replay_prefix_reconstructions === 0
  && performance.structural_budgets.warm_assembly_allocations === 0,
"structural performance budget drift");
assert(allTrue(performance.contracts), "performance contract is incomplete");

assert(equal(policy.ci.native_profiles, ci.native_profiles.map(({ id }) => id)),
  "native CI profile drift");
assert(equal(policy.ci.compile_only_profiles, ci.compile_only_profiles.map(({ id }) => id)),
  "compile-only CI profile drift");
assert(policy.ci.native_claims.length === 5, "native CI claim count drift");
assert(allTrue(policy.ci.contracts), "CI contract is incomplete");

const release = policy.release_scaffold;
assert(release.planned_phases === 9 && release.planned_batches === 48,
  "release execution denominator drift");
assert(release.required_source_obligations === 7913
  && release.required_mechanic_rules === 1224
  && release.required_fixture_families === 18
  && release.required_policy_boundaries === 16,
"release content denominator drift");
assert(release.required_gates.length === 13, "release gate denominator drift");
assert(release.release_batch === "G14-P8-B4", "release batch drift");
assert(allTrue(release.contracts), "release contract scaffold is incomplete");
assert(allTrue(policy.coverage_matrix.contracts), "coverage matrix contract is incomplete");
assert(allTrue(policy.contracts), "phase-0 top-level contract is incomplete");

const evidence = json("evidence/gold-and-gears-runtime-v1/foundation/phase0-summary.json");
assert(evidence.result === "Pass"
  && evidence.batches_complete === 4
  && evidence.seeded_complete_runs === 25
  && evidence.policy_boundaries === 16
  && evidence.native_handlers_admitted === 0,
"Phase 0 evidence summary drift");
assert(evidence.checks.length === 7
  && evidence.checks.every(({ result }) => result === "Pass"),
"Phase 0 evidence contains an incomplete check");

const status = text("docs/goals/14-gold-and-gears-runtime-status.md");
for (const batch of ["G14-P0-B1", "G14-P0-B2", "G14-P0-B3", "G14-P0-B4"])
  assert(status.includes(`| \`${batch}\` | \`Complete\` |`),
    `${batch} is not complete in the Goal 14 ledger`);
assert(status.includes(
  "| Phase 0 — Contract, audit and execution plan | `Complete` |",
), "Goal 14 Phase 0 is not complete");
assert(status.includes("| Next unblocked batch | `G14-P1-B1` |"),
  "Goal 14 next batch is not G14-P1-B1");

console.log(
  "Goal 14 Phase 0 verified (25-run matrix; first slice; 16 policy owners; " +
  "7 workloads; 3 native + 3 compile-only CI profiles; 13 release gates).",
);

function unique(entries, field) {
  return new Set(entries.map((entry) => entry[field])).size;
}
function allTrue(values) {
  return Object.values(values).every((value) => value === true);
}
function equal(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
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
