#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  ACCESS_DATE,
  ROW_SCHEMA,
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

async function localRows(relative) {
  return JSON.parse(await fs.readFile(path.join(root, relative), "utf8"));
}

function localRef(relative, row, locator) {
  return {
    source_id: `source.goal08.inherited.${slug(relative)}.${slug(locator)}`,
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

function textHash(reference) {
  return reference?.Hash === undefined ? "" : String(reference.Hash);
}

function parameters(row) {
  return (row.ParamList ?? []).map(({ Value: value }, index) => ({
    index: index + 1,
    value: decimal(value),
  }));
}

const standardPaths = await localRows(
  "content-reference/standard-universe-v1/paths.json",
);
const standardResonances = await localRows(
  "content-reference/standard-universe-v1/resonances.json",
);
const dicePathValues = await localRows(
  "content-reference/gold-and-gears-v1/dice-path-values.json",
);
const manifest = await localRows(
  "content-manifests/gold-and-gears-v1/content-manifest.json",
);
const aeonEntries = await context.table("RogueNousAeon");
const bonusEntries = await context.table("RogueBonus");
const crossEntries = await context.table("RogueNousAeonCross");
const groupEntries = await context.table("RogueBuffGroup");
const buffEntries = await context.table("RogueBuff");
const mazeEntries = await context.table("RogueMazeBuff");
const abilityDocument = await context.readSource(
  "Config/ConfigAbility/Level/Level_RogueBuff_Ability_Nous.json",
);

const pathByAeon = new Map(standardPaths.map((row) => [
  String(row.source_ids[0]),
  row,
]));
const aeonEntryById = new Map(aeonEntries.map((entry) => [
  String(entry.row.AeonID),
  entry,
]));
const resonanceById = new Map(standardResonances.map((row) => [row.id, row]));
const mazeEntryById = new Map();
for (const entry of mazeEntries) {
  const key = `${entry.row.ID}:${entry.row.Lv}`;
  if (!mazeEntryById.has(key)) mazeEntryById.set(key, entry);
}
const buffEntryByTag = new Map(buffEntries.map((entry) => [
  String(entry.row.RogueBuffTag),
  entry,
]));
const groupEntryById = new Map(groupEntries.map((entry) => [
  String(entry.row.GMLOGNJAIGI),
  entry,
]));
const abilityByName = new Map(
  abilityDocument.AbilityList.map((ability, index) => [
    ability.Name,
    { ability, index },
  ]),
);

const abilityProperty = new Map([
  ["1", "ShieldTakenRatio"],
  ["2", "StatusProbabilityBase"],
  ["3", "DotDamageAddedRatio"],
  ["4", "HealTakenRatio"],
  ["5", "CriticalDamageBase"],
  ["6", "AllDamageTypeAddedRatio"],
  ["7", "FollowUpAttackDamageRatio"],
  ["8", "BasicAttackDamageRatio"],
  ["9", "UltimateDamageRatio"],
]);
const boostStat = new Map([
  ["1", "ShieldGain"],
  ["2", "EffectHitRate"],
  ["3", "DamageOverTime"],
  ["4", "OutgoingHealing"],
  ["5", "CriticalDamage"],
  ["6", "DamageDealt"],
  ["7", "FollowUpAttackDamage"],
  ["8", "BasicAttackDamage"],
  ["9", "UltimateDamage"],
]);

const controllerPolicyRef = await context.policyRef(
  "resonance-extrapolation-controller",
  "Released groups prove each Path's base and enhanced BattleEvent bindings, " +
  "and released text proves that Resonance Extrapolation acts during the Third " +
  "Plane boss battle. The pinned rows do not expose the complete offered-Path " +
  "enumeration, formation draw, action-gauge initialization, tie ordering, or " +
  "the generic player-to-enemy polarity transform. Candidate ordering is by " +
  "stable source tag; selected formations use the seeded Activity stream; " +
  "unresolved action and polarity lowering fails closed.",
  "Replace when pinned released engine code proves candidate enumeration, " +
  "formation selection, action scheduling and target-polarity lowering.",
);
const extrapolationPublicRef = context.publicRef({
  id: "gold-gears-resonance-extrapolation-boundary",
  url: "https://honkai-star-rail.fandom.com/wiki/" +
    "Simulated_Universe%3A_Gold_and_Gears/Exploration",
  locator: "Resonance Extrapolation",
  fact: "Resonance Extrapolation occurs during the Third Plane boss fight, " +
    "simulates Path Resonances and has a serious impact on all allies.",
});

const paths = standardPaths.map((standardRow, index) => {
  const aeonId = String(standardRow.source_ids[0]);
  const aeonEntry = aeonEntryById.get(aeonId);
  if (!aeonEntry) throw new Error(`missing RogueNous Aeon ${aeonId}`);
  return {
    ...context.envelope({
      id: standardRow.id,
      kind: "Path",
      nameEn: standardRow.name_en,
      nameZh: standardRow.name_zh_cn,
      summaryEn:
        `Gold and Gears reuses the released ${standardRow.name_en} Path identity.`,
      summaryZh:
        `黄金与机械复用已发布的${standardRow.name_zh_cn}命途身份。`,
      ownership: "Shared",
      sourceRefs: [
        localRef(
          "content-reference/standard-universe-v1/paths.json",
          standardRow,
          index,
        ),
        context.sourceRef(aeonEntry),
      ],
      tags: ["path", "shared"],
    }),
    source_id: aeonId,
    sort: aeonEntry.row.Sort,
    buff_type: aeonEntry.row.RogueBuffType,
    shared_resonance_id: standardRow.resonance_id,
    shared_formation_ids: standardRow.formation_ids,
    path_boost_id: `gold-gears.path-boost.${aeonEntry.row.EffectParam1[0]}`,
    normal_battle_event_group: String(aeonEntry.row.BattleEventBuffGroup),
    enhanced_battle_event_group:
      String(aeonEntry.row.BattleEventEnhanceBuffGroup),
  };
}).sort((left, right) => left.sort - right.sort || left.id.localeCompare(right.id));

const pathIdByAeon = new Map(paths.map((row) => [row.source_id, row.id]));

const resonances = standardResonances.map((standardRow, index) => {
  const sourceId = String(standardRow.source_ids[0]);
  const mazeEntry = mazeEntryById.get(`${sourceId}:1`);
  if (!mazeEntry) throw new Error(`missing shared Resonance ${sourceId}`);
  return {
    ...context.envelope({
      id: standardRow.id,
      kind: "Resonance",
      nameEn: standardRow.name_en,
      nameZh: standardRow.name_zh_cn,
      summaryEn:
        `Gold and Gears exposes this shared ${standardRow.kind.toLowerCase()} ` +
        "through its selected-Path BattleEvent group.",
      summaryZh:
        `黄金与机械通过所选命途的战斗事件组提供该共享${standardRow.kind === "Resonance" ? "命途回响" : "回响构音"}。`,
      ownership: "Shared",
      sourceRefs: [
        localRef(
          "content-reference/standard-universe-v1/resonances.json",
          standardRow,
          index,
        ),
        context.sourceRef(mazeEntry),
      ],
      tags: ["path-resonance", "shared", standardRow.kind.toLowerCase()],
    }),
    source_id: sourceId,
    path_id: standardRow.path_id,
    resonance_kind: standardRow.kind,
    threshold: standardRow.threshold,
    energy_max: standardRow.energy_max,
    initial_energy: standardRow.initial_energy,
    parameter_values: standardRow.parameter_values,
    mechanic_tags: standardRow.mechanic_tags,
    source_modifier_name: standardRow.source_modifier_name,
    source_binding_type: standardRow.source_binding_type,
    source_binding_key: standardRow.source_binding_key,
    inherited_rule_ids: standardRow.rule_ids,
    source_description_sha256_en:
      standardRow.source_description_sha256_en,
    source_description_sha256_zh_cn:
      standardRow.source_description_sha256_zh_cn,
  };
}).sort((left, right) =>
  left.path_id.localeCompare(right.path_id)
  || left.resonance_kind.localeCompare(right.resonance_kind)
  || left.id.localeCompare(right.id));

const pathBoosts = aeonEntries.map((entry) => {
  const aeonId = String(entry.row.AeonID);
  const sourceId = String(entry.row.EffectParam1[0]);
  const pathRow = pathByAeon.get(aeonId);
  const abilityName = `StageAbility_${sourceId}`;
  const abilityEntry = abilityByName.get(abilityName);
  if (!pathRow || !abilityEntry)
    throw new Error(`missing Path boost binding ${aeonId}:${sourceId}`);
  const abilitySource = fileEntry(
    "Config/ConfigAbility/Level/Level_RogueBuff_Ability_Nous.json",
    `AbilityList[${abilityEntry.index}]`,
    abilityEntry.ability,
  );
  const values = dicePathValues
    .filter(({ path_source_id: id }) => id === aeonId)
    .sort((left, right) => left.dice_source_id.localeCompare(right.dice_source_id));
  const descriptionEn = context.text(entry.row.EffectDesc1, "en");
  const descriptionZh = context.text(entry.row.EffectDesc1, "zh_cn");
  return {
    ...context.envelope({
      id: `gold-gears.path-boost.${sourceId}`,
      kind: "PathBoost",
      nameEn: `${pathRow.name_en} Path Boost`,
      nameZh: `${pathRow.name_zh_cn}命途强化`,
      summaryEn:
        `Custom Dice passive triggers add the selected ${pathRow.name_en} ` +
        `boost to ${boostStat.get(aeonId)}.`,
      summaryZh:
        `自定义骰被动触发时，叠加所选${pathRow.name_zh_cn}命途的` +
        `${boostStat.get(aeonId)}强化。`,
      sourceRefs: [
        context.sourceRef(entry),
        context.sourceRef(abilitySource),
      ],
      tags: ["battle-stat", "path-boost"],
    }),
    source_id: sourceId,
    path_id: pathRow.id,
    aeon_source_id: aeonId,
    effect_type: entry.row.EffectType1,
    ability_name: abilityName,
    target_team: "TeamLight",
    target_property: abilityProperty.get(aeonId),
    boost_stat: boostStat.get(aeonId),
    stacking: "AdditiveContribution",
    source_value_conversion: "PercentInputDividedBy100ByStageAbility",
    dice_path_value_ids: values.map(({ id }) => id),
    allowed_increment_values: [...new Set(values.map(({ boost_value: value }) =>
      value))].sort((left, right) => Number(left) - Number(right)),
    description_text_hash: textHash(entry.row.EffectDesc1),
    source_description_sha256_en: sha256(descriptionEn),
    source_description_sha256_zh_cn: sha256(descriptionZh),
    rule_contribution_id: `gold-gears.rule.path-boost.${sourceId}`,
  };
}).sort((left, right) => left.path_id.localeCompare(right.path_id));

function buffBinding(sourceTag) {
  const buffEntry = buffEntryByTag.get(String(sourceTag));
  if (!buffEntry) throw new Error(`missing RogueBuff tag ${sourceTag}`);
  const mazeEntry = mazeEntryById.get(
    `${buffEntry.row.MazeBuffID}:${buffEntry.row.MazeBuffLevel}`,
  );
  if (!mazeEntry) throw new Error(`missing RogueMazeBuff tag ${sourceTag}`);
  return { buffEntry, mazeEntry };
}

const extrapolations = manifest.categories.resonance_extrapolations.records
  .map((record) => {
    const groupEntry = groupEntryById.get(String(record.buff_group_id));
    const { buffEntry, mazeEntry } = buffBinding(record.id);
    const sharedId = `universe.resonance.${buffEntry.row.MazeBuffID}`;
    const shared = resonanceById.get(sharedId);
    const pathId = pathIdByAeon.get(String(record.aeon_id));
    if (!groupEntry || !shared || !pathId)
      throw new Error(`incomplete Extrapolation ${record.id}`);
    const enhanced =
      buffEntry.row.BattleEventBuffType === "BattleEventBuffEnhance";
    return {
      ...context.envelope({
        id: `gold-gears.resonance-extrapolation.${record.id}`,
        kind: "ResonanceExtrapolation",
        nameEn: enhanced
          ? `Formation Extrapolation: ${shared.name_en.replace(/^Resonance Formation: /u, "")}`
          : `Resonance Extrapolation: ${pathByAeon.get(String(record.aeon_id)).name_en}`,
        nameZh: enhanced
          ? `构音推演：${shared.name_zh_cn.replace(/^回响构音：/u, "")}`
          : `回响推演：「${pathByAeon.get(String(record.aeon_id)).name_zh_cn}」`,
        summaryEn:
          "This Third Plane boss BattleEvent binding reuses a released shared " +
          "Resonance effect under the Resonance Extrapolation controller.",
        summaryZh:
          "该第三位面首领战斗事件绑定在回响推演控制器下复用已发布的共享回响效果。",
        sourceRefs: [
          context.sourceRef(groupEntry),
          context.sourceRef(buffEntry),
          context.sourceRef(mazeEntry),
          extrapolationPublicRef,
          controllerPolicyRef,
        ],
        tags: [
          "resonance-extrapolation",
          enhanced ? "formation" : "resonance",
        ],
      }),
      mechanism_quality: "ExactStructuredWithPolicyFields",
      quality_overrides: [{
        field: "controller_policy",
        evidence_quality: "ProjectPolicy",
        policy_id: "resonance-extrapolation-controller-v1",
        replacement_condition:
          "Replace when pinned engine evidence proves selection, action " +
          "scheduling and target-polarity lowering.",
      }],
      source_id: String(record.id),
      path_id: pathId,
      aeon_source_id: String(record.aeon_id),
      buff_group_id: String(record.buff_group_id),
      enhanced,
      shared_resonance_id: sharedId,
      shared_resonance_kind: shared.kind,
      source_battle_event_type: buffEntry.row.BattleEventBuffType,
      source_modifier_name: mazeEntry.row.ModifierName,
      source_binding_type: mazeEntry.row.InBattleBindingType,
      source_binding_key: mazeEntry.row.InBattleBindingKey,
      source_parameters: parameters(mazeEntry.row),
      source_description_sha256_en: shared.source_description_sha256_en,
      source_description_sha256_zh_cn: shared.source_description_sha256_zh_cn,
      battle_scope: "ThirdPlaneBossBattle",
      controller_policy: {
        policy_id: "resonance-extrapolation-controller-v1",
        evidence_quality: "ProjectPolicy",
        candidate_order: "stable-source-tag-ascending",
        formation_selection: "seeded-activity-stream-without-replacement",
        base_formation_count: "1",
        auxiliary_conundrum_bonus_count: "1",
        action_and_polarity_lowering: "UnresolvedFailClosed",
      },
      rule_contribution_id:
        `gold-gears.rule.resonance-extrapolation.${record.id}`,
    };
  }).sort((left, right) =>
    left.path_id.localeCompare(right.path_id)
    || Number(left.enhanced) - Number(right.enhanced)
    || left.id.localeCompare(right.id));

const interplays = crossEntries.map((crossEntry) => {
  const groupEntry = groupEntryById.get(String(crossEntry.row.BuffGroup));
  if (!groupEntry || groupEntry.row.HECJCAMDGNO.length !== 1)
    throw new Error(`invalid interplay group ${crossEntry.row.BuffGroup}`);
  const sourceTag = String(groupEntry.row.HECJCAMDGNO[0]);
  const { buffEntry, mazeEntry } = buffBinding(sourceTag);
  const mainPathId = pathIdByAeon.get(String(crossEntry.row.MainAeonID));
  const subPathId = pathIdByAeon.get(String(crossEntry.row.SubAeonID));
  const nameEn = context.text(mazeEntry.row.BuffName, "en");
  const nameZh = context.text(mazeEntry.row.BuffName, "zh_cn");
  const descriptionEn = context.text(mazeEntry.row.BuffDesc, "en");
  const descriptionZh = context.text(mazeEntry.row.BuffDesc, "zh_cn");
  return {
    ...context.envelope({
      id: `gold-gears.resonance-interplay.${sourceTag}`,
      kind: "ResonanceInterplay",
      nameEn,
      nameZh,
      summaryEn:
        "This Gold and Gears Resonance Interplay combines the selected main " +
        "Path with its released secondary-Path threshold.",
      summaryZh:
        "该黄金与机械回响交错将所选主命途与已发布的副命途阈值组合。",
      sourceRefs: [
        context.sourceRef(crossEntry),
        context.sourceRef(groupEntry),
        context.sourceRef(buffEntry),
        context.sourceRef(mazeEntry),
      ],
      tags: ["path-resonance", "resonance-interplay"],
    }),
    source_id: sourceTag,
    main_path_id: mainPathId,
    sub_path_id: subPathId,
    main_blessing_threshold: crossEntry.row.MainAeonNum,
    sub_blessing_threshold: crossEntry.row.SubAeonNum,
    buff_group_id: String(crossEntry.row.BuffGroup),
    shared_maze_buff_id: String(buffEntry.row.MazeBuffID),
    source_modifier_name: mazeEntry.row.ModifierName,
    source_binding_type: mazeEntry.row.InBattleBindingType,
    source_binding_key: mazeEntry.row.InBattleBindingKey,
    source_parameters: parameters(mazeEntry.row),
    name_text_hash: textHash(mazeEntry.row.BuffName),
    description_text_hash: textHash(mazeEntry.row.BuffDesc),
    source_description_sha256_en: sha256(descriptionEn),
    source_description_sha256_zh_cn: sha256(descriptionZh),
    rule_contribution_id: `gold-gears.rule.resonance-interplay.${sourceTag}`,
  };
}).sort((left, right) =>
  left.main_path_id.localeCompare(right.main_path_id)
  || left.sub_path_id.localeCompare(right.sub_path_id)
  || left.id.localeCompare(right.id));

function bonusContribution(sourceId) {
  return new Map([
    [201, {
      operation: "AddCosmicFragments",
      scope: "Activity",
      value: "150",
      unit: "CosmicFragment",
      mechanism_quality: "ExactStructured",
    }],
    [202, {
      operation: "OfferRandomBlessing",
      scope: "Activity",
      choice_count: "1",
      minimum_rarity: "1",
      maximum_rarity: "2",
      pool_binding_state: "DeferredToG08P2B1",
      mechanism_quality: "ExactStructured",
    }],
    [203, {
      operation: "OfferRandomCurio",
      scope: "Activity",
      choice_count: "1",
      pool_binding_state: "DeferredToG08P2B2",
      mechanism_quality: "ExactStructured",
    }],
    [204, {
      operation: "AddDiceCheatAttempts",
      scope: "Activity",
      value: "1",
      unit: "Count",
      mechanism_quality: "ExactStructured",
    }],
    [205, {
      operation: "GrantCuriosByCategory",
      scope: "Activity",
      grants: [
        { category: "Negative", count: "1" },
        { category: "ErrorCode", count: "1" },
      ],
      pool_binding_state: "DeferredToG08P2B2",
      mechanism_quality: "ExactStructured",
    }],
  ]).get(sourceId);
}

const bonuses = bonusEntries
  .filter(({ row }) => row.BonusID >= 201 && row.BonusID <= 205)
  .map((entry) => {
    const sourceId = entry.row.BonusID;
    const nameEn = context.text(entry.row.BonusTitle, "en");
    const nameZh = context.text(entry.row.BonusTitle, "zh_cn");
    const descriptionEn = context.text(entry.row.BonusDesc, "en");
    const descriptionZh = context.text(entry.row.BonusDesc, "zh_cn");
    return {
      ...context.envelope({
        id: `gold-gears.trailblaze-bonus.${sourceId}`,
        kind: "TrailblazeBonus",
        nameEn,
        nameZh,
        summaryEn:
          `This selectable Trailblaze Bonus grants the released ${nameEn} effect.`,
        summaryZh:
          `该可选开拓祝福提供已发布的${nameZh}效果。`,
        sourceRefs: [context.sourceRef(entry)],
        tags: ["trailblaze-bonus"],
      }),
      source_id: String(sourceId),
      bonus_event_id: String(entry.row.BonusEvent),
      title_text_hash: textHash(entry.row.BonusTitle),
      description_text_hash: textHash(entry.row.BonusDesc),
      tag_text_hash: textHash(entry.row.BonusTag),
      source_description_sha256_en: sha256(descriptionEn),
      source_description_sha256_zh_cn: sha256(descriptionZh),
      effect_contributions: [bonusContribution(sourceId)],
      rule_contribution_id: `gold-gears.rule.trailblaze-bonus.${sourceId}`,
    };
  }).sort((left, right) => left.id.localeCompare(right.id));

for (const rows of [paths, resonances, pathBoosts, extrapolations, interplays, bonuses])
  for (const row of rows)
    if (row.schema_revision !== ROW_SCHEMA)
      throw new Error(`${row.id} schema revision drift`);

await writeOrCheck(context, new Map([
  ["paths.json", paths],
  ["resonances.json", resonances],
  ["path-boosts.json", pathBoosts],
  ["resonance-extrapolations.json", extrapolations],
  ["resonance-interplays.json", interplays],
  ["bonuses.json", bonuses],
]), check);
console.log(
  `${check ? "Checked" : "Wrote"} 9 Paths, 36 Resonances, 9 Path boosts, ` +
  "36 Resonance Extrapolation bindings, 18 Interplays and 5 Trailblaze Bonuses.",
);
