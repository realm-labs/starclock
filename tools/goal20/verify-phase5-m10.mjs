#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/mechanics/service-rules.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-service-rule-execution.v1"
  && evidence.batch === "G20-P5-M10" && evidence.result === "Pass", "P5-M10 identity drift");
const partition = evidence.partition;
assert(partition.executor === "SwarmServiceRuleExecutor" && partition.expected_rules === 1
  && partition.executed_rules === 1 && partition.exact_structured_rules === 0
  && partition.project_policy_rules === 1 && partition.families.join(",") === "service-and-adventure",
  "P5-M10 partition drift");
const runtime = evidence.runtime_binding;
assert(runtime.revision === "swarm-disaster-service-rule-runtime-v1" && runtime.domain === "Activity"
  && runtime.source_disposition === "ReferenceOnly"
  && runtime.runtime_disposition === "ProductionActivityProgramsAndExternalSettlement"
  && runtime.rule_id === "swarm-disaster.mechanic-rule.service-and-adventure"
  && runtime.triggers.join(",") === "ServicePurchaseAccepted,AdventureOutcomeOffered"
  && runtime.ordered_operation_count === 3 && runtime.unresolved_behavior === "FailClosed"
  && runtime.service_runtime === "ExistingServiceAdventureRuntimeCatalog", "Service binding drift");
const execution = evidence.production_execution;
assert(execution.services === 15 && execution.service_rules === 15 && execution.beacon_rules === 4
  && execution.blessing_shops === 5 && execution.curio_shops === 4 && execution.adventure_outcomes === 6
  && execution.capture_monster_rooms === 3 && execution.destroy_prop_rooms === 3
  && execution.initial_cosmic_fragments === 50
  && execution.accepted_external_tiers.join(",") === "Tier1,Tier2,Tier3"
  && execution.action_input_simulation === "Excluded" && execution.shop_rng_label === "Shop"
  && execution.insufficient_resource === "AtomicReject"
  && execution.stale_purchase_or_settlement === "AtomicReject"
  && execution.fixture_service === "swarm-disaster.service.universe-service-shop-100011"
  && execution.fixture_beacon === "swarm-disaster.service-rule.beacon.1"
  && execution.fixture_adventure === "swarm-disaster.adventure-outcome.1210601", "Service production drift");
assert(evidence.compatibility.runtime_digest
  === "ab5aa4deba286f6b387f6d60425e2026e69a812ec92c1c94b9f3c724201576de"
  && evidence.compatibility.service_catalog_digest
    === "71d9b473f30b853b58c2cd5e56f02c9620093d3975a542fbcdf3fc4acebb1d80"
  && evidence.compatibility.adventure_catalog_digest
    === "e174154cd9307d88075ffc2cad131ed03bc5c1440b86350b33092b46844762f3"
  && evidence.compatibility.selected_blessing_ids.join(",") === "140,34"
  && evidence.compatibility.selected_curio_ids.join(",") === "1011,1002", "P5-M10 digest drift");
const fixtures = evidence.deferred_fixtures;
assert(fixtures.fixture_ids.join(",") === "swarm-disaster.fixture.service-and-adventure"
  && fixtures.ordered_operation_count === 3 && fixtures.expected_fact_count === 4
  && fixtures.source_record_count === 3 && fixtures.execution_batch === "G20-P5-B1"
  && fixtures.state === "Pending" && fixtures.production_binding_state === "PreTerminalProductionParity",
  "P5-M10 fixture overclaim");
const validation = evidence.validation;
assert(validation.runtime_json_file_reads === 0 && validation.second_activity_state_machine_added === false
  && validation.new_public_mode_types === 0 && validation.public_runtime_methods_added === 1
  && validation.public_reexports_added === 0 && validation.source_policy_handwritten_files === 925
  && validation.source_policy_public_reexports === 72
  && validation.private_serde_json_owner_registered.endsWith("service_rule_runtime.rs"),
  "P5-M10 validation drift");
for (const [file, maximum] of [
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs", 200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/service_adventure_runtime.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/service_rule_runtime.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/service_rule_runtime_tests.rs", 800],
]) assert(text(file).trimEnd().split(/\r?\n/u).length <= maximum, `P5-M10 source boundary exceeded: ${file}`);
const frozen = json("content-manifests/swarm-disaster-runtime-v1/rule-partitions.json")
  .partitions.find((row) => row.id === "G20-P5-M10");
assert(frozen?.expected_rules === 1 && frozen.executor === partition.executor
  && frozen.rule_ids.join(",") === runtime.rule_id
  && frozen.fixture_ids.join(",") === fixtures.fixture_ids.join(","), "frozen P5-M10 assignment drift");
const dispositions = json("content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json");
assert(dispositions.mechanic_rules.some((rule) => rule.id === runtime.rule_id
  && rule.implementation_batch === "G20-P5-M10" && rule.current_state === "Pending"
  && rule.native_handler_id === null), "P5-M10 disposition drift");
for (const protectedRoot of ["evidence/swarm-disaster-reference-v1", "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1", "config/swarm-disaster/data", "config/swarm-disaster-generated"])
  assert(captureGit(["status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot]).trim() === "",
    `protected root changed: ${protectedRoot}`);
const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P5-M10` | `Complete` |") && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P5-M11` |"), "Goal 20 did not advance after P5-M10");
const tests = evidence.tests;
assert(tests.service_rule_unit_passed === 3 && tests.entry_lifecycle_unit_passed === 108
  && tests.swarm_unit_passed === 119 && tests.identity_integration_passed === 5
  && tests.clippy_passed === true && tests.dependency_policy_passed === true
  && tests.source_policy_passed === true && tests.quick_gate_result === "Pass"
  && nonPending(tests.quick_gate_seconds) && tests.full_gate_passed === true
  && nonPending(tests.full_gate_seconds), "P5-M10 terminal tests are incomplete");
console.log("Goal 20 P5-M10 verified (Service purchases and external Adventure settlement execute through the existing Activity runtime).");

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function captureGit(args) { return execFileSync("git", args, { cwd: root, encoding: "utf8" }); }
function assert(condition, message) { if (!condition) throw new Error(message); }
function nonPending(value) { return value !== null && value !== undefined && value !== "Pending"; }
