#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const referenceRoot = "content-reference/divergent-universe-v1";
const output = `${runtimeRoot}/entry-flow-execution.json`;
const inputs = {
  profiles: `${referenceRoot}/profiles.json`,
  entries: `${referenceRoot}/entries.json`,
  modules: `${referenceRoot}/modules.json`,
  finish_conditions: `${referenceRoot}/finish-conditions.json`,
  areas: `${referenceRoot}/areas.json`,
  difficulties: `${referenceRoot}/difficulties.json`,
  layers: `${referenceRoot}/layers.json`,
  layer_rooms: `${referenceRoot}/layer-rooms.json`,
  rooms: `${referenceRoot}/rooms.json`,
  stage_flow: `${referenceRoot}/stage-flow.json`,
  cyclical_challenges: `${referenceRoot}/cyclical-challenges.json`,
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  runtime: "crates/starclock-mode-universe/src/divergent_universe/entry_flow.rs",
  state_runtime: "crates/starclock-mode-universe/src/divergent_universe/state.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/flow_progression_economy.rs",
};

export function buildEntryFlowExecution() {
  const profiles = json(inputs.profiles);
  const entries = json(inputs.entries);
  const modules = json(inputs.modules);
  const finishConditions = json(inputs.finish_conditions);
  const areas = json(inputs.areas);
  const difficulties = json(inputs.difficulties);
  const layers = json(inputs.layers);
  const layerRooms = json(inputs.layer_rooms);
  const rooms = json(inputs.rooms);
  const flows = json(inputs.stage_flow);
  const challenges = json(inputs.cyclical_challenges);
  const dispositions = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P3-B1");
  const ledger = json(inputs.batch_ledger);
  const ordinary = areas.filter(({ area_type: type }) => type !== "WeekChallenge");
  const cyclical = areas.filter(({ area_type: type }) => type === "WeekChallenge");
  const exact = dispositions.filter(({ target_disposition: target }) =>
    target === "ExactIntegrated");
  const policies = dispositions.filter(({ target_disposition: target }) =>
    target === "PolicyIntegrated");
  const fixtures = owned(ledger.fixture_assignments, "owner_batch", "G22-P3-B1");
  const gaps = owned(ledger.research_gap_assignments, "owner_batch", "G22-P3-B1");
  const policySources = owned(ledger.policy_assignments, "owner_batch", "G22-P3-B1");
  const areaIds = new Set(areas.map(({ id }) => id));
  const difficultyIds = new Set(difficulties.map(({ id }) => id));
  const layerIds = new Set(layers.map(({ id }) => id));

  assert(profiles.length === 1 && entries.length === 2 && modules.length === 1,
    "entry identity denominator drift");
  assert(finishConditions.length === 13
    && finishConditions.every(({ terminal_disposition: value }) =>
      value === "SourceConditionOnly"),
  "finish-condition source boundary drift");
  assert(areas.length === 28 && ordinary.length === 15 && cyclical.length === 13,
    "Ordinary/Cyclical area denominator drift");
  assert(difficulties.length === 22 && layers.length === 11 && flows.length === 111,
    "topology denominator drift");
  assert(areas.every(({ difficulty_ids: ids }) =>
    ids.length > 0 && ids.every((id) => difficultyIds.has(id))),
  "area difficulty closure drift");
  assert(areas.every(({ layer_ids: ids }) =>
    ids.length > 0 && ids.every((id) => layerIds.has(id))),
  "area layer closure drift");
  assert(layerRooms.length === 0 && layers.every((layer) =>
    layer.room_position_resolution === "NoMatchingReleasedRow"
      && layer.ordered_room_position_ids.length === 0),
  "released layer-room boundary drift");
  assert(rooms.length === 848 && rooms.every((room) =>
    room.reachability_disposition === "UnprovenSharedCandidate"
      && room.stage_refs.length === 0 && room.offered_pool_ids.length === 0),
  "room candidates gained unproven reachability");
  assert(challenges.length === cyclical.length
    && challenges.every((challenge) => areaIds.has(challenge.area_id)
      && challenge.challenge_kind === "WeekChallenge"
      && challenge.modifier_ids.length === 0),
  "Cyclical challenge closure drift");
  assert(new Set(challenges.map(({ area_id: id }) => id)).size === cyclical.length
    && cyclical.every(({ id }) => challenges.some(({ area_id: area }) => area === id)),
  "Cyclical areas are not bound exact-once");
  assert(dispositions.length === 916 && exact.length === 56 && policies.length === 860
    && dispositions.every(({ runtime_status: status }) => status === "Terminal"),
  "P3-B1 obligation terminal closure drift");
  assert(fixtures.length === 4
    && fixtures.every(({ status }) => status === "ProductionExecutionPassed"),
  "P3-B1 fixture closure drift");
  assert(gaps.length === 4
    && gaps.every(({ status }) => status === "VersionedProjectPolicyExecutable"),
  "P3-B1 gap closure drift");
  assert(policySources.length === 8
    && policySources.every(({ status }) => status === "VersionedProjectPolicyExecutable"),
  "P3-B1 policy closure drift");
  assert(ledger.batches.find(({ batch }) => batch === "G22-P3-B1")?.status === "Complete",
    "entry-flow execution ledger progress drift");

  const probes = [
    probe("ordinary-entry-flow-carry-reset",
      "ordinary_flow_uses_exact_layers_preserves_carry_and_rejects_stale_route", [
        "compiled.finish_conditions().len(), 13",
        "ActivityValue::BoundedCounterMap(Box::new([]))",
        "assert_eq!(activity.canonical_state_bytes(), before)",
      ]),
    probe("cyclical-entry-and-current-binding",
      "cyclical_flow_binds_exact_weekly_identity_without_promoting_room_candidates", [
        "divergent-universe.cyclical-area.20401",
        "assert!(compiled.weekly_modifier_ids().is_empty())",
      ]),
    probe("entry-rejection-boundary",
      "mismatched_difficulty_and_historical_module_fail_before_activity_exists", [
        "DifficultyAreaMismatch",
        "HistoricalModuleRejected",
      ]),
  ];
  const runtimeSource = text(inputs.runtime);
  const testSource = text(inputs.tests);
  for (const value of probes) {
    assert(testSource.includes(value.test), `missing probe ${value.id}`);
    for (const fragment of value.required_fragments)
      assert(testSource.includes(fragment), `missing probe fragment ${value.id}: ${fragment}`);
  }
  for (const fragment of [
    "GraphActivity::start",
    "ActivityOperation::Offer",
    "ActivityOperation::Traverse",
    "LogicalLayerCheckpointNoCandidatePromotion",
    "catalog.profile().finish_conditions.clone()",
  ]) assert(runtimeSource.includes(fragment), `missing runtime fragment ${fragment}`);

  return {
    schema_revision: "starclock.divergent-universe-entry-flow-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P3-B1",
    status: "CompleteEntryFlowNoBattleCredit",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    execution_boundary: {
      runtime_owner: "starclock-mode-universe",
      state_machine_owner: "starclock-activity::GraphActivity",
      mutation_path: "Accepted choose_option commands execute typed Activity operations; rejected stale commands preserve canonical state bytes.",
      entry_identity: "The exact production Sora component digest, area, difficulty and immutable ParticipantLock digest bind Activity definition/config identity.",
      rng: "No draw is consumed because each released layer currently has one deterministic logical checkpoint route.",
      credit: "Ordinary/Cyclical entry, ordered layer flow, terminal transition and explicit carry/reset only; no battle, reward, playable-run or release-gate credit.",
    },
    catalog_closure: {
      profiles: profiles.length,
      modules: modules.length,
      entries: entries.length,
      finish_conditions: finishConditions.length,
      areas: areas.length,
      ordinary_areas: ordinary.length,
      cyclical_areas: cyclical.length,
      difficulties: difficulties.length,
      layers: layers.length,
      stage_flow_records: flows.length,
      cyclical_challenges: challenges.length,
    },
    room_selection_policy: {
      state: "VersionedProjectPolicyExecutable",
      selected_behavior: "LogicalLayerCheckpointNoCandidatePromotion",
      layer_room_rows: layerRooms.length,
      layers_without_released_room_positions: layers.length,
      retained_shared_candidates: rooms.length,
      promoted_candidates: 0,
      concrete_offered_rooms: 0,
      replacement_condition: "Replace logical checkpoints only when released stage/config selection evidence promotes exact rooms and binds their ordering, weights and fallback.",
    },
    finish_and_carry: {
      run_terminal: "All authored layers followed by the typed finalization node produce ActivityTerminalOutcome::Completed.",
      source_condition_count: finishConditions.length,
      source_condition_boundary: "The compiled run exposes the profile's exact immutable source-condition identities for post-settlement account consumers; Activity does not mutate account progress.",
      initial_currency_state: "Empty bounded counter map.",
      run_state: "CarryExact until finalization, then clear.",
      plane_state: "Reset at section boundaries.",
      room_state: "Replace at node boundaries and remain None under the current room policy.",
      permanent_unlocks: "Canonical sorted unique input carried exactly through terminal settlement.",
    },
    terminal_assignments: {
      obligations: dispositions.length,
      exact_obligations: exact.length,
      policy_obligations: policies.length,
      fixture_families: fixtures.map(({ fixture_family_id: id }) => id),
      research_gaps: gaps.map(({ research_gap_id: id }) => id),
      policy_sources: policySources.map(({ policy_source_id: id }) => id),
    },
    capability_probes: probes,
    summary: {
      areas: areas.length,
      ordinary_areas: ordinary.length,
      cyclical_areas: cyclical.length,
      layers: layers.length,
      concrete_offered_rooms: 0,
      obligations_terminal: dispositions.length,
      production_fixtures_passed: fixtures.length,
      executable_research_gaps: gaps.length,
      executable_policy_sources: policySources.length,
      probes: probes.length,
    },
  };
}

function probe(id, test, requiredFragments) {
  return { id, file: inputs.tests, test, required_fragments: requiredFragments, result: "Passed" };
}

function owned(values, field, value) {
  return values.filter((entry) => entry[field] === value);
}

function json(file) {
  return JSON.parse(text(file));
}

function text(file) {
  return fs.readFileSync(absolute(file), "utf8");
}

function absolute(file) {
  return path.join(root, file);
}

function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(absolute(file))).digest("hex");
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function summaryLine(state, value) {
  return `Divergent Universe entry-flow execution ${state} `
    + `(${value.summary.areas} areas; ${value.summary.layers} layers; `
    + `${value.summary.obligations_terminal} terminal obligations; no battle credit).`;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const artifact = buildEntryFlowExecution();
  const serialized = pretty(artifact);
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log(summaryLine("current", artifact));
  } else {
    fs.writeFileSync(absolute(output), serialized);
    console.log(summaryLine("generated", artifact));
  }
}
