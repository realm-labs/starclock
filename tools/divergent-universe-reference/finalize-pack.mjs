#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
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

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(valueAfter("--root")
  ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."));
const context = await createContext(root, valueAfter("--source-cache"));
const referenceRoot = path.join(root, "content-reference/divergent-universe-v1");
const manifestPath =
  "content-manifests/divergent-universe-v1/content-manifest.json";
const inventoryPath =
  "content-manifests/divergent-universe-v1/source-inventory.json";
const schemaPath =
  "content-manifests/divergent-universe-v1/normalized-schema.json";
const fixturePath =
  "content-manifests/divergent-universe-v1/fixture-contract.json";
const authoringPath =
  "content-manifests/divergent-universe-v1/authoring-contract.json";
const reconciliationPath =
  "evidence/divergent-universe-reference-v1/reconciliation-checkpoints.json";
const [
  manifest,
  inventory,
  schema,
  fixtureContract,
  authoringContract,
  reconciliationEvidence,
] =
  await Promise.all([
    localJson(manifestPath),
    localJson(inventoryPath),
    localJson(schemaPath),
    localJson(fixturePath),
    localJson(authoringPath),
    localJson(reconciliationPath),
  ]);
const manifestBytes = await fs.readFile(path.join(root, manifestPath));
const inventoryBytes = await fs.readFile(path.join(root, inventoryPath));
const schemaBytes = await fs.readFile(path.join(root, schemaPath));
const fixtureBytes = await fs.readFile(path.join(root, fixturePath));
const reconciliationBytes = await fs.readFile(
  path.join(root, reconciliationPath),
);
if (
  reconciliationEvidence.schema_revision !==
    "starclock.divergent-universe-reconciliation-checkpoints.v1" ||
  reconciliationEvidence.result !== "pass"
) {
  throw new Error("reconciliation checkpoint evidence envelope drift");
}
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
  outputs.set(contract.file, await localJson(
    `content-reference/divergent-universe-v1/${contract.file}`,
  ));
}

const manifestRef = localRootRef(
  "content-manifest",
  manifestPath,
  manifestBytes,
  schema.bound_content_manifest_sha256,
  "ManifestClosure",
);
const inventoryRef = localRootRef(
  "source-inventory",
  inventoryPath,
  inventoryBytes,
  inventory.schema_revision,
  "InventoryClosure",
);
const schemaRef = localRootRef(
  "normalized-schema",
  schemaPath,
  schemaBytes,
  schema.schema_revision,
  "SchemaContract",
);
const fixtureRef = localRootRef(
  "fixture-contract",
  fixturePath,
  fixtureBytes,
  fixtureContract.schema_revision,
  "FixtureContract",
);

const inventoryByPath = new Map(inventory.records.map((record) =>
  [record.path, record]));
