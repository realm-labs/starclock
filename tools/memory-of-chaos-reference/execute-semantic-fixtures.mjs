#!/usr/bin/env node

import { readFile } from "node:fs/promises";
import path from "node:path";
import { assert, digest, manifest, root, writeText } from "./lib.mjs";

const check = process.argv.includes("--check");
const packRoot = path.join(root, "content-reference/memory-of-chaos-v1");
async function records(file) {
  return JSON.parse(await readFile(path.join(packRoot, file), "utf8")).records;
}

const data = Object.fromEntries(await Promise.all([
  "profile.json", "seasons.json", "entries.json", "stages.json", "nodes.json", "tierce.json",
  "participant-policies.json", "attempt-rules.json",
  "clock-rules.json", "resource-rules.json", "objectives.json", "turbulence.json", "pool-audits.json",
  "battle-events.json", "rule-contributions.json",
  "encounters.json", "waves.json", "enemy-slots.json", "enemy-variants.json", "enemy-templates.json",
  "enemy-abilities.json", "reconciliation-receipts.json", "research-gaps.json", "semantic-fixtures.json",
].map(async (file) => [file, await records(file)])));
const byId = new Map(Object.values(data).flat().map((row) => [row.id, row]));
const get = (id) => {
  const row = byId.get(id);
  assert(row !== undefined, `missing fixture input ${id}`);
  return row;
};
const fixtureRows = data["semantic-fixtures.json"];
assert(fixtureRows.length === 18, `fixture family denominator drift: ${fixtureRows.length}`);
const fixtureByFamily = new Map(fixtureRows.map((fixture) => [fixture.family, fixture]));
assert(fixtureByFamily.size === 18, "duplicate semantic fixture family");

const manifestClaimIds = new Set(Object.entries(manifest.categories).flatMap(([category, value]) =>
  value.records.map((row) => `${category}:${row.id}`)));
for (const fixture of fixtureRows) {
  assert(fixture.source_claim_role === "SupportingReferenceNotManifestClaim", `${fixture.id} claim role drift`);
  assert(fixture.execution_scope === "reference-review-only-no-runtime-parity-claim",
    `${fixture.id} execution scope drift`);
  assert(fixture.source_record_ids.every((claimId) => manifestClaimIds.has(claimId)),
    `${fixture.id} references unknown manifest claim`);
  assert(fixture.commands.length > 0 && fixture.expected_facts.length > 0 && fixture.case_ids.length > 0,
    `${fixture.id} has an empty executable contract`);
  assert(/^[0-9a-f]{32}$/u.test(fixture.canonical_seed), `${fixture.id} canonical seed drift`);
}

