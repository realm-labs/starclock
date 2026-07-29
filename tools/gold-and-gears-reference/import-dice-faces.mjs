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

function ordered(records, fields = ["id"]) {
  return records.sort((left, right) => {
    for (const field of fields) {
      const a = left[field];
      const b = right[field];
      if (a < b) return -1;
      if (a > b) return 1;
    }
    return 0;
  });
}
function sourceIds(values) {
  return (values ?? []).map(String);
}
function localized(reference, fallbackEn, fallbackZh) {
  return {
    en: context.text(reference, "en") || fallbackEn,
    zh: context.text(reference, "zh_cn") || fallbackZh,
  };
}
function textHash(reference) {
  return reference?.Hash === undefined ? "" : String(reference.Hash);
}
function parameters(values) {
  return (values ?? []).map(({ Value: value }) => decimal(value));
}

const slotEntries = await context.table("RogueNousDiceSlot");
const slots = slotEntries.map((entry) => {
  const name = localized(
    entry.row.SlotName,
    `Dice Slot ${entry.row.SlotID}`,
    `骰子槽位 ${entry.row.SlotID}`,
  );
  const upgradedName = localized(
    entry.row.UpgradedSlotName,
    `Upgraded Dice Slot ${entry.row.SlotID}`,
    `升级骰子槽位 ${entry.row.SlotID}`,
  );
  return {
    ...context.envelope({
      id: `gold-gears.dice-slot.${entry.row.SlotID}`,
      kind: "DiceSlot",
      nameEn: `${name.en} ${entry.row.SlotID}`,
      nameZh: `${name.zh} ${entry.row.SlotID}`,
      summaryEn:
        `Slot ${entry.row.SlotID} accepts rarity ${entry.row.MaxRarity} by default and rarity ${entry.row.ExtraMaxRarity ?? entry.row.MaxRarity} after its released upgrade.`,
      summaryZh:
        `槽位 ${entry.row.SlotID} 默认接受 ${entry.row.MaxRarity} 星骰面，按已发布升级后接受 ${entry.row.ExtraMaxRarity ?? entry.row.MaxRarity} 星骰面。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["custom-dice", "dice-slot"],
    }),
    source_id: String(entry.row.SlotID),
    slot_index: entry.row.SlotID,
    base_name_text_hash: textHash(entry.row.SlotName),
    upgraded_name_en: upgradedName.en,
    upgraded_name_zh_cn: upgradedName.zh,
    upgraded_name_text_hash: textHash(entry.row.UpgradedSlotName),
    base_max_rarity: entry.row.MaxRarity,
    extra_max_rarity: entry.row.ExtraMaxRarity ?? null,
    upgraded_max_rarity: entry.row.ExtraMaxRarity ?? entry.row.MaxRarity,
  };
});
outputs.set("dice-slots.json", ordered(slots, ["slot_index", "id"]));

const tagCodeBySourceId = new Map(Object.entries({
  2: "SpecialType",
  3: "BlockChange",
  4: "Move",
  6: "Mark",
  7: "Buff",
  8: "BuffProMax",
  9: "Miracle",
  10: "Coin",
  11: "Replicate",
  12: "ActionPoint",
}));
const tagPolicy = await context.policyRef(
  "dice-face-filter-tag-code-map",
  "The released filter-tag rows provide numeric IDs and localized concepts while face rows provide mechanical string codes without an explicit join. This one-to-one semantic mapping makes the join deterministic and auditable.",
  "Replace when a pinned released table or engine enum exposes the numeric filter-tag to mechanical-code relation.",
);
const tagEntries = await context.table("RogueNousSurfaceTag");
const tags = tagEntries.map((entry) => {
  const id = String(entry.row.TagID);
  const mechanicalCode = tagCodeBySourceId.get(id);
  if (!mechanicalCode) throw new Error(`surface tag ${id} has no code policy`);
  const name = localized(
    entry.row.TagName,
    `Dice Face Tag ${id}`,
    `骰面标签 ${id}`,
  );
  return {
    ...context.envelope({
      id: `gold-gears.dice-face-tag.${id}`,
      kind: "DiceFaceTag",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `Filter tag ${id} maps to mechanical face code ${mechanicalCode} under the replaceable project join policy.`,
      summaryZh:
        `筛选标签 ${id} 依据可替换的项目关联策略映射到机械骰面代码 ${mechanicalCode}。`,
      sourceRefs: [context.sourceRef(entry), tagPolicy],
      tags: ["custom-dice", "dice-face-tag"],
    }),
    source_id: id,
    sort: entry.row.Sort,
    name_text_hash: textHash(entry.row.TagName),
    mechanical_code: mechanicalCode,
    mapping_evidence_quality: "ProjectPolicy",
    mapping_replacement_condition:
      "Replace when a pinned released numeric-to-code relation is available.",
  };
});
outputs.set("dice-face-tags.json", ordered(tags, ["sort", "id"]));

const slotIds = new Set(slots.map(({ source_id: id }) => id));
const tagByCode = new Map(tags.map((tag) => [tag.mechanical_code, tag]));
const noTargetNoEffectIds = new Set(["2058", "2070", "2071"]);
const faceEntries = await context.table("RogueNousDiceSurface");
const faces = faceEntries.map((entry) => {
  const id = String(entry.row.SurfaceID);
  const name = localized(
    entry.row.SurfaceName,
    `Dice Face ${id}`,
    `骰面 ${id}`,
  );
  const description = localized(
    entry.row.SurfaceDesc,
    `Released effect template for Dice Face ${id}.`,
    `骰面 ${id} 的已发布效果模板。`,
  );
  const allowedSlotSourceIds = sourceIds(entry.row.SlotList);
  const mechanicalTagCodes = [...entry.row.TagList];
  for (const slotId of allowedSlotSourceIds)
    if (!slotIds.has(slotId))
      throw new Error(`face ${id} references unknown slot ${slotId}`);
  for (const code of mechanicalTagCodes)
    if (!tagByCode.has(code))
      throw new Error(`face ${id} references unmapped tag code ${code}`);
  const noTargetBehavior = noTargetNoEffectIds.has(id)
    ? "NoEffect"
    : "Unspecified";
  if (noTargetBehavior === "NoEffect"
    && !description.en.includes("will not take effect when no"))
    throw new Error(`face ${id} lost released no-target wording`);
  return {
    ...context.envelope({
      id: `gold-gears.dice-face.${id}`,
      kind: "DiceFace",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn: description.en,
      summaryZh: description.zh,
      sourceRefs: [context.sourceRef(entry), tagPolicy],
      tags: ["custom-dice", "dice-face"],
    }),
    source_id: id,
    sort: entry.row.Sort,
    item_id: String(entry.row.ItemID),
    rarity: entry.row.Rarity,
    activation_stage: entry.row.DiceActiveStage,
    parameters: parameters(entry.row.DescParam),
    name_text_hash: textHash(entry.row.SurfaceName),
    effect_text_hash: textHash(entry.row.SurfaceDesc),
    extra_description_ids: sourceIds(entry.row.ExtraDesc),
    allowed_slot_ids: allowedSlotSourceIds
      .map((slotId) => `gold-gears.dice-slot.${slotId}`),
    allowed_slot_source_ids: allowedSlotSourceIds,
    mechanical_tag_codes: mechanicalTagCodes,
    filter_tag_ids: mechanicalTagCodes
      .map((code) => tagByCode.get(code).id).sort(),
    tag_mapping_evidence_quality: "ProjectPolicy",
    unlock_display_id: String(entry.row.UnlockDisplayID),
    allowed_dice_ids: sourceIds(entry.row.BranchLimitaion)
      .map((diceId) => `gold-gears.custom-dice.${diceId}`),
    allowed_dice_source_ids: sourceIds(entry.row.BranchLimitaion),
    universal_dice_eligibility: entry.row.BranchLimitaion.length === 12,
    no_legal_target_behavior: noTargetBehavior,
    no_legal_target_evidence_quality:
      noTargetBehavior === "NoEffect" ? "ExactStructured" : "Unspecified",
    icon_path: entry.row.Icon,
  };
});
outputs.set("dice-faces.json", ordered(faces, ["rarity", "sort", "id"]));

await writeOrCheck(context, outputs, check);
console.log(
  `${check ? "Checked" : "Wrote"} 6 dice slots, 80 faces, and 10 filter tags.`,
);