const mechanicSources = [];
const mechanicRules = [];
for (const record of manifest.categories.mechanic_source_files.records) {
  const sourceValue = await readPinnedSource(record.source);
  const sourceEntry = {
    sourcePath: record.source,
    locator: "root",
    row: sourceValue,
    evidenceRow: sourceValue,
  };
  const sourceRef = context.sourceRef(sourceEntry);
  const inventoryRecord = required(
    inventoryByPath,
    record.source,
    `inventory record ${record.source}`,
  );
  const audit = operationAudit(sourceValue);
  const classification = classifyMechanic(record.source, inventoryRecord.family);
  const sourceId =
    `divergent-universe.mechanic-source.${slug(record.source)}`;
  const ruleId = `divergent-universe.mechanic-rule.${slug(record.source)}`;
  mechanicSources.push({
    ...context.envelope({
      id: sourceId,
      kind: "DivergentUniverseMechanicSourceFile",
      nameEn: `Mechanic Source ${path.basename(record.source)}`,
      nameZh: `机制源文件 ${path.basename(record.source)}`,
      summaryEn:
        `Pinned ${classification.scope} source preserves ${audit.types.length} structured operation or shape type(s) for audit without runtime lowering.`,
      summaryZh:
        `固定的${classification.scope}源文件保留 ${audit.types.length} 个结构化操作或形状类型，用于审计且不进行运行时 lowering。`,
      sourceRefs: [sourceRef],
      tags: ["mechanic-source", classification.family, classification.scope],
    }),
    source_id: record.id,
    source_path: record.source,
    source_sha256: record.evidence_sha256,
    mechanic_family: classification.family,
    scope: classification.scope,
    operation_types: audit.types,
    operation_occurrence_count: audit.total,
    disposition: "ReferenceOnlyNotLowered",
    consumer_rule_ids: [ruleId],
    runtime_lowered: false,
  });
  mechanicRules.push({
    ...context.envelope({
      id: ruleId,
      kind: "DivergentUniverseMechanicRule",
      nameEn: `${path.basename(record.source)} Mechanic Boundary`,
      nameZh: `${path.basename(record.source)} 机制边界`,
      summaryEn:
        `Reference-only ${classification.scope} rule preserves ${audit.types.length} ordered operation or shape type(s); it is not executable behavior.`,
      summaryZh:
        `仅供资料使用的${classification.scope}规则保留 ${audit.types.length} 个有序操作或形状类型；它不是可执行行为。`,
      sourceRefs: [sourceRef],
      tags: ["mechanic-rule", classification.family, classification.scope],
    }),
    source_id: record.id,
    source_file_id: sourceId,
    scope: classification.scope,
    trigger: classification.trigger,
    ordered_operations: audit.types.map((item, index) => ({
      ordinal: index + 1,
      operation_type: item.operation_type,
      source_occurrences: item.source_occurrences,
    })),
    state_lifecycle: classification.lifecycle,
    fixture_ids: [
      `divergent-universe.review-fixture.${classification.fixtureFamily}`,
    ],
    runtime_lowered: false,
  });
}
mechanicSources.sort(compareSourceId);
mechanicRules.sort(compareSourceId);
outputs.set("mechanic-source-files.json", mechanicSources);
outputs.set("mechanic-rules.json", mechanicRules);

const familySelections = fixtureSelections();
const familyPolicyRefs = new Map();
for (const family of fixtureContract.required_families)
  familyPolicyRefs.set(family.id, await context.policyRef(
    `semantic-fixture-${family.id}`,
    `Reference review covers ${family.must_cover.join(", ")} using final ` +
      "normalized facts and explicit unavailable/policy boundaries. It does " +
      "not claim runtime execution.",
    `Replace policy-bound assertions for ${family.id} only when released ` +
      "structured evidence or a reproducible public observation supplies the " +
      "missing selector, magnitude, order, timing, or lifecycle.",
  ));

