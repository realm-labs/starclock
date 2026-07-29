#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  createContext,
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

const styleEntries = await context.table("RogueMagicStyleTypeSelect");
const displayEntries = await context.table("RogueMagicMiscDisplay");
const scepterEntries = await context.table("RogueMagicScepter");
const areaEntries = await context.table("RogueMagicArea");
const displayById = new Map(displayEntries.map((entry) => [
  entry.row.DisplayID,
  entry,
]));
const fallback = {
  Break: { en: "Break", zh: "击破" },
  Dot: { en: "Damage over Time", zh: "持续伤害" },
  Follow: { en: "Follow-up Attack", zh: "追加攻击" },
  Ultimate: { en: "Ultimate", zh: "终结技" },
};
const alignments = styleEntries.map((entry) => {
  const style = entry.row.EnumType;
  const display = displayById.get(entry.row.DisplayID);
  if (!display) throw new Error(`missing Alignment display ${entry.row.DisplayID}`);
  const names = fallback[style];
  const displayEn = context.text(display.row.DisplayContent, "en");
  const displayZh = context.text(display.row.DisplayContent, "zh_cn");
  const scepterIds = [...new Set(scepterEntries
    .filter(({ row }) => row.StyleType === style)
    .map(({ row }) => `unknowable-domain.scepter.${row.ScepterID}`))].sort();
  const areaIds = areaEntries
    .filter(({ row }) => row.DefaultStyle === style)
    .map(({ row }) => `unknowable-domain.area.${row.AreaID}`)
    .sort();
  return {
    ...context.envelope({
      id: `unknowable-domain.alignment.${slug(style)}`,
      kind: "ExtrapolationAlignment",
      nameEn: names.en,
      nameZh: names.zh,
      summaryEn:
        `${names.en} is a released Extrapolation Alignment with ${scepterIds.length} directly style-matched Scepter candidates and ${areaIds.length} default-area binding(s).`,
      summaryZh:
        `${names.zh}是已发布的推演倾向，拥有 ${scepterIds.length} 个直接按风格匹配的权杖候选与 ${areaIds.length} 个默认区域绑定。`,
      sourceRefs: [
        context.sourceRef(entry),
        context.sourceRef(display),
      ],
      tags: ["alignment", slug(style)],
    }),
    source_id: style,
    display_id: String(entry.row.DisplayID),
    display_text_en: displayEn,
    display_text_zh_cn: displayZh,
    unlock_id: entry.row.UnlockID === undefined ? "" : String(entry.row.UnlockID),
    eligibility: entry.row.UnlockID === undefined
      ? "AvailableByDefault"
      : "RequiresUnlock",
    selection_cardinality: "Unspecified",
    default_area_ids: areaIds,
    scepter_candidate_ids: scepterIds,
    component_candidate_ids: [],
    component_pool_resolution: "Unspecified",
    pool_ids: [`unknowable-domain.pool.scepters.${slug(style)}`],
    rule_contribution_ids: [],
    contribution_resolution: "DeferredToScepterAndComponentRules",
  };
}).sort((left, right) => left.id.localeCompare(right.id));

await writeOrCheck(
  context,
  new Map([["alignments.json", alignments]]),
  check,
);
console.log(
  `Unknowable Domain Alignments ${check ? "verified" : "generated"}: ` +
  `${alignments.length} Alignments and ` +
  `${alignments.reduce((sum, row) => sum + row.scepter_candidate_ids.length, 0)} ` +
  "style-matched Scepter bindings.",
);
