#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  ACCESS_DATE,
  canonical,
  createContext,
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

const standardBlessings = await localRows(
  "content-reference/standard-universe-v1/blessings.json",
);
const standardLevels = await localRows(
  "content-reference/standard-universe-v1/blessing-levels.json",
);
const goldPaths = await localRows(
  "content-reference/gold-and-gears-v1/paths.json",
);
const manifest = await localRows(
  "content-manifests/gold-and-gears-v1/content-manifest.json",
);
const buffEntries = await context.table("RogueBuff");
const mazeEntries = await context.table("RogueMazeBuff");

const pathById = new Map(goldPaths.map((row, index) => [
  row.id,
  { row, index },
]));
const blessingById = new Map(standardBlessings.map((row) => [row.id, row]));
const buffByMazeId = new Map();
for (const entry of buffEntries) {
  const key = String(entry.row.MazeBuffID);
  if (!buffByMazeId.has(key)) buffByMazeId.set(key, entry);
}
const mazeByIdAndLevel = new Map(mazeEntries.map((entry) => [
  `${entry.row.ID}:${entry.row.Lv}`,
  entry,
]));

const blessingManifestIds = new Set(
  manifest.categories.blessings.records.map(({ id }) => id),
);
const levelManifestIds = new Set(
  manifest.categories.blessing_levels.records.map(({ id }) => id),
);

const blessings = standardBlessings
  .filter(({ id }) => blessingManifestIds.has(id))
  .map((standardRow, index) => {
    const sourceId = String(standardRow.source_ids[0]);
    const buffEntry = buffByMazeId.get(sourceId);
    const path = pathById.get(standardRow.path_id);
    if (!buffEntry || !path)
      throw new Error(`incomplete shared Blessing ${standardRow.id}`);
    return {
      ...context.envelope({
        id: standardRow.id,
        kind: "Blessing",
        nameEn: standardRow.name_en,
        nameZh: standardRow.name_zh_cn,
        summaryEn:
          `Gold and Gears inherits this released ${standardRow.rarity}-star ` +
          `${path.row.name_en} Blessing and both authored levels unchanged.`,
        summaryZh:
          `黄金与机械原样继承该已发布的${standardRow.rarity}星` +
          `${path.row.name_zh_cn}祝福及两个配置等级。`,
        ownership: "Shared",
        sourceRefs: [
          localRef(
            "content-reference/standard-universe-v1/blessings.json",
            standardRow,
            index,
          ),
          context.sourceRef(buffEntry),
          localRef(
            "content-reference/gold-and-gears-v1/paths.json",
            path.row,
            path.index,
          ),
        ],
        tags: ["blessing", `rarity-${standardRow.rarity}`, "shared"],
      }),
      source_id: sourceId,
      source_mode_owner: standardRow.mode_owner,
      pool_membership: "InheritedSharedPathPool",
      reachability_path_id: standardRow.path_id,
      path_id: standardRow.path_id,
      rarity: standardRow.rarity,
      level_ids: standardRow.level_ids,
      prerequisite_ids: standardRow.prerequisite_ids,
      pool_tags: standardRow.pool_tags,
      extra_effect_source_ids: standardRow.extra_effect_source_ids,
      inherited_rule_ids: standardRow.rule_ids,
      mechanic_tags: standardRow.mechanic_tags,
      source_description_sha256_en:
        standardRow.source_description_sha256_en,
      source_description_sha256_zh_cn:
        standardRow.source_description_sha256_zh_cn,
    };
  }).sort((left, right) =>
    left.path_id.localeCompare(right.path_id)
    || left.rarity - right.rarity
    || left.id.localeCompare(right.id));

const levels = standardLevels
  .filter(({ id }) => levelManifestIds.has(id))
  .map((standardRow, index) => {
    const blessing = blessingById.get(standardRow.blessing_id);
    const path = pathById.get(blessing?.path_id);
    const sourceId = String(standardRow.source_ids[0]);
    const mazeEntry = mazeByIdAndLevel.get(
      `${sourceId}:${standardRow.level}`,
    );
    if (!blessing || !path || !mazeEntry)
      throw new Error(`incomplete shared Blessing level ${standardRow.id}`);
    return {
      ...context.envelope({
        id: standardRow.id,
        kind: "BlessingLevel",
        nameEn: standardRow.name_en,
        nameZh: standardRow.name_zh_cn,
        summaryEn:
          `Gold and Gears inherits authored level ${standardRow.level} ` +
          `parameters and binding for ${blessing.name_en}.`,
        summaryZh:
          `黄金与机械继承${blessing.name_zh_cn}已配置的等级` +
          `${standardRow.level}参数与绑定。`,
        ownership: "Shared",
        sourceRefs: [
          localRef(
            "content-reference/standard-universe-v1/blessing-levels.json",
            standardRow,
            index,
          ),
          context.sourceRef(mazeEntry),
          localRef(
            "content-reference/gold-and-gears-v1/paths.json",
            path.row,
            path.index,
          ),
        ],
        tags: ["blessing-level", `level-${standardRow.level}`, "shared"],
      }),
      source_id: sourceId,
      source_mode_owner: standardRow.mode_owner,
      pool_membership: "InheritedSharedPathPool",
      reachability_path_id: blessing.path_id,
      blessing_id: standardRow.blessing_id,
      level: standardRow.level,
      parameter_values: standardRow.parameter_values,
      inherited_rule_ids: standardRow.rule_ids,
      source_modifier_name: standardRow.source_modifier_name,
      source_binding_type: standardRow.source_binding_type,
      source_binding_key: standardRow.source_binding_key,
      source_maze_buff_type: standardRow.source_maze_buff_type,
      source_description_sha256_en:
        standardRow.source_description_sha256_en,
      source_description_sha256_zh_cn:
        standardRow.source_description_sha256_zh_cn,
    };
  }).sort((left, right) =>
    left.blessing_id.localeCompare(right.blessing_id)
    || left.level - right.level);

await writeOrCheck(context, new Map([
  ["blessings.json", blessings],
  ["blessing-levels.json", levels],
]), check);
console.log(
  `${check ? "Checked" : "Wrote"} ${blessings.length} shared Blessings and ` +
  `${levels.length} authored levels.`,
);
