#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  ACCESS_DATE,
  canonical,
  createContext,
  decimal,
  sha256,
  slug,
  writeOrCheck,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const context = await createContext(root);
const outputs = new Map();

async function localRows(relative) {
  return JSON.parse(await fs.readFile(path.join(root, relative), "utf8"));
}
function localRef(relative, row, locator) {
  return {
    source_id: `source.goal09.inherited.${slug(relative)}.${slug(locator)}`,
    repository: "starclock",
    revision: "goal03-standard-universe-v1",
    path: relative,
    locator: String(locator),
    sha256: sha256(canonical(row)),
    access_date: ACCESS_DATE,
    evidence_quality: "ExactStructured",
  };
}
function fileEntry(sourcePath, locator, row) {
  return { sourcePath, locator: String(locator), row };
}
function ordered(records, fields = ["id"]) {
  return records.sort((left, right) => {
    for (const field of fields) {
      if (left[field] < right[field]) return -1;
      if (left[field] > right[field]) return 1;
    }
    return 0;
  });
}
function parameters(row) {
  return (row.ParamList ?? []).map(({ Value: value }, index) => ({
    index: index + 1,
    value: decimal(value),
  }));
}

const boundaryPolicy = await context.policyRef(
  "path-resonance-boundaries",
  "Apply the selected Path's AddMazeBuff program after Path selection at run start. Count distinct owned blessing identities per Path, re-evaluate Resonance Interplay thresholds after an accepted blessing-inventory mutation, and apply each unlocked binding once.",
  "Replace lifecycle boundaries, once scope or threshold counting if released engine evidence establishes different authoritative timing.",
);
const bonusPolicy = await context.policyRef(
  "trailblaze-bonus-boundary",
  "Execute the chosen Trailblaze Bonus as one accepted run-start Activity transaction. Reject an unaffordable cost without mutation and draw random rewards from stable eligible pools using a labeled Activity RNG stream.",
  "Replace transaction timing, failure behavior or RNG labels if released engine evidence supplies the authoritative bonus controller.",
);

const standardPathRelative =
  "content-reference/standard-universe-v1/paths.json";
const standardResonanceRelative =
  "content-reference/standard-universe-v1/resonances.json";
const standardPaths = await localRows(standardPathRelative);
const standardResonances = await localRows(standardResonanceRelative);
const manifest = await localRows(
  "content-manifests/swarm-disaster-v1/content-manifest.json",
);
const requiredPathIds = new Set(manifest.categories.paths.records
  .map(({ id }) => id));
const requiredResonanceIds = new Set(manifest.categories.resonances.records
  .map(({ id }) => id));
const pathEntries = await context.table("RogueDLCAeon");
const pathEntryByAeon = new Map(pathEntries.map((entry) => [
  String(entry.row.RogueAeonDisplayID),
  entry,
]));
const standardPathIndex = new Map(standardPaths.map((row, index) => [
  row.id,
  { row, index },
]));

const paths = [...requiredPathIds].map((id) => {
  const inherited = standardPathIndex.get(id);
  if (!inherited) throw new Error(`missing inherited Path ${id}`);
  const aeonId = String(inherited.row.source_ids[0]);
  const entry = pathEntryByAeon.get(aeonId);
  if (!entry) throw new Error(`missing RogueDLCAeon ${aeonId}`);
  const propagation = id === "universe.path.propagation";
  return {
    ...context.envelope({
      id: `swarm-disaster.path-binding.${slug(id)}`,
      kind: "SwarmPathBinding",
      nameEn: inherited.row.name_en,
      nameZh: inherited.row.name_zh_cn,
      summaryEn:
        `Swarm Disaster exposes the shared ${inherited.row.name_en} Path with its released Audience Die and Resonance groups.`,
      summaryZh:
        `寰宇蝗灾提供共享的${inherited.row.name_zh_cn}命途及其已发布的觐见之骰与回响组。`,
      ownership: "Shared",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        localRef(standardPathRelative, inherited.row, inherited.index),
        context.sourceRef(entry),
        boundaryPolicy,
      ],
      tags: [
        "path-binding",
        "shared",
        ...(propagation ? ["propagation"] : []),
        "project-policy",
      ],
    }),
    source_id: aeonId,
    shared_path_id: id,
    selectable: true,
    sort: entry.row.Sort,
    audience_die_id:
      `swarm-disaster.audience-die.${entry.row.AeonDiceID}`,
    mode_unlock_id: entry.row.UnlockID === undefined
      ? ""
      : `swarm-disaster.pathstrider-unlock.${entry.row.UnlockID}`,
    propagation_unlock: {
      is_propagation: propagation,
      required_unlock_id: propagation
        ? `swarm-disaster.pathstrider-unlock.${entry.row.UnlockID}`
        : "",
      unlock_state: propagation
        ? "ReleasedUnlockRowBound"
        : "NotApplicable",
    },
    resonance_id: inherited.row.resonance_id,
    formation_ids: inherited.row.formation_ids,
    battle_event_groups: [
      String(entry.row.BattleEventBuffGroup),
      String(entry.row.BattleEventEnhanceBuffGroup),
    ],
    extra_effect_ids: (entry.row.ExtraEffect ?? []).map(String),
  };
});
outputs.set("paths.json", ordered(paths, ["sort", "id"]));
const pathIdByAeon = new Map(paths.map((row) => [
  row.source_id,
  row.shared_path_id,
]));

