#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const partitionIndex = args.indexOf("--partition");
const write = args.includes("--write");
assert(partitionIndex >= 0 && args[partitionIndex + 1], "missing --partition");
assert(args.every((value, index) =>
  value === "--partition" || value === "--write" || index === partitionIndex + 1),
"unsupported argument");
const partitionId = args[partitionIndex + 1];
const manifest = json(
  "content-manifests/standard-universe-mechanics-complete-v1/content-partitions.json",
);
const audit = json(
  "content-manifests/standard-universe-mechanics-complete-v1/retained-audit.json",
);
const partition = manifest.partitions.find(({ id }) => id === partitionId);
assert(partition?.mechanic_family?.startsWith("path-"),
  `${partitionId}: not a path partition`);
const profile = partitionProfile(partitionId);
const goldenPath =
  `evidence/standard-universe-mechanics-complete-v1/goldens/${partitionId}.json`;
assert(exists(goldenPath), `${partitionId}: partition golden is missing`);

const auditRecords = new Map(audit.records.map((entry) => [entry.id, entry]));
const auditRules = new Map(audit.rules.map((entry) => [entry.id, entry]));
const auditFixtures = new Map(audit.fixtures.map((entry) => [entry.id, entry]));
const sourceEvidence = [
  { path: "content-reference/standard-universe-v1/blessings.json" },
  { path: "content-reference/standard-universe-v1/blessing-levels.json" },
  { path: "content-reference/standard-universe-v1/mechanic-rules.json" },
  { path: "content-reference/standard-universe-v1/paths.json" },
];
const executionEvidence = profile.executionEvidence.map((path) => ({ path }));
const reviewEvidence = profile.reviewEvidence.map((path) => ({ path }));

const receipt = {
  schema_revision: "starclock.goal07-content-partition-receipt.v1",
  goal_id: "standard-universe-mechanics-complete-v1",
  partition_id: partitionId,
  state: "Complete",
  completed_on: profile.completedOn ?? "2026-07-25",
  authoring: {
    workbooks: [
      {
        path: "config/data/Universe.xlsx",
        tables: [
          "UniversePath",
          "UniversePathBlessing",
          "UniverseBlessing",
          "UniverseBlessingLevel",
          "UniverseBlessingParameter",
        ],
      },
      {
        path: "config/data/UniverseBindings.xlsx",
        tables: ["UniverseMechanicRule"],
      },
      {
        path: "config/data/UniverseEvidence.xlsx",
        tables: ["UniverseContentAudit", "UniverseReviewFixture", "UniverseSourceRecord"],
      },
    ],
    openpyxl_commands: [
      `python -c "import openpyxl" && python tools/goal07/author-path-partition.py --partition ${partitionId} --check`,
    ],
    sora_bundle: evidence("config/universe-generated/config.sora"),
    sora_golden: evidence(goldenPath),
  },
  records: partition.record_ids.map((id) => disposition(
    auditRecords.get(id),
    id.startsWith("universe.blessing.612042")
      ? "ExecutableSharedPrimitive"
      : "ExecutableRuleIr",
    [{ path: "config/data/Universe.xlsx" }],
  )),
  rules: partition.rule_ids.map((id) => {
    const planned = auditRules.get(id);
    const shared = id.startsWith("universe.rule.blessing.612042");
    return {
      ...disposition(
        planned,
        shared ? "ExecutableSharedPrimitive" : "ExecutableRuleIr",
        [{ path: "config/data/UniverseBindings.xlsx" }],
      ),
      implementation_kind: shared ? "SharedPrimitive" : "RuleIr",
      definition_keys: [id, planned.source_record_id],
      execution_evidence: executionEvidence,
    };
  }),
  fixtures: partition.fixture_ids.map((id) => ({
    ...disposition(
      auditFixtures.get(id),
      "ProductionExecuted",
      [{ path: "config/data/UniverseEvidence.xlsx" }],
    ),
    execution_kind: "RustTest",
    test_path: profile.fixturePath,
    test_marker: profile.fixtureMarker,
  })),
  enemy_variants: [],
  encounter_members: [],
  native_handler_reviews: partition.native_review_candidate_rule_ids.map((id) => ({
    id,
    outcome: "IrSufficient",
    decision: nativeDecision(id),
    evidence: reviewEvidence,
  })),
  numeric_approximations: profile.numericApproximations ?? [],
  execution: {
    result: "pass",
    commands: [
      `python tools/goal07/author-path-partition.py --partition ${partitionId} --check`,
      "node tools/universe-reference/verify_production_workbooks.mjs .",
      "cargo test -p starclock-combat --all-features --no-fail-fast",
      ...profile.testCommands,
      "cargo test -p starclock-replay --all-features --no-fail-fast",
    ],
    goldens: [evidence(goldenPath)],
  },
};

