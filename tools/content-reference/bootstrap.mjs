import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

const args = parseArgs(process.argv.slice(2));
const repoRoot = requiredPath(args, "repo-root");
const turnDataRoot = requiredPath(args, "turn-data");
const starRailResRoot = requiredPath(args, "starrail-res");
const outputRoot = requiredPath(args, "out", false);

const turnExcel = path.join(turnDataRoot, "ExcelOutput");
const turnText = path.join(turnDataRoot, "TextMap");
const resEn = path.join(starRailResRoot, "index_new", "en");
const resZh = path.join(starRailResRoot, "index_new", "cn");

const sourceFiles = new Map();
const textEn = readJson(path.join(turnText, "TextMapEN.json"));
const textZh = readJson(path.join(turnText, "TextMapCHS.json"));
const profiles = readCharacterProfiles(path.join(repoRoot, "docs", "characters"));

const sourceManifest = {
  schema_revision: "content-reference-v1",
  snapshot: "4.4",
  generated_on: "2026-07-17",
  repositories: [
    repositoryEvidence("dimbreath-turnbasedgamedata", turnDataRoot),
    repositoryEvidence("mar-7th-star-rail-res", starRailResRoot),
  ],
};

const res = {
  characters_en: readResJson("en", "characters.json"),
  characters_zh: readResJson("cn", "characters.json"),
  skills_en: readResJson("en", "character_skills.json"),
  skills_zh: readResJson("cn", "character_skills.json"),
  traces_en: readResJson("en", "character_skill_trees.json"),
  traces_zh: readResJson("cn", "character_skill_trees.json"),
  ranks_en: readResJson("en", "character_ranks.json"),
  ranks_zh: readResJson("cn", "character_ranks.json"),
  promotions: readResJson("en", "character_promotions.json"),
  cones_en: readResJson("en", "light_cones.json"),
  cones_zh: readResJson("cn", "light_cones.json"),
  cone_ranks_en: readResJson("en", "light_cone_ranks.json"),
  cone_ranks_zh: readResJson("cn", "light_cone_ranks.json"),
};

const raw = {
  avatars: readTurnTable("AvatarConfig.json"),
  avatarPromotions: readTurnTable("AvatarPromotionConfig.json"),
  avatarSkills: readTurnTable("AvatarSkillConfig.json"),
  avatarTraces: readTurnTable("AvatarSkillTreeConfig.json"),
  avatarRanks: readTurnTable("AvatarRankConfig.json"),
  equipment: readTurnTable("EquipmentConfig.json"),
  equipmentPromotions: readTurnTable("EquipmentPromotionConfig.json"),
  equipmentSkills: readTurnTable("EquipmentSkillConfig.json"),
  monsters: readTurnTable("MonsterConfig.json"),
  monsterTemplates: readTurnTable("MonsterTemplateConfig.json"),
  monsterSkills: readTurnTable("MonsterSkillConfig.json"),
  monsterStatuses: readTurnTable("MonsterStatusConfig.json"),
  stages: readTurnTable("StageConfig.json"),
};

const abilityIndex = buildAbilityIndex([
  path.join(turnDataRoot, "Config", "ConfigAbility", "Avatar"),
  path.join(turnDataRoot, "Config", "ConfigAbility", "Monster"),
]);

const characterResult = buildCharacters();
const lightCones = buildLightCones();
const enemyResult = buildEnemies();
const encounters = buildEncounters(enemyResult.variantBySourceId);

const coverage = {
  schema_revision: "content-reference-v1",
  snapshot: "4.4",
  generated_on: "2026-07-17",
  characters: summarizeQuality(characterResult.characters),
  character_abilities: summarizeQuality(characterResult.abilities),
  character_traces: summarizeQuality(characterResult.traces),
  character_eidolons: summarizeQuality(characterResult.eidolons),
  light_cones: summarizeQuality(lightCones),
  enemy_templates: summarizeQuality(enemyResult.templates),
  enemy_variants: summarizeQuality(enemyResult.variants),
  enemy_abilities: summarizeQuality(enemyResult.abilities),
  encounters: summarizeQuality(encounters),
  notes: [
    "Released 4.4 data is primary; Saber and Archer use the pinned 4.3 release index because their licensed collaboration records are absent from the 4.4 release dump.",
    "Long source descriptions and assets are not emitted. Text hashes and pinned source locators preserve evidence without redistributing prose.",
    "ExactStructured means the value or relationship is transcribed from released structured data. Derived means Starclock grouped or named records deterministically. Approximate requires an explicit reason on the record.",
    "Operation type summaries are evidence aids, not executable Rule IR and do not determine undocumented event ordering.",
  ],
};

fs.mkdirSync(outputRoot, { recursive: true });
writeJson("manifest.json", sourceManifest);
writeJson("characters.json", characterResult.characters);
writeJson("character-abilities.json", characterResult.abilities);
writeJson("character-traces.json", characterResult.traces);
writeJson("character-eidolons.json", characterResult.eidolons);
writeJson("light-cones.json", lightCones);
writeJson("enemy-templates.json", enemyResult.templates);
writeJson("enemy-variants.json", enemyResult.variants);
writeJson("enemy-abilities.json", enemyResult.abilities);
writeJson("encounters.json", encounters);
writeJson("sources.json", [...sourceFiles.values()].sort(byId));
writeJson("coverage.json", coverage);
writePackIndex();

