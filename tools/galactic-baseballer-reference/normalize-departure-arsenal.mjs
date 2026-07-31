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
const outputRoot = path.join(root, "content-reference", "galactic-baseballer-v1");
const profileId = "galactic-baseballer.departure.v2_2";
const rowRevision = "starclock.galactic-baseballer-row.v1";
const manifest = JSON.parse(await readFile(path.join(
  root,
  "content-manifests",
  "galactic-baseballer-v1",
  "content-manifest.json",
), "utf8"));

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

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) =>
      `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function digest(value) {
  return createHash("sha256").update(
    Buffer.isBuffer(value) ? value : canonical(value),
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

const readSource = async (relativePath) =>
  losslessJson(await readFile(path.join(sourceRoot, relativePath)));

function manifestRecord(category, id) {
  const record = manifest.categories[category].records.find(
    ({ id: recordId }) => recordId === id,
  );
  if (record === undefined) throw new Error(`missing manifest record: ${id}`);
  return record;
}

function structuredSource(record, mechanismQuality, note) {
  return {
    source_id: `source.goal16.${record.evidence_sha256.slice(0, 16)}`,
    repository_or_url: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision_or_access_date:
      "fd978d6ef09f941fba644c731ab54abd6f7c3568",
    game_version: "4.4",
    path_or_page: record.source_path,
    locator: record.row_locator,
    sha256: record.evidence_sha256,
    evidence_quality: "ExactStructured",
    mechanism_quality: mechanismQuality,
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
    ownership: "Departure",
    coverage_state: "Researched",
    evidence_quality: "ExactStructured",
    mechanism_quality: "ExactRelationship",
    manifest_record_ids: [...manifestIds].sort(),
    source_refs: sourceRefs,
    tags: [...tags].sort(),
  };
}

function collectStringFields(value, field, output = new Set()) {
  if (Array.isArray(value)) {
    for (const child of value) collectStringFields(child, field, output);
  } else if (value !== null && typeof value === "object") {
    if (typeof value[field] === "string") output.add(value[field]);
    for (const child of Object.values(value))
      collectStringFields(child, field, output);
  }
  return output;
}

function programSummary(program, bindingKey) {
  const abilities = program.AbilityList.filter(({ Name }) =>
    Name === bindingKey || Name.startsWith(`${bindingKey}_`));
  if (abilities.length === 0)
    throw new Error(`program binding missing: ${bindingKey}`);
  const modifierNames = new Set();
  for (const ability of abilities) {
    for (const name of Object.keys(ability.Modifiers ?? {}))
      modifierNames.add(name);
  }
  return {
    binding_key: bindingKey,
    ability_names: abilities.map(({ Name }) => Name).sort(),
    modifier_names: [...modifierNames].sort(),
    trigger_events: [...collectStringFields(abilities, "Event")].sort(),
    operation_types: [...collectStringFields(abilities, "$type")].sort(),
    program_fragment_sha256: digest(abilities),
  };
}

const collectionPath = "ExcelOutput/EvolveBuildGearCollection.json";
const gearConfigPath = "ExcelOutput/EvolveBuildGearConfig.json";
const mazeBuffPath = "ExcelOutput/EvolveBuildMazeBuff.json";
const forgePath = "ExcelOutput/EvolveBuildForgeMaterial.json";
const weaponProgramPath =
  "Config/ConfigAbility/BattleEvent/EvolveBuild_01_Weapon.json";
const accessoryProgramPath =
  "Config/ConfigAbility/BattleEvent/EvolveBuild_02_Accessory.json";
const collectionRows = await readSource(collectionPath);
const gearConfigRows = await readSource(gearConfigPath);
const mazeBuffRows = await readSource(mazeBuffPath);
const forgeRows = await readSource(forgePath);
const weaponProgram = await readSource(weaponProgramPath);
const accessoryProgram = await readSource(accessoryProgramPath);
const chs = await readSource("TextMap/TextMapCHS.json");
const en = await readSource("TextMap/TextMapEN.json");

const collectionIndex = new Map(
  collectionRows.map((row, index) => [String(row.ID), { row, index }]),
);
const mazeBuffIndex = new Map(
  mazeBuffRows.map((row, index) => [
    `${String(row.ID)}:${row.Lv}`,
    { row, index },
  ]),
);
const gearRowsById = new Map();
for (const [index, row] of gearConfigRows.entries()) {
  const id = String(row.GearID);
  const existing = gearRowsById.get(id) ?? [];
  existing.push({ row, index });
  gearRowsById.set(id, existing);
}
const weaponCollections = collectionRows
  .map((row, index) => ({ row, index }))
  .filter(({ row }) => row.DamageCustomName !== "");
const accessoryCollections = collectionRows
  .map((row, index) => ({ row, index }))
  .filter(({ row }) => row.DamageCustomName === "");

function namesForCollection(row) {
  const hash = String(row.Name.Hash);
  const nameEn = en[hash];
  const nameZh = chs[hash];
  if (typeof nameEn !== "string" || typeof nameZh !== "string")
    throw new Error(`collection localization missing: ${row.ID}/${hash}`);
  return { hash, nameEn, nameZh };
}

function collectionManifest(index) {
  const id =
    `${profileId}:EvolveBuildGearCollection:${String(index).padStart(4, "0")}`;
  return {
    id,
    record: manifestRecord("weapon_collections", id),
  };
}

const weapons = weaponCollections.map(({ row, index }) => {
  const { hash, nameEn, nameZh } = namesForCollection(row);
  const { id: manifestId, record } = collectionManifest(index);
  const tier = row.DamageCustomName.endsWith("_Max")
    ? "Legendary"
    : "Standard";
  return {
    ...envelope({
      id: `galactic-baseballer.departure.weapon.${row.ID}`,
      kind: "Weapon",
      nameEn,
      nameZh,
      summaryEn:
        `${tier} Departure weapon with ${row.LvMax} authored level(s) and exact trigger-program binding.`,
      summaryZh:
        `启程篇${tier === "Legendary" ? "传说" : "普通"}武器，含 ${row.LvMax} 个作者等级与精确触发程序绑定。`,
      manifestIds: [manifestId],
      sourceRefs: [structuredSource(
        record,
        "ExactRelationship",
        "exact released weapon identity and collection metadata",
      )],
      tags: ["departure", tier.toLowerCase(), "weapon"],
    }),
    source_numeric_id: String(row.ID),
    source_name_hash: hash,
    tier,
    maximum_level: row.LvMax,
    damage_custom_name: row.DamageCustomName,
    element_ids: row.ElementList.map(String),
    tag_ids: row.TagList.map(String),
    season: row.Season,
  };
});

const accessories = accessoryCollections.map(({ row, index }) => {
  const { hash, nameEn, nameZh } = namesForCollection(row);
  const { id: manifestId, record } = collectionManifest(index);
  return {
    ...envelope({
      id: `galactic-baseballer.departure.accessory.${row.ID}`,
      kind: "Accessory",
      nameEn,
      nameZh,
      summaryEn:
        `Departure accessory with ${row.LvMax} authored levels and an exact battle binding.`,
      summaryZh:
        `启程篇配饰，含 ${row.LvMax} 个作者等级与精确战斗绑定。`,
      manifestIds: [manifestId],
      sourceRefs: [structuredSource(
        record,
        "ExactRelationship",
        "exact released accessory identity and collection metadata",
      )],
      tags: ["accessory", "departure"],
    }),
    source_numeric_id: String(row.ID),
    source_name_hash: hash,
    maximum_level: row.LvMax,
    element_ids: row.ElementList.map(String),
    tag_ids: row.TagList.map(String),
    season: row.Season,
  };
});

function levelRows(collection, kind) {
  const output = [];
  for (const { row: definition } of collection) {
    const gearRows = gearRowsById.get(String(definition.ID));
    if (gearRows === undefined)
      throw new Error(`gear levels missing: ${definition.ID}`);
    for (const { row, index } of gearRows) {
      const mazeMatch = mazeBuffIndex.get(
        `${String(row.MazeBuffID)}:${row.Level}`,
      );
      if (mazeMatch === undefined)
        throw new Error(`MazeBuff missing: ${row.MazeBuffID}`);
      const { row: mazeBuff, index: mazeIndex } = mazeMatch;
      const gearManifestId =
        `${profileId}:EvolveBuildGearConfig:${String(index).padStart(4, "0")}`;
      const mazeManifestId =
        `${profileId}:EvolveBuildMazeBuff:${String(mazeIndex).padStart(4, "0")}`;
      const gearManifest = manifestRecord("weapon_levels", gearManifestId);
      const mazeManifest = manifestRecord("accessory_levels", mazeManifestId);
      const definitionId = kind === "Weapon"
        ? `galactic-baseballer.departure.weapon.${definition.ID}`
        : `galactic-baseballer.departure.accessory.${definition.ID}`;
      output.push({
        ...envelope({
          id: `${definitionId}.level.${row.Level}`,
          kind: `${kind}Level`,
          nameEn: `${kind} ${definition.ID} level ${row.Level}`,
          nameZh: `${kind === "Weapon" ? "武器" : "配饰"} ${definition.ID} 等级 ${row.Level}`,
          summaryEn:
            `Exact ${kind.toLowerCase()} level parameters and battle-binding record.`,
          summaryZh: `精确${kind === "Weapon" ? "武器" : "配饰"}等级参数与战斗绑定记录。`,
          manifestIds: [gearManifestId, mazeManifestId],
          sourceRefs: [
            structuredSource(
              gearManifest,
              "ExactRelationship",
              "exact GearConfig level-to-MazeBuff binding",
            ),
            structuredSource(
              mazeManifest,
              "ExactProgram",
              "exact MazeBuff parameter vector and program binding",
            ),
          ],
          tags: [kind.toLowerCase(), "departure", "level"],
        }),
        parent_id: definitionId,
        level: row.Level,
        maze_buff_id: String(row.MazeBuffID),
        index_list: row.IndexList,
        simple_index_list: row.SimpIndexList,
        dynamic_index_list: row.DynamicIndexList,
        modifier_name: mazeBuff.ModifierName,
        binding_key: mazeBuff.InBattleBindingKey,
        parameter_values: mazeBuff.ParamList.map(({ Value }) =>
          String(Value)),
        buff_name_hash: String(mazeBuff.BuffName.Hash),
        buff_description_hash: String(mazeBuff.BuffDesc.Hash),
        buff_battle_description_hash: String(mazeBuff.BuffDescBattle.Hash),
        buff_simple_description_hash: String(mazeBuff.BuffSimpleDesc.Hash),
        buff_rarity: mazeBuff.BuffRarity,
        buff_series: mazeBuff.BuffSeries,
        maze_buff_type: mazeBuff.MazeBuffType,
      });
    }
  }
  return output;
}

const weaponLevels = levelRows(weaponCollections, "Weapon");
const accessoryLevels = levelRows(accessoryCollections, "Accessory");
const weaponProgramManifest = manifestRecord(
  "config_programs",
  weaponProgramPath,
);
const accessoryProgramManifest = manifestRecord(
  "config_programs",
  accessoryProgramPath,
);

function bindingRows(definitions, levels, kind, program, programManifest) {
  return definitions.map((definition) => {
    const level = levels.find(({ parent_id: id }) => id === definition.id);
    if (level === undefined)
      throw new Error(`binding level missing: ${definition.id}`);
    const summary = programSummary(program, level.binding_key);
    return {
      ...envelope({
        id: `${definition.id}.binding`,
        kind: `${kind}Binding`,
        nameEn: `${definition.name_en} battle binding`,
        nameZh: `${definition.name_zh_cn}战斗绑定`,
        summaryEn:
          `Structural summary of the released ${kind.toLowerCase()} program without copying the source program.`,
        summaryZh:
          `已发布${kind === "Weapon" ? "武器" : "配饰"}程序的结构摘要，不复制源程序。`,
        manifestIds: [
          ...definition.manifest_record_ids,
          programManifest.id,
        ],
        sourceRefs: [
          ...definition.source_refs,
          structuredSource(
            programManifest,
            "ExactProgram",
            "whole-file program digest; normalized row retains only mechanical identifiers and structural sets",
          ),
        ],
        tags: ["binding", "departure", kind.toLowerCase()],
      }),
      parent_id: definition.id,
      ...summary,
      runtime_executable: false,
    };
  });
}

const weaponTriggers = bindingRows(
  weapons,
  weaponLevels,
  "Weapon",
  weaponProgram,
  weaponProgramManifest,
);
const accessoryBindings = bindingRows(
  accessories,
  accessoryLevels,
  "Accessory",
  accessoryProgram,
  accessoryProgramManifest,
);

const synthesisRecipes = [];
const synthesisInputs = [];
for (const [index, row] of forgeRows.entries()) {
  const recipeManifestId =
    `${profileId}:EvolveBuildForgeMaterial:${String(index).padStart(4, "0")}`;
  const recipeManifest = manifestRecord("synthesis_materials", recipeManifestId);
  const outputId = String(row.ForgeGearID);
  const outputDefinition = weapons.find(({ source_numeric_id: id }) =>
    id === outputId);
  if (outputDefinition === undefined || outputDefinition.tier !== "Legendary")
    throw new Error(`legendary output missing: ${outputId}`);
  const materialEntries = Object.entries(row.MaterialGearList)
    .map(([gearId, requiredLevel]) => {
      const collection = collectionIndex.get(gearId);
      if (collection === undefined)
        throw new Error(`recipe material missing: ${gearId}`);
      return {
        gearId,
        requiredLevel,
        collection: collection.row,
        consumed: row.CostGearList.map(String).includes(gearId),
      };
    })
    .sort((left, right) => {
      const leftWeapon = left.collection.DamageCustomName !== "";
      const rightWeapon = right.collection.DamageCustomName !== "";
      if (leftWeapon !== rightWeapon) return leftWeapon ? -1 : 1;
      return left.gearId.localeCompare(right.gearId, "en");
    });
  const recipeId = `galactic-baseballer.departure.recipe.${outputId}`;
  synthesisRecipes.push({
    ...envelope({
      id: recipeId,
      kind: "SynthesisRecipe",
      nameEn: `Synthesize ${outputDefinition.name_en}`,
      nameZh: `合成${outputDefinition.name_zh_cn}`,
      summaryEn:
        "Exact acyclic Standard-weapon plus accessory recipe for one Legendary weapon.",
      summaryZh: "普通武器加指定配饰合成一件传说武器的精确无环配方。",
      manifestIds: [recipeManifestId],
      sourceRefs: [structuredSource(
        recipeManifest,
        "ExactRelationship",
        "exact ForgeMaterial output, prerequisites and consumption list",
      )],
      tags: ["departure", "legendary", "synthesis"],
    }),
    tier: "Legendary",
    output_weapon_id: outputDefinition.id,
    input_count: materialEntries.length,
    validation_order: "all-prerequisites-before-consumption",
    failure_behavior: "ProjectPolicy: no mutation; covered by the approximation register",
  });
  for (const [ordinal, material] of materialEntries.entries()) {
    const weapon = material.collection.DamageCustomName !== "";
    synthesisInputs.push({
      ...envelope({
        id: `${recipeId}.input.${ordinal}`,
        kind: "SynthesisInput",
        nameEn: `Recipe input ${ordinal} for ${outputDefinition.name_en}`,
        nameZh: `${outputDefinition.name_zh_cn}配方材料 ${ordinal}`,
        summaryEn:
          `Ordered exact ${weapon ? "weapon" : "accessory"} prerequisite and consumption flag.`,
        summaryZh:
          `有序的精确${weapon ? "武器" : "配饰"}前置等级与消耗标记。`,
        manifestIds: [recipeManifestId],
        sourceRefs: [structuredSource(
          recipeManifest,
          "ExactRelationship",
          "exact ForgeMaterial input level and CostGearList membership",
        )],
        tags: ["departure", "recipe-input", weapon ? "weapon" : "accessory"],
      }),
      recipe_id: recipeId,
      input_order: ordinal,
      input_kind: weapon ? "Weapon" : "Accessory",
      input_id: weapon
        ? `galactic-baseballer.departure.weapon.${material.gearId}`
        : `galactic-baseballer.departure.accessory.${material.gearId}`,
      required_level: material.requiredLevel,
      consumed: material.consumed,
    });
  }
}

for (const rows of [
  weapons,
  weaponLevels,
  weaponTriggers,
  accessories,
  accessoryLevels,
  accessoryBindings,
  synthesisRecipes,
  synthesisInputs,
]) rows.sort((left, right) => left.id.localeCompare(right.id, "en"));

const outputs = new Map([
  ["weapons.json", weapons],
  ["weapon-levels.json", weaponLevels],
  ["weapon-triggers.json", weaponTriggers],
  ["accessories.json", accessories],
  ["accessory-levels.json", accessoryLevels],
  ["accessory-bindings.json", accessoryBindings],
  ["synthesis-recipes.json", synthesisRecipes],
  ["synthesis-inputs.json", synthesisInputs],
]);
await mkdir(outputRoot, { recursive: true });
for (const [file, value] of outputs) {
  const target = path.join(outputRoot, file);
  const encoded = `${JSON.stringify(canonicalValue(value), null, 2)}\n`;
  if (check) {
    const existing = await readFile(target, "utf8");
    if (existing !== encoded)
      throw new Error(`Departure arsenal drift: ${target}`);
  } else {
    await writeFile(target, encoded);
  }
}

console.log(
  `Departure arsenal ${check ? "verified" : "wrote"}: `
  + `${weapons.length} weapons/${weaponLevels.length} levels, `
  + `${accessories.length} accessories/${accessoryLevels.length} levels, `
  + `${synthesisRecipes.length} recipes/${synthesisInputs.length} inputs`,
);
