#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile, writeFile, mkdir } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(".");
const sourceCache = path.resolve(option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/content-reference"));
const fallbackValue = option("--fallback-source-cache")
  ?? process.env.STARCLOCK_FALLBACK_SOURCE_CACHE;
const fallbackCache = fallbackValue === undefined
  ? undefined
  : path.resolve(fallbackValue);
const sourceRoot = path.join(sourceCache, "turnbasedgamedata");
const fallbackRoot = fallbackCache === undefined
  ? undefined
  : path.join(fallbackCache, "turnbasedgamedata");
const outputRoot = path.join(
  root,
  "content-reference/anomaly-arbitration-v1",
);
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
const manifest = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/content-manifest.json",
), "utf8"));

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

function canonical(value) {
  if (Array.isArray(value))
    return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value).sort(compareText).map((key) =>
      `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function digest(value) {
  return createHash("sha256").update(
    Buffer.isBuffer(value) ? value : canonical(value),
  ).digest("hex");
}

function git(gitArgs, options = {}) {
  return execFileSync("git", ["-C", sourceRoot, ...gitArgs], {
    encoding: "utf8",
    maxBuffer: 512 * 1024 * 1024,
    env: {
      ...process.env,
      GIT_NO_LAZY_FETCH: "1",
      ...(fallbackRoot === undefined
        ? {}
        : {
          GIT_ALTERNATE_OBJECT_DIRECTORIES:
            path.join(fallbackRoot, ".git", "objects"),
        }),
    },
    ...options,
  });
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function assertCache(repositoryRoot, label) {
  const actual = execFileSync(
    "git",
    ["-C", repositoryRoot, "rev-parse", "HEAD"],
    { encoding: "utf8" },
  ).trim();
  assert(actual === revision, `${label} revision drift: ${actual}`);
  const dirty = execFileSync(
    "git",
    ["-C", repositoryRoot, "status", "--porcelain"],
    { encoding: "utf8" },
  ).trim();
  assert(!dirty, `${label} source cache is dirty`);
}

function losslessJson(bytes) {
  const text = bytes.toString("utf8").replace(
    /(:\s*|[\[,]\s*)(-?\d{16,})(?=\s*[,}\]])/gu,
    '$1"$2"',
  );
  return JSON.parse(text);
}

async function sourceBytes(relativePath) {
  try {
    return await readFile(path.join(sourceRoot, relativePath));
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
    return git(["cat-file", "blob", `HEAD:${relativePath}`], {
      encoding: null,
    });
  }
}

function manifestRecord(category, id) {
  const record = manifest.categories[category].records.find(
    (candidate) => candidate.id === id,
  );
  assert(record !== undefined, `missing manifest record ${category}:${id}`);
  return record;
}

function sourceRef(category, id, note) {
  const record = manifestRecord(category, id);
  return {
    source_id: `turnbasedgamedata:${record.source_path}:${record.row_locator}`,
    repository_or_url:
      "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision_or_access_date: revision,
    game_version: "4.4",
    path_or_page: record.source_path,
    locator: record.row_locator,
    sha256: record.evidence_sha256,
    evidence_quality: record.evidence_quality,
    mechanism_quality: "ExactRelationship",
    note,
  };
}

function textRef(locale, hash, value, note) {
  const sourcePath = locale === "zh_cn"
    ? "TextMap/TextMapCHS.json"
    : "TextMap/TextMapEN.json";
  return {
    source_id: `turnbasedgamedata:${sourcePath}:Hash=${hash}`,
    repository_or_url:
      "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision_or_access_date: revision,
    game_version: "4.4",
    path_or_page: sourcePath,
    locator: `Hash=${hash}`,
    sha256: digest({ hash, value }),
    evidence_quality: "ExactStructured",
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
  ownership = "Shared",
  manifestIds,
  sources,
  tags,
  fields,
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
    evidence_quality: "ExactStructured",
    mechanism_quality: "ExactRelationship",
    manifest_record_ids: [...manifestIds].sort(compareText),
    source_refs: sources,
    tags: [...tags].sort(compareText),
    ...fields,
    runtime_executable: false,
  };
}

function decimal(value) {
  if (value === undefined || value === null) return null;
  const raw = typeof value === "object" && "Value" in value
    ? value.Value
    : value;
  return String(raw);
}

function values(rows, field) {
  return rows.map((row) => row[field]).filter((value) => value !== undefined);
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

assertCache(sourceRoot, "primary");
if (fallbackRoot !== undefined) assertCache(fallbackRoot, "fallback");
const tablePaths = {
  stage: "ExcelOutput/StageConfig.json",
  monster: "ExcelOutput/MonsterConfig.json",
  template: "ExcelOutput/MonsterTemplateConfig.json",
  skill: "ExcelOutput/MonsterSkillConfig.json",
  status: "ExcelOutput/MonsterStatusConfig.json",
  textEn: "TextMap/TextMapEN.json",
  textZh: "TextMap/TextMapCHS.json",
};
const parsed = {};
for (const [key, relativePath] of Object.entries(tablePaths))
  parsed[key] = losslessJson(await sourceBytes(relativePath));
const stagesDocument = JSON.parse(await readFile(path.join(
  outputRoot,
  "stages.json",
), "utf8"));
const normalizedStageBySource = new Map(stagesDocument.records.map((record) =>
  [String(record.source_stage_id), record]));

const stageIds = manifest.categories.stage_configs.records.map(
  ({ id }) => id.split(":")[1],
);
const stageRows = parsed.stage.filter(
  ({ StageID }) => stageIds.includes(String(StageID)),
);
const monsterIds = new Set(manifest.categories.enemy_variants.records.map(
  ({ id }) => Number(id.split(":")[1]),
));
const templateIds = new Set(manifest.categories.enemy_templates.records.map(
  ({ id }) => Number(id.split(":")[1]),
));
const skillIds = new Set(manifest.categories.enemy_skills.records.map(
  ({ id }) => Number(id.split(":")[1]),
));
const statusIds = new Set(manifest.categories.enemy_statuses.records.map(
  ({ id }) => Number(id.split(":")[1]),
));
const monsters = parsed.monster.filter(({ MonsterID }) =>
  monsterIds.has(MonsterID)).sort((left, right) =>
  left.MonsterID - right.MonsterID);
const templates = parsed.template.filter(({ MonsterTemplateID }) =>
  templateIds.has(MonsterTemplateID)).sort((left, right) =>
  left.MonsterTemplateID - right.MonsterTemplateID);
const skills = parsed.skill.filter(({ SkillID }) =>
  skillIds.has(SkillID)).sort((left, right) => left.SkillID - right.SkillID);
const statuses = parsed.status.filter(({ StatusID }) =>
  statusIds.has(StatusID)).sort((left, right) =>
  left.StatusID - right.StatusID);
assert(stageRows.length === 5 && monsters.length === 27
  && templates.length === 26 && skills.length === 115
  && statuses.length === 52,
"encounter source closure drift");

for (const [category, rows, idOf] of [
  ["stage_configs", stageRows, (row) => `stage:${row.StageID}`],
  ["enemy_variants", monsters, (row) => `monster:${row.MonsterID}`],
  ["enemy_templates", templates,
    (row) => `template:${row.MonsterTemplateID}`],
  ["enemy_skills", skills, (row) => `skill:${row.SkillID}`],
  ["enemy_statuses", statuses, (row) => `status:${row.StatusID}`],
]) {
  for (const row of rows)
    assert(digest(row) === manifestRecord(category, idOf(row)).evidence_sha256,
      `${category}:${idOf(row)} source row digest drift`);
}

const templateById = new Map(templates.map((row) =>
  [row.MonsterTemplateID, row]));
const monsterById = new Map(monsters.map((row) => [row.MonsterID, row]));
const skillById = new Map(skills.map((row) => [row.SkillID, row]));
const textEn = parsed.textEn;
const textZh = parsed.textZh;
function translated(hash, locale, fallback) {
  if (hash === undefined || hash === null) return fallback;
  const map = locale === "en" ? textEn : textZh;
  return map[String(hash)] ?? fallback;
}

const stageSpecs = stageRows.map((row) => {
  const normalized = normalizedStageBySource.get(String(row.StageID));
  assert(normalized !== undefined, `missing normalized stage ${row.StageID}`);
  const pairs = new Map((row.StageConfigData ?? []).map(
    ({ BFLIFKBEOPJ, MNDFOPKBHKP }) => [BFLIFKBEOPJ, MNDFOPKBHKP],
  ));
  return {
    row,
    stageId: normalized.id,
    stageOrder: normalized.stage_order,
    difficulty: normalized.difficulty,
    nameEn: normalized.name_en,
    nameZh: normalized.name_zh_cn,
    battleEventId: Number(pairs.get("_CreateBattleEvent")),
    infiniteGroup: String(pairs.get("_StageInfiniteGroup")),
  };
}).sort((left, right) => left.stageOrder - right.stageOrder);

const encounterRows = stageSpecs.map((spec) => {
  const bossIds = spec.row.MonsterList.flatMap((wave) =>
    Object.entries(wave).filter(([key]) => /^Monster\d+$/u.test(key))
      .map(([, id]) => id))
    .filter((id) => ["Boss", "LittleBoss"].includes(
      templateById.get(monsterById.get(id).MonsterTemplateID).Rank,
    ));
  return envelope({
    id: `encounter.${spec.row.StageID}`,
    kind: "Encounter",
    nameEn: `${spec.nameEn} encounter`,
    nameZh: `${spec.nameZh}遭遇`,
    summaryEn:
      `${spec.nameEn} is a released two-wave level-${spec.row.Level} challenge encounter.`,
    summaryZh:
      `${spec.nameZh}是已发布的${spec.row.Level}级双波次挑战遭遇。`,
    manifestIds: [`stage_configs:stage:${spec.row.StageID}`],
    sources: [sourceRef(
      "stage_configs",
      `stage:${spec.row.StageID}`,
      "Active profile selects this exact StageConfig encounter.",
    )],
    tags: ["encounter", spec.difficulty.toLowerCase()],
    fields: {
      stage_order: spec.stageOrder,
      difficulty: spec.difficulty,
      stage_id: spec.stageId,
      source_stage_id: spec.row.StageID,
      level: spec.row.Level,
      hard_level_group: spec.row.HardLevelGroup,
      level_graph_path: spec.row.LevelGraphPath,
      wave_count: spec.row.MonsterList.length,
      battle_event_id: `battle-event.${spec.battleEventId}`,
      infinite_group: spec.infiniteGroup,
      boss_enemy_ids: [...new Set(bossIds)].sort((a, b) => a - b)
        .map((id) => `enemy.${id}`),
      release: spec.row.Release,
      forbid_exit_battle: spec.row.ForbidExitBattle,
      monster_warning_ratio: decimal(spec.row.MonsterWarningRatio),
    },
  });
});

const waveRows = stageSpecs.flatMap((spec) =>
  spec.row.MonsterList.map((wave, index) => {
    const monsterIdsForWave = Object.entries(wave)
      .filter(([key]) => /^Monster\d+$/u.test(key))
      .sort(([left], [right]) =>
        Number(left.slice(7)) - Number(right.slice(7)))
      .map(([, id]) => id);
    return envelope({
      id: `encounter-wave.${spec.row.StageID}.${index + 1}`,
      kind: "EncounterWave",
      nameEn: `${spec.nameEn} wave ${index + 1}`,
      nameZh: `${spec.nameZh}第${index + 1}波`,
      summaryEn:
        `Wave ${index + 1} installs ${monsterIdsForWave.length} explicit StageConfig enemy slots.`,
      summaryZh:
        `第${index + 1}波安装${monsterIdsForWave.length}个 StageConfig 明示敌方槽位。`,
      manifestIds: [`stage_configs:stage:${spec.row.StageID}`],
      sources: [sourceRef(
        "stage_configs",
        `stage:${spec.row.StageID}`,
        `MonsterList wave ${index + 1} preserves source slot order.`,
      )],
      tags: ["encounter-wave", spec.difficulty.toLowerCase()],
      fields: {
        encounter_id: `encounter.${spec.row.StageID}`,
        wave_order: index + 1,
        enemy_ids: monsterIdsForWave.map((id) => `enemy.${id}`),
        enemy_count: monsterIdsForWave.length,
        clock_scope: spec.stageId,
        carries_stage_clock: index > 0,
      },
    });
  }));

const slotRows = stageSpecs.flatMap((spec) =>
  spec.row.MonsterList.flatMap((wave, waveIndex) =>
    Object.entries(wave).filter(([key]) => /^Monster\d+$/u.test(key))
      .sort(([left], [right]) =>
        Number(left.slice(7)) - Number(right.slice(7)))
      .map(([, monsterId], slotIndex) => envelope({
        id: `enemy-slot.${spec.row.StageID}.${waveIndex + 1}.${slotIndex + 1}`,
        kind: "EnemySlot",
        nameEn: `${spec.nameEn} wave ${waveIndex + 1} slot ${slotIndex + 1}`,
        nameZh:
          `${spec.nameZh}第${waveIndex + 1}波第${slotIndex + 1}槽位`,
        summaryEn:
          `This slot explicitly selects enemy variant ${monsterId}.`,
        summaryZh: `该槽位明示选择敌方变体${monsterId}。`,
        manifestIds: [`enemy_variants:monster:${monsterId}`],
        sources: [
          sourceRef(
            "stage_configs",
            `stage:${spec.row.StageID}`,
            `MonsterList wave ${waveIndex + 1} slot ${slotIndex + 1}.`,
          ),
          sourceRef(
            "enemy_variants",
            `monster:${monsterId}`,
            "Exact concrete enemy variant selected by this slot.",
          ),
        ],
        tags: ["direct-stage-enemy", "enemy-slot"],
        fields: {
          encounter_id: `encounter.${spec.row.StageID}`,
          wave_order: waveIndex + 1,
          slot_order: slotIndex + 1,
          enemy_id: `enemy.${monsterId}`,
          source_numeric_id: monsterId,
        },
      }))),
);

const directMonsterIds = new Set(slotRows.map(
  ({ source_numeric_id: id }) => id,
));
const parentsBySummon = new Map(monsters.map(({ MonsterID }) =>
  [MonsterID, []]));
for (const monster of monsters) {
  const summons = [
    ...(monster.SummonIDList ?? []),
    ...(monster.CustomValues ?? []).filter(
      ({ BFLIFKBEOPJ: key, MNDFOPKBHKP: value }) =>
        typeof key === "string" && /SummonID/iu.test(key)
          && Number.isSafeInteger(value),
    ).map(({ MNDFOPKBHKP: value }) => value),
  ];
  for (const summonId of new Set(summons))
    if (parentsBySummon.has(summonId))
      parentsBySummon.get(summonId).push(monster.MonsterID);
}
const firstVariantForTemplate = new Map();
for (const monster of monsters)
  if (!firstVariantForTemplate.has(monster.MonsterTemplateID))
    firstVariantForTemplate.set(monster.MonsterTemplateID, monster.MonsterID);

const enemyRows = monsters.map((monster) => {
  const template = templateById.get(monster.MonsterTemplateID);
  const nameHash = String(monster.MonsterName?.Hash
    ?? template.MonsterName?.Hash);
  const nameEn = translated(nameHash, "en", `Enemy ${monster.MonsterID}`);
  const nameZh = translated(nameHash, "zh", `敌方${monster.MonsterID}`);
  const isTemplateOwner =
    firstVariantForTemplate.get(monster.MonsterTemplateID)
      === monster.MonsterID;
  const ownedSkills = (monster.SkillList ?? []).map((id) => skillById.get(id));
  const phaseMarkers = [...new Set(ownedSkills.flatMap(
    ({ PhaseList = [] }) => PhaseList,
  ))].sort((left, right) => left - right);
  const summons = [...new Set([
    ...(monster.SummonIDList ?? []),
    ...(monster.CustomValues ?? []).filter(
      ({ BFLIFKBEOPJ: key, MNDFOPKBHKP: value }) =>
        typeof key === "string" && /SummonID/iu.test(key)
          && Number.isSafeInteger(value),
    ).map(({ MNDFOPKBHKP: value }) => value),
  ])].filter((id) => monsterById.has(id)).sort((a, b) => a - b);
  return envelope({
    id: `enemy.${monster.MonsterID}`,
    kind: "EnemyVariant",
    nameEn,
    nameZh,
    summaryEn:
      `${nameEn} is a reachable level-scaled ${template.Rank} variant with an exact template, skill and summon closure.`,
    summaryZh:
      `${nameZh}是可达的等级缩放${template.Rank}变体，保留精确模板、技能与召唤闭包。`,
    manifestIds: [
      `enemy_variants:monster:${monster.MonsterID}`,
      ...(isTemplateOwner
        ? [`enemy_templates:template:${monster.MonsterTemplateID}`]
        : []),
    ],
    sources: [
      sourceRef(
        "enemy_variants",
        `monster:${monster.MonsterID}`,
        directMonsterIds.has(monster.MonsterID)
          ? "Active StageConfig selects this concrete variant."
          : "An explicit reachable summon reference selects this variant.",
      ),
      sourceRef(
        "enemy_templates",
        `template:${monster.MonsterTemplateID}`,
        "MonsterConfig explicitly selects this template.",
      ),
      textRef("en", nameHash, nameEn, "Exact released English enemy name."),
      textRef("zh_cn", nameHash, nameZh,
        "Exact released Simplified Chinese enemy name."),
    ],
    tags: [
      "enemy",
      directMonsterIds.has(monster.MonsterID) ? "stage-direct" : "summoned",
      template.Rank.toLowerCase(),
    ],
    fields: {
      source_numeric_id: monster.MonsterID,
      source_template_id: monster.MonsterTemplateID,
      rank: template.Rank,
      direct_stage_member: directMonsterIds.has(monster.MonsterID),
      summon_parent_ids: parentsBySummon.get(monster.MonsterID)
        .sort((a, b) => a - b).map((id) => `enemy.${id}`),
      summon_enemy_ids: summons.map((id) => `enemy.${id}`),
      skill_ids: (monster.SkillList ?? []).map((id) => `enemy-skill.${id}`),
      ability_names: monster.AbilityNameList ?? [],
      override_ai_path: monster.OverrideAIPath || null,
      template_ai_path: template.AIPath || null,
      character_config_path: template.JsonConfig || null,
      ai_skill_sequence: values(
        monster.OverrideAISkillSequence ?? [],
        "MNAHFIGOHML",
      ).map((id) => `enemy-skill.${id}`),
      template_ai_skill_sequence: values(
        template.AISkillSequence ?? [],
        "MNAHFIGOHML",
      ).map((id) => `enemy-skill.${id}`),
      phase_markers: phaseMarkers,
      attack_ratio: decimal(monster.AttackModifyRatio),
      defence_ratio: decimal(monster.DefenceModifyRatio),
      hp_ratio: decimal(monster.HPModifyRatio),
      speed_ratio: decimal(monster.SpeedModifyRatio),
      stance_ratio: decimal(monster.StanceModifyRatio),
      stance_value: decimal(monster.StanceModifyValue),
      weaknesses: monster.StanceWeakList ?? [],
      damage_resistances: (monster.DamageTypeResistance ?? []).map(
        ({ DamageType, Value }) => ({
          damage_type: DamageType,
          value: decimal(Value),
        }),
      ),
      debuff_resistances: (monster.DebuffResist ?? []).map(
        ({ Key, Value }) => ({ key: Key, value: decimal(Value) }),
      ),
      base_stats: {
        attack: decimal(template.AttackBase),
        defence: decimal(template.DefenceBase),
        hp: decimal(template.HPBase),
        speed: decimal(template.SpeedBase),
        stance: decimal(template.StanceBase),
      },
      presentation_assets_included: false,
    },
  });
});

const ownersBySkill = new Map(skills.map(({ SkillID }) => [SkillID, []]));
for (const monster of monsters)
  for (const [skillOrder, skillId] of (monster.SkillList ?? []).entries())
    ownersBySkill.get(skillId).push({ monsterId: monster.MonsterID, skillOrder });
const skillRows = skills.map((skill) => {
  const owners = ownersBySkill.get(skill.SkillID).sort(
    (left, right) => left.monsterId - right.monsterId,
  );
  assert(owners.length > 0, `skill ${skill.SkillID} lacks explicit owner`);
  const nameHash = String(skill.SkillName?.Hash);
  const nameEn = translated(nameHash, "en", `Skill ${skill.SkillID}`);
  const nameZh = translated(nameHash, "zh", `技能${skill.SkillID}`);
  return envelope({
    id: `enemy-skill.${skill.SkillID}`,
    kind: "EnemySkill",
    nameEn,
    nameZh,
    summaryEn:
      `${nameEn} is explicitly listed by ${owners.length} reachable enemy variant${owners.length === 1 ? "" : "s"}.`,
    summaryZh:
      `${nameZh}由${owners.length}个可达敌方变体明示列出。`,
    manifestIds: [`enemy_skills:skill:${skill.SkillID}`],
    sources: [
      sourceRef(
        "enemy_skills",
        `skill:${skill.SkillID}`,
        "A reachable MonsterConfig SkillList explicitly selects this row.",
      ),
      ...owners.map(({ monsterId }) => sourceRef(
        "enemy_variants",
        `monster:${monsterId}`,
        `MonsterConfig ${monsterId} explicitly lists this skill.`,
      )),
      textRef("en", nameHash, nameEn, "Exact released English skill name."),
      textRef("zh_cn", nameHash, nameZh,
        "Exact released Simplified Chinese skill name."),
    ],
    tags: ["enemy-skill", skill.AttackType ?? "unknown"],
    fields: {
      enemy_id: `enemy.${owners[0].monsterId}`,
      enemy_ids: owners.map(({ monsterId }) => `enemy.${monsterId}`),
      skill_order: owners[0].skillOrder + 1,
      source_numeric_id: skill.SkillID,
      trigger_key: skill.SkillTriggerKey ?? null,
      attack_type: skill.AttackType ?? null,
      damage_type: skill.DamageType ?? null,
      phase_list: skill.PhaseList ?? [],
      parameters: (skill.ParamList ?? []).map(decimal),
      modifier_names: skill.ModifierList ?? [],
      extra_effect_ids: skill.ExtraEffectIDList ?? [],
      ai_cd: decimal(skill.AI_CD),
      ai_icd: decimal(skill.AI_ICD),
      delay_ratio: decimal(skill.DelayRatio),
      sp_hit_base: decimal(skill.SPHitBase),
      presentation_assets_included: false,
    },
  });
}).sort((left, right) =>
  left.enemy_id.localeCompare(right.enemy_id)
    || left.skill_order - right.skill_order
    || left.id.localeCompare(right.id));

const configRecords = manifest.categories.config_programs.records;
const configTexts = new Map();
for (const record of configRecords)
  configTexts.set(record.source_path,
    (await sourceBytes(record.source_path)).toString("utf8"));
const statusRows = statuses.map((status) => {
  const paths = configRecords.filter((record) =>
    configTexts.get(record.source_path).includes(
      JSON.stringify(status.ModifierName),
    )).map(({ source_path: sourcePath }) => sourcePath).sort(compareText);
  assert(paths.length > 0,
    `status ${status.StatusID} lacks an exact program reference`);
  const nameHash = String(status.StatusName?.Hash);
  const nameEn = translated(nameHash, "en", status.ModifierName);
  const nameZh = translated(nameHash, "zh", status.ModifierName);
  return envelope({
    id: `enemy-status.${status.StatusID}`,
    kind: "EnemyStatus",
    nameEn,
    nameZh,
    summaryEn:
      `${nameEn} is referenced by ${paths.length} enabled mechanical configuration program${paths.length === 1 ? "" : "s"}.`,
    summaryZh: `${nameZh}由${paths.length}个已启用机械配置程序引用。`,
    manifestIds: [`enemy_statuses:status:${status.StatusID}`],
    sources: [
      sourceRef(
        "enemy_statuses",
        `status:${status.StatusID}`,
        "ModifierName is present in the active configuration closure.",
      ),
      ...paths.map((sourcePath) => sourceRef(
        "config_programs",
        `config:${sourcePath}`,
        `This program contains the exact ModifierName ${status.ModifierName}.`,
      )),
      textRef("en", nameHash, nameEn, "Exact released English status name."),
      textRef("zh_cn", nameHash, nameZh,
        "Exact released Simplified Chinese status name."),
    ],
    tags: ["enemy-status", status.StatusType ?? "unknown"],
    fields: {
      enemy_id: "config-program-closure",
      owner_resolution: "TransitiveProgramClosure",
      source_numeric_id: status.StatusID,
      modifier_name: status.ModifierName,
      status_type: status.StatusType ?? null,
      read_parameters: status.ReadParamList ?? [],
      status_tags: status.TagList ?? [],
      referencing_program_paths: paths,
      presentation_assets_included: false,
    },
  });
}).sort((left, right) =>
  left.enemy_id.localeCompare(right.enemy_id)
    || left.id.localeCompare(right.id));

function programOwner(sourcePath) {
  if (sourcePath.includes("BattleEvent")
    || sourcePath.includes("ChallengePeakBattle"))
    return "encounter.system";
  if (sourcePath.includes("Stage")) return "encounter.stage-graph";
  return "enemy.config-closure";
}
const orderByOwner = new Map();
const abilityRows = configRecords.map((record) => {
  const ownerId = programOwner(record.source_path);
  const bindingOrder = (orderByOwner.get(ownerId) ?? 0) + 1;
  orderByOwner.set(ownerId, bindingOrder);
  return envelope({
    id: `ability-binding.${String(bindingOrder).padStart(3, "0")}.${
      digest(record.source_path).slice(0, 12)}`,
    kind: "AbilityBinding",
    nameEn: `Configuration binding ${bindingOrder}`,
    nameZh: `配置绑定${bindingOrder}`,
    summaryEn:
      `${ownerId} reaches ${record.source_path} through the frozen selector closure.`,
    summaryZh:
      `${ownerId}通过冻结选择器闭包到达${record.source_path}。`,
    ownership: record.ownership,
    manifestIds: [`config_programs:${record.id}`],
    sources: [sourceRef(
      "config_programs",
      record.id,
      "Exact source path, locator, selector and reachability receipt.",
    )],
    tags: ["ability-binding", ownerId],
    fields: {
      owner_id: ownerId,
      binding_order: bindingOrder,
      source_path: record.source_path,
      source_locator: record.row_locator,
      selector: record.selector,
      reachability: record.reachability,
      program_sha256: record.evidence_sha256,
      program_body_imported: false,
    },
  });
}).sort((left, right) =>
  left.owner_id.localeCompare(right.owner_id)
    || left.binding_order - right.binding_order
    || left.id.localeCompare(right.id));

const outputs = {
  "encounters.json": file("encounters.json", "Encounter", encounterRows),
  "encounter-waves.json": file(
    "encounter-waves.json",
    "EncounterWave",
    waveRows,
  ),
  "enemy-slots.json": file("enemy-slots.json", "EnemySlot", slotRows),
  "enemies.json": file("enemies.json", "EnemyVariant", enemyRows),
  "enemy-skills.json": file("enemy-skills.json", "EnemySkill", skillRows),
  "enemy-statuses.json": file(
    "enemy-statuses.json",
    "EnemyStatus",
    statusRows,
  ),
  "ability-bindings.json": file(
    "ability-bindings.json",
    "AbilityBinding",
    abilityRows,
  ),
};
await mkdir(outputRoot, { recursive: true });
for (const [name, document] of Object.entries(outputs)) {
  const bytes = `${JSON.stringify(document, null, 2)}\n`;
  const target = path.join(outputRoot, name);
  if (check) {
    const existing = await readFile(target, "utf8").catch(() => "");
    assert(existing === bytes, `${name} generation drift`);
  } else {
    await writeFile(target, bytes);
  }
}
console.log(
  "Anomaly Arbitration encounters generated: "
    + Object.entries(outputs).map(([name, { records }]) =>
      `${name}=${records.length}`).join(", "),
);
