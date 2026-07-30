#!/usr/bin/env node

import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const check = process.argv.includes("--check");
const root = path.resolve(".");
const cache = path.resolve(option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/galactic-baseballer-source"));
const sourceRoot = path.join(cache, "turnbasedgamedata");
const outputRoot = path.join(
  root,
  "content-reference",
  "galactic-baseballer-v1",
);
const manifest = JSON.parse(await readFile(path.join(
  root,
  "content-manifests",
  "galactic-baseballer-v1",
  "content-manifest.json",
), "utf8"));
const publicSources = JSON.parse(await readFile(path.join(
  root,
  "content-manifests",
  "galactic-baseballer-v1",
  "public-source-inventory.json",
), "utf8"));
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
const profileId = "galactic-baseballer.departure.v2_2";
const rowRevision = "starclock.galactic-baseballer-row.v1";

function option(name) {
  const index = process.argv.indexOf(name);
  if (index === -1) return undefined;
  const value = process.argv[index + 1];
  if (value === undefined || value.startsWith("--"))
    throw new Error(`${name} requires a path`);
  return value;
}

function losslessJson(bytes) {
  return JSON.parse(bytes.toString("utf8").replace(
    /(:\s*|[\[,]\s*)(-?\d{16,})(?=\s*[,}\]])/gu,
    '$1"$2"',
  ));
}

function digest(value) {
  return createHash("sha256").update(
    Buffer.isBuffer(value) ? value : JSON.stringify(value),
  ).digest("hex");
}

function canonicalValue(value) {
  if (Array.isArray(value)) return value.map(canonicalValue);
  if (value !== null && typeof value === "object")
    return Object.fromEntries(Object.entries(value)
      .map(([key, child]) => [key, canonicalValue(child)]));
  if (typeof value === "number" && !Number.isInteger(value))
    return String(value);
  return value;
}

function manifestRecord(category, id) {
  const record = manifest.categories[category].records.find(
    ({ id: recordId }) => recordId === id,
  );
  if (record === undefined) throw new Error(`missing manifest record: ${id}`);
  return record;
}

function structuredSource(record, mechanismQuality) {
  return {
    source_id: `source.goal16.${record.evidence_sha256.slice(0, 16)}`,
    repository_or_url: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision_or_access_date: revision,
    game_version: "4.4",
    path_or_page: record.source_path,
    locator: record.row_locator,
    sha256: record.evidence_sha256,
    evidence_quality: "ExactStructured",
    mechanism_quality: mechanismQuality,
    note: "pinned released Version 4.4 structured row",
  };
}

function officialSource(id, note) {
  const source = publicSources.official_pages.find(({ id: sourceId }) =>
    sourceId === id);
  if (source === undefined) throw new Error(`missing official source: ${id}`);
  return {
    source_id: `source.goal16.${id}`,
    repository_or_url: source.url,
    revision_or_access_date: publicSources.access_date,
    game_version: "4.4",
    path_or_page: source.role,
    locator: "publisher page identity and released-version statement",
    sha256: digest(source),
    evidence_quality: "ExactPublicText",
    mechanism_quality: "IdentityCrossCheck",
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
  manifestIds,
  sourceRefs,
  tags,
}) {
  return {
    id,
    schema_revision: rowRevision,
    kind,
    name_en: nameEn,
    name_zh_cn: nameZh,
    summary_en: summaryEn,
    summary_zh_cn: summaryZh,
    profile_ids: [profileId],
    ownership: "Departure",
    coverage_state: "Researched",
    evidence_quality: sourceRefs.some(({ evidence_quality: quality }) =>
      quality === "ExactStructured") ? "ExactStructured" : "ExactPublicText",
    mechanism_quality: "ExactRelationship",
    manifest_record_ids: manifestIds,
    source_refs: sourceRefs,
    tags: [...tags].sort(),
  };
}

const readSource = async (relativePath) =>
  losslessJson(await readFile(path.join(sourceRoot, relativePath)));
const stagePath = "ExcelOutput/EvolveBuildStageConfig.json";
const periodPath = "ExcelOutput/EvolveBuildStagePeriod.json";
const constantPath = "ExcelOutput/EvolveBuildConstValueCommon.json";
const stagesSource = await readSource(stagePath);
const periodsSource = await readSource(periodPath);
const constants = await readSource(constantPath);
const chs = await readSource("TextMap/TextMapCHS.json");
const en = await readSource("TextMap/TextMapEN.json");

const profileManifest = manifestRecord("profiles", profileId);
const unlockConstantIndex = constants.findIndex(({ ConstValueName }) =>
  ConstValueName === "EvolveBuild_EarlyAccess_UnlockQuest");
if (unlockConstantIndex === -1) throw new Error("Departure unlock constant missing");
const unlockConstant = constants[unlockConstantIndex];
const unlockManifestId =
  `${profileId}:EvolveBuildConstValueCommon:${String(unlockConstantIndex).padStart(4, "0")}`;
