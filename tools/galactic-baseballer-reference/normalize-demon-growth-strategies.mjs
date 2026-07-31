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
const approximationRegister = JSON.parse(await readFile(path.join(
  root,
  "content-manifests",
  "galactic-baseballer-v1",
  "approximation-register.json",
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

function policySource(policyIds, note) {
  const records = approximationRegister.records.filter(({ id }) =>
    policyIds.includes(id));
  if (records.length !== policyIds.length)
    throw new Error(`missing policy: ${policyIds.join(",")}`);
  return {
    source_id: `source.goal16.policy.${digest(records).slice(0, 16)}`,
    repository_or_url: "starclock",
    revision_or_access_date: approximationRegister.schema_revision,
    game_version: "4.4",
    path_or_page:
      "content-manifests/galactic-baseballer-v1/approximation-register.json",
    locator: policyIds.join(","),
    sha256: digest(records),
    evidence_quality: "ProjectPolicy",
    mechanism_quality: "PolicyBoundary",
    note,
    replacement_condition:
      "replace only when released structured logic or reproducible public observation closes the named hidden behavior",
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

const constantPath = "ExcelOutput/EvoBdSCConstValueCommon.json";
const cardPath = "ExcelOutput/EvoBdSCCardConfig.json";
const cardTypePath = "ExcelOutput/EvoBdSCCardType.json";
const mazePath = "ExcelOutput/EvoBdSCMazeBuff.json";
const expProgramPath =
  "Config/ConfigAbility/BattleEvent/EvolveBuild_00_ExpAndLevel.json";
const cardProgramPath =
  "Config/ConfigAbility/BattleEvent/EvolveBuild_03_Card_SC.json";
const constants = await readSource(constantPath);
const cards = await readSource(cardPath);
const cardTypes = await readSource(cardTypePath);
const mazeBuffs = await readSource(mazePath);
const expProgram = await readSource(expProgramPath);
const cardProgram = await readSource(cardProgramPath);
const chs = await readSource("TextMap/TextMapCHS.json");
const en = await readSource("TextMap/TextMapEN.json");
const expProgramManifest = manifestRecord("config_programs", expProgramPath);
const cardProgramManifest = manifestRecord("config_programs", cardProgramPath);
const strategyPublicSource = await communitySource(
  "Legend of the Galactic Baseballer: Demon King/Adventure Strategy",
  "==General Enhancement==",
  "revision-pinned released strategy identity/effect cross-check",
);

function constant(name) {
  const fullName = `EvolveBuildSC_${name}`;
  const index = constants.findIndex(({ ConstValueName }) =>
    ConstValueName === fullName);
  if (index === -1) throw new Error(`constant missing: ${fullName}`);
  const manifestId =
    `${profileId}:EvoBdSCConstValueCommon:${String(index).padStart(4, "0")}`;
  return {
    name: fullName,
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

const thresholdNames = [
  "Exp_Multiplier_Wave_N",
  "Exp_LvN",
  "Exp_Basic_Normal1",
  "Exp_Basic_Normal2",
  "Exp_Basic_Elite",
  "Exp_Basic_Boss",
];
const basicAbility = expProgram.AbilityList.find(({ Name }) =>
  Name === "StageAbility_VS_Common_Basic");
if (basicAbility === undefined) throw new Error("shared exp program missing");
const expThresholds = [];
function collectExpThresholds(value) {
  if (Array.isArray(value)) {
    for (const child of value) collectExpThresholds(child);
  } else if (value !== null && typeof value === "object") {
    if (value.DynamicKey?.Value === "expForLevel"
      && value.FixedValue?.Value !== undefined)
      expThresholds.push(value.FixedValue.Value);
    if (value.DynamicKey?.Value === "expForLevel"
      && value.Value?.FixedValue?.Value !== undefined)
      expThresholds.push(value.Value.FixedValue.Value);
    for (const child of Object.values(value)) collectExpThresholds(child);
  }
}
collectExpThresholds(basicAbility);
if ([...new Set(expThresholds)].join(",") !== "40")
  throw new Error("team level threshold drift");
const levelThresholds = [{
  ...envelope({
    id: "galactic-baseballer.demon-king.level-threshold.shared",
    kind: "LevelThreshold",
    nameEn: "Demon King team level threshold",
    nameZh: "魔王篇队伍等级阈值",
    summaryEn:
      "The shared released program initializes 40 experience per team level; Demon King constants provide exact scaling and enemy awards.",
    summaryZh:
      "共享已发布程序把每级经验初始化为 40；魔王篇常量提供精确缩放与敌人经验。",
    manifestIds: [...constantIds(thresholdNames), expProgramManifest.id],
    sourceRefs: [
      ...constantRefs(thresholdNames, "exact Demon King experience constant"),
      structuredSource(
        expProgramManifest,
        "ExactProgram",
        "shared whole-file program receipt and exact expForLevel initialization",
      ),
    ],
    tags: ["demon-king", "experience", "level-threshold"],
  }),
  experience_threshold: "40",
  wave_multiplier: String(constant("Exp_Multiplier_Wave_N").value.DoubleValue),
  level_scaling_parameters: constant("Exp_LvN").value.ArrayValue
    .map((entry) => String(entry.DoubleValue ?? entry.IntValue)),
  experience_awards: {
    normal_1: constant("Exp_Basic_Normal1").value.IntValue,
    normal_2: constant("Exp_Basic_Normal2").value.IntValue,
    elite: constant("Exp_Basic_Elite").value.IntValue,
    boss: constant("Exp_Basic_Boss").value.IntValue,
    special: "UnspecifiedNoDemonKingConstant",
  },
  program_summary: programSummary(
    expProgram,
    "StageAbility_VS_Common_LevelUp_InsertAbility",
  ),
}];

const mazeIndex = new Map(mazeBuffs.map((row, index) => [
  `${row.ID}:${row.Lv}`,
  { row, index },
]));
const typeIndex = new Map(cardTypes.map((row, index) => [
  row.Type ?? "General",
  {
    row,
    index,
    manifestId:
      `${profileId}:EvoBdSCCardType:${String(index).padStart(4, "0")}`,
  },
]));
const adventureStrategies = cards.map((card, index) => {
  const maze = mazeIndex.get(`${card.ID}:1`);
  if (maze === undefined) throw new Error(`strategy MazeBuff missing: ${card.ID}`);
  if (String(card.LvID) !== `${card.ID}1`)
    throw new Error(`strategy composite LvID drift: ${card.ID}`);
  const type = typeIndex.get(card.Type ?? "General");
  if (type === undefined) throw new Error(`strategy type missing: ${card.Type}`);
  const cardManifestId =
    `${profileId}:EvoBdSCCardConfig:${String(index).padStart(4, "0")}`;
  const mazeManifestId =
    `${profileId}:EvoBdSCMazeBuff:${String(maze.index).padStart(4, "0")}`;
  const cardManifest = manifestRecord("upgrade_cards", cardManifestId);
  const mazeManifest = manifestRecord("accessory_levels", mazeManifestId);
  const typeManifest = manifestRecord("upgrade_card_types", type.manifestId);
  const nameHash = String(maze.row.BuffName.Hash);
  if (typeof en[nameHash] !== "string" || typeof chs[nameHash] !== "string")
    throw new Error(`strategy localization missing: ${card.ID}`);
  const ultimate = String(card.ID) === "3113799";
  return {
    ...envelope({
      id: `galactic-baseballer.demon-king.strategy.${card.ID}`,
      kind: "AdventureStrategy",
      nameEn: en[nameHash],
      nameZh: chs[nameHash],
      summaryEn:
        "Demon King Adventure Strategy with exact type, level-one parameters, selection periods and structural program binding.",
      summaryZh: "魔王篇冒险策略，含精确类型、一级参数、可选阶段与程序结构绑定。",
      manifestIds: [
        cardManifestId,
        mazeManifestId,
        type.manifestId,
        cardProgramManifest.id,
      ],
      sourceRefs: [
        structuredSource(
          cardManifest,
          "ExactRelationship",
          "exact strategy identity, unlock and selection-period metadata",
        ),
        structuredSource(
          mazeManifest,
          "ExactProgram",
          "exact level-one strategy parameters and binding",
        ),
        structuredSource(
          typeManifest,
          "ExactRelationship",
          "exact strategy type definition",
        ),
        structuredSource(
          cardProgramManifest,
          "ExactProgram",
          "whole-file card program receipt; structural identifiers only",
        ),
        strategyPublicSource,
      ],
      tags: ["adventure-strategy", "demon-king", card.Type ?? "general"],
    }),
    source_numeric_id: String(card.ID),
    source_level_id: String(card.LvID),
    source_type: card.Type ?? "General",
    maximum_level: 1,
    unlock_quest_id: String(card.UnlockQuest),
    selectable_period_ids: card.CardSelectablePeriod.map(String),
    influence_scope: card.InfluenceScope ?? "Unspecified",
    card_parameters: card.ParamList.map(({ Value }) => String(Value)),
    maze_buff_id: String(maze.row.ID),
    maze_buff_parameters: maze.row.ParamList.map(({ Value }) => String(Value)),
    source_name_hash: nameHash,
    source_description_hash: String(maze.row.BuffDesc.Hash),
    program_summary: programSummary(cardProgram, maze.row.InBattleBindingKey),
    public_unlock_condition: ultimate
      ? "Reach the Demon King Challenge phase in the Demon King's Den"
      : undefined,
  };
});

const candidatePools = [...typeIndex.entries()].map(
  ([typeName, type], poolOrder) => {
    const typeManifest = manifestRecord("upgrade_card_types", type.manifestId);
    const strategies = adventureStrategies.filter(({ source_type: value }) =>
      value === typeName);
    return {
      ...envelope({
        id: `galactic-baseballer.demon-king.candidate.strategy.${typeName.toLowerCase()}`,
        kind: "CandidatePool",
        nameEn: `${typeName} Adventure Strategy pool`,
        nameZh: `${typeName} 冒险策略候选池`,
        summaryEn:
          "Exact source-type partition with deterministic stable-ID candidate ordering.",
        summaryZh: "精确源类型分区，采用确定性稳定 ID 候选顺序。",
        manifestIds: [
          type.manifestId,
          ...strategies.flatMap(({ manifest_record_ids: ids }) => ids),
        ],
        sourceRefs: [
          structuredSource(
            typeManifest,
            "ExactRelationship",
            "exact strategy type partition",
          ),
        ],
        tags: ["candidate", "demon-king", "strategy"],
      }),
      pool_order: poolOrder,
      strategy_type: typeName,
      candidate_ids: strategies.map(({ id }) => id).sort(),
    };
  },
);

const weightNames = Array.from({ length: 15 }, (_, index) =>
  `Weight${index + 1}`);
const decisionNames = [
  "LevelUp_AbilityName",
  "MazeBuffID_ReRoll",
  "MazeBuffID_LostCount",
  "Reset_UnlockQuest",
  "Remove_UnlockQuest",
  "Reset_Num",
  "Remove_Num",
  "Skip_UnlockQuest",
  "Card_UnlockQuest",
  "Card_Reset_Num",
];
const candidateManifestIds = [
  ...constantIds([...weightNames, ...decisionNames]),
  ...candidatePools.flatMap(({ manifest_record_ids: ids }) => ids),
];
const candidatePolicies = [
  {
    ...envelope({
      id: "galactic-baseballer.demon-king.candidate-policy.source-parameters",
      kind: "CandidatePolicy",
      nameEn: "Demon King candidate source parameters",
      nameZh: "魔王篇候选源参数",
      summaryEn:
        "Exact candidate resource counts and an unassigned 15-value source weight vector.",
      summaryZh: "精确候选资源次数与尚未指派含义的 15 项源权重向量。",
      manifestIds: candidateManifestIds,
      sourceRefs: constantRefs(
        [...weightNames, ...decisionNames],
        "exact Demon King candidate/decision constant",
      ),
      tags: ["candidate-policy", "demon-king"],
    }),
    decision_order: 0,
    weight_vector: weightNames.map((name, ordinal) => ({
      ordinal,
      source_constant: `EvolveBuildSC_${name}`,
      weight: constant(name).value.IntValue,
    })),
    weight_mapping_status:
      "Unspecified: no card/type mapping inferred from ordinal alone",
    reroll_count: constant("Reset_Num").value.IntValue,
    exclusion_count: constant("Remove_Num").value.IntValue,
    strategy_reroll_count: constant("Card_Reset_Num").value.IntValue,
    reroll_unlock_quest_id:
      String(constant("Reset_UnlockQuest").value.IntValue),
    exclusion_unlock_quest_id:
      String(constant("Remove_UnlockQuest").value.IntValue),
    strategy_unlock_quest_id:
      String(constant("Card_UnlockQuest").value.IntValue),
    skip_unlock_quest_id:
      String(constant("Skip_UnlockQuest").value.IntValue),
  },
  {
    ...envelope({
      id: "galactic-baseballer.demon-king.candidate-policy.deterministic-fallback",
      kind: "CandidatePolicy",
      nameEn: "Demon King deterministic candidate fallback",
      nameZh: "魔王篇确定性候选回退",
      summaryEn:
        "ReferenceOnly labeled integer RNG, stable ordering and failure boundary.",
      summaryZh: "仅供资料使用的标签化整数 RNG、稳定顺序与失败边界。",
      manifestIds: candidateManifestIds,
      sourceRefs: [policySource([
        "gb.policy.upgrade-candidate-weight",
        "gb.policy.upgrade-candidate-order",
        "gb.policy.no-legal-candidate",
        "gb.policy.refresh-exclusion",
      ], "explicit deterministic fallback; not observed parity")],
      tags: ["candidate-policy", "demon-king", "project-policy"],
      evidenceQuality: "ProjectPolicy",
      mechanismQuality: "PolicyBoundary",
    }),
    decision_order: 1,
    rng_label:
      "galactic-baseballer/{profile-id}/{activity-instance-id}/strategy-candidate/{decision-ordinal}",
    candidate_order: "stable Starclock strategy ID ascending",
    selected_policy:
      "uniform project integer sampling until released weight-to-candidate mapping is available",
    rejected_alternatives: [
      "guess the meaning of Weight1 through Weight15 from ordinal position",
      "use localized names or collection iteration order",
    ],
    affected_fixture_ids: [
      "random-upgrade-candidates",
      "adventure-strategy",
      "no-legal-candidate-failure-invariance",
    ],
    confidence: "Low",
    replacement_condition:
      "released logic maps all 15 weights, legal periods, rerolls and empty-pool branches",
  },
];

const slotNames = [
  "Slot_Weapon_Unlocked",
  "Slot_Weapon_Total",
  "Slot_Accessory_Unlocked",
  "Slot_Accessory_Total",
  "OriginStage_Weapon_Slot",
  "OriginStage_Accessory_Slot",
];
function slotPolicy(kind, scope, unlockedName, totalName) {
  return {
    ...envelope({
      id: `galactic-baseballer.demon-king.inventory-slot.${scope}.${kind.toLowerCase()}`,
      kind: "InventorySlotPolicy",
      nameEn: `Demon King ${scope} ${kind} slots`,
      nameZh: `魔王篇${scope}${kind === "Weapon" ? "武器" : "配饰"}槽位`,
      summaryEn: `Exact ${scope} ${kind.toLowerCase()} slot capacity.`,
      summaryZh: `精确${scope}${kind === "Weapon" ? "武器" : "配饰"}槽位容量。`,
      manifestIds: constantIds([unlockedName, totalName]),
      sourceRefs: constantRefs(
        [unlockedName, totalName],
        "exact Demon King slot-capacity constant",
      ),
      tags: ["demon-king", kind.toLowerCase(), "slot"],
    }),
    slot_kind: kind,
    scope,
    initially_unlocked: constant(unlockedName).value.IntValue,
    total_capacity: constant(totalName).value.IntValue,
  };
}
const inventorySlots = [
  slotPolicy("Weapon", "Standard", "Slot_Weapon_Unlocked", "Slot_Weapon_Total"),
  slotPolicy(
    "Accessory",
    "Standard",
    "Slot_Accessory_Unlocked",
    "Slot_Accessory_Total",
  ),
  slotPolicy(
    "Weapon",
    "OriginStage",
    "OriginStage_Weapon_Slot",
    "OriginStage_Weapon_Slot",
  ),
  slotPolicy(
    "Accessory",
    "OriginStage",
    "OriginStage_Accessory_Slot",
    "OriginStage_Accessory_Slot",
  ),
];

const operationSpecs = [
  ["acquire-new", "AcquireNew", "insert only into an unlocked empty slot"],
  ["upgrade-duplicate", "UpgradeDuplicate", "increment the exact owned item when below maximum"],
  ["reject-max-duplicate", "RejectMaximumDuplicate", "reject a maximum-level duplicate without mutation"],
  ["reject-full-new", "RejectFullInventoryNew", "reject a new item when no unlocked slot exists"],
  ["expand-slot", "ExpandSlot", "increase unlocked capacity without exceeding authored total"],
];
const inventoryFamilyIds = [
  "weapon-acquisition-duplicate-upgrade",
  "accessory-acquisition-duplicate-upgrade",
  "slot-capacity-expansion-replacement",
  "no-legal-candidate-failure-invariance",
];
const inventoryOperations = operationSpecs.map(
  ([suffix, operation, selectedPolicy], operationOrder) => ({
    ...envelope({
      id: `galactic-baseballer.demon-king.inventory-operation.${suffix}`,
      kind: "InventoryOperation",
      nameEn: `Demon King inventory operation: ${operation}`,
      nameZh: `魔王篇库存操作：${operation}`,
      summaryEn:
        "ReferenceOnly deterministic inventory boundary with failure invariance.",
      summaryZh: "仅供资料使用的确定性库存边界，失败时保持不变。",
      manifestIds: [...constantIds(slotNames), ...inventoryFamilyIds],
      sourceRefs: [
        ...constantRefs(slotNames, "exact Demon King slot constant"),
        policySource(
          ["gb.policy.no-legal-candidate"],
          "inventory edge behavior policy; not observed parity",
        ),
      ],
      tags: ["demon-king", "inventory-operation", "project-policy"],
      evidenceQuality: "ProjectPolicy",
      mechanismQuality: "PolicyBoundary",
    }),
    operation_order: operationOrder,
    operation,
    selected_policy: selectedPolicy,
    rejected_alternatives: [
      "silently overwrite an existing item",
      "downgrade or remove an unrelated item",
    ],
    state_owner: "battle-local Demon King inventory",
    failure_invariance: true,
    affected_fixture_ids: inventoryFamilyIds,
    confidence: "Low",
    replacement_condition:
      "released logic or reproducible observation closes the exact transition",
  }),
);

for (const rows of [
  levelThresholds,
  adventureStrategies,
  candidatePools,
  candidatePolicies,
  inventorySlots,
  inventoryOperations,
]) rows.sort((left, right) => left.id.localeCompare(right.id, "en"));
const outputs = new Map([
  ["demon-level-thresholds.json", levelThresholds],
  ["demon-adventure-strategies.json", adventureStrategies],
  ["demon-candidate-pools.json", candidatePools],
  ["demon-candidate-policies.json", candidatePolicies],
  ["demon-inventory-slots.json", inventorySlots],
  ["demon-inventory-operations.json", inventoryOperations],
]);
await mkdir(outputRoot, { recursive: true });
for (const [file, value] of outputs) {
  const target = path.join(outputRoot, file);
  const encoded = `${JSON.stringify(canonicalValue(value), null, 2)}\n`;
  if (check) {
    const existing = await readFile(target, "utf8");
    if (existing !== encoded)
      throw new Error(`Demon King growth/strategy drift: ${target}`);
  } else {
    await writeFile(target, encoded);
  }
}

console.log(
  `Demon King growth/strategies ${check ? "verified" : "wrote"}: `
  + `${levelThresholds.length} threshold, `
  + `${adventureStrategies.length} strategies, ${candidatePools.length} pools, `
  + `${candidatePolicies.length} candidate policies, `
  + `${inventorySlots.length} slot/${inventoryOperations.length} operation rows`,
);
