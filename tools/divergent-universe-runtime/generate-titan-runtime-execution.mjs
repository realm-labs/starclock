#!/usr/bin/env node

// Current selection/descriptor inventory, not a runtime execution receipt.
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const referenceRoot = "content-reference/divergent-universe-v1";
const output = `${runtimeRoot}/titan-runtime-execution.json`;
const inputs = {
  titan_types: `${referenceRoot}/titan-types.json`,
  titan_boons: `${referenceRoot}/titan-boons.json`,
  titan_talents: `${referenceRoot}/titan-talents.json`,
  titan_choices: `${referenceRoot}/titan-choices.json`,
  titan_contributions: `${referenceRoot}/titan-contributions.json`,
  review_fixtures: `${referenceRoot}/review-fixtures.json`,
  research_gaps: `${referenceRoot}/research-gaps.json`,
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  data_catalog: "crates/starclock-data/src/divergent_universe_titan_catalog.rs",
  runtime: "crates/starclock-mode-universe/src/divergent_universe/titan_runtime.rs",
  state: "crates/starclock-mode-universe/src/divergent_universe/state.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/titan_runtime.rs",
  battle_assembly: "crates/starclock-mode-universe/src/divergent_universe/battle_assembly_runtime.rs",
  battle_assembly_tests: "crates/starclock-mode-universe/src/divergent_universe/tests/battle_assembly_runtime.rs",
};

