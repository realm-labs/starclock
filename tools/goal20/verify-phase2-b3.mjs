#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/swarm-disaster-runtime-v1/topology/domain-beacon-overlay.json",
);

assert(evidence.schema_revision === "starclock.swarm-disaster-map-overlay.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P2-B3"
  && evidence.result === "Pass",
"Goal 20 map-overlay evidence drift");
const input = evidence.catalog_input;
assert(input.candidate_bundle_sha256
  === "385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362"
  && input.chessboards === 101
  && input.map_events === 349
  && input.block_create_rules === 1_212
  && input.domains === 12
  && input.beacons === 4
  && input.room_bindings === 861
  && input.assigned_source_obligations === 2_438,
"map-overlay input denominator drift");

const creation = evidence.creation_execution;
assert(creation.formal_planes_executed === 3
  && creation.route_nodes === 47
  && creation.operation_counts_per_plane.join(",") === "33,84,24"
  && creation.terminal_domain_ids.join(",") === "4,4,8"
  && creation.node_state_created === 1
  && creation.canonical_target_order === "stable-global-node-id"
  && creation.capacity_overflow_policy === "no-legal-target-no-op"
  && creation.remaining_node_policy === "explicit-empty-domain"
  && creation.graph_digest_before_after
    === "e371d5f7d68f589e50dd57e033a857241663a1a10260216b3342a0faac4f1c80",
"map creation execution drift");
const mutation = evidence.mutation_execution;
assert(mutation.replacement_state === 2
  && mutation.copy_state === 3
  && mutation.blanked_state === 4
  && mutation.replacement_without_beacon === "preserve-target-beacon"
  && mutation.copy_policy === "copy-source-domain-preserve-target-beacon"
  && mutation.blanking_policy
    === "clear-domain-preserve-target-beacon-and-filter-incoming-route"
  && mutation.invalid_graph_node_rejected === true
  && mutation.immutable_graph_definition === true,
"map mutation semantics drift");
const ordering = evidence.event_ordering;
assert(ordering.selection_precedes_creation === true
  && ordering.event_descriptor_operations === 3
  && ordering.descriptor_order.join(",")
    === "event-id,effect-kind,primary-parameter"
  && ordering.creation_operations_follow_descriptors === true
  && ordering.empty_candidate_draws === 0
  && ordering.semantic_fixture_state === "PendingG20P5M02AndG20P5B1",
"topology-event ordering drift");
const rng = evidence.rng_contract;
assert(rng.stream === "Graph"
  && rng.map_event_purpose === "0x5301"
  && rng.create_count_purpose === "0x5302"
  && rng.beacon_purpose === "0x5303"
  && rng.other_active_streams === 0
  && rng.weighted_integer_sampling === true
  && rng.stable_candidate_order === true,
"map RNG contract drift");

const expectedPolicies = new Map([
  ["swarm-disaster.research-gap.source-goal09-project-policy-domains",
    ["VersionedExecutablePolicy", 14]],
  ["swarm-disaster.research-gap.source-goal09-project-policy-topology-consequences",
    ["VersionedExecutablePolicy", 17]],
  ["swarm-disaster.research-gap.source-goal09-project-policy-beacons",
    ["InheritedPolicy", 12]],
]);
assert(evidence.policy_boundaries.length === expectedPolicies.size,
  "map-overlay policy boundary count drift");
for (const policy of evidence.policy_boundaries) {
  const expected = expectedPolicies.get(policy.boundary_id);
  assert(expected
    && policy.state === expected[0]
    && policy.affected_record_count === expected[1]
    && policy.accuracy === "ProjectPolicy"
    && nonEmpty(policy.implemented_revision)
    && policy.semantic_fixture_state === "PendingG20P5M02AndG20P5B1"
    && nonEmpty(policy.replacement_condition),
  `map-overlay policy is mislabeled: ${policy.boundary_id}`);
}
assert(evidence.policy_boundaries.find((policy) =>
  policy.boundary_id.endsWith("project-policy-beacons"))?.remaining_owner
    === "G20-P4-B5",
"beacon boundary was terminalized before its remaining owner");
const validation = evidence.validation;
assert(validation.external_runtime_json_reads === 0
  && validation.embedded_sora_json_lowered_once_at_factory_load === true
  && validation.authoritative_float_fields === 0
  && validation.generated_public_types === 0
  && validation.public_reexports_added === 0
  && validation.rejected_programs_leave_state_unchanged === true,
"map-overlay validation evidence drift");
const tests = evidence.tests;
assert(tests.entry_and_overlay_unit_passed === 8
  && tests.swarm_unit_passed === 19
  && tests.identity_integration_passed === 5
  && tests.clippy_passed === true
  && tests.quick_gate_result === "TimeoutAfterSelectedTests"
  && tests.quick_gate_budget_seconds === 180
  && tests.quick_selected_harnesses === 7
  && tests.quick_selected_build_seconds === "50.4"
  && tests.quick_selected_tests_seconds === "136.0"
  && tests.full_gate_passed === true
  && nonEmpty(tests.full_gate_seconds)
  && tests.full_generated_checks === 33
  && tests.full_source_cache_skips === 4
  && tests.full_workspace_harnesses === 34
  && nonEmpty(tests.full_workspace_tests_seconds),
"map-overlay test evidence drift");

