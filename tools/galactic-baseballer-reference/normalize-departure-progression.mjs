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
const packRoot = path.join(
  root,
  "content-reference",
  "galactic-baseballer-v1",
);
const fragmentRoot = path.join(packRoot, "fragments");
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
  return index === -1 ? undefined : process.argv[index + 1];
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
  return createHash("sha256").update(canonical(value)).digest("hex");
}
function canonicalValue(value) {
  if (Array.isArray(value)) return value.map(canonicalValue);
  if (value !== null && typeof value === "object") {
    return Object.fromEntries(Object.entries(value)
      .map(([key, child]) => [key, canonicalValue(child)]));
  }
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
    ownership: "Departure",
    coverage_state: "Researched",
    evidence_quality: "ExactStructured",
    mechanism_quality: mechanismQuality,
    manifest_record_ids: [...new Set(manifestIds)].sort(),
    source_refs: sourceRefs,
    tags: [...tags].sort(),
  };
}

const shopSource = await readSource("ExcelOutput/EvolveBuildShopConfig.json");
const mazeBuffSource = await readSource("ExcelOutput/EvolveBuildMazeBuff.json");
const constantSource = await readSource(
  "ExcelOutput/EvolveBuildConstValueCommon.json",
);
const tutorialSource = await readSource("ExcelOutput/EvolveBuildTutorial.json");
const tagSource = await readSource("ExcelOutput/EvolveBuildTagConfig.json");
const stages = JSON.parse(await readFile(path.join(packRoot, "stages.json")));
const departureStages = stages.filter(({ profile_ids: ids }) =>
  ids.includes(profileId));

function constant(name) {
  const index = constantSource.findIndex(({ ConstValueName }) =>
    ConstValueName === name);
  if (index === -1) throw new Error(`constant missing: ${name}`);
  const manifestId =
    `${profileId}:EvolveBuildConstValueCommon:${String(index).padStart(4, "0")}`;
  return {
    value: canonicalValue(constantSource[index].Value),
    manifestId,
    manifest: manifestRecord("mode_constants", manifestId),
  };
}
function manifestForRow(category, family, index) {
  return manifestRecord(
    category,
    `${profileId}:${family}:${String(index).padStart(4, "0")}`,
  );
}

const currencyConstantNames = [
  "EvolveBuild_Coin_ItemID",
  "EvolveBuild_Gold_MaxLevel",
  "EvolveBuild_Gold_Basic_Normal1",
  "EvolveBuild_Gold_Basic_Normal2",
  "EvolveBuild_Gold_Basic_Elite",
  "EvolveBuild_Gold_Basic_Boss",
  "EvolveBuild_Gold_Basic_Special",
  "EvolveBuild_Gold_Basic_Chest",
  "EvolveBuild_Chest_Basic_Gold",
  "EvolveBuild_Chest_Alter_Gold",
  "EvolveBuild_Chest_Probability",
  "EvolveBuild_Chest_Probability_Step",
];
const currencyConstants = Object.fromEntries(currencyConstantNames.map(
  (name) => [name, constant(name)],
));
const currencies = [{
  ...envelope({
    id: "galactic-baseballer.departure.currency.raccoon-gold",
    kind: "Currency",
    nameEn: "Departure Raccoon Gold",
    nameZh: "启程篇浣熊硬币",
    summaryEn:
      "Exact profile-owned currency identity, cap, combat income and chest vectors.",
    summaryZh: "精确的 Profile 所有货币身份、上限、战斗收入与宝箱向量。",
    manifestIds: currencyConstantNames.map((name) =>
      currencyConstants[name].manifestId),
    sourceRefs: currencyConstantNames.map((name) => structuredSource(
      currencyConstants[name].manifest,
      "ExactRelationship",
      "exact released Departure currency constant",
    )),
    tags: ["currency", "departure", "persistent-progression"],
  }),
  state_owner: "profile-persistent Departure progression",
  source_item_id: String(
    currencyConstants.EvolveBuild_Coin_ItemID.value.IntValue,
  ),
  maximum_balance: null,
  maximum_balance_disposition: "UnspecifiedInFrozenStructuredFamily",
  gold_max_level_vector:
    currencyConstants.EvolveBuild_Gold_MaxLevel.value.ArrayValue
      .map(({ IntValue }) => IntValue),
  enemy_income: {
    normal_1:
      currencyConstants.EvolveBuild_Gold_Basic_Normal1.value.IntValue,
    normal_2:
      currencyConstants.EvolveBuild_Gold_Basic_Normal2.value.IntValue,
    elite: currencyConstants.EvolveBuild_Gold_Basic_Elite.value.IntValue,
    boss: currencyConstants.EvolveBuild_Gold_Basic_Boss.value.IntValue,
    special: currencyConstants.EvolveBuild_Gold_Basic_Special.value.IntValue,
  },
  chest_income_vector:
    currencyConstants.EvolveBuild_Gold_Basic_Chest.value.ArrayValue
      .map(({ IntValue }) => IntValue),
  chest_basic_gold:
    currencyConstants.EvolveBuild_Chest_Basic_Gold.value.ArrayValue
      .map(({ IntValue }) => IntValue),
  chest_alternate_gold:
    currencyConstants.EvolveBuild_Chest_Alter_Gold.value.ArrayValue
      .map(({ IntValue }) => IntValue),
  chest_probability_vector:
    currencyConstants.EvolveBuild_Chest_Probability.value.ArrayValue
      .map(({ DoubleValue, IntValue }) => DoubleValue ?? IntValue),
  chest_probability_step:
    currencyConstants.EvolveBuild_Chest_Probability_Step.value.ArrayValue
      .map(({ DoubleValue, IntValue }) => DoubleValue ?? IntValue),
}];

