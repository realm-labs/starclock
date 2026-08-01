#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/mechanics/path-rules.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-path-rule-execution.v1"
  && evidence.batch === "G20-P5-M07" && evidence.result === "Pass", "P5-M07 identity drift");
const partition = evidence.partition;
assert(partition.executor === "SwarmPathRuleExecutor" && partition.expected_rules === 2
  && partition.executed_rules === 2 && partition.exact_structured_rules === 0
  && partition.project_policy_rules === 2 && partition.families.join(",")
    === "path-and-propagation-unlock,resonance-interplay", "P5-M07 partition drift");
const runtime = evidence.runtime_binding;
assert(runtime.revision === "swarm-disaster-path-rule-runtime-v1"
  && runtime.domains.join(",") === "Activity,CrossBattle" && runtime.source_disposition === "ReferenceOnly"
  && runtime.rule_ids.every((id, index) => id === `swarm-disaster.mechanic-rule.${partition.families[index]}`)
  && runtime.ordered_operation_counts.join(",") === "3,4" && runtime.unresolved_behavior === "FailClosed"
  && runtime.path_runtime === "ExistingPathRuntimeCatalog", "Path binding drift");
const execution = evidence.production_execution;
assert(execution.paths === 8 && execution.path_boosts === 8 && execution.resonances === 32
  && execution.base_resonances === 8 && execution.formations === 24 && execution.propagation_paths === 1
  && execution.propagation_unlock === "swarm-disaster.pathstrider-unlock.1000008"
  && execution.resonance_interplays === 16 && execution.interplays_per_path === 2
  && execution.main_path_threshold === 3 && execution.sub_path_threshold === 3
  && execution.counting_policy === "DistinctOwnedBlessingIdentity" && execution.rng_draws === 0
  && execution.stale_program === "AtomicReject", "Path production drift");
assert(evidence.compatibility.runtime_digest
  === "a421ce1b0868170f00273ee9e72f021399732b9b154d46126aad5a50821d4cc6"
  && evidence.compatibility.propagation_path_digest
    === "649f1d4c80be34556fd0c0e00bf1dc866815487b27e1371dd88631f464cd11b2", "P5-M07 digest drift");
const fixtures = evidence.deferred_fixtures;
assert(fixtures.ordered_operation_count === 7 && fixtures.expected_fact_count === 9
  && fixtures.source_record_count === 5 && fixtures.execution_batch === "G20-P5-B1"
  && fixtures.state === "Pending" && fixtures.production_binding_state === "PreTerminalProductionParity",
  "P5-M07 fixture overclaim");
const validation = evidence.validation;
assert(validation.runtime_json_file_reads === 0 && validation.second_activity_state_machine_added === false
  && validation.new_public_mode_types === 0 && validation.public_runtime_methods_added === 1
  && validation.public_reexports_added === 0 && validation.source_policy_handwritten_files === 919
  && validation.source_policy_public_reexports === 72
  && validation.private_serde_json_owner_registered.endsWith("path_rule_runtime.rs"), "P5-M07 validation drift");
for (const [file, maximum] of [
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs", 200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/path_rule_runtime.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/path_rule_runtime_tests.rs", 800],
]) assert(text(file).trimEnd().split(/\r?\n/u).length <= maximum, `P5-M07 source boundary exceeded: ${file}`);
const frozen = json("content-manifests/swarm-disaster-runtime-v1/rule-partitions.json")
  .partitions.find((row) => row.id === "G20-P5-M07");
assert(frozen?.expected_rules === 2 && frozen.executor === partition.executor
  && frozen.rule_ids.join(",") === runtime.rule_ids.join(",")
  && frozen.fixture_ids.join(",") === fixtures.fixture_ids.join(","), "frozen P5-M07 assignment drift");
const dispositions = json("content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json");
assert(runtime.rule_ids.every((id) => dispositions.mechanic_rules.some((rule) =>
  rule.id === id && rule.implementation_batch === "G20-P5-M07"
    && rule.current_state === "Pending" && rule.native_handler_id === null)), "P5-M07 disposition drift");
for (const protectedRoot of ["evidence/swarm-disaster-reference-v1", "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1", "config/swarm-disaster/data", "config/swarm-disaster-generated"])
  assert(captureGit(["status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot]).trim() === "",
    `protected root changed: ${protectedRoot}`);
const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P5-M07` | `Complete` |") && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P5-M08` |"), "Goal 20 did not advance after P5-M07");
const tests = evidence.tests;
assert(tests.path_rule_unit_passed === 3 && tests.entry_lifecycle_unit_passed === 99
  && tests.swarm_unit_passed === 110 && tests.identity_integration_passed === 5
  && tests.clippy_passed === true && tests.quick_gate_result === "Pass"
  && nonPending(tests.quick_gate_seconds) && tests.full_gate_passed === true
  && nonPending(tests.full_gate_seconds), "P5-M07 terminal tests are incomplete");
console.log("Goal 20 P5-M07 verified (2/2 Path rules execute through the existing Path runtime).");

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function captureGit(args) { return execFileSync("git", args, { cwd: root, encoding: "utf8" }); }
function assert(condition, message) { if (!condition) throw new Error(message); }
function nonPending(value) { return value !== null && value !== undefined && value !== "Pending"; }
