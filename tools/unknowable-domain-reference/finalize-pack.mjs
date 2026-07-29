#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import {
  ACCESS_DATE,
  GAME_VERSION,
  SOURCE_REVISION,
  canonical,
  createContext,
  sha256,
  slug,
  writeOrCheck,
} from "./lib/common.mjs";

const GOAL08_CHECKPOINT = "2f7b3ccf699c52c2738136b8636d140e053bb2eb";
const GOAL08_MANIFEST_SHA256 =
  "88885b409da0037b4db6a41fcfc6adbbb1bc15a681c519e192251e7fef476085";
const GOAL09_CHECKPOINT = "9bd2ad285de4c10e7ab060f00bf078855923a09c";
const GOAL09_MANIFEST_SHA256 =
  "e466cae0481d93241eaadf6d894b82898d47c9d4863fea262134cbbac10b8850";
const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const context = await createContext(root);
const referenceRoot = path.join(root, "content-reference/unknowable-domain-v1");
const manifestPath =
  "content-manifests/unknowable-domain-v1/content-manifest.json";
const inventoryPath =
  "content-manifests/unknowable-domain-v1/source-inventory.json";
const schemaPath =
  "content-manifests/unknowable-domain-v1/normalized-schema.json";
const fixtureContractPath =
  "content-manifests/unknowable-domain-v1/fixture-contract.json";
const authoringContractPath =
  "content-manifests/unknowable-domain-v1/authoring-contract.json";
const [
  schema,
  sourceManifest,
  sourceInventory,
  fixtureContract,
  authoringContract,
] = await Promise.all([
  json(schemaPath),
  json(manifestPath),
  json(inventoryPath),
  json(fixtureContractPath),
  json(authoringContractPath),
]);
const finalFiles = new Set([
  "mechanic-source-files.json",
  "mechanic-rules.json",
  "sources.json",
  "coverage.json",
  "research-gaps.json",
  "semantic-fixture-families.json",
  "review-fixtures.json",
  "reconciliation-receipts.json",
  "manifest.json",
  "pack-index.json",
]);
const outputs = new Map();
for (const contract of schema.files) {
  if (finalFiles.has(contract.file)) continue;
  outputs.set(contract.file, await json(
    `content-reference/unknowable-domain-v1/${contract.file}`,
  ));
}
const manifestBytes = await fs.readFile(path.join(root, manifestPath));
const inventoryBytes = await fs.readFile(path.join(root, inventoryPath));
const schemaBytes = await fs.readFile(path.join(root, schemaPath));
const fixtureContractBytes = await fs.readFile(
  path.join(root, fixtureContractPath),
);
const manifestRootRef = localRootRef({
  id: "content-manifest",
  relative: manifestPath,
  bytes: manifestBytes,
  revision: schema.bound_content_manifest_sha256,
  mechanismQuality: "ManifestClosure",
});
const inventoryRootRef = localRootRef({
  id: "source-inventory",
  relative: inventoryPath,
  bytes: inventoryBytes,
  revision: "starclock.unknowable-domain-source-inventory.v1",
  mechanismQuality: "InventoryClosure",
});
const schemaRootRef = localRootRef({
  id: "normalized-schema",
  relative: schemaPath,
  bytes: schemaBytes,
  revision: schema.schema_revision,
  mechanismQuality: "SchemaContract",
});
const fixtureContractRootRef = localRootRef({
  id: "fixture-contract",
  relative: fixtureContractPath,
  bytes: fixtureContractBytes,
  revision: fixtureContract.schema_revision,
  mechanismQuality: "FixtureContract",
});

function rows(file) {
  const value = outputs.get(file);
  if (!Array.isArray(value)) throw new Error(`${file} is not a row array`);
  return value;
}

function pick(file, predicate = () => true, label = file) {
  const row = rows(file).find(predicate);
  if (!row) throw new Error(`missing ${label} in ${file}`);
  return row;
}