const resonanceIndex = new Map(standardResonances.map((row, index) => [
  row.id,
  { row, index },
]));
const resonances = [...requiredResonanceIds].map((id) => {
  const inherited = resonanceIndex.get(id);
  if (!inherited) throw new Error(`missing inherited Resonance ${id}`);
  if (!requiredPathIds.has(inherited.row.path_id))
    throw new Error(`Resonance ${id} belongs to excluded Path`);
  return {
    ...context.envelope({
      id: `swarm-disaster.resonance-binding.${id.split(".").at(-1)}`,
      kind: "SwarmResonanceBinding",
      nameEn: inherited.row.name_en,
      nameZh: inherited.row.name_zh_cn,
      summaryEn:
        `Swarm Disaster reuses this shared ${inherited.row.kind.toLowerCase()} and its exact released modifier binding.`,
      summaryZh:
        `寰宇蝗灾复用该共享${inherited.row.kind === "Resonance" ? "命途回响" : "回响构音"}及其精确的已发布修改器绑定。`,
      ownership: "Shared",
      sourceRefs: [
        localRef(standardResonanceRelative, inherited.row, inherited.index),
      ],
      tags: [
        "resonance-binding",
        inherited.row.kind.toLowerCase(),
        "shared",
      ],
    }),
    source_id: String(inherited.row.source_ids[0]),
    shared_resonance_id: id,
    path_id: inherited.row.path_id,
    kind: inherited.row.kind,
    threshold: String(inherited.row.threshold),
    energy_max: inherited.row.energy_max,
    initial_energy: inherited.row.initial_energy,
    parameter_values: inherited.row.parameter_values,
    mechanic_tags: inherited.row.mechanic_tags,
    effect_program: {
      modifier_name: inherited.row.source_modifier_name,
      binding_type: inherited.row.source_binding_type,
      binding_key: inherited.row.source_binding_key,
      rule_ids: inherited.row.rule_ids,
    },
  };
});
outputs.set(
  "resonances.json",
  ordered(resonances, ["path_id", "kind", "id"]),
);

const boostAbilityPath =
  "Config/ConfigAbility/Level/Level_RogueBuff_Ability_DLC1_Other.json";
const boostDocument = await context.readSource(boostAbilityPath);
const abilityByName = new Map(boostDocument.AbilityList.map((ability, index) => [
  ability.Name,
  { ability, index },
]));
const boosts = pathEntries.map((entry) => {
  const sourceId = String(entry.row.EffectParam1[0]);
  const abilityName = `StageAbility_${sourceId}`;
  const abilityEntry = abilityByName.get(abilityName);
  const pathId = pathIdByAeon.get(String(entry.row.RogueAeonDisplayID));
  if (!abilityEntry || !pathId)
    throw new Error(`missing Path boost ${sourceId}`);
  const abilitySource = fileEntry(
    boostAbilityPath,
    `AbilityList[${abilityEntry.index}]`,
    abilityEntry.ability,
  );
  const pathBinding = paths.find(({ shared_path_id: id }) => id === pathId);
  return {
    ...context.envelope({
      id: `swarm-disaster.path-boost.${sourceId}`,
      kind: "SwarmPathBoost",
      nameEn: `${pathBinding.name_en} Path Boost`,
      nameZh: `${pathBinding.name_zh_cn}命途强化`,
      summaryEn:
        `Selecting ${pathBinding.name_en} applies released stage ability ${abilityName}.`,
      summaryZh:
        `选择${pathBinding.name_zh_cn}后应用已发布的关卡能力 ${abilityName}。`,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        context.sourceRef(entry),
        context.sourceRef(abilitySource),
        boundaryPolicy,
      ],
      tags: ["path-boost", "battle-projection", "project-policy"],
    }),
    source_id: sourceId,
    path_id: pathId,
    effect_program: {
      operation: entry.row.EffectType1,
      stage_ability: abilityName,
      level_parameters: (entry.row.EffectParam2 ?? []).map(decimal),
      source_program_sha256: sha256(canonical(abilityEntry.ability)),
    },
    application_boundary: "AfterPathSelectionAtRunStart",
  };
});
outputs.set("path-boosts.json", ordered(boosts, ["path_id", "id"]));