function buildCharacters() {
  const avatarById = new Map(raw.avatars.map((row) => [String(row.AvatarID), row]));
  const rawGroups = new Map();
  for (const avatar of raw.avatars.filter((row) => row.Release !== false)) {
    const pathName = normalizePath(avatar.AvatarBaseType);
    const nameEn = resolveText(avatar.AvatarName, textEn) || avatar.AvatarVOTag;
    const nameZh = resolveText(avatar.AvatarName, textZh) || nameEn;
    const id = characterId(nameEn, pathName, avatar.AvatarID);
    const group = rawGroups.get(id) ?? {
      id,
      name_en: displayCharacterName(nameEn, pathName, avatar.AvatarID),
      name_zh_cn: displayCharacterName(nameZh, pathName, avatar.AvatarID),
      path: pathName,
      element: normalizeElement(avatar.DamageType),
      rarity: parseRarity(avatar.Rarity),
      source_avatar_ids: [],
      avatar_rows: [],
    };
    group.source_avatar_ids.push(String(avatar.AvatarID));
    group.avatar_rows.push(avatar);
    rawGroups.set(id, group);
  }

  for (const sourceId of ["1014", "1015"]) {
    if (avatarById.has(sourceId)) continue;
    const entry = res.characters_en[sourceId];
    const entryZh = res.characters_zh[sourceId] ?? entry;
    if (!entry) continue;
    const pathName = normalizePath(entry.path);
    const id = characterId(entry.name, pathName, Number(sourceId));
    rawGroups.set(id, {
      id,
      name_en: entry.name,
      name_zh_cn: entryZh.name ?? entry.name,
      path: pathName,
      element: normalizeElement(entry.element),
      rarity: entry.rarity,
      source_avatar_ids: [sourceId],
      avatar_rows: [],
      fallback_res: true,
    });
  }

  const skillRowsById = groupBy(raw.avatarSkills, (row) => String(row.SkillID));
  const promotionRowsByAvatar = groupBy(raw.avatarPromotions, (row) => String(row.AvatarID));
  const traceRowsByAvatar = groupBy(raw.avatarTraces, (row) => String(row.AvatarID));
  const rankRowsById = new Map(raw.avatarRanks.map((row) => [String(row.RankID), row]));

  const characters = [];
  const abilities = [];
  const traces = [];
  const eidolons = [];

  for (const group of [...rawGroups.values()].sort(byId)) {
    const representative = group.avatar_rows[0];
    const profile = findProfile(group.name_en, group.path);
    const characterAbilities = [];
    const characterTraces = [];
    const characterEidolons = [];

    if (representative) {
      const configSkills = new Map();
      for (const avatar of group.avatar_rows) {
        for (const entry of readCharacterConfigSkills(avatar.JsonPath)) {
          const key = entry.Name ?? "";
          if (!configSkills.has(key)) configSkills.set(key, entry);
        }
      }

      const sourceSkillIds = unique(group.avatar_rows.flatMap((row) => row.SkillList ?? []))
        .map(String)
        .sort(numericTextCompare);
      const abilityDrafts = [];
      for (const sourceSkillId of sourceSkillIds) {
        const levels = (skillRowsById.get(sourceSkillId) ?? []).sort((a, b) => a.Level - b.Level);
        if (levels.length === 0) continue;
        const first = levels[0];
        const nameEn = resolveText(first.SkillName, textEn) || `Ability ${sourceSkillId}`;
        const nameZh = resolveText(first.SkillName, textZh) || nameEn;
        const config = configSkills.get(first.SkillTriggerKey) ?? null;
        const entryAbility = config?.EntryAbility ?? "";
        const evidence = abilityEvidence(entryAbility);
        const mechanicText = sourceTextValue(first.SkillDesc, textEn);
        const hints = mechanicHints(mechanicText);
        const authoredTarget = normalizeTarget(config?.TargetInfo);
        abilityDrafts.push({
          id_base: `ability.${slug(nameEn)}.${slug(first.AttackType || first.SkillTriggerKey || "passive")}`,
          name_en: nameEn,
          name_zh_cn: nameZh,
          source_skill_ids: [sourceSkillId],
          kind: first.AttackType || "Passive",
          effect_category: first.SkillEffect || "",
          trigger_key: first.SkillTriggerKey || "",
          use_type: config?.UseType ?? "",
          target: authoredTarget.target_type ? authoredTarget : inferredTarget(hints),
          entry_ability: entryAbility,
          source_ability_files: evidence.files,
          operation_types: evidence.operationTypes,
          max_level: first.MaxLevel,
          energy_gain: decimal(first.SPBase),
          skill_point_cost: decimal(first.BPNeed),
          initial_cooldown: scalar(first.InitCoolDown),
          cooldown: scalar(first.CoolDown),
          delay_ratio: decimal(first.DelayRatio),
          display_toughness: decimals(first.ShowStanceList),
          display_damage: decimals(first.ShowDamageList),
          display_heal: decimals(first.ShowHealList),
          levels: levels.map((row) => ({
            level: row.Level,
            parameters: decimals(row.ParamList),
            simple_parameters: decimals(row.SimpleParamList),
          })),
          mechanic_hints: hints,
          source_text: textEvidence(first.SkillDesc, textEn),
          mechanism_quality: config || first.AttackType === "MazeNormal" ? "ExactStructured" : "ApproximateFromReleasedText",
          quality: "ExactStructured",
        });
      }
      for (const draft of mergeDrafts(abilityDrafts)) {
        const id = `${group.id}.${draft.id_base}`;
        abilities.push({ id, character_id: group.id, ...without(draft, "id_base") });
        characterAbilities.push(id);
      }

      const sourceTraceRows = group.source_avatar_ids.flatMap((id) => traceRowsByAvatar.get(id) ?? []);
      const traceGroups = groupBy(sourceTraceRows, (row) => traceSemanticKey(row));
      let traceOrdinal = 0;
      for (const rows of [...traceGroups.values()].sort(compareTraceRows)) {
        traceOrdinal += 1;
        const ordered = [...rows].sort((a, b) => (a.Level ?? 0) - (b.Level ?? 0));
        const first = ordered[0];
        const sourceIds = unique(ordered.map((row) => String(row.PointID))).sort(numericTextCompare);
        const fallback = res.traces_en[sourceIds[0]];
        const fallbackZh = res.traces_zh[sourceIds[0]];
        const nameEn = resolveText(first.PointName, textEn) || fallback?.name || `Trace ${traceOrdinal}`;
        const nameZh = resolveText(first.PointName, textZh) || fallbackZh?.name || nameEn;
        const id = `${group.id}.trace.${pad(traceOrdinal)}.${slug(nameEn)}`;
        traces.push({
          id,
          character_id: group.id,
          name_en: nameEn,
          name_zh_cn: nameZh,
          source_point_ids: sourceIds,
          point_type: first.PointType,
          anchor: first.AnchorType ?? fallback?.anchor ?? "",
          max_level: first.MaxLevel ?? fallback?.max_level ?? 1,
          default_unlocked: Boolean(first.DefaultUnlock),
          prerequisites: unique(ordered.flatMap((row) => row.PrePoint ?? []).map(String)).sort(numericTextCompare),
          level_up_skill_source_ids: unique(ordered.flatMap((row) => row.LevelUpSkillID ?? []).map(String)).sort(numericTextCompare),
          status_additions: normalizeRawList(ordered.flatMap((row) => row.StatusAddList ?? [])),
          ability_names: unique(ordered.map((row) => row.AbilityName).filter(Boolean)).sort(),
          trigger_keys: unique(ordered.map((row) => row.PointTriggerKey).filter(Boolean)).sort(),
          levels: ordered.map((row) => ({ level: row.Level ?? 1, parameters: decimals(row.ParamList) })),
          mechanic_hints: mechanicHints(sourceTextValue(first.PointDesc, textEn, fallback?.desc)),
          source_text: textEvidence(first.PointDesc, textEn, fallback?.desc),
          quality: "ExactStructured",
        });
        characterTraces.push(id);
      }

      const sourceRankIds = unique(group.avatar_rows.flatMap((row) => row.RankIDList ?? [])).map(String);
      const sourceRanksByRank = groupBy(sourceRankIds, (sourceRankId) => {
        const row = rankRowsById.get(sourceRankId);
        return String(row?.Rank ?? res.ranks_en[sourceRankId]?.rank ?? "");
      });
      for (const [rankText, rankSourceIds] of [...sourceRanksByRank.entries()].sort(([a], [b]) => Number(a) - Number(b))) {
        const sourceRankId = rankSourceIds.sort(numericTextCompare)[0];
        const row = rankRowsById.get(sourceRankId);
        const fallback = res.ranks_en[sourceRankId];
        const fallbackZh = res.ranks_zh[sourceRankId];
        const rank = Number(rankText);
        if (!rank) continue;
        const nameEn = fallback?.name || `Eidolon ${rank}`;
        const nameZh = fallbackZh?.name || nameEn;
        const id = `${group.id}.eidolon.${rank}`;
        eidolons.push({
          id,
          character_id: group.id,
          rank,
          name_en: nameEn,
          name_zh_cn: nameZh,
          source_rank_ids: rankSourceIds.sort(numericTextCompare),
          parameters: decimals(row?.Param),
          skill_level_additions: normalizeKeyValueMap(row?.SkillAddLevelList ?? fallback?.level_up_skills),
          ability_names: unique(row?.RankAbility ?? []).sort(),
          mechanic_hints: mechanicHints(sourceTextValue(row?.Desc, textEn, fallback?.desc)),
          source_text: textEvidence(row?.Desc, textEn, fallback?.desc),
          quality: row ? "ExactStructured" : "ExactPreviousRelease",
        });
        characterEidolons.push(id);
      }
    } else {
      const entry = res.characters_en[group.source_avatar_ids[0]];
      const entryZh = res.characters_zh[group.source_avatar_ids[0]] ?? entry;
      for (const sourceSkillId of entry.skills ?? []) {
        const skill = res.skills_en[sourceSkillId];
        const skillZh = res.skills_zh[sourceSkillId] ?? skill;
        if (!skill) continue;
        const id = `${group.id}.ability.${slug(skill.name)}.${slug(skill.type || "passive")}`;
        abilities.push({
          id,
          character_id: group.id,
          name_en: skill.name,
          name_zh_cn: skillZh.name ?? skill.name,
          source_skill_ids: [String(sourceSkillId)],
          kind: skill.type,
          effect_category: skill.effect,
          trigger_key: "",
          use_type: "",
          target: inferredTarget(mechanicHints(skill.desc), skill.effect ?? ""),
          entry_ability: "",
          source_ability_files: [],
          operation_types: [],
          max_level: skill.max_level,
          energy_gain: null,
          skill_point_cost: null,
          initial_cooldown: null,
          cooldown: null,
          delay_ratio: null,
          display_toughness: [],
          display_damage: [],
          display_heal: [],
          levels: (skill.params ?? []).map((parameters, index) => ({
            level: index + 1,
            parameters: decimals(parameters),
            simple_parameters: [],
          })),
          mechanic_hints: mechanicHints(skill.desc),
          source_text: textEvidence(null, null, skill.desc),
          mechanism_quality: "ExactPreviousReleaseText",
          quality: "ExactPreviousRelease",
        });
        characterAbilities.push(id);
      }
      for (const sourceTraceId of entry.skill_trees ?? []) {
        const trace = res.traces_en[sourceTraceId];
        const traceZh = res.traces_zh[sourceTraceId] ?? trace;
        if (!trace) continue;
        const id = `${group.id}.trace.${sourceTraceId}`;
        traces.push({
          id,
          character_id: group.id,
          name_en: trace.name || `Trace ${sourceTraceId}`,
          name_zh_cn: traceZh.name || trace.name || `Trace ${sourceTraceId}`,
          source_point_ids: [String(sourceTraceId)],
          point_type: null,
          anchor: trace.anchor,
          max_level: trace.max_level,
          default_unlocked: false,
          prerequisites: (trace.pre_points ?? []).map(String),
          level_up_skill_source_ids: (trace.level_up_skills ?? []).map((row) => String(row.id)),
          status_additions: normalizeRawList((trace.levels ?? []).flatMap((row) => row.properties ?? [])),
          ability_names: [],
          trigger_keys: [],
          levels: (trace.params ?? []).map((parameters, index) => ({ level: index + 1, parameters: decimals(parameters) })),
          mechanic_hints: mechanicHints(trace.desc),
          source_text: textEvidence(null, null, trace.desc),
          quality: "ExactPreviousRelease",
        });
        characterTraces.push(id);
      }
      for (const sourceRankId of entry.ranks ?? []) {
        const rankEntry = res.ranks_en[sourceRankId];
        const rankZh = res.ranks_zh[sourceRankId] ?? rankEntry;
        if (!rankEntry) continue;
        const id = `${group.id}.eidolon.${rankEntry.rank}`;
        eidolons.push({
          id,
          character_id: group.id,
          rank: rankEntry.rank,
          name_en: rankEntry.name,
          name_zh_cn: rankZh.name ?? rankEntry.name,
          source_rank_ids: [String(sourceRankId)],
          parameters: [],
          skill_level_additions: normalizeKeyValueMap(rankEntry.level_up_skills),
          ability_names: [],
          mechanic_hints: mechanicHints(rankEntry.desc),
          source_text: textEvidence(null, null, rankEntry.desc),
          quality: "ExactPreviousRelease",
        });
        characterEidolons.push(id);
      }
    }

    const promotions = representative
      ? uniqueByJson(group.source_avatar_ids.flatMap((id) => promotionRowsByAvatar.get(id) ?? []).map(normalizeAvatarPromotion))
      : normalizeResPromotions(res.promotions[group.source_avatar_ids[0]]);
    characters.push({
      id: group.id,
      name_en: group.name_en,
      name_zh_cn: group.name_zh_cn,
      path: group.path,
      element: group.element,
      rarity: group.rarity,
      max_energy: representative ? decimal(representative.SPNeed) : decimal(res.characters_en[group.source_avatar_ids[0]]?.max_sp),
      source_avatar_ids: group.source_avatar_ids.sort(numericTextCompare),
      behavior_summary_en: profile?.core_loop ?? "",
      engine_contract_en: profile?.engine_contract ?? "",
      promotions,
      ability_ids: characterAbilities.sort(),
      trace_ids: characterTraces.sort(),
      eidolon_ids: characterEidolons.sort(),
      quality: representative ? "ExactStructured" : "ExactPreviousRelease",
    });
  }

  return {
    characters: characters.sort(byId),
    abilities: abilities.sort(byId),
    traces: traces.sort(byId),
    eidolons: eidolons.sort(byId),
  };
}