const mechanicManifest = sourceManifest.categories.mechanic_source_files;
const mechanicSourceRows = [];
const mechanicRuleRows = [];
for (const record of mechanicManifest.records) {
  const sourceValue = await context.readSource(record.source);
  const sourceEntry = {
    sourcePath: record.source,
    locator: "root",
    row: sourceValue,
  };
  const sourceRef = context.sourceRef(sourceEntry);
  const operationAudit = collectOperationTypes(sourceValue);
  const classification = classifyMechanicSource(record.source);
  const sourceId = `unknowable-domain.mechanic-source.${slug(record.source)}`;
  const ruleId = `unknowable-domain.mechanic-rule.${slug(record.source)}`;
  mechanicSourceRows.push({
    ...context.envelope({
      id: sourceId,
      kind: "MechanicSourceFile",
      nameEn: `Mechanic Source ${path.basename(record.source)}`,
      nameZh: `机制源文件 ${path.basename(record.source)}`,
      summaryEn:
        `Pinned ${classification.scope} source preserves ` +
        `${operationAudit.types.length} distinct structured operation type(s) ` +
        "for reference review without runtime lowering.",
      summaryZh:
        `固定的${scopeZh(classification.scope)}源文件保留 ` +
        `${operationAudit.types.length} 个不同结构化操作类型，` +
        "仅用于资料审查，不进行运行时 lowering。",
      sourceRefs: [sourceRef],
      tags: ["mechanic-source", classification.family_id, classification.scope],
    }),
    source_id: record.id,
    path: record.source,
    source_sha256: record.evidence_sha256,
    source_ref_sha256: sourceRef.sha256,
    scope: classification.scope,
    operation_types: operationAudit.types,
    operation_occurrence_count: operationAudit.total,
    operation_types_sha256: sha256(canonical(operationAudit.types)),
    consumer_rule_ids: [ruleId],
    runtime_lowered: false,
  });
  mechanicRuleRows.push({
    ...context.envelope({
      id: ruleId,
      kind: "UnknowableMechanicRule",
      nameEn: `${path.basename(record.source)} Mechanic Boundary`,
      nameZh: `${path.basename(record.source)} 机制边界`,
      summaryEn:
        `Reference-only ${classification.scope} rule preserves the source ` +
        `trigger boundary and ${operationAudit.types.length} ordered operation ` +
        "type(s); it is not executable runtime behavior.",
      summaryZh:
        `仅供资料使用的${scopeZh(classification.scope)}规则保留源触发边界与 ` +
        `${operationAudit.types.length} 个有序操作类型；它不是可执行运行时行为。`,
      sourceRefs: [sourceRef],
      tags: ["mechanic-rule", classification.family_id, classification.scope],
    }),
    source_id: record.id,
    source_file_id: sourceId,
    family_id: classification.family_id,
    scope: classification.scope,
    trigger: classification.trigger,
    ordered_operations: operationAudit.types.map((value, index) => ({
      ordinal: index + 1,
      operation_type: value.operation_type,
      source_occurrences: value.source_occurrences,
    })),
    battle_projection: classification.scope === "Battle"
      ? "SourceProgramPreservedNotLowered"
      : "NotApplicable",
    fixture_ids: [
      `unknowable-domain.review-fixture.${classification.family_id}`,
    ],
    runtime_lowered: false,
  });
}
mechanicSourceRows.sort((left, right) => left.source_id.localeCompare(right.source_id));
mechanicRuleRows.sort((left, right) => left.source_id.localeCompare(right.source_id));
outputs.set("mechanic-source-files.json", mechanicSourceRows);

const familySpecs = familySpecifications();
const policyRefs = new Map();
for (const family of fixtureContract.required_families) {
  const spec = required(familySpecs, family.id, `fixture spec ${family.id}`);
  policyRefs.set(family.id, await context.policyRef(
    `semantic-fixture-${family.id}`,
    `Reference fixture reviews ${family.must_cover.join(", ")} against ` +
      `${spec.records().length} selected normalized source record(s). It ` +
      "preserves exact facts and explicit Unspecified/ProjectPolicy boundaries " +
      "without claiming runtime executability.",
    `Replace the policy-bound assertions only when released structured data ` +
      `or a reproducible public observation proves ${family.must_cover.join(", ")}.`,
  ));
}

const semanticFamilies = fixtureContract.required_families.map((family) => {
  const spec = required(familySpecs, family.id, `fixture spec ${family.id}`);
  const selected = uniqueRows(spec.records(), family.id);
  const policyRef = required(policyRefs, family.id, `policy ref ${family.id}`);
  return {
    ...context.envelope({
      id: `unknowable-domain.semantic-family.${family.id}`,
      kind: "SemanticFixtureFamily",
      nameEn: `${family.id} Semantic Fixture Family`,
      nameZh: `${family.id} 语义夹具族`,
      summaryEn:
        `Non-shrinking review family covering ${family.must_cover.length} ` +
        "required fact(s) with at least one reference-only case.",
      summaryZh:
        `不可缩减的审查族，覆盖 ${family.must_cover.length} 个必需事实，` +
        "并至少包含一个仅供资料使用的案例。",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [fixtureContractRootRef, policyRef],
      tags: ["semantic-family", family.id],
    }),
    source_id: family.id,
    minimum_cases: family.minimum_cases,
    must_cover: family.must_cover,
    selected_source_record_ids: selected.map(({ id }) => id).sort(),
    runtime_executable: false,
  };
}).sort((left, right) => left.source_id.localeCompare(right.source_id));
outputs.set("semantic-fixture-families.json", semanticFamilies);

