#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  ACCESS_DATE,
  GAME_VERSION,
  createContext,
  decimal,
  sha256,
  writeOrCheck,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(valueAfter("--root")
  ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."));
const context = await createContext(root, valueAfter("--source-cache"));
const outputs = new Map([
  ["blessing-paths.json", []],
  ["blessings.json", []],
  ["blessing-levels.json", []],
  ["blessing-groups.json", []],
  ["formulas.json", []],
  ["formula-displays.json", []],
  ["formula-randomizers.json", []],
  ["formula-recipes.json", []],
  ["formula-contributions.json", []],
]);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
function normalize(value) {
  if (value === undefined || value === null) return "";
  if (Array.isArray(value)) return value.map(normalize);
  if (typeof value === "number") return decimal(value);
  if (typeof value !== "object") return value;
  if (Object.keys(value).length === 1 && Object.hasOwn(value, "Value"))
    return decimal(value);
  return Object.fromEntries(Object.entries(value)
    .map(([key, entry]) => [key, normalize(entry)]));
}

const manifestRelative =
  "content-manifests/currency-wars-v1/content-manifest.json";
const manifestBytes = await fs.readFile(path.join(root, manifestRelative));
const manifest = JSON.parse(manifestBytes);
const category = manifest.categories.blessings_levels_formulas;
if (category.count !== 125)
  throw new Error("Blessing/formula manifest denominator drift");
const manifestRef = {
  source_id: "source.goal12.manifest.blessing-formula-closure",
  repository: "starclock",
  revision: manifest.schema_revision,
  path: manifestRelative,
  locator: "categories.blessings_levels_formulas",
  sha256: sha256(manifestBytes),
  access_date: ACCESS_DATE,
  game_version: GAME_VERSION,
  evidence_quality: "ExactStructured",
  mechanism_quality: "GeneratedClosedManifest",
  note:
    "The complete direct GridFight closure contains Affix rows and seven MazeBuff enhancements, but no Blessing, Blessing path, formula, recipe, group or randomizer identity.",
};

outputs.get("blessing-paths.json").push({
  ...context.envelope({
    id: "currency-wars.blessing-closure.no-reachable-blessing-path",
    kind: "CurrencyWarsBlessingClosure",
    nameEn: "No reachable Blessing path",
    nameZh: "无可达祝福命途",
    summaryEn:
      "The frozen GridFight closure contains no Blessing path or Blessing identity; enemy Affixes remain a separate system.",
    summaryZh:
      "冻结的 GridFight 闭包不含祝福命途或祝福身份；敌人词缀保持为独立系统。",
    sourceRefs: [manifestRef],
    tags: ["blessing", "generated-closure", "proven-empty"],
  }),
  path_id: "none",
  offer_roles: [],
  formula_roles: [],
  closure: {
    direct_blessing_identity_count: "0",
    reachable_shared_blessing_identity_count: "0",
    excluded_affix_rows: "118",
    retained_maze_buff_enhancement_rows: "7",
  },
});

const enhancements = await context.table("GridFightMazeBuffEnhance");
for (const entry of enhancements) {
  const row = entry.row;
  const id = String(row.ID);
  const nameEn =
    context.text(row.EnhanceName, "en") || `MazeBuff enhancement ${id}`;
  const nameZh =
    context.text(row.EnhanceName, "zh_cn") || `MazeBuff 强化 ${id}`;
  const refs = [
    context.sourceRef(entry),
    ...context.bilingualTextRefs(String(row.EnhanceName.Hash)),
    ...context.bilingualTextRefs(String(row.EnhanceDesc.Hash)),
  ];
  outputs.get("blessing-levels.json").push({
    ...context.envelope({
      id: `currency-wars.maze-buff-enhancement.${id}`,
      kind: "CurrencyWarsMazeBuffEnhancement",
      nameEn,
      nameZh,
      summaryEn:
        `MazeBuff enhancement ${id} binds ability ${row.AbilityName} with ${row.ParamList.length} canonical parameter(s); it is not promoted to a Blessing identity.`,
      summaryZh:
        `MazeBuff 强化 ${id} 绑定能力 ${row.AbilityName} 与 ${row.ParamList.length} 个规范参数；它不会被提升为祝福身份。`,
      sourceRefs: refs,
      tags: ["maze-buff-enhancement", "not-a-blessing"],
    }),
    blessing_id: "none:maze-buff-enhancement",
    level: id,
    parameters: normalize(row.ParamList),
    effect_ids: [`ability:${row.AbilityName}`],
  });
}
outputs.get("blessing-levels.json")
  .sort((left, right) => compare(left.id, right.id));

outputs.get("formulas.json").push({
  ...context.envelope({
    id: "currency-wars.formula-closure.no-reachable-formula",
    kind: "CurrencyWarsFormulaClosure",
    nameEn: "No reachable Blessing formula",
    nameZh: "无可达祝福公式",
    summaryEn:
      "The frozen GridFight closure contains no formula, Equation-like recipe, completion state or formula randomizer.",
    summaryZh:
      "冻结的 GridFight 闭包不含公式、方程式类配方、完成状态或公式随机器。",
    sourceRefs: [manifestRef],
    tags: ["formula", "generated-closure", "proven-empty"],
  }),
  formula_kind: "ProvenEmptyDirectAndSharedClosure",
  recipe_id: "none",
  progress_states: [],
  effect_ids: [],
});

await writeOrCheck(context, outputs, check);
console.log(
  `Currency Wars Blessing/formula closure ${check ? "verified" : "generated"}: ` +
  `${enhancements.length} MazeBuff enhancements, zero reachable Blessings ` +
  "and zero reachable formulas.",
);
