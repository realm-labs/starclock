#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile, mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(".");
const sourceCache = path.resolve(option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/galactic-baseballer-source"));
const sourceRoot = path.join(sourceCache, "turnbasedgamedata");
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
const inventoryPath = path.join(
  root,
  "content-manifests",
  "galactic-baseballer-v1",
  "source-inventory.json",
);
const output = path.join(
  root,
  "content-manifests",
  "galactic-baseballer-v1",
  "content-manifest.json",
);
const profileTableCategory = new Map([
  ["BoxGroup", "offer_box_groups"],
  ["BoxItem", "offer_box_items"],
  ["CardConfig", "upgrade_cards"],
  ["CardType", "upgrade_card_types"],
  ["ConstValueCommon", "mode_constants"],
  ["ForgeMaterial", "synthesis_materials"],
  ["GearCollection", "weapon_collections"],
  ["GearConfig", "weapon_levels"],
  ["GearTypeConfig", "weapon_types"],
  ["MazeBuff", "accessory_levels"],
  ["MonsteCollection", "enemy_collection_locators"],
  ["RaccoonTalk", "presentation_locators"],
  ["Reward", "reward_locators"],
  ["ShopConfig", "shop_progression"],
  ["StageConfig", "profile_stages"],
  ["StagePeriod", "stage_periods"],
  ["TagConfig", "content_tags"],
  ["Tutorial", "tutorial_entries"],
]);
const tablePaths = {
  stage: "ExcelOutput/StageConfig.json",
  infiniteGroup: "ExcelOutput/StageInfiniteGroup.json",
  infiniteWave: "ExcelOutput/StageInfiniteWaveConfig.json",
  infiniteMonsterGroup: "ExcelOutput/StageInfiniteMonsterGroup.json",
  monster: "ExcelOutput/MonsterConfig.json",
  monsterTemplate: "ExcelOutput/MonsterTemplateConfig.json",
  monsterSkill: "ExcelOutput/MonsterSkillConfig.json",
  monsterStatus: "ExcelOutput/MonsterStatusConfig.json",
};
const semanticFamilies = [
  "profile-version-selection",
  "stage-difficulty-selection",
  "wave-battle-phase-progression",
  "experience-team-level-up",
  "random-upgrade-candidates",
  "weapon-acquisition-duplicate-upgrade",
  "accessory-acquisition-duplicate-upgrade",
  "slot-capacity-expansion-replacement",
  "weapon-automatic-action",
  "character-action-triggered-weapon",
  "resonance-accessory-binding",
  "legendary-weapon-synthesis",
  "twin-weapon-synthesis",
  "supreme-weapon-synthesis",
  "adventure-strategy",
  "team-bonus",
  "galactic-store-progression",
  "score-rating-clear",
  "boss-phase-final-settlement",
  "no-legal-candidate-failure-invariance",
];

function option(name) {
  const index = args.indexOf(name);
  if (index === -1) return undefined;
  if (args[index + 1] === undefined || args[index + 1].startsWith("--"))
    throw new Error(`${name} requires a path`);
  return args[index + 1];
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function git(gitArgs) {
  return execFileSync("git", ["-C", sourceRoot, ...gitArgs], {
    encoding: "utf8",
    env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
    maxBuffer: 512 * 1024 * 1024,
  });
}

function losslessJson(bytes) {
  return JSON.parse(bytes.toString("utf8").replace(
    /(:\s*|[\[,]\s*)(-?\d{16,})(?=\s*[,}\]])/gu,
    '$1"$2"',
  ));
}

function canonical(value) {
  if (Array.isArray(value))
    return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value).sort(compareText).map((key) =>
      `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function sha256(value) {
  return createHash("sha256").update(
    Buffer.isBuffer(value) ? value : canonical(value),
  ).digest("hex");
}

function tableSuffix(relativePath) {
  const stem = path.basename(relativePath, ".json");
  return stem.replace(/^EvoBdSC|^EvolveBuild/u, "");
}

function keyLocator(row) {
  const preferred = [
    "ID",
    "GroupID",
    "LvID",
    "Season",
    "ConstValueName",
    "ForgeGearID",
    "GearID",
    "Level",
    "StageMergedID",
    "StagePeriodID",
    "StageID",
    "RewardID",
    "RaccoonState",
  ];
  const parts = preferred
    .filter((key) => row[key] !== undefined)
    .slice(0, 3)
    .map((key) => `${key}=${String(row[key])}`);
  return parts.length === 0 ? "" : `;${parts.join(";")}`;
}

function receipt({
  id,
  sourcePath,
  row,
  index,
  ownership,
  selector,
  disposition = "ReferenceOnly",
  note,
}) {
  return {
    id,
    game_version: "4.4",
    repository: "turnbasedgamedata",
    repository_revision: revision,
    source_path: sourcePath,
    row_locator: index === undefined
      ? "whole-file"
      : `row=${index}${keyLocator(row)}`,
    evidence_sha256: sha256(row),
    evidence_quality: "ExactStructured",
    ownership,
    selector,
    runtime_disposition: disposition,
    data_status: disposition === "EvidenceOnly"
      ? "EvidenceOnly"
      : "Pending",
    note,
  };
}

function category(id, membershipBasis, records, closureTarget = "DataReady") {
  records.sort((left, right) => compareText(left.id, right.id));
  return {
    id,
    membership_basis: membershipBasis,
    closure_target: closureTarget,
    count: records.length,
    records,
  };
}

function sourceBytes(relativePath) {
  return readFile(path.join(sourceRoot, relativePath));
}

function matchingRows(rows, field, ids) {
  return rows
    .map((row, index) => ({ row, index }))
    .filter(({ row }) => ids.has(row[field]));
}

if (git(["rev-parse", "HEAD"]).trim() !== revision)
  throw new Error("turnbasedgamedata revision drift");
if (git(["status", "--porcelain"]).trim() !== "")
  throw new Error("turnbasedgamedata source cache is dirty");

const inventoryBytes = await readFile(inventoryPath);
const inventory = JSON.parse(inventoryBytes);
if (inventory.schema_revision
  !== "starclock.galactic-baseballer-source-inventory.v1")
  throw new Error("source inventory schema drift");
const sourceInventorySha256 = sha256(inventoryBytes);
const modeRecords = inventory.records.filter((record) =>
  record.repository === "turnbasedgamedata"
  && ["departure-or-shared-candidate", "demon-king-candidate"]
    .includes(record.classification));
const tableRecords = modeRecords.filter((record) =>
  record.path.startsWith("ExcelOutput/") && record.json_shape === "array");
const programRecords = modeRecords.filter((record) =>
  record.path.startsWith("Config/"));
if (tableRecords.length !== 29 || programRecords.length !== 35)
  throw new Error("candidate table/program denominator drift");

const categories = {};
const categoryBuckets = new Map();
for (const record of tableRecords) {
  const suffix = tableSuffix(record.path);
  const categoryId = profileTableCategory.get(suffix);
  if (categoryId === undefined)
    throw new Error(`unclassified profile table: ${record.path}`);
  const bytes = await sourceBytes(record.path);
  const rows = losslessJson(bytes);
  const demonKing = record.path.includes("EvoBdSC");
  const ownership = demonKing ? "DemonKing" : "Departure";
  const profile = demonKing
    ? "galactic-baseballer.demon-king.v3_3"
    : "galactic-baseballer.departure.v2_2";
  const bucket = categoryBuckets.get(categoryId) ?? [];
  for (const [index, row] of rows.entries()) {
    const locatorOnly = ["presentation_locators", "reward_locators"]
      .includes(categoryId);
    bucket.push(receipt({
      id: `${profile}:${path.basename(record.path, ".json")}:`
        + String(index).padStart(4, "0"),
      sourcePath: record.path,
      row,
      index,
      ownership,
      selector: `${profile} dedicated table family at the pinned 4.4 revision`,
      disposition: locatorOnly ? "EvidenceOnly" : "ReferenceOnly",
      note: locatorOnly
        ? "retained for exclusion/reconciliation; account reward, story and presentation data do not enter the simulation core"
        : "profile-owned source obligation; names and adjacent IDs do not grant cross-profile membership",
    }));
  }
  categoryBuckets.set(categoryId, bucket);
}

const programReceipts = [];
for (const record of programRecords) {
  const bytes = await sourceBytes(record.path);
  const demonKing = /(?:SC_|_SC|_S2)/u.test(record.path)
    || record.path.includes("/EvolveBuildSC");
  programReceipts.push(receipt({
    id: record.path,
    sourcePath: record.path,
    row: bytes,
    ownership: demonKing ? "DemonKing" : "SharedBase",
    selector: demonKing
      ? "Demon King program carries an exact SC or S2 source identifier"
      : "shared-base program is paired with explicit profile adapters; reuse remains a modeling boundary, not an observed parity claim",
    note: demonKing
      ? "Demon King-specific executable evidence retained as reference only"
      : "base executable evidence retained once and referenced by both versioned profiles",
  }));
}
categoryBuckets.set("config_programs", programReceipts);

const loadedTables = {};
const tableBlobReceipts = [];
for (const [name, relativePath] of Object.entries(tablePaths)) {
  const bytes = await sourceBytes(relativePath);
  loadedTables[name] = losslessJson(bytes);
  const oid = git(["rev-parse", `HEAD:${relativePath}`]).trim();
  tableBlobReceipts.push({
    path: relativePath,
    git_blob_oid: oid,
    bytes: bytes.length,
    sha256: sha256(bytes),
    purpose: [
      "stage",
      "infinite-group",
      "infinite-wave",
      "infinite-monster-group",
      "enemy-identity",
    ][Math.min(tableBlobReceipts.length, 4)],
  });
}

const departurePeriods = losslessJson(await sourceBytes(
  "ExcelOutput/EvolveBuildStagePeriod.json",
));
const demonPeriods = losslessJson(await sourceBytes(
  "ExcelOutput/EvoBdSCStagePeriod.json",
));
const stageIds = new Set(
  [...departurePeriods, ...demonPeriods].map((row) => row.StageID),
);
const stageRows = matchingRows(
  loadedTables.stage,
  "StageID",
  stageIds,
);
const foundStageIds = new Set(stageRows.map(({ row }) => row.StageID));
const missingStageIds = [...stageIds]
  .filter((id) => !foundStageIds.has(id))
  .sort((left, right) => left - right);

const groupIds = new Set(stageRows.flatMap(({ row }) =>
  row.StageConfigData
    .filter((entry) => entry.BFLIFKBEOPJ === "_StageInfiniteGroup")
    .map((entry) => Number(entry.MNDFOPKBHKP))));
const groupRows = matchingRows(
  loadedTables.infiniteGroup,
  "WaveGroupID",
  groupIds,
);
const waveIds = new Set(groupRows.flatMap(({ row }) => row.WaveIDList));
const waveRows = matchingRows(
  loadedTables.infiniteWave,
  "InfiniteWaveID",
  waveIds,
);
const monsterGroupIds = new Set(
  waveRows.flatMap(({ row }) => row.MonsterGroupIDList),
);
const monsterGroupRows = matchingRows(
  loadedTables.infiniteMonsterGroup,
  "InfiniteMonsterGroupID",
  monsterGroupIds,
);
const monsterIds = new Set(
  monsterGroupRows.flatMap(({ row }) => row.MonsterList),
);
const monsterRows = matchingRows(
  loadedTables.monster,
  "MonsterID",
  monsterIds,
);
const templateIds = new Set(
  monsterRows.map(({ row }) => row.MonsterTemplateID),
);
const templateRows = matchingRows(
  loadedTables.monsterTemplate,
  "MonsterTemplateID",
  templateIds,
);
const skillIds = new Set(monsterRows.flatMap(({ row }) => row.SkillList));
const skillRows = matchingRows(
  loadedTables.monsterSkill,
  "SkillID",
  skillIds,
);
const effectIds = new Set(
  skillRows.flatMap(({ row }) => row.ExtraEffectIDList),
);
const statusRows = matchingRows(
  loadedTables.monsterStatus,
  "StatusID",
  effectIds,
);
const foundStatusIds = new Set(statusRows.map(({ row }) => row.StatusID));
const unresolvedEffectIds = [...effectIds]
  .filter((id) => !foundStatusIds.has(id))
  .sort((left, right) => left - right);

function sharedReceipts({
  rows,
  name,
  tableKey,
  selector,
  note,
}) {
  const sourcePath = tablePaths[name];
  return rows.map(({ row, index }) => receipt({
    id: `${tableKey}:${String(row[tableKey])}`,
    sourcePath,
    row,
    index,
    ownership: "Shared",
    selector,
    note,
  }));
}

categoryBuckets.set("shared_stage_configs", sharedReceipts({
  rows: stageRows,
  name: "stage",
  tableKey: "StageID",
  selector: "exact StagePeriod.StageID reference from either versioned profile",
  note: "shared combat stage definition; profile ownership remains on the referring StagePeriod row",
}));
categoryBuckets.set("infinite_stage_groups", sharedReceipts({
  rows: groupRows,
  name: "infiniteGroup",
  tableKey: "WaveGroupID",
  selector: "exact StageConfig._StageInfiniteGroup reference",
  note: "ordered wave-group closure",
}));
categoryBuckets.set("infinite_waves", sharedReceipts({
  rows: waveRows,
  name: "infiniteWave",
  tableKey: "InfiniteWaveID",
  selector: "exact StageInfiniteGroup.WaveIDList reference",
  note: "ordered infinite-wave closure",
}));
categoryBuckets.set("infinite_monster_groups", sharedReceipts({
  rows: monsterGroupRows,
  name: "infiniteMonsterGroup",
  tableKey: "InfiniteMonsterGroupID",
  selector: "exact StageInfiniteWaveConfig.MonsterGroupIDList reference",
  note: "ordered enemy-slot closure",
}));
categoryBuckets.set("enemy_variants", sharedReceipts({
  rows: monsterRows,
  name: "monster",
  tableKey: "MonsterID",
  selector: "exact StageInfiniteMonsterGroup.MonsterList reference",
  note: "shared stable enemy variant identity; source data is referenced, never copied into another mode partition",
}));
categoryBuckets.set("enemy_templates", sharedReceipts({
  rows: templateRows,
  name: "monsterTemplate",
  tableKey: "MonsterTemplateID",
  selector: "exact MonsterConfig.MonsterTemplateID reference",
  note: "shared stable enemy template identity",
}));
categoryBuckets.set("enemy_skills", sharedReceipts({
  rows: skillRows,
  name: "monsterSkill",
  tableKey: "SkillID",
  selector: "exact MonsterConfig.SkillList reference",
  note: "shared enemy skill evidence",
}));
categoryBuckets.set("enemy_statuses", sharedReceipts({
  rows: statusRows,
  name: "monsterStatus",
  tableKey: "StatusID",
  selector: "exact MonsterSkillConfig.ExtraEffectIDList reference with matching MonsterStatusConfig.StatusID",
  note: "shared enemy status evidence",
}));

categoryBuckets.set("profiles", [
  {
    id: "galactic-baseballer.departure.v2_2",
    game_version: "4.4",
    source_path: "docs/goals/16-galactic-baseballer-reference-data.md",
    row_locator: "profile contract: Departure",
    evidence_sha256: sha256({
      profile: "galactic-baseballer.departure.v2_2",
      release: "2.2",
      retained_at: "4.4",
    }),
    evidence_quality: "ProjectPolicy",
    ownership: "Departure",
    selector: "released Version 2.2 profile retained independently at the Version 4.4 baseline",
    runtime_disposition: "ReferenceOnly",
    data_status: "Pending",
    note: "does not get overwritten by Demon King",
  },
  {
    id: "galactic-baseballer.demon-king.v3_3",
    game_version: "4.4",
    source_path: "docs/goals/16-galactic-baseballer-reference-data.md",
    row_locator: "profile contract: Demon King",
    evidence_sha256: sha256({
      profile: "galactic-baseballer.demon-king.v3_3",
      release: "3.3",
      retained_at: "4.4",
    }),
    evidence_quality: "ProjectPolicy",
    ownership: "DemonKing",
    selector: "released Version 3.3 profile retained independently at the Version 4.4 baseline",
    runtime_disposition: "ReferenceOnly",
    data_status: "Pending",
    note: "extends the shared base but does not replace Departure",
  },
]);
categoryBuckets.set("semantic_fixture_families", semanticFamilies.map((id) => ({
  id,
  game_version: "4.4",
  source_path: "docs/goals/16-galactic-baseballer-reference-data.md",
  row_locator: `semantic family=${id}`,
  evidence_sha256: sha256({ id, goal: "galactic-baseballer-reference-v1" }),
  evidence_quality: "ProjectPolicy",
  ownership: "SharedBase",
  selector: "non-shrinking Goal 16 semantic obligation",
  runtime_disposition: "ReferenceOnly",
  data_status: "Pending",
  note: "P0-B4 defines the rule/fixture contract; later batches must close at least one executable review fixture",
})));

const orderedCategoryIds = [
  "profiles",
  "profile_stages",
  "stage_periods",
  "weapon_collections",
  "weapon_levels",
  "weapon_types",
  "accessory_levels",
  "synthesis_materials",
  "upgrade_cards",
  "upgrade_card_types",
  "offer_box_groups",
  "offer_box_items",
  "mode_constants",
  "shop_progression",
  "content_tags",
  "tutorial_entries",
  "enemy_collection_locators",
  "reward_locators",
  "presentation_locators",
  "config_programs",
  "shared_stage_configs",
  "infinite_stage_groups",
  "infinite_waves",
  "infinite_monster_groups",
  "enemy_variants",
  "enemy_templates",
  "enemy_skills",
  "enemy_statuses",
  "semantic_fixture_families",
];
for (const categoryId of orderedCategoryIds) {
  const records = categoryBuckets.get(categoryId);
  if (records === undefined)
    throw new Error(`missing category records: ${categoryId}`);
  const evidenceOnly = ["reward_locators", "presentation_locators"]
    .includes(categoryId);
  categories[categoryId] = category(
    categoryId,
    categoryId.startsWith("shared_") || categoryId.startsWith("infinite_")
      || categoryId.startsWith("enemy_")
      ? "explicit recursive stable-ID reachability"
      : "complete exact rows from the dedicated versioned source table or fixed Goal obligation",
    records,
    evidenceOnly ? "EvidenceOnly" : "DataReady",
  );
}

const categoryCounts = Object.fromEntries(
  Object.entries(categories).map(([id, value]) => [id, value.count]),
);
const activeRecords = Object.values(categories)
  .flatMap(({ records }) => records);
const countsByOwnership = {};
for (const record of activeRecords)
  countsByOwnership[record.ownership]
    = (countsByOwnership[record.ownership] ?? 0) + 1;
const evidenceOnlyRecords = activeRecords.filter(
  ({ data_status: status }) => status === "EvidenceOnly",
);

const legacyStageReferences = [];
for (const [profile, sourcePath, rows] of [
  [
    "galactic-baseballer.departure.v2_2",
    "ExcelOutput/EvolveBuildStagePeriod.json",
    departurePeriods,
  ],
  [
    "galactic-baseballer.demon-king.v3_3",
    "ExcelOutput/EvoBdSCStagePeriod.json",
    demonPeriods,
  ],
]) {
  for (const [index, row] of rows.entries()) {
    if (!missingStageIds.includes(row.StageID)) continue;
    legacyStageReferences.push({
      ...receipt({
        id: `${profile}:missing-stage:${row.StageID}`,
        sourcePath,
        row,
        index,
        ownership: profile.includes("demon-king") ? "DemonKing" : "Departure",
        selector: "exact profile StagePeriod row whose StageID has no matching pinned StageConfig row",
        disposition: "EvidenceOnly",
        note: "the StagePeriod fact remains in the non-shrinking denominator; only its unresolved shared-stage expansion is excluded",
      }),
      replacement_condition:
        "a released pinned StageConfig row or official migration mapping identifies the missing stage",
    });
  }
}

const manifest = {
  schema_revision: "starclock.galactic-baseballer-content-manifest.v1",
  goal_id: "galactic-baseballer-reference-v1",
  snapshot: {
    game_version: "4.4",
    structured_access_date: "2026-07-22",
    source_revision: revision,
    source_inventory_sha256: sourceInventorySha256,
  },
  profiles: [
    "galactic-baseballer.departure.v2_2",
    "galactic-baseballer.demon-king.v3_3",
  ],
  ownership_policy: {
    Departure: "records sourced from the released EvolveBuild table family",
    DemonKing: "records sourced from the released EvoBdSC/SC/S2 family",
    SharedBase: "one reference-only mechanism/program obligation used by versioned profile adapters; this classification is a ProjectPolicy unless explicit structured reachability is present",
    Shared: "existing stage, wave, enemy or battle identity admitted only through an exact stable-ID field reference",
    fail_closed: "prefixes discover candidate families; names, ID ranges and similarity never prove cross-profile identity, synthesis or shared membership",
  },
  denominator_policy: {
    source_obligation: "every row in all 29 dedicated tables and every one of 35 selected programs is retained exactly once",
    shared_reachability: "shared stages, waves and enemies are the recursive closure of explicit StagePeriod and stable-ID fields",
    evidence_only: "reward and presentation rows remain counted as locators but cannot enter the simulation core",
    no_shrink: "later normalization may split an obligation but cannot delete, merge or reclassify it without a new manifest revision and evidence",
  },
  reconciliation: {
    profile_table_rows: {
      departure: tableRecords
        .filter(({ path: sourcePath }) => sourcePath.includes("/EvolveBuild"))
        .reduce((sum, record) => sum + record.json_rows, 0),
      demon_king: tableRecords
        .filter(({ path: sourcePath }) => sourcePath.includes("/EvoBdSC"))
        .reduce((sum, record) => sum + record.json_rows, 0),
    },
    dedicated_tables: tableRecords.length,
    config_programs: programRecords.length,
    explicit_shared_stage_id: 4140116,
    explicit_shared_stage_proof:
      "both versioned StagePeriod tables contain direct StageID=4140116 references",
    shared_stage_ids: [...foundStageIds].sort((left, right) => left - right),
    recursive_counts: {
      stage_configs: stageRows.length,
      infinite_stage_groups: groupRows.length,
      infinite_waves: waveRows.length,
      infinite_monster_groups: monsterGroupRows.length,
      enemy_variants: monsterRows.length,
      enemy_templates: templateRows.length,
      enemy_skills: skillRows.length,
      enemy_statuses: statusRows.length,
    },
  },
  source_augmentation: {
    relation_to_p0_b2:
      "these tables extend P0-B3 transitive reachability and do not alter the committed 81-file P0-B2 discovery inventory",
    records: tableBlobReceipts.filter(({ path: sourcePath }) =>
      [
        tablePaths.infiniteGroup,
        tablePaths.infiniteWave,
        tablePaths.infiniteMonsterGroup,
      ].includes(sourcePath)),
  },
  categories,
  exclusions_and_replacement_boundaries: {
    legacy_stage_references: legacyStageReferences,
    unresolved_enemy_effect_ids: unresolvedEffectIds.map((id) => ({
      id: String(id),
      source_path: tablePaths.monsterSkill,
      row_locator: `ExtraEffectIDList contains ${id}`,
      evidence_sha256: sha256({ id, source: tablePaths.monsterSkill }),
      evidence_quality: "ExactStructured",
      ownership: "Shared",
      runtime_disposition: "EvidenceOnly",
      reason: "referenced effect ID has no matching MonsterStatusConfig.StatusID at the pinned revision",
      replacement_condition:
        "a released pinned table maps this effect identity to an authoritative status/effect record",
    })),
    account_reward_policy:
      "reward and presentation rows are retained as EvidenceOnly locators until P2-B3 separates mechanical progression from account-only output",
  },
  counts: {
    categories: Object.keys(categories).length,
    records: activeRecords.length,
    data_ready_required: activeRecords.length - evidenceOnlyRecords.length,
    evidence_only: evidenceOnlyRecords.length,
    ownership: Object.fromEntries(
      Object.entries(countsByOwnership).sort(([left], [right]) =>
        compareText(left, right)),
    ),
    category_records: categoryCounts,
    replacement_boundaries:
      legacyStageReferences.length + unresolvedEffectIds.length,
  },
};

const encoded = `${JSON.stringify(manifest, null, 2)}\n`;
if (check) {
  const existing = await readFile(output, "utf8");
  if (existing !== encoded)
    throw new Error(`generated manifest drift: ${output}`);
} else {
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, encoded);
}

console.log(
  `Galactic Baseballer manifest ${check ? "verified" : "wrote"}: `
  + `${manifest.counts.records} records, `
  + `${manifest.counts.replacement_boundaries} replacement boundaries`,
);