const reviewFixtures = fixtureContract.required_families.map((family) => {
  const spec = required(familySpecs, family.id, `fixture spec ${family.id}`);
  const selected = uniqueRows(spec.records());
  const policyRef = required(policyRefs, family.id, `policy ref ${family.id}`);
  const refs = uniqueRefs([
    ...selected.flatMap(({ source_refs: values }) => values ?? []),
    policyRef,
  ]);
  return {
    ...context.envelope({
      id: `unknowable-domain.review-fixture.${family.id}`,
      kind: "SemanticReviewFixture",
      nameEn: `${family.id} Reference Review`,
      nameZh: `${family.id} 资料审查`,
      summaryEn:
        `Reference-only semantic review asserts all ${family.must_cover.length} ` +
        "contract facts as exact or explicitly policy-bound.",
      summaryZh:
        `仅供资料使用的语义审查将全部 ${family.must_cover.length} 个契约事实` +
        "断言为精确事实或显式策略边界。",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: refs,
      tags: ["review-fixture", family.id],
    }),
    source_id: family.id,
    family_id: family.id,
    source_record_ids: selected.map(({ id }) => id).sort(),
    preconditions: {
      candidate_lane: "ReferenceCandidate",
      runtime_loading: "Forbidden",
      source_record_count: selected.length,
    },
    input: {
      kind: "SemanticReview",
      candidate_order: "StableIdAscending",
      unresolved_behavior: "FailClosed",
    },
    ordered_operations: family.must_cover.map((fact, index) => ({
      ordinal: index + 1,
      operation: "ReviewRequiredFact",
      fact,
    })),
    expected_facts: family.must_cover.map((fact) => ({
      fact,
      assertion: "ExactOrExplicitlyPolicyBound",
      runtime_claim: false,
    })),
    evidence_refs: refs.map(({ source_id: id }) => id),
    fixture_evidence_quality: "ProjectPolicy",
    runtime_executable: false,
  };
}).sort((left, right) => left.family_id.localeCompare(right.family_id));
outputs.set("review-fixtures.json", reviewFixtures);
outputs.set("mechanic-rules.json", mechanicRuleRows);

const researchGaps = fixtureContract.required_families.map((family) => {
  const spec = required(familySpecs, family.id, `fixture spec ${family.id}`);
  const selected = uniqueRows(spec.records());
  const policyRef = required(policyRefs, family.id, `policy ref ${family.id}`);
  return {
    ...context.envelope({
      id: `unknowable-domain.research-gap.${family.id}`,
      kind: "ReferenceResearchGap",
      nameEn: `${family.id} Replacement Boundary`,
      nameZh: `${family.id} 替换边界`,
      summaryEn:
        "Nonblocking reference boundary preserves current exact facts and " +
        "fail-closed policy fields until stronger released evidence exists.",
      summaryZh:
        "非阻塞资料边界保留当前精确事实与失败关闭策略字段，直至出现更强的已发布证据。",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [policyRef],
      tags: ["nonblocking", "research-gap", family.id],
    }),
    source_id: family.id,
    state: "PolicyBound",
    blocking: false,
    owner: "G10-P4-B2",
    field: family.id,
    known_fact:
      `${selected.length} selected normalized record(s) cover the released ` +
      `portion of ${family.must_cover.join(", ")}.`,
    policy:
      "Preserve exact facts, label unavailable semantics, use stable order " +
      "only in review fixtures, and fail closed without mutation or hidden RNG.",
    affected_data_ids: selected.map(({ id }) => id).sort(),
    replacement_condition: policyRef.replacement_condition,
  };
}).sort((left, right) => left.source_id.localeCompare(right.source_id));
outputs.set("research-gaps.json", researchGaps);

const coverage = [];
for (const [categoryId, category] of
  Object.entries(sourceManifest.categories).sort(([left], [right]) =>
    left.localeCompare(right))) {
  for (const record of category.records) {
    const dataIds = resolveDataIds(categoryId, record);
    if (dataIds.length === 0)
      throw new Error(`no normalized row for ${categoryId}/${record.id}`);
    coverage.push({
      ...context.envelope({
        id:
          `unknowable-domain.coverage.${slug(categoryId)}.` +
          `${slug(record.id)}.${record.evidence_sha256.slice(0, 12)}`,
        kind: "ReferenceCoverage",
        nameEn: `${categoryId}/${record.id} Coverage`,
        nameZh: `${categoryId}/${record.id} 覆盖`,
        summaryEn:
          `Frozen ${categoryId} obligation ${record.id} resolves to ` +
          `${dataIds.length} DataReady normalized row(s).`,
        summaryZh:
          `冻结的 ${categoryId} 义务 ${record.id} 解析到 ` +
          `${dataIds.length} 条 DataReady 规范化记录。`,
        ownership: record.ownership,
        sourceRefs: [manifestRootRef],
        tags: ["coverage", categoryId],
      }),
      source_id: `${categoryId}:${record.id}`,
      manifest_category: categoryId,
      manifest_record_id: String(record.id),
      source_locator: record.source,
      source_evidence_sha256: record.evidence_sha256,
      state: "DataReady",
      data_ids: dataIds,
      blocking_gap_ids: [],
    });
  }
}
coverage.sort((left, right) =>
  left.manifest_category.localeCompare(right.manifest_category)
  || left.manifest_record_id.localeCompare(right.manifest_record_id));
outputs.set("coverage.json", coverage);

const receipts = reconciliationReceipts();
outputs.set("reconciliation-receipts.json", receipts);

