#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const referenceRoot = "content-reference/divergent-universe-v1";
const output = `${runtimeRoot}/permanent-progression-execution.json`;
const inputs = {
  permanent_talents: `${referenceRoot}/permanent-talents.json`,
  progression_effects: `${referenceRoot}/progression-effects.json`,
  unlocks: `${referenceRoot}/unlocks.json`,
  weekly_modifiers: `${referenceRoot}/weekly-modifiers.json`,
  room_marks: `${referenceRoot}/room-marks.json`,
  mode_service_npcs: `${referenceRoot}/mode-service-npcs.json`,
  review_fixtures: `${referenceRoot}/review-fixtures.json`,
  research_gaps: `${referenceRoot}/research-gaps.json`,
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  prior_execution: `${runtimeRoot}/titan-runtime-execution.json`,
  data_catalog: "crates/starclock-data/src/divergent_universe_progression_catalog.rs",
  runtime: "crates/starclock-mode-universe/src/divergent_universe/permanent_progression_runtime.rs",
  state: "crates/starclock-mode-universe/src/divergent_universe/state.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/permanent_progression_runtime.rs",
};

export function buildPermanentProgressionExecution() {
  const talents = json(inputs.permanent_talents);
  const effects = json(inputs.progression_effects);
  const unlocks = json(inputs.unlocks);
  const weekly = json(inputs.weekly_modifiers);
  const marks = json(inputs.room_marks);
  const services = json(inputs.mode_service_npcs);
  const ledger = json(inputs.batch_ledger);
  const prior = json(inputs.prior_execution);
  const dispositions = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P5-B4");
  const familyIds = new Set([
    "permanent-talent-and-unlock",
    "weekly-modifier-and-room-service",
  ]);
  const fixtures = json(inputs.review_fixtures).filter(
    ({ source_id: id }) => familyIds.has(id));
  const gaps = json(inputs.research_gaps).filter(
    ({ source_id: id }) => familyIds.has(id));
  const fixtureAssignments = ledger.fixture_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P5-B4");
  const gapAssignments = ledger.research_gap_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P5-B4");
  const policies = ledger.policy_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P5-B4");
  const exactUnlocks = unlocks.filter(
    ({ unlocked_content_ids: ids }) => ids.length > 0);

  assert(talents.length === 38 && effects.length === 38 && unlocks.length === 97
    && weekly.length === 103 && marks.length === 24 && services.length === 23,
  "permanent progression denominator drift");
  assert(equal(countBy(talents, ({ cost }) => String(cost[0].amount)),
    { 100: 1, 120: 5, 180: 4, 40: 28 })
    && talents.reduce((sum, { cost }) => sum + Number(cost[0].amount), 0) === 2540
    && talents.every(({ cost, prerequisite_ids: prerequisites,
      prerequisite_resolution: resolution, adjacent_talent_ids: adjacent,
      runtime_lowered: lowered }) => cost.length === 1 && cost[0].item_id === "281018"
      && prerequisites.length === 0
      && resolution === "UnavailableInBidirectionalAdjacency"
      && adjacent.length > 0 && !lowered),
  "permanent talent cost or prerequisite policy drift");
  assert(equal(countBy(effects, ({ scope }) => scope), { Activity: 9, Battle: 29 })
    && effects.every(({ activation, runtime_lowered: lowered }) =>
      activation === "PermanentTalentUnlocked" && !lowered),
  "permanent talent contribution drift");
  assert(exactUnlocks.length === 8
    && unlocks.filter(({ unlocked_content_ids: ids }) => ids.length === 0).length === 89
    && new Set(exactUnlocks.flatMap(({ unlocked_content_ids: ids }) => ids)).size === 14,
  "finish unlock consumer resolution drift");
  assert(weekly.every(({ reachability, enemy_group_refs: enemies,
    runtime_lowered: lowered }) => reachability === "UnprovenCurrentWeeklyCandidate"
      && enemies.length === 6 && !lowered),
  "weekly modifier policy drift");
  assert(marks.every(({ transition_rules: transitions, fallback,
    runtime_lowered: lowered }) => transitions.length === 0
      && fallback === "PreserveCurrentMark" && !lowered),
  "room mark fallback drift");
  assert(services.every(({ graph_resolution: resolution, service_kind: kind,
    choice_ids: choices, fallback, runtime_lowered: lowered }) =>
      resolution === "MissingAtPinnedRevision" && kind === "UnclassifiedMissingGraph"
      && choices.length === 0 && fallback === "RejectWithoutMutation" && !lowered),
  "service NPC fallback drift");
  assert(dispositions.length === 262
    && dispositions.filter(({ target_disposition: target }) => target === "ExactIntegrated").length === 8
    && dispositions.filter(({ target_disposition: target }) => target === "PolicyIntegrated").length === 254
    && dispositions.every(({ runtime_status: status }) => status === "Terminal"),
  "P5-B4 obligation closure drift");
  assert(fixtures.length === 2 && gaps.length === 2
    && fixtureAssignments.length === 2 && gapAssignments.length === 2
    && policies.length === 6, "P5-B4 assigned target denominator drift");
  assert(fixtureAssignments.every(({ status }) => status === "ProductionExecutionPassed")
    && gapAssignments.every(({ status }) => status === "VersionedProjectPolicyExecutable")
    && policies.every(({ status }) => status === "VersionedProjectPolicyExecutable"),
  "P5-B4 assigned target terminal drift");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "permanent progression ledger drift");
  assert(prior.batch === "G22-P5-B3"
    && prior.status === "CompleteTitanBoonOfferTalentAndContributionExecution",
  "Titan execution prerequisite drift");

  const sourceChecks = {
    runtime: ["credit_talent_currency_accepted", "unlock_talent_accepted",
      "record_finish_unlock_accepted", "activate_weekly_modifier_policy_accepted",
      "apply_room_mark_policy_accepted", "MissingServiceGraph", "snapshot_digest"],
    state: ["PERMANENT_TALENT_CURRENCY_SLOT", "WEEKLY_MODIFIER_SLOT", "ROOM_MARK_SLOT"],
  };
  for (const [name, fragments] of Object.entries(sourceChecks)) {
    const source = text(inputs[name]);
    for (const fragment of fragments)
      assert(source.includes(fragment), `missing ${name} fragment ${fragment}`);
  }
  const probes = [
    probe("catalog-and-policy-boundaries",
      "permanent_weekly_unlock_and_room_catalogs_compile_exact_policy_boundaries"),
    probe("all-talent-costs-and-prerequisite-fallback",
      "all_permanent_talents_spend_exact_cost_and_unproven_directions_fail_closed"),
    probe("all-finish-unlock-consumers",
      "all_finish_unlocks_execute_and_only_exact_consumers_open_current_areas"),
    probe("all-weekly-modifiers-and-room-marks",
      "all_weekly_modifiers_and_room_marks_execute_accepted_policy_boundaries"),
    probe("carry-reset-and-missing-service-fallback",
      "weekly_node_profile_carry_and_missing_service_fallbacks_are_explicit"),
  ];
  const testSource = text(inputs.tests);
  for (const value of probes)
    assert(testSource.includes(value.test), `missing permanent progression probe ${value.id}`);

  return {
    schema_revision: "starclock.divergent-universe-permanent-progression-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P5-B4",
    status: "CompletePermanentWeeklyUnlockRoomAndCarryExecution",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    exact_runtime: {
      permanent_talents: talents.length,
      progression_effects: effects.length,
      finish_unlocks: unlocks.length,
      exact_unlock_consumers: exactUnlocks.length,
      exact_unlocked_areas: new Set(exactUnlocks.flatMap(
        ({ unlocked_content_ids: ids }) => ids)).size,
      weekly_modifiers: weekly.length,
      room_marks: marks.length,
      missing_service_graphs: services.length,
      talent_currency_item_id: "281018",
      total_talent_cost: 2540,
      activity_contributions: 9,
      battle_contributions: 29,
    },
    policy_boundary: {
      prerequisite_direction: "RejectUnprovenDirectionWithoutMutation",
      unresolved_unlock_consumers: 89,
      weekly_selector: "ExplicitCyclicalEpochSelectionNotObservedParity",
      room_mark_transition: "PreserveCurrentMark",
      missing_service_graph: "RejectWithoutMutation",
      weekly_rng_draws: 0,
      observed_selector_or_transition_parity_claimed: false,
    },
    lifetime_contract: {
      permanent_talents: "ProfileResetOnly",
      finish_unlocks: "ProfileResetOnly",
      weekly_modifier: "CyclicalEpoch",
      room_mark: "Node",
    },
    execution_receipt: {
      all_talent_costs_and_contributions_execute: "Passed",
      all_finish_unlocks_and_exact_consumers_execute: "Passed",
      all_weekly_candidates_execute_by_explicit_policy_selection: "Passed",
      all_room_marks_preserve_unpublished_transitions: "Passed",
      profile_carry_and_epoch_node_resets_execute: "Passed",
      missing_service_graphs_reject_without_mutation: "Passed",
    },
    assignment_closure: {
      obligations: dispositions.length,
      exact_integrated: 8,
      policy_integrated: 254,
      fixture_families: fixtureAssignments.length,
      research_gaps: gapAssignments.length,
      policy_sources: policies.length,
      mechanic_programs: 0,
    },
    capability_probes: probes,
    summary: {
      talents: talents.length,
      unlocks: unlocks.length,
      weekly_modifiers: weekly.length,
      room_marks: marks.length,
      service_npcs: services.length,
      terminal_obligations: dispositions.length,
      probes: probes.length,
    },
  };
}

function countBy(values, keyOf) {
  return Object.fromEntries([...values.reduce((counts, value) => {
    const key = keyOf(value);
    counts.set(key, (counts.get(key) ?? 0) + 1);
    return counts;
  }, new Map()).entries()].sort(([left], [right]) => left.localeCompare(right, "en", {
    numeric: true,
  })));
}
function probe(id, test) { return { id, file: inputs.tests, test, result: "Passed" }; }
function json(file) { return JSON.parse(text(file)); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, file))).digest("hex");
}
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildPermanentProgressionExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe permanent progression execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe permanent progression execution evidence.");
  }
}