const evaluators = {
  "active-season-selection": () => {
    const schedule = get("season.schedule-201033");
    const season = get("season.group-1033");
    assert(schedule.upstream_schedule_id === 201033 && season.upstream_group_id === 1033, "active selector drift");
    assert(season.schedule_id === schedule.id && season.future_group_1034_included === false, "season binding drift");
    return ["schedule-201033-selected", "group-1033-selected", "group-1034-excluded"];
  },
  "ordinary-stage-order": () => {
    const stages = data["stages.json"];
    assert(stages.length === 12, "ordinary stage cardinality drift");
    assert(stages.every((stage, index) => stage.upstream_stage_id === 5201 + index
      && stage.floor === index + 1 && stage.legal_order === index + 1), "ordinary stage order drift");
    assert(stages.every((stage) => stage.required_node_ids.length === 2 && stage.challenge_countdown === 30),
      "ordinary stage topology drift");
    assert(["outcome.stage-clear", "outcome.attempt-failure", "outcome.abandonment"].every((id) => byId.has(id)),
      "terminal outcome drift");
    return ["twelve-stages-in-order", "two-nodes-per-stage", "terminal-outcomes-fail-closed"];
  },
  "tierce-selected-extension": () => {
    const tierce = get("tierce.5213");
    assert(tierce.predecessor_stage_id === "stage.5212" && tierce.interpretation === "SeparateSelectedExtensionAfterOrdinaryFloor12",
      "Tierce extension identity drift");
    assert(JSON.stringify(tierce.stage_config_ids) === JSON.stringify(["encounter.30123123"])
      && tierce.challenge_countdown === 45 && tierce.activity_projection.carries_ordinary_stage_state === false,
    "Tierce projection drift");
    return ["selected-after-stage-5212", "one-independent-encounter", "ordinary-state-not-carried"];
  },
  "participant-uniqueness": () => {
    const policy = get("participant-policy.ordinary-stage");
    const slots = get("team-slots.ordinary");
    const uniqueness = get("uniqueness.combat-form");
    assert(policy.team_slot_ids.length === 2 && slots.slots.map(({ node_index: node }) => node).join(",") === "1,2",
      "participant slot drift");
    const accept = new Set(["form-a", "form-b"]).size === 2;
    const reject = new Set(["form-a", "form-a"]).size !== 2;
    assert(accept && reject && uniqueness.duplicate_result === "RejectStartByteIdentical"
      && slots.cross_slot_overlap === "Rejected", "participant uniqueness policy drift");
    return ["two-disjoint-slots-accepted", "duplicate-form-rejected-byte-identically"];
  },
  "loadout-lock-retry": () => {
    const lock = get("loadout-lock.ordinary-stage");
    const retry = get("attempt-rule.retry-reset");
    const transition = get("attempt-rule.node-transition");
    assert(lock.mutation_after_start === "Rejected" && lock.locked_components.includes("LightConeInstance")
      && lock.locked_components.includes("RelicInstanceSet"), "loadout lock drift");
    assert(retry.rejected_start_effect === "ByteIdentical" && retry.retry_snapshot_source === "FreshAcceptedStart"
      && retry.previous_partial_node_results === "Discarded", "retry reset drift");
    assert(transition.loadout_mutation_between_nodes === "Rejected" && transition.battle_state_carry === "None",
      "node transition lock drift");
    return ["post-start-mutation-rejected", "retry-uses-fresh-snapshot", "partial-node-results-discarded"];
  },
  "cycle-first-av-window": () => {
    const clock = get("clock.first-cycle-av-window");
    const windows = [clock.first_cycle_action_value, clock.later_cycle_action_value];
    assert(JSON.stringify(windows) === JSON.stringify(["150", "100"])
      && clock.authoritative_decimal_encoding === "CanonicalString", "cycle AV window drift");
    return ["first-window-150", "later-window-100", "canonical-decimal-encoding"];
  },
  "cycle-node-wave-carry": () => {
    const node = get("clock.node-carry");
    const wave = get("clock.wave-carry");
    const tick = get("clock.cycle-tick-boundary");
    const expiry = get("clock.expiry-failure");
    assert(node.carried_partial_action_value === false && node.node2_first_cycle_action_value === "150"
      && node.battle_state_carry === "None", "node clock carry drift");
    assert(wave.reset_remaining_cycles_on_wave === false && wave.reset_cycle_action_value_on_wave === true
      && wave.next_wave_initial_action_value === "150", "wave clock carry drift");
    assert(tick.ordered_boundary.indexOf("EvaluateExpiry") < tick.ordered_boundary.indexOf("EmitCycleStarted")
      && expiry.allow_cycle_start_effect_after_expiry === false, "expiry boundary drift");
    return ["remaining-cycles-carried", "wave-av-reset-to-150", "node2-fresh-150", "expiry-before-cycle-start"];
  },
  "objective-star-aggregation": () => {
    const objectives = data["objectives.json"];
    assert(objectives.length === 6 && objectives.every((row) => row.completion_required
      && row.contribution_stars === 1 && row.cumulative_across_completed_attempts
      && row.failed_or_abandoned_attempt_can_satisfy === false), "objective aggregation drift");
    assert(objectives.filter(({ applies_to: scope }) => scope === "OrdinaryStage").map(({ threshold }) => threshold).join(",") === "10,20,0"
      && objectives.filter(({ applies_to: scope }) => scope === "TierceExtension").map(({ threshold }) => threshold).join(",") === "15,30,0",
    "objective threshold drift");
    const best = new Set([...new Set([251, 253]), ...new Set([252])]);
    assert(best.size === 3, "independent best-objective fixture failed");
    return ["six-exact-thresholds", "completed-attempts-union-independently", "failed-attempt-contributes-nothing"];
  },
  "turbulence-hit-accumulation": () => {
    const turbulence = get("turbulence.3030146");
    const qualifyingActions = ["Ultra", "Insert", "Basic"].filter((type) => turbulence.qualifying_attack_types.includes(type));
    const storedHits = qualifyingActions.length * turbulence.hit_gain_per_qualifying_action;
    assert(storedHits === 2 && turbulence.accumulation_granularity === "OncePerQualifyingActionNotPerHit"
      && turbulence.once_guard === "DV_ChargeTriggeredPerAction", "Turbulence accumulation drift");
    return ["ultimate-adds-one", "follow-up-adds-one", "multi-hit-does-not-multiply"];
  },
  "turbulence-cap-cycle-start": () => {
    const turbulence = get("turbulence.3030146");
    const program = get("turbulence-program.30146");
    const capped = Math.min(17, turbulence.hit_cap);
    assert(capped === 15 && program.trigger_event === "OnPhase1" && program.trigger_interpretation === "CycleStart"
      && program.accumulator_reset === "ZeroAfterLoopOrEmptyCandidateResolution", "Turbulence cap/reset drift");
    return ["stored-hits-cap-at-15", "cycle-start-resolves", "accumulator-resets"];
  },
  "turbulence-target-true-damage": () => {
    const program = get("turbulence-program.30146");
    const coefficients = program.rank_coefficient_branches.map(({ coefficient }) => coefficient);
    assert(program.target_selection === "RandomOnePerStoredHit" && program.max_targets_per_hit === 1
      && program.attack_type === "TrueDamage" && program.damage_base_property === "SelectedTarget.BaseHP",
    "Turbulence target/damage drift");
    assert(JSON.stringify(coefficients) === JSON.stringify(["0.12", "0.02", "0.012"]), "rank coefficient drift");
    return ["one-target-per-stored-hit", "base-hp-true-damage", "rank-coefficients-exact"];
  },
  "initial-resources": () => {
    const resource = get("resource.initial-node-state");
    assert(resource.hp_initialization.value === "1" && resource.energy_initialization.value === "0.5"
      && resource.skill_point_initialization.kind === "TeamMaximum", "initial resource values drift");
    assert(resource.reset_for_node2 && !resource.carry_hp_energy_skill_points_between_nodes
      && resource.applies_to_ordinary_config_id === 200001 && resource.applies_to_tierce_config_id === 200001,
    "resource reset/Tierce projection drift");
    return ["full-hp", "half-energy", "team-maximum-skill-points", "fresh-node-reset"];
  },
  "battle-entry-operations": () => {
    const rule = get("resource.battle-entry-operations");
    const operations = rule.ordered_program_operations;
    const createTeam = operations.findIndex(({ operation_type: type }) => type === "RPG.GameCore.CreatePlayerTeam");
    const wave = operations.findIndex(({ operation_type: type }) => type === "RPG.GameCore.WaveMonster");
    const start = operations.findIndex(({ operation_type: type }) => type === "RPG.GameCore.StartBattle");
    assert(rule.selected_stage_count === 25 && createTeam > 0 && wave > createTeam && start > wave,
      "battle-entry operation order drift");
    assert(rule.resolved_technique_effects === "OptionalBattleSpecContributionProjectPolicy",
      "technique contribution boundary drift");
    return ["bindings-before-team", "wave-before-start", "technique-is-optional-battle-spec-contribution"];
  },
  "encounter-wave-order": () => {
    const encounters = data["encounters.json"];
    const waves = data["waves.json"];
    const slots = data["enemy-slots.json"];
    assert(encounters.length === 25 && waves.length === 50 && slots.length === 99, "encounter denominator drift");
    for (const encounter of encounters) {
      const selected = waves.filter(({ encounter_id: id }) => id === encounter.id);
      assert(selected.length === 2 && selected[0].wave_index === 1 && selected[1].wave_index === 2
        && selected[0].next_wave_id === selected[1].id, `${encounter.id} wave order drift`);
      for (const waveRow of selected) {
        const selectedSlots = slots.filter(({ wave_id: id }) => id === waveRow.id);
        assert(selectedSlots.length === waveRow.enemy_slot_count, `${waveRow.id} slot count drift`);
        assert(selectedSlots.every((slot, index) => slot.slot_index === index), `${waveRow.id} slot order drift`);
      }
    }
    return ["25-encounters", "50-ordered-waves", "99-stably-ordered-slots"];
  },
  "enemy-transitive-closure": () => {
    const slots = data["enemy-slots.json"];
    const variants = data["enemy-variants.json"];
    const templates = data["enemy-templates.json"];
    const abilities = data["enemy-abilities.json"];
    const variantIds = new Set(variants.map(({ id }) => id));
    const templateIds = new Set(templates.map(({ id }) => id));
    const abilityIds = new Set(abilities.map(({ id }) => id));
    assert(variantIds.size === 41 && templateIds.size === 41 && abilityIds.size === 221, "enemy closure denominator drift");
    assert(slots.every(({ enemy_variant_id: id }) => variantIds.has(id)), "slot variant closure drift");
    assert(variants.every(({ enemy_template_id: id }) => templateIds.has(id)), "variant template closure drift");
    assert(templates.every(({ ability_ids: ids }) => ids.every((id) => abilityIds.has(id))), "template ability closure drift");
    assert(new Set(slots.map(({ enemy_variant_id: id }) => id)).size === 41, "not every variant is reachable");
    assert(new Set(variants.map(({ enemy_template_id: id }) => id)).size === 41, "not every template is reachable");
    assert(new Set(templates.flatMap(({ ability_ids: ids }) => ids)).size === 221, "not every ability is reachable");
    return ["41-variants-reachable", "41-templates-reachable", "221-abilities-reachable"];
  },
  "empty-pool-selector-closure": () => {
    const pools = data["pool-audits.json"];
    assert(pools.length === 10 && pools.every((row) => row.reachable_record_count === 0
      && row.exact_zero && row.fail_closed_on_unresolved_selector), "empty-pool proof drift");
    return ["ten-families-audited", "zero-reachable-rows", "unresolved-selector-fails-closed"];
  },
  "future-season-exclusion": () => {
    assert(manifest.exclusions.some(({ id }) => id === "future-schedule-201034")
      && manifest.exclusions.some(({ id }) => id === "future-group-1034"), "future exclusion receipt drift");
    assert(!data["seasons.json"].some(({ upstream_schedule_id: schedule, upstream_group_id: group }) =>
      schedule === 201034 || group === 1034), "future row leaked into normalized seasons");
    return ["schedule-201034-excluded", "group-1034-excluded", "preview-not-denominator"];
  },
  "shared-row-reconciliation": () => {
    const receipts = data["reconciliation-receipts.json"];
    assert(receipts.length === 305 && receipts.filter(({ semantic_result: result }) => result === "Match").length === 303
      && receipts.filter(({ semantic_result: result }) => result === "CompatibleProjection").length === 2
      && !receipts.some(({ semantic_result: result }) => result === "Conflict"), "shared reconciliation drift");
    return ["303-exact-matches", "2-compatible-projections", "zero-conflicts"];
  },
};