const manifestRow = {
  ...context.envelope({
    id: "unknowable-domain.reference-manifest.v1",
    kind: "ReferenceManifestSummary",
    nameEn: "Unknowable Domain Reference Manifest",
    nameZh: "不可知域资料清单",
    summaryEn:
      `Version 4.4 Candidate reference manifest closes ` +
      `${sourceManifest.counts.records} frozen obligations across ` +
      `${sourceManifest.counts.categories} categories without runtime publication.`,
    summaryZh:
      `Version 4.4 Candidate 资料清单闭合 ` +
      `${sourceManifest.counts.categories} 个类别中的 ` +
      `${sourceManifest.counts.records} 个冻结义务，且不发布运行时内容。`,
    sourceRefs: [manifestRootRef, inventoryRootRef, schemaRootRef],
    tags: ["candidate", "manifest", "reference-only"],
  }),
  source_id: "unknowable-domain-reference-v1",
  goal_id: "unknowable-domain-reference-v1",
  profile_id: "unknowable-domain.profile.v1",
  snapshot: sourceInventory.snapshot,
  source_inventory_sha256: sha256(inventoryBytes),
  content_manifest_sha256: sha256(manifestBytes),
  normalized_schema_sha256: sha256(schemaBytes),
  structured_source_revision: SOURCE_REVISION,
  bilingual_index_revision:
    "7b349e39ee0f6f3bf814567995829b99c95e7a93",
  category_counts: Object.fromEntries(
    Object.entries(sourceManifest.categories)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([id, category]) => [id, category.count]),
  ),
  ownership_counts: sourceManifest.counts.ownership,
  frozen_source_obligations: sourceManifest.counts.records,
  data_ready_source_obligations: coverage.length,
  coverage_percent: "100",
  normalized_file_count: schema.files.length,
  mechanic_source_count: mechanicSourceRows.length,
  mechanic_rule_count: mechanicRuleRows.length,
  semantic_fixture_family_count: semanticFamilies.length,
  research_gap_count: researchGaps.length,
  blocking_research_gap_count:
    researchGaps.filter(({ blocking }) => blocking).length,
  reconciliation_receipt_count: receipts.length,
  runtime_loading: "ForbiddenReferenceOnly",
  authoring_target: "ExcelOpenPyxlThenSora030",
  candidate_quality: true,
};
outputs.set("manifest.json", [manifestRow]);

const sourceRows = sourceRegistry(collectSourceRefs());
outputs.set("sources.json", sourceRows);

const expectedPreIndexCount = schema.files.length - 1;
if (outputs.size !== expectedPreIndexCount)
  throw new Error(
    `expected ${expectedPreIndexCount} pre-index files, got ${outputs.size}`,
  );
outputs.set("pack-index.json", [packIndexRow()]);
const expectedFiles = schema.files.map(({ file }) => file).sort();
const actualFiles = [...outputs.keys()].sort();
if (canonical(expectedFiles) !== canonical(actualFiles))
  throw new Error("normalized output file set drift");

await writeOrCheck(context, outputs, check);
console.log(
  `Unknowable Domain pack ${check ? "verified" : "finalized"}: ` +
  `${mechanicSourceRows.length} mechanic sources/rules, ${sourceRows.length} ` +
  `source rows, ${coverage.length} coverage rows, ${researchGaps.length} ` +
  `nonblocking gaps, ${reviewFixtures.length} fixtures, ${receipts.length} ` +
  `Goal 08/09 receipts, ${schema.files.length} files.`,
);

async function json(relative) {
  return JSON.parse(await fs.readFile(path.join(root, relative), "utf8"));
}

function localRootRef({
  id,
  relative,
  bytes,
  revision,
  mechanismQuality,
}) {
  return {
    source_id: `source.goal10.local.${id}`,
    repository: "starclock",
    revision,
    path: relative,
    locator: "root",
    sha256: sha256(bytes),
    access_date: ACCESS_DATE,
    game_version: GAME_VERSION,
    evidence_quality: "ExactStructured",
    mechanism_quality: mechanismQuality,
  };
}

function collectOperationTypes(value) {
  const order = [];
  const counts = new Map();
  function visit(node) {
    if (Array.isArray(node)) {
      for (const child of node) visit(child);
      return;
    }
    if (!node || typeof node !== "object") return;
    if (typeof node.$type === "string") {
      if (!counts.has(node.$type)) order.push(node.$type);
      counts.set(node.$type, (counts.get(node.$type) ?? 0) + 1);
    }
    for (const child of Object.values(node)) visit(child);
  }
  visit(value);
  if (order.length === 0) {
    const structural = Array.isArray(value)
      ? ["Structure:Array"]
      : Object.keys(value).sort().map((key) => `Structure:${key}`);
    for (const key of structural) {
      order.push(key);
      counts.set(key, 1);
    }
  }
  return {
    total: [...counts.values()].reduce((sum, count) => sum + count, 0),
    types: order.map((operationType) => ({
      operation_type: operationType,
      source_occurrences: counts.get(operationType),
    })),
  };
}

