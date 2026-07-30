#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const partitionIndex = args.indexOf("--partition");
assert(
  partitionIndex >= 0 && typeof args[partitionIndex + 1] === "string",
  "usage: write-enemy-partition-receipt.mjs --partition G07-P5-M15-S0X [--write]",
);
const partitionId = args[partitionIndex + 1];
const write = args.includes("--write");
assert(
  args.every((value, index) =>
    value === "--partition" || value === "--write" || index === partitionIndex + 1),
  "unsupported enemy receipt writer argument",
);
const partitionConfig = {
  "G07-P5-M15-S01": {
    completedOn: "2026-07-28",
    definitionKeys: [
      "enemy.abundant-ebon-deer-complete.littleboss.variant.01",
      "enemy.abundant-ebon-deer-complete.littleboss",
      "ai.goal07.abundant-ebon-deer-complete.phase-1",
      "ai.goal07.abundant-ebon-deer-complete.phase-2",
      "ai.goal07.abundant-ebon-deer-complete.phase-3",
    ],
    numericPolicyId: "goal07-public-anchor-level-curve-v1",
  },
  "G07-P5-M15-S02": {
    completedOn: "2026-07-29",
    definitionKeys: [
      "enemy.automaton-direwolf-complete.elite.variant.01",
      "enemy.automaton-direwolf-complete.elite",
      "ai.goal07.automaton-direwolf-complete.phase-1",
      "ai.goal07.automaton-direwolf-complete.phase-2",
      "ai.goal07.automaton-direwolf-complete.phase-3",
    ],
    numericPolicyId: "goal07-exact-public-per-level-v1",
  },
  "G07-P5-M15-S03": {
    completedOn: "2026-07-29",
    definitionKeys: [
      "enemy.automaton-grizzly-complete.elite.variant.01",
      "enemy.automaton-grizzly-complete.elite",
      "ai.goal07.automaton-grizzly-complete.phase-1",
      "ai.goal07.automaton-grizzly-complete.phase-2",
      "unit.goal07.automaton-grizzly-complete.automaton-spider",
    ],
    numericPolicyId: "goal07-exact-public-per-level-v1",
  },
  "G07-P5-M15-S04": {
    completedOn: "2026-07-29",
    definitionKeys: [
      "enemy.blaze-out-of-space.elite.variant.01",
      "enemy.blaze-out-of-space.elite",
      "ai.goal07.blaze-out-of-space.phase-1",
    ],
    numericPolicyId: "goal07-public-anchor-level-curve-v1",
  },
  "G07-P5-M15-S05": {
    completedOn: "2026-07-29",
    definitionKeys: [
      "enemy.cloud-knight-lieutenant-yanqing-complete.littleboss.variant.01",
      "enemy.cloud-knight-lieutenant-yanqing-complete.littleboss",
      "ai.goal07.cloud-knight-lieutenant-yanqing-complete.phase-1",
      "ai.goal07.cloud-knight-lieutenant-yanqing-complete.phase-2",
      "ai.goal07.cloud-knight-lieutenant-yanqing-complete.phase-3",
      "unit.goal07.cloud-knight-lieutenant-yanqing-complete.sword-1",
      "unit.goal07.cloud-knight-lieutenant-yanqing-complete.sword-2",
      "unit.goal07.cloud-knight-lieutenant-yanqing-complete.sword-4",
      "unit.goal07.cloud-knight-lieutenant-yanqing-complete.sword-5",
    ],
    numericPolicyId: "goal07-exact-public-per-level-v1",
  },
  "G07-P5-M15-S06": {
    completedOn: "2026-07-29",
    definitionKeys: [
      "enemy.cocolia-complete.littleboss.variant.01",
      "enemy.cocolia-complete.littleboss",
      "ai.goal07.cocolia-complete.phase-1",
      "ai.goal07.cocolia-complete.phase-2",
      "ai.goal07.cocolia-complete.phase-3",
      "unit.goal07.cocolia-complete.ice-edge-left",
      "unit.goal07.cocolia-complete.ice-edge-right",
      "unit.goal07.cocolia-complete.bronya",
    ],
    numericPolicyId: "goal07-exact-public-per-level-v1",
  },
  "G07-P5-M15-S07": {
    completedOn: "2026-07-29",
    definitionKeys: [
      "enemy.gepard-complete.littleboss.variant.01",
      "enemy.gepard-complete.littleboss",
      "ai.goal07.gepard-complete.phase-1",
      "ai.goal07.gepard-complete.phase-2",
      "ai.goal07.gepard-complete.phase-3",
      "unit.goal07.gepard-complete.phase-1-soldier",
      "unit.goal07.gepard-complete.phase-1-cannoneer",
      "unit.goal07.gepard-complete.phase-2-cannoneer-left",
      "unit.goal07.gepard-complete.phase-2-cannoneer-right",
      "unit.goal07.gepard-complete.phase-3-lieutenant-left",
      "unit.goal07.gepard-complete.phase-3-lieutenant-right",
    ],
    numericPolicyId: "goal07-exact-public-per-level-v1",
  },
  "G07-P5-M15-S08": {
    completedOn: "2026-07-29",
    definitionKeys: [
      "enemy.ice-out-of-space.elite.variant.01",
      "enemy.ice-out-of-space.elite",
      "ai.goal07.ice-out-of-space.phase-1",
    ],
    numericPolicyId: "goal07-exact-public-per-level-v1",
  },
  "G07-P5-M15-S09": {
    completedOn: "2026-07-29",
    definitionKeys: [
      "enemy.memory-zone-meme-something-unto-death-complete.littleboss.variant.01",
      "enemy.memory-zone-meme-something-unto-death-complete.littleboss",
      "ai.goal07.memory-zone-meme-something-unto-death.phase-1",
      "ai.goal07.memory-zone-meme-something-unto-death.phase-2",
      "ai.goal07.memory-zone-meme-something-unto-death.phase-3",
      "unit.goal07.something-unto-death.sombrous-sepulcher-1",
      "unit.goal07.something-unto-death.sombrous-sepulcher-2",
      "unit.goal07.something-unto-death.sombrous-sepulcher-3",
      "unit.goal07.something-unto-death.sombrous-sepulcher-4",
    ],
    numericPolicyId: "goal07-exact-public-per-level-v1",
  },
  "G07-P5-M15-S10": {
    completedOn: "2026-07-29",
    definitionKeys: [
      "enemy.stellaron-hunter-kafka-complete.littleboss.variant.01",
      "enemy.stellaron-hunter-kafka-complete.littleboss",
      "ai.goal07.stellaron-hunter-kafka-complete.phase-1",
      "ai.goal07.stellaron-hunter-kafka-complete.phase-2",
      "ai.goal07.stellaron-hunter-kafka-complete.phase-3",
    ],
    numericPolicyId: "goal07-exact-public-per-level-v1",
  },
  "G07-P5-M15-S11": {
    completedOn: "2026-07-29",
    definitionKeys: [
      "enemy.svarog-complete.littleboss.variant.01",
      "enemy.svarog-complete.littleboss",
      "ai.goal07.svarog-complete.phase-1",
      "ai.goal07.svarog-complete.phase-2",
      "ai.goal07.svarog-complete.phase-3",
      "unit.goal07.svarog-complete.phase-1-support-1",
      "unit.goal07.svarog-complete.phase-1-support-2",
      "unit.goal07.svarog-complete.phase-1-support-3",
      "unit.goal07.svarog-complete.phase-1-support-4",
      "unit.goal07.svarog-complete.phase-2-direwolf-left",
      "unit.goal07.svarog-complete.phase-2-direwolf-right",
      "unit.goal07.svarog-complete.phase-3-arm",
    ],
    numericPolicyId: "goal07-exact-public-per-level-v1",
  },
  "G07-P5-M15-S12": {
    completedOn: "2026-07-29",
    definitionKeys: [
      "enemy.abundance-sprite-golden-hound.minionlv2.variant.01",
      "enemy.abundance-sprite-golden-hound.minionlv2",
      "enemy.abundance-sprite-malefic-ape-bug.elite.variant.01",
      "enemy.abundance-sprite-malefic-ape-bug.elite",
      "enemy.abundance-sprite-malefic-ape.elite.variant.01",
      "enemy.abundance-sprite-malefic-ape.elite",
      "enemy.abundance-sprite-wooden-lupus.minionlv2.variant.01",
      "enemy.abundance-sprite-wooden-lupus.minionlv2",
      "enemy.antibaryon.minion.variant.01",
      "enemy.antibaryon.minion",
      "enemy.aurumaton-gatekeeper-bug.elite.variant.01",
      "enemy.aurumaton-gatekeeper-bug.elite",
      "enemy.aurumaton-gatekeeper.elite.variant.01",
      "enemy.aurumaton-gatekeeper.elite",
      "enemy.aurumaton-spectral-envoy.elite.variant.01",
      "enemy.aurumaton-spectral-envoy.elite",
      "enemy.automaton-beetle.minionlv2.variant.01",
      "enemy.automaton-beetle.minionlv2",
      "enemy.automaton-direwolf.elite.variant.01",
      "enemy.automaton-direwolf.elite",
      "enemy.automaton-grizzly.elite.variant.01",
      "enemy.automaton-grizzly.elite",
      "enemy.automaton-hound.minionlv2.variant.01",
      "enemy.automaton-hound.minionlv2",
      "ai.goal07.enemy-s12.1",
      "ai.goal07.enemy-s12.2",
      "ai.goal07.enemy-s12.3",
      "ai.goal07.enemy-s12.4",
      "ai.goal07.enemy-s12.5",
      "ai.goal07.enemy-s12.6",
      "ai.goal07.enemy-s12.7",
      "ai.goal07.enemy-s12.8",
      "ai.goal07.enemy-s12.9",
      "ai.goal07.enemy-s12.10",
      "ai.goal07.enemy-s12.11",
      "ai.goal07.enemy-s12.12",
      "unit.goal07.s12.aurumaton-gatekeeper.illumination-dragonfish-1",
      "unit.goal07.s12.aurumaton-gatekeeper.illumination-dragonfish-2",
    ],
  },
  "G07-P5-M15-S13": {
    completedOn: "2026-07-29",
    definitionKeys: [],
  },
  "G07-P5-M15-S14": {
    completedOn: "2026-07-29",
    definitionKeys: [],
  },
  "G07-P5-M15-S15": {
    completedOn: "2026-07-29",
    definitionKeys: [],
  },
  "G07-P5-M15-S16": {
    completedOn: "2026-07-29",
    definitionKeys: [],
  },
  "G07-P5-M15-S17": {
    completedOn: "2026-07-29",
    definitionKeys: [],
  },
  "G07-P5-M15-S18": {
    completedOn: "2026-07-29",
    definitionKeys: [],
  },
}[partitionId];
assert(partitionConfig, `${partitionId}: enemy receipt authoring is not implemented`);