const semanticFamilies = [];
const reviewFixtures = [];
const researchGaps = [];
for (const family of fixtureContract.required_families) {
  const selected = selectRows(required(
    familySelections,
    family.id,
    `fixture selection ${family.id}`,
  ));
  const policyRef = required(
    familyPolicyRefs,
    family.id,
    `fixture policy ${family.id}`,
  );
  const refs = uniqueRefs([
    ...selected.flatMap((row) => row.source_refs ?? []),
    policyRef,
  ]);
  semanticFamilies.push({
    ...context.envelope({
      id: `divergent-universe.semantic-family.${family.id}`,
      kind: "DivergentUniverseSemanticFixtureFamily",
      nameEn: `${family.id} Semantic Fixture Family`,
      nameZh: `${family.id} 语义夹具族`,
      summaryEn:
        `Non-shrinking reference family covers ${family.must_cover.length} required semantic fact(s).`,
      summaryZh:
        `不可缩减的资料族覆盖 ${family.must_cover.length} 个必需语义事实。`,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [fixtureRef, policyRef],
      tags: ["semantic-family", family.id],
    }),
    source_id: family.id,
    minimum_cases: family.minimum_cases,
    must_cover: family.must_cover,
    selected_source_record_ids: selected.map(({ id }) => id),
    runtime_executable: false,
  });
  reviewFixtures.push({
    ...context.envelope({
      id: `divergent-universe.review-fixture.${family.id}`,
      kind: "DivergentUniverseReviewFixture",
      nameEn: `${family.id} Reference Review`,
      nameZh: `${family.id} 资料审查`,
      summaryEn:
        `Reference-only fixture asserts all ${family.must_cover.length} contract facts as exact or explicitly policy-bound.`,
      summaryZh:
        `仅供资料使用的夹具将全部 ${family.must_cover.length} 个契约事实断言为精确事实或显式策略边界。`,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: refs,
      tags: ["review-fixture", family.id],
    }),
    source_id: family.id,
    family_id: family.id,
    source_record_ids: selected.map(({ id }) => id),
    preconditions: {
      content_lane: "CandidateReference",
      runtime_loading: "Forbidden",
      source_record_count: selected.length,
    },
    input: {
      kind: "SemanticReferenceReview",
      ordering: "StableIdAscendingUnlessExactAuthoredOrder",
      unavailable_behavior: "FailClosedWithoutMutation",
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
    evidence_quality: "ProjectPolicy",
    runtime_executable: false,
  });
  researchGaps.push({
    ...context.envelope({
      id: `divergent-universe.research-gap.${family.id}`,
      kind: "DivergentUniverseResearchGap",
      nameEn: `${family.id} Replacement Boundary`,
      nameZh: `${family.id} 替换边界`,
      summaryEn:
        "Nonblocking boundary preserves exact released facts and explicit fail-closed fields until stronger released evidence exists.",
      summaryZh:
        "非阻塞边界保留精确已发布事实与显式失败关闭字段，直至出现更强的已发布证据。",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [policyRef],
      tags: ["nonblocking", "research-gap", family.id],
    }),
    source_id: family.id,
    state: "PolicyBound",
    blocking: false,
    owner: "G11-P4-B2",
    field: family.id,
    known_facts: [
      `${selected.length} normalized row(s) preserve the released portion.`,
      ...family.must_cover.map((fact) => `Required review fact: ${fact}`),
    ],
    selected_policy:
      "Preserve exact facts and explicit unavailable fields; reject or leave " +
      "authoritative state unchanged when a legal candidate cannot be proven.",
    alternatives: [
      "Infer from table prefix, ID range, adjacency, matching name, or display-only linkage.",
      "Borrow an older module or another mode's selector.",
    ],
    affected_data_ids: selected.map(({ id }) => id),
    replacement_condition: policyRef.replacement_condition,
  });
}
semanticFamilies.sort(compareSourceId);
reviewFixtures.sort((left, right) =>
  left.family_id.localeCompare(right.family_id));
researchGaps.sort(compareSourceId);
outputs.set("semantic-fixture-families.json", semanticFamilies);
outputs.set("review-fixtures.json", reviewFixtures);
outputs.set("research-gaps.json", researchGaps);

const categoryFiles = new Map();
for (const contract of schema.files)
  for (const category of contract.manifest_category_inputs) {
    const files = categoryFiles.get(category) ?? [];
    files.push(contract.file);
    categoryFiles.set(category, files);
  }
const coverage = [];
for (const [categoryId, category] of Object.entries(manifest.categories)
  .sort(([left], [right]) => left.localeCompare(right)))
  for (const record of category.records) {
    const matched = coverageRows(categoryId, record, categoryFiles);
    if (matched.length === 0)
      throw new Error(`no coverage row for ${categoryId}/${record.id}`);
    const policyBound = matched.some((row) =>
      row.coverage_state !== "DataReady"
        || row.evidence_quality === "ProjectPolicy");
    coverage.push({
      ...context.envelope({
        id:
          `divergent-universe.coverage.${slug(categoryId)}.` +
          `${slug(record.id)}.${record.evidence_sha256.slice(0, 12)}`,
        kind: "DivergentUniverseCoverage",
        nameEn: `${categoryId}/${record.id} Coverage`,
        nameZh: `${categoryId}/${record.id} 覆盖`,
        summaryEn:
          `Frozen ${categoryId} obligation ${record.id} has a final ${policyBound ? "policy-bound" : "exact"} normalized disposition.`,
        summaryZh:
          `冻结的 ${categoryId} 义务 ${record.id} 已获得最终${policyBound ? "策略边界" : "精确"}规范化处置。`,
        ownership: record.ownership === "Shared" ? "Shared" : "DivergentUniverse",
        sourceRefs: [manifestRef],
        tags: ["coverage", categoryId, policyBound ? "policy-bound" : "exact"],
      }),
      source_id: `${categoryId}:${record.id}`,
      manifest_category: categoryId,
      manifest_record_id: String(record.id),
      source_locator: record.source,
      source_evidence_sha256: record.evidence_sha256,
      normalized_record_ids: matched.map(({ id }) => id),
      state: "DataReady",
      disposition: policyBound
        ? "NormalizedPolicyOrExclusionBoundary"
        : "NormalizedExact",
      blocking_gap_ids: [],
    });
  }
coverage.sort((left, right) =>
  left.manifest_category.localeCompare(right.manifest_category)
    || left.manifest_record_id.localeCompare(right.manifest_record_id));
outputs.set("coverage.json", coverage);
const reconciliationRef = localRootRef(
  "reconciliation-checkpoints",
  reconciliationPath,
  reconciliationBytes,
  reconciliationEvidence.schema_revision,
  "ReconciliationCheckpoint",
);
outputs.set("reconciliation-receipts.json", reconciliationReceipts(
  reconciliationRef,
));

const rootRefs = [
  manifestRef,
  inventoryRef,
  schemaRef,
  fixtureRef,
  reconciliationRef,
];
const sourceRows = sourceRegistry(uniqueRefs([
  ...collectSourceRefs(),
  ...rootRefs,
]));
outputs.set("sources.json", sourceRows);

const recordCounts = Object.fromEntries(schema.files.map(({ file }) => [
  file,
  file === "manifest.json" || file === "pack-index.json"
    ? 1
    : (outputs.get(file)?.length ?? 0),
]));
const normalizedFiles = schema.files.map(({ file }) => file).sort();
const manifestRow = {
  ...context.envelope({
    id: "divergent-universe.reference-manifest.v1",
    kind: "DivergentUniverseReferenceManifest",
    nameEn: "Divergent Universe Version 4.4 Reference Manifest",
    nameZh: "差分宇宙 4.4 版本资料清单",
    summaryEn:
      `Candidate manifest accounts exactly once for ${manifest.counts.records} frozen obligations across ${manifest.counts.categories} categories without runtime publication.`,
    summaryZh:
      `Candidate 清单对 ${manifest.counts.categories} 个类别中的 ${manifest.counts.records} 个冻结义务完成逐一对账，且不发布运行时内容。`,
    sourceRefs: rootRefs,
    tags: ["candidate", "manifest", "reference-only"],
  }),
  source_id: "divergent-universe-reference-v1",
  goal_id: "divergent-universe-reference-v1",
  content_manifest_sha256: sha256(manifestBytes),
  source_inventory_sha256: sha256(inventoryBytes),
  normalized_schema_sha256: sha256(schemaBytes),
  structured_source_revision: SOURCE_REVISION,
  bilingual_index_revision:
    "7b349e39ee0f6f3bf814567995829b99c95e7a93",
  frozen_source_obligations: manifest.counts.records,
  data_ready_source_obligations: coverage.length,
  coverage_percent: "100",
  normalized_files: normalizedFiles,
  record_counts: recordCounts,
  mechanic_source_count: mechanicSources.length,
  mechanic_rule_count: mechanicRules.length,
  source_evidence_count: sourceRows.length,
  semantic_fixture_family_count: semanticFamilies.length,
  reconciliation_receipt_count:
    outputs.get("reconciliation-receipts.json").length,
  nonblocking_research_gap_count: researchGaps.length,
  blocking_research_gap_count: 0,
  runtime_loading: "ForbiddenReferenceOnly",
  authoring_target: "ExcelOpenPyxlThenSora030",
  candidate_quality: true,
};
outputs.set("manifest.json", [manifestRow]);

if (outputs.size !== schema.files.length - 1)
  throw new Error(
    `expected ${schema.files.length - 1} pre-index files, got ${outputs.size}`,
  );
outputs.set("pack-index.json", [packIndexRow()]);
const actualFiles = [...outputs.keys()].sort();
if (canonical(actualFiles) !== canonical(normalizedFiles))
  throw new Error("normalized output file set drift");

await writeOrCheck(context, outputs, check);
console.log(
  `Divergent Universe pack ${check ? "verified" : "finalized"}: ` +
  `${mechanicSources.length} mechanic sources/rules; ${sourceRows.length} ` +
  `sources; ${coverage.length}/${manifest.counts.records} DataReady coverage; ` +
  `${semanticFamilies.length} fixtures/gaps; ${schema.files.length} files.`,
);

function reconciliationReceipts(evidenceRef) {
  const localRefs = new Map(
    collectSourceRefs().map((ref) => [
      `${ref.path}\0${ref.locator}\0${ref.sha256}`,
      ref,
    ]),
  );
  const receipts = [];
  for (const checkpoint of reconciliationEvidence.checkpoints) {
    const checkpointRef = {
      source_id:
        `source.goal11.${checkpoint.goal.toLowerCase()}-reconciliation-checkpoint`,
      repository: "starclock",
      revision: checkpoint.commit,
      path: checkpoint.sources_path,
      locator: "root",
      sha256: checkpoint.sources_sha256,
      access_date: ACCESS_DATE,
      game_version: GAME_VERSION,
      evidence_quality: "ExactStructured",
      mechanism_quality: "ReconciliationCheckpoint",
    };
    for (const match of checkpoint.exact_matches) {
      const key =
        `${match.source_path}\0${match.row_locator}\0${match.evidence_sha256}`;
      const localRef = localRefs.get(key);
      if (!localRef) {
        throw new Error(
          `${checkpoint.goal}: exact reconciliation source ${key} is missing`,
        );
      }
      receipts.push({
        ...context.envelope({
          id:
            `divergent-universe.reconciliation.` +
            `${checkpoint.goal.toLowerCase()}.${slug(match.source_path)}.` +
            `${slug(match.row_locator)}.${match.evidence_sha256.slice(0, 12)}`,
          kind: "DivergentUniverseOwnershipReconciliationReceipt",
          nameEn:
            `${checkpoint.goal} Shared Source ${match.row_locator}`,
          nameZh:
            `${checkpoint.goal} 共享来源 ${match.row_locator}`,
          summaryEn:
            `Goal 11 and ${checkpoint.goal} use the exact same source path, ` +
            "row locator and evidence digest; per-pack content ownership " +
            "remains independent.",
          summaryZh:
            `Goal 11 与 ${checkpoint.goal} 使用完全相同的源路径、行定位和证据摘要；` +
            "各资料包的内容归属仍独立记录。",
          ownership: "Shared",
          sourceRefs: [localRef, checkpointRef, evidenceRef],
          tags: [
            checkpoint.goal.toLowerCase(),
            "matched",
            "ownership-reconciliation",
          ],
        }),
        source_id:
          `${checkpoint.goal}:${match.source_path}:${match.row_locator}`,
        source_path: match.source_path,
        row_locator: match.row_locator,
        evidence_sha256: match.evidence_sha256,
        checkpoint: `${checkpoint.goal}@${checkpoint.commit}`,
        checkpoint_goal: checkpoint.goal,
        checkpoint_commit: checkpoint.commit,
        checkpoint_source_id: match.checkpoint_source_id,
        checkpoint_ownership: "SharedSourceEvidence",
        goal11_source_id: match.goal11_source_id,
        goal11_ownership: "SharedSourceEvidence",
        outcome: "MatchedShared",
        note:
          "The factual source identity agrees exactly. Mode-specific rows " +
          "that cite it retain their own independently proven reachability.",
        blocking: false,
      });
    }
  }
  receipts.sort((left, right) =>
    left.checkpoint_goal.localeCompare(right.checkpoint_goal) ||
    left.source_path.localeCompare(right.source_path) ||
    left.row_locator.localeCompare(right.row_locator) ||
    left.id.localeCompare(right.id)
  );
  if (
    receipts.length !==
    reconciliationEvidence.summary.exact_shared_source_records
  ) {
    throw new Error("reconciliation receipt denominator drift");
  }
  return receipts;
}

async function localJson(relative) {
  return JSON.parse(await fs.readFile(path.join(root, relative), "utf8"));
}

async function readPinnedSource(relative) {
  try {
    return await context.readSource(relative);
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
    const raw = execFileSync(
      "git",
      ["show", `${SOURCE_REVISION}:${relative}`],
      {
        cwd: context.sourceRoot,
        encoding: "utf8",
        maxBuffer: 128 * 1024 * 1024,
      },
    );
    return JSON.parse(
      raw.replace(/("Hash"\s*:\s*)(-?\d{16,})/gu, '$1"$2"'),
    );
  }
}

function localRootRef(id, relative, bytes, revision, mechanismQuality) {
  return {
    source_id: `source.goal11.local.${id}`,
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

function operationAudit(value) {
  const order = [];
  const counts = new Map();
  function add(type) {
    if (!counts.has(type)) order.push(type);
    counts.set(type, (counts.get(type) ?? 0) + 1);
  }
  function visit(node) {
    if (Array.isArray(node)) {
      for (const child of node) visit(child);
      return;
    }
    if (!node || typeof node !== "object") return;
    for (const field of ["$type", "Type", "type", "ActionType"])
      if (typeof node[field] === "string") add(node[field]);
    for (const child of Object.values(node)) visit(child);
  }
  visit(value);
  if (order.length === 0) {
    const shapes = Array.isArray(value)
      ? ["Structure:Array"]
      : Object.keys(value).sort().map((key) => `Structure:${key}`);
    for (const shape of shapes.length > 0 ? shapes : ["Structure:EmptyObject"])
      add(shape);
  }
  return {
    total: [...counts.values()].reduce((sum, count) => sum + count, 0),
    types: order.map((operationType) => ({
      operation_type: operationType,
      source_occurrences: counts.get(operationType),
    })),
  };
}

function classifyMechanic(sourcePath, inventoryFamily) {
  if (sourcePath.includes(".layout."))
    return classification(
      "EvidenceLayout",
      "NoRuntimeTrigger",
      "SourceLayoutOnly",
      "simultaneous-trigger-order",
      inventoryFamily,
    );
  if (/ConfigAbility|BattleEvent/iu.test(sourcePath))
    return classification(
      "Battle",
      "BattleOrAbilityLifecycle",
      "BattleScopedSourceProgram",
      "battle-visible-and-cross-battle-contribution",
      inventoryFamily,
    );
  if (/Adventure|RogueAdventure/iu.test(sourcePath))
    return classification(
      "CrossBattle",
      "AcceptedExternalAdventureResult",
      "ExternalOutcomeSettlement",
      "adventure-abstract-outcome",
      inventoryFamily,
    );
  if (/NPC|Dialogue|Occurrence|Event/iu.test(sourcePath))
    return classification(
      "Activity",
      "AcceptedModeDecision",
      "CrossBattleDecisionLifecycle",
      "occurrence-choice-cost-and-outcome",
      inventoryFamily,
    );
  return classification(
    "Activity",
    "ModeOrRoomLifecycle",
    "CrossBattleStateLifecycle",
    "weekly-modifier-and-room-service",
    inventoryFamily,
  );
}

function classification(scope, trigger, lifecycle, fixtureFamily, family) {
  return { scope, trigger, lifecycle, fixtureFamily, family };
}

function fixtureSelections() {
  return new Map([
    ["adventure-abstract-outcome", ["adventure-outcomes.json"]],
    ["area-difficulty-layer-transition",
      ["areas.json", "difficulties.json", "layers.json", "stage-flow.json"]],
    ["arithmetic-mapping-eligibility",
      ["arithmetic-mapping-eligibility.json"]],
    ["arithmetic-mapping-refresh-and-teardown",
      ["arithmetic-mapping-builds.json", "arithmetic-mapping-rules.json"]],
    ["astronomical-division", ["astronomical-divisions.json"]],
    ["battle-visible-and-cross-battle-contribution",
      ["protocols.json", "curio-states.json", "titan-contributions.json"]],
    ["curio-weight-charge-destruction-repair",
      ["curios.json", "curio-states.json", "curio-lifecycle-rules.json"]],
    ["divergent-blessing-level-and-transform",
      ["blessings.json", "blessing-levels.json",
        "blessing-rewrite-rules.json",
        "blessing-equation-contributions.json"]],
    ["encounter-wave-and-boss-binding",
      ["encounter-groups.json", "encounter-waves.json",
        "enemy-slots.json", "boss-pools.json"]],
    ["equation-offer-recipe-progress-expansion",
      ["equations.json", "equation-offers.json", "equation-recipes.json",
        "equation-progress.json", "equation-expansion-states.json"]],
    ["equation-replacement-and-contribution",
      ["equation-replacement-rules.json",
        "blessing-equation-contributions.json"]],
    ["finish-and-cross-battle-reset",
      ["finish-conditions.json", "stage-flow.json"]],
    ["gamble-offer-outcome-and-fallback",
      ["gamble-groups.json", "gamble-units.json", "service-offer-rules.json"]],
    ["golden-blood-titan-choice-and-level",
      ["titan-types.json", "titan-boons.json", "titan-choices.json",
        "titan-contributions.json"]],
    ["grand-miracle-eligibility-and-lifecycle",
      ["grand-miracles.json", "grand-miracle-eligibility.json",
        "grand-miracle-states.json"]],
    ["no-legal-candidate-fallback",
      ["service-rules.json", "service-offer-rules.json",
        "equation-replacement-rules.json"]],
    ["occurrence-choice-cost-and-outcome",
      ["occurrences.json", "occurrence-variants.json"]],
    ["ordinary-and-cyclical-entry",
      ["profiles.json", "cyclical-challenges.json", "weekly-modifiers.json"]],
    ["permanent-talent-and-unlock",
      ["permanent-talents.json", "unlocks.json", "progression-effects.json"]],
    ["profile-and-module-selection",
      ["profiles.json", "modules.json", "entries.json"]],
    ["simultaneous-trigger-order",
      ["curio-lifecycle-rules.json", "titan-contributions.json"]],
    ["star-pioneer-practice-and-cognoculi",
      ["star-pioneer-practice.json", "cognoculi.json"]],
    ["threshold-protocol", ["protocols.json"]],
    ["weekly-modifier-and-room-service",
      ["weekly-modifiers.json", "room-marks.json", "mode-service-npcs.json"]],
    ["workbench-operation-and-price",
      ["workbenches.json", "workbench-functions.json",
        "currencies.json", "service-rules.json"]],
  ]);
}

function selectRows(files) {
  const selected = [];
  for (const file of files) {
    const values = outputs.get(file);
    if (!Array.isArray(values)) throw new Error(`missing fixture file ${file}`);
    if (values.length > 0) selected.push(values[0]);
  }
  const byId = new Map(selected.map((row) => [row.id, row]));
  if (byId.size === 0) throw new Error(`fixture selected no rows: ${files}`);
  return [...byId.values()].sort((left, right) =>
    left.id.localeCompare(right.id));
}

function coverageRows(categoryId, record, categoryFiles) {
  const files = required(categoryFiles, categoryId, `category files ${categoryId}`);
  const candidates = files.flatMap((file) => outputs.get(file) ?? []);
  let matched = candidates.filter((row) =>
    String(row.source_id ?? "") === String(record.id)
      || sourceMatches(row.source_refs ?? [], record.source));
  if (categoryId === "profiles")
    matched = outputs.get("profiles.json");
  if (categoryId === "area_groups")
    matched = outputs.get("areas.json");
  if (categoryId === "arithmetic_mapping_avatars")
    matched = [
      ...outputs.get("arithmetic-mapping-eligibility.json"),
      ...outputs.get("arithmetic-mapping-builds.json"),
    ]
      .filter(({ avatar_id: id }) => String(id) === String(record.id));
  return [...new Map(matched.map((row) => [row.id, row])).values()]
    .sort((left, right) => left.id.localeCompare(right.id));
}

function sourceMatches(refs, manifestSource) {
  const index = manifestSource.lastIndexOf("#");
  const sourcePath = index < 0
    ? manifestSource
    : manifestSource.slice(0, index);
  const locator = index < 0 ? "root" : manifestSource.slice(index + 1);
  return refs.some((ref) =>
    ref.path === sourcePath && (ref.locator === locator || locator === "root"));
}

function collectSourceRefs() {
  return [...outputs.values()].flatMap((rows) =>
    Array.isArray(rows)
      ? rows.flatMap((row) => row.source_refs ?? [])
      : []);
}

function uniqueRefs(refs) {
  const byId = new Map();
  for (const ref of refs) {
    const prior = byId.get(ref.source_id);
    if (prior && canonical(prior) !== canonical(ref))
      throw new Error(`conflicting source ref ${ref.source_id}`);
    byId.set(ref.source_id, ref);
  }
  return [...byId.values()].sort((left, right) =>
    left.source_id.localeCompare(right.source_id));
}

function sourceRegistry(refs) {
  return refs.map((ref) => ({
    ...context.envelope({
      id:
        `divergent-universe.source.${slug(ref.source_id)}.` +
        `${ref.sha256.slice(0, 12)}`,
      kind: "DivergentUniverseSource",
      nameEn: `Source ${ref.path} ${ref.locator}`,
      nameZh: `来源 ${ref.path} ${ref.locator}`,
      summaryEn:
        `${ref.evidence_quality} evidence at ${ref.path} ${ref.locator}, pinned by revision and SHA-256.`,
      summaryZh:
        `${ref.evidence_quality} 证据位于 ${ref.path} ${ref.locator}，由修订与 SHA-256 固定。`,
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
  })).sort(compareSourceId);
}

function packIndexRow() {
  const fileDigests = [...outputs.entries()].map(([file, value]) => {
    const bytes = `${JSON.stringify(value, null, 2)}\n`;
    return {
      file,
      rows: value.length,
      bytes: Buffer.byteLength(bytes),
      sha256: sha256(bytes),
    };
  }).sort((left, right) => left.file.localeCompare(right.file));
  const packDigest = sha256(fileDigests.map(({ file, sha256: digest }) =>
    `${file}\0${digest}`).join("\n"));
  const stableIdIndex = [...outputs.entries()].flatMap(([file, rows]) =>
    rows.map(({ id }) => ({ id, file }))).sort((left, right) =>
    left.id.localeCompare(right.id) || left.file.localeCompare(right.file));
  const digestByFile = new Map(fileDigests.map((entry) => [entry.file, entry]));
  const componentDigests = Object.fromEntries(
    authoringContract.workbooks.map((workbook) => [
      workbook.file,
      sha256(workbook.normalized_files
        .filter((file) => file !== "pack-index.json")
        .sort()
        .map((file) => {
          const entry = required(digestByFile, file, `file digest ${file}`);
          return `${file}\0${entry.sha256}`;
        }).join("\n")),
    ]).sort(([left], [right]) => left.localeCompare(right)),
  );
  return {
    ...context.envelope({
      id: "divergent-universe.pack-index.v1",
      kind: "DivergentUniversePackIndex",
      nameEn: "Divergent Universe Canonical Pack Index",
      nameZh: "差分宇宙规范资料包索引",
      summaryEn:
        `${fileDigests.length} pre-index files are digest-bound into one Candidate reference pack.`,
      summaryZh:
        `${fileDigests.length} 个索引前文件通过摘要绑定为一个 Candidate 资料包。`,
      sourceRefs: [manifestRef, schemaRef],
      tags: ["candidate", "pack-index", "reference-only"],
    }),
    source_id: "divergent-universe-reference-v1",
    pack_digest: packDigest,
    file_digests: fileDigests,
    stable_id_index: stableIdIndex,
    component_digests: componentDigests,
    runtime_loading: "ForbiddenReferenceOnly",
  };
}

function required(map, key, label) {
  const value = map.get(key);
  if (value === undefined) throw new Error(`missing ${label}`);
  return value;
}

function compareSourceId(left, right) {
  return String(left.source_id).localeCompare(String(right.source_id));
}

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}