function classifyMechanicSource(sourcePath) {
  if (sourcePath.includes(".layout."))
    return {
      scope: "EvidenceLayout",
      trigger: "NoRuntimeTrigger",
      family_id: "simultaneous-trigger-order",
    };
  if (sourcePath.includes("BattleEvent/"))
    return {
      scope: "Battle",
      trigger: "BattleEventLifecycle",
      family_id: "scepter-activation",
    };
  if (sourcePath.includes("ConfigAbility/Level/"))
    return {
      scope: "Battle",
      trigger: "ModeAbilityLifecycle",
      family_id: "scepter-charge-and-speed",
    };
  if (sourcePath.includes("ConfigAdventureModifier/"))
    return {
      scope: "CrossBattle",
      trigger: "AdventureLifecycle",
      family_id: "service-and-adventure",
    };
  if (sourcePath.includes("Content_Reforge"))
    return {
      scope: "Activity",
      trigger: "ReforgeServiceLifecycle",
      family_id: "component-reforge",
    };
  if (sourcePath.includes("Content_Shop"))
    return {
      scope: "Activity",
      trigger: "ShopServiceLifecycle",
      family_id: "workbench-offer-and-cost",
    };
  if (sourcePath.includes("Content_Common"))
    return {
      scope: "Activity",
      trigger: "CommonServiceLifecycle",
      family_id: "service-and-adventure",
    };
  if (sourcePath.includes("Prop_"))
    return {
      scope: "Activity",
      trigger: "RoomDoorLifecycle",
      family_id: "area-layer-room-transition",
    };
  if (sourcePath.includes("Group_Monster"))
    return {
      scope: "Activity",
      trigger: "RoomEncounterLifecycle",
      family_id: "encounter-selection",
    };
  return {
    scope: "Activity",
    trigger: "ProgressionOrRoomLifecycle",
    family_id: "talent-and-unlock",
  };
}

function scopeZh(value) {
  return new Map([
    ["Activity", "活动域"],
    ["Battle", "战斗域"],
    ["CrossBattle", "跨战斗域"],
    ["EvidenceLayout", "证据布局"],
  ]).get(value) ?? value;
}

function familySpecifications() {
  return new Map([
    ["profile-entry-and-finish", {
      records: () => [
        pick("profiles.json", ({ kind }) => kind === "UnknowableProfile"),
        pick("profiles.json", ({ kind }) => kind === "EntryPoint"),
        rows("finish-conditions.json")[0],
      ],
    }],
    ["area-layer-room-transition", {
      records: () => [
        rows("areas.json")[0],
        rows("layers.json")[0],
        rows("layer-rooms.json")[0],
        rows("rooms.json")[0],
        rows("stage-flow.json")[0],
      ],
    }],
    ["difficulty-composition", {
      records: () => [
        pick("difficulty-compositions.json",
          ({ kind }) => kind === "DifficultyComposition"),
        pick("difficulty-compositions.json",
          ({ kind }) => kind === "DifficultyDropBinding"),
      ],
    }],
    ["alignment-selection", { records: () => rows("alignments.json") }],
    ["scepter-activation", {
      records: () => [
        rows("scepters.json")[0],
        rows("scepter-activation-rules.json")[0],
        rows("scepter-state-transitions.json")[0],
      ],
    }],
    ["scepter-charge-and-speed", {
      records: () => [
        ...rows("scepters.json").filter(({ function_type: value }) =>
          ["Charge", "Speed"].includes(value)).slice(0, 2),
        ...rows("scepter-state-transitions.json").slice(0, 2),
      ],
    }],
    ["component-slot-legality", {
      records: () => [
        rows("component-levels.json")[0],
        rows("component-slot-compatibility.json")[0],
        rows("slot-layouts.json")[0],
      ],
    }],
    ["component-insertion-removal-replacement", {
      records: () => [
        rows("loadouts.json")[0],
        ...rows("loadout-transition-rules.json"),
      ],
    }],
    ["decision-component-choice", {
      records: () => [
        rows("decision-components.json")[0],
        rows("component-choice-programs.json")[0],
      ],
    }],
    ["component-synthesis", {
      records: () => rows("synthesis-rules.json"),
    }],
    ["component-upgrade", { records: () => rows("upgrade-rules.json") }],
    ["component-reforge", { records: () => rows("reforge-rules.json") }],
    ["workbench-offer-and-cost", {
      records: () => [
        rows("workbenches.json")[0],
        rows("workbench-functions.json")[0],
        rows("service-offer-rules.json")[0],
      ],
    }],
    ["gamble-offer-and-outcome", {
      records: () => [
        rows("gamble-groups.json")[0],
        rows("gamble-units.json")[0],
        pick("service-offer-rules.json",
          ({ source_id: sourceId }) => sourceId?.startsWith("gamble-group:"),
          "gamble offer rule"),
      ],
    }],
    ["talent-and-unlock", {
      records: () => [rows("talents.json")[0], rows("unlocks.json")[0]],
    }],
    ["layer-and-difficulty-effect", {
      records: () => [
        rows("layer-effects.json")[0],
        rows("progression-effects.json")[0],
        rows("maze-buffs.json")[0],
      ],
    }],
    ["curio-lifecycle", {
      records: () => [
        rows("curios.json")[0],
        rows("curio-states.json")[0],
        rows("curio-rules.json")[0],
      ],
    }],
    ["occurrence-choice", {
      records: () => [
        rows("occurrences.json")[0],
        rows("occurrence-variants.json")[0],
        rows("occurrence-choices.json")[0],
      ],
    }],
    ["service-and-adventure", {
      records: () => [
        rows("service-rules.json")[0],
        rows("adventure-outcomes.json")[0],
        rows("mode-service-npcs.json")[0],
      ],
    }],
    ["encounter-selection", {
      records: () => [
        pick("encounter-source-obligations.json",
          ({ expansion_state: state }) =>
            state === "UnresolvedNoReleasedSelector"),
        rows("boss-pools.json")[0],
      ],
    }],
    ["wave-and-boss-binding", {
      records: () => [
        rows("boss-choices.json")[0],
        rows("boss-pools.json")[0],
        pick("encounter-source-obligations.json",
          ({ parent_kind: kind }) => kind === "DisplayedBossIdentity"),
      ],
    }],
    ["cross-battle-carry-reset", {
      records: () => [
        rows("stage-flow.json")[0],
        rows("finish-conditions.json")[0],
        pick("profiles.json", ({ kind }) => kind === "UnknowableProfile"),
      ],
    }],
    ["simultaneous-trigger-order", {
      records: () => rows("scepter-state-transitions.json").slice(0, 3),
    }],
    ["no-legal-candidate-fallback", {
      records: () => [
        rows("loadout-transition-rules.json")[0],
        rows("synthesis-rules.json")[0],
        rows("service-offer-rules.json")[0],
      ],
    }],
  ]);
}