function buildLightCones() {
  const promotionRows = groupBy(raw.equipmentPromotions, (row) => String(row.EquipmentID));
  const skillRows = groupBy(raw.equipmentSkills, (row) => String(row.SkillID));
  const collisionCount = new Map();
  const result = [];
  for (const row of raw.equipment.filter((entry) => entry.Release !== false).sort((a, b) => a.EquipmentID - b.EquipmentID)) {
    const nameEn = resolveText(row.EquipmentName, textEn) || `Light Cone ${row.EquipmentID}`;
    const nameZh = resolveText(row.EquipmentName, textZh) || nameEn;
    const baseId = `light-cone.${slug(nameEn)}`;
    const ordinal = (collisionCount.get(baseId) ?? 0) + 1;
    collisionCount.set(baseId, ordinal);
    const id = ordinal === 1 ? baseId : `${baseId}.${pad(ordinal)}`;
    const levels = (skillRows.get(String(row.SkillID)) ?? []).sort((a, b) => a.Level - b.Level);
    const fallback = res.cone_ranks_en[String(row.EquipmentID)];
    result.push({
      id,
      name_en: nameEn,
      name_zh_cn: nameZh,
      path: normalizePath(row.AvatarBaseType),
      rarity: parseRarity(row.Rarity),
      source_equipment_id: String(row.EquipmentID),
      max_promotion: row.MaxPromotion,
      max_superimposition: row.MaxRank,
      promotions: (promotionRows.get(String(row.EquipmentID)) ?? []).map(normalizeEquipmentPromotion),
      passive: {
        name_en: levels.length ? resolveText(levels[0].SkillName, textEn) : fallback?.skill ?? "",
        name_zh_cn: levels.length ? resolveText(levels[0].SkillName, textZh) : res.cone_ranks_zh[String(row.EquipmentID)]?.skill ?? fallback?.skill ?? "",
        source_skill_id: String(row.SkillID),
        ability_name: levels[0]?.AbilityName ?? "",
        superimpositions: levels.length
          ? levels.map((entry) => ({ rank: entry.Level, parameters: decimals(entry.ParamList), properties: normalizeRawList(entry.AbilityProperty ?? []) }))
          : (fallback?.params ?? []).map((parameters, index) => ({ rank: index + 1, parameters: decimals(parameters), properties: [] })),
        mechanic_hints: mechanicHints(sourceTextValue(levels[0]?.SkillDesc, textEn, fallback?.desc)),
        source_text: textEvidence(levels[0]?.SkillDesc, textEn, fallback?.desc),
      },
      quality: levels.length ? "ExactStructured" : "ExactPreviousRelease",
    });
  }
  return result.sort(byId);
}