const groupEntries = await context.table("RogueBuffGroup");
const buffEntries = await context.table("RogueBuff");
const mazeEntries = await context.table("RogueMazeBuff");
const groupById = new Map(groupEntries.map((entry) => [
  String(entry.row.GMLOGNJAIGI),
  entry,
]));
const buffByTag = new Map(buffEntries.map((entry) => [
  String(entry.row.RogueBuffTag),
  entry,
]));
const mazeById = new Map();
for (const entry of mazeEntries) {
  const key = `${entry.row.ID}:${entry.row.Lv}`;
  if (!mazeById.has(key)) mazeById.set(key, entry);
}
const crossEntries = await context.table("RogueDLCAeonCross");
const interplays = crossEntries.map((cross) => {
  const group = groupById.get(String(cross.row.BuffGroup));
  if (!group || group.row.HECJCAMDGNO.length !== 1)
    throw new Error(`invalid Interplay group ${cross.row.BuffGroup}`);
  const sourceTag = String(group.row.HECJCAMDGNO[0]);
  const buff = buffByTag.get(sourceTag);
  if (!buff) throw new Error(`missing Interplay buff ${sourceTag}`);
  const maze = mazeById.get(`${buff.row.MazeBuffID}:${buff.row.MazeBuffLevel}`);
  if (!maze) throw new Error(`missing Interplay MazeBuff ${sourceTag}`);
  const mainPathId = pathIdByAeon.get(String(cross.row.MainAeonID));
  const subPathId = pathIdByAeon.get(String(cross.row.SubAeonID));
  if (!mainPathId || !subPathId)
    throw new Error(`Interplay ${sourceTag} references excluded Path`);
  const nameEn = context.text(maze.row.BuffName, "en")
    || `Resonance Interplay ${sourceTag}`;
  const nameZh = context.text(maze.row.BuffName, "zh_cn")
    || `回响交错 ${sourceTag}`;
  const descriptionEn = context.text(maze.row.BuffDesc, "en");
  const descriptionZh = context.text(maze.row.BuffDesc, "zh_cn");
  return {
    ...context.envelope({
      id: `swarm-disaster.resonance-interplay.${sourceTag}`,
      kind: "ResonanceInterplay",
      nameEn,
      nameZh,
      summaryEn:
        `${nameEn} unlocks at the released 3+3 Path thresholds and applies modifier ${maze.row.ModifierName}.`,
      summaryZh:
        `${nameZh}在已发布的 3+3 命途阈值下解锁，并应用修改器 ${maze.row.ModifierName}。`,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        context.sourceRef(cross),
        context.sourceRef(group),
        context.sourceRef(buff),
        context.sourceRef(maze),
        boundaryPolicy,
      ],
      tags: ["resonance-interplay", "battle-projection", "project-policy"],
    }),
    source_id: sourceTag,
    main_path_id: mainPathId,
    sub_path_id: subPathId,
    thresholds: {
      main_path_blessings: String(cross.row.MainAeonNum),
      sub_path_blessings: String(cross.row.SubAeonNum),
      comparison: "GreaterEqual",
      counting_policy: "DistinctOwnedBlessingIdentity",
    },
    effect_program: {
      buff_group_id: String(cross.row.BuffGroup),
      maze_buff_id: String(buff.row.MazeBuffID),
      modifier_name: maze.row.ModifierName,
      binding_type: maze.row.InBattleBindingType,
      binding_key: maze.row.InBattleBindingKey,
      parameters: parameters(maze.row),
      source_description_sha256_en: sha256(descriptionEn),
      source_description_sha256_zh_cn: sha256(descriptionZh),
    },
    application_boundary: "AfterAcceptedBlessingInventoryMutation",
    once_scope: `ResonanceInterplay:${sourceTag}`,
  };
});
outputs.set(
  "resonance-interplays.json",
  ordered(interplays, ["main_path_id", "sub_path_id", "id"]),
);

