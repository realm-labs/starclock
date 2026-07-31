#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/cognition/cognition-runtime.json",
);

assert(evidence.schema_revision
  === "starclock.gold-and-gears-cognition-runtime.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P2-B4"
  && evidence.result === "Pass",
"Goal 14 Cognition evidence drift");
assert(evidence.catalog_input.candidate_bundle_sha256
  === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b"
  && evidence.catalog_input.cognition_ranges === 13
  && evidence.catalog_input.secrets === 20
  && evidence.catalog_input.mode_constants === 22
  && evidence.catalog_input.terminal_secrets === 10,
"Cognition input denominator drift");

const lifecycle = evidence.cognition_lifecycle;
assert(lifecycle.initial_value === 0
  && lifecycle.global_minimum === -40
  && lifecycle.global_maximum === 40
  && lifecycle.adjustment_order.join(",")
    === [
      "apply-cognition-delta",
      "clamp-to-global-range",
      "clamp-to-selected-area-range",
    ].join(",")
  && lifecycle.bounds === "inclusive"
  && lifecycle.slot_scope === "Activity"
  && lifecycle.carry === "CarryExact"
  && lifecycle.new_run_reset === "ActivityStart-to-zero"
  && lifecycle.rng_draws === 0,
"Cognition lifecycle drift");

const frontier = evidence.secret_frontier;
assert(frontier.evaluation_boundary === "AfterCurrentPlaneBossDefeat"
  && frontier.layers["1"] === 2
  && frontier.layers["2"] === 8
  && frontier.layers["3"] === 10
  && frontier.required_area_counts["401"] === 10
  && frontier.required_area_counts["403"] === 6
  && frontier.required_area_counts["404"] === 4
  && frontier.eligibility_order.join(",")
    === [
      "required-area-at-or-below-selected-area",
      "current-plane-layer",
      "predecessor-secret-frontier",
      "inclusive-cognition-range",
    ].join(",")
  && frontier.multiple_match_order.join(",")
    === "minimum-cognition,maximum-cognition,source-secret-id"
  && frontier.unlock_count_per_evaluation === "zero-or-one"
  && frontier.all_authored_secrets_reachable_in_valid_frontier === 20,
"Secret-frontier contract drift");

const operations = evidence.activity_operations;
assert(operations.set_query === "ActivityCondition::OrderedIdSetContains"
  && operations.set_mutation === "ActivityOperation::InsertOrderedId"
  && operations.duplicate_insert === "accepted-no-op"
  && operations.canonical_order === "ascending-u64"
  && operations.cognition_mutation === "ActivityOperation::SetSlot"
  && operations.plane_evaluation_markers === "ActivityOperation::AddCounter",
"typed Activity operation evidence drift");

const policy = evidence.cognition_policy;
assert(policy.register_id === "G14-R03"
  && policy.terminal_state === "VersionedExecutablePolicy"
  && policy.revision === "gold-and-gears-cognition-policy-v1"
  && policy.accuracy === "ProjectPolicy"
  && nonEmpty(policy.replacement_condition),
"G14-R03 is not a truthful terminal executable policy");
assert(Object.values(evidence.validation).every((value) => value === true || value === 0),
"Cognition validation evidence drift");
assert(evidence.tests.activity_transaction_unit_passed === 7
  && evidence.tests.activity_full_suite_passed === true
  && evidence.tests.entry_and_cognition_unit_passed === 16
  && evidence.tests.cognition_specific_unit_passed === 5
  && evidence.tests.clippy_activity_passed === true
  && evidence.tests.clippy_universe_passed === true
  && evidence.tests.cold_quick_attempt
    === "BudgetExceededDuringSelectedTestDispatch"
  && evidence.tests.quick_gate_passed === true
  && evidence.tests.quick_gate_seconds === "98.8"
  && evidence.tests.quick_selected_harnesses === 67
  && evidence.tests.quick_direct_packages === 2
  && evidence.tests.quick_downstream_packages_checked === 7
  && evidence.tests.full_gate_passed === true
  && evidence.tests.full_gate_seconds === "427.6"
  && evidence.tests.full_workspace_harnesses === 138,
"Cognition test evidence drift");

const cognition = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/cognition.rs",
);
const api = text("crates/starclock-mode-universe/src/gold_gears_entry/api.rs");
const activityProgram = text("crates/starclock-activity/src/program.rs");
const activitySet = text(
  "crates/starclock-activity/src/transaction/ordered_id_set.rs",
);
for (const literal of [
  'GOLD_AND_GEARS_COGNITION_REVISION: &str = "gold-and-gears-cognition-policy-v1"',
  "ActivityExpression::Add",
  "ActivityExpression::Minimum",
  "ActivityExpression::Maximum",
  "compile_plane_boss_evaluation",
  "secret.required_area <= range.area_source",
  "secret.minimum, secret.maximum, secret.order_id",
])
  assert(cognition.includes(literal), `missing Cognition contract ${literal}`);
for (const literal of [
  "OrderedIdSetContains",
  "InsertOrderedId",
])
  assert(activityProgram.includes(literal) && activitySet.includes("insert_ordered_id"),
    `missing generic Activity set contract ${literal}`);
assert(api.includes("compile_cognition_adjustment")
  && api.includes("compile_cognition_carry")
  && api.includes("compile_plane_boss_cognition_evaluation")
  && api.includes("secret_frontier"),
"public Cognition compiler facade drift");
assert(!cognition.includes("ActivityRng")
  && !cognition.includes("thread_rng")
  && !cognition.includes("SystemTime")
  && !cognition.includes("content-reference"),
"Cognition execution introduced RNG, time or normalized-data reads");

for (const relative of [
  "crates/starclock-activity/src/program.rs",
  "crates/starclock-activity/src/transaction.rs",
  "crates/starclock-activity/src/transaction/condition.rs",
  "crates/starclock-activity/src/transaction/ordered_id_set.rs",
  "crates/starclock-test-kit/tests/suites/activity/activity/activity_transaction.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/api.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/cognition.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/cognition_tests.rs",
])
  assert(text(relative).split(/\r?\n/u).length <= 1200,
    `Cognition source exceeds the handwritten limit: ${relative}`);

for (const protectedRoot of [
  "evidence/gold-and-gears-reference-v1",
  "content-manifests/gold-and-gears-v1",
  "content-reference/gold-and-gears-v1",
  "config/gold-and-gears/data",
  "config/gold-and-gears-generated",
])
  assert(captureGit([
    "status",
    "--porcelain=v1",
    "--untracked-files=all",
    "--",
    protectedRoot,
  ]).trim() === "", `protected root has worktree changes: ${protectedRoot}`);

const dependencyPolicy = text("tools/dependency-policy/verify.mjs");
assert(dependencyPolicy.includes(
  '"crates/starclock-mode-universe/src/gold_gears_entry/cognition.rs"',
), "private embedded-field lowering owner is not release-validated");
const status = text("docs/goals/14-gold-and-gears-runtime-status.md");
assert(status.includes("| `G14-P2-B4` | `Complete` |"),
  "G14-P2-B4 is incomplete");
assert(status.includes("| `G14-R03` | `VersionedExecutablePolicy` |"),
  "G14-R03 is not terminal");

console.log(
  "Goal 14 P2-B4 verified (13 ranges; 20 Secrets; " +
  "three deterministic frontiers; G14-R03 terminal).",
);

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function captureGit(args) {
  return execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
}
function nonEmpty(value) {
  return typeof value === "string" && value.length > 0;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
