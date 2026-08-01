#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/content/blessing-curio-runtime.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-content-runtime.v1"
  && evidence.batch === "G20-P4-B4" && evidence.result === "Pass", "P4-B4 identity drift");
const input = evidence.catalog_input;
assert(input.assigned_source_obligations === 564 && input.blessings === 144
  && input.blessing_levels === 288 && input.curios === 66 && input.curio_states === 66
  && input.supporting_pool_memberships === 184 && input.supporting_curio_rules === 66,
"P4-B4 denominator drift");
const blessing = evidence.blessing_contract;
assert(blessing.shared_standard_definitions === 162 && blessing.reachable_definitions === 144
  && blessing.reachable_paths === 8 && blessing.levels_per_blessing === 2
  && blessing.inventory_maximum_entries === 144 && blessing.inventory_maximum_stack === 2
  && blessing.integer_weight === 1 && blessing.rng_label === "Reward"
  && blessing.acquire_enhance_replace_owner === "SharedBlessingRuntimeCatalog",
"shared Blessing contract drift");
const curio = evidence.curio_contract;
assert(curio.mode_copy_definitions === 66 && curio.normal === 53 && curio.negative === 7
  && curio.error_code === 6 && curio.charged === 6 && curio.source_condition_destroyed === 3
  && curio.repairing_to_fixed === 6 && curio.replace_all === 1
  && curio.absent_offer_binding === "FailClosed" && curio.no_legal_replacement === "NoOp",
"Swarm Curio contract drift");
assert(evidence.compatibility.content_runtime_digest
  === "5840363010af31710d2db6438eafb8dac613beef9c42e9d29ba36d82f8d5f6eb"
  && JSON.stringify(evidence.compatibility.blessing_offer_ids) === "[35,36,88]"
  && JSON.stringify(evidence.compatibility.curio_offer_ids) === "[1019,1002]"
  && evidence.compatibility.activity_state_slots === 16
  && evidence.compatibility.activity_inventories === 2, "P4-B4 compatibility drift");
const policy = evidence.policy_boundaries[0];
assert(evidence.policy_boundaries.length === 1
  && policy.boundary_id.endsWith("project-policy-curio-selection-and-lifecycle")
  && policy.state === "VersionedExecutablePolicy" && policy.accuracy === "ProjectPolicy"
  && policy.affected_record_count === 200 && policy.remaining_owner === null,
"P4-B4 policy terminal state drift");
const deferred = evidence.deferred_semantics;
assert(deferred.semantic_fixture_id === "swarm-disaster.fixture.curio-lifecycle"
  && deferred.ordered_operation_count === 3 && deferred.expected_fact_count === 4
  && deferred.source_record_count === 3 && deferred.execution_batch === "G20-P5-B1"
  && deferred.state === "Pending" && deferred.mechanic_rule_batch === "G20-P5-M08"
  && deferred.mechanic_rule_state === "Pending", "P4-B4 deferred fixture overclaim");
const validation = evidence.validation;
assert(validation.runtime_json_file_reads === 0 && validation.second_activity_state_machine_added === false
  && validation.new_public_mode_types === 0 && validation.public_runtime_methods_added === 18
  && validation.public_reexports_added === 0 && validation.source_policy_handwritten_files === 897
  && validation.source_policy_public_reexports === 72, "P4-B4 validation drift");
for (const [file, maximum] of [
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs", 200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/content_runtime.rs", 1200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/path_runtime.rs", 1200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/content_runtime_tests.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_content/inventory_access.rs", 800],
]) assert(text(file).split(/\r?\n/u).length <= maximum, `P4-B4 source boundary exceeded: ${file}`);
const dispositions = json("content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json");
const obligations = dispositions.source_obligations.filter((row) => row.execution_batch === "G20-P4-B4");
assert(obligations.length === 564 && new Set(obligations.map((row) => row.id)).size === 564,
  "P4-B4 obligation exact-once drift");
const frozen = dispositions.policy_boundaries.find((row) => row.id === policy.boundary_id);
assert(frozen?.current_state === "InheritedPolicy" && frozen.implementation_batches.length === 1
  && frozen.implementation_batches[0] === "G20-P4-B4", "frozen policy assignment drift");
for (const protectedRoot of ["evidence/swarm-disaster-reference-v1", "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1", "config/swarm-disaster/data", "config/swarm-disaster-generated"])
  assert(captureGit(["status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot]).trim() === "",
    `protected root changed: ${protectedRoot}`);
const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P4-B4` | `Complete` |")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P4-B5` |")
  && status.includes("15 inherited / 16 terminal / 20 pending"), "Goal 20 did not advance after P4-B4");
const tests = evidence.tests;
assert(tests.content_runtime_unit_passed === 6 && tests.entry_lifecycle_unit_passed === 70
  && tests.swarm_unit_passed === 81 && tests.identity_integration_passed === 5
  && tests.clippy_passed === true && tests.quick_gate_result === "Pass"
  && nonPending(tests.quick_gate_seconds) && tests.full_gate_passed === true
  && nonPending(tests.full_gate_seconds), "P4-B4 terminal tests are incomplete");
console.log("Goal 20 P4-B4 verified (564 obligations, 144 shared Blessings, 66 Swarm Curios).");

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function captureGit(args) { return execFileSync("git", args, { cwd: root, encoding: "utf8" }); }
function assert(condition, message) { if (!condition) throw new Error(message); }
function nonPending(value) { return value !== null && value !== undefined && value !== "Pending"; }
