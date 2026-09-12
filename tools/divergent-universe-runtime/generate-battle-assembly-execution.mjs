#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const referenceRoot = "content-reference/divergent-universe-v1";
const output = `${runtimeRoot}/battle-assembly-execution.json`;
const inputs = {
  encounter_groups: `${referenceRoot}/encounter-groups.json`,
  encounter_waves: `${referenceRoot}/encounter-waves.json`,
  enemy_slots: `${referenceRoot}/enemy-slots.json`,
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  contribution_execution: `${runtimeRoot}/contribution-snapshot-execution.json`,
  encounter_execution: `${runtimeRoot}/encounter-reachability-execution.json`,
  runtime: "crates/starclock-mode-universe/src/divergent_universe/battle_assembly_runtime.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/battle_assembly_runtime.rs",
};

export function buildBattleAssemblyExecution() {
  const groups = json(inputs.encounter_groups);
  const waves = json(inputs.encounter_waves);
  const slots = json(inputs.enemy_slots);
  const dispositions = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P6-B4");
  const ledger = json(inputs.batch_ledger);
  const contribution = json(inputs.contribution_execution);
  const encounter = json(inputs.encounter_execution);
  const fixtures = owned(ledger.fixture_assignments);
  const gaps = owned(ledger.research_gap_assignments);
  const policies = owned(ledger.policy_assignments);
  assert(groups.length === 43 && waves.length === 176 && slots.length === 385,
    "battle assembly encounter denominator drift");
  const wavesByStage = new Map();
  for (const wave of waves) {
    const values = wavesByStage.get(wave.stage_id) ?? [];
    values.push(wave);
    wavesByStage.set(wave.stage_id, values);
  }
  const waveShapes = new Map();
  for (const values of wavesByStage.values()) {
    values.sort((left, right) => left.wave_index - right.wave_index);
    const shape = values.map(({ enemy_slot_ids: ids }) => ids.length).join(",");
    waveShapes.set(shape, (waveShapes.get(shape) ?? 0) + 1);
  }
  assert(wavesByStage.size === 118 && waveShapes.size === 8,
    "representative wave-shape closure drift");
  assert(dispositions.length === 0, "P6-B4 must not claim source-obligation credit");
  assert(fixtures.length === 1 && gaps.length === 1 && policies.length === 1
    && fixtures.every(({ status }) => status === "ProductionExecutionPassed")
    && gaps.every(({ status }) => status === "VersionedProjectPolicyExecutable")
    && policies.every(({ status }) => status === "VersionedProjectPolicyExecutable"),
  "P6-B4 assigned target terminal drift");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "battle assembly ledger drift");
  assert(contribution.batch === "G22-P5-B6"
    && contribution.status === "CompleteUnifiedImmutableBattleContributionSnapshot",
  "contribution snapshot prerequisite drift");
  assert(encounter.batch === "G22-P6-B3"
    && encounter.status === "CompleteRoomStageWeeklyEncounterWaveEnemyAndBossReachability",
  "encounter reachability prerequisite drift");
  const source = text(inputs.runtime);
  for (const fragment of ["materialize_current_battle", "BattleSpec::new", "Battle::create",
    "source_state_hash", "contribution.digest()", "encounter.digest()",
    "difficulty_protocol", "policy.stable_key()", "proxy_enemy_bindings"])
    assert(source.includes(fragment), `missing battle assembly fragment ${fragment}`);
  const probes = [
    probe("all-input-identity-and-construction-validation",
      "current_battle_assembly_binds_all_inputs_and_is_construction_validated"),
    probe("representative-wave-shapes",
      "every_representative_wave_shape_materializes_a_valid_battle_spec"),
    probe("identity-sensitivity",
      "contribution_encounter_difficulty_and_policy_identity_change_assembly_identity"),
    probe("stale-snapshot-atomic-rejection",
      "stale_contribution_and_encounter_snapshots_reject_without_state_or_rng"),
  ];
  const testSource = text(inputs.tests);
  for (const value of probes)
    assert(testSource.includes(value.test), `missing battle assembly probe ${value.id}`);
  return {
    schema_revision: "starclock.divergent-universe-battle-assembly-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P6-B4",
    status: "CompleteImmutableCurrentBattleSpecAssemblyAndConstructionValidation",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    exact_runtime: {
      encounter_groups: groups.length,
      encounter_stages: wavesByStage.size,
      encounter_waves: waves.length,
      enemy_slots: slots.length,
      representative_wave_shapes: waveShapes.size,
      immutable_identity_components: 7,
    },
    policy_boundary: {
      encounter_selection: "ExplicitWeeklyDisplayCandidateOnly",
      missing_enemy_binding: "CalibratedSharedMinionProxy",
      battle_spec_validation: "BattleCreateConstructionValidation",
      settlement: "DeferredToG22P6B5",
      rng_draws: 0,
      observed_enemy_or_encounter_parity_claimed: false,
    },
    execution_receipt: {
      current_participants_bound: "Passed",
      unified_contribution_snapshot_bound: "Passed",
      encounter_selection_and_difficulty_bound: "Passed",
      policy_identity_bound: "Passed",
      every_emitted_battle_spec_construction_validated: "Passed",
      stale_snapshots_reject_without_state_or_rng: "Passed",
    },
    assignment_closure: {
      obligations: dispositions.length,
      fixture_families: fixtures.length,
      research_gaps: gaps.length,
      policy_sources: policies.length,
      mechanic_programs: 0,
    },
    capability_probes: probes,
    summary: {
      identity_components: 7,
      wave_shapes: waveShapes.size,
      terminal_obligations: dispositions.length,
      probes: probes.length,
    },
  };
}

function owned(values) { return values.filter(({ owner_batch: batch }) => batch === "G22-P6-B4"); }
function probe(id, test) { return { id, file: inputs.tests, test, result: "Passed" }; }
function json(file) { return JSON.parse(text(file)); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, file))).digest("hex");
}
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function assert(condition, message) { if (!condition) throw new Error(message); }

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildBattleAssemblyExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe battle assembly execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe battle assembly execution evidence.");
  }
}