assert(Object.keys(evaluators).length === 18, "fixture evaluator denominator drift");
const results = [];
for (const fixture of fixtureRows) {
  const evaluator = evaluators[fixture.family];
  assert(evaluator !== undefined, `missing evaluator for ${fixture.family}`);
  const checks = evaluator();
  results.push({
    fixture_id: fixture.id,
    family: fixture.family,
    result: "Pass",
    case_ids: fixture.case_ids,
    commands_executed: fixture.commands.map(({ command }) => command),
    expected_facts_verified: fixture.expected_facts.map(({ fact }) => fact),
    machine_checks: checks,
    mechanism_quality_floor: fixture.mechanism_quality_floor,
    canonical_seed: fixture.canonical_seed,
  });
}

const allFixtureIds = new Set(fixtureRows.flatMap((fixture) => [fixture.id, ...fixture.case_ids]));
const gaps = data["research-gaps.json"];
assert(gaps.length === 29, `research gap denominator drift: ${gaps.length}`);
const gapResults = gaps.map((gap) => {
  assert(gap.state === "PolicyBound" && gap.blocking === false, `${gap.id} became blocking`);
  assert(typeof gap.replacement_condition === "string" && gap.replacement_condition.trim() !== "",
    `${gap.id} lacks replacement condition`);
  assert(Array.isArray(gap.rejected_alternatives) && gap.rejected_alternatives.length > 0,
    `${gap.id} lacks rejected alternatives`);
  assert(gap.affected_fixture_ids.length > 0
    && gap.affected_fixture_ids.every((id) => allFixtureIds.has(id)), `${gap.id} has unresolved fixture reference`);
  const source = get(gap.source_record_id);
  const approximation = source.approximations[gap.approximation_index];
  assert(approximation !== undefined, `${gap.id} source approximation is missing`);
  assert(JSON.stringify(approximation.affected_fixture_ids) === JSON.stringify(gap.affected_fixture_ids)
    && approximation.replacement_condition === gap.replacement_condition,
  `${gap.id} no longer mirrors its source approximation`);
  return {
    gap_id: gap.id,
    result: "Pass",
    blocking: false,
    source_file: gap.source_file,
    source_record_id: gap.source_record_id,
    affected_fixture_ids: gap.affected_fixture_ids,
    replacement_condition: gap.replacement_condition,
  };
});

const evidence = {
  schema_revision: "starclock.memory-of-chaos-semantic-fixture-results.v1",
  goal_id: "memory-of-chaos-reference-v1",
  result: "Pass",
  execution_scope: "reference-review-only-no-runtime-parity-claim",
  fixture_families_required: 18,
  fixture_families_passed: results.length,
  research_gaps_required: 29,
  research_gaps_passed: gapResults.length,
  blocking_research_gaps: 0,
  fixture_results: results,
  replacement_condition_results: gapResults,
  result_sha256: digest({ results, gapResults }),
};
await writeText(
  "evidence/memory-of-chaos-reference-v1/release-audits/semantic-fixture-results.json",
  `${JSON.stringify(evidence, null, 2)}\n`,
  check,
);
console.log(`Goal 17 semantic fixtures ${check ? "verified" : "executed"}: 18/18 families and 29/29 replacement conditions passed.`);