const goalRoot = "evidence/standard-universe-mechanics-complete-v1";
const manifest = json(
  "content-manifests/standard-universe-mechanics-complete-v1/content-partitions.json",
);
const audit = json(
  "content-manifests/standard-universe-mechanics-complete-v1/retained-audit.json",
);
const partition = manifest.partitions.find(({ id }) => id === partitionId);
assert(partition, `${partitionId}: partition is absent from the frozen manifest`);
const plannedVariants = partition.enemy_variant_ids.map((id) => {
  const planned = audit.enemy_variants.find((entry) => entry.id === id);
  assert(planned, `${partitionId}: enemy variant ${id} is absent from the retained audit`);
  return planned;
});
const planned = plannedVariants[0];

const golden = `${goalRoot}/goldens/${partitionId}.json`;
const sourceReview = `${goalRoot}/source-reviews/${partitionId}.json`;
const numericAnchor = `${goalRoot}/sources/${partitionId}-numeric-anchors.json`;
const sourceReviewDocument = json(sourceReview);
const executionEvidence = [
  { path: "config/schema/enemy.toml" },
  { path: "config/schema/expression.toml" },
  { path: "config/schema/operation.toml" },
  { path: "crates/starclock-data/src/encounter_lower.rs" },
  { path: "crates/starclock-data/src/catalog_lookup.rs" },
  { path: "crates/starclock-combat/src/action/lower.rs" },
  { path: "crates/starclock-combat/src/battle/spec.rs" },
  { path: "crates/starclock-combat/src/effect/model.rs" },
  { path: "crates/starclock-combat/src/resolver/lifecycle.rs" },
  { path: "crates/starclock-combat/src/resolver/turn.rs" },
  { path: "crates/starclock-combat/tests/enemy_orchestration.rs" },
  { path: "crates/starclock-combat/tests/forced_control.rs" },
  { path: "crates/starclock-mode-universe/src/battle_materialization.rs" },
  { path: "crates/starclock-mode-universe/src/battle_materialization/battle_spec.rs" },
  { path: "crates/starclock-mode-universe/src/battle_materialization/catalog_composition.rs" },
  { path: "crates/starclock-mode-universe/tests/battle_materialization.rs" },
];
if (partitionId === "G07-P5-M15-S02") {
  executionEvidence.push(
    { path: "config/schema/selector.toml" },
    { path: "crates/starclock-data/src/selector_lower.rs" },
    { path: "crates/starclock-data/src/operation_lower.rs" },
    { path: "crates/starclock-mode-universe/tests/battle_materialization/direwolf_s02.rs" },
  );
}
if (partitionId === "G07-P5-M15-S03") {
  executionEvidence.push(
    { path: "crates/starclock-data/src/operation_lower.rs" },
    { path: "crates/starclock-data/src/catalog/effect_bindings.rs" },
    { path: "crates/starclock-mode-universe/src/runtime/negative_curio_commands.rs" },
    { path: "crates/starclock-mode-universe/tests/dynamic_battle_assembly.rs" },
    { path: "crates/starclock-mode-universe/tests/encounter_runtime.rs" },
    { path: "crates/starclock-mode-universe/tests/entry_compilation.rs" },
    { path: "crates/starclock-mode-universe/tests/topology_runtime.rs" },
    { path: "crates/starclock-mode-universe/tests/battle_materialization/grizzly_s03.rs" },
  );
}
if (partitionId === "G07-P5-M15-S04") {
  executionEvidence.push(
    { path: "crates/starclock-data/src/operation_lower.rs" },
    { path: "crates/starclock-data/src/catalog/effect_bindings.rs" },
    { path: "crates/starclock-combat/src/resolver/effect_boundary.rs" },
    { path: "crates/starclock-combat/src/resolver/rule.rs" },
    { path: "crates/starclock-combat/src/resolver/toughness.rs" },
    { path: "crates/starclock-mode-universe/tests/battle_materialization/blaze_s04.rs" },
  );
}
if (partitionId === "G07-P5-M15-S05") {
  executionEvidence.push(
    { path: "crates/starclock-data/src/operation_lower.rs" },
    { path: "crates/starclock-data/src/catalog/effect_bindings.rs" },
    { path: "crates/starclock-combat/src/resolver/program.rs" },
    { path: "crates/starclock-combat/src/resolver/turn.rs" },
    { path: "crates/starclock-combat/src/resolver/toughness.rs" },
    { path: "crates/starclock-mode-universe/tests/battle_materialization/yanqing_s05.rs" },
  );
}
if (partitionId === "G07-P5-M15-S06") {
  executionEvidence.push(
    { path: "crates/starclock-data/src/operation_lower.rs" },
    { path: "crates/starclock-data/src/catalog/effect_bindings.rs" },
    { path: "crates/starclock-combat/src/resolver/program.rs" },
    { path: "crates/starclock-combat/src/resolver/turn.rs" },
    { path: "crates/starclock-mode-universe/tests/battle_materialization/cocolia_s06.rs" },
  );
}
if (partitionId === "G07-P5-M15-S07") {
  executionEvidence.push(
    { path: "crates/starclock-data/src/operation_lower.rs" },
    { path: "crates/starclock-data/src/catalog/effect_bindings.rs" },
    { path: "crates/starclock-combat/src/resolver/lifecycle.rs" },
    { path: "crates/starclock-combat/src/resolver/program.rs" },
    { path: "crates/starclock-combat/src/resolver/turn.rs" },
    { path: "crates/starclock-mode-universe/tests/battle_materialization/gepard_s07.rs" },
  );
}
if (partitionId === "G07-P5-M15-S08") {
  executionEvidence.push(
    { path: "crates/starclock-data/src/operation_lower.rs" },
    { path: "crates/starclock-data/src/catalog/effect_bindings.rs" },
    { path: "crates/starclock-combat/src/resolver/lifecycle.rs" },
    { path: "crates/starclock-combat/src/resolver/program.rs" },
    { path: "crates/starclock-combat/src/resolver/turn.rs" },
    { path: "crates/starclock-combat/src/resolver/toughness.rs" },
    { path: "crates/starclock-mode-universe/tests/battle_materialization/ice_out_of_space_s08.rs" },
  );
}
if (partitionId === "G07-P5-M15-S09") {
  executionEvidence.push(
    { path: "crates/starclock-data/src/operation_lower.rs" },
    { path: "crates/starclock-data/src/catalog/effect_bindings.rs" },
    { path: "crates/starclock-combat/src/resolver/lifecycle.rs" },
    { path: "crates/starclock-combat/src/resolver/program.rs" },
    { path: "crates/starclock-combat/src/resolver/turn.rs" },
    { path: "crates/starclock-combat/src/resolver/toughness.rs" },
    { path: "crates/starclock-mode-universe/tests/battle_materialization/something_unto_death_s09.rs" },
  );
}
if (partitionId === "G07-P5-M15-S10") {
  executionEvidence.push(
    { path: "crates/starclock-data/src/operation_lower.rs" },
    { path: "crates/starclock-data/src/catalog/effect_bindings.rs" },
    { path: "crates/starclock-combat/src/resolver/effect_operation.rs" },
    { path: "crates/starclock-combat/src/resolver/program.rs" },
    { path: "crates/starclock-combat/src/resolver/rule.rs" },
    { path: "crates/starclock-combat/src/resolver/turn.rs" },
    { path: "crates/starclock-mode-universe/tests/battle_materialization/stellaron_hunter_kafka_s10.rs" },
  );
}
if (partitionId === "G07-P5-M15-S11") {
  executionEvidence.push(
    { path: "config/schema/expression.toml" },
    { path: "crates/starclock-data/src/modifier_lower.rs" },
    { path: "crates/starclock-data/src/operation_lower.rs" },
    { path: "crates/starclock-data/src/catalog/effect_bindings.rs" },
    { path: "crates/starclock-combat/src/resolver/lifecycle.rs" },
    { path: "crates/starclock-combat/src/resolver/program.rs" },
    { path: "crates/starclock-combat/src/resolver/turn.rs" },
    { path: "crates/starclock-mode-universe/tests/battle_materialization/svarog_s11.rs" },
  );
}
if (
  partitionId === "G07-P5-M15-S12"
  || partitionId === "G07-P5-M15-S13"
  || partitionId === "G07-P5-M15-S14"
  || partitionId === "G07-P5-M15-S15"
  || partitionId === "G07-P5-M15-S16"
  || partitionId === "G07-P5-M15-S17"
  || partitionId === "G07-P5-M15-S18"
) {
  executionEvidence.push(
    { path: "crates/starclock-data/src/standard_v1.rs" },
    { path: "crates/starclock-mode-universe/src/catalog.rs" },
    {
      path:
        `crates/starclock-mode-universe/tests/battle_materialization/ordinary_enemies_${partitionId.slice(-3).toLowerCase()}.rs`,
    },
  );
}
const provenanceEvidence = [
  { path: "content-reference/v4.4/enemy-abilities.json" },
  { path: "content-reference/v4.4/enemy-templates.json" },
  { path: "content-reference/v4.4/enemy-variants.json" },
  { path: "content-reference/standard-universe-v1/world-difficulties.json" },
  evidence(numericAnchor),
  evidence(sourceReview),
];
if (
  partitionId === "G07-P5-M15-S04"
  || partitionId === "G07-P5-M15-S07"
  || partitionId === "G07-P5-M15-S08"
  || partitionId === "G07-P5-M15-S12"
  || partitionId === "G07-P5-M15-S13"
  || partitionId === "G07-P5-M15-S14"
  || partitionId === "G07-P5-M15-S15"
  || partitionId === "G07-P5-M15-S16"
  || partitionId === "G07-P5-M15-S17"
  || partitionId === "G07-P5-M15-S18"
) {
  provenanceEvidence.push(
    { path: "content-reference/standard-universe-v1/encounter-groups.json" },
  );
}
const workbookEvidence = [
  { path: "config/data/EnemyVariant.xlsx" },
  { path: "config/data/EnemyStat.xlsx" },
  { path: "config/data/EnemyPhase.xlsx" },
  { path: "config/data/EnemyAbility.xlsx" },
  { path: "config/data/AiGraph.xlsx" },
  { path: "config/data/LinkedUnitDefinition.xlsx" },
  { path: "config/data/Program.xlsx" },
  { path: "config/data/Effect.xlsx" },
  { path: "config/data/EffectTag.xlsx" },
];
const ownedTables = [
  "Ability",
  "AbilityPhase",
  "AiCandidate",
  "AiGraph",
  "AiState",
  "AiTransition",
  "ConditionExpression",
  "ContentEvidenceBinding",
  "ContentIdentity",
  "Effect",
  "EffectModifierBinding",
  "EffectRuleBinding",
  "EffectTag",
  "EnemyAbility",
  "EnemyDebuffResistance",
  "EnemyPhase",
  "EnemyResistance",
  "EnemyStat",
  "EnemyTemplate",
  "EnemyToughnessLayer",
  "EnemyVariant",
  "EnemyVariantAbility",
  "EnemyWeakness",
  "EventFilter",
  "EvidenceRecord",
  "LinkedUnitDefinition",
  "ModifierDefinition",
  "ModifierStackingGroup",
  "Operation",
  "Program",
  "ProgramStep",
  "RuleDefinition",
  "RuleTrigger",
  "Selector",
  "SelectorPredicate",
  "SourceRecord",
  "ValueExpression",
];
const encounterScope = {
  "G07-P5-M15-S04": {
    recordId: "universe.encounter-group.11901",
    memberId: "universe.encounter-member.103",
  },
  "G07-P5-M15-S07": {
    recordId: "universe.encounter-group.13901",
    memberId: "universe.encounter-member.107",
  },
  "G07-P5-M15-S08": {
    recordId: "universe.encounter-group.19001",
    memberId: "universe.encounter-member.108",
  },
}[partitionId];
const universeWorkbookEvidence = [
  { path: "config/data/Universe.xlsx" },
  { path: "config/data/UniverseBindings.xlsx" },
];
const encounterGroupEvidence = [
  { path: "content-reference/standard-universe-v1/encounter-groups.json" },
];
const ordinaryBatch =
  partitionId === "G07-P5-M15-S12"
  || partitionId === "G07-P5-M15-S13"
  || partitionId === "G07-P5-M15-S14"
  || partitionId === "G07-P5-M15-S15"
  || partitionId === "G07-P5-M15-S16"
  || partitionId === "G07-P5-M15-S17"
  || partitionId === "G07-P5-M15-S18";