function resolveDataIds(categoryId, record) {
  const direct = (file, sourceId = record.id, predicate = () => true) =>
    rows(file).filter((row) =>
      String(row.source_id ?? "") === String(sourceId) && predicate(row))
      .map(({ id }) => id);
  const prefixed = (file, prefix) => direct(file, `${prefix}${record.id}`);
  let values;
  switch (categoryId) {
    case "profiles":
      values = rows("profiles.json")
        .filter(({ kind }) => kind === "UnknowableProfile").map(({ id }) => id);
      break;
    case "entry_points":
      values = direct("profiles.json", String(record.id).split(":").slice(1).join(":"),
        ({ kind }) => kind === "EntryPoint");
      break;
    case "areas": values = direct("areas.json"); break;
    case "difficulty_compositions":
      values = direct("difficulty-compositions.json", record.id,
        ({ kind }) => kind === "DifficultyComposition");
      break;
    case "difficulty_drops":
      values = direct("difficulty-compositions.json", record.id,
        ({ kind }) => kind === "DifficultyDropBinding");
      break;
    case "layers": values = direct("layers.json"); break;
    case "layer_rooms": values = direct("layer-rooms.json"); break;
    case "rooms": values = direct("rooms.json"); break;
    case "room_types":
      values = rows("rooms.json")
        .filter(({ room_type: type }) => type === record.id).map(({ id }) => id);
      break;
    case "finish_conditions": values = direct("finish-conditions.json"); break;
    case "alignments": values = direct("alignments.json"); break;
    case "scepters": values = direct("scepters.json"); break;
    case "scepter_levels": values = direct("scepter-levels.json"); break;
    case "scepter_locked_components":
      values = direct(
        "scepter-levels.json",
        `${record.scepter_id}:${record.scepter_level}`,
      );
      break;
    case "slot_layouts": values = direct("slot-layouts.json"); break;
    case "components":
    case "component_categories":
    case "component_types":
      values = direct("components.json");
      break;
    case "component_levels": values = direct("component-levels.json"); break;
    case "decision_components": values = direct("decision-components.json"); break;
    case "component_effects":
      values = rows("component-levels.json")
        .filter(({ effect_source_id: id }) => id === record.id)
        .map(({ id }) => id);
      break;
    case "mode_constants": values = direct("mode-constants.json"); break;
    case "layer_effects": values = direct("layer-effects.json"); break;
    case "maze_buffs": values = direct("maze-buffs.json"); break;
    case "talents": values = direct("talents.json"); break;
    case "unlocks": values = direct("unlocks.json"); break;
    case "score_inputs": values = direct("score-inputs.json"); break;
    case "workbenches": values = direct("workbenches.json"); break;
    case "workbench_functions": values = direct("workbench-functions.json"); break;
    case "gamble_groups": values = direct("gamble-groups.json"); break;
    case "gamble_units": values = direct("gamble-units.json"); break;
    case "adventure_outcomes":
      values = prefixed("adventure-outcomes.json", "adventure-outcome:");
      break;
    case "curios":
      values = rows("curios.json")
        .filter(({ handbook_id: id }) => id === record.id).map(({ id }) => id);
      break;
    case "curio_states":
      values = prefixed("curio-states.json", "curio-state:");
      break;
    case "curio_groups":
      values = prefixed("curio-groups.json", "curio-group:");
      break;
    case "occurrences":
      values = rows("occurrences.json")
        .filter(({ handbook_id: id }) => id === record.id).map(({ id }) => id);
      break;
    case "occurrence_variants":
      values = prefixed("occurrence-variants.json", "occurrence-variant:");
      break;
    case "mode_service_npcs":
      values = prefixed("mode-service-npcs.json", "mode-service-npc:");
      break;
    case "boss_choices": values = direct("boss-choices.json"); break;
    case "encounter_source_obligations":
      values = direct("encounter-source-obligations.json");
      break;
    case "mechanic_source_files":
      values = direct("mechanic-source-files.json");
      break;
    case "semantic_fixture_families":
      values = [
        ...direct("semantic-fixture-families.json"),
        ...direct("review-fixtures.json"),
      ];
      break;
    default:
      if (categoryId === "blessings" && sourceManifest.categories.blessings.count === 0)
        values = [];
      else throw new Error(`missing coverage resolver for ${categoryId}`);
  }
  return [...new Set(values)].sort();
}

