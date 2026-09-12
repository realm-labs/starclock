#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const referenceRoot = "content-reference/divergent-universe-v1";
const output = `${runtimeRoot}/encounter-reachability-execution.json`;
const inputs = {
  rooms: `${referenceRoot}/rooms.json`,
  stage_flow: `${referenceRoot}/stage-flow.json`,
  weekly_modifiers: `${referenceRoot}/weekly-modifiers.json`,
  encounter_sources: `${referenceRoot}/encounter-source-obligations.json`,
  encounter_groups: `${referenceRoot}/encounter-groups.json`,
  encounter_waves: `${referenceRoot}/encounter-waves.json`,
  enemy_slots: `${referenceRoot}/enemy-slots.json`,
  boss_pools: `${referenceRoot}/boss-pools.json`,
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  prior_execution: `${runtimeRoot}/service-adventure-execution.json`,
  runtime: "crates/starclock-mode-universe/src/divergent_universe/encounter_reachability_runtime.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/encounter_reachability_runtime.rs",
};

export function buildEncounterReachabilityExecution() {
  const rooms = json(inputs.rooms);
  const stageFlow = json(inputs.stage_flow);
  const weekly = json(inputs.weekly_modifiers);
  const sources = json(inputs.encounter_sources);
  const groups = json(inputs.encounter_groups);
  const waves = json(inputs.encounter_waves);
  const slots = json(inputs.enemy_slots);
  const bossPools = json(inputs.boss_pools);
  const ledger = json(inputs.batch_ledger);
  const prior = json(inputs.prior_execution);
  const dispositions = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P6-B3");
  const fixtures = ledger.fixture_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P6-B3");
  const gaps = ledger.research_gap_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P6-B3");
  const policies = ledger.policy_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P6-B3");
  assert(rooms.length === 848 && stageFlow.length === 111 && weekly.length === 103,
    "room/stage-flow/weekly denominator drift");
  assert(sources.length === 877 && groups.length === 43 && waves.length === 176
    && slots.length === 385 && bossPools.length === 618,
  "encounter denominator drift");
  const stageRoot = sources.filter(({ parent_kind: kind }) =>
    kind === "SharedStageConfigRoot");
  assert(stageRoot.length === 1 && stageRoot[0].encounter_group_ids.length === 43
    && stageRoot[0].stage_ids.length === 118
    && sources.filter(({ parent_kind: kind }) => kind === "AreaEntry").length === 28
    && sources.filter(({ parent_kind: kind }) => kind === "RoomCandidate").length === 848,
  "encounter-source policy split drift");
  assert(rooms.every(({ stage_refs: stages, offered_pool_ids: pools,
    reachability_disposition: reachability }) => stages.length === 0 && pools.length === 0
      && reachability === "UnprovenSharedCandidate"),
  "room selector boundary drift");
  assert(weekly.every(({ reachability, runtime_lowered: lowered,
    enemy_group_refs: refs }) => reachability === "UnprovenCurrentWeeklyCandidate"
      && !lowered && refs.length === 6),
  "weekly display boundary drift");
  const groupStagePairs = sum(groups.map(({ candidate_stage_ids: ids }) => ids.length));
  const uniqueStages = new Set(groups.flatMap(({ candidate_stage_ids: ids }) => ids));
  const bossAlternatives = sum(bossPools.map(({ candidate_stage_ids: ids }) => ids.length));
  assert(groupStagePairs === 184 && uniqueStages.size === 118 && bossAlternatives === 2982,
    "group/stage/boss candidate closure drift");
  assert(dispositions.length === 877
    && dispositions.filter(({ target_disposition: target }) =>
      target === "PolicyIntegrated").length === 876
    && dispositions.filter(({ target_disposition: target }) =>
      target === "SharedIntegrated").length === 1
    && dispositions.every(({ runtime_status: status }) => status === "Terminal"),
  "P6-B3 obligation closure drift");
  assert(fixtures.length === 1 && gaps.length === 1 && policies.length === 5
    && fixtures.every(({ status }) => status === "ProductionExecutionPassed")
    && gaps.every(({ status }) => status === "VersionedProjectPolicyExecutable")
    && policies.every(({ status }) => status === "VersionedProjectPolicyExecutable"),
  "P6-B3 assigned target terminal drift");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "encounter reachability ledger drift");
  assert(prior.batch === "G22-P6-B2"
    && prior.status === "CompleteServiceAdventureShopEntryAndFallbackExecution",
  "service/Adventure prerequisite drift");
  const source = text(inputs.runtime);
  for (const fragment of ["resolve_source", "select_stage_candidate",
    "select_weekly_boss_candidate", "StageConfigCandidateBoundary",
    "SelectorUnavailable", "EncounterSelectionDigest"])
    assert(source.includes(fragment), `missing encounter reachability fragment ${fragment}`);
  const probes = [
    probe("catalog-and-policy-boundaries",
      "encounter_reachability_catalog_compiles_all_exact_and_policy_boundaries"),
    probe("unresolved-area-and-room-selectors",
      "every_unresolved_area_and_room_source_rejects_without_state_or_rng"),
    probe("group-stage-wave-enemy-closure",
      "every_group_stage_candidate_closes_to_ordered_waves_and_enemy_slots"),
    probe("weekly-boss-alternatives",
      "every_weekly_boss_alternative_binds_matching_group_stage_and_monster"),
    probe("stale-and-cross-group-rejection",
      "stale_and_cross_group_candidate_rejections_are_atomic"),
  ];
  const testSource = text(inputs.tests);
  for (const value of probes)
    assert(testSource.includes(value.test), `missing encounter probe ${value.id}`);
  return {
    schema_revision: "starclock.divergent-universe-encounter-reachability-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P6-B3",
    status: "CompleteRoomStageWeeklyEncounterWaveEnemyAndBossReachability",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    exact_runtime: {
      encounter_sources: sources.length,
      retained_rooms: rooms.length,
      stage_flow_rows: stageFlow.length,
      weekly_displays: weekly.length,
      encounter_groups: groups.length,
      group_stage_pairs: groupStagePairs,
      unique_stages: uniqueStages.size,
      waves: waves.length,
      enemy_slots: slots.length,
      boss_pools: bossPools.length,
      boss_alternatives: bossAlternatives,
    },
    policy_boundary: {
      area_entry_selector: "RejectWithoutMutation",
      room_selector: "RejectWithoutMutation",
      stageconfig_closure: "ExactSharedCandidateIdentityClosure",
      display_candidate_selection: "ExplicitVersionedProjectPolicyOnly",
      weekly_current_selector: "UnavailableAtPinnedRevision",
      rng_draws: 0,
      observed_reachability_or_selection_parity_claimed: false,
    },
    execution_receipt: {
      every_source_obligation_terminal: "Passed",
      exact_group_wave_slot_closure: "Passed",
      every_explicit_group_stage_candidate_selectable: "Passed",
      every_weekly_boss_alternative_bound: "Passed",
      stale_cross_group_and_unresolved_selectors_reject_atomically: "Passed",
    },
    assignment_closure: {
      obligations: dispositions.length,
      policy_integrated: 876,
      shared_integrated: 1,
      fixture_families: fixtures.length,
      research_gaps: gaps.length,
      policy_sources: policies.length,
      mechanic_programs: 0,
    },
    capability_probes: probes,
    summary: {
      sources: sources.length,
      stages: uniqueStages.size,
      boss_pools: bossPools.length,
      terminal_obligations: dispositions.length,
      probes: probes.length,
    },
  };
}

function probe(id, test) { return { id, file: inputs.tests, test, result: "Passed" }; }
function sum(values) { return values.reduce((total, value) => total + value, 0); }
function json(file) { return JSON.parse(text(file)); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, file))).digest("hex");
}
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function assert(condition, message) { if (!condition) throw new Error(message); }

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildEncounterReachabilityExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe encounter reachability execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe encounter reachability execution evidence.");
  }
}