const shopUpgrades = [];
for (const [shopIndex, shop] of shopSource.entries()) {
  const shopManifest = manifestForRow(
    "shop_progression",
    "EvolveBuildShopConfig",
    shopIndex,
  );
  for (const [priceIndex, price] of shop.PriceList.entries()) {
    const purchaseLevel = price.MDBPDAJJLMG;
    const mazeIndex = shop.MazeBuffID === undefined ? -1
      : mazeBuffSource.findIndex(({ ID, Lv }) =>
        ID === shop.MazeBuffID && Lv === purchaseLevel);
    const mazeManifest = mazeIndex === -1 ? undefined : manifestForRow(
      "accessory_levels",
      "EvolveBuildMazeBuff",
      mazeIndex,
    );
    const mazeBuff = mazeIndex === -1 ? undefined : mazeBuffSource[mazeIndex];
    shopUpgrades.push({
      ...envelope({
        id:
          `galactic-baseballer.departure.shop.${shop.ID}.level.${purchaseLevel}`,
        kind: "ShopUpgrade",
        nameEn: `Departure store upgrade ${shop.ID} level ${purchaseLevel}`,
        nameZh: `启程篇商店升级 ${shop.ID} 等级 ${purchaseLevel}`,
        summaryEn:
          "Exact store price level, effect type and optional MazeBuff binding.",
        summaryZh: "精确商店价格等级、效果类型与可选 MazeBuff 绑定。",
        manifestIds: [
          shopManifest.id,
          ...(mazeManifest === undefined ? [] : [mazeManifest.id]),
        ],
        sourceRefs: [
          structuredSource(
            shopManifest,
            "ExactRelationship",
            "exact released Departure store definition and price list",
          ),
          ...(mazeManifest === undefined ? [] : [structuredSource(
            mazeManifest,
            "ExactProgram",
            "exact released Departure store MazeBuff level",
          )]),
        ],
        tags: ["departure", "persistent-progression", "shop"],
      }),
      source_numeric_id: String(shop.ID),
      purchase_level: purchaseLevel,
      maximum_level: shop.LvMax,
      cost_currency_id: currencies[0].source_item_id,
      cost: price.KDKPDJNMMCM,
      shop_type: shop.ShopType,
      shop_parameter_values: (shop.ParamList ?? [])
        .map(({ Value }) => String(Value)),
      maze_buff_id: mazeBuff === undefined ? undefined : String(mazeBuff.ID),
      maze_buff_parameters: mazeBuff === undefined ? []
        : mazeBuff.ParamList.map(({ Value }) => String(Value)),
      state_owner: "profile-persistent Departure progression",
      failure_behavior:
        "ProjectPolicy: validate level and balance before atomic commit",
      price_order: priceIndex,
    });
  }
}

const tutorialUnlocks = tutorialSource.map((row, index) => {
  const record = manifestForRow(
    "tutorial_entries",
    "EvolveBuildTutorial",
    index,
  );
  return {
    ...envelope({
      id:
        `galactic-baseballer.departure.unlock.tutorial.${row.StageMergedID}.${row.ID}`,
      kind: "UnlockRule",
      nameEn: `Departure tutorial locator ${row.TutorialID}`,
      nameZh: `启程篇教程定位 ${row.TutorialID}`,
      summaryEn:
        "Mechanical tutorial locator retained without importing presentation content.",
      summaryZh: "保留机械教程定位，不导入表现内容。",
      manifestIds: [record.id],
      sourceRefs: [structuredSource(
        record,
        "ContextOnly",
        "exact stage/tutorial locator; presentation content excluded",
      )],
      tags: ["departure", "evidence-only", "tutorial"],
      mechanismQuality: "ContextOnly",
    }),
    unlock_order: index,
    unlock_kind: "TutorialLocator",
    stage_numeric_id: String(row.StageMergedID),
    tutorial_sequence_id: String(row.ID),
    tutorial_id: String(row.TutorialID),
    disposition: "EvidenceOnlyPresentationLocator",
    runtime_effect: false,
  };
});