function buildEnemies() {
  const variantsByTemplate = groupBy(raw.monsters, (row) => String(row.MonsterTemplateID));
  const skillRowsById = new Map(raw.monsterSkills.map((row) => [String(row.SkillID), row]));
  const statusById = new Map(raw.monsterStatuses.map((row) => [String(row.StatusID), row]));
  const templates = [];
  const variants = [];
  const abilities = [];
  const variantBySourceId = new Map();
  const nameOrdinals = new Map();

  for (const template of [...raw.monsterTemplates].sort((a, b) => a.MonsterTemplateID - b.MonsterTemplateID)) {
    const nameEn = resolveText(template.MonsterName, textEn) || `Enemy ${template.MonsterTemplateID}`;
    const nameZh = resolveText(template.MonsterName, textZh) || nameEn;
    const baseId = `enemy.${slug(nameEn)}.${slug(template.Rank || "unknown")}`;
    const ordinal = (nameOrdinals.get(baseId) ?? 0) + 1;
    nameOrdinals.set(baseId, ordinal);
    const id = ordinal === 1 ? baseId : `${baseId}.${pad(ordinal)}`;
    const configSkills = new Map(readCharacterConfigSkills(template.JsonConfig).map((entry) => [entry.Name ?? "", entry]));
    const templateVariants = (variantsByTemplate.get(String(template.MonsterTemplateID)) ?? []).sort((a, b) => a.MonsterID - b.MonsterID);
    const skillIds = unique([
      ...extractSequenceSkillIds(template.AISkillSequence),
      ...templateVariants.flatMap((variant) => variant.SkillList ?? []),
    ]).map(String).sort(numericTextCompare);
    const abilityIds = [];
    const abilityOrdinals = new Map();
    for (const sourceSkillId of skillIds) {
      const skill = skillRowsById.get(sourceSkillId);
      if (!skill) continue;
      const skillNameEn = resolveText(skill.SkillName, textEn) || `Ability ${sourceSkillId}`;
      const skillNameZh = resolveText(skill.SkillName, textZh) || skillNameEn;
      const config = configSkills.get(skill.SkillTriggerKey) ?? null;
      const entryAbility = config?.EntryAbility ?? "";
      const evidence = abilityEvidence(entryAbility);
      const mechanicText = sourceTextValue(skill.SkillDesc, textEn);
      const hints = mechanicHints(mechanicText);
      const authoredTarget = normalizeTarget(config?.TargetInfo);
      const abilityIdBase = `${id}.ability.${slug(skillNameEn)}.${slug(skill.SkillTriggerKey || "source")}`;
      const abilityOrdinal = (abilityOrdinals.get(abilityIdBase) ?? 0) + 1;
      abilityOrdinals.set(abilityIdBase, abilityOrdinal);
      const abilityId = abilityOrdinal === 1 ? abilityIdBase : `${abilityIdBase}.${pad(abilityOrdinal)}`;
      abilities.push({
        id: abilityId,
        enemy_id: id,
        name_en: skillNameEn,
        name_zh_cn: skillNameZh,
        source_skill_id: sourceSkillId,
        trigger_key: skill.SkillTriggerKey ?? "",
        attack_type: skill.AttackType ?? "",
        damage_type: normalizeElement(skill.DamageType),
        use_type: config?.UseType ?? "",
        target: authoredTarget.target_type ? authoredTarget : inferredTarget(hints),
        entry_ability: entryAbility,
        source_ability_files: evidence.files,
        operation_types: evidence.operationTypes,
        energy_on_hit: decimal(skill.SPHitBase),
        delay_ratio: decimal(skill.DelayRatio),
        ai_cooldown: scalar(skill.AI_CD),
        ai_initial_cooldown: scalar(skill.AI_ICD),
        phases: normalizeRawList(skill.PhaseList ?? []),
        parameters: decimals(skill.ParamList),
        modifiers: (skill.ModifierList ?? []).map((modifier) => ({
          name: modifier.ModifierName ?? "",
          parameters: decimals(modifier.ParamList),
        })),
        status_refs: (skill.ExtraEffectIDList ?? []).map((statusId) => normalizeStatus(statusById.get(String(statusId)), statusId)),
        mechanic_hints: hints,
        source_text: textEvidence(skill.SkillDesc, textEn),
        mechanism_quality: config ? "ExactStructured" : "ApproximateFromReleasedText",
        quality: "ExactStructured",
      });
      abilityIds.push(abilityId);
    }

    const aiEvidence = sourceConfigEvidence(template.AIPath);
    templates.push({
      id,
      name_en: nameEn,
      name_zh_cn: nameZh,
      rank: template.Rank,
      source_template_id: String(template.MonsterTemplateID),
      base_stats: {
        hp: decimal(template.HPBase),
        atk: decimal(template.AttackBase),
        def: decimal(template.DefenceBase),
        spd: decimal(template.SpeedBase),
        toughness: decimal(template.StanceBase),
        crit_damage: decimal(template.CriticalDamageBase),
        effect_res: decimal(template.StatusResistanceBase),
        initial_delay_ratio: decimal(template.InitialDelayRatio),
      },
      toughness_layers: template.StanceCount ?? 1,
      toughness_type: normalizeElement(template.StanceType),
      minimum_fatigue_ratio: decimal(template.MinimumFatigueRatio),
      source_character_config: sourceConfigEvidence(template.JsonConfig),
      source_ai: aiEvidence,
      ai_sequence_source_skill_ids: extractSequenceSkillIds(template.AISkillSequence).map(String),
      ability_ids: abilityIds.sort(),
      quality: "ExactStructured",
    });

    let variantOrdinal = 0;
    for (const variant of templateVariants) {
      variantOrdinal += 1;
      const variantId = `${id}.variant.${pad(variantOrdinal)}`;
      variantBySourceId.set(String(variant.MonsterID), variantId);
      variants.push({
        id: variantId,
        enemy_id: id,
        source_monster_id: String(variant.MonsterID),
        stat_multipliers: {
          hp: decimal(variant.HPModifyRatio),
          atk: decimal(variant.AttackModifyRatio),
          def: decimal(variant.DefenceModifyRatio),
          spd: decimal(variant.SpeedModifyRatio),
          toughness: decimal(variant.StanceModifyRatio),
        },
        weaknesses: (variant.StanceWeakList ?? []).map(normalizeElement).sort(),
        resistances: (variant.DamageTypeResistance ?? []).map((entry) => ({
          element: normalizeElement(entry.DamageType),
          value: decimal(entry.Value),
        })).sort((a, b) => a.element.localeCompare(b.element)),
        debuff_resistances: (variant.DebuffResist ?? []).map((entry) => ({
          effect: entry.Key,
          value: decimal(entry.Value),
        })).sort((a, b) => a.effect.localeCompare(b.effect)),
        source_skill_ids: (variant.SkillList ?? []).map(String).sort(numericTextCompare),
        summon_source_ids: (variant.SummonIDList ?? []).map(String).sort(numericTextCompare),
        ability_names: unique(variant.AbilityNameList ?? []).sort(),
        ai_override: sourceConfigEvidence(variant.OverrideAIPath),
        ai_sequence_override_source_skill_ids: extractSequenceSkillIds(variant.OverrideAISkillSequence).map(String),
        custom_value_tags: unique(variant.CustomValueTags ?? []).sort(),
        custom_values: normalizeRawList(variant.CustomValues ?? []),
        dynamic_values: normalizeRawList(variant.DynamicValues ?? []),
        quality: "ExactStructured",
      });
    }
  }
  return {
    templates: templates.sort(byId),
    variants: variants.sort(byId),
    abilities: abilities.sort(byId),
    variantBySourceId,
  };
}

