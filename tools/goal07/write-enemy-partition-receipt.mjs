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
const planned = audit.enemy_variants.find(
  ({ id }) => id === partition.enemy_variant_ids[0],
);
assert(planned, `${partitionId}: enemy variant is absent from the retained audit`);

const golden = `${goalRoot}/goldens/${partitionId}.json`;
const sourceReview = `${goalRoot}/source-reviews/${partitionId}.json`;
const numericAnchor = `${goalRoot}/sources/${partitionId}-numeric-anchors.json`;
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
const provenanceEvidence = [
  { path: "content-reference/v4.4/enemy-abilities.json" },
  { path: "content-reference/v4.4/enemy-templates.json" },
  { path: "content-reference/v4.4/enemy-variants.json" },
  { path: "content-reference/standard-universe-v1/world-difficulties.json" },
  evidence(numericAnchor),
  evidence(sourceReview),
];
if (partitionId === "G07-P5-M15-S04" || partitionId === "G07-P5-M15-S07") {
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
}[partitionId];

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
  records: encounterScope ? [
    {
      id: encounterScope.recordId,
      runtime_disposition: "Metadata",
      accuracy_disposition: "NotApplicable",
      workbook_evidence: [
        { path: "config/data/Universe.xlsx" },
        { path: "config/data/UniverseBindings.xlsx" },
      ],
      provenance_evidence: [
        { path: "content-reference/standard-universe-v1/encounter-groups.json" },
      ],
    },
  ] : [],
  rules: [],
  fixtures: [],
  enemy_variants: [
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
  encounter_members: encounterScope ? [
    {
      id: encounterScope.memberId,
      runtime_disposition: "ExecutableShared",
      accuracy_disposition: "ExactPublic",
      workbook_evidence: [
        { path: "config/data/Universe.xlsx" },
        { path: "config/data/UniverseBindings.xlsx" },
      ],
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