function reconciliationReceipts() {
  const checkpoints = [
    checkpoint({
      goal: "Goal08",
      commit: GOAL08_CHECKPOINT,
      manifestPath:
        "content-manifests/gold-and-gears-v1/content-manifest.json",
      expectedSha256: GOAL08_MANIFEST_SHA256,
    }),
    checkpoint({
      goal: "Goal09",
      commit: GOAL09_CHECKPOINT,
      manifestPath:
        "content-manifests/swarm-disaster-v1/content-manifest.json",
      expectedSha256: GOAL09_MANIFEST_SHA256,
    }),
  ];
  const result = [];
  for (const checkpointValue of checkpoints) {
    const otherByIdentity = new Map();
    for (const [categoryId, category] of
      Object.entries(checkpointValue.value.categories))
      for (const record of category.records)
        otherByIdentity.set(`${record.source}\0${record.id}`, {
          categoryId,
          record,
        });
    for (const categoryId of ["curios", "occurrences", "boss_choices"]) {
      for (const record of sourceManifest.categories[categoryId].records) {
        const other = otherByIdentity.get(`${record.source}\0${record.id}`);
        if (!other) continue;
        if (record.evidence_sha256 !== other.record.evidence_sha256)
          throw new Error(
            `${checkpointValue.goal} evidence conflict ` +
            `${record.source}/${record.id}`,
          );
        const [sourcePath, rowLocator = String(record.id)] =
          record.source.split("#", 2);
        const sameOwnership = record.ownership === other.record.ownership;
        result.push({
          ...context.envelope({
            id:
              `unknowable-domain.reconciliation.` +
              `${checkpointValue.goal.toLowerCase()}.${slug(sourcePath)}.` +
              `${slug(rowLocator)}.${slug(record.id)}`,
            kind: "OwnershipReconciliationReceipt",
            nameEn:
              `${checkpointValue.goal} Reconciliation ${categoryId}/${record.id}`,
            nameZh:
              `${checkpointValue.goal} 对账 ${categoryId}/${record.id}`,
            summaryEn:
              `Goal 10 and ${checkpointValue.goal} use the same source path, ` +
              "row locator and evidence digest; per-pack ownership is recorded " +
              "without mutating either goal.",
            summaryZh:
              `Goal 10 与 ${checkpointValue.goal} 使用相同源路径、行定位与证据摘要；` +
              "按资料包记录归属，且不修改任一 Goal。",
            ownership: "Shared",
            sourceRefs: [manifestRootRef, checkpointValue.ref],
            tags: [
              "ownership-reconciliation",
              checkpointValue.goal.toLowerCase(),
              sameOwnership ? "matched" : "divergent-representation",
            ],
          }),
          source_id:
            `${checkpointValue.goal}:${categoryId}:${record.id}`,
          source_path: sourcePath,
          row_locator: rowLocator,
          evidence_sha256: record.evidence_sha256,
          checkpoint_goal: checkpointValue.goal,
          checkpoint_commit: checkpointValue.commit,
          checkpoint_category: other.categoryId,
          checkpoint_record_id: String(other.record.id),
          checkpoint_ownership: other.record.ownership,
          goal10_category: categoryId,
          goal10_record_id: String(record.id),
          goal10_ownership: record.ownership,
          outcome: sameOwnership ? "MatchedShared" : "DivergentRepresentation",
          note: sameOwnership
            ? "Shared source identity and ownership agree at the frozen checkpoint."
            : "Source fact agrees; ownership labels describe different per-pack reachability and remain isolated pending merge review.",
          blocking: false,
        });
      }
    }
  }
  return result.sort((left, right) =>
    left.checkpoint_goal.localeCompare(right.checkpoint_goal)
    || left.source_path.localeCompare(right.source_path)
    || left.row_locator.localeCompare(right.row_locator)
    || left.id.localeCompare(right.id));
}

function checkpoint({ goal, commit, manifestPath: relative, expectedSha256 }) {
  const object = `${commit}:${relative}`;
  const result = spawnSync("git", ["cat-file", "blob", object], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
  });
  if (result.status !== 0)
    throw new Error(`cannot read ${goal} checkpoint: ${result.stderr.trim()}`);
  if (sha256(result.stdout) !== expectedSha256)
    throw new Error(`${goal} checkpoint manifest digest drift`);
  return {
    goal,
    commit,
    value: JSON.parse(result.stdout),
    ref: {
      source_id: `source.goal10.${goal.toLowerCase()}-checkpoint`,
      repository: "starclock",
      revision: commit,
      path: relative,
      locator: "root",
      sha256: expectedSha256,
      access_date: ACCESS_DATE,
      game_version: GAME_VERSION,
      evidence_quality: "ExactStructured",
      mechanism_quality: "ReconciliationCheckpoint",
    },
  };
}

