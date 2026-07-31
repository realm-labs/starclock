#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/mechanics/semantic-fixture-execution.json",
);
assert(evidence.schema_revision
  === "starclock.gold-and-gears-semantic-fixture-execution.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P5-B1"
  && evidence.result === "Pass",
"Goal 14 P5-B1 evidence drift");

const frozen = evidence.frozen_disposition;
assert(frozen.path
  === "content-manifests/gold-and-gears-runtime-v1/runtime-dispositions.json"
  && sha256File(frozen.path) === frozen.sha256
  && frozen.fixture_source_path
    === "content-reference/gold-and-gears-v1/review-fixtures.json"
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
assert(dispositions.length === 18
  && sha256(dispositions.map((fixture) => fixture.id).join("\n"))
    === frozen.ordered_fixture_ids_sha256
  && sha256(JSON.stringify(rows)) === frozen.disposition_rows_sha256
  && sha256(dispositions.flatMap((fixture) => fixture.source_record_ids).join("\n"))
    === frozen.source_record_ids_sha256
  && dispositions.every((fixture) =>
    fixture.execution_batch === "G14-P5-B1"
      && fixture.target_runtime_disposition === "ProductionSemanticFixture"
      && fixture.ownership === "GoldAndGears"),
"P5-B1 fixture disposition drift");

const denominators = evidence.fixture_denominators;
assert(denominators.fixture_families === 18
  && denominators.ordered_operations === 63
  && denominators.expected_facts === 36
  && denominators.source_record_bindings === 46
  && denominators.project_policy_fixtures === 13
  && denominators.exact_public_text_fixtures === 1
  && denominators.exact_structured_fixtures === 4
  && denominators.mechanic_fixture_families === 9
  && denominators.system_fixture_families === 9
  && denominators.terminal_mechanic_rules === 1224
  && denominators.terminal_disposition === "ProductionSemanticFixture",
"P5-B1 fixture denominator drift");
assert(dispositions.reduce((sum, fixture) =>
  sum + fixture.ordered_operation_count, 0) === denominators.ordered_operations
  && dispositions.reduce((sum, fixture) =>
    sum + fixture.expected_fact_count, 0) === denominators.expected_facts
  && dispositions.reduce((sum, fixture) =>
    sum + fixture.source_record_ids.length, 0) === denominators.source_record_bindings,
"P5-B1 fixture payload denominator drift");

const mechanicRules = evidence.mechanic_rule_families;
assert(Object.keys(mechanicRules).length === 9
  && Object.values(mechanicRules).reduce((sum, count) => sum + count, 0) === 1224
  && mechanicRules["profile-entry"] === 5
  && mechanicRules["conundrum-stats"] === 6
  && mechanicRules["conundrum-auxiliary"] === 6
  && mechanicRules["neural-network-effect"] === 40
  && mechanicRules["curio-lifecycle"] === 160
  && mechanicRules["occurrence-choice"] === 384
  && mechanicRules["service-and-adventure"] === 38
  && mechanicRules["path-boost"] === 495
  && mechanicRules["resonance-extrapolation"] === 90,
"P5-B1 mechanic family drift");

const runtime = evidence.runtime;
assert(runtime.execution_revision
  === "gold-and-gears-semantic-fixture-execution-v1"
  && runtime.binding_api
    === "GoldAndGearsRuntimeFactory::semantic_fixture_bindings"
  && runtime.execution_digest
    === "2b69ec29dde6fde1dc6cac9ea10baea5d34c28f39d3d03a41f74d5f340b52832"
  && runtime.production_runtime_regressions === 17
  && runtime.production_catalog_probes === 1
  && runtime.runtime_json_file_reads === 0,
"P5-B1 runtime contract drift");

const regressions = evidence.production_regressions;
assert(Object.keys(regressions).length === 18
  && JSON.stringify(Object.keys(regressions))
    === JSON.stringify(dispositions.map((fixture) => fixture.family_id))
  && new Set(Object.values(regressions)).size === 18,
"P5-B1 production regression mapping drift");
const regressionSource = rustTestSource(
  "crates/starclock-mode-universe/src/gold_gears_entry",
);
for (const [family, regression] of Object.entries(regressions))
  assert(regressionSource.includes(`fn ${regression}(`),
    `P5-B1 missing production regression ${family}: ${regression}`);

const encounter = evidence.encounter_catalog_probe;
assert(encounter.fixture_id === "gold-gears.fixture.encounter-selection"
  && encounter.source_group_id === "223003"
  && encounter.enemy_slot_count === 2
  && encounter.boss_choice_count === 2
  && encounter.fixture_disposition === "ProductionCatalogProbe"
  && encounter.actual_selection_execution_owner === "G14-P6-B1",
"P5-B1 encounter metadata boundary drift");
assert(Object.values(evidence.validation).every(Boolean),
"P5-B1 validation evidence drift");

const tests = evidence.tests;
assert(tests.focused_semantic_fixture_tests_passed === 4
  && tests.entry_suite_passed === 112
  && tests.clippy_passed === true
  && tests.dependency_policy_passed === true
  && tests.workspace_check_passed === true
  && tests.quick_gate_passed === true
  && Number(tests.quick_gate_seconds) > 0
  && tests.quick_selected_harnesses > 0
  && tests.quick_direct_packages >= 1
  && tests.quick_downstream_packages_checked >= 0
  && ["CacheHit", "CacheMiss"].includes(tests.quick_rust_receipt)
  && Number(tests.final_quick_gate_seconds) > 0
  && tests.final_quick_rust_receipt === "CacheHit"
  && Number.isInteger(tests.final_quick_deferred_inputs)
  && tests.full_gate_required === true
  && tests.full_gate_passed === true
  && Number(tests.full_gate_seconds) > 0
  && tests.full_workspace_harnesses > 0
  && Number.isInteger(tests.full_cache_dependent_checks_skipped),
"P5-B1 test evidence drift");

const source = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/semantic_fixture_runtime.rs",
);
for (const literal of [
  "semantic_fixture_bindings",
  "ProductionRuntime",
  "ProductionCatalogProbe",
  "encounter_selection_fixture_shape",
  "gold-gears.encounter-group.223003",
])
  assert(source.includes(literal), `missing P5-B1 runtime contract ${literal}`);
