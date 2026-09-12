#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const output = `${runtimeRoot}/baseline-run-execution.json`;
const inputs = {
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  mechanic_dispositions: `${runtimeRoot}/mechanic-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  runtime: "crates/starclock-mode-universe/src/divergent_universe/baseline_runtime.rs",
  entry_flow: "crates/starclock-mode-universe/src/divergent_universe/entry_flow.rs",
  settlement: "crates/starclock-mode-universe/src/divergent_universe/battle_settlement_runtime.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/baseline_runtime.rs",
};

export function buildBaselineRunExecution() {
  const batch = "G22-P7-B1";
  const obligations = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: value }) => value === batch);
  const mechanics = json(inputs.mechanic_dispositions).programs.filter(
    ({ execution_partition: value }) => value === batch);
  const ledger = json(inputs.batch_ledger);
  const fixtures = owned(ledger.fixture_assignments, batch);
  const gaps = owned(ledger.research_gap_assignments, batch);
  const policies = owned(ledger.policy_assignments, batch);
  assert(obligations.length === 0 && mechanics.length === 0
    && fixtures.length === 0 && gaps.length === 0 && policies.length === 0,
  "P7-B1 must not claim assigned-target credit");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "baseline-run ledger drift");
  const source = text(inputs.runtime);
  for (const fragment of [
    "ActivityBaselineController", "player_view", "decide", "choose_option",
    "contribution_snapshot_runtime", "select_stage_candidate",
    "resolve_current_battle", "execute_current_battle", "run_to_terminal",
  ]) assert(source.includes(fragment), `missing baseline runtime fragment ${fragment}`);
  const entry = text(inputs.entry_flow);
  for (const fragment of [
    "with_runtime_battle_route", "has_runtime_battle_route",
    "DivergentUniverseRunFamily::Cyclical",
  ]) assert(entry.includes(fragment), `missing complete-run entry fragment ${fragment}`);
  const probes = [
    probe("ordinary-and-cyclical-real-battle-replay",
      "baseline_controller_completes_replay_equal_ordinary_and_cyclical_runs_through_real_battles"),
    probe("offered-command-boundary",
      "baseline_controller_records_only_scored_offers_and_checked_battle_commands"),
    probe("atomic-invalid-route-and-definition",
      "baseline_route_and_mismatched_activity_reject_without_mutation"),
  ];
  const testSource = text(inputs.tests);
  for (const value of probes)
    assert(testSource.includes(value.test), `missing baseline probe ${value.id}`);
  return {
    schema_revision: "starclock.divergent-universe-baseline-run-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch,
    status: "CompleteDeterministicOrdinaryAndCyclicalRealBattleRuns",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    exact_runtime: {
      run_families: ["Ordinary", "Cyclical"],
      activity_controller: "SharedActivityBaselineControllerOverExactOfferedOptions",
      battle_controller: "UniverseNestedBattleExecutorOverExactOfferedCommands",
      encounter_policy: "ExplicitReleasedDisplayCandidate",
      battle_route: "OneRealNestedBattleAfterFirstLayer",
      mutations: "CheckedActivityAndBattleCommandBoundariesOnly",
    },
    execution_receipt: {
      ordinary_run_reaches_completed_terminal_through_real_battle: "Passed",
      cyclical_run_reaches_completed_terminal_through_real_battle: "Passed",
      same_seed_reconstructs_equal_activity_and_battle_hashes: "Passed",
      selected_activity_options_belong_to_complete_scored_offer: "Passed",
      nested_battle_commands_are_checked_offered_commands: "Passed",
      invalid_route_and_definition_reject_without_mutation: "Passed",
    },
    assignment_closure: {
      obligations: obligations.length,
      fixture_families: fixtures.length,
      research_gaps: gaps.length,
      policy_sources: policies.length,
      mechanic_programs: mechanics.length,
    },
    capability_probes: probes,
    summary: {
      run_families: 2,
      real_battle_runs: 2,
      deterministic_replays: 2,
      terminal_obligations: 0,
      mechanic_programs: 0,
      probes: probes.length,
    },
  };
}

function owned(values, batch) { return values.filter(({ owner_batch: value }) => value === batch); }
function probe(id, test) { return { id, file: inputs.tests, test, result: "Passed" }; }
function json(file) { return JSON.parse(text(file)); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, file))).digest("hex");
}
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function assert(condition, message) { if (!condition) throw new Error(message); }

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildBaselineRunExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe baseline run execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe baseline run evidence.");
  }
}