function collectSourceRefs() {
  const refs = new Map();
  for (const value of outputs.values()) {
    if (!Array.isArray(value)) continue;
    for (const row of value)
      for (const ref of row.source_refs ?? []) {
        const prior = refs.get(ref.source_id);
        if (prior && canonical(prior) !== canonical(ref))
          throw new Error(`conflicting source ref ${ref.source_id}`);
        refs.set(ref.source_id, ref);
      }
  }
  return [...refs.values()].sort((left, right) =>
    left.source_id.localeCompare(right.source_id));
}

function sourceRegistry(refs) {
  return refs.map((ref) => ({
    ...context.envelope({
      id:
        `unknowable-domain.source.${slug(ref.source_id)}.` +
        `${ref.sha256.slice(0, 12)}`,
      kind: "SourceEvidence",
      nameEn: `Source ${ref.path} ${ref.locator}`,
      nameZh: `来源 ${ref.path} ${ref.locator}`,
      summaryEn:
        `${ref.evidence_quality} source evidence at ${ref.path} ` +
        `${ref.locator}, pinned by repository revision and SHA-256.`,
      summaryZh:
        `${ref.evidence_quality} 来源证据位于 ${ref.path} ${ref.locator}，` +
        "由仓库修订与 SHA-256 固定。",
      evidenceQuality: ref.evidence_quality,
      sourceRefs: [ref],
      tags: ["source-evidence", ref.evidence_quality.toLowerCase()],
    }),
    source_id: ref.source_id,
    repository: ref.repository,
    revision: ref.revision,
    path: ref.path,
    locator: ref.locator,
    sha256: ref.sha256,
    access_date: ref.access_date,
    game_version: ref.game_version,
    mechanism_quality: ref.mechanism_quality,
    note: ref.note ?? "",
    replacement_condition: ref.replacement_condition ?? "",
  })).sort((left, right) => left.source_id.localeCompare(right.source_id));
}

function packIndexRow() {
  const fileDigests = [...outputs.entries()].map(([file, value]) => {
    const bytes = `${JSON.stringify(value, null, 2)}\n`;
    return {
      file,
      rows: Array.isArray(value) ? value.length : 1,
      bytes: Buffer.byteLength(bytes),
      sha256: sha256(bytes),
    };
  }).sort((left, right) => left.file.localeCompare(right.file));
  const packDigest = sha256(
    fileDigests.map(({ file, sha256: digest }) =>
      `${file}\0${digest}`).join("\n"),
  );
  const byFile = new Map(fileDigests.map((entry) => [entry.file, entry]));
  const componentDigests = Object.fromEntries(
    authoringContract.workbooks.map((workbook) => [
      workbook.file,
      sha256(workbook.normalized_files.filter((file) =>
        file !== "pack-index.json").sort().map((file) => {
        const entry = required(byFile, file, `pack digest ${file}`);
        return `${file}\0${entry.sha256}`;
      }).join("\n")),
    ]).sort(([left], [right]) => left.localeCompare(right)),
  );
  const componentDigest = sha256(canonical(componentDigests));
  return {
    ...context.envelope({
      id: "unknowable-domain.pack-index.v1",
      kind: "ReferencePackIndex",
      nameEn: "Unknowable Domain Canonical Pack Index",
      nameZh: "不可知域规范资料包索引",
      summaryEn:
        `${fileDigests.length} pre-index normalized files are digest-bound ` +
        "into one Candidate reference pack and three authoring components.",
      summaryZh:
        `${fileDigests.length} 个索引前规范化文件通过摘要绑定为一个 ` +
        "Candidate 资料包与三个作者组件。",
      sourceRefs: [manifestRootRef, schemaRootRef],
      tags: ["candidate", "pack-index", "reference-only"],
    }),
    source_id: "unknowable-domain-reference-v1",
    file_digests: fileDigests,
    pack_digest: packDigest,
    component_digest: componentDigest,
    component_digests: componentDigests,
    runtime_loading: "ForbiddenReferenceOnly",
  };
}

function uniqueRows(values, label = "fixture") {
  const byId = new Map();
  for (const value of values) {
    if (!value?.id) continue;
    const prior = byId.get(value.id);
    if (prior && canonical(prior) !== canonical(value))
      throw new Error(`conflicting selected row ${value.id}`);
    byId.set(value.id, value);
  }
  if (byId.size === 0) {
    throw new Error(`${label} selected no source rows`);
  }
  return [...byId.values()].sort((left, right) => left.id.localeCompare(right.id));
}

function uniqueRefs(values) {
  const byId = new Map();
  for (const value of values) {
    const prior = byId.get(value.source_id);
    if (prior && canonical(prior) !== canonical(value))
      throw new Error(`conflicting fixture source ref ${value.source_id}`);
    byId.set(value.source_id, value);
  }
  return [...byId.values()].sort((left, right) =>
    left.source_id.localeCompare(right.source_id));
}

function required(map, key, label) {
  const value = map.get(key);
  if (!value) throw new Error(`missing ${label}`);
  return value;
}
