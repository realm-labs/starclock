#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = json("policy/goal20-coverage-and-release.json");
const ci = json(policy.inputs.ci_matrix.path);
const matrix = json("evidence/swarm-disaster-runtime-v1/foundation/coverage-matrix.json");
assert(policy.schema_revision === "starclock.goal20-coverage-and-release.v1",
  "unsupported Goal 20 coverage/release contract");
assert(policy.goal_id === "swarm-disaster-runtime-v1"
  && policy.batch === "G20-P0-B4", "Goal 20 Phase 0 identity drift");

assert(policy.first_vertical_slice.required_boundaries.length === 10,
  "first vertical-slice boundary count drift");
assert(allTrue(policy.first_vertical_slice.contracts),
  "first vertical-slice contract incomplete");
assert(matrix.summary.complete_runs === 16
  && matrix.summary.formal_difficulties === 5
  && matrix.summary.audience_paths === 8
  && matrix.summary.audience_dice === 8
  && matrix.summary.reachable_die_faces === 42
  && matrix.summary.boundary_cases === 8
  && matrix.summary.policy_probes === 31,
"seeded coverage matrix denominator drift");
assert(allTrue(policy.coverage_matrix.contracts), "coverage matrix contract incomplete");

const performance = policy.performance;
assert(performance.workloads.length === 7, "performance workload count drift");
assert(unique(performance.workloads, "id") === 7, "duplicate performance workload");
assert(performance.structural_budgets.catalog_clones_per_run === 0
  && performance.structural_budgets.catalog_compositions_per_factory === 1
  && performance.structural_budgets.replay_prefix_reconstructions === 0
  && performance.structural_budgets.warm_assembly_allocations === 0,
"structural performance budget drift");
assert(allTrue(performance.contracts), "performance contract incomplete");

assert(equal(policy.ci.native_profiles, ci.native_profiles.map(({ id }) => id)),
  "native CI profile drift");
assert(equal(policy.ci.compile_only_profiles,
  ci.compile_only_profiles.map(({ id }) => id)), "compile-only CI profile drift");
assert(policy.ci.native_claims.length === 5, "native CI claim count drift");
assert(allTrue(policy.ci.contracts), "CI contract incomplete");

const release = policy.release_scaffold;
assert(release.planned_phases === 9 && release.planned_batches === 51,
  "release execution denominator drift");
assert(release.required_source_obligations === 6963
  && release.required_mechanic_rules === 23
  && release.required_fixture_families === 23
  && release.required_policy_boundaries === 31,
"release content denominator drift");
assert(release.required_gates.length === 13, "release gate count drift");
assert(release.release_batch === "G20-P8-B4", "release batch drift");
assert(allTrue(release.contracts), "release scaffold incomplete");
assert(allTrue(policy.contracts), "Phase 0 top-level contract incomplete");

const evidence = json("evidence/swarm-disaster-runtime-v1/foundation/phase0-summary.json");
assert(evidence.result === "Pass" && evidence.batches_complete === 4
  && evidence.seeded_complete_runs === 16 && evidence.policy_boundaries === 31
  && evidence.native_handlers_admitted === 0,
"Phase 0 evidence summary drift");
assert(evidence.checks.length === 7
  && evidence.checks.every(({ result }) => result === "Pass"),
"Phase 0 evidence contains incomplete check");

const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
for (const batch of ["G20-P0-B1", "G20-P0-B2", "G20-P0-B3", "G20-P0-B4"])
  assert(status.includes(`| \`${batch}\` | \`Complete\` |`),
    `${batch} is not complete in the Goal 20 ledger`);
assert(status.includes(
  "| Phase 0 — Contract, audit and execution plan | `Complete` |",
), "Goal 20 Phase 0 is not complete");
assert(status.includes("| Next unblocked batch | `G20-P1-B1` |"),
  "Goal 20 next batch is not G20-P1-B1");

console.log(
  "Goal 20 Phase 0 verified (16-run matrix; 31 policy probes; 7 workloads; " +
  "3 native + 3 compile-only profiles; 13 release gates).",
);

function unique(entries, field) { return new Set(entries.map((e) => e[field])).size; }
function allTrue(values) { return Object.values(values).every((value) => value === true); }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function assert(condition, message) { if (!condition) throw new Error(message); }