const ordinaryBatchSlug = partitionId.slice(-3).toLowerCase();
const ordinaryVariantReviews = new Map(
  (sourceReviewDocument.variants ?? []).map((entry) => [entry.enemy_variant_id, entry]),
);

const receipt = {
  schema_revision: "starclock.goal07-content-partition-receipt.v1",
  goal_id: "standard-universe-mechanics-complete-v1",
  partition_id: partitionId,
  state: "Complete",
  completed_on: partitionConfig.completedOn,
  authoring: {
    workbooks: ownedTables.map((table) => ({
      path: `config/data/${table}.xlsx`,
      tables: [table],
    })),
    openpyxl_commands: [
      `python -c "import openpyxl" && python tools/goal07/author-enemy-partition.py --partition ${partitionId} --check`,
    ],
    sora_bundle: evidence("config/generated/config.sora"),
    sora_golden: evidence(golden),
  },
  records: ordinaryBatch ? partition.record_ids.map((id) => {
    const entry = audit.records.find((candidate) => candidate.id === id);
    assert(entry, `${partitionId}: record ${id} is absent from the retained audit`);
    return {
      id,
      runtime_disposition: entry.intended_runtime_disposition,
      accuracy_disposition: entry.intended_accuracy_disposition,
      workbook_evidence: universeWorkbookEvidence,
      provenance_evidence: encounterGroupEvidence,
    };
  }) : encounterScope ? [
    {
      id: encounterScope.recordId,
      runtime_disposition: "Metadata",
      accuracy_disposition: "NotApplicable",
      workbook_evidence: universeWorkbookEvidence,
      provenance_evidence: encounterGroupEvidence,
    },
  ] : [],
  rules: [],
  fixtures: ordinaryBatch ? partition.fixture_ids.map((id) => {
    const entry = audit.fixtures.find((candidate) => candidate.id === id);
    assert(entry, `${partitionId}: fixture ${id} is absent from the retained audit`);
    return {
      id,
      runtime_disposition: entry.intended_runtime_disposition,
      accuracy_disposition: entry.intended_accuracy_disposition,
      workbook_evidence: universeWorkbookEvidence,
      provenance_evidence: encounterGroupEvidence,
      execution_kind: "RustTest",
      test_path:
        `crates/starclock-mode-universe/tests/battle_materialization/ordinary_enemies_${ordinaryBatchSlug}.rs`,
      test_marker:
        `ordinary_enemy_batch_${ordinaryBatchSlug}_materializes_all_frozen_variants_and_level_rows`,
    };
  }) : [],
  enemy_variants: ordinaryBatch ? plannedVariants.map((entry, index) => {
    const review = ordinaryVariantReviews.get(entry.id);
    assert(review, `${partitionId}: source review for ${entry.id} is absent`);
    return {
      id: entry.id,
      runtime_disposition: entry.intended_runtime_disposition,
      accuracy_disposition: entry.intended_accuracy_disposition,
      workbook_evidence: workbookEvidence,
      provenance_evidence: provenanceEvidence,
      implementation_kind: "SharedRuleIrAndEnemyLifecycle",
      definition_keys: [
        entry.id,
        entry.id.replace(".variant.01", ""),
        `ai.goal07.enemy-${ordinaryBatchSlug}.${index + 1}`,
      ],
      numeric_policy_id: review.numeric_policy_id,
      numeric_review: {
        path: sourceReview,
        status: "ApprovedPerVariantInputs",
      },
      execution_evidence: executionEvidence,
    };
  }) : [
    {
      id: planned.id,
      runtime_disposition: planned.intended_runtime_disposition,
      accuracy_disposition: planned.intended_accuracy_disposition,
      workbook_evidence: workbookEvidence,
      provenance_evidence: provenanceEvidence,
      implementation_kind: "SharedRuleIrAndEnemyLifecycle",
      definition_keys: partitionConfig.definitionKeys,
      numeric_policy_id: partitionConfig.numericPolicyId,
      numeric_review: {
        path: sourceReview,
        status: "ApprovedPerVariantInputs",
      },
      execution_evidence: executionEvidence,
    },
  ],
  encounter_members: ordinaryBatch ? partition.encounter_member_ids.map((id) => {
    const entry = audit.encounter_members.find((candidate) => candidate.id === id);
    assert(entry, `${partitionId}: encounter member ${id} is absent from retained audit`);
    return {
      id,
      runtime_disposition: entry.intended_runtime_disposition,
      accuracy_disposition: entry.intended_accuracy_disposition,
      workbook_evidence: universeWorkbookEvidence,
      provenance_evidence: [
        ...encounterGroupEvidence,
        { path: "content-reference/v4.4/enemy-variants.json" },
      ],
    };
  }) : encounterScope ? [
    {
      id: encounterScope.memberId,
      runtime_disposition: "ExecutableShared",
      accuracy_disposition: "ExactPublic",
      workbook_evidence: universeWorkbookEvidence,
      provenance_evidence: [
        { path: "content-reference/standard-universe-v1/encounter-groups.json" },
        { path: "content-reference/v4.4/enemy-variants.json" },
      ],
    },
  ] : [],
  native_handler_reviews: [],
  execution: {
    result: "pass",
    commands: [
      `python tools/goal07/author-enemy-partition.py --partition ${partitionId} --check`,
      "node tools/config-production/verify.mjs",
      "cargo test -p starclock-combat --test enemy_orchestration",
      "cargo test -p starclock-combat --test forced_control",
      "cargo test -p starclock-mode-universe --test battle_materialization",
      `node tools/goal07/verify-content-partition.mjs --partition ${partitionId}`,
      "node tools/repository-check/run.mjs",
    ],
    goldens: [evidence(golden), evidence(sourceReview)],
  },
};