export function buildTitanRuntimeExecution({ read = text } = {}) {
  const json = (file) => JSON.parse(read(file));
  const types = json(inputs.titan_types);
  const boons = json(inputs.titan_boons);
  const talents = json(inputs.titan_talents);
  const choices = json(inputs.titan_choices);
  const contributions = json(inputs.titan_contributions);
  const dispositions = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P5-B3");
  const ledger = json(inputs.batch_ledger);
  const fixtures = json(inputs.review_fixtures).filter(
    ({ source_id: id }) => id === "golden-blood-titan-choice-and-level");
  const gaps = json(inputs.research_gaps).filter(
    ({ source_id: id }) => id === "golden-blood-titan-choice-and-level");
  const fixtureAssignments = ledger.fixture_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P5-B3");
  const gapAssignments = ledger.research_gap_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P5-B3");
  const policies = ledger.policy_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P5-B3");
  const boonContributions = contributions.filter(
    ({ activation }) => activation === "AcceptedGoldenBloodBoon");
  const talentContributions = contributions.filter(
    ({ activation }) => activation === "TalentUnlocked");

  assert(types.length === 12 && boons.length === 84 && talents.length === 36
    && choices.length === 36 && contributions.length === 120,
  "Titan denominator drift");
  assert(types.every(({ boon_ids: boonIds, talent_ids: talentIds, runtime_lowered: lowered }) =>
    boonIds.length === 7 && talentIds.length === 3 && !lowered),
  "Titan type child closure drift");
  assert(equal(countBy(boons, ({ level }) => String(level)), { 1: 12, 2: 36, 3: 36 })
    && boons.every(({ binding_type: binding, maze_buff_level: level,
      runtime_lowered: lowered }) => binding === "StageAbilityBeforeCharacterBorn"
      && level === 1 && !lowered),
  "Golden Blood Boon level or binding drift");
  assert(equal(countBy(choices,
    ({ level, candidate_ids: candidates }) => `${level}:${candidates.length}`),
  { "1:1": 12, "2:3": 12, "3:3": 12 })
    && choices.every(({ level, eligibility, ordering, selection_count: count,
      reroll, fallback, runtime_lowered: lowered }) => eligibility === ({
      1: "TitanTypeActivated",
      2: "PriorBoonLevel1Accepted",
      3: "PriorBoonLevel2Accepted",
    })[level] && ordering === "StableCandidateId" && count === 1
      && reroll === "Unspecified" && fallback === "RejectWithoutMutation" && !lowered),
  "Golden Blood offer policy drift");
  assert(equal(countBy(talents, ({ cost }) => cost[0].amount),
    { 50: 12, 75: 12, 100: 12 })
    && talents.filter(({ predecessor_id: predecessor }) => predecessor === "").length === 1
    && talents.every(({ cost, presentation_graph_excluded: excluded,
      runtime_lowered: lowered }) => cost.length === 1 && cost[0].item_id === "281020"
      && excluded && !lowered),
  "Titan permanent talent cost or prerequisite drift");
  const totalTalentCost = talents.reduce((sum, { cost }) => sum + BigInt(cost[0].amount), 0n).toString();
  assert(totalTalentCost === "2700", "Titan total talent cost drift");
  assert(acyclicTalentGraph(talents), "Titan talent prerequisite graph drift");
  assert(boonContributions.length === 84 && talentContributions.length === 36
    && boonContributions.every(({ scope, teardown, ordered_effects: effects,
      runtime_lowered: lowered }) => scope === "Battle" && teardown === "BattleEnd"
      && effects.length === 1 && !lowered)
    && talentContributions.every(({ teardown, ordered_effects: effects,
      runtime_lowered: lowered }) => teardown === "ProfileResetOnly"
      && effects.length === 1 && !lowered),
  "Titan contribution lifecycle drift");
  assert(equal(countBy(talentContributions, ({ scope }) => scope),
    { Activity: 10, Battle: 26 }), "Titan talent contribution scope drift");
  assert(dispositions.length === 132
    && dispositions.every(({ target_disposition: target, runtime_status: status }) =>
      target === "ExactIntegrated" && status === "Pending"),
  "Titan descriptors cannot receive terminal runtime coverage");
  const currentIds = [...types, ...boons, ...talents].map(({ id }) => id).sort();
  const assignedIds = dispositions.flatMap(({ normalized_record_ids: ids }) => ids).sort();
  assert(equal(currentIds, assignedIds), "Titan exact-once source/definition closure drift");
  assert(fixtures.length === 1 && gaps.length === 1
    && fixtureAssignments.length === 1 && gapAssignments.length === 1
    && policies.length === 2, "P5-B3 assigned target denominator drift");
  assert(fixtureAssignments.every(({ status }) => status === "PendingProductionExecution")
    && gapAssignments.every(({ status }) => status === "PendingExecutableDisposition")
    && policies.every(({ status }) => status === "PendingExecutableDisposition"),
  "Titan descriptor fixtures/policies must remain pending execution");

  const sourceChecks = {
    runtime: ["activate_type_accepted", "next_offer", "accept_boon_accepted",
      "unlock_talent_accepted", "MissingTalentPrerequisite", "snapshot_digest"],
    state: ["TITAN_TYPE_SLOT", "TITAN_BOONS_SLOT", "TITAN_TALENTS_SLOT",
      "TITAN_TALENT_CURRENCY_SLOT"],
  };
  for (const [name, fragments] of Object.entries(sourceChecks)) {
    const source = read(inputs[name]);
    for (const fragment of fragments)
      assert(source.includes(fragment), `missing ${name} fragment ${fragment}`);
  }
  const targets = [
    target("catalog-offer-and-descriptor-closure",
      "titan_catalog_compiles_all_exact_rows_and_policy_offer_shapes"),
    target("all-golden-blood-selection-records",
      "every_golden_blood_boon_executes_through_its_exact_level_offer"),
    target("all-talent-cost-prerequisite-and-descriptor-order",
      "all_permanent_titan_talents_enforce_costs_prerequisites_and_stack_contributions"),
    target("selection-rejection-rng-and-reconstruction",
      "titan_offer_rejections_and_replay_are_state_hash_and_rng_inert"),
    target("all-boon-descriptors-rejected-at-assembly",
      "titan_boon_descriptors_cannot_silently_enter_current_battles", inputs.battle_assembly_tests),
    target("all-talent-descriptors-rejected-at-assembly",
      "titan_talent_descriptors_cannot_silently_enter_current_battles", inputs.battle_assembly_tests),
  ];
  for (const value of targets)
    assert(read(value.file).includes(`fn ${value.test}(`), `missing Titan test target ${value.id}`);
  const assembly = read(inputs.battle_assembly);
  assert(assembly.includes("if !contribution.titan().contributions().is_empty()")
    && assembly.includes("return Err(DivergentUniverseBattleAssemblyError::UnimplementedTitanEffects)"),
  "current assembly must reject unsupported Titan effect descriptors");

  return {
    status: "TitanSelectionDescriptorsEffectsPending",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: crypto.createHash("sha256").update(read(file)).digest("hex") },
    ])),
    reference_shape: {
      titan_types: types.length,
      golden_blood_boons: boons.length,
      boon_choices: choices.length,
      permanent_talent_levels: talents.length,
      contributions: contributions.length,
      talent_currency_item_id: "281020",
      total_talent_cost: totalTalentCost,
      activity_contributions: talentContributions.filter(
        ({ scope }) => scope === "Activity").length,
      battle_contributions: boonContributions.length + talentContributions.filter(
        ({ scope }) => scope === "Battle").length,
    },
    policy_boundary: {
      offer_accuracy:
        "VersionedProjectPolicyAcceptedOfferTimingAndExplicitStableIdSelectionNotObservedParity",
      exact_candidate_grouping: true,
      no_legal_candidate_fallback: "RejectWithoutMutation",
      offer_rng_draws: 0,
      exact_offer_timing_parity_claimed: false,
    },
    current_boundary: {
      selection_and_unlock_state_commands: "PresentRequiresNativeVerification",
      contribution_representation: "SourceDescriptorsOnly",
      activity_effect_consumers_implemented: false,
      battle_effect_consumers_implemented: false,
      public_offer_admission_implemented: false,
      unsupported_current_battle_assembly: "RejectUnimplementedTitanEffectsBeforeCacheLookup",
      terminal_coverage_credit: 0,
      required_next_work: "Lower and execute the 10 Activity and 110 battle contribution effects; bind public offers and independent production/replay fixtures before terminal coverage.",
    },
    pending_assignments: {
      obligations: dispositions.length,
      obligation_ids: dispositions.map(({ obligation_id: id }) => id).sort(),
      fixture_families: fixtureAssignments.length,
      research_gaps: gapAssignments.length,
      policy_sources: policies.length,
      mechanic_programs: 0,
    },
    native_test_targets: targets,
    summary: {
      titan_types: types.length,
      boons: boons.length,
      talents: talents.length,
      contributions: contributions.length,
      pending_obligations: dispositions.length,
      terminal_coverage_credit: 0,
      native_test_targets: targets.length,
    },
  };
}

function acyclicTalentGraph(talents) {
  const remaining = new Map(talents.map((value) => [value.id, value.predecessor_id]));
  const complete = new Set();
  while (remaining.size > 0) {
    const ready = [...remaining].filter(([, predecessor]) =>
      predecessor === "" || complete.has(predecessor));
    if (ready.length === 0) return false;
    for (const [id] of ready) {
      remaining.delete(id);
      complete.add(id);
    }
  }
  return true;
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
function target(id, test, file = inputs.tests) { return { id, file, test }; }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildTitanRuntimeExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe Titan descriptor inventory is current; no execution credit.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated current Titan descriptor inventory; 132 obligations remain pending.");
  }
}