function buildEncounters(variantBySourceId) {
  const result = [];
  const typeOrdinals = new Map();
  const seenCompositions = new Set();
  const standardTypes = new Set(["Mainline", "Cocoon", "FarmElement"]);
  for (const stage of raw.stages.filter((row) => row.Release !== false && standardTypes.has(row.StageType)).sort((a, b) => a.StageID - b.StageID)) {
    const stageType = stage.StageType || "Unknown";
    const waves = (stage.MonsterList ?? []).map((wave, waveIndex) => ({
      order: waveIndex + 1,
      slots: Object.entries(wave)
        .filter(([, sourceMonsterId]) => sourceMonsterId)
        .map(([slot, sourceMonsterId]) => ({
          slot,
          enemy_variant_id: variantBySourceId.get(String(sourceMonsterId)) ?? null,
          source_monster_id: String(sourceMonsterId),
        }))
        .sort((a, b) => a.slot.localeCompare(b.slot)),
    }));
    const composition = JSON.stringify(waves.map((wave) => wave.slots.map((slot) => slot.source_monster_id)));
    if (seenCompositions.has(composition)) continue;
    seenCompositions.add(composition);
    const ordinal = (typeOrdinals.get(stageType) ?? 0) + 1;
    typeOrdinals.set(stageType, ordinal);
    result.push({
      id: `encounter.${slug(stageType)}.${pad(ordinal, 4)}`,
      source_stage_id: String(stage.StageID),
      stage_type: stageType,
      level: stage.Level,
      hard_level_group: stage.HardLevelGroup,
      waves,
      stage_ability_config: normalizeRawList(stage.StageAbilityConfig ?? []),
      win_conditions: normalizeRawList(stage.LevelWinCondition ?? []),
      lose_conditions: normalizeRawList(stage.LevelLoseCondition ?? []),
      quality: waves.every((wave) => wave.slots.every((slot) => slot.enemy_variant_id)) ? "ExactStructured" : "DerivedWithMissingReferences",
    });
  }
  return result.sort(byId);
}

