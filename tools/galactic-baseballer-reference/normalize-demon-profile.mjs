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
  "fragments",
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
const profileId = "galactic-baseballer.demon-king.v3_3";
const departureProfileId = "galactic-baseballer.departure.v2_2";
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

function structuredSource(record, mechanismQuality, note =
  "pinned released Version 4.4 structured row") {
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
    note,
  };
}

function officialSource(id, locator, note) {
  const source = publicSources.official_pages.find(({ id: sourceId }) =>
    sourceId === id);
  if (source === undefined) throw new Error(`missing official source: ${id}`);
  return {
    source_id: `source.goal16.${id}.${digest(locator).slice(0, 12)}`,
    repository_or_url: source.url,
    revision_or_access_date: publicSources.access_date,
    game_version: "4.4",
    path_or_page: source.role,
    locator,
    sha256: digest({ source, locator, note }),
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
  mechanismQuality = "ExactRelationship",
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
    ownership: "DemonKing",
    coverage_state: "Researched",
    evidence_quality: sourceRefs.some(({ evidence_quality: quality }) =>
      quality === "ExactStructured") ? "ExactStructured" : "ExactPublicText",
    mechanism_quality: mechanismQuality,
    manifest_record_ids: [...manifestIds].sort(),
    source_refs: sourceRefs,
    tags: [...tags].sort(),
  };
}

function cleanName(value) {
  return value.replaceAll("<unbreak>", "").replaceAll("</unbreak>", "");
}

function comparableName(value, prefix) {
  return value.ConstValueName.replace(prefix, "");
}

const readSource = async (relativePath) =>
  losslessJson(await readFile(path.join(sourceRoot, relativePath)));
const demonStagePath = "ExcelOutput/EvoBdSCStageConfig.json";
const demonPeriodPath = "ExcelOutput/EvoBdSCStagePeriod.json";
const demonConstantPath = "ExcelOutput/EvoBdSCConstValueCommon.json";
const departureConstantPath = "ExcelOutput/EvolveBuildConstValueCommon.json";
const stagesSource = await readSource(demonStagePath);
const periodsSource = await readSource(demonPeriodPath);
const demonConstants = await readSource(demonConstantPath);
const departureConstants = await readSource(departureConstantPath);
const chs = await readSource("TextMap/TextMapCHS.json");
const en = await readSource("TextMap/TextMapEN.json");

const profileManifest = manifestRecord("profiles", profileId);
const constantIndex = new Map(demonConstants.map((row, index) => [
  row.ConstValueName,
  { index, row },
]));
function demonConstant(name) {
  const found = constantIndex.get(`EvolveBuildSC_${name}`);
  if (found === undefined) throw new Error(`missing Demon King constant: ${name}`);
  const manifestId =
    `${profileId}:EvoBdSCConstValueCommon:${String(found.index).padStart(4, "0")}`;
  return {
    ...found,
    manifestId,
    manifest: manifestRecord("mode_constants", manifestId),
  };
}

