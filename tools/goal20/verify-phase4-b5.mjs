#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/content/occurrence-service-adventure-runtime.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-interaction-runtime.v1"
  && evidence.batch === "G20-P4-B5" && evidence.result === "Pass", "P4-B5 identity drift");
const input = evidence.catalog_input;
assert(input.assigned_source_obligations === 153 && input.occurrences === 75
  && input.occurrence_variants === 57 && input.shared_services === 15
  && input.adventure_outcomes === 6 && input.supporting_occurrence_choices === 308
  && input.supporting_service_rules === 19 && input.supporting_currencies === 1,
"P4-B5 denominator drift");
const occurrence = evidence.occurrence_contract;
assert(occurrence.occurrence_pool === 55 && occurrence.the_swarm_pool === 14
  && occurrence.encounter_pool === 3 && occurrence.deal_pool === 3
  && occurrence.seeded_random_choices === 60 && occurrence.rng_label === "Occurrence"
  && occurrence.absent_or_cross_pool_binding === "FailClosed" && occurrence.zero_work_draws === 0,
"Occurrence runtime contract drift");
const service = evidence.service_contract;
assert(service.shared_standard_definitions === 15 && service.blessing_shops === 5
  && service.curio_shops === 4 && service.other_services === 6
  && service.beacon_contributions === 4 && service.initial_cosmic_fragments === 50
  && service.shop_rng_label === "Shop" && service.authored_costs === "ExactInheritedAllowlist"
  && service.insufficient_resource === "AtomicReject" && service.stale_purchase === "AtomicReject",
"Service runtime contract drift");
const adventure = evidence.adventure_contract;
assert(adventure.capture_monster_rooms === 3 && adventure.destroy_prop_rooms === 3
  && JSON.stringify(adventure.accepted_external_tiers) === '["Tier1","Tier2","Tier3"]'
  && adventure.action_input_simulation === "Excluded"
  && adventure.settlement_scope === "ExactlyOncePerAdventureRoom"
  && adventure.unresolved_payload === "RejectWithoutMutation", "Adventure runtime contract drift");
assert(evidence.compatibility.occurrence_runtime_digest
  === "d3d1a2fe70dc05cbd8046df2e7f56f1a2cb8668739dc9da1f5ff6527d0607bd1"
  && evidence.compatibility.service_runtime_digest
  === "71d9b473f30b853b58c2cd5e56f02c9620093d3975a542fbcdf3fc4acebb1d80"
  && evidence.compatibility.adventure_runtime_digest
  === "e174154cd9307d88075ffc2cad131ed03bc5c1440b86350b33092b46844762f3"
  && JSON.stringify(evidence.compatibility.selected_outcome_candidates) === "[10,30]"
  && JSON.stringify(evidence.compatibility.selected_blessing_ids) === "[140,34]"
  && JSON.stringify(evidence.compatibility.selected_curio_ids) === "[1011,1002]",
"P4-B5 compatibility drift");
const expectedPolicies = new Map([
  ["abstract-adventure-outcome", 8], ["beacons", 12], ["occurrence-pool-selection", 75],
  ["occurrence-random-outcome", 60], ["service-transaction-boundary", 37],
  ["shared-content-pool-weight", 328],
]);
assert(evidence.policy_boundaries.length === 6, "P4-B5 policy count drift");
for (const policy of evidence.policy_boundaries) {
  const suffix = [...expectedPolicies.keys()].find((candidate) => policy.boundary_id.endsWith(`project-policy-${candidate}`));
  assert(suffix && policy.state === "VersionedExecutablePolicy" && policy.accuracy === "ProjectPolicy"
    && policy.affected_record_count === expectedPolicies.get(suffix) && policy.remaining_owner === null,
  `P4-B5 policy terminal state drift: ${policy.boundary_id}`);
}
assert(evidence.deferred_semantics.length === 2, "P4-B5 fixture count drift");
for (const fixture of evidence.deferred_semantics)
  assert(fixture.execution_batch === "G20-P5-B1" && fixture.state === "Pending"
    && fixture.mechanic_rule_state === "Pending", `P4-B5 fixture overclaim: ${fixture.semantic_fixture_id}`);
assert(evidence.deferred_semantics[0].ordered_operation_count === 4
  && evidence.deferred_semantics[0].expected_fact_count === 5
  && evidence.deferred_semantics[0].source_record_count === 2
  && evidence.deferred_semantics[0].mechanic_rule_batch === "G20-P5-M09"
  && evidence.deferred_semantics[1].ordered_operation_count === 3
  && evidence.deferred_semantics[1].expected_fact_count === 4
  && evidence.deferred_semantics[1].source_record_count === 3
  && evidence.deferred_semantics[1].mechanic_rule_batch === "G20-P5-M10", "P4-B5 fixture contract drift");
const validation = evidence.validation;
assert(validation.runtime_json_file_reads === 0 && validation.second_activity_state_machine_added === false
  && validation.new_public_mode_types === 0 && validation.public_runtime_methods_added === 16
  && validation.public_reexports_added === 0 && validation.source_policy_handwritten_files === 903
  && validation.source_policy_public_reexports === 72
  && validation.private_serde_json_owners_registered.length === 2,
"P4-B5 validation drift");
for (const [file, maximum] of [
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs", 200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/content_runtime.rs", 1200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/occurrence_runtime.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/occurrence_runtime_tests.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/service_adventure_runtime.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/service_adventure_runtime_tests.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/test_modules.rs", 200],
  ["crates/starclock-mode-universe/src/swarm_disaster_content/interaction_access.rs", 800],
]) assert(text(file).split(/\r?\n/u).length <= maximum, `P4-B5 source boundary exceeded: ${file}`);
const dispositions = json("content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json");
const obligations = dispositions.source_obligations.filter((row) => row.execution_batch === "G20-P4-B5");
assert(obligations.length === 153 && new Set(obligations.map((row) => row.id)).size === 153,
  "P4-B5 obligation exact-once drift");
for (const policy of evidence.policy_boundaries) {
  const frozen = dispositions.policy_boundaries.find((row) => row.id === policy.boundary_id);
  assert(frozen?.current_state === "InheritedPolicy" && frozen.implementation_batches.includes("G20-P4-B5"),
    `frozen policy assignment drift: ${policy.boundary_id}`);
}
for (const protectedRoot of ["evidence/swarm-disaster-reference-v1", "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1", "config/swarm-disaster/data", "config/swarm-disaster-generated"])
  assert(captureGit(["status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot]).trim() === "",
    `protected root changed: ${protectedRoot}`);
const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P4-B5` | `Complete` |")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P5-M01` |")
  && status.includes("9 inherited / 22 terminal / 20 pending"), "Goal 20 did not advance after P4-B5");
const tests = evidence.tests;
assert(tests.occurrence_runtime_unit_passed === 4 && tests.service_adventure_runtime_unit_passed === 4
  && tests.entry_lifecycle_unit_passed === 78 && tests.swarm_unit_passed === 89
  && tests.identity_integration_passed === 5 && tests.clippy_passed === true
  && tests.quick_gate_result === "Pass" && nonPending(tests.quick_gate_seconds)
  && tests.full_gate_passed === true && nonPending(tests.full_gate_seconds),
"P4-B5 terminal tests are incomplete");
console.log("Goal 20 P4-B5 verified (153 obligations, 75 Occurrences, 15 Services, 6 Adventures).");

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function captureGit(args) { return execFileSync("git", args, { cwd: root, encoding: "utf8" }); }
function assert(condition, message) { if (!condition) throw new Error(message); }
function nonPending(value) { return value !== null && value !== undefined && value !== "Pending"; }