function buildAbilityIndex(roots) {
  const index = new Map();
  for (const root of roots) {
    if (!fs.existsSync(root)) continue;
    for (const file of walkJson(root)) {
      let json;
      try {
        json = readGameJson(file);
      } catch {
        continue;
      }
      const abilities = Array.isArray(json.AbilityList) ? json.AbilityList : [];
      if (abilities.length === 0) continue;
      const relative = forwardSlash(path.relative(turnDataRoot, file));
      const source = recordSourceFile("dimbreath-turnbasedgamedata", file, relative);
      for (const ability of abilities) {
        if (!ability?.Name) continue;
        const record = {
          file: source.id,
          operationTypes: collectOperationTypes(ability),
        };
        const entries = index.get(ability.Name) ?? [];
        entries.push(record);
        index.set(ability.Name, entries);
      }
    }
  }
  return index;
}

function abilityEvidence(entryAbility) {
  if (!entryAbility) return { files: [], operationTypes: [] };
  const entries = abilityIndex.get(entryAbility) ?? [];
  return {
    files: unique(entries.map((entry) => entry.file)).sort(),
    operationTypes: unique(entries.flatMap((entry) => entry.operationTypes)).sort(),
  };
}

function readCharacterConfigSkills(relativePath) {
  if (!relativePath) return [];
  const file = path.join(turnDataRoot, ...forwardSlash(relativePath).split("/"));
  if (!fs.existsSync(file)) return [];
  recordSourceFile("dimbreath-turnbasedgamedata", file, forwardSlash(relativePath));
  const json = readGameJson(file);
  return Array.isArray(json.SkillList) ? json.SkillList : [];
}

function sourceConfigEvidence(relativePath) {
  if (!relativePath) return null;
  const file = path.join(turnDataRoot, ...forwardSlash(relativePath).split("/"));
  if (!fs.existsSync(file)) {
    return { path: forwardSlash(relativePath), source_file_id: null, operation_types: [] };
  }
  const source = recordSourceFile("dimbreath-turnbasedgamedata", file, forwardSlash(relativePath));
  let operationTypes = [];
  try {
    operationTypes = collectOperationTypes(readGameJson(file));
  } catch {
    operationTypes = [];
  }
  return { path: forwardSlash(relativePath), source_file_id: source.id, operation_types: operationTypes };
}

function readTurnTable(name) {
  const file = path.join(turnExcel, name);
  recordSourceFile("dimbreath-turnbasedgamedata", file, `ExcelOutput/${name}`);
  return readGameJson(file);
}

function readResJson(language, name) {
  const file = path.join(starRailResRoot, "index_new", language, name);
  recordSourceFile("mar-7th-star-rail-res", file, `index_new/${language}/${name}`);
  return readJson(file);
}

function recordSourceFile(repository, absolutePath, relativePath) {
  const id = `source-file.${repository}.${slug(relativePath)}`;
  if (!sourceFiles.has(id)) {
    sourceFiles.set(id, {
      id,
      repository,
      path: forwardSlash(relativePath),
      sha256: sha256File(absolutePath),
    });
  }
  return sourceFiles.get(id);
}

function repositoryEvidence(id, root) {
  return {
    id,
    revision: execFileSync("git", ["-C", root, "rev-parse", "HEAD"], { encoding: "utf8" }).trim(),
    committed_at: execFileSync("git", ["-C", root, "log", "-1", "--format=%cI"], { encoding: "utf8" }).trim(),
    remote: execFileSync("git", ["-C", root, "remote", "get-url", "origin"], { encoding: "utf8" }).trim(),
    usage: "Public released-data transcription aid; raw cache remains local and uncommitted.",
  };
}