const unlockManifest = manifestRecord("mode_constants", unlockManifestId);
const profiles = [{
  ...envelope({
    id: profileId,
    kind: "Profile",
    nameEn: "Legend of the Galactic Baseballer: Departure",
    nameZh: "银河球棒侠传说：启程篇",
    summaryEn:
      "Independent Version 2.2 rules profile retained at the Version 4.4 baseline over the shared Galactic Baseballer system.",
    summaryZh:
      "在 Version 4.4 基线上保留的独立 Version 2.2 规则 Profile，共用银河球棒侠基础系统。",
    manifestIds: [profileId, unlockManifestId].sort(),
    sourceRefs: [
      structuredSource(profileManifest, "PolicyBoundary"),
      structuredSource(unlockManifest, "ExactRelationship"),
      officialSource(
        "hoyolab-version-2.2-update",
        "publisher-operated released Version 2.2 identity and entry cross-check",
      ),
      officialSource(
        "hoyolab-original-event-notice",
        "publisher-operated original event rules/window cross-check",
      ),
    ],
    tags: ["departure", "profile", "version-2.2"],
  }),
  released_version: "2.2",
  retained_baseline_version: "4.4",
  source_season: "EarlyAccess",
  shared_system_id: "galactic-baseballer.shared-base.v1",
  activity_module_id: "5003501",
  entry_unlock_quest_id: String(unlockConstant.Value.IntValue),
  availability: "PermanentRetainedProfile",
  runtime_enabled: false,
}];

const releaseBoundaries = [
  {
    ...envelope({
      id: "galactic-baseballer.release-boundary.departure-gameplay",
      kind: "ReleaseBoundary",
      nameEn: "Departure gameplay retention",
      nameZh: "启程篇玩法保留",
      summaryEn:
        "The released Departure mechanics remain an independently selectable reference profile at the 4.4 baseline.",
      summaryZh:
        "已发布的启程篇机制在 4.4 基线上继续作为可独立选择的资料 Profile。",
      manifestIds: [profileId],
      sourceRefs: [
        structuredSource(profileManifest, "PolicyBoundary"),
        officialSource(
          "hoyowiki-original-entry-2508",
          "publisher-operated released mechanic entry cross-check",
        ),
      ],
      tags: ["gameplay", "permanent-retention", "release-boundary"],
    }),
    release_order: 1,
    surface: "MechanicalGameplay",
    disposition: "ReferenceOnlyPermanent",
    account_projection: false,
  },
  {
    ...envelope({
      id: "galactic-baseballer.release-boundary.departure-limited-rewards",
      kind: "ReleaseBoundary",
      nameEn: "Departure limited account rewards",
      nameZh: "启程篇限时账号奖励",
      summaryEn:
        "Limited account rewards remain locators only and do not enter gameplay simulation data.",
      summaryZh:
        "限时账号奖励仅保留定位信息，不进入玩法模拟资料。",
      manifestIds: [profileId],
      sourceRefs: [
        structuredSource(profileManifest, "PolicyBoundary"),
        officialSource(
          "hoyolab-original-event-notice",
          "publisher-operated original event-window and reward-surface locator",
        ),
      ],
      tags: ["account-only", "evidence-only", "release-boundary"],
    }),
    release_order: 2,
    surface: "LimitedAccountRewards",
    disposition: "EvidenceOnly",
    account_projection: false,
  },
];

const parentStageByPeriod = new Map();
const resolvedSharedStageIds = new Set(
  manifest.categories.shared_stage_configs.records.map(({ id }) =>
    id.slice(id.indexOf(":") + 1)),
);
for (const row of stagesSource) {
  for (const field of [
    "StagePeriod1",
    "StagePeriod2",
    "StagePeriod3",
    "StagePeriod4",
  ]) {
    for (const periodId of row[field])
      parentStageByPeriod.set(String(periodId), String(row.StageMergedID));
  }
}

