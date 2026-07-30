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
const outputRoot = path.join(
  root,
  "content-reference",
  "galactic-baseballer-v1",
  "fragments",
);
const profileId = "galactic-baseballer.demon-king.v3_3";
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
  mechanismQuality = "ExactRelationship",
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
    ownership: "DemonKing",
    coverage_state: "Researched",
    evidence_quality: sourceRefs.some(({ evidence_quality: quality }) =>
      quality === "ProjectPolicy") ? "ProjectPolicy" : "ExactStructured",
    mechanism_quality: mechanismQuality,
    manifest_record_ids: [...new Set(manifestIds)].sort(),
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

function collectMatchingStrings(value, pattern, output = new Set()) {
  if (typeof value === "string") {
    if (pattern.test(value)) output.add(value);
  } else if (Array.isArray(value)) {
    for (const child of value) collectMatchingStrings(child, pattern, output);
  } else if (value !== null && typeof value === "object") {
    for (const child of Object.values(value))
      collectMatchingStrings(child, pattern, output);
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

const collectionPath = "ExcelOutput/EvoBdSCGearCollection.json";
const gearConfigPath = "ExcelOutput/EvoBdSCGearConfig.json";
const mazeBuffPath = "ExcelOutput/EvoBdSCMazeBuff.json";
const forgePath = "ExcelOutput/EvoBdSCForgeMaterial.json";
const typePath = "ExcelOutput/EvoBdSCGearTypeConfig.json";
const weaponProgramPath =
  "Config/ConfigAbility/BattleEvent/EvolveBuild_01_Weapon_S2.json";
const accessoryProgramPath =
  "Config/ConfigAbility/BattleEvent/EvolveBuild_02_Accessory_SC.json";
const actorProgramPaths = [
  "Config/ConfigCharacter/BattleEvent/EvolveBuild_S2_Claymore_00_Config.json",
  "Config/ConfigCharacter/BattleEvent/EvolveBuild_S2_Shooter_Config.json",
  "Config/ConfigCharacter/BattleEvent/EvolveBuild_S2_Shooter_Partner01_Config.json",
];
const collectionRows = await readSource(collectionPath);
const gearConfigRows = await readSource(gearConfigPath);
const mazeBuffRows = await readSource(mazeBuffPath);
const forgeRows = await readSource(forgePath);
const typeRows = await readSource(typePath);
const weaponProgram = await readSource(weaponProgramPath);
const accessoryProgram = await readSource(accessoryProgramPath);
const actorPrograms = await Promise.all(actorProgramPaths.map(async (file) => ({
  file,
  program: await readSource(file),
  manifest: manifestRecord("config_programs", file),
})));
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
const typeIndex = new Map(typeRows.map((row, index) => [
  row.ID ?? "Base",
  {
    row,
    index,
    manifestId:
      `${profileId}:EvoBdSCGearTypeConfig:${String(index).padStart(4, "0")}`,
  },
]));

function sourceType(row) {
  return row.Type ?? (row.DamageCustomName === "" ? "Plugin" : "Base");
}

function tierFor(row) {
  const type = sourceType(row);
  const tiers = {
    Base: "Standard",
    Forge: "Legendary",
    DuelForge: "Twin",
    UltraForge: "Supreme",
  };
  const tier = tiers[type];
  if (tier === undefined) throw new Error(`not a weapon type: ${type}`);
  return tier;
}

function typeManifest(row) {
  const type = sourceType(row);
  const found = typeIndex.get(type);
  if (found === undefined) throw new Error(`gear type missing: ${type}`);
  return {
    ...found,
    manifest: manifestRecord("weapon_types", found.manifestId),
  };
}

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
    `${profileId}:EvoBdSCGearCollection:${String(index).padStart(4, "0")}`;
  return {
    id,
    record: manifestRecord("weapon_collections", id),
  };
}

const weaponCollections = collectionRows
  .map((row, index) => ({ row, index }))
  .filter(({ row }) => row.DamageCustomName !== "");
const accessoryCollections = collectionRows
  .map((row, index) => ({ row, index }))
  .filter(({ row }) => row.DamageCustomName === "");

const weapons = weaponCollections.map(({ row, index }) => {
  const { hash, nameEn, nameZh } = namesForCollection(row);
  const { id: manifestId, record } = collectionManifest(index);
  const type = typeManifest(row);
  const tier = tierFor(row);
  return {
    ...envelope({
      id: `galactic-baseballer.demon-king.weapon.${row.ID}`,
      kind: "Weapon",
      nameEn,
      nameZh,
      summaryEn:
        `${tier} Demon King weapon with ${row.LvMax} authored level(s) and exact trigger-program binding.`,
      summaryZh:
        `魔王篇${tier === "Standard" ? "普通" : tier === "Legendary"
          ? "传说" : tier === "Twin" ? "双重" : "至尊"}武器，含 ${row.LvMax} 个作者等级与精确触发程序绑定。`,
      manifestIds: [manifestId, type.manifestId],
      sourceRefs: [
        structuredSource(
          record,
          "ExactRelationship",
          "exact released weapon identity and collection metadata",
        ),
        structuredSource(
          type.manifest,
          "ExactRelationship",
          "exact gear-type definition used without name or ID inference",
        ),
      ],
      tags: ["demon-king", tier.toLowerCase(), "weapon"],
    }),
    source_numeric_id: String(row.ID),
    source_name_hash: hash,
    source_type: sourceType(row),
    tier,
    maximum_level: row.LvMax,
    unlock_quest_id: row.UnlockQuest === undefined
      ? undefined
      : String(row.UnlockQuest),
    damage_custom_name: row.DamageCustomName,
    element_ids: row.ElementList.map(String),
    tag_ids: row.TagList.map(String),
    season: row.Season,
  };
});

const accessories = accessoryCollections.map(({ row, index }) => {
  const { hash, nameEn, nameZh } = namesForCollection(row);
  const { id: manifestId, record } = collectionManifest(index);
  const type = typeManifest(row);
  return {
    ...envelope({
      id: `galactic-baseballer.demon-king.accessory.${row.ID}`,
      kind: "Accessory",
      nameEn,
      nameZh,
      summaryEn:
        `Demon King accessory with ${row.LvMax} authored levels and an exact battle binding.`,
      summaryZh:
        `魔王篇配饰，含 ${row.LvMax} 个作者等级与精确战斗绑定。`,
      manifestIds: [manifestId, type.manifestId],
      sourceRefs: [
        structuredSource(
          record,
          "ExactRelationship",
          "exact released accessory identity and collection metadata",
        ),
        structuredSource(
          type.manifest,
          "ExactRelationship",
          "exact Plugin gear-type definition",
        ),
      ],
      tags: ["accessory", "demon-king"],
    }),
    source_numeric_id: String(row.ID),
    source_name_hash: hash,
    source_type: sourceType(row),
    maximum_level: row.LvMax,
    unlock_quest_id: row.UnlockQuest === undefined
      ? undefined
      : String(row.UnlockQuest),
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
        throw new Error(`MazeBuff missing: ${row.MazeBuffID}/${row.Level}`);
      const { row: mazeBuff, index: mazeIndex } = mazeMatch;
      const gearManifestId =
        `${profileId}:EvoBdSCGearConfig:${String(index).padStart(4, "0")}`;
      const mazeManifestId =
        `${profileId}:EvoBdSCMazeBuff:${String(mazeIndex).padStart(4, "0")}`;
      const gearManifest = manifestRecord("weapon_levels", gearManifestId);
      const mazeManifest = manifestRecord("accessory_levels", mazeManifestId);
      const definitionId = kind === "Weapon"
        ? `galactic-baseballer.demon-king.weapon.${definition.ID}`
        : `galactic-baseballer.demon-king.accessory.${definition.ID}`;
      const correctionIds = String(definition.ID) === "3113002"
        && (row.Level === 7 || row.Level === 8)
        ? ["galactic-baseballer.correction.v3_4.ruinbot-level-7-8"]
        : [];
      output.push({
        ...envelope({
          id: `${definitionId}.level.${row.Level}`,
          kind: `${kind}Level`,
          nameEn: `${kind} ${definition.ID} level ${row.Level}`,
          nameZh: `${kind === "Weapon" ? "武器" : "配饰"} ${definition.ID} 等级 ${row.Level}`,
          summaryEn:
            `Exact post-correction ${kind.toLowerCase()} level parameters and battle binding.`,
          summaryZh:
            `精确的修正后${kind === "Weapon" ? "武器" : "配饰"}等级参数与战斗绑定。`,
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
              "exact Version 4.4 MazeBuff parameter vector and program binding",
            ),
          ],
          tags: [kind.toLowerCase(), "demon-king", "level"],
        }),
        parent_id: definitionId,
        level: row.Level,
        maze_buff_id: String(row.MazeBuffID),
        index_list: row.IndexList,
        simple_index_list: row.SimpIndexList,
        dynamic_index_list: row.DynamicIndexList,
        modifier_name: mazeBuff.ModifierName,
        binding_key: mazeBuff.InBattleBindingKey,
        parameter_values: mazeBuff.ParamList.map(({ Value }) => String(Value)),
        buff_name_hash: String(mazeBuff.BuffName.Hash),
        buff_description_hash: String(mazeBuff.BuffDesc.Hash),
        buff_battle_description_hash: String(mazeBuff.BuffDescBattle.Hash),
        buff_simple_description_hash: String(mazeBuff.BuffSimpleDesc.Hash),
        buff_rarity: mazeBuff.BuffRarity,
        buff_series: mazeBuff.BuffSeries,
        maze_buff_type: mazeBuff.MazeBuffType,
        released_correction_ids: correctionIds,
      });
    }
  }
  return output;
}

