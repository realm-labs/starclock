#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/mechanics/semantic-fixture-execution.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-semantic-fixture-execution.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1" && evidence.batch === "G20-P5-B1"
  && evidence.result === "Pass", "Goal 20 P5-B1 evidence drift");
const frozen = evidence.frozen_disposition;
assert(frozen.path === "content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json"
  && sha256File(frozen.path) === frozen.sha256
  && frozen.fixture_source_path === "content-reference/swarm-disaster-v1/review-fixtures.json"
  && sha256File(frozen.fixture_source_path) === frozen.fixture_source_sha256,
  "P5-B1 frozen input drift");
const dispositions = json(frozen.path).semantic_fixtures;
const rows = dispositions.map((fixture) => ({
  id: fixture.id,
  family_id: fixture.family_id,
  ownership: fixture.ownership,
  evidence_quality: fixture.evidence_quality,
  ordered_operation_count: fixture.ordered_operation_count,
  expected_fact_count: fixture.expected_fact_count,
  target_runtime_disposition: fixture.target_runtime_disposition,
  execution_batch: fixture.execution_batch,
}));
assert(dispositions.length === 23
  && sha256(dispositions.map((fixture) => fixture.id).join("\n")) === frozen.ordered_fixture_ids_sha256
  && sha256(JSON.stringify(rows)) === frozen.disposition_rows_sha256
  && sha256(dispositions.flatMap((fixture) => fixture.source_record_ids).join("\n"))
    === frozen.source_record_ids_sha256
  && dispositions.every((fixture) => fixture.execution_batch === "G20-P5-B1"
    && fixture.target_runtime_disposition === "ProductionSemanticFixture"
    && fixture.ownership === "SwarmDisaster"), "P5-B1 fixture disposition drift");
const denominators = evidence.fixture_denominators;
assert(denominators.fixture_families === 23 && denominators.ordered_operations === 85
  && denominators.expected_facts === 108 && denominators.source_record_bindings === 76
  && denominators.project_policy_fixtures === 20 && denominators.exact_structured_fixtures === 3
  && denominators.terminal_mechanic_rules === 23
  && denominators.terminal_disposition === "ProductionSemanticFixture", "P5-B1 denominator drift");
assert(dispositions.reduce((sum, fixture) => sum + fixture.ordered_operation_count, 0) === 85
  && dispositions.reduce((sum, fixture) => sum + fixture.expected_fact_count, 0) === 108
  && dispositions.reduce((sum, fixture) => sum + fixture.source_record_ids.length, 0) === 76,
  "P5-B1 fixture payload drift");
const runtime = evidence.runtime;
assert(runtime.execution_revision === "swarm-disaster-semantic-fixture-execution-v1"
  && runtime.binding_api === "SwarmDisasterRuntimeFactory::semantic_fixture_execution_digest"
  && runtime.execution_digest === "1171feaf374e837b1c0bd863be336fc29eaf506bb61df53b1ff55e5768e9f25b"
  && runtime.production_runtime_regressions === 22 && runtime.production_catalog_probes === 1
  && runtime.runtime_json_file_reads === 0, "P5-B1 runtime contract drift");
const regressions = evidence.production_regressions;
assert(Object.keys(regressions).length === 23
  && JSON.stringify(Object.keys(regressions))
    === JSON.stringify(dispositions.map((fixture) => fixture.family_id))
  && new Set(Object.values(regressions)).size === 23, "P5-B1 regression mapping drift");
const regressionSource = rustTestSource("crates/starclock-mode-universe/src/swarm_disaster_entry");
for (const [family, regression] of Object.entries(regressions))
  assert(regressionSource.includes(`fn ${regression}(`), `missing production regression ${family}: ${regression}`);
const encounter = evidence.encounter_catalog_probe;
assert(encounter.fixture_id === "swarm-disaster.fixture.encounter-selection"
  && encounter.source_group_id === "swarm-disaster.encounter-group.120001"
  && encounter.source_wave_id === "swarm-disaster.encounter-wave.120001.1200011.1"
  && encounter.enemy_slot_count === 3 && encounter.boss_pool_count === 1
  && encounter.fixture_disposition === "ProductionCatalogProbe"
  && encounter.actual_selection_execution_owner === "G20-P6-B1"
  && encounter.battle_spec_owner === "G20-P6-B2", "P5-B1 encounter boundary drift");