const stages = stagesSource.map((row, index) => {
  const manifestId =
    `${profileId}:EvolveBuildStageConfig:${String(index).padStart(4, "0")}`;
  const record = manifestRecord("profile_stages", manifestId);
  const nameHash = String(row.Name.Hash);
  const stageId = String(row.StageMergedID);
  const nameEn = en[nameHash];
  const nameZh = chs[nameHash];
  if (typeof nameEn !== "string" || typeof nameZh !== "string")
    throw new Error(`stage localization missing: ${stageId}/${nameHash}`);
  return {
    ...envelope({
      id: `galactic-baseballer.departure.stage.${stageId}`,
      kind: "Stage",
      nameEn,
      nameZh,
      summaryEn:
        `Departure stage ${stageId}, difficulty ${row.Difficulty ?? 0}, with exact initial loadout, team bonus, recommendations and rating thresholds.`,
      summaryZh:
        `启程篇关卡 ${stageId}，难度 ${row.Difficulty ?? 0}；保留精确初始配置、队伍加成、推荐组合与评级阈值。`,
      manifestIds: [manifestId],
      sourceRefs: [structuredSource(record, "ExactRelationship")],
      tags: ["departure", "stage"],
    }),
    source_numeric_id: stageId,
    source_name_hash: nameHash,
    intro_id: String(row.IntroID),
    season: row.Season,
    difficulty: row.Difficulty ?? 0,
    weapon_selectable: row.WeaponSelectable ?? false,
    unlock_quest_id: row.UnlockQuest === undefined
      ? undefined
      : String(row.UnlockQuest),
    team_bonus_maze_buff_id: String(row.TeamBonusMazeBuffID),
    team_bonus_short_desc_hash: String(row.TeamBonusShortDesc.Hash),
    team_bonus_format_hash: String(row.BuffTextFormat.Hash),
    period_ids_by_phase: [
      row.StagePeriod1,
      row.StagePeriod2,
      row.StagePeriod3,
      row.StagePeriod4,
    ].map((ids) => ids.map(String)),
    initial_weapon_ids: row.InitialWeapon.map(String),
    trial_avatar_ids: row.TrialAvatar.map(String),
    recommended_weapon_levels: row.RecommendList.map((entry, ordinal) => ({
      ordinal,
      weapon_id: String(entry.BOKJJKFCFME),
      level: entry.AAGKEBFHLMC,
    })),
    recommended_accessory_ids: row.GearRecommendList.map(String),
    rating_thresholds: row.RankList.map((entry, ordinal) => ({
      ordinal,
      rating: entry.OENAMINOLLF,
      minimum_score: entry.MNAKIEOGPDK ?? 0,
    })),
  };
});

const stagePeriods = periodsSource.map((row, index) => {
  const manifestId =
    `${profileId}:EvolveBuildStagePeriod:${String(index).padStart(4, "0")}`;
  const record = manifestRecord("stage_periods", manifestId);
  const id = String(row.StagePeriodID);
  const parent = parentStageByPeriod.get(id);
  const unresolvedSharedStage = !resolvedSharedStageIds.has(String(row.StageID));
  return {
    ...envelope({
      id: `galactic-baseballer.departure.stage-period.${id}`,
      kind: "StagePeriod",
      nameEn: `Departure stage period ${id}`,
      nameZh: `启程篇关卡阶段 ${id}`,
      summaryEn:
        `Ordered Departure stage-period definition ${id} with exact wave, timer, weakness and score parameters.`,
      summaryZh:
        `启程篇有序关卡阶段定义 ${id}，保留精确波次、计时、弱点与分数参数。`,
      manifestIds: [manifestId],
      sourceRefs: [structuredSource(record, "ExactRelationship")],
      tags: [
        "departure",
        unresolvedSharedStage ? "legacy-stage-reference" : "stage-period",
      ],
    }),
    source_numeric_id: id,
    parent_stage_id: parent === undefined
      ? undefined
      : `galactic-baseballer.departure.stage.${parent}`,
    shared_stage_config_id: String(row.StageID),
    battle_event_id: String(row.EventID),
    period_rank: row.PeriodRank,
    wave_count: row.WaveCount,
    countdown_by_wave: row.CountdownList,
    weakness_order: row.WeaknessList.map((entry, ordinal) => ({
      ordinal,
      damage_type: entry.GIBLLBPOJHN,
      preferred: entry.NBPFHHJJBFG ?? false,
    })),
    period_score: row.PeriodScore,
    emotion_thresholds: row.EmotionList.map(String),
    battle_area_id: String(row.BattleArea),
    deadline_position: row.DeadLinePosition === undefined
      ? undefined
      : String(row.DeadLinePosition.Value),
    stage_score: row.StageScore,
    selection_weight: row.Weight,
    special_monster_scores: canonicalValue(row.SpecialMonsterScoreList),
    unresolved_shared_stage: unresolvedSharedStage,
  };
});

for (const rows of [profiles, releaseBoundaries, stages, stagePeriods]) {
  rows.sort((left, right) => left.id.localeCompare(right.id, "en"));
}
const outputs = new Map([
  ["profiles.json", profiles],
  ["release-boundaries.json", releaseBoundaries],
  ["stages.json", stages],
  ["stage-periods.json", stagePeriods],
]);
await mkdir(outputRoot, { recursive: true });
for (const [file, value] of outputs) {
  const target = path.join(outputRoot, file);
  const encoded = `${JSON.stringify(value, null, 2)}\n`;
  if (check) {
    const existing = await readFile(target, "utf8");
    if (existing !== encoded)
      throw new Error(`Departure normalization drift: ${target}`);
  } else {
    await writeFile(target, encoded);
  }
}

console.log(
  `Departure normalization ${check ? "verified" : "wrote"}: `
  + `${profiles.length} profile, ${releaseBoundaries.length} boundaries, `
  + `${stages.length} stages, ${stagePeriods.length} periods`,
);