const activityModule = demonConstant("Activity_Module_ID");
const originStage = demonConstant("OriginStage_ID");
const shopUnlock = demonConstant("Shop_UnlockQuest");
const rewardUnlock = demonConstant("Reward_UnlockQuest");
const skipOriginUnlock = demonConstant("SkipOriginStage_UnlockQuest");
const profileConstantRows = [
  activityModule,
  originStage,
  shopUnlock,
  rewardUnlock,
  skipOriginUnlock,
];
const profile = [{
  ...envelope({
    id: profileId,
    kind: "Profile",
    nameEn: "Legend of the Galactic Baseballer: Demon King",
    nameZh: "银河球棒侠传说：魔王篇",
    summaryEn:
      "Independent Version 3.3 rules profile retained at the Version 4.4 baseline over the shared Galactic Baseballer system; it does not replace Departure.",
    summaryZh:
      "在 Version 4.4 基线上保留的独立 Version 3.3 规则 Profile；共用银河球棒侠基础系统，但不覆盖启程篇。",
    manifestIds: [
      profileId,
      ...profileConstantRows.map(({ manifestId }) => manifestId),
    ],
    sourceRefs: [
      structuredSource(profileManifest, "PolicyBoundary"),
      ...profileConstantRows.map(({ manifest }) =>
        structuredSource(manifest, "ExactRelationship")),
      officialSource(
        "hoyolab-version-3.3-update",
        "Version 3.3 update / New Events / Legend of the Galactic Baseballer: Demon King",
        "released Version 3.3 identity, event window, Trailblaze Level 21 requirement and Finality's Vision entry",
      ),
    ],
    tags: ["demon-king", "profile", "version-3.3"],
    mechanismQuality: "PolicyBoundary",
  }),
  released_version: "3.3",
  retained_baseline_version: "4.4",
  source_season: "SecondChapter",
  shared_system_id: "galactic-baseballer.shared-base.v1",
  does_not_replace_profile_ids: [departureProfileId],
  activity_module_id: String(activityModule.row.Value.IntValue),
  origin_stage_numeric_id: String(originStage.row.Value.IntValue),
  released_entry_requirement: {
    minimum_trailblaze_level: 21,
    finalitys_vision_early_access: true,
  },
  structured_unlock_quest_locators: {
    reward_unlock_quest_id: String(rewardUnlock.row.Value.IntValue),
    shop_unlock_quest_id: String(shopUnlock.row.Value.IntValue),
    skip_origin_stage_unlock_quest_id:
      String(skipOriginUnlock.row.Value.IntValue),
  },
  availability: "PermanentRetainedProfile",
  runtime_enabled: false,
}];

const releaseBoundaries = [
  {
    ...envelope({
      id: "galactic-baseballer.release-boundary.demon-king-gameplay",
      kind: "ReleaseBoundary",
      nameEn: "Demon King gameplay retention",
      nameZh: "魔王篇玩法保留",
      summaryEn:
        "Released Demon King mechanics remain an independent reference profile at the 4.4 baseline.",
      summaryZh:
        "已发布的魔王篇机制在 4.4 基线上继续作为独立资料 Profile。",
      manifestIds: [profileId],
      sourceRefs: [
        structuredSource(profileManifest, "PolicyBoundary"),
        officialSource(
          "hoyolab-version-3.3-update",
          "Version 3.3 update / event release and limited-time window",
          "released mechanic identity and original event-window cross-check",
        ),
      ],
      tags: ["gameplay", "permanent-retention", "release-boundary"],
      mechanismQuality: "PolicyBoundary",
    }),
    release_order: 1,
    surface: "MechanicalGameplay",
    disposition: "ReferenceOnlyPermanent",
    account_projection: false,
  },
  {
    ...envelope({
      id: "galactic-baseballer.release-boundary.demon-king-limited-rewards",
      kind: "ReleaseBoundary",
      nameEn: "Demon King limited account rewards",
      nameZh: "魔王篇限时账号奖励",
      summaryEn:
        "The original event window and account rewards remain evidence locators only.",
      summaryZh:
        "原始活动窗口与账号奖励仅作为证据定位保留。",
      manifestIds: [profileId],
      sourceRefs: [
        officialSource(
          "hoyolab-version-3.3-update",
          "Event Period: after Version 3.3 update through 2025-06-30 03:59 server time",
          "limited event-window and reward-surface locator",
        ),
      ],
      tags: ["account-only", "evidence-only", "release-boundary"],
      mechanismQuality: "PolicyBoundary",
    }),
    release_order: 2,
    surface: "LimitedAccountRewards",
    disposition: "EvidenceOnly",
    account_projection: false,
  },
  {
    ...envelope({
      id: "galactic-baseballer.release-boundary.demon-king-v3_4-corrections",
      kind: "ReleaseBoundary",
      nameEn: "Demon King released Version 3.4 corrections",
      nameZh: "魔王篇 Version 3.4 已发布修正",
      summaryEn:
        "The Version 4.4 reference baseline uses the released post-3.4 mechanical state.",
      summaryZh:
        "Version 4.4 资料基线采用 3.4 正式修正后的机械状态。",
      manifestIds: [profileId],
      sourceRefs: [
        officialSource(
          "hoyolab-version-3.4-update",
          "Gameplay fixes / Demon King: RuinBot Lv7-Lv8 and D007 Adventure Score",
          "publisher-operated released correction statement",
        ),
      ],
      tags: ["correction", "release-boundary", "version-3.4"],
      mechanismQuality: "PolicyBoundary",
    }),
    release_order: 3,
    surface: "ReleasedMechanicalCorrections",
    disposition: "ReferenceOnlyRetainedCorrection",
    account_projection: false,
  },
];