for (const forbidden of [
  "std::fs",
  "read_to_string",
  "SystemTime",
  "HashMap",
  "f32",
  "f64",
])
  assert(!source.includes(forbidden),
    `P5-B1 fixture runtime gained forbidden dependency ${forbidden}`);

for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_content/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/content_link_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/semantic_fixture_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/semantic_fixture_runtime_tests.rs",
])
  assert(physicalLineCount(text(relative)) <= 1200,
    `P5-B1 source exceeds handwritten limit: ${relative}`);

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

const status = text("docs/goals/14-gold-and-gears-runtime-status.md");
assert(status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G14-P5-B2` |")
  && status.includes("| `G14-P5-B1` | `Complete` |"),
"G14-P5-B1 ledger is incomplete");

console.log(
  "Goal 14 P5-B1 verified (18/18 semantic fixture families bound to " +
  "production runtime regressions or the explicit pre-P6 catalog probe).",
);

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}
function sha256File(relative) {
  return crypto.createHash("sha256").update(
    fs.readFileSync(path.join(root, relative)),
  ).digest("hex");
}
function rustTestSource(relative) {
  return fs.readdirSync(path.join(root, relative), { withFileTypes: true })
    .filter((entry) => entry.isFile()
      && (entry.name === "tests.rs" || entry.name.endsWith("_tests.rs")))
    .map((entry) => text(path.join(relative, entry.name)))
    .join("\n");
}
function captureGit(args) {
  return execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
}
function physicalLineCount(contents) {
  const lines = contents.split(/\r?\n/u);
  return lines.at(-1) === "" ? lines.length - 1 : lines.length;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