const tagDefinitions = tagSource.map((row, index) => {
  const record = manifestForRow(
    "content_tags",
    "EvolveBuildTagConfig",
    index,
  );
  return {
    ...envelope({
      id: `galactic-baseballer.departure.progression.tag.${row.ID}`,
      kind: "ProgressionRule",
      nameEn: `Departure store tag ${row.ID}`,
      nameZh: `启程篇商店标签 ${row.ID}`,
      summaryEn:
        "Exact released store tag definition retained without inferring membership.",
      summaryZh: "保留精确正式发布商店标签定义，不推断成员关系。",
      manifestIds: [record.id],
      sourceRefs: [structuredSource(
        record,
        "ExactRelationship",
        "exact released profile-owned store tag definition",
      )],
      tags: ["departure", "progression", "store-tag"],
    }),
    progression_kind: "StoreTagDefinition",
    source_numeric_id: String(row.ID),
    source_name_hash: String(row.TagName?.Hash ?? ""),
    membership_inference_allowed: false,
  };
});

const stageByBonusId = new Map(departureStages.map((stage) => [
  stage.team_bonus_maze_buff_id,
  stage,
]));
const teamBonusRows = mazeBuffSource
  .map((row, index) => ({ row, index }))
  .filter(({ row }) => row.ID >= 3106601 && row.ID <= 3106608)
  .map(({ row, index }) => {
    const record = manifestForRow(
      "accessory_levels",
      "EvolveBuildMazeBuff",
      index,
    );
    const stage = stageByBonusId.get(String(row.ID));
    return {
      ...envelope({
        id: `galactic-baseballer.departure.progression.team-bonus.${row.ID}`,
        kind: "ProgressionRule",
        nameEn: `Departure team bonus ${row.ID}`,
        nameZh: `启程篇队伍加成 ${row.ID}`,
        summaryEn:
          "Exact released team-bonus definition with an explicit stage binding when authored.",
        summaryZh: "精确正式发布队伍加成定义；仅在有作者关系时绑定关卡。",
        manifestIds: [
          record.id,
          ...(stage?.manifest_record_ids ?? []),
        ],
        sourceRefs: [
          structuredSource(
            record,
            "ExactProgram",
            "exact released team-bonus MazeBuff definition",
          ),
          ...(stage?.source_refs ?? []),
        ],
        tags: ["departure", "progression", "team-bonus"],
      }),
      progression_kind: "TeamBonusDefinition",
      source_maze_buff_id: String(row.ID),
      source_level: row.Lv,
      parameter_values: row.ParamList.map(({ Value }) => String(Value)),
      binding_key: row.InBattleBindingKey,
      stage_id: stage?.id ?? null,
      source_disposition: stage === undefined
        ? "ReleasedDefinitionWithoutAuthoredStageBinding"
        : "ExactStageBinding",
    };
  });
const progression = [...tagDefinitions, ...teamBonusRows]
  .sort((left, right) => left.id.localeCompare(right.id, "en"));

const outputs = new Map([
  ["departure-currencies.json", currencies],
  ["departure-shop-upgrades.json", shopUpgrades],
  ["departure-unlocks.json", tutorialUnlocks],
  ["departure-progression.json", progression],
]);
await mkdir(fragmentRoot, { recursive: true });
for (const [file, value] of outputs) {
  const target = path.join(fragmentRoot, file);
  const encoded = `${JSON.stringify(canonicalValue(value), null, 2)}\n`;
  if (check) {
    const existing = await readFile(target, "utf8");
    if (existing !== encoded)
      throw new Error(`Departure progression drift: ${target}`);
  } else {
    await writeFile(target, encoded);
  }
}
console.log(
  `Departure progression ${check ? "verified" : "wrote"}: `
  + `${currencies.length} currency, ${shopUpgrades.length} shop levels, `
  + `${tutorialUnlocks.length} tutorial locators, `
  + `${tagDefinitions.length} tags and ${teamBonusRows.length} team bonuses`,
);