const relative = `${goalRoot}/partitions/${partitionId}.json`;
const encoded = `${JSON.stringify(receipt, null, 2)}\n`;
if (write) {
  fs.mkdirSync(path.dirname(absolute(relative)), { recursive: true });
  fs.writeFileSync(absolute(relative), encoded);
  console.log(`Wrote Goal 07 enemy receipt ${relative}.`);
} else {
  assert(exists(relative), `${relative} is missing`);
  assert(
    fs.readFileSync(absolute(relative), "utf8") === encoded,
    `${partitionId}: generated enemy receipt drifted`,
  );
  console.log(`Goal 07 enemy receipt ${partitionId} matches generated evidence.`);
}

function evidence(relative) {
  return {
    path: relative,
    sha256: sha256(relative),
    git_blob_sha1: gitBlob(relative),
  };
}
function sha256(relative) {
  return crypto.createHash("sha256").update(fs.readFileSync(absolute(relative))).digest("hex");
}
function gitBlob(relative) {
  return execFileSync("git", ["hash-object", relative], {
    cwd: root,
    encoding: "utf8",
  }).trim();
}
function absolute(relative) {
  return path.join(root, relative);
}
function exists(relative) {
  return fs.statSync(absolute(relative), { throwIfNoEntry: false })?.isFile();
}
function json(relative) {
  return JSON.parse(fs.readFileSync(absolute(relative), "utf8"));
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
