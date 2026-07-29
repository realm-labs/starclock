#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  createContext,
  decimal,
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

function localized(reference, fallbackEn, fallbackZh) {
  return {
    en: context.text(reference, "en") || fallbackEn,
    zh: context.text(reference, "zh_cn") || fallbackZh,
  };
}
function common(values) {
  return context.envelope(values);
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

const targetPolicy = await context.policyRef(
  "dice-target-rules",
  "Resolve each face against its typed effect candidate filter, sort candidates by stable domain/node ID, and treat an empty legal set as a no-op without consuming additional randomness.",
  "Replace the per-effect candidate filter, cardinality or empty-target behavior when released engine evidence provides an exact rule.",
);
const faceEntries = await context.table("RogueDLCAeonDiceSurface");
const diceFaces = [];
const targetRules = [];
for (const face of faceEntries) {
  const faceId = face.row.AeonSurfaceDiceID;
  const name = localized(
    face.row.DiceSurfaceName,
    `Audience Die Face ${faceId}`,
    `觐见行迹骰面 ${faceId}`,
  );
  const description = localized(
    face.row.DiceSurfaceDesc,
    `Execute ${face.row.DiceEffectType}.`,
    `执行 ${face.row.DiceEffectType}。`,
  );
  diceFaces.push({
    ...common({
      id: `swarm-disaster.dice-face.${faceId}`,
      kind: "AudienceDieFace",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn: description.en,
      summaryZh: description.zh,
      sourceRefs: [context.sourceRef(face)],
      tags: ["dice-face", face.row.DiceEffectType],
    }),
    source_id: String(faceId),
    audience_die_id: `swarm-disaster.audience-die.${face.row.AeonDiceID}`,
    sort: face.row.Sort,
    rarity_id: `swarm-disaster.dice-rarity.${face.row.Rarity}`,
    activation_stage: face.row.DiceActiveStage,
    target_rule_id: `swarm-disaster.dice-target.${faceId}`,
    effect_program: [{
      order: 0,
      operation: face.row.DiceEffectType,
      parameters: (face.row.DiceEffectParam ?? []).map(String),
      description_parameters: (face.row.DescParam ?? []).map(decimal),
      extra_effect_refs: (face.row.ExtraEffect ?? []).map((id) =>
        `source-effect.${id}`),
    }],
  });
  targetRules.push({
    ...common({
      id: `swarm-disaster.dice-target.${faceId}`,
      kind: "DiceTargetRule",
      nameEn: `${name.en} Target Rule`,
      nameZh: `${name.zh}目标规则`,
      summaryEn:
        `Typed ${face.row.DiceEffectType} candidates use stable ordering and an explicit empty-target fallback.`,
      summaryZh:
        `类型化 ${face.row.DiceEffectType} 候选使用稳定排序与显式空目标回退。`,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [context.sourceRef(face), targetPolicy],
      tags: ["dice-target", face.row.DiceEffectType, "project-policy"],
    }),
    source_id: String(faceId),
    candidate_filter: {
      effect_type: face.row.DiceEffectType,
      authored_parameters: (face.row.DiceEffectParam ?? []).map(String),
    },
    ordering: "StableDomainThenNodeId",
    cardinality: "AuthoredEffectDefined",
    no_legal_target: "NoOp",
  });
}
outputs.set("dice-faces.json", ordered(
  diceFaces,
  ["audience_die_id", "sort", "id"],
));
outputs.set("dice-target-rules.json", ordered(targetRules));

const rarityEntries = await context.table("RogueDLCDiceSurfaceRarity");
const rarityNames = {
  1: ["Rare", "稀有"],
  2: ["Epic", "史诗"],
  3: ["Legendary", "传奇"],
};
const rarities = rarityEntries.map((rarity) => {
  const [nameEn, nameZh] = rarityNames[rarity.row.Rarity]
    ?? [`Rank ${rarity.row.Rarity}`, `等级 ${rarity.row.Rarity}`];
  return {
    ...common({
      id: `swarm-disaster.dice-rarity.${rarity.row.Rarity}`,
      kind: "DiceFaceRarity",
      nameEn,
      nameZh,
      summaryEn:
        `Released Audience Die face rarity rank ${rarity.row.Rarity}.`,
      summaryZh:
        `已发布的觐见行迹骰面稀有度等级 ${rarity.row.Rarity}。`,
      sourceRefs: [context.sourceRef(rarity)],
      tags: ["dice-rarity"],
    }),
    source_id: String(rarity.row.Rarity),
    rank: rarity.row.Rarity,
    name_color: rarity.row.NameColor,
  };
});
outputs.set("dice-rarities.json", ordered(rarities));

const commonConstants = await context.table("RogueDLCConstValueCommon");
const abandon = commonConstants.find(({ row }) =>
  row.ConstValueName === "RogueDLC_DiceSurface_AbandonReward");
const clientConstants = await context.table("RogueDLCConstValueClient");
const skipUnlock = clientConstants.find(({ row }) =>
  row.ConstValueName === "RogueDLC_SkipRoll_Unlock");
if (!abandon || !skipUnlock) throw new Error("missing dice control constants");
const controlPolicy = await context.policyRef(
  "dice-roll-controls",
  "Roll samples from authored ordered faces; reroll and cheat consume one typed charge; abandon returns the released reward; unavailable controls reject without state mutation.",
  "Replace resource accounting, result timing or selection behavior when released engine evidence supplies an exact control protocol.",
);
const controlDefinitions = [
  {
    id: "roll",
    operation: "Roll",
    resourceCost: { resource: "None", amount: "0" },
    fallback: "RejectEmptyFaceSet",
  },
  {
    id: "reroll",
    operation: "Reroll",
    resourceCost: { resource: "RerollCharge", amount: "1" },
    fallback: "RejectInsufficientCharge",
  },
  {
    id: "cheat",
    operation: "Cheat",
    resourceCost: { resource: "CheatCharge", amount: "1" },
    fallback: "RejectInsufficientChargeOrInvalidFace",
  },
  {
    id: "abandon",
    operation: "Abandon",
    resourceCost: { resource: "SelectedFace", amount: "1" },
    fallback: "RejectWithoutSelectedFace",
  },
];
const controls = controlDefinitions.map((definition) => ({
  ...common({
    id: `swarm-disaster.dice-control.${definition.id}`,
    kind: "DiceRollControl",
    nameEn: `Audience Die ${definition.operation}`,
    nameZh: `觐见行迹骰${{
      Roll: "投掷",
      Reroll: "重投",
      Cheat: "作弊",
      Abandon: "放弃",
    }[definition.operation]}`,
    summaryEn:
      `${definition.operation} uses a typed resource transition and stable result ordering.`,
    summaryZh:
      `${definition.operation} 使用类型化资源变更与稳定结果顺序。`,
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [
      context.sourceRef(abandon),
      context.sourceRef(skipUnlock),
      controlPolicy,
    ],
    tags: ["dice-control", definition.id, "project-policy"],
  }),
  operation: definition.operation,
  resource_cost: definition.resourceCost,
  result_order: "AuthoredSortThenStableFaceId",
  fallback_policy: definition.fallback,
  abandon_reward: definition.operation === "Abandon" ? "10" : "0",
  unlock_id: definition.operation === "Abandon"
    ? String(skipUnlock.row.Value.IntValue)
    : "",
}));
outputs.set("dice-roll-controls.json", ordered(controls));

await writeOrCheck(context, outputs, check);
console.log(
  `Swarm Disaster dice faces ${check ? "verified" : "generated"}: ` +
  `${diceFaces.length} faces, ${rarities.length} rarities, ` +
  `${targetRules.length} target rules and ${controls.length} controls.`,
);
