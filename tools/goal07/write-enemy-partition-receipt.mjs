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
  "usage: write-enemy-partition-receipt.mjs --partition G07-P5-M15-S01 [--write]",
);
const partitionId = args[partitionIndex + 1];
const write = args.includes("--write");
assert(
  args.every((value, index) =>
    value === "--partition" || value === "--write" || index === partitionIndex + 1),
  "unsupported enemy receipt writer argument",
);
assert(partitionId === "G07-P5-M15-S01", `${partitionId}: enemy receipt authoring is not implemented`);

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
const provenanceEvidence = [
  { path: "content-reference/v4.4/enemy-abilities.json" },
  { path: "content-reference/v4.4/enemy-templates.json" },
  { path: "content-reference/v4.4/enemy-variants.json" },
  { path: "content-reference/standard-universe-v1/world-difficulties.json" },
  evidence(numericAnchor),
  evidence(sourceReview),
];
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

const receipt = {
  schema_revision: "starclock.goal07-content-partition-receipt.v1",
  goal_id: "standard-universe-mechanics-complete-v1",
  partition_id: partitionId,
  state: "Complete",
  completed_on: "2026-07-28",
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
  records: [],
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
      definition_keys: [
        planned.id,
        "enemy.abundant-ebon-deer-complete.littleboss",
        "ai.goal07.abundant-ebon-deer-complete.phase-1",
        "ai.goal07.abundant-ebon-deer-complete.phase-2",
        "ai.goal07.abundant-ebon-deer-complete.phase-3",
      ],
      numeric_policy_id: "goal07-public-anchor-level-curve-v1",
      numeric_review: {
        path: sourceReview,
        status: "ApprovedPerVariantInputs",
      },
      execution_evidence: executionEvidence,
    },
  ],
  encounter_members: [],
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