function bonusEffect(sourceId) {
  const programs = new Map([
    [101, [{
      order: 0,
      operation: "AddCosmicFragments",
      value: "150",
    }]],
    [102, [{
      order: 0,
      operation: "GrantRandomBlessings",
      count: "1",
      minimum_rarity: "1",
      maximum_rarity: "2",
      pool_binding_state: "DeferredToG09P2B1",
    }]],
    [103, [{
      order: 0,
      operation: "GrantRandomCurios",
      count: "1",
      category: "AnyEligible",
      pool_binding_state: "DeferredToG09P2B2",
    }]],
    [104, [{
      order: 0,
      operation: "SpendCosmicFragments",
      value: "100",
    }, {
      order: 1,
      operation: "GrantRandomCurios",
      count: "1",
      category: "ErrorCode",
      pool_binding_state: "DeferredToG09P2B2",
    }]],
    [105, [{
      order: 0,
      operation: "GrantRandomCurios",
      count: "2",
      category: "AnyEligible",
      pool_binding_state: "DeferredToG09P2B2",
    }, {
      order: 1,
      operation: "GrantRandomCurios",
      count: "1",
      category: "Negative",
      pool_binding_state: "DeferredToG09P2B2",
    }]],
    [106, [{
      order: 0,
      operation: "AdjustCountdown",
      value: "-2",
    }, {
      order: 1,
      operation: "GrantRandomBlessings",
      count: "3",
      minimum_rarity: "1",
      maximum_rarity: "2",
      pool_binding_state: "DeferredToG09P2B1",
    }]],
  ]);
  return programs.get(sourceId);
}

const bonusEntries = (await context.table("RogueBonus"))
  .filter(({ row }) => row.BonusID >= 101 && row.BonusID <= 106);
const bonusSummaries = new Map([
  [101, {
    en: "At run start, gain 150 Cosmic Fragments.",
    zh: "开局时获得 150 枚宇宙碎片。",
  }],
  [102, {
    en: "At run start, gain one random 1- or 2-star Blessing.",
    zh: "开局时获得一个随机 1 星或 2 星祝福。",
  }],
  [103, {
    en: "At run start, gain one random eligible Curio.",
    zh: "开局时获得一个随机可用奇物。",
  }],
  [104, {
    en: "Atomically spend 100 Cosmic Fragments and gain one random Error Code Curio.",
    zh: "以原子操作消耗 100 枚宇宙碎片，并获得一个随机错误代码奇物。",
  }],
  [105, {
    en: "At run start, gain two random eligible Curios and one random Negative Curio.",
    zh: "开局时获得两个随机可用奇物和一个随机负面奇物。",
  }],
  [106, {
    en: "At run start, reduce Countdown by 2 and gain three random 1- or 2-star Blessings.",
    zh: "开局时倒计时减少 2，并获得三个随机 1 星或 2 星祝福。",
  }],
]);
const bonuses = bonusEntries.map((entry) => {
  const sourceId = entry.row.BonusID;
  const nameEn = context.text(entry.row.BonusTitle, "en")
    || `Trailblaze Bonus ${sourceId}`;
  const nameZh = context.text(entry.row.BonusTitle, "zh_cn")
    || `开拓祝福 ${sourceId}`;
  const descriptionEn = context.text(entry.row.BonusDesc, "en")
    || `Apply Trailblaze Bonus event ${entry.row.BonusEvent}.`;
  const descriptionZh = context.text(entry.row.BonusDesc, "zh_cn")
    || `应用开拓祝福事件 ${entry.row.BonusEvent}。`;
  const summary = bonusSummaries.get(sourceId);
  if (!summary) throw new Error(`missing Trailblaze Bonus ${sourceId}`);
  return {
    ...context.envelope({
      id: `swarm-disaster.trailblaze-bonus.${sourceId}`,
      kind: "SwarmTrailblazeBonus",
      nameEn,
      nameZh,
      summaryEn: summary.en,
      summaryZh: summary.zh,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [context.sourceRef(entry), bonusPolicy],
      tags: ["trailblaze-bonus", "activity-operation", "project-policy"],
    }),
    source_id: String(sourceId),
    bonus_event: String(entry.row.BonusEvent),
    effect_program: {
      transaction: "AtomicAcceptedActivityOperations",
      operations: bonusEffect(sourceId),
      random_stream: `swarm-disaster.trailblaze-bonus.${sourceId}`,
      source_description_sha256_en: sha256(descriptionEn),
      source_description_sha256_zh_cn: sha256(descriptionZh),
    },
    application_boundary: "AfterTrailblazeBonusSelectionAtRunStart",
  };
});
outputs.set("bonuses.json", ordered(bonuses));

await writeOrCheck(context, outputs, check);
console.log(
  `Swarm Disaster Paths ${check ? "verified" : "generated"}: ` +
  `${paths.length} Paths, ${resonances.length} Resonances/Formations, ` +
  `${boosts.length} boosts, ${interplays.length} Interplays and ` +
  `${bonuses.length} Trailblaze Bonuses.`,
);
