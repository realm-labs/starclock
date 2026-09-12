#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const output = `${runtimeRoot}/matrix-execution.json`;
const inputs = {
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  mechanic_dispositions: `${runtimeRoot}/mechanic-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  verification_contract: `${runtimeRoot}/verification-contract.json`,
  baseline_fixture:
    "crates/starclock-mode-universe/src/divergent_universe/baseline_fixture.rs",
  baseline_replay:
    "crates/starclock-mode-universe/src/divergent_universe/baseline_replay.rs",
  matrix_test: "crates/starclock-mode-universe/tests/divergent_universe_matrix.rs",
  package: "crates/starclock-mode-universe/Cargo.toml",
};

export function buildMatrixExecution() {
  const batch = "G22-P7-B6";
  const ledger = json(inputs.batch_ledger);
  const contract = json(inputs.verification_contract);
  const obligations = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: value }) => value === batch);
  const mechanics = json(inputs.mechanic_dispositions).programs.filter(
    ({ execution_partition: value }) => value === batch);
  const fixtures = owned(ledger.fixture_assignments, batch);
  const gaps = owned(ledger.research_gap_assignments, batch);
  const policies = owned(ledger.policy_assignments, batch);
  assert(obligations.length === 0 && mechanics.length === 0
    && fixtures.length === 0 && gaps.length === 0 && policies.length === 0,
  "P7-B6 must not claim assigned-target credit");
  assert(contract.status === "BehavioralAuditIncomplete"
    && contract.matrix_cases.length === 104
    && contract.matrix_cases.every(({ status }) => status === "PendingGameplayAxisExecution"),
  "matrix execution status drift");

  const test = text(inputs.matrix_test);
  for (const fragment of [
    "flow_for_selection", "record_divergent_universe_selected_run",
    "verify_divergent_universe_selected_replay", "completed_battles()",
    "battle_command_count()", "collect_targets", "axis_denominators",
  ]) assert(test.includes(fragment), `missing matrix test fragment ${fragment}`);
  assert(text(inputs.package).includes('name = "divergent_universe_matrix"'),
    "matrix test target is not explicit");

  const axes = collectAxes(contract.matrix_cases);
  for (const [axis, denominator] of Object.entries(contract.axis_denominators))
    assert((axes.get(axis) ?? new Set()).size === denominator,
      `${axis} matrix execution denominator drift`);
  const familyCounts = countBy(contract.matrix_cases,
    ({ selected_run: { run_family: family } }) => family);

  return {
    schema_revision: "starclock.divergent-universe-matrix-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch,
    status: "TargetAssignmentDeclaredGameplayExecutionIncomplete",
    runtime_release_ready: false,
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    execution_contract: {
      command: "cargo test --release -p starclock-mode-universe --test divergent_universe_matrix -- --ignored --exact generated_legal_matrix_completes_real_battles_and_fresh_replay",
      cases: contract.matrix_cases.length,
      run_family_cases: familyCounts,
      selected_areas: contract.axis_denominators.areas,
      selected_difficulties: contract.axis_denominators.difficulties,
      selected_layers: contract.axis_denominators.layers,
      runtime_obligation_receipts: contract.axis_denominators.runtime_obligations,
      mechanic_partitions: contract.axis_denominators.mechanic_partitions,
      semantic_fixtures: contract.axis_denominators.semantic_fixtures,
      policy_sources: contract.axis_denominators.policy_sources,
    },
    verification_requirements: {
      result: "NotExecutedByGenerator",
      baseline_scope: "AreaDifficultyLayerJoinSingleBattleAndFreshReplay",
      gameplay_axis_execution: "PendingProductionFixtures",
      declaration_is_execution_evidence: false,
    },
    axis_assignment_counts: Object.fromEntries(
      [...axes.entries()].map(([axis, values]) => [axis, values.size])),
    assignment_closure: {
      obligations: obligations.length,
      fixture_families: fixtures.length,
      research_gaps: gaps.length,
      policy_sources: policies.length,
      mechanic_programs: mechanics.length,
      matrix_cases: contract.matrix_cases.length,
    },
    summary: {
      matrix_cases: contract.matrix_cases.length,
      real_battles: null,
      fresh_replays: null,
      run_families: Object.keys(familyCounts).length,
      selected_areas: contract.axis_denominators.areas,
      selected_difficulties: contract.axis_denominators.difficulties,
      selected_layers: contract.axis_denominators.layers,
    },
  };
}

function collectAxes(cases) {
  const result = new Map();
  for (const matrixCase of cases)
    for (const [axis, values] of Object.entries(matrixCase.targets)) {
      const targets = result.get(axis) ?? new Set();
      for (const value of values) assert(!targets.has(value), `${axis} duplicate ${value}`);
      for (const value of values) targets.add(value);
      result.set(axis, targets);
    }
  return result;
}
function countBy(values, keyOf) {
  const result = {};
  for (const value of values) {
    const key = keyOf(value);
    result[key] = (result[key] ?? 0) + 1;
  }
  return result;
}
function owned(values, batch) { return values.filter(({ owner_batch: value }) => value === batch); }
function json(file) { return JSON.parse(text(file)); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, file))).digest("hex");
}
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function assert(condition, message) { if (!condition) throw new Error(message); }

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildMatrixExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe matrix execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe matrix declaration (no execution claim).");
  }
}