const source = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/map_overlay.rs",
);
const access = text(
  "crates/starclock-mode-universe/src/swarm_disaster_content/map_access.rs",
);
const instance = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/instance.rs",
);
for (const literal of [
  "MAP_EVENT_PURPOSE: u16 = 0x5301",
  "CREATE_COUNT_PURPOSE: u16 = 0x5302",
  "BEACON_PURPOSE: u16 = 0x5303",
  "NODE_STATE_CREATED: i64 = 1",
  "NODE_STATE_REPLACED: i64 = 2",
  "NODE_STATE_COPIED: i64 = 3",
  "NODE_STATE_BLANKED: i64 = 4",
  "counter(NODE_BEACON, target)",
  "operations.extend(self.creation_operations",
]) assert(source.includes(literal), `missing map-overlay contract ${literal}`);
for (const literal of [
  "compile_plane_creation",
  "compile_map_event_then_creation",
  "compile_node_replacement",
  "compile_node_copy",
  "compile_node_blanking",
  "node_is_blanked",
]) assert(instance.includes(literal), `missing map-overlay API ${literal}`);
assert(access.includes("serde_json::from_str")
  && access.includes("map_runtime_input"),
"embedded Sora map lowering drift");
assert(!source.includes("rand::") && !source.includes("thread_rng")
  && !source.includes("SystemTime") && !source.includes("serde_json")
  && !source.includes("f32") && !source.includes("f64"),
"map execution introduced nondeterminism, floats or runtime JSON");

for (const relative of [
  "crates/starclock-mode-universe/src/swarm_disaster_content/map_access.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/instance.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/map_overlay.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/map_overlay_tests.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_structural/map_access.rs",
]) assert(text(relative).split(/\r?\n/u).length <= 800,
  `Swarm runtime source should be split before 800 lines: ${relative}`);
assert(text("crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs")
  .split(/\r?\n/u).length <= 200,
"Swarm entry facade exceeds the 200-line limit");

const dispositions = json(
  "content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json",
);
const assigned = dispositions.source_obligations.filter((row) =>
  row.execution_batch === "G20-P2-B3");
assert(assigned.length === 2_438, "P2-B3 source-obligation assignment drift");
const categories = counts(assigned.map((row) => row.category));
assert(categories.beacons === 4
  && categories.block_create_rules === 1_212
  && categories.domains === 12
  && categories.map_events === 349
  && categories.room_bindings === 861,
"P2-B3 category denominator drift");
for (const [id, [, affected]] of expectedPolicies) {
  const boundary = dispositions.policy_boundaries.find((row) => row.id === id);
  assert(boundary?.current_state === "InheritedPolicy"
    && boundary.affected_record_count === affected
    && boundary.implementation_batches.includes("G20-P2-B3"),
  `frozen P0 policy assignment drift: ${id}`);
}

for (const protectedRoot of [
  "evidence/swarm-disaster-reference-v1",
  "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1",
  "config/swarm-disaster/data",
  "config/swarm-disaster-generated",
]) assert(captureGit([
  "status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot,
]).trim() === "", `protected root has worktree changes: ${protectedRoot}`);

const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P2-B3` | `Complete` |")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P2-B4` |")
  && status.includes("28 inherited / 3 terminal / 28 pending"),
"Goal 20 did not advance after P2-B3");

console.log(
  "Goal 20 P2-B3 verified (2,438 inputs; 47 node overlays; "
    + "Graph-only RNG; 2 newly terminal policies).",
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
function counts(values) {
  return values.reduce((result, value) => {
    result[value] = (result[value] ?? 0) + 1;
    return result;
  }, {});
}
function nonEmpty(value) {
  return typeof value === "string" && value.length > 0;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