function readCharacterProfiles(directory) {
  const result = new Map();
  for (const file of fs.readdirSync(directory).filter((name) => /^profiles-.*\.md$/.test(name)).sort()) {
    const content = fs.readFileSync(path.join(directory, file), "utf8");
    const sections = content.split(/^## /m).slice(1);
    for (const section of sections) {
      const [heading, ...lines] = section.split(/\r?\n/);
      const name = heading.split(" — ")[0].trim();
      const core = lines.find((line) => line.startsWith("- **Core loop:**"));
      const contract = lines.find((line) => line.startsWith("- **Engine contract:**"));
      result.set(name, {
        core_loop: core?.replace("- **Core loop:**", "").trim() ?? "",
        engine_contract: contract?.replace("- **Engine contract:**", "").trim() ?? "",
      });
    }
  }
  return result;
}

function findProfile(name, pathName) {
  const candidates = [
    name,
    `${name} (${pathName})`,
    name === "Trailblazer" ? `Trailblazer (${pathName})` : "",
    name === "March 7th" ? `March 7th (${pathName})` : "",
  ];
  for (const candidate of candidates) {
    if (profiles.has(candidate)) return profiles.get(candidate);
  }
  return null;
}

function characterId(name, pathName, sourceId) {
  if (sourceId >= 8001 && sourceId <= 8010) return `character.trailblazer.${slug(pathName)}`;
  if (name === "March 7th") return `character.march-7th.${slug(pathName)}`;
  return `character.${slug(name)}`;
}

function displayCharacterName(name, pathName, sourceId) {
  if (sourceId >= 8001 && sourceId <= 8010) return `Trailblazer (${pathName})`;
  if (name === "March 7th" && pathName !== "Preservation") return `March 7th (${pathName})`;
  return name;
}

function normalizePath(value) {
  const paths = {
    Knight: "Preservation",
    Rogue: "The Hunt",
    Warrior: "Destruction",
    Mage: "Erudition",
    Shaman: "Harmony",
    Warlock: "Nihility",
    Priest: "Abundance",
    Memory: "Remembrance",
    Elation: "Elation",
  };
  return paths[value] ?? value ?? "Unknown";
}

function normalizeElement(value) {
  const elements = { Thunder: "Lightning" };
  return elements[value] ?? value ?? "";
}

function normalizeTarget(targetInfo) {
  if (!targetInfo || typeof targetInfo !== "object") return { target_type: "", details: {} };
  const { TargetType = "", ...details } = targetInfo;
  return { target_type: TargetType, details: normalizeRaw(details) };
}

function normalizeAvatarPromotion(row) {
  return {
    max_level: row.MaxLevel,
    hp_base: decimal(row.HPBase),
    hp_per_level: decimal(row.HPAdd),
    atk_base: decimal(row.AttackBase),
    atk_per_level: decimal(row.AttackAdd),
    def_base: decimal(row.DefenceBase),
    def_per_level: decimal(row.DefenceAdd),
    spd: decimal(row.SpeedBase),
    crit_rate: decimal(row.CriticalChance),
    crit_damage: decimal(row.CriticalDamage),
    aggro: decimal(row.BaseAggro),
  };
}

function normalizeResPromotions(entry) {
  return (entry?.values ?? []).map((value, index) => ({
    promotion: index,
    hp_base: decimal(value.hp?.base),
    hp_per_level: decimal(value.hp?.step),
    atk_base: decimal(value.atk?.base),
    atk_per_level: decimal(value.atk?.step),
    def_base: decimal(value.def?.base),
    def_per_level: decimal(value.def?.step),
    spd: decimal(value.spd?.base),
    crit_rate: decimal(value.crit_rate?.base),
    crit_damage: decimal(value.crit_dmg?.base),
    aggro: decimal(value.taunt?.base),
  }));
}

function normalizeEquipmentPromotion(row) {
  return {
    max_level: row.MaxLevel,
    hp_base: decimal(row.BaseHP),
    hp_per_level: decimal(row.BaseHPAdd),
    atk_base: decimal(row.BaseAttack),
    atk_per_level: decimal(row.BaseAttackAdd),
    def_base: decimal(row.BaseDefence),
    def_per_level: decimal(row.BaseDefenceAdd),
  };
}

function normalizeStatus(row, fallbackId) {
  if (!row) return { source_status_id: String(fallbackId), name_en: "", category: "", dispellable: null };
  return {
    source_status_id: String(row.StatusID),
    name_en: resolveText(row.StatusName, textEn),
    category: row.StatusType,
    dispellable: row.CanDispel,
    modifier_name: row.ModifierName,
  };
}

function traceSemanticKey(row) {
  const name = resolveText(row.PointName, textEn);
  return [row.AnchorType ?? "", row.PointType ?? "", name, row.PointTriggerKey ?? "", row.AbilityName ?? ""].join("|");
}

function compareTraceRows(a, b) {
  const left = a[0];
  const right = b[0];
  return String(left.AnchorType ?? "").localeCompare(String(right.AnchorType ?? ""))
    || Number(left.PointType ?? 0) - Number(right.PointType ?? 0)
    || Number(left.PointID ?? 0) - Number(right.PointID ?? 0);
}

function mergeDrafts(drafts) {
  const groups = groupBy(drafts, (draft) => draft.id_base);
  return [...groups.values()].map((entries) => {
    const first = entries[0];
    return {
      ...first,
      source_skill_ids: unique(entries.flatMap((entry) => entry.source_skill_ids)).sort(numericTextCompare),
      source_ability_files: unique(entries.flatMap((entry) => entry.source_ability_files)).sort(),
      operation_types: unique(entries.flatMap((entry) => entry.operation_types)).sort(),
    };
  }).sort((a, b) => a.id_base.localeCompare(b.id_base));
}

function textEvidence(reference, map, fallback = "") {
  const text = sourceTextValue(reference, map, fallback);
  const sourceHash = reference && typeof reference === "object" ? String(reference.Hash ?? "") : "";
  return {
    source_hash: sourceHash,
    sha256: text ? sha256Text(text) : "",
    emitted: false,
  };
}

function sourceTextValue(reference, map, fallback = "") {
  if (reference && map) {
    const resolved = resolveTextRaw(reference, map);
    if (resolved) return resolved;
  }
  return fallback ?? "";
}

function mechanicHints(text) {
  const normalized = String(text ?? "").toLowerCase();
  const tags = [];
  const tests = [
    ["damage", /\b(dmg|damage)\b/],
    ["heal", /\b(heal|restores? hp|regenerates? hp)\b/],
    ["shield", /\bshield\b/],
    ["buff", /\b(increases?|boosts?|buff)\b/],
    ["debuff", /\b(reduces?|debuff|vulnerability)\b/],
    ["dot", /\b(dot|damage over time|shock|burn|bleed|wind shear)\b/],
    ["crowd_control", /\b(freeze|frozen|entanglement|imprisonment|crowd control)\b/],
    ["dispel", /\b(dispel|remove.*buff)\b/],
    ["cleanse", /\b(cleanse|remove.*debuff)\b/],
    ["summon", /\b(summon|memosprite|servant)\b/],
    ["action_advance", /\badvance(?:s|d)? forward|action advance\b/],
    ["action_delay", /\bdelay(?:s|ed)? (?:the )?action\b/],
    ["toughness", /\b(toughness|weakness break|super break)\b/],
    ["resource", /\b(energy|skill point|stack|charge)\b/],
    ["transform", /\b(transform|enhanced state|enters? .*state)\b/],
    ["revive", /\b(revive|killing blow|knocked down)\b/],
  ];
  for (const [tag, expression] of tests) if (expression.test(normalized)) tags.push(tag);
  let targetHint = "";
  if (/\b(all enemies|all enemy targets|all targets)\b/.test(normalized)) targetHint = "AllEnemies";
  else if (/\b(adjacent targets?|adjacent enemies|enemies adjacent)\b/.test(normalized)) targetHint = "Blast";
  else if (/\b(random enem(?:y|ies)|random target)\b/.test(normalized)) targetHint = "RandomEnemy";
  else if (/\b(single enemy|one enemy|target enemy|designated enemy|single target|one designated target)\b/.test(normalized)) targetHint = "SingleEnemy";
  else if (/\b(all allies|all ally targets|all friendly targets|all friendly units)\b/.test(normalized)) targetHint = "AllAllies";
  else if (/\b(single ally|one ally|designated ally)\b/.test(normalized)) targetHint = "SingleAlly";
  else if (/\b(target team|opposing team)\b/.test(normalized)) targetHint = "OpposingTeam";
  else if (/\b(the wearer|the caster|the user|themself|self|this unit)\b/.test(normalized)) targetHint = "Self";
  else if (/\b(summons?|action order|battlefield)\b/.test(normalized)) targetHint = "Battlefield";
  return { target_hint: targetHint, operation_tags: tags.sort() };
}

function inferredTarget(hints, fallback = "") {
  const targetType = hints?.target_hint || fallback;
  return {
    target_type: targetType,
    details: targetType ? { evidence: "released_text_inference" } : {},
  };
}

function resolveText(reference, map) {
  return cleanDisplayText(resolveTextRaw(reference, map));
}

function resolveTextRaw(reference, map) {
  if (!reference || !map) return "";
  if (typeof reference === "string") return map[reference] ?? "";
  if (typeof reference === "number" || typeof reference === "bigint") return map[String(reference)] ?? "";
  return map[String(reference.Hash ?? "")] ?? "";
}

function cleanDisplayText(value) {
  return String(value ?? "")
    .replace(/<[^>]+>/g, "")
    .replaceAll("&nbsp;", " ")
    .replace(/\s+/g, " ")
    .trim();
}

function collectOperationTypes(value, found = new Set()) {
  if (Array.isArray(value)) {
    for (const child of value) collectOperationTypes(child, found);
  } else if (value && typeof value === "object") {
    if (typeof value.$type === "string") found.add(value.$type);
    for (const child of Object.values(value)) collectOperationTypes(child, found);
  }
  return [...found].sort();
}

function extractSequenceSkillIds(sequence) {
  if (!Array.isArray(sequence)) return [];
  const result = [];
  for (const entry of sequence) {
    if (typeof entry === "number" || typeof entry === "string") result.push(entry);
    else if (entry && typeof entry === "object") {
      for (const value of Object.values(entry)) {
        if (typeof value === "number" || /^\d+$/.test(String(value))) result.push(value);
      }
    }
  }
  return result;
}

function summarizeQuality(records) {
  const byQuality = {};
  const byMechanismQuality = {};
  for (const record of records) byQuality[record.quality] = (byQuality[record.quality] ?? 0) + 1;
  for (const record of records) {
    if (record.mechanism_quality) {
      byMechanismQuality[record.mechanism_quality] = (byMechanismQuality[record.mechanism_quality] ?? 0) + 1;
    }
  }
  const result = { total: records.length, by_quality: sortObject(byQuality) };
  if (Object.keys(byMechanismQuality).length) result.by_mechanism_quality = sortObject(byMechanismQuality);
  return result;
}

function normalizeKeyValueMap(value) {
  if (Array.isArray(value)) {
    return value.map((entry) => ({ source_skill_id: String(entry.id ?? ""), levels: entry.num ?? 0 }));
  }
  if (!value || typeof value !== "object") return [];
  return Object.entries(value).map(([key, levels]) => ({ source_skill_id: String(key), levels })).sort((a, b) => numericTextCompare(a.source_skill_id, b.source_skill_id));
}

function normalizeRawList(value) {
  return Array.isArray(value) ? value.map(normalizeRaw) : [];
}

function normalizeRaw(value) {
  if (Array.isArray(value)) return value.map(normalizeRaw);
  if (value && typeof value === "object") {
    if (Object.keys(value).length === 1 && Object.hasOwn(value, "Value")) return decimal(value);
    return Object.fromEntries(Object.entries(value).sort(([a], [b]) => a.localeCompare(b)).map(([key, child]) => [key, normalizeRaw(child)]));
  }
  return typeof value === "number" ? canonicalNumber(value) : value;
}

function decimal(value) {
  if (value === undefined || value === null || value === "") return null;
  if (typeof value === "object" && Object.hasOwn(value, "Value")) return decimal(value.Value);
  if (typeof value === "number") return canonicalNumber(value);
  if (typeof value === "string" && /^-?\d+(\.\d+)?([eE][+-]?\d+)?$/.test(value)) return canonicalNumber(Number(value));
  return null;
}

function decimals(values) {
  if (!Array.isArray(values)) return [];
  return values.map(decimal);
}

function scalar(value) {
  return value === undefined || value === null ? null : value;
}

function canonicalNumber(value) {
  if (!Number.isFinite(value)) return String(value);
  if (Object.is(value, -0)) return "0";
  return value.toString().replace("e+", "e");
}

function parseRarity(value) {
  const match = String(value ?? "").match(/(\d+)$/);
  return match ? Number(match[1]) : Number(value) || null;
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function readGameJson(file) {
  const content = fs.readFileSync(file, "utf8");
  const hashesAsText = content.replace(/("Hash"\s*:\s*)(\d{16,})/g, '$1"$2"');
  return JSON.parse(hashesAsText);
}

function writeJson(name, value) {
  fs.writeFileSync(path.join(outputRoot, name), `${JSON.stringify(value, null, 2)}\n`, "utf8");
}

function writePackIndex() {
  const files = fs.readdirSync(outputRoot)
    .filter((name) => name.endsWith(".json") && name !== "pack-index.json")
    .sort()
    .map((name) => ({ name, sha256: sha256File(path.join(outputRoot, name)) }));
  const digestInput = files.map((entry) => `${entry.name}\0${entry.sha256}\n`).join("");
  writeJson("pack-index.json", {
    schema_revision: "content-reference-pack-v1",
    files,
    pack_sha256: sha256Text(digestInput),
  });
}

function sha256File(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function sha256Text(value) {
  return crypto.createHash("sha256").update(value, "utf8").digest("hex");
}

function walkJson(root) {
  const result = [];
  const stack = [root];
  while (stack.length) {
    const current = stack.pop();
    for (const entry of fs.readdirSync(current, { withFileTypes: true }).sort((a, b) => b.name.localeCompare(a.name))) {
      const full = path.join(current, entry.name);
      if (entry.isDirectory()) stack.push(full);
      else if (entry.isFile() && entry.name.endsWith(".json") && !entry.name.endsWith(".layout.json")) result.push(full);
    }
  }
  return result.sort();
}

function groupBy(values, keyOf) {
  const result = new Map();
  for (const value of values) {
    const key = keyOf(value);
    const group = result.get(key) ?? [];
    group.push(value);
    result.set(key, group);
  }
  return result;
}

function unique(values) {
  return [...new Set(values)];
}

function uniqueByJson(values) {
  const seen = new Set();
  return values.filter((value) => {
    const key = JSON.stringify(value);
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

function without(value, key) {
  const { [key]: ignored, ...rest } = value;
  void ignored;
  return rest;
}

function sortObject(value) {
  return Object.fromEntries(Object.entries(value).sort(([a], [b]) => a.localeCompare(b)));
}

function slug(value) {
  const result = String(value ?? "")
    .normalize("NFKD")
    .replace(/[’']/g, "")
    .replace(/[^\p{Letter}\p{Number}]+/gu, "-")
    .replace(/^-|-$/g, "")
    .toLowerCase();
  return result || "unnamed";
}

function pad(value, width = 2) {
  return String(value).padStart(width, "0");
}

function byId(a, b) {
  return a.id.localeCompare(b.id);
}

function numericTextCompare(a, b) {
  return String(a).localeCompare(String(b), "en", { numeric: true });
}

function forwardSlash(value) {
  return value.replaceAll("\\", "/");
}

function parseArgs(values) {
  const result = {};
  for (let index = 0; index < values.length; index += 2) {
    const key = values[index];
    if (!key?.startsWith("--") || values[index + 1] === undefined) throw new Error(`Invalid argument near ${key ?? "<end>"}`);
    result[key.slice(2)] = values[index + 1];
  }
  return result;
}

function requiredPath(values, key, mustExist = true) {
  const value = values[key];
  if (!value) throw new Error(`Missing --${key}`);
  const resolved = path.resolve(value);
  if (mustExist && !fs.existsSync(resolved)) throw new Error(`Path does not exist for --${key}: ${resolved}`);
  return resolved;
}