assert(Object.values(evidence.validation).every(Boolean), "P5-B1 validation evidence drift");
const api = evidence.api_and_policy;
assert(api.new_public_mode_types === 0 && api.public_runtime_methods_added === 1
  && api.public_reexports_added === 0 && api.source_policy_handwritten_files === 932
  && api.source_policy_public_reexports === 72 && api.second_activity_state_machine_added === false,
  "P5-B1 API or source-policy drift");
const source = text("crates/starclock-mode-universe/src/swarm_disaster_entry/semantic_fixture_runtime.rs");
for (const literal of ["semantic_fixture_execution_digest", "ProductionRuntime", "ProductionCatalogProbe",
  "encounter_fixture_shape", "swarm-disaster-semantic-fixture-execution-v1"])
  assert(source.includes(literal), `missing P5-B1 runtime contract ${literal}`);
for (const forbidden of ["std::fs", "read_to_string", "SystemTime", "HashMap", "f32", "f64"])
  assert(!source.includes(forbidden), `P5-B1 fixture runtime gained forbidden dependency ${forbidden}`);
for (const [relative, maximum] of [
  ["crates/starclock-mode-universe/src/swarm_disaster_content/mod.rs", 200],
  ["crates/starclock-mode-universe/src/swarm_disaster_content/semantic_access.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs", 200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/semantic_fixture_runtime.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/semantic_fixture_runtime_tests.rs", 800],
]) assert(physicalLineCount(text(relative)) <= maximum, `P5-B1 source boundary exceeded: ${relative}`);
for (const protectedRoot of ["evidence/swarm-disaster-reference-v1", "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1", "config/swarm-disaster/data", "config/swarm-disaster-generated"])
  assert(captureGit(["status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot]).trim() === "",
    `protected root changed: ${protectedRoot}`);
const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| Active batch | None |") && status.includes("| Next unblocked batch | `G20-P5-B2` |")
  && status.includes("| `G20-P5-B1` | `Complete` |"), "G20-P5-B1 ledger is incomplete");
const tests = evidence.tests;
assert(tests.focused_semantic_fixture_tests_passed === 4 && tests.entry_suite_passed === 118
  && tests.swarm_suite_passed === 129 && tests.identity_integration_passed === 5
  && tests.clippy_passed === true && tests.dependency_policy_passed === true
  && tests.source_policy_passed === true && tests.quick_gate_passed === true
  && nonPending(tests.quick_gate_seconds) && ["CacheHit", "CacheMiss"].includes(tests.quick_rust_receipt)
  && tests.full_gate_required === true && tests.full_gate_passed === true
  && nonPending(tests.full_gate_seconds), "P5-B1 terminal tests are incomplete");
console.log("Goal 20 P5-B1 verified (23/23 semantic fixtures bind to production regressions or the explicit pre-P6 encounter catalog probe).");

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(value) { return crypto.createHash("sha256").update(value).digest("hex"); }
function sha256File(relative) { return sha256(fs.readFileSync(path.join(root, relative))); }
function rustTestSource(relative) { return fs.readdirSync(path.join(root, relative), { withFileTypes: true })
  .filter((entry) => entry.isFile() && (entry.name === "tests.rs" || entry.name.endsWith("_tests.rs")))
  .map((entry) => text(path.join(relative, entry.name))).join("\n"); }
function captureGit(args) { return execFileSync("git", args, { cwd: root, encoding: "utf8" }); }
function physicalLineCount(contents) { const lines = contents.split(/\r?\n/u); return lines.at(-1) === "" ? lines.length - 1 : lines.length; }
function nonPending(value) { return value !== null && value !== undefined && value !== "Pending"; }
function assert(condition, message) { if (!condition) throw new Error(message); }
