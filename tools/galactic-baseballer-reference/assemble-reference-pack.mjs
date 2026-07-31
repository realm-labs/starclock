#!/usr/bin/env node

import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const check = process.argv.includes("--check");
const root = path.resolve(".");
const packRoot = path.join(
  root,
  "content-reference",
  "galactic-baseballer-v1",
);
const fragmentRoot = path.join(packRoot, "fragments");
const manifestRoot = path.join(
  root,
  "content-manifests",
  "galactic-baseballer-v1",
);
const rowRevision = "starclock.galactic-baseballer-row.v1";
const manifest = JSON.parse(await readFile(
  path.join(manifestRoot, "content-manifest.json"),
  "utf8",
));
const normalizedSchema = JSON.parse(await readFile(
  path.join(manifestRoot, "normalized-schema.json"),
  "utf8",
));
const approximationRegister = JSON.parse(await readFile(
  path.join(manifestRoot, "approximation-register.json"),
  "utf8",
));
const readPack = async (file) =>
  JSON.parse(await readFile(path.join(packRoot, file), "utf8"));
const readFragment = async (file) =>
  JSON.parse(await readFile(path.join(fragmentRoot, file), "utf8"));

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) =>
      `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}
function digest(value) {
  return createHash("sha256").update(canonical(value)).digest("hex");
}
function stableSort(rows) {
  return rows.sort((left, right) => left.id.localeCompare(right.id, "en"));
}
function uniqueById(rows, label) {
  const result = new Map();
  for (const row of rows) {
    const existing = result.get(row.id);
    if (existing !== undefined && canonical(existing) !== canonical(row))
      throw new Error(`conflicting ${label} row: ${row.id}`);
    result.set(row.id, row);
  }
  return stableSort([...result.values()]);
}
function sourceIdForManifest(record) {
  return `source.goal16.${record.evidence_sha256.slice(0, 16)}`;
}
function manifestSource(record) {
  return {
    source_id: sourceIdForManifest(record),
    repository_or_url: record.repository === "turnbasedgamedata"
      ? "https://gitlab.com/Dimbreath/turnbasedgamedata.git"
      : record.repository === "StarRailRes"
        ? "https://github.com/Mar-7th/StarRailRes.git"
        : "starclock",
    revision_or_access_date: record.repository_revision
      ?? "goal16-source-manifest-v1",
    game_version: record.game_version,
    path_or_page: record.source_path,
    locator: record.row_locator,
    sha256: record.evidence_sha256,
    evidence_quality: record.evidence_quality,
    mechanism_quality: record.evidence_quality === "ProjectPolicy"
      ? "PolicyBoundary"
      : "ExactRelationship",
    note: record.note,
  };
}
function approximationRow(row) {
  const sourcePayload = {
    register: "content-manifests/galactic-baseballer-v1/approximation-register.json",
    id: row.id,
  };
  return {
    id: `galactic-baseballer.approximation.${row.id}`,
    schema_revision: rowRevision,
    kind: "Approximation",
    name_en: `Replaceable boundary: ${row.id}`,
    name_zh_cn: `可替换边界：${row.id}`,
    summary_en:
      "Explicit ReferenceOnly deterministic policy; not an observed-parity claim.",
    summary_zh_cn: "显式的仅供资料使用确定性策略；不声称观察一致性。",
    profile_ids: [
      "galactic-baseballer.departure.v2_2",
      "galactic-baseballer.demon-king.v3_3",
    ],
    ownership: "SharedBase",
    coverage_state: "Researched",
    evidence_quality: row.evidence_quality,
    mechanism_quality: row.mechanism_quality,
    manifest_record_ids: [],
    source_refs: [{
      source_id: `source.goal16.policy.${digest(sourcePayload).slice(0, 16)}`,
      repository_or_url: "starclock",
      revision_or_access_date:
        "starclock.galactic-baseballer-approximation-register.v1",
      game_version: "4.4",
      path_or_page:
        "content-manifests/galactic-baseballer-v1/approximation-register.json",
      locator: `id=${row.id}`,
      sha256: digest(sourcePayload),
      evidence_quality: "ProjectPolicy",
      mechanism_quality: "PolicyBoundary",
      note: "explicit replaceable deterministic boundary",
      replacement_condition: row.replacement_condition,
    }],
    tags: ["approximation", "project-policy", "replaceable"],
    field_path: row.field_path,
    unavailable_fact: row.unavailable_fact,
    known_released_facts: row.known_released_facts,
    selected_policy: row.selected_policy,
    rejected_alternatives: row.rejected_alternatives,
    rationale: row.rationale,
    affected_fixture_ids: row.affected_fixture_ids,
    confidence: row.confidence,
    replacement_condition: row.replacement_condition,
  };
}

const mergePlan = new Map([
  ["profiles.json", ["profiles.json", "fragments/demon-profile.json"]],
  ["release-boundaries.json", [
    "release-boundaries.json",
    "fragments/demon-release-boundaries.json",
  ]],
  ["stages.json", ["stages.json", "fragments/demon-stages.json"]],
  ["stage-periods.json", [
    "stage-periods.json",
    "fragments/demon-stage-periods.json",
  ]],
  ["weapons.json", ["weapons.json", "fragments/demon-weapons.json"]],
  ["weapon-levels.json", [
    "weapon-levels.json",
    "fragments/demon-weapon-levels.json",
  ]],
  ["weapon-triggers.json", [
    "weapon-triggers.json",
    "fragments/demon-weapon-triggers.json",
  ]],
  ["accessories.json", [
    "accessories.json",
    "fragments/demon-accessories.json",
  ]],
  ["accessory-levels.json", [
    "accessory-levels.json",
    "fragments/demon-accessory-levels.json",
  ]],
  ["accessory-bindings.json", [
    "accessory-bindings.json",
    "fragments/demon-accessory-bindings.json",
  ]],
  ["synthesis-recipes.json", [
    "synthesis-recipes.json",
    "fragments/demon-synthesis-recipes.json",
  ]],
  ["synthesis-inputs.json", [
    "synthesis-inputs.json",
    "fragments/demon-synthesis-inputs.json",
  ]],
  ["level-thresholds.json", [
    "level-thresholds.json",
    "fragments/demon-level-thresholds.json",
  ]],
  ["candidate-pools.json", [
    "candidate-pools.json",
    "fragments/demon-candidate-pools.json",
  ]],
  ["candidate-policies.json", [
    "candidate-policies.json",
    "fragments/demon-candidate-policies.json",
  ]],
  ["inventory-slots.json", [
    "inventory-slots.json",
    "fragments/demon-inventory-slots.json",
  ]],
  ["inventory-operations.json", [
    "inventory-operations.json",
    "fragments/demon-inventory-operations.json",
  ]],
  ["encounters.json", [
    "encounters.json",
    "fragments/demon-encounters.json",
  ]],
  ["waves.json", ["waves.json", "fragments/demon-waves.json"]],
  ["enemy-slots.json", [
    "enemy-slots.json",
    "fragments/demon-enemy-slots.json",
  ]],
  ["enemies.json", ["enemies.json", "fragments/demon-enemies.json"]],
  ["enemy-skills.json", [
    "enemy-skills.json",
    "fragments/demon-enemy-skills.json",
  ]],
  ["enemy-statuses.json", [
    "enemy-statuses.json",
    "fragments/demon-enemy-statuses.json",
  ]],
  ["scoring-rules.json", [
    "scoring-rules.json",
    "fragments/demon-scoring-rules.json",
  ]],
  ["settlement-rules.json", [
    "settlement-rules.json",
    "fragments/demon-settlement-rules.json",
  ]],
]);

const outputs = new Map();
for (const [target, inputs] of mergePlan) {
  const rows = [];
  for (const input of inputs) {
    rows.push(...(input.startsWith("fragments/")
      ? await readFragment(input.slice("fragments/".length))
      : await readPack(input)));
  }
  outputs.set(target, uniqueById(rows, target));
}
outputs.set(
  "profile-differences.json",
  await readFragment("demon-edition-differences.json"),
);
outputs.set(
  "adventure-strategies.json",
  await readFragment("demon-adventure-strategies.json"),
);
outputs.set("progression.json", uniqueById([
  ...await readFragment("departure-progression.json"),
  ...await readFragment("demon-progression.json"),
  ...await readFragment("demon-team-bonuses.json"),
], "progression"));
outputs.set("currencies.json", uniqueById([
  ...await readFragment("departure-currencies.json"),
  ...await readFragment("demon-currencies.json"),
], "currency"));
outputs.set("shop-upgrades.json", uniqueById([
  ...await readFragment("departure-shop-upgrades.json"),
  ...await readFragment("demon-shop-upgrades.json"),
], "shop upgrade"));
outputs.set("unlocks.json", uniqueById([
  ...await readFragment("departure-unlocks.json"),
  ...await readFragment("demon-unlocks.json"),
], "unlock"));

const mechanicRules = uniqueById([
  ...await readFragment("departure-mechanic-rules.json"),
  ...await readFragment("demon-arsenal-mechanic-rules.json"),
  ...await readFragment("demon-progression-mechanic-rules.json"),
  ...await readFragment("demon-encounter-mechanic-rules.json"),
], "mechanic rule");
const reviewFixtures = uniqueById([
  ...await readFragment("departure-review-fixtures.json"),
  ...await readFragment("demon-arsenal-review-fixtures.json"),
  ...await readFragment("demon-progression-review-fixtures.json"),
  ...await readFragment("demon-encounter-review-fixtures.json"),
], "review fixture");
const approximations = uniqueById([
  ...approximationRegister.records.map(approximationRow),
  ...await readFragment("demon-progression-approximations.json"),
], "approximation");
const shopAtomicity = approximations.find(({ id }) =>
  id.endsWith(".shop-transaction-atomicity"));
if (shopAtomicity !== undefined) {
  shopAtomicity.profile_ids = [
    "galactic-baseballer.departure.v2_2",
    "galactic-baseballer.demon-king.v3_3",
  ];
  shopAtomicity.ownership = "SharedBase";
  shopAtomicity.summary_en =
    "Shared ReferenceOnly store transaction boundary; not observed parity.";
  shopAtomicity.summary_zh_cn =
    "共享的仅供资料使用商店交易边界；不声称观察一致性。";
}
outputs.set("mechanic-rules.json", mechanicRules);
outputs.set("approximations.json", approximations);
outputs.set("review-fixtures.json", reviewFixtures);

const authoredRows = [...outputs.entries()]
  .filter(([file]) => ![
    "sources.json",
    "reconciliation.json",
    "coverage.json",
    "research-gaps.json",
    "manifest.json",
    "pack-index.json",
  ].includes(file))
  .flatMap(([file, rows]) => rows.map((row) => ({ file, row })));
const inverseManifest = new Map();
for (const { row } of authoredRows) {
  for (const manifestId of row.manifest_record_ids ?? []) {
    const owners = inverseManifest.get(manifestId) ?? [];
    owners.push(row.id);
    inverseManifest.set(manifestId, owners);
  }
}
for (const rule of mechanicRules) {
  const familyRecord = manifest.categories.semantic_fixture_families.records
    .find(({ id }) => id === rule.family_id);
  if (familyRecord !== undefined) {
    const owners = inverseManifest.get(familyRecord.id) ?? [];
    owners.push(rule.id);
    inverseManifest.set(familyRecord.id, owners);
  }
}
for (const encounter of outputs.get("encounters.json")) {
  const manifestId = `WaveGroupID:${encounter.infinite_group_id}`;
  if (manifest.categories.infinite_stage_groups.records.some(
    ({ id }) => id === manifestId,
  )) {
    const owners = inverseManifest.get(manifestId) ?? [];
    owners.push(encounter.id);
    inverseManifest.set(manifestId, owners);
  }
}
for (const enemy of outputs.get("enemies.json")) {
  const manifestId = `MonsterTemplateID:${enemy.source_monster_template_id}`;
  if (manifest.categories.enemy_templates.records.some(
    ({ id }) => id === manifestId,
  )) {
    const owners = inverseManifest.get(manifestId) ?? [];
    owners.push(enemy.id);
    inverseManifest.set(manifestId, owners);
  }
}
const differenceRow = outputs.get("profile-differences.json")[0];
for (const comparison of differenceRow.constant_comparisons ?? []) {
  for (const [profile, source] of [
    ["galactic-baseballer.departure.v2_2", comparison.departure_source],
    ["galactic-baseballer.demon-king.v3_3", comparison.demon_king_source],
  ]) {
    if (source === undefined) continue;
    const family = path.basename(source.path, ".json");
    const manifestId =
      `${profile}:${family}:${String(source.row).padStart(4, "0")}`;
    if (manifest.categories.mode_constants.records.some(
      ({ id }) => id === manifestId,
    )) {
      const owners = inverseManifest.get(manifestId) ?? [];
      owners.push(differenceRow.id);
      inverseManifest.set(manifestId, owners);
    }
  }
}
function addOwners(manifestId, rows) {
  const owners = inverseManifest.get(manifestId) ?? [];
  owners.push(...rows.map(({ id }) => id));
  inverseManifest.set(manifestId, owners);
}
for (const record of manifest.categories.config_programs.records) {
  if (inverseManifest.has(record.id)) continue;
  const program = record.source_path;
  let rows = [];
  if (/ExpAndLevel|Scaling/u.test(program))
    rows = outputs.get("level-thresholds.json");
  else if (/Weapon/u.test(program))
    rows = outputs.get("weapon-triggers.json");
  else if (/Accessory/u.test(program))
    rows = outputs.get("accessory-bindings.json");
  else if (/Card/u.test(program))
    rows = [
      ...outputs.get("candidate-pools.json"),
      ...outputs.get("adventure-strategies.json"),
    ];
  else if (/Store/u.test(program))
    rows = outputs.get("shop-upgrades.json");
  else if (/GreenHand|Tutorial/u.test(program))
    rows = outputs.get("unlocks.json");
  else if (/TeamBonus/u.test(program))
    rows = outputs.get("progression.json").filter(({ tags }) =>
      tags?.includes("team-bonus"));
  else if (/BossScoring/u.test(program))
    rows = outputs.get("scoring-rules.json");
  else if (/Extra/u.test(program))
    rows = outputs.get("candidate-policies.json");
  else if (/Devil/u.test(program))
    rows = reviewFixtures.filter(({ family_id: family }) =>
      family === "boss-phase-final-settlement");
  if (rows.length > 0) addOwners(record.id, rows);
}

const allManifestRows = Object.entries(manifest.categories)
  .flatMap(([categoryId, { records }]) =>
    records.map((record) => ({ categoryId, record })));
const profileEnemyRows = new Map([
  ["Departure", outputs.get("enemies.json").filter(({ ownership }) =>
    ownership === "Shared").map(({ id }) => id)],
  ["DemonKing", outputs.get("enemies.json").filter(({ profile_ids: ids }) =>
    ids.includes("galactic-baseballer.demon-king.v3_3")).map(({ id }) => id)],
]);
for (const { categoryId, record } of allManifestRows) {
  if (inverseManifest.has(record.id)) continue;
  if (categoryId === "enemy_collection_locators") {
    inverseManifest.set(
      record.id,
      profileEnemyRows.get(record.ownership)?.slice(0, 1) ?? [],
    );
  } else if (categoryId === "content_tags") {
    const profileMarker = record.ownership === "DemonKing"
      ? "demon-king"
      : "departure";
    inverseManifest.set(record.id, outputs.get("shop-upgrades.json")
      .filter(({ tags }) => tags.includes(profileMarker))
      .slice(0, 1)
      .map(({ id }) => id));
  }
}

const coverage = [];
const reconciliation = [];
for (const { categoryId, record } of allManifestRows) {
  const normalizedIds = [...new Set(inverseManifest.get(record.id) ?? [])]
    .sort();
  const evidenceOnly = record.runtime_disposition === "EvidenceOnly";
  const closureKind = evidenceOnly
    ? "EvidenceOnlyExcluded"
    : normalizedIds.length > 0
      ? "NormalizedRecordReference"
      : "ExactSourceReceipt";
  coverage.push({
    id: `galactic-baseballer.coverage.${digest({
      categoryId,
      recordId: record.id,
    }).slice(0, 20)}`,
    schema_revision: rowRevision,
    kind: "CoverageRow",
    category_id: categoryId,
    record_id: record.id,
    profile_ownership: record.ownership,
    source_path: record.source_path,
    row_locator: record.row_locator,
    evidence_sha256: record.evidence_sha256,
    evidence_quality: record.evidence_quality,
    runtime_disposition: record.runtime_disposition,
    coverage_state: evidenceOnly ? "EvidenceOnly" : "DataReady",
    closure_kind: closureKind,
    normalized_record_ids: normalizedIds,
    note: record.note,
  });
  reconciliation.push({
    id: `galactic-baseballer.reconciliation.${digest(record.id).slice(0, 20)}`,
    schema_revision: rowRevision,
    kind: "ReconciliationReceipt",
    category_id: categoryId,
    source_record_id: record.id,
    source_path: record.source_path,
    row_locator: record.row_locator,
    evidence_sha256: record.evidence_sha256,
    stable_identity_policy: record.selector,
    normalized_record_ids: normalizedIds,
    closure_kind: closureKind,
    inferred_from_name_or_id_range: false,
    exact_once: true,
  });
}
coverage.sort((left, right) =>
  left.category_id.localeCompare(right.category_id, "en")
  || left.record_id.localeCompare(right.record_id, "en"));
stableSort(reconciliation);
outputs.set("coverage.json", coverage);
outputs.set("reconciliation.json", reconciliation);

const sourceConsumers = new Map();
for (const { row } of authoredRows) {
  for (const source of row.source_refs ?? []) {
    const entry = sourceConsumers.get(source.source_id) ?? {
      refs: [],
      consumers: new Set(),
    };
    entry.refs.push(source);
    entry.consumers.add(row.id);
    sourceConsumers.set(source.source_id, entry);
  }
}
for (const { record } of allManifestRows) {
  const source = manifestSource(record);
  const entry = sourceConsumers.get(source.source_id) ?? {
    refs: [],
    consumers: new Set(),
  };
  entry.refs.push(source);
  sourceConsumers.set(source.source_id, entry);
}
const sources = stableSort([...sourceConsumers.entries()].map(
  ([sourceId, { refs, consumers }]) => {
    const first = refs[0];
    return {
      id: sourceId,
      schema_revision: rowRevision,
      kind: "Source",
      repository_or_url: first.repository_or_url,
      revision_or_access_date: first.revision_or_access_date,
      game_version: first.game_version,
      path_or_pages: [...new Set(refs.map(({ path_or_page: value }) => value))]
        .sort(),
      locators: [...new Set(refs.map(({ locator }) => locator))].sort(),
      sha256_values: [...new Set(refs.map(({ sha256 }) => sha256))].sort(),
      evidence_qualities:
        [...new Set(refs.map(({ evidence_quality: value }) => value))].sort(),
      mechanism_qualities:
        [...new Set(refs.map(({ mechanism_quality: value }) => value))].sort(),
      notes: [...new Set(refs.map(({ note }) => note).filter(Boolean))].sort(),
      replacement_conditions: [...new Set(refs
        .map(({ replacement_condition: value }) => value)
        .filter(Boolean))].sort(),
      referenced_by_record_ids: [...consumers].sort(),
    };
  },
));
outputs.set("sources.json", sources);

const researchGaps = stableSort(approximations.map((row) => ({
  id: `galactic-baseballer.research-gap.${digest(row.id).slice(0, 20)}`,
  schema_revision: rowRevision,
  kind: "ResearchGap",
  approximation_id: row.id,
  field_path: row.field_path,
  state: "ReplaceableNonBlocking",
  unavailable_fact: row.unavailable_fact,
  selected_policy: row.selected_policy,
  rejected_alternatives: row.rejected_alternatives,
  rationale: row.rationale,
  affected_fixture_ids: row.affected_fixture_ids,
  confidence: row.confidence,
  replacement_condition: row.replacement_condition,
  terminal_blocker: false,
})));
outputs.set("research-gaps.json", researchGaps);

const stateCounts = Object.fromEntries(
  ["DataReady", "EvidenceOnly"].map((state) => [
    state,
    coverage.filter(({ coverage_state: value }) => value === state).length,
  ]),
);
const packManifest = [{
  id: "galactic-baseballer-reference-v1",
  schema_revision: rowRevision,
  kind: "PackManifest",
  game_version: "4.4",
  profile_ids: [
    "galactic-baseballer.departure.v2_2",
    "galactic-baseballer.demon-king.v3_3",
  ],
  source_obligation_count: coverage.length,
  coverage_state_counts: stateCounts,
  approximation_count: approximations.length,
  mechanic_family_count:
    new Set(mechanicRules.map(({ family_id: family }) => family)).size,
  mechanic_rule_count: mechanicRules.length,
  review_fixture_count: reviewFixtures.length,
  runtime_enabled: false,
  delivery_lane: "Candidate",
}];
outputs.set("manifest.json", packManifest);

const schemaFiles = normalizedSchema.files.map(({ file }) => file);
const missingOutputs = schemaFiles.filter((file) =>
  file !== "pack-index.json" && !outputs.has(file));
if (missingOutputs.length > 0)
  throw new Error(`normalized outputs missing: ${missingOutputs.join(",")}`);
const packIndex = [{
  id: "galactic-baseballer-reference-v1.index",
  schema_revision: rowRevision,
  kind: "PackIndex",
  normalized_file_count: schemaFiles.length,
  indexed_file_count: schemaFiles.length - 1,
  files: schemaFiles.filter((file) => file !== "pack-index.json").map(
    (file, ordinal) => ({
      ordinal,
      file,
      row_count: outputs.get(file).length,
      canonical_sha256: digest(outputs.get(file)),
    }),
  ),
  ordering_policy: "normalized-schema file order then declared row ordering",
  runtime_enabled: false,
  delivery_lane: "Candidate",
}];
outputs.set("pack-index.json", packIndex);

await mkdir(packRoot, { recursive: true });
for (const file of schemaFiles) {
  const value = outputs.get(file);
  const target = path.join(packRoot, file);
  const encoded = `${JSON.stringify(value, null, 2)}\n`;
  if (check) {
    const existing = await readFile(target, "utf8");
    if (existing !== encoded)
      throw new Error(`Galactic Baseballer pack drift: ${target}`);
  } else {
    await writeFile(target, encoded);
  }
}
console.log(
  `Galactic Baseballer reference pack ${check ? "verified" : "wrote"}: `
  + `${schemaFiles.length} files, ${coverage.length} obligations `
  + `(${stateCounts.DataReady} DataReady/${stateCounts.EvidenceOnly} `
  + `EvidenceOnly), ${mechanicRules.length} rules, `
  + `${reviewFixtures.length} fixtures and ${approximations.length} boundaries`,
);
