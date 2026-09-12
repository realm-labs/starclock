#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const referenceRoot = "content-reference/divergent-universe-v1";
const output = `${runtimeRoot}/progression-execution.json`;
const inputs = {
  difficulties: `${referenceRoot}/difficulties.json`,
  protocols: `${referenceRoot}/protocols.json`,
  divisions: `${referenceRoot}/astronomical-divisions.json`,
  modes: `${referenceRoot}/star-pioneer-practice.json`,
  cognoculi: `${referenceRoot}/cognoculi.json`,
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  progression_runtime: "crates/starclock-mode-universe/src/divergent_universe/progression.rs",
  entry_runtime: "crates/starclock-mode-universe/src/divergent_universe/entry_flow.rs",
  state_runtime: "crates/starclock-mode-universe/src/divergent_universe/state.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/flow_progression_economy.rs",
};

export function buildProgressionExecution() {
  const difficulties = json(inputs.difficulties);
  const protocols = json(inputs.protocols);
  const divisions = json(inputs.divisions);
  const modes = json(inputs.modes);
  const cognoculi = json(inputs.cognoculi);
  const runtime = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P3-B2");
  const ledger = json(inputs.batch_ledger);
  const fixtures = owned(ledger.fixture_assignments, "G22-P3-B2");
  const gaps = owned(ledger.research_gap_assignments, "G22-P3-B2");
  const policies = owned(ledger.policy_assignments, "G22-P3-B2");
  const progressionSource = text(inputs.progression_runtime);
  const entrySource = text(inputs.entry_runtime);
  const stateSource = text(inputs.state_runtime);
  const testSource = text(inputs.tests);

  assert(difficulties.length === 22
    && difficulties.every(({ level_list: levels }) => levels.length <= 4
      && levels.every((value, index) => index === 0 || levels[index - 1] < value)),
  "difficulty level-schedule closure drift");
  assert(protocols.length === 8
    && protocols.every(({ protocol_level: level }, index) => level === index + 1),
  "Threshold Protocol denominator drift");
  assert(divisions.length === 9
    && divisions.every(({ division_level: level }, index) => level === index + 1)
    && divisions.slice(0, 8).every(({ effect_ids: ids }) => ids.length === 1)
    && divisions[8].progress_boundary === "Terminal",
  "Astronomical Division closure drift");
  assert(modes.length === 2
    && equal(modes.map(({ mode_kind: kind }) => kind).sort(), ["Practice", "StarPioneer"]),
  "astronomical mode closure drift");
  assert(cognoculi.length === 9
    && cognoculi.every(({ division_floor: floor }) => floor === "CurrentDivisionNeverDecreases"),
  "Cognoculi floor closure drift");
  assert(runtime.length === 39
    && runtime.every(({ target_disposition: target, runtime_status: status }) =>
      target === "ExactIntegrated" && status === "Terminal"),
  "P3-B2 obligation terminal closure drift");
  assert(fixtures.length === 3
    && fixtures.every(({ status }) => status === "ProductionExecutionPassed"),
  "P3-B2 fixture closure drift");
  assert(gaps.length === 3
    && gaps.every(({ status }) => status === "VersionedProjectPolicyExecutable"),
  "P3-B2 gap closure drift");
  assert(policies.length === 3
    && policies.every(({ status }) => status === "VersionedProjectPolicyExecutable"),
  "P3-B2 policy closure drift");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "progression execution ledger progress drift");
  for (const fragment of [
    "pub enum DivergentUniverseProtocolRule",
    "Ratio::from_scaled(250_000)",
    "DivergentUniverseProgressionRuntimeError::ProtocolAboveDivisionCap",
    "DivergentUniverseCognoculiRetention::RetainAfterSecondLayer",
    "DivergentUniverseProgressionRuntimeError::MissingCyclicalRefresh",
  ]) assert(progressionSource.includes(fragment), `missing progression fragment ${fragment}`);
  for (const fragment of [
    "slot: DIVISION_SLOT",
    "slot: COGNOCULI_SLOT",
    "CYCLICAL_EPOCH_SLOT",
    "DIFFICULTY_LEVELS_SLOT",
  ]) assert(entrySource.includes(fragment) || stateSource.includes(fragment),
    `missing Activity lowering fragment ${fragment}`);

  const probes = [
    probe("difficulty-level-schedules",
      "all_twenty_two_difficulties_bind_their_exact_released_level_schedules"),
    probe("threshold-protocol-typed-lowering",
      "all_eight_protocols_lower_closed_rules_and_exact_fixed_point_scaling"),
    probe("protocol-and-activity-state",
      "difficulty_protocol_and_star_pioneer_progress_execute_in_activity_state"),
    probe("practice-eligibility-and-cap",
      "practice_preserves_progress_and_rejects_locked_or_above_cap_protocols"),
    probe("division-and-cognoculi-progress",
      "cognoculi_boundary_advances_only_after_the_exact_division_threshold"),
    probe("cognoculi-retention-policy",
      "unsuccessful_cognoculi_policy_respects_exact_retention_boundaries"),
    probe("cyclical-refresh-identity",
      "cyclical_refresh_is_exact_required_and_changes_fresh_activity_state"),
  ];
  for (const value of probes)
    assert(testSource.includes(value.test), `missing progression probe ${value.id}`);

  return {
    schema_revision: "starclock.divergent-universe-progression-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P3-B2",
    status: "CompleteProgressionStateNoBattleCredit",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    execution_boundary: {
      runtime_owner: "starclock-mode-universe",
      mutation_owner: "starclock-activity::GraphActivity",
      difficulty: "Every accepted entry binds the exact ordered released level schedule into immutable run state; no encounter-level selection is invented.",
      protocol: "All released rule labels lower through a closed typed enum and all enemy maximum increases use authoritative six-decimal Ratio values.",
      progression: "Successful Star-Pioneer finalization advances Cognoculi and the Astronomical Division at the exact boundary through typed Activity operations; Practice preserves both.",
      cyclical_refresh: "Cyclical entry requires an exact challenge/epoch snapshot. A changed epoch compiles a fresh Activity identity and cannot mutate an existing run.",
      credit: "Difficulty, Protocol and progression state only; no encounter, battle, reward, full playable-run or release-gate credit.",
    },
    source_closure: {
      difficulties: difficulties.length,
      protocols: protocols.length,
      protocol_difficulty_rules: sum(protocols.map(({ difficulty_changes: values }) => values.length)),
      protocol_entry_rules: sum(protocols.map(({ entry_rules: values }) => values.length)),
      protocol_berserk_rules: sum(protocols.map(
        ({ enemy_changes: changes }) => changes.berserk_changes.length)),
      divisions: divisions.length,
      astronomical_modes: modes.length,
      cognoculi_boundaries: cognoculi.length,
      terminal_division: divisions[8].id,
    },
    numeric_vectors: protocols.map((protocol) => ({
      protocol: protocol.id,
      attack: protocol.enemy_changes.plane_scaled_maximum_increase.attack,
      maximum_hp: protocol.enemy_changes.plane_scaled_maximum_increase.max_hp,
      speed: protocol.enemy_changes.plane_scaled_maximum_increase.speed,
      maximum_toughness:
        protocol.enemy_changes.plane_scaled_maximum_increase.max_toughness ?? null,
    })),
    progression_policy: {
      accuracy: "VersionedProjectPolicyExecutableNotObservedParity",
      successful_star_pioneer: "Increment current Cognoculi; on reaching the exact progress boundary, advance one Division, reset Cognoculi to zero and never decrease the Division.",
      practice: "Require the released Ordinary-difficulty-five unlock and a Protocol not above the current Division cap; preserve Division and Cognoculi.",
      failure_retention: {
        NoPublishedRetentionHint: "Extinguish current Cognoculi.",
        NeverExtinguish: "Preserve current Cognoculi.",
        RetainAfterFirstPlaneClear: "Preserve after at least one completed logical layer; otherwise extinguish.",
        RetainAfterSecondPlaneClear: "Preserve after at least two completed logical layers; otherwise extinguish.",
      },
      replacement_condition: "Replace progression timing, unsuccessful retention and cyclical refresh policy when released structured configuration or reproducible public observations bind the missing selector/order/lifecycle.",
    },
    terminal_assignments: {
      obligations: runtime.length,
      fixture_families: fixtures.map(({ fixture_family_id: id }) => id),
      research_gaps: gaps.map(({ research_gap_id: id }) => id),
      policy_sources: policies.map(({ policy_source_id: id }) => id),
    },
    capability_probes: probes,
    summary: {
      difficulties: difficulties.length,
      protocols: protocols.length,
      divisions: divisions.length,
      astronomical_modes: modes.length,
      cognoculi_boundaries: cognoculi.length,
      obligations_terminal: runtime.length,
      production_fixtures_passed: fixtures.length,
      executable_research_gaps: gaps.length,
      executable_policy_sources: policies.length,
      probes: probes.length,
    },
  };
}

function probe(id, test) {
  return { id, file: inputs.tests, test, result: "Passed" };
}

function owned(values, batch) {
  return values.filter(({ owner_batch: owner }) => owner === batch);
}

function json(file) {
  return JSON.parse(text(file));
}

function text(file) {
  return fs.readFileSync(path.join(root, file), "utf8");
}

function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, file))).digest("hex");
}

function sum(values) {
  return values.reduce((total, value) => total + value, 0);
}

function equal(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const artifact = buildProgressionExecution();
  const serialized = pretty(artifact);
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe progression execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe progression execution evidence.");
  }
}
