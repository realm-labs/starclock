#!/usr/bin/env node

import { createHash } from "node:crypto";
import { readdir, readFile, stat, writeFile } from "node:fs/promises";
import path from "node:path";

const root = path.resolve(".");
const check = process.argv.includes("--check");
const packRoot = path.join(
  root,
  "content-reference/anomaly-arbitration-v1",
);
const manifestPath = path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/content-manifest.json",
);
const schemaPath = path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/normalized-schema.json",
);
const fixturePath = path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/fixture-contract.json",
);
const inventoryPath = path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/source-inventory.json",
);
const manifestBytes = await readFile(manifestPath);
const schemaBytes = await readFile(schemaPath);
const fixtureBytes = await readFile(fixturePath);
const inventoryBytes = await readFile(inventoryPath);
const manifest = JSON.parse(manifestBytes);
const schema = JSON.parse(schemaBytes);
const fixtureContract = JSON.parse(fixtureBytes);
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function digest(value) {
  const bytes = Buffer.isBuffer(value) ? value : Buffer.from(value);
  return createHash("sha256").update(bytes).digest("hex");
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function manifestRecord(category, id) {
  const record = manifest.categories[category].records.find(
    (candidate) => candidate.id === id,
  );
  assert(record !== undefined, `missing manifest record ${category}:${id}`);
  return record;
}

function manifestRef(category, id, note, mechanismQuality) {
  const record = manifestRecord(category, id);
  const isRepositorySource = !record.source_path.startsWith("docs/");
  return {
    source_id: isRepositorySource
      ? `turnbasedgamedata:${record.source_path}:${record.row_locator}`
      : `starclock:${record.source_path}:${record.row_locator}`,
    repository_or_url: isRepositorySource
      ? "https://gitlab.com/Dimbreath/turnbasedgamedata.git"
      : "https://github.com/realm-labs/starclock.git",
    revision_or_access_date: isRepositorySource ? revision : "2026-07-29",
    game_version: "4.4",
    path_or_page: record.source_path,
    locator: record.row_locator,
    sha256: record.evidence_sha256,
    evidence_quality: record.evidence_quality,
    mechanism_quality: mechanismQuality ?? (
      record.evidence_quality === "ProjectPolicy"
        ? "PolicyBoundary"
        : "ExactRelationship"
    ),
    note,
  };
}

function envelope({
  id,
  kind,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  ownership = "AnomalyArbitration",
  manifestIds,
  sources,
  tags,
  fields,
  evidenceQuality = "ProjectPolicy",
  mechanismQuality = "PolicyBoundary",
}) {
  return {
    id,
    schema_revision: "starclock.anomaly-arbitration-row.v1",
    kind,
    name_en: nameEn,
    name_zh_cn: nameZh,
    summary_en: summaryEn,
    summary_zh_cn: summaryZh,
    ownership,
    coverage_state: "DataReady",
    evidence_quality: evidenceQuality,
    mechanism_quality: mechanismQuality,
    manifest_record_ids: [...manifestIds].sort(compareText),
    source_refs: sources,
    tags: [...tags].sort(compareText),
    ...fields,
    runtime_executable: false,
  };
}

function file(name, kind, records) {
  return {
    schema_revision: "starclock.anomaly-arbitration-normalized-file.v1",
    goal_id: "anomaly-arbitration-reference-v1",
    profile: "anomaly-arbitration-v1",
    file: name,
    record_kind: kind,
    records,
  };
}

const metaFiles = new Set([
  "mechanic-rules.json",
  "sources.json",
  "reconciliation.json",
  "coverage.json",
  "research-gaps.json",
  "review-fixtures.json",
  "manifest.json",
  "pack-index.json",
]);
const contentDocuments = [];
for (const name of (await readdir(packRoot)).sort(compareText)) {
  if (!name.endsWith(".json") || metaFiles.has(name)) continue;
  const document = JSON.parse(await readFile(path.join(packRoot, name)));
  assert(Array.isArray(document.records), `${name} lacks normalized records`);
  contentDocuments.push(document);
}
assert(contentDocuments.length === 29,
  `normalized content-file count drift: ${contentDocuments.length}`);
const contentRecords = contentDocuments.flatMap(({ records }) => records);
const recordById = new Map(contentRecords.map((record) =>
  [record.id, record]));

const familySourceRecords = {
  "battle-event-countdown": [
    "battle-event.30502", "battle-event.30503", "battle-event.30504",
    "clock.warning-threshold", "clock.expiry-and-failure",
  ],
  "best-progress-aggregation": [
    "aggregation.current-stage-progress",
    "aggregation.simultaneous-three-knight-best",
    "aggregation.retained-historical-best",
  ],
  "clock-first-cycle": [
    "clock.first-cycle-action-value", "clock.knight-cycle-limit",
    "clock.normal-king-cycle-limit", "clock.plight-cycle-limit",
  ],
  "clock-warning-expiry": [
    "clock.warning-threshold", "clock.low-cycle-combat-effect",
    "clock.expiry-and-failure",
  ],
  "empty-pool-proof": [
    "pool-audit.blessings", "pool-audit.curios",
    "pool-audit.occurrences", "pool-audit.gameplay-services",
    "pool-audit.currencies", "pool-audit.random-content-pools",
  ],
  "encounter-enemy-closure": [
    "encounter.30508011", "encounter.30508012", "encounter.30508013",
    "encounter.30508021", "encounter.30508022",
  ],
  "king-protection": [
    "king-protection.composition",
    "king-protection.knight-clear-contribution",
    "king-protection.reset-and-teardown",
  ],
  "loadout-record": [
    "loadout-record.successful-clear-snapshot",
    "loadout-record.cross-team-equipment-invalidation",
  ],
  "plight-shortcut": [
    "king-state.normal", "king-state.plight",
    "king-protection.direct-plight-shortcut",
  ],
  "profile-entry": [
    "anomaly-arbitration-v1", "period.8",
    "outcome.knight-stage-clear", "outcome.stage-attempt-failure",
  ],
  "quadrant-contribution": [
    "quadrant-option.3033066", "quadrant-option.3033067",
    "quadrant-option.3033068", "quadrant-selection.attempt-teardown",
  ],
  "quadrant-selection": [
    "quadrant-selection.active-period",
    "quadrant-selection.no-selection",
    "quadrant-selection.attempt-teardown",
  ],
  "record-replacement-reset": [
    "progress-record.rechallenge-eligibility",
    "progress-record.record-replacement-choice",
    "progress-record.record-erasure-on-reset",
  ],
  "stage-order": [
    "stage.knight-1", "stage.knight-2", "stage.knight-3",
    "stage.king-normal", "stage.king-plight",
  ],
  "target-evaluation": [
    "objective.evaluate-cycle-targets",
    "objective.evaluate-no-downed",
    "objective.stage-star-evaluation",
    "aggregation.king-medal-rating",
  ],
  "team-uniqueness": [
    "participant-policy.character-and-combat-form-uniqueness",
    "participant-policy.light-cone-instance-uniqueness",
    "participant-policy.relic-instance-uniqueness",
  ],
  "trait-contribution": [
    "trait.3033023", "trait.3033051", "trait.3033052",
    "trait.3033069", "trait.3033070",
  ],
  "wave-carry": [
    "encounter-wave.30508011.1", "encounter-wave.30508011.2",
    "clock.wave-transition-carry", "clock.warning-threshold",
  ],
};
for (const [familyId, ids] of Object.entries(familySourceRecords))
  for (const id of ids)
    assert(recordById.has(id), `${familyId} references missing record ${id}`);

const ruleRows = fixtureContract.required_families.map((family) => {
  const sourceRecords = familySourceRecords[family.id];
  const policySource = manifestRef(
    "semantic_fixture_families",
    family.id,
    "Goal 13 freezes this non-shrinking semantic review family.",
  );
  const factualSources = sourceRecords.flatMap(
    (id) => recordById.get(id).source_refs.slice(0, 1),
  );
  return envelope({
    id: `mechanic-rule.${family.id}`,
    kind: "MechanicRule",
    nameEn: `${family.id} review rule`,
    nameZh: `${family.id}审查规则`,
    summaryEn:
      `This rule requires ordered review of ${family.must_cover.length} facts without claiming runtime lowering.`,
    summaryZh:
      `该规则要求按序审查${family.must_cover.length}项事实，且不声称运行时降级。`,
    manifestIds: [`semantic_fixture_families:${family.id}`],
    sources: [policySource, ...factualSources],
    tags: ["mechanic-rule", family.id],
    fields: {
      family_id: family.id,
      source_record_ids: sourceRecords,
      fixture_ids: [],
      required_fact_order: family.must_cover.map((fact, index) => ({
        order: index + 1,
        fact,
      })),
      minimum_cases: family.minimum_cases,
      execution_surface: "ReferenceReviewOnly",
    },
  });
});

const poolCases = familySourceRecords["empty-pool-proof"];
const fixtureRows = [];
for (const family of fixtureContract.required_families) {
  const caseSources = family.id === "empty-pool-proof"
    ? poolCases.map((id) => [id])
    : [familySourceRecords[family.id]];
  for (const [caseIndex, sourceRecordIds] of caseSources.entries()) {
    const suffix = family.id === "empty-pool-proof"
      ? `.${sourceRecordIds[0].replace("pool-audit.", "")}`
      : "";
    const sourceRecords = sourceRecordIds.map((id) => recordById.get(id));
    fixtureRows.push(envelope({
      id: `review-fixture.${family.id}${suffix}`,
      kind: "ReviewFixture",
      nameEn: `${family.id} semantic review${suffix}`,
      nameZh: `${family.id}语义审查${suffix}`,
      summaryEn:
        `A deterministic reference-only fixture reviews ${family.id} case ${caseIndex + 1}.`,
      summaryZh:
        `确定性的纯资料夹具审查${family.id}第${caseIndex + 1}个用例。`,
      manifestIds: [`semantic_fixture_families:${family.id}`],
      sources: [
        manifestRef(
          "semantic_fixture_families",
          family.id,
          "Fixture shape and minimum case count.",
        ),
        ...sourceRecords.flatMap(({ source_refs: refs }) => refs.slice(0, 1)),
      ],
      tags: ["review-fixture", family.id],
      fields: {
        family_id: family.id,
        source_record_ids: sourceRecordIds,
        preconditions: {
          profile: "anomaly-arbitration-v1",
          game_version: "4.4",
          source_records_data_ready: true,
        },
        input: {
          kind: "ReferenceReview",
          case: caseIndex + 1,
        },
        ordered_operations: family.must_cover.map((fact, index) => ({
          order: index + 1,
          operation: "VerifyDeclaredFact",
          fact,
        })),
        expected_facts: family.must_cover.map((fact) => ({
          scope: family.id,
          fact,
          expected: "CoveredByEvidenceOrExplicitPolicyBoundary",
        })),
        evidence_refs: [],
        fixture_evidence_quality: "ProjectPolicy",
        fixture_mechanism_quality: "PolicyBoundary",
        executable_runtime_fixture: false,
      },
    }));
  }
}
fixtureRows.sort((left, right) =>
  left.family_id.localeCompare(right.family_id)
    || left.id.localeCompare(right.id));
for (const rule of ruleRows)
  rule.fixture_ids = fixtureRows.filter(
    ({ family_id: familyId }) => familyId === rule.family_id,
  ).map(({ id }) => id);

const sharedRecords = Object.entries(manifest.categories).flatMap(
  ([category, { records = [] }]) => records.filter(
    ({ ownership }) => ownership === "Shared",
  ).map((record) => ({ category, record })),
);
assert(sharedRecords.length === 316, "shared reconciliation count drift");
const reconciliationRows = sharedRecords.map(({ category, record }) =>
  envelope({
    id: `reconciliation.${digest(
      `${record.source_path}\0${record.row_locator}`,
    ).slice(0, 20)}`,
    kind: "ReconciliationReceipt",
    nameEn: `${category}:${record.id} shared receipt`,
    nameZh: `${category}:${record.id}共享对账`,
    summaryEn:
      "The shared row is reconciled by exact source path, locator and digest without editing a peer Goal.",
    summaryZh:
      "该共享行通过精确源路径、定位与摘要对账，且不修改其他 Goal。",
    ownership: "Shared",
    manifestIds: [`${category}:${record.id}`],
    sources: [manifestRef(
      category,
      record.id,
      "Exact shared-row source identity used for merge-time reconciliation.",
    )],
    tags: ["reconciliation", "shared"],
    fields: {
      source_path: record.source_path,
      row_locator: record.row_locator,
      evidence_sha256: record.evidence_sha256,
      peer_goal_id: "goal03-standard-universe-reference-v1",
      peer_match_state: "AbsentFromCommittedPeerManifest",
      conflict_state: "None",
      merge_action: "RetainGoal13Classification",
    },
    evidenceQuality: "ExactStructured",
    mechanismQuality: "IdentityCrossCheck",
  })).sort((left, right) =>
  left.source_path.localeCompare(right.source_path)
    || left.row_locator.localeCompare(right.row_locator)
    || left.peer_goal_id.localeCompare(right.peer_goal_id)
    || left.id.localeCompare(right.id));

const gapSpecs = [
  ["period-boundary-instant", "G13-P4-B2", "period.8",
    "The released sources expose two displayed end dates but no timezone-aware instant.",
    "Replace when released fixed-version evidence exposes the exact timestamp and offset."],
  ["normal-king-unlock", "G13-P4-B2", "stage.king-normal",
    "Normal King unlock ordering is supported by released text but not an exact structured transition.",
    "Replace when released configuration or reproducible observation exposes the transition."],
  ["combat-form-identity", "G13-P4-B2",
    "participant-policy.character-and-combat-form-uniqueness",
    "Alternate-Path combat-form identity remains a fail-closed project policy.",
    "Replace when released rules explicitly define alternate-Path uniqueness."],
  ["king-protection-arithmetic", "G13-P4-B2",
    "king-protection.composition",
    "Released sources name three contributions but not numeric composition.",
    "Replace when released mechanics expose contribution arithmetic and reset order."],
  ["first-cycle-action-value", "G13-P4-B2",
    "clock.first-cycle-action-value",
    "Released text says the first cycle is larger without giving its numeric Action Value.",
    "Replace when released configuration or a reproducible trace exposes the exact value."],
  ["warning-and-low-cycle-effect", "G13-P4-B2",
    "clock.warning-threshold",
    "Warning threshold and low-cycle buff identity/parameters are unavailable.",
    "Replace when released evidence supplies the threshold, buff identity and parameters."],
  ["quadrant-plugin-bodies", "G13-P4-B2",
    "quadrant-option.3033066",
    "Plugins 0022 and 0023 are named by layout but absent from the extracted ability list.",
    "Replace only with released fixed-version program bodies and update affected fixtures."],
  ["color-medal-target", "G13-P4-B2",
    "aggregation.king-medal-rating",
    "ColorMedalTarget=6 has no released mechanical interpretation.",
    "Replace when released evidence defines whether it is a threshold, identity or presentation selector."],
  ["program-runtime-semantics", "LaterRuntimeGoal",
    "mechanic-contribution.config.001",
    "Configuration bodies are evidence inputs, not imported runtime programs.",
    "Replace only in a separately authorized runtime-lowering goal."],
];
const gapRows = gapSpecs.map(
  ([gapId, ownerBatch, sourceRecordId, boundary, replacementCondition]) => {
    const sourceRecord = recordById.get(sourceRecordId);
    assert(sourceRecord !== undefined, `gap source missing ${sourceRecordId}`);
    return envelope({
      id: `research-gap.${gapId}`,
      kind: "ResearchGap",
      nameEn: `${gapId} evidence boundary`,
      nameZh: `${gapId}证据边界`,
      summaryEn: boundary,
      summaryZh: `该字段保留为显式证据边界：${boundary}`,
      manifestIds: sourceRecord.manifest_record_ids.length > 0
        ? sourceRecord.manifest_record_ids
        : ["semantic_fixture_families:clock-warning-expiry"],
      sources: sourceRecord.source_refs,
      tags: ["nonblocking", "research-gap"],
      fields: {
        blocking: false,
        owner_batch: ownerBatch,
        affected_record_ids: [sourceRecordId],
        evidence_boundary: boundary,
        selected_policy: "PreserveUnavailableWithoutInventingParity",
        alternatives: [
          "Released structured field",
          "Reproducible released observation",
        ],
        replacement_condition: replacementCondition,
      },
    });
  },
).sort((left, right) =>
  Number(left.blocking) - Number(right.blocking)
    || left.owner_batch.localeCompare(right.owner_batch)
    || left.id.localeCompare(right.id));

const manifestRows = Object.entries(manifest.categories).flatMap(
  ([category, { records = [] }]) => records.map((record) => ({
    category,
    record,
    manifestId: `${category}:${record.id}`,
  })),
);
assert(manifestRows.length === 392, "manifest obligation count drift");
const projectedRecords = [
  ...contentRecords,
  ...ruleRows,
  ...fixtureRows,
];
const projections = new Map(manifestRows.map(
  ({ manifestId }) => [manifestId, []],
));
for (const record of projectedRecords)
  for (const manifestId of record.manifest_record_ids ?? [])
    if (projections.has(manifestId))
      projections.get(manifestId).push(record.id);
for (const [manifestId, ids] of projections)
  assert(ids.length > 0, `uncovered manifest obligation ${manifestId}`);
const coverageRows = manifestRows.map(({ category, record, manifestId }) =>
  envelope({
    id: `coverage.${category}.${digest(record.id).slice(0, 16)}`,
    kind: "Coverage",
    nameEn: `${category}:${record.id} coverage`,
    nameZh: `${category}:${record.id}覆盖`,
    summaryEn:
      `${manifestId} resolves to ${projections.get(manifestId).length} normalized reference projection(s).`,
    summaryZh:
      `${manifestId}解析到${projections.get(manifestId).length}个规范资料投影。`,
    ownership: record.ownership,
    manifestIds: [manifestId],
    sources: [manifestRef(
      category,
      record.id,
      "Frozen manifest obligation and exact source receipt.",
    )],
    tags: ["coverage", category],
    fields: {
      manifest_category: category,
      manifest_record_id: record.id,
      required: 1,
      accounted: 1,
      data_ready: 1,
      coverage_percent: "100",
      normalized_record_ids: [...projections.get(manifestId)]
        .sort(compareText),
    },
    evidenceQuality: record.evidence_quality,
    mechanismQuality: record.evidence_quality === "ProjectPolicy"
      ? "PolicyBoundary"
      : "ExactRelationship",
  })).sort((left, right) =>
  left.manifest_category.localeCompare(right.manifest_category)
    || left.manifest_record_id.localeCompare(right.manifest_record_id)
    || left.id.localeCompare(right.id));

const profileManifestId = "profiles:anomaly-arbitration-v1";
const receiptSource = manifestRef(
  "profiles",
  "anomaly-arbitration-v1",
  "Goal-owned pack identity and isolated authoring boundary.",
);
const manifestReceiptRows = [envelope({
  id: "manifest-receipt.anomaly-arbitration-v1",
  kind: "ManifestReceipt",
  nameEn: "Anomaly Arbitration pack manifest receipt",
  nameZh: "异相仲裁资料包清单收据",
  summaryEn:
    "This receipt binds the frozen source inventory, ownership manifest and normalized contracts.",
  summaryZh: "该收据绑定冻结源清单、所有权清单与规范化契约。",
  manifestIds: [profileManifestId],
  sources: [receiptSource],
  tags: ["manifest", "pack-receipt"],
  fields: {
    game_version: "4.4",
    source_revision: revision,
    identity_revision: "7b349e39ee0f6f3bf814567995829b99c95e7a93",
    source_inventory_path:
      "content-manifests/anomaly-arbitration-v1/source-inventory.json",
    source_inventory_sha256: digest(inventoryBytes),
    content_manifest_path:
      "content-manifests/anomaly-arbitration-v1/content-manifest.json",
    content_manifest_sha256: digest(manifestBytes),
    content_manifest_obligations: 392,
    normalized_schema_sha256: digest(schemaBytes),
    fixture_contract_sha256: digest(fixtureBytes),
    normalized_file_count: schema.files.length,
    content_lane: "Experimental",
    bundle_state: "Candidate",
  },
})];

const preSourceMetaRows = [
  ...ruleRows,
  ...reconciliationRows,
  ...coverageRows,
  ...gapRows,
  ...fixtureRows,
  ...manifestReceiptRows,
];
function stableSourceId(source) {
  return `source.${digest(source.source_id).slice(0, 24)}`;
}
for (const fixture of fixtureRows)
  fixture.evidence_refs = [...new Set(fixture.source_refs.map(stableSourceId))]
    .sort(compareText);
const allReferencingRecords = [...contentRecords, ...preSourceMetaRows];
const sourceById = new Map();
for (const record of allReferencingRecords) {
  for (const source of record.source_refs) {
    const id = stableSourceId(source);
    const entry = sourceById.get(id) ?? {
      source,
      manifestIds: new Set(),
      recordIds: new Set(),
    };
    for (const manifestId of record.manifest_record_ids ?? [])
      entry.manifestIds.add(manifestId);
    entry.recordIds.add(record.id);
    sourceById.set(id, entry);
  }
}
const sourceRows = [...sourceById.entries()].map(
  ([id, { source, manifestIds, recordIds }]) => envelope({
    id,
    kind: "Source",
    nameEn: `${source.path_or_page} ${source.locator}`,
    nameZh: `${source.path_or_page} ${source.locator}来源`,
    summaryEn:
      `A frozen ${source.evidence_quality} source receipt supports ${recordIds.size} normalized record(s).`,
    summaryZh:
      `冻结的${source.evidence_quality}来源收据支持${recordIds.size}个规范记录。`,
    ownership: "Shared",
    manifestIds: manifestIds.size === 0
      ? [profileManifestId]
      : [...manifestIds],
    sources: [source],
    tags: ["provenance", "source"],
    fields: {
      source_id: source.source_id,
      locator: source.locator,
      repository_or_url: source.repository_or_url,
      revision_or_access_date: source.revision_or_access_date,
      game_version: source.game_version,
      path_or_page: source.path_or_page,
      evidence_sha256: source.sha256,
      source_evidence_quality: source.evidence_quality,
      source_mechanism_quality: source.mechanism_quality,
      normalized_record_ids: [...recordIds].sort(compareText),
    },
    evidenceQuality: source.evidence_quality,
    mechanismQuality: source.mechanism_quality,
  }),
).sort((left, right) =>
  left.source_id.localeCompare(right.source_id)
    || left.locator.localeCompare(right.locator)
    || left.id.localeCompare(right.id));
const sourceIds = new Set(sourceRows.map(({ id }) => id));
for (const fixture of fixtureRows)
  assert(fixture.evidence_refs.every((id) => sourceIds.has(id)),
    `${fixture.id} evidence reference drift`);

const outputs = {
  "mechanic-rules.json": file(
    "mechanic-rules.json",
    "MechanicRule",
    ruleRows,
  ),
  "sources.json": file("sources.json", "Source", sourceRows),
  "reconciliation.json": file(
    "reconciliation.json",
    "ReconciliationReceipt",
    reconciliationRows,
  ),
  "coverage.json": file("coverage.json", "Coverage", coverageRows),
  "research-gaps.json": file(
    "research-gaps.json",
    "ResearchGap",
    gapRows,
  ),
  "review-fixtures.json": file(
    "review-fixtures.json",
    "ReviewFixture",
    fixtureRows,
  ),
  "manifest.json": file(
    "manifest.json",
    "ManifestReceipt",
    manifestReceiptRows,
  ),
};
for (const [name, document] of Object.entries(outputs)) {
  const bytes = `${JSON.stringify(document, null, 2)}\n`;
  const target = path.join(packRoot, name);
  if (check) {
    const existing = await readFile(target, "utf8").catch(() => "");
    assert(existing === bytes, `${name} generation drift`);
  } else {
    await writeFile(target, bytes);
  }
}

const fileOrder = schema.files.map(({ file: name }) => name)
  .filter((name) => name !== "pack-index.json");
const indexRows = [];
for (const [index, name] of fileOrder.entries()) {
  const target = path.join(packRoot, name);
  const bytes = await readFile(target);
  const document = JSON.parse(bytes);
  const metadata = await stat(target);
  indexRows.push(envelope({
    id: `pack-index.${String(index + 1).padStart(2, "0")}.${name}`,
    kind: "PackIndex",
    nameEn: `${name} pack index`,
    nameZh: `${name}资料包索引`,
    summaryEn:
      `${name} is frozen at ${document.records.length} records and ${bytes.length} bytes.`,
    summaryZh:
      `${name}冻结为${document.records.length}条记录、${bytes.length}字节。`,
    manifestIds: [profileManifestId],
    sources: [receiptSource],
    tags: ["pack-index"],
    fields: {
      file_order: index + 1,
      record_order: 0,
      file_name: name,
      record_kind: document.record_kind,
      row_count: document.records.length,
      byte_count: metadata.size,
      sha256: digest(bytes),
    },
  }));
}
const packIndex = file("pack-index.json", "PackIndex", indexRows);
const packIndexBytes = `${JSON.stringify(packIndex, null, 2)}\n`;
const packIndexTarget = path.join(packRoot, "pack-index.json");
if (check) {
  const existing = await readFile(packIndexTarget, "utf8").catch(() => "");
  assert(existing === packIndexBytes, "pack-index.json generation drift");
} else {
  await writeFile(packIndexTarget, packIndexBytes);
}
console.log(
  `Anomaly Arbitration pack generated: 37 files, ${sourceRows.length} sources, `
    + `${coverageRows.length} coverage rows, ${fixtureRows.length} fixtures, `
    + `${gapRows.length} nonblocking gaps.`,
);