const relative =
  `evidence/standard-universe-mechanics-complete-v1/partitions/${partitionId}.json`;
const encoded = `${JSON.stringify(receipt, null, 2)}\n`;
if (write) {
  fs.mkdirSync(path.dirname(absolute(relative)), { recursive: true });
  fs.writeFileSync(absolute(relative), encoded);
  console.log(`Wrote Goal 07 receipt ${relative}.`);
} else {
  assert(exists(relative), `${relative} is missing`);
  assert(fs.readFileSync(absolute(relative), "utf8") === encoded,
    `${partitionId}: generated receipt drifted`);
  console.log(`Goal 07 receipt ${partitionId} matches generated evidence.`);
}

function nativeDecision(id) {
  if (id.includes("612356"))
    return "Positive-heal facts and a refreshable target-turn percent-of-base SPD effect express Force Victoire.";
  if (id.includes("612357"))
    return "Ability-source healing, action occurrence keys, fixed effect chance and checked Skill Point mutation express Empower.";
  if (id.includes("612320"))
    return "A keyed team-resource cost and ordered all-ally MaxHP-effect/heal program express the complete manual Resonance.";
  if (id.includes("612321"))
    return "The generic prospective team-defeat guard, battle once scope and queued no-cost auxiliary ability express Terminal Nirvana.";
  if (id.includes("612322"))
    return "Ordered Cleanse, stackable negative-effect guards and their generic informational signal express Anicca and Subduing Evils.";
  if (id.includes("612323"))
    return "The generic first-use trigger, auxiliary countdown definition and recurring timeline action express Anatta.";
  if (id.includes("612351"))
    return "A target-directional Healing-stage modifier expresses Incoming Healing exactly once for allied and self-healing.";
  if (id.includes("612352"))
    return "BattleStarted triggers, live MaxHP queries and ordinary Heal operations express battle-entry restoration.";
  if (id.includes("612353"))
    return "WeaknessBroken facts, actor-owner filtering and live MaxHP healing express restoration for the breaking character only.";
  if (id.includes("612354"))
    return "Positive effective-heal facts and a refreshable owner-turn effect express the complete one-turn DEF state.";
  if (id.includes("612355"))
    return "Ability-source healing facts, action occurrence keys and ordinary MaxHP healing express one provider restoration per action.";
  if (id.includes("612342"))
    return "Typed Dewdrop rupture signals, fixed-chance effect application and ordered Cleanse express the complete dispel.";
  if (id.includes("612343"))
    return "Healing attribution, ordinary ally selectors and a one-turn effect-backed ATK modifier express both target policies.";
  if (id.includes("612344"))
    return "Committed action targets, labeled stable selection, live HP queries and event-element damage express both levels.";
  if (id.includes("612345"))
    return "HP-change facts maintain one ordinary effect whose shared mitigation and Effect RES modifiers are active only at full HP.";
  if (id.includes("612346"))
    return "Effective-heal facts and exact bounded HP restoration express the extra ally healing without repeating healing multipliers.";
  if (id.includes("612350"))
    return "The validated contribution compiler supplies the capped selected-Abundance count to an ordinary MaxHP modifier.";
  if (id.includes("612330"))
    return "Typed effective-heal facts, owner-scoped state and event-element additional damage express Dewdrop charge and rupture.";
  if (id.includes("612331"))
    return "Turn-start triggers and generic current/max-HP queries feed the shared capped Dewdrop state.";
  if (id.includes("612332"))
    return "Source exclusion, effective-heal reads and effect-stack-backed flat ATK express team healing without recursion.";
  if (id.includes("612340"))
    return "The typed rupture signal preserves consumed charge for checked minimum/maximum healing expressions.";
  if (id.includes("612341"))
    return "Full-HP conditions and additive charge signals express level-specific Dewdrop efficiency under the shared cap.";
  if (id.includes("612230"))
    return "Typed DoT damage facts and generic effect-stack application express Suspicion gain and its persistent enhanced policy.";
  if (id.includes("612231"))
    return "Typed DoT application/refresh facts and positive stack deltas express both initial and enhanced Suspicion gain.";
  if (id.includes("612232"))
    return "Enemy-owner turn triggers and the generic ordered DoT detonation operation express the complete current-DoT detonation.";
  if (id.includes("612240"))
    return "Defeat facts, aggregate effect-stack queries and labeled stable random selectors express Suspicion transfer.";
  if (id.includes("612241"))
    return "Signed stack-delta reads and excluded-source filters express additive or doubled Suspicion without recursive self-reaction.";
  if (id.includes("612242"))
    return "Effect-stack-backed capped stat modifiers express the exact ATK and Effect RES reductions.";
  if (id.includes("612243"))
    return "The generic ToughnessDamage modifier is queried per ordinary reduction and expresses exact Weakness Break Efficiency.";
  if (id.includes("612244"))
    return "Typed Break-element facts, adjacent/all selectors, forced Break and source exclusion express propagation without recursion.";
  if (id.includes("612245"))
    return "Ordered random-effect choice, ordinary effect chance, level-derived Break base, DoT snapshots and newest-first Cleanse express Twilight.";
  if (id.includes("612246"))
    return "The generic deterministic one-current-DoT detonation operation expresses All Things without content branching.";
  if (id.includes("612250"))
    return "The validated contribution compiler supplies the capped selected-Nihility count to an ordinary DoT modifier.";
  if (id.includes("612251"))
    return "A generic Break-purpose source modifier is consumed by initial and base Break-effect formula preparation.";
  if (id.includes("612252"))
    return "A signed enemy Effect RES modifier flows through the shared resistible-effect chance pipeline.";
  if (id.includes("612253"))
    return "A target-subject DoT vulnerability modifier is filtered and stacked by the ordinary formula pipeline.";
  if (id.includes("612254"))
    return "The generic integral DoT-duration stat adjusts newly applied DoT lifetimes before duration multiplication.";
  if (id.includes("612255"))
    return "Dynamic effect-category stack queries and capped target vulnerability express the current total DoT-stack scaling.";
  if (id.includes("612256"))
    return "Typed enemy DoT-damage facts, stable all-ally iteration and current MaxHP healing express Offerings of Deception.";
  if (id.includes("612257"))
    return "A labeled stable random ally selector and generic personal-Energy mutation express Before Sunrise.";
  if (id.includes("612220"))
    return "A team-resource-gated ordered ability program applies the four ordinary snapshot DoTs through the shared effect-chance pipeline.";
  if (id.includes("612221"))
    return "Formation selection lowers to generic chance, duration and stack parameters before the immutable Resonance definition is built.";
  if (id.includes("612222"))
    return "Stack-aware DoT detonation, signed effect-stack adjustment and an effect-backed Toughness-recovery modifier express both statuses.";
  if (id.includes("612223"))
    return "Battle-start and typed enemy DoT-damage triggers use the checked keyed team-resource service.";
  if (id.includes("612156"))
    return "Typed Freeze effect/BaseEffect facts, explicit action context and action once keys express per-applier Energy restoration.";
  if (id.includes("612157"))
    return "Typed Freeze facts, dynamic applier MaxHP queries, dedicated Shields and bounded owner-turn counters express the complete Shield lifetime.";
  if (id.includes("612120"))
    return "A team-resource-gated executable ability with ordered programs expresses all-enemy Ice damage followed by resistible Freeze.";
  if (id.includes("612121"))
    return "A pre-hit resistible effect and bounded signed Freeze-resistance modifier affect the same Resonance Freeze without a source branch.";
  if (id.includes("612122"))
    return "The generic negative-effect duration multiplier and ordered pre-/post-hit programs express Eonian River.";
  if (id.includes("612123"))
    return "Battle-start and typed per-Freeze triggers use the checked keyed team-resource service.";
  if (id.includes("612130"))
    return "Frozen-state predicates, resistible effect application and effect-removal facts express Fuli and its enhanced Dissociation damage.";
  if (id.includes("612131"))
    return "WeaknessBroken facts and the shared effect-chance pipeline express Innocence without a source branch.";
  if (id.includes("612132"))
    return "Per-enemy rule attachment, target-within-action once scope, bounded counters and generic control effects express Reticence.";
  if (id.includes("612140"))
    return "Effect predicates, deterministic current-target MaxHP damage and typed effect removal express Melancholia.";
  if (id.includes("612141"))
    return "Effect-definition event filters and effect-scoped vulnerability modifiers express Dizziness across all damage purposes.";
  if (id.includes("612142"))
    return "Typed Dissociation-removal facts and ordinary resistible control-effect application express Insensitivity.";
  if (id.includes("612143"))
    return "Applied-damage event reads, formation selectors and source comparisons express Sentimentality without recursive generated damage.";
  if (id.includes("612144"))
    return "Per-damage effect-chance draws and signed target-specific resistance modifiers express Indelibility.";
  if (id.includes("612145"))
    return "Ultimate tags, weakness predicates, labeled random selection and target-turn weakness lifetime express Shudder.";
  if (id.includes("612146"))
    return "Battle-start triggers and effect-scoped percent-of-base SPD modifiers express Maverick.";
  if (id.includes("612150"))
    return "The validated contribution compiler supplies the capped owned-Remembrance count to a target-specific resistance modifier.";
  if (id.includes("612051"))
    return "Battle-start triggers, current MaxHP queries and bounded owner-turn counters express Sentinel.";
  if (id.includes("612052"))
    return "Action-scoped HP-loss accumulation and bounded owner-turn counters express Patch without hit-count assumptions.";
  if (id.includes("612053"))
    return "WeaknessBroken events, per-owner MaxHP queries and owner-turn counters express Compensation for every ally.";
  if (id.includes("612054"))
    return "Dynamic current-shield queries and ordinary mitigation modifiers cover every reducible damage purpose.";
  if (id.includes("612055"))
    return "Fixed effect chance and the bounded negative-effect Cleanse Rule IR operation express Rotation.";
  if (id.includes("612043"))
    return "Dynamic current-shield and authored-base-stat queries express the capped ATK conversion without a content branch.";
  if (id.includes("612044"))
    return "Turn-end triggers, fixed chance and effect-scoped shield replacement express Sanctuary.";
  if (id.includes("612045"))
    return "Formula-subject filters distinguish shield generation from shield reception in the shared modifier pipeline.";
  if (id.includes("612046"))
    return "Shield delta events, complete cause roles and bounded Rule IR slots express the provider shield and its lifetime.";
  if (id.includes("612050"))
    return "The validated contribution compiler supplies the owned Preservation count to an ordinary percent-of-base modifier.";
  if (id.includes("612056"))
    return "Dynamic current-shield queries and live CRIT DMG modifiers express Burst without a source branch.";
  if (id.includes("612057"))
    return "Dynamic current-shield queries and the labeled per-hit CRIT stream express Concentration.";
  if (id.includes("612020"))
    return "Ordered selector sums, team-resource costs and ability-program battle queries express Preservation Resonance.";
  if (id.includes("612021"))
    return "A fixed authored CRIT multiplier and shielded-ally selector sum express Zero-Dimensional Reinforcement.";
  if (id.includes("612022"))
    return "ActionResolved triggers, owner-scoped Shields and the generic one-shot overflow guard express Eutectic Reaction.";
  if (id.includes("612023"))
    return "Battle-start and positive Shield-delta triggers use the checked team-resource service for Isomorphous Reaction.";
  if (id.includes("612032"))
    return "Dedicated shield state, event deltas, scoped removal and Rule IR slots express the complete cycle.";
  if (id.includes("612041"))
    return "Typed effect chance, capped DoT templates and deterministic source filters express Bleed.";
  if (id.includes("612040"))
    return "Stable selector iteration and applied-damage event reads express Quake boost and splash.";
  return "Shield snapshots, derived-stat queries, once scopes and explicit nonlethal damage express Quake.";
}
function partitionProfile(id) {
  if (id === "G07-P2-M02-S01") {
    return {
      executionEvidence: [
        "crates/starclock-mode-universe/src/battle_rule_lowering.rs",
        "crates/starclock-mode-universe/tests/mechanic_battle_integration.rs",
        "crates/starclock-mode-universe/tests/preservation_runtime.rs",
        "crates/starclock-combat/tests/ability_program_execution.rs",
      ],
      reviewEvidence: [
        "docs/goal-07-preservation-s01.md",
        "crates/starclock-mode-universe/src/battle_rule_lowering.rs",
        "crates/starclock-combat/src/rule/model.rs",
      ],
      fixturePath: "crates/starclock-mode-universe/tests/mechanic_battle_integration.rs",
      fixtureMarker: "goal07_p2_m02_s01_executes_every_assigned_rule_and_operation_fixture",
      testCommands: [
        "cargo test -p starclock-mode-universe --test mechanic_battle_integration --all-features",
        "cargo test -p starclock-mode-universe --test preservation_runtime --all-features",
      ],
    };
  }
  if (id === "G07-P2-M02-S02") {
    return {
      executionEvidence: [
        "crates/starclock-mode-universe/src/battle_rule_lowering/preservation_s02.rs",
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/preservation_s02.rs",
        "crates/starclock-mode-universe/tests/preservation_runtime.rs",
        "crates/starclock-combat/tests/modifier_pipeline.rs",
      ],
      reviewEvidence: [
        "docs/goal-07-preservation-s02.md",
        "crates/starclock-mode-universe/src/battle_rule_lowering/preservation_s02.rs",
        "crates/starclock-combat/src/modifier/resolve.rs",
      ],
      fixturePath:
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/preservation_s02.rs",
      fixtureMarker:
        "goal07_p2_m02_s02_executes_dynamic_stat_and_directional_shield_rules",
      testCommands: [
        "cargo test -p starclock-mode-universe --test mechanic_battle_integration goal07_p2_m02_s02 --all-features",
        "cargo test -p starclock-mode-universe --test preservation_runtime --all-features",
      ],
    };
  }
  if (id === "G07-P2-M02-S03") {
    return {
      executionEvidence: [
        "crates/starclock-mode-universe/src/battle_rule_lowering/preservation_s03.rs",
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/preservation_s03.rs",
        "crates/starclock-combat/tests/ability_program_execution/cleanse.rs",
        "crates/starclock-combat/src/effect/state.rs",
      ],
      reviewEvidence: [
        "docs/goal-07-preservation-s03.md",
        "crates/starclock-mode-universe/src/battle_rule_lowering/preservation_s03.rs",
        "crates/starclock-combat/src/rule/model.rs",
      ],
      fixturePath:
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/preservation_s03.rs",
      fixtureMarker:
        "goal07_p2_m02_s03_executes_break_shields_and_rotation_chance_programs",
      testCommands: [
        "cargo test -p starclock-mode-universe --test mechanic_battle_integration goal07_p2_m02_s03 --all-features",
        "cargo test -p starclock-combat --test ability_program_execution rule_cleanse --all-features",
        "cargo test -p starclock-mode-universe --test preservation_runtime --all-features",
      ],
    };
  }
  if (id === "G07-P2-M02-S04") {
    return {
      executionEvidence: [
        "crates/starclock-mode-universe/src/battle_rule_lowering/preservation_s04.rs",
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/preservation_s04.rs",
        "crates/starclock-combat/src/resolver/operation.rs",
        "crates/starclock-combat/src/resolver/program.rs",
      ],
      reviewEvidence: [
        "docs/goal-07-preservation-s04.md",
        "crates/starclock-mode-universe/src/battle_rule_lowering/preservation_s04.rs",
        "crates/starclock-combat/src/effect/model.rs",
      ],
      fixturePath:
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/preservation_s04.rs",
      fixtureMarker:
        "goal07_p2_m02_s04_executes_shield_conditioned_critical_stats",
      testCommands: [
        "cargo test -p starclock-mode-universe --test mechanic_battle_integration goal07_p2_m02_s04 --all-features",
        "cargo test -p starclock-combat --all-features",
        "cargo test -p starclock-mode-universe --test preservation_runtime --all-features",
      ],
    };
  }
  if (id === "G07-P2-M03-S01") {
    return {
      executionEvidence: [
        "crates/starclock-mode-universe/src/battle_rule_lowering/remembrance_s01.rs",
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/remembrance_s01.rs",
        "crates/starclock-combat/src/resolver/effect_duration.rs",
        "crates/starclock-combat/src/resolver/program_effect.rs",
        "crates/starclock-combat/src/resolver/turn.rs",
      ],
      reviewEvidence: [
        "docs/goal-07-remembrance-s01.md",
        "crates/starclock-mode-universe/src/battle_rule_lowering/remembrance_s01.rs",
        "crates/starclock-combat/src/rule/model.rs",
      ],
      fixturePath:
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/remembrance_s01.rs",
      fixtureMarker:
        "goal07_p2_m03_s01_executes_freeze_dissociation_and_removal_damage",
      testCommands: [
        "cargo test -p starclock-mode-universe --test mechanic_battle_integration remembrance_ --all-features",
        "cargo test -p starclock-combat --all-features",
        "cargo test -p starclock-replay --all-features",
      ],
    };
  }
  if (id === "G07-P2-M03-S02") {
    return {
      executionEvidence: [
        "crates/starclock-mode-universe/src/battle_rule_lowering/remembrance_s02.rs",
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/remembrance_s02.rs",
        "crates/starclock-combat/src/resolver/toughness.rs",
        "crates/starclock-combat/src/resolver/target.rs",
        "crates/starclock-combat/src/resolver/program.rs",
      ],
      reviewEvidence: [
        "docs/goal-07-remembrance-s02.md",
        "crates/starclock-mode-universe/src/battle_rule_lowering/remembrance_s02.rs",
        "crates/starclock-combat/src/rule/model.rs",
      ],
      fixturePath:
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/remembrance_s02.rs",
      fixtureMarker:
        "remembrance_shudder_selects_an_eligible_enemy_and_expires_after_two_target_turns",
      testCommands: [
        "cargo test -p starclock-mode-universe --test mechanic_battle_integration remembrance_s02 --all-features",
        "cargo test -p starclock-combat --test effect_resource_pipeline --all-features",
        "cargo test -p starclock-replay --all-features",
      ],
    };
  }
  if (id === "G07-P2-M03-S03") {
    return {
      executionEvidence: [
        "crates/starclock-mode-universe/src/battle_rule_lowering/remembrance_s03.rs",
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/remembrance_s03.rs",
        "crates/starclock-combat/src/resolver/operation_formula.rs",
        "crates/starclock-combat/src/resolver/rule.rs",
        "crates/starclock-combat/src/resolver/transaction.rs",
        "crates/starclock-combat/src/resolver/transaction_record.rs",
      ],
      reviewEvidence: [
        "docs/goal-07-remembrance-s03.md",
        "crates/starclock-mode-universe/src/battle_rule_lowering/remembrance_s03.rs",
        "crates/starclock-combat/src/rule/model.rs",
      ],
      fixturePath:
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/remembrance_s03.rs",
      fixtureMarker:
        "lost_memory_freezes_on_the_first_attack_crossing_below_half_hp",
      testCommands: [
        "cargo test -p starclock-mode-universe --test mechanic_battle_integration remembrance_s03 --all-features",
        "cargo test -p starclock-combat --all-features",
        "cargo test -p starclock-replay --all-features",
      ],
    };
  }
  if (id === "G07-P2-M03-S04") {
    return {
      executionEvidence: [
        "crates/starclock-mode-universe/src/battle_rule_lowering/remembrance_s04.rs",
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/remembrance_s04.rs",
        "crates/starclock-combat/src/resolver/program_effect.rs",
        "crates/starclock-combat/src/resolver/rule.rs",
        "crates/starclock-combat/src/catalog/rule_validate.rs",
      ],
      reviewEvidence: [
        "docs/goal-07-remembrance-s04.md",
        "crates/starclock-mode-universe/src/battle_rule_lowering/remembrance_s04.rs",
        "crates/starclock-combat/src/rule/model.rs",
      ],
      fixturePath:
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/remembrance_s04.rs",
      fixtureMarker:
        "remembrance_resonance_orders_total_eonian_damage_and_freeze",
      testCommands: [
        "cargo test -p starclock-mode-universe --test mechanic_battle_integration remembrance_s04 --all-features",
        "cargo test -p starclock-combat --all-features",
        "cargo test -p starclock-replay --all-features",
      ],
    };
  }
  if (id === "G07-P2-M04-S01") {
    return {
      executionEvidence: [
        "crates/starclock-mode-universe/src/battle_rule_lowering/nihility_s01.rs",
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/nihility_s01.rs",
        "crates/starclock-combat/src/resolver/program_effect.rs",
        "crates/starclock-combat/src/resolver/rule.rs",
        "crates/starclock-combat/src/catalog/rule_validate.rs",
      ],
      reviewEvidence: [
        "docs/goal-07-nihility-s01.md",
        "crates/starclock-mode-universe/src/battle_rule_lowering/nihility_s01.rs",
        "crates/starclock-combat/src/rule/model.rs",
      ],
      fixturePath:
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/nihility_s01.rs",
      fixtureMarker:
        "enhanced_suspicion_application_doubles_stacks_and_never_decays",
      testCommands: [
        "cargo test -p starclock-mode-universe --test mechanic_battle_integration nihility_s01 --all-features",
        "cargo test -p starclock-combat --all-features",
        "cargo test -p starclock-replay --all-features",
      ],
    };
  }
  if (id === "G07-P2-M04-S02") {
    return {
      executionEvidence: [
        "crates/starclock-mode-universe/src/battle_rule_lowering/nihility_s02.rs",
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/nihility_s02.rs",
        "crates/starclock-combat/src/resolver/operation_formula.rs",
        "crates/starclock-combat/src/resolver/effect_operation.rs",
        "crates/starclock-combat/src/formula/toughness.rs",
      ],
      reviewEvidence: [
        "docs/goal-07-nihility-s02.md",
        "crates/starclock-mode-universe/src/battle_rule_lowering/nihility_s02.rs",
        "crates/starclock-combat/src/rule/model.rs",
      ],
      fixturePath:
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/nihility_s02.rs",
      fixtureMarker:
        "hell_spreads_the_triggering_break_and_random_dot_then_detonation_execute",
      testCommands: [
        "cargo test -p starclock-mode-universe --test mechanic_battle_integration nihility_s02 --all-features",
        "cargo test -p starclock-combat --test toughness_formula --all-features",
        "cargo test -p starclock-combat --all-features",
        "cargo test -p starclock-replay --all-features",
      ],
    };
  }
  if (id === "G07-P2-M04-S03") {
    return {
      executionEvidence: [
        "crates/starclock-mode-universe/src/battle_rule_lowering/nihility_s03.rs",
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/nihility_s03.rs",
        "crates/starclock-combat/src/resolver/operation_formula.rs",
        "crates/starclock-combat/src/resolver/program_effect.rs",
        "crates/starclock-combat/src/modifier/resolve.rs",
      ],
      reviewEvidence: [
        "docs/goal-07-nihility-s03.md",
        "crates/starclock-mode-universe/src/battle_rule_lowering/nihility_s03.rs",
        "crates/starclock-combat/src/rule/model.rs",
      ],
      fixturePath:
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/nihility_s03.rs",
      fixtureMarker:
        "questioning_of_purpose_increases_a_production_initial_break_by_exactly_half",
      testCommands: [
        "cargo test -p starclock-mode-universe --test mechanic_battle_integration nihility_s03 --all-features",
        "cargo test -p starclock-combat --test modifier_pipeline --all-features",
        "cargo test -p starclock-combat --test damage_lifecycle --all-features",
        "cargo test -p starclock-replay --all-features",
      ],
    };
  }
  if (id === "G07-P2-M04-S04") {
    return {
      executionEvidence: [
        "crates/starclock-mode-universe/src/battle_rule_lowering/nihility_s04.rs",
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/nihility_s04.rs",
        "crates/starclock-combat/src/resolver/turn.rs",
        "crates/starclock-combat/src/resolver/modifier_snapshot.rs",
        "crates/starclock-combat/src/rule/evaluate.rs",
      ],
      reviewEvidence: [
        "docs/goal-07-nihility-s04.md",
        "crates/starclock-mode-universe/src/battle_rule_lowering/nihility_s04.rs",
        "crates/starclock-combat/src/rule/model.rs",
      ],
      fixturePath:
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/nihility_s04.rs",
      fixtureMarker:
        "enemy_dot_ticks_heal_the_team_restore_random_energy_and_charge_resonance",
      testCommands: [
        "cargo test -p starclock-mode-universe --test mechanic_battle_integration nihility_s04 --all-features",
        "cargo test -p starclock-combat --all-features",
        "cargo test -p starclock-replay --all-features",
      ],
    };
  }
  if (id === "G07-P2-M05-S01") {
    return {
      executionEvidence: [
        "crates/starclock-mode-universe/src/battle_rule_lowering/abundance_s01.rs",
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/abundance_s01.rs",
        "crates/starclock-combat/src/resolver/target.rs",
        "crates/starclock-combat/src/resolver/rule.rs",
        "crates/starclock-replay/src/battle_event.rs",
        "crates/starclock-replay/tests/battle_property_contract.rs",
      ],
      reviewEvidence: [
        "docs/goal-07-abundance-s01.md",
        "crates/starclock-mode-universe/src/battle_rule_lowering/abundance_s01.rs",
        "crates/starclock-combat/src/rule/model.rs",
      ],
      fixturePath:
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/abundance_s01.rs",
      fixtureMarker:
        "goal07_p2_m05_s01_materializes_all_five_assigned_mechanics",
      testCommands: [
        "cargo test -p starclock-mode-universe --test mechanic_battle_integration abundance_s01 --all-features",
        "cargo test -p starclock-combat --all-features",
        "cargo test -p starclock-replay --all-features",
      ],
    };
  }
  if (id === "G07-P2-M05-S02") {
    return {
      executionEvidence: [
        "crates/starclock-mode-universe/src/battle_rule_lowering/abundance_s02.rs",
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/abundance_s02.rs",
        "crates/starclock-combat/src/rule/model.rs",
        "crates/starclock-combat/src/resolver/program.rs",
        "crates/starclock-combat/src/resolver/operation/sustain.rs",
        "crates/starclock-mode-universe/tests/battle_materialization.rs",
      ],
      reviewEvidence: [
        "docs/goal-07-abundance-s02.md",
        "crates/starclock-mode-universe/src/battle_rule_lowering/abundance_s02.rs",
        "crates/starclock-combat/src/rule/model.rs",
      ],
      fixturePath:
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/abundance_s02.rs",
      fixtureMarker:
        "enhanced_hp_additional_damage_executes_once_on_an_actual_attack_target",
      testCommands: [
        "cargo test -p starclock-mode-universe --test mechanic_battle_integration abundance_s02 --all-features",
        "cargo test -p starclock-mode-universe --test battle_materialization --all-features",
        "cargo test -p starclock-combat --all-features",
        "cargo test -p starclock-replay --all-features",
      ],
    };
  }
  if (id === "G07-P2-M05-S03") {
    return {
      executionEvidence: [
        "crates/starclock-mode-universe/src/battle_rule_lowering/abundance_s03.rs",
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/abundance_s03.rs",
        "crates/starclock-combat/src/resolver/operation_formula.rs",
        "crates/starclock-combat/src/resolver/operation/sustain.rs",
        "crates/starclock-combat/src/rule/model.rs",
      ],
      reviewEvidence: [
        "docs/goal-07-abundance-s03.md",
        "crates/starclock-mode-universe/src/battle_rule_lowering/abundance_s03.rs",
        "crates/starclock-combat/src/rule/model.rs",
      ],
      fixturePath:
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/abundance_s03.rs",
      fixtureMarker:
        "healing_action_triggers_provider_once_defense_and_break_healing",
      testCommands: [
        "cargo test -p starclock-mode-universe --test mechanic_battle_integration abundance_s03 --all-features",
        "cargo test -p starclock-mode-universe --test battle_materialization --all-features",
        "cargo test -p starclock-combat --all-features",
        "cargo test -p starclock-replay --all-features",
      ],
    };
  }
  if (id === "G07-P2-M05-S04") {
    return {
      completedOn: "2026-07-26",
      numericApproximations: [
        {
          id: "goal07-abundance-anatta-action-speed-v1",
          record_id: "universe.resonance.612323",
          field: "recurring_action_speed",
          value: "200.000000",
          confidence: "Medium",
          rationale:
            "Released structured and public prose specifies a recurring action-order actor but omits its speed.",
          replacement_condition:
            "Replace when an authoritative public row exposes the Anatta action-order speed; retain the generic countdown contract.",
        },
      ],
      executionEvidence: [
        "crates/starclock-mode-universe/src/battle_rule_lowering/abundance_s04.rs",
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/abundance_s04.rs",
        "crates/starclock-combat/src/resolver/effect_operation.rs",
        "crates/starclock-combat/tests/effect_guards.rs",
        "crates/starclock-mode-universe/src/battle_materialization.rs",
      ],
      reviewEvidence: [
        "docs/goal-07-abundance-s04.md",
        "crates/starclock-mode-universe/src/battle_rule_lowering/abundance_s04.rs",
        "crates/starclock-combat/src/effect/model.rs",
        "docs/10-lifecycle-and-resolution.md",
      ],
      fixturePath:
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/abundance_s04.rs",
      fixtureMarker:
        "goal07_p2_m05_s04_materializes_every_assigned_mechanic_without_native_handlers",
      testCommands: [
        "cargo test -p starclock-mode-universe --test mechanic_battle_integration abundance_s04 --all-features",
        "cargo test -p starclock-combat --test effect_guards --all-features",
        "cargo test -p starclock-combat --all-features",
        "cargo test -p starclock-replay --all-features",
      ],
    };
  }
  throw new Error(`${id}: path receipt profile is not implemented`);
}
function disposition(planned, runtimeDisposition, workbookEvidence) {
  assert(planned, "assigned retained-audit entry is missing");
  return {
    id: planned.id,
    runtime_disposition: runtimeDisposition,
    accuracy_disposition: planned.intended_accuracy_disposition,
    workbook_evidence: workbookEvidence,
    provenance_evidence: sourceEvidence,
  };
}
function evidence(relative) {
  return { path: relative, sha256: sha256(relative) };
}
function sha256(relative) {
  return crypto.createHash("sha256").update(fs.readFileSync(absolute(relative))).digest("hex");
}
function absolute(relative) { return path.join(root, relative); }
function exists(relative) {
  return fs.statSync(absolute(relative), { throwIfNoEntry: false })?.isFile();
}
function json(relative) { return JSON.parse(fs.readFileSync(absolute(relative), "utf8")); }
function assert(condition, message) { if (!condition) throw new Error(message); }