const weaponLevels = levelRows(weaponCollections, "Weapon");
const accessoryLevels = levelRows(accessoryCollections, "Accessory");
const weaponProgramManifest = manifestRecord("config_programs", weaponProgramPath);
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
    const actorBindings = definition.source_numeric_id === "3113003"
      ? actorPrograms.map(({ file, program: actor, manifest: actorManifest }) => ({
        program_path: file,
        source_id: `source.goal16.${actorManifest.evidence_sha256.slice(0, 16)}`,
        ability_names: [...collectMatchingStrings(
          actor,
          /^StageAbility_VS_Weapon_S2_003/u,
        )].sort(),
        skill_types: [...collectStringFields(actor, "SkillType")].sort(),
        program_sha256: actorManifest.evidence_sha256,
      }))
      : [];
    const actorManifests = definition.source_numeric_id === "3113003"
      ? actorPrograms.map(({ manifest: actorManifest }) => actorManifest)
      : [];
    return {
      ...envelope({
        id: `${definition.id}.binding`,
        kind: `${kind}Binding`,
        nameEn: `${definition.name_en} battle binding`,
        nameZh: `${definition.name_zh_cn}战斗绑定`,
        summaryEn:
          `Structural summary of the released ${kind.toLowerCase()} program without copying source programs.`,
        summaryZh:
          `已发布${kind === "Weapon" ? "武器" : "配饰"}程序的结构摘要，不复制源程序。`,
        manifestIds: [
          ...definition.manifest_record_ids,
          programManifest.id,
          ...actorManifests.map(({ id }) => id),
        ],
        sourceRefs: [
          ...definition.source_refs,
          structuredSource(
            programManifest,
            "ExactProgram",
            "whole-file program digest; normalized row retains structural identifiers only",
          ),
          ...actorManifests.map((actorManifest) => structuredSource(
            actorManifest,
            "ExactProgram",
            "exact summoned-actor ability binding for Ranger's Badge",
          )),
        ],
        tags: ["binding", "demon-king", kind.toLowerCase()],
      }),
      parent_id: definition.id,
      ...summary,
      actor_program_bindings: actorBindings,
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
    `${profileId}:EvoBdSCForgeMaterial:${String(index).padStart(4, "0")}`;
  const recipeManifest = manifestRecord("synthesis_materials", recipeManifestId);
  const outputId = String(row.ForgeGearID);
  const outputDefinition = weapons.find(({ source_numeric_id: id }) =>
    id === outputId);
  if (outputDefinition === undefined || outputDefinition.tier === "Standard")
    throw new Error(`synthesis output missing: ${outputId}`);
  const materialEntries = Object.entries(row.MaterialGearList)
    .map(([gearId, requiredLevel]) => {
      const collection = collectionIndex.get(gearId);
      if (collection === undefined)
        throw new Error(`recipe material missing: ${gearId}`);
      const consumptionOrder = row.CostGearList.map(String).indexOf(gearId);
      return {
        gearId,
        requiredLevel,
        collection: collection.row,
        consumed: consumptionOrder !== -1,
        consumptionOrder: consumptionOrder === -1
          ? undefined
          : consumptionOrder,
      };
    })
    .sort((left, right) => left.gearId.localeCompare(right.gearId, "en"));
  const recipeId = `galactic-baseballer.demon-king.recipe.${outputId}`;
  const type = typeManifest(collectionIndex.get(outputId).row);
  synthesisRecipes.push({
    ...envelope({
      id: recipeId,
      kind: "SynthesisRecipe",
      nameEn: `Synthesize ${outputDefinition.name_en}`,
      nameZh: `合成${outputDefinition.name_zh_cn}`,
      summaryEn:
        `Exact acyclic ${outputDefinition.tier} synthesis recipe with explicit prerequisites and source consumption order.`,
      summaryZh:
        `精确无环的${outputDefinition.tier === "Legendary" ? "传说"
          : outputDefinition.tier === "Twin" ? "双重" : "至尊"}合成配方，含显式前置条件与源消耗顺序。`,
      manifestIds: [recipeManifestId, type.manifestId],
      sourceRefs: [
        structuredSource(
          recipeManifest,
          "ExactRelationship",
          "exact ForgeMaterial output, prerequisites and CostGearList order",
        ),
        structuredSource(
          type.manifest,
          "ExactRelationship",
          "exact output gear type",
        ),
      ],
      tags: ["demon-king", outputDefinition.tier.toLowerCase(), "synthesis"],
    }),
    tier: outputDefinition.tier,
    output_weapon_id: outputDefinition.id,
    input_count: materialEntries.length,
    validation_order: "input stable ID ascending",
    consumption_order: row.CostGearList.map((gearId, ordinal) => ({
      ordinal,
      input_source_numeric_id: String(gearId),
    })),
    candidate_precedence:
      "ProjectPolicy: Supreme, Twin, Legendary, then stable recipe ID; synthesis before ordinary duplicate upgrade",
    failure_behavior:
      "ProjectPolicy: reject without inventory mutation or resource consumption",
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
          `Ordered exact ${weapon ? "weapon" : "accessory"} prerequisite, level and consumption position.`,
        summaryZh:
          `有序的精确${weapon ? "武器" : "配饰"}前置、等级与消耗位置。`,
        manifestIds: [recipeManifestId],
        sourceRefs: [structuredSource(
          recipeManifest,
          "ExactRelationship",
          "exact ForgeMaterial input level and CostGearList position",
        )],
        tags: ["demon-king", "recipe-input", weapon ? "weapon" : "accessory"],
      }),
      recipe_id: recipeId,
      input_order: ordinal,
      input_kind: weapon ? "Weapon" : "Accessory",
      input_id: weapon
        ? `galactic-baseballer.demon-king.weapon.${material.gearId}`
        : `galactic-baseballer.demon-king.accessory.${material.gearId}`,
      required_level: material.requiredLevel,
      consumed: material.consumed,
      consumption_order: material.consumptionOrder,
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
  ["demon-weapons.json", weapons],
  ["demon-weapon-levels.json", weaponLevels],
  ["demon-weapon-triggers.json", weaponTriggers],
  ["demon-accessories.json", accessories],
  ["demon-accessory-levels.json", accessoryLevels],
  ["demon-accessory-bindings.json", accessoryBindings],
  ["demon-synthesis-recipes.json", synthesisRecipes],
  ["demon-synthesis-inputs.json", synthesisInputs],
]);
await mkdir(outputRoot, { recursive: true });
for (const [file, value] of outputs) {
  const target = path.join(outputRoot, file);
  const encoded = `${JSON.stringify(canonicalValue(value), null, 2)}\n`;
  if (check) {
    const existing = await readFile(target, "utf8");
    if (existing !== encoded)
      throw new Error(`Demon King arsenal drift: ${target}`);
  } else {
    await writeFile(target, encoded);
  }
}

console.log(
  `Demon King arsenal ${check ? "verified" : "wrote"}: `
  + `${weapons.length} weapons/${weaponLevels.length} levels, `
  + `${accessories.length} accessories/${accessoryLevels.length} levels, `
  + `${synthesisRecipes.length} recipes/${synthesisInputs.length} inputs`,
);
