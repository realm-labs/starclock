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
const publicRoot = path.join(cache, "public-revisions");
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
const publicInventory = JSON.parse(await readFile(path.join(
  root,
  "content-manifests",
  "galactic-baseballer-v1",
  "public-source-inventory.json",
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

async function communitySource(title, locator, note) {
  const receipt = publicInventory.community_pages.find(({ title: value }) =>
    value === title);
  if (receipt === undefined) throw new Error(`public receipt missing: ${title}`);
  const content = await readFile(
    path.join(publicRoot, `${receipt.revision_id}.wikitext`),
    "utf8",
  );
  if (digest(Buffer.from(content)) !== receipt.content_sha256)
    throw new Error(`public revision digest drift: ${title}`);
  if (!content.includes(locator))
    throw new Error(`public revision locator missing: ${title}/${locator}`);
  return {
    source_id: `source.goal16.wiki.${receipt.revision_id}.${digest(locator).slice(0, 10)}`,
    repository_or_url: receipt.url,
    revision_or_access_date: String(receipt.revision_id),
    game_version: "3.3 released / 4.4 retained",
    path_or_page: title,
    locator,
    sha256: receipt.content_sha256,
    evidence_quality: "PublicCommunityCrossCheck",
    mechanism_quality: "IdentityCrossCheck",
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
  evidenceQuality = "ExactStructured",
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
    evidence_quality: evidenceQuality,
    mechanism_quality: mechanismQuality,
    manifest_record_ids: [...new Set(manifestIds)].sort(),
    source_refs: sourceRefs,
    tags: [...tags].sort(),
  };
}

const constantPath = "ExcelOutput/EvoBdSCConstValueCommon.json";
const boxGroupPath = "ExcelOutput/EvoBdSCBoxGroup.json";
const boxItemPath = "ExcelOutput/EvoBdSCBoxItem.json";
const mazePath = "ExcelOutput/EvoBdSCMazeBuff.json";
const shopPath = "ExcelOutput/EvoBdSCShopConfig.json";
const tagPath = "ExcelOutput/EvoBdSCTagConfig.json";
const tutorialPath = "ExcelOutput/EvoBdSCTutorial.json";
const constants = await readSource(constantPath);
const boxGroups = await readSource(boxGroupPath);
const boxItems = await readSource(boxItemPath);
const mazeBuffs = await readSource(mazePath);
const shops = await readSource(shopPath);
const tags = await readSource(tagPath);
const tutorials = await readSource(tutorialPath);
const chs = await readSource("TextMap/TextMapCHS.json");
const en = await readSource("TextMap/TextMapEN.json");
const weapons = JSON.parse(await readFile(
  path.join(outputRoot, "demon-weapons.json"),
  "utf8",
));
const accessories = JSON.parse(await readFile(
  path.join(outputRoot, "demon-accessories.json"),
  "utf8",
));

function constant(name) {
  const fullName = `EvolveBuildSC_${name}`;
  const index = constants.findIndex(({ ConstValueName }) =>
    ConstValueName === fullName);
  if (index === -1) throw new Error(`constant missing: ${fullName}`);
  const manifestId =
    `${profileId}:EvoBdSCConstValueCommon:${String(index).padStart(4, "0")}`;
  return {
    value: canonicalValue(constants[index].Value),
    manifestId,
    manifest: manifestRecord("mode_constants", manifestId),
  };
}

function constantIds(names) {
  return names.map((name) => constant(name).manifestId);
}

function constantRefs(names, note) {
  return names.map((name) =>
    structuredSource(constant(name).manifest, "ExactRelationship", note));
}

const reputationTitle =
  "Legend of the Galactic Baseballer: Demon King/Cosmic Reputation";
const reputationReceipt = publicInventory.community_pages.find(({ title }) =>
  title === reputationTitle);
if (reputationReceipt === undefined)
  throw new Error("Cosmic Reputation receipt missing");
const reputationContent = await readFile(
  path.join(publicRoot, `${reputationReceipt.revision_id}.wikitext`),
  "utf8",
);
if (digest(Buffer.from(reputationContent)) !==
  reputationReceipt.content_sha256)
  throw new Error("Cosmic Reputation revision digest drift");
const fallbackCostMatch = reputationContent.match(
  /^\|cost\s*=\s*(\d+)\s*$/mu,
);
if (fallbackCostMatch === null) throw new Error("reputation fallback cost missing");
const fallbackCost = Number(fallbackCostMatch[1]);
const reputationCosts = Array.from({ length: 20 }, (_, index) => {
  const level = index + 1;
  const match = reputationContent.match(
    new RegExp(`^\\|cost_${level}\\s*=\\s*(\\d+)\\s*$`, "mu"),
  );
  return match === null ? fallbackCost : Number(match[1]);
});
if (reputationCosts.join(",") !== [
  ...Array(5).fill(3000),
  ...Array(5).fill(4500),
  ...Array(10).fill(6000),
].join(",")) throw new Error("Cosmic Reputation cost vector drift");
const reputationSource = await communitySource(
  reputationTitle,
  "==Reputation Rewards==",
  "revision-pinned 20-rank cost cross-check; account rewards are excluded",
);
const storeSource = await communitySource(
  "Legend of the Galactic Baseballer: Demon King/Cosmic Store",
  "==Stat Boost==",
  "revision-pinned store identity/effect cross-check",
);

const raccoonConstantNames = [
  "Gold_Basic_Normal1",
  "Gold_Basic_Normal2",
  "Gold_Basic_Elite",
  "Gold_Basic_Boss",
  "Gold_Basic_Chest",
  "Gold_MaxLevel",
  "Chest_Basic_Gold",
  "Chest_Alter_Gold",
  "Chest_Probability",
  "Chest_Probability_Step",
  "Coin_ItemID",
  "Gold_MaxLimit",
];
const reputationConstantNames = ["OfferingType", "Offering_MaxLimit"];
const currencies = [
  {
    ...envelope({
      id: "galactic-baseballer.demon-king.currency.raccoon-gold",
      kind: "Currency",
      nameEn: "Raccoon Gold Coin",
      nameZh: "浣熊金币",
      summaryEn:
        "Persistent Demon King shop currency with exact item ID, enemy/chest income vectors and maximum balance.",
      summaryZh: "魔王篇持久商店货币，含精确物品 ID、敌人/宝箱收入向量与余额上限。",
      manifestIds: constantIds(raccoonConstantNames),
      sourceRefs: [
        ...constantRefs(
          raccoonConstantNames,
          "exact released Raccoon Gold constant",
        ),
        storeSource,
      ],
      tags: ["currency", "demon-king", "shop"],
    }),
    state_owner: "profile-persistent Demon King progression",
    source_item_id: String(constant("Coin_ItemID").value.IntValue),
    maximum_balance: constant("Gold_MaxLimit").value.IntValue,
    enemy_income: {
      normal_1: constant("Gold_Basic_Normal1").value.IntValue,
      normal_2: constant("Gold_Basic_Normal2").value.IntValue,
      elite: constant("Gold_Basic_Elite").value.IntValue,
      boss: constant("Gold_Basic_Boss").value.IntValue,
    },
    chest_income_by_authored_ordinal:
      constant("Gold_Basic_Chest").value.ArrayValue
        .map((entry, ordinal) => ({
          ordinal,
          value: entry.IntValue,
        })),
    chest_probability_vector:
      constant("Chest_Probability").value.ArrayValue
        .map((entry, ordinal) => ({
          ordinal,
          value: String(entry.DoubleValue ?? entry.IntValue),
        })),
    chest_probability_step_vector:
      constant("Chest_Probability_Step").value.ArrayValue
        .map((entry, ordinal) => ({
          ordinal,
          value: String(entry.DoubleValue ?? entry.IntValue),
        })),
    probability_ordinal_mapping_status:
      "Unspecified: exact vectors retained without naming hidden ordinal meanings",
  },
  {
    ...envelope({
      id: "galactic-baseballer.demon-king.currency.cosmic-reputation",
      kind: "Currency",
      nameEn: "Cosmic Reputation",
      nameZh: "银河声望",
      summaryEn:
        "Reference-only 20-rank progression currency; reward payloads remain excluded account locators.",
      summaryZh: "仅供资料使用的 20 级进度货币；奖励内容继续作为排除的账号定位信息。",
      manifestIds: constantIds(reputationConstantNames),
      sourceRefs: [
        ...constantRefs(
          reputationConstantNames,
          "exact offering type and maximum",
        ),
        reputationSource,
      ],
      tags: ["currency", "demon-king", "reputation"],
      evidenceQuality: "ApproximateFromReleasedText",
      mechanismQuality: "ContextOnly",
    }),
    state_owner: "profile-persistent account progression",
    offering_type: constant("OfferingType").value.IntValue,
    maximum_balance: constant("Offering_MaxLimit").value.IntValue,
    rank_count: 20,
    account_reward_payloads_imported: false,
    approximation_id:
      "galactic-baseballer.demon-king.approximation.reputation-rank-costs",
  },
];

let cumulativeReputation = 0;
const progression = reputationCosts.map((cost, index) => {
  cumulativeReputation += cost;
  const rank = index + 1;
  return {
    ...envelope({
      id: `galactic-baseballer.demon-king.progression.reputation.${rank}`,
      kind: "ProgressionRule",
      nameEn: `Cosmic Reputation rank ${rank}`,
      nameZh: `银河声望等级 ${rank}`,
      summaryEn:
        "Revision-pinned public rank cost retained without importing account rewards.",
      summaryZh: "保留固定公开修订的等级费用，不导入账号奖励。",
      manifestIds: constantIds(reputationConstantNames),
      sourceRefs: [
        ...constantRefs(
          reputationConstantNames,
          "exact offering type and maximum",
        ),
        reputationSource,
      ],
      tags: ["demon-king", "progression", "reputation"],
      evidenceQuality: "ApproximateFromReleasedText",
      mechanismQuality: "ContextOnly",
    }),
    progression_kind: "CosmicReputationRank",
    rank,
    incremental_cost: cost,
    cumulative_cost: cumulativeReputation,
    mechanical_effects: [],
    account_reward_disposition: "EvidenceOnlyNotImported",
    approximation_id:
      "galactic-baseballer.demon-king.approximation.reputation-rank-costs",
  };
});

const collectionByNumericId = new Map(
  [...weapons, ...accessories].map((record) => [
    record.source_numeric_id,
    record.id,
  ]),
);
const boxItemById = new Map(boxItems.map((row) => [String(row.ID), row]));
for (const [index, row] of boxGroups.entries()) {
  const manifestId =
    `${profileId}:EvoBdSCBoxGroup:${String(index).padStart(4, "0")}`;
  const record = manifestRecord("offer_box_groups", manifestId);
  progression.push({
    ...envelope({
      id: `galactic-baseballer.demon-king.progression.treasure-group.${row.GroupID}`,
      kind: "ProgressionRule",
      nameEn: `Demon King's Treasure group ${row.GroupID}`,
      nameZh: `魔王宝藏组 ${row.GroupID}`,
      summaryEn: "Exact ordered list of authored treasure item-pool IDs.",
      summaryZh: "精确有序的宝藏物品池 ID 列表。",
      manifestIds: [manifestId],
      sourceRefs: [structuredSource(
        record,
        "ExactRelationship",
        "exact BoxGroup to BoxItem list",
      )],
      tags: ["demon-king", "treasure", "candidate-group"],
    }),
    progression_kind: "TreasureGroup",
    source_group_id: String(row.GroupID),
    box_item_pool_ids: row.BoxItemIDList.map((id, ordinal) => ({
      ordinal,
      source_box_item_id: String(id),
    })),
    selection_approximation_id:
      "galactic-baseballer.demon-king.approximation.treasure-selection",
  });
}
for (const [index, row] of boxItems.entries()) {
  const manifestId =
    `${profileId}:EvoBdSCBoxItem:${String(index).padStart(4, "0")}`;
  const record = manifestRecord("offer_box_items", manifestId);
  const candidates = row.ItemIDList.map((id, ordinal) => {
    const stableId = collectionByNumericId.get(String(id));
    if (stableId === undefined)
      throw new Error(`treasure candidate collection missing: ${id}`);
    return { ordinal, source_numeric_id: String(id), stable_id: stableId };
  });
  progression.push({
    ...envelope({
      id: `galactic-baseballer.demon-king.progression.treasure-pool.${row.ID}`,
      kind: "ProgressionRule",
      nameEn: `Demon King's Treasure pool ${row.ID}`,
      nameZh: `魔王宝藏物品池 ${row.ID}`,
      summaryEn:
        "Exact authored candidate-entry order with duplicates preserved as distinct positions.",
      summaryZh: "精确作者候选条目顺序；重复项作为不同位置保留。",
      manifestIds: [manifestId],
      sourceRefs: [structuredSource(
        record,
        "ExactRelationship",
        "exact BoxItem candidate entry list",
      )],
      tags: ["candidate-pool", "demon-king", "treasure"],
    }),
    progression_kind: "TreasureCandidatePool",
    source_box_item_id: String(row.ID),
    candidate_entries: candidates,
    rng_label:
      "galactic-baseballer/{profile-id}/{activity-instance-id}/treasure/{decision-ordinal}",
    selection_approximation_id:
      "galactic-baseballer.demon-king.approximation.treasure-selection",
  });
}
for (const group of boxGroups) {
  for (const id of group.BoxItemIDList) {
    if (!boxItemById.has(String(id)))
      throw new Error(`BoxGroup reference missing: ${id}`);
  }
}

const mazeIndex = new Map(mazeBuffs.map((row, index) => [
  `${row.ID}:${row.Lv}`,
  { row, index },
]));
const tagByShopId = new Map(tags.map((row, index) => [
  String(row.ShopSkillID),
  { row, index },
]));
const shopUpgrades = [];
for (const [index, shop] of shops.entries()) {
  const shopManifestId =
    `${profileId}:EvoBdSCShopConfig:${String(index).padStart(4, "0")}`;
  const shopManifest = manifestRecord("shop_progression", shopManifestId);
  const nameHash = String(shop.Name.Hash);
  if (typeof en[nameHash] !== "string" || typeof chs[nameHash] !== "string")
    throw new Error(`shop localization missing: ${shop.ID}`);
  const tag = tagByShopId.get(String(shop.ID));
  const tagManifestId = tag === undefined ? undefined
    : `${profileId}:EvoBdSCTagConfig:${String(tag.index).padStart(4, "0")}`;
  const tagManifest = tagManifestId === undefined ? undefined
    : manifestRecord("content_tags", tagManifestId);
  for (const [levelIndex, price] of shop.PriceList.entries()) {
    const level = levelIndex + 1;
    if (price.MDBPDAJJLMG !== level)
      throw new Error(`shop price level drift: ${shop.ID}/${level}`);
    const maze = shop.MazeBuffID === undefined
      ? undefined
      : mazeIndex.get(`${shop.MazeBuffID}:${level}`);
    if (shop.MazeBuffID !== undefined && maze === undefined)
      throw new Error(`shop MazeBuff missing: ${shop.ID}/${level}`);
    const mazeManifestId = maze === undefined ? undefined
      : `${profileId}:EvoBdSCMazeBuff:${String(maze.index).padStart(4, "0")}`;
    const mazeManifest = mazeManifestId === undefined ? undefined
      : manifestRecord("accessory_levels", mazeManifestId);
    shopUpgrades.push({
      ...envelope({
        id: `galactic-baseballer.demon-king.shop.${shop.ID}.level.${level}`,
        kind: "ShopUpgrade",
        nameEn: `${en[nameHash]} level ${level}`,
        nameZh: `${chs[nameHash]} 等级 ${level}`,
        summaryEn:
          "Exact persistent Cosmic Store price step and mechanical effect binding.",
        summaryZh: "精确持久银河商店价格等级与机械效果绑定。",
        manifestIds: [
          shopManifestId,
          ...(mazeManifestId === undefined ? [] : [mazeManifestId]),
          ...(tagManifestId === undefined ? [] : [tagManifestId]),
        ],
        sourceRefs: [
          structuredSource(
            shopManifest,
            "ExactRelationship",
            "exact shop type, price level and base parameter",
          ),
          ...(mazeManifest === undefined ? [] : [structuredSource(
            mazeManifest,
            "ExactProgram",
            "exact purchased MazeBuff level and cumulative parameter vector",
          )]),
          ...(tagManifest === undefined ? [] : [structuredSource(
            tagManifest,
            "ExactRelationship",
            "exact weapon-tag effect and ShopSkill binding",
          )]),
          storeSource,
        ],
        tags: ["demon-king", "persistent", "shop-upgrade"],
      }),
      source_numeric_id: String(shop.ID),
      purchase_level: level,
      maximum_level: shop.LvMax,
      shop_type: shop.ShopType,
      cost_currency_id:
        "galactic-baseballer.demon-king.currency.raccoon-gold",
      cost: price.KDKPDJNMMCM,
      maze_buff_id: shop.MazeBuffID === undefined
        ? undefined
        : String(shop.MazeBuffID),
      maze_buff_parameters: maze?.row.ParamList.map(({ Value }) =>
        String(Value)) ?? [],
      shop_parameter_values: shop.ParamList.map(({ Value }) => String(Value)),
      content_tag: tag === undefined ? undefined : {
        tag_id: String(tag.row.ID),
        extra_effect_id: String(tag.row.ExtraEffectID),
      },
      state_owner: "profile-persistent Demon King progression",
      failure_behavior:
        "ProjectPolicy: insufficient balance or wrong current level rejects without mutation",
    });
  }
}

const unlockNames = [
  "Shop_UnlockQuest",
  "Reward_UnlockQuest",
  "Reset_UnlockQuest",
  "Remove_UnlockQuest",
  "Skip_UnlockQuest",
  "Reset_ShopUnlockQuest",
  "Remove_ShopUnlockQuest",
  "Card_UnlockQuest",
  "SkipOriginStage_UnlockQuest",
  "OptionalBox_QuestID",
];
const unlocks = unlockNames.map((name, unlockOrder) => ({
  ...envelope({
    id: `galactic-baseballer.demon-king.unlock.${name.toLowerCase()}`,
    kind: "UnlockRule",
    nameEn: `Demon King unlock: ${name}`,
    nameZh: `魔王篇解锁：${name}`,
    summaryEn: "Exact structured quest locator and mechanical/account disposition.",
    summaryZh: "精确结构化任务定位与机械/账号归属。",
    manifestIds: constantIds([name]),
    sourceRefs: constantRefs([name], "exact released unlock constant"),
    tags: ["demon-king", "unlock"],
  }),
  unlock_order: unlockOrder,
  unlock_kind: name,
  quest_id: String(constant(name).value.IntValue),
  disposition: name === "Reward_UnlockQuest"
    ? "EvidenceOnlyAccountReward"
    : "ReferenceOnlyMechanical",
}));
for (const [index, tutorial] of tutorials.entries()) {
  const manifestId =
    `${profileId}:EvoBdSCTutorial:${String(index).padStart(4, "0")}`;
  const record = manifestRecord("tutorial_entries", manifestId);
  unlocks.push({
    ...envelope({
      id:
        `galactic-baseballer.demon-king.unlock.tutorial.${tutorial.StageMergedID}.${tutorial.ID}`,
      kind: "UnlockRule",
      nameEn: `Tutorial locator ${tutorial.TutorialID}`,
      nameZh: `教程定位 ${tutorial.TutorialID}`,
      summaryEn:
        "Mechanical tutorial locator retained without importing presentation content.",
      summaryZh: "保留机械教程定位，不导入表现内容。",
      manifestIds: [manifestId],
      sourceRefs: [structuredSource(
        record,
        "ContextOnly",
        "exact stage/tutorial locator; presentation content excluded",
      )],
      tags: ["demon-king", "evidence-only", "tutorial"],
      mechanismQuality: "ContextOnly",
    }),
    unlock_order: unlockNames.length + index,
    unlock_kind: "TutorialLocator",
    stage_numeric_id: String(tutorial.StageMergedID),
    tutorial_sequence_id: String(tutorial.ID),
    tutorial_id: String(tutorial.TutorialID),
    disposition: "EvidenceOnlyPresentationLocator",
    runtime_effect: false,
  });
}

for (const rows of [currencies, progression, shopUpgrades, unlocks])
  rows.sort((left, right) => left.id.localeCompare(right.id, "en"));
const outputs = new Map([
  ["demon-currencies.json", currencies],
  ["demon-progression.json", progression],
  ["demon-shop-upgrades.json", shopUpgrades],
  ["demon-unlocks.json", unlocks],
]);
await mkdir(outputRoot, { recursive: true });
for (const [file, value] of outputs) {
  const target = path.join(outputRoot, file);
  const encoded = `${JSON.stringify(canonicalValue(value), null, 2)}\n`;
  if (check) {
    const existing = await readFile(target, "utf8");
    if (existing !== encoded)
      throw new Error(`Demon King progression drift: ${target}`);
  } else {
    await writeFile(target, encoded);
  }
}

console.log(
  `Demon King progression ${check ? "verified" : "wrote"}: `
  + `${currencies.length} currencies, ${progression.length} progression rows, `
  + `${shopUpgrades.length} shop levels, ${unlocks.length} unlock rows`,
);