const parentStageByPeriod = new Map();
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
const resolvedSharedStageIds = new Set(
  manifest.categories.shared_stage_configs.records.map(({ id }) =>
    id.slice(id.indexOf(":") + 1)),
);

const stages = stagesSource.map((row, index) => {
  const manifestId =
    `${profileId}:EvoBdSCStageConfig:${String(index).padStart(4, "0")}`;
  const record = manifestRecord("profile_stages", manifestId);
  const stageId = String(row.StageMergedID);
  const nameHash = String(row.Name.Hash);
  const nameEn = en[nameHash];
  const nameZh = chs[nameHash];
  if (typeof nameEn !== "string" || typeof nameZh !== "string")
    throw new Error(`Demon King stage localization missing: ${stageId}`);
  return {
    ...envelope({
      id: `galactic-baseballer.demon-king.stage.${stageId}`,
      kind: "Stage",
      nameEn: cleanName(nameEn),
      nameZh: cleanName(nameZh),
      summaryEn:
        `Demon King ${stageId === "424000" ? "origin" : "challenge"} stage ${stageId} with exact loadout, team bonus, recommendations and rating thresholds.`,
      summaryZh:
        `魔王篇${stageId === "424000" ? "初始" : "挑战"}关卡 ${stageId}，保留精确初始配置、队伍加成、推荐组合与评级阈值。`,
      manifestIds: [manifestId],
      sourceRefs: [structuredSource(record, "ExactRelationship")],
      tags: ["demon-king", stageId === "424000" ? "origin-stage" : "stage"],
    }),
    source_numeric_id: stageId,
    source_name_hash: nameHash,
    stage_role: stageId === "424000" ? "Origin" : "Challenge",
    intro_id: String(row.IntroID),
    season: row.Season,
    difficulty: row.Difficulty ?? 0,
    weapon_selectable: row.WeaponSelectable ?? false,
    first_win_quest_ids: row.FirstWinQuest.map(String),
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
    `${profileId}:EvoBdSCStagePeriod:${String(index).padStart(4, "0")}`;
  const record = manifestRecord("stage_periods", manifestId);
  const id = String(row.StagePeriodID);
  const parent = parentStageByPeriod.get(id);
  const unresolvedSharedStage = !resolvedSharedStageIds.has(String(row.StageID));
  return {
    ...envelope({
      id: `galactic-baseballer.demon-king.stage-period.${id}`,
      kind: "StagePeriod",
      nameEn: `Demon King stage period ${id}`,
      nameZh: `魔王篇关卡阶段 ${id}`,
      summaryEn:
        `Ordered Demon King stage-period definition ${id} with exact wave, timer, weakness and score parameters.`,
      summaryZh:
        `魔王篇有序关卡阶段定义 ${id}，保留精确波次、计时、弱点与分数参数。`,
      manifestIds: [manifestId],
      sourceRefs: [structuredSource(record, "ExactRelationship")],
      tags: ["demon-king", "stage-period"],
    }),
    source_numeric_id: id,
    parent_stage_id: parent === undefined
      ? undefined
      : `galactic-baseballer.demon-king.stage.${parent}`,
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

const departureByName = new Map(departureConstants.map((row, index) => [
  comparableName(row, /^EvolveBuild_/u),
  { index, row },
]));
const demonByName = new Map(demonConstants.map((row, index) => [
  comparableName(row, /^EvolveBuildSC_/u),
  { index, row },
]));
const comparableNames = [...new Set([
  ...departureByName.keys(),
  ...demonByName.keys(),
])].sort((left, right) => left.localeCompare(right, "en"));
const constantComparisons = comparableNames.map((name, ordinal) => {
  const departure = departureByName.get(name);
  const demon = demonByName.get(name);
  let relationship;
  if (departure === undefined) relationship = "DemonKingAdded";
  else if (demon === undefined) relationship = "DepartureOnlyNotInherited";
  else if (JSON.stringify(canonicalValue(departure.row.Value))
    === JSON.stringify(canonicalValue(demon.row.Value)))
    relationship = "SharedValueExplicitlyRepeated";
  else relationship = "DemonKingChanged";
  return {
    ordinal,
    normalized_name: name,
    relationship,
    departure_source: departure === undefined ? undefined : {
      path: departureConstantPath,
      row: departure.index,
      const_value_name: departure.row.ConstValueName,
      value: canonicalValue(departure.row.Value),
    },
    demon_king_source: demon === undefined ? undefined : {
      path: demonConstantPath,
      row: demon.index,
      const_value_name: demon.row.ConstValueName,
      value: canonicalValue(demon.row.Value),
    },
  };
});
const relationshipCounts = Object.fromEntries([
  "SharedValueExplicitlyRepeated",
  "DemonKingChanged",
  "DemonKingAdded",
  "DepartureOnlyNotInherited",
].map((relationship) => [
  relationship,
  constantComparisons.filter((row) => row.relationship === relationship).length,
]));
const editionDifferences = [{
  ...envelope({
    id: "galactic-baseballer.demon-king.edition-difference-index",
    kind: "EditionDifferenceIndex",
    nameEn: "Demon King edition difference index",
    nameZh: "魔王篇版本差异索引",
    summaryEn:
      "Lossless field-name comparison of all Departure and Demon King constants; matching names do not imply shared record identity.",
    summaryZh:
      "对启程篇与魔王篇全部常量进行无损字段名对账；同名不代表共享记录身份。",
    manifestIds: [profileId],
    sourceRefs: [
      structuredSource(profileManifest, "PolicyBoundary"),
    ],
    tags: ["difference-index", "demon-king", "shared-base"],
    mechanismQuality: "PolicyBoundary",
  }),
  comparison_key_policy:
    "Strip only the exact EvolveBuild_ or EvolveBuildSC_ prefix; never infer identity from IDs, names or values.",
  stage_identity_policy:
    "No cross-profile stage aliases are admitted; only exact shared StageConfig references from StagePeriod rows are shared.",
  relationship_counts: relationshipCounts,
  constant_comparisons: constantComparisons,
  later_detail_owners: {
    arsenal_and_synthesis: "G16-P2-B2",
    progression_and_store: "G16-P2-B3",
    encounters_and_score: "G16-P2-B4",
  },
}];

const corrections = [
  {
    id: "galactic-baseballer.correction.v3_4.ruinbot-level-7-8",
    target_family: "WeaponLevel",
    released_statement:
      "Version 3.4 fixed incorrect level 7 and level 8 ability effects for RuinBot.",
    retained_baseline_policy:
      "Use only the pinned Version 4.4 structured level rows as corrected authoritative facts.",
    unknown_fields: [
      "the pre-fix level 7 value",
      "the pre-fix level 8 value",
      "the exact correction delta",
    ],
    rejected_alternatives: [
      "infer the erroneous values from adjacent levels",
      "treat an unpinned Version 3.3 community capture as authoritative",
    ],
    rationale:
      "The publisher identifies the affected levels but does not publish the erroneous values; reconstructing them would misstate released evidence.",
    affected_fixtures: [
      "fixture.galactic-baseballer.demon-king.weapon-ruinbot-level-7",
      "fixture.galactic-baseballer.demon-king.weapon-ruinbot-level-8",
    ],
    field_confidence: {
      retained_v4_4_state: "High",
      pre_fix_values: "Unknown",
    },
    replacement_condition:
      "Replace only if a pinned released structured Version 3.3 source exposes the exact pre-fix rows.",
    disposition: "ReferenceOnlyReleasedCorrection",
  },
  {
    id: "galactic-baseballer.correction.v3_4.d007-adventure-score",
    target_family: "StageScore",
    released_statement:
      "Version 3.4 fixed abnormal Adventure Score acquisition under specific conditions on D007 - Blissdream Planet.",
    retained_baseline_policy:
      "Use only pinned Version 4.4 D007 stage, period and scoring rows; do not reproduce the obsolete abnormal path.",
    unknown_fields: [
      "the triggering conditions",
      "the obsolete score mutation",
      "the pre-fix failure ordering",
    ],
    rejected_alternatives: [
      "model the obsolete bug as an optional scoring rule",
      "guess the trigger from the D007 team bonus or enemy composition",
    ],
    rationale:
      "The publisher confirms the correction but does not disclose the erroneous path; the retained reference profile targets post-fix behavior.",
    affected_fixtures: [
      "fixture.galactic-baseballer.demon-king.d007-score",
      "fixture.galactic-baseballer.score-rating-clear",
    ],
    field_confidence: {
      retained_v4_4_state: "High",
      pre_fix_trigger_and_delta: "Unknown",
    },
    replacement_condition:
      "Replace only if a released reproducible pre-fix trace and exact score delta become available.",
    disposition: "ReferenceOnlyReleasedCorrection",
  },
  {
    id: "galactic-baseballer.correction.v3_4.boothill-ultimate-visual",
    target_family: "Presentation",
    released_statement:
      "Version 3.4 fixed Boothill Ultimate visual effects against the Black Cloak Demon King.",
    retained_baseline_policy:
      "Keep the correction as an EvidenceOnly locator because it has no published mechanical effect.",
    unknown_fields: [],
    rejected_alternatives: [
      "treat the visual correction as a damage-rule change",
      "import visual-effect assets into the simulation data",
    ],
    rationale:
      "The released statement is explicitly visual and outside the mechanically relevant reference scope.",
    affected_fixtures: [],
    field_confidence: {
      non_mechanical_disposition: "High",
    },
    replacement_condition:
      "Reclassify only if released evidence proves a mechanically observable effect.",
    disposition: "EvidenceOnly",
  },
].map((correction, ordinal) => ({
  ...envelope({
    id: correction.id,
    kind: "ReleasedCorrection",
    nameEn: `Released correction ${ordinal + 1}`,
    nameZh: `已发布修正 ${ordinal + 1}`,
    summaryEn: correction.released_statement,
    summaryZh:
      "保留正式发布修正的机械边界，并以 Version 4.4 结构化事实作为当前权威状态。",
    manifestIds: [profileId],
    sourceRefs: [
      officialSource(
        "hoyolab-version-3.4-update",
        `Gameplay fixes / ${correction.target_family}`,
        correction.released_statement,
      ),
    ],
    tags: ["correction", "demon-king", "version-3.4"],
    mechanismQuality: correction.disposition === "EvidenceOnly"
      ? "IdentityCrossCheck"
      : "PolicyBoundary",
  }),
  correction_ordinal: ordinal,
  ...correction,
}));

for (const rows of [
  profile,
  releaseBoundaries,
  stages,
  stagePeriods,
  editionDifferences,
  corrections,
]) rows.sort((left, right) => left.id.localeCompare(right.id, "en"));
const outputs = new Map([
  ["demon-profile.json", profile],
  ["demon-release-boundaries.json", releaseBoundaries],
  ["demon-stages.json", stages],
  ["demon-stage-periods.json", stagePeriods],
  ["demon-edition-differences.json", editionDifferences],
  ["demon-released-corrections.json", corrections],
]);
await mkdir(outputRoot, { recursive: true });
for (const [file, value] of outputs) {
  const target = path.join(outputRoot, file);
  const encoded = `${JSON.stringify(value, null, 2)}\n`;
  if (check) {
    const existing = await readFile(target, "utf8");
    if (existing !== encoded)
      throw new Error(`Demon King profile normalization drift: ${target}`);
  } else {
    await writeFile(target, encoded);
  }
}

console.log(
  `Demon King profile normalization ${check ? "verified" : "wrote"}: `
  + `${profile.length} profile, ${releaseBoundaries.length} boundaries, `
  + `${stages.length} stage rows (6 challenges), ${stagePeriods.length} periods, `
  + `${constantComparisons.length} constant comparisons, `
  + `${corrections.length} released corrections`,
);
