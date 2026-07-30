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
const approximationRegister = JSON.parse(await readFile(path.join(
  root,
  "content-manifests",
  "galactic-baseballer-v1",
  "approximation-register.json",
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
function localPolicySource(policyIds, note) {
  const records = approximationRegister.records.filter(({ id }) =>
    policyIds.includes(id));
  if (records.length !== policyIds.length)
    throw new Error(`missing approximation policy: ${policyIds.join(",")}`);
  return {
    source_id: `source.goal16.policy.${digest(records).slice(0, 16)}`,
    repository_or_url: "starclock",
    revision_or_access_date:
      "starclock.galactic-baseballer-approximation-register.v1",
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
    ownership: "Departure",
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

const constantPath = "ExcelOutput/EvolveBuildConstValueCommon.json";
const cardPath = "ExcelOutput/EvolveBuildCardConfig.json";
const mazePath = "ExcelOutput/EvolveBuildMazeBuff.json";
const expProgramPath =
  "Config/ConfigAbility/BattleEvent/EvolveBuild_00_ExpAndLevel.json";
const cardProgramPath =
  "Config/ConfigAbility/BattleEvent/EvolveBuild_03_Card.json";
const constants = await readSource(constantPath);
const cards = await readSource(cardPath);
const mazeBuffs = await readSource(mazePath);
const expProgram = await readSource(expProgramPath);
const cardProgram = await readSource(cardProgramPath);
const chs = await readSource("TextMap/TextMapCHS.json");
const en = await readSource("TextMap/TextMapEN.json");
const expProgramManifest = manifestRecord("config_programs", expProgramPath);
const cardProgramManifest = manifestRecord("config_programs", cardProgramPath);

function constant(name) {
  const index = constants.findIndex(({ ConstValueName }) =>
    ConstValueName === name);
  if (index === -1) throw new Error(`constant missing: ${name}`);
  const manifestId =
    `${profileId}:EvolveBuildConstValueCommon:${String(index).padStart(4, "0")}`;
  return {
    name,
    value: canonicalValue(constants[index].Value),
    manifestId,
    manifest: manifestRecord("mode_constants", manifestId),
  };
}
function constantRefs(names, note) {
  return names.map((name) => {
    const item = constant(name);
    return structuredSource(item.manifest, "ExactRelationship", note);
  });
}
function constantIds(names) {
  return names.map((name) => constant(name).manifestId);
}

const thresholdConstantNames = [
  "EvolveBuild_Exp_Multiplier_Wave_N",
  "EvolveBuild_Exp_LvN",
  "EvolveBuild_Exp_Basic_Normal1",
  "EvolveBuild_Exp_Basic_Normal2",
  "EvolveBuild_Exp_Basic_Elite",
  "EvolveBuild_Exp_Basic_Boss",
  "EvolveBuild_Exp_Basic_Special",
];
const basicAbility = expProgram.AbilityList.find(({ Name }) =>
  Name === "StageAbility_VS_Common_Basic");
if (basicAbility === undefined) throw new Error("basic exp program missing");
const expForLevelAssignments = [];
function collectExpAssignments(value) {
  if (Array.isArray(value)) {
    for (const child of value) collectExpAssignments(child);
  } else if (value !== null && typeof value === "object") {
    if (value.DynamicKey?.Value === "expForLevel"
      && value.FixedValue?.Value !== undefined) {
      expForLevelAssignments.push(value.FixedValue.Value);
    }
    if (value.DynamicKey?.Value === "expForLevel"
      && value.Value?.FixedValue?.Value !== undefined) {
      expForLevelAssignments.push(value.Value.FixedValue.Value);
    }
    for (const child of Object.values(value)) collectExpAssignments(child);
  }
}
collectExpAssignments(basicAbility);
const uniqueThresholds = [...new Set(expForLevelAssignments)];
if (uniqueThresholds.length !== 1 || uniqueThresholds[0] !== 40)
  throw new Error("expForLevel threshold drift");
const levelThresholds = [{
  ...envelope({
    id: "galactic-baseballer.departure.level-threshold.shared",
    kind: "LevelThreshold",
    nameEn: "Departure team level threshold",
    nameZh: "启程篇队伍等级阈值",
    summaryEn:
      "The released base program initializes the team-level experience threshold to 40 and the constants define exact enemy/wave experience parameters.",
    summaryZh:
      "已发布基础程序把队伍升级经验阈值初始化为 40，常量定义精确敌人/波次经验参数。",
    manifestIds: [
      ...constantIds(thresholdConstantNames),
      expProgramManifest.id,
    ],
    sourceRefs: [
      ...constantRefs(
        thresholdConstantNames,
        "exact released experience/scaling constant",
      ),
      structuredSource(
        expProgramManifest,
        "ExactProgram",
        "whole-file program digest; exact expForLevel initialization reviewed structurally",
      ),
    ],
    tags: ["departure", "experience", "level-threshold"],
  }),
  level_scope: "all authored team levels until the released program changes the threshold",
  experience_threshold: "40",
  wave_multiplier: String(
    constant("EvolveBuild_Exp_Multiplier_Wave_N").value.DoubleValue,
  ),
  level_scaling_parameters: constant("EvolveBuild_Exp_LvN").value.ArrayValue
    .map((entry) => String(entry.DoubleValue ?? entry.IntValue)),
  experience_awards: {
    normal_1: constant("EvolveBuild_Exp_Basic_Normal1").value.IntValue,
    normal_2: constant("EvolveBuild_Exp_Basic_Normal2").value.IntValue,
    elite: constant("EvolveBuild_Exp_Basic_Elite").value.IntValue,
    boss: constant("EvolveBuild_Exp_Basic_Boss").value.IntValue,
    special: constant("EvolveBuild_Exp_Basic_Special").value.IntValue,
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
const candidatePools = cards.map((card, index) => {
  const maze = mazeIndex.get(`${card.ID}:1`);
  if (maze === undefined) throw new Error(`card MazeBuff missing: ${card.ID}`);
  const cardManifestId =
    `${profileId}:EvolveBuildCardConfig:${String(index).padStart(4, "0")}`;
  const mazeManifestId =
    `${profileId}:EvolveBuildMazeBuff:${String(maze.index).padStart(4, "0")}`;
  const cardManifest = manifestRecord("upgrade_cards", cardManifestId);
  const mazeManifest = manifestRecord("accessory_levels", mazeManifestId);
  const nameHash = String(maze.row.BuffName.Hash);
  if (typeof en[nameHash] !== "string" || typeof chs[nameHash] !== "string")
    throw new Error(`card localization missing: ${card.ID}`);
  return {
    ...envelope({
      id: `galactic-baseballer.departure.candidate.strategy.${card.ID}`,
      kind: "CandidatePool",
      nameEn: en[nameHash],
      nameZh: chs[nameHash],
      summaryEn:
        "Departure Adventure Strategy candidate with exact type, parameters and battle-program binding.",
      summaryZh: "启程篇冒险策略候选，含精确类型、参数与战斗程序绑定。",
      manifestIds: [cardManifestId, mazeManifestId, cardProgramManifest.id],
      sourceRefs: [
        structuredSource(
          cardManifest,
          "ExactRelationship",
          "exact released card identity and selection metadata",
        ),
        structuredSource(
          mazeManifest,
          "ExactProgram",
          "exact level-one MazeBuff parameters and binding",
        ),
        structuredSource(
          cardProgramManifest,
          "ExactProgram",
          "whole-file card program digest; normalized row retains structural summary only",
        ),
      ],
      tags: ["adventure-strategy", "candidate", "departure"],
    }),
    source_numeric_id: String(card.ID),
    source_level_id: String(card.LvID),
    strategy_type: card.Type ?? "Unspecified",
    source_name_hash: nameHash,
    card_parameters: card.ParamList.map(({ Value }) => String(Value)),
    selectable_period_ids: card.CardSelectablePeriod.map(String),
    maze_buff_id: String(maze.row.ID),
    maze_buff_parameters: maze.row.ParamList.map(({ Value }) => String(Value)),
    program_summary: programSummary(cardProgram, maze.row.InBattleBindingKey),
  };
});

const weightNames = Array.from(
  { length: 11 },
  (_, index) => `EvolveBuild_Weight${index + 1}`,
);
const decisionConstantNames = [
  "EvolveBuild_LevelUp_AbilityName",
  "EvolveBuild_MazeBuffID_ReRoll",
  "EvolveBuild_MazeBuffID_LostCount",
  "EvolveBuild_Reset_UnlockQuest",
  "EvolveBuild_Remove_UnlockQuest",
  "EvolveBuild_Reset_Num",
  "EvolveBuild_Remove_Num",
  "EvolveBuild_Skip_UnlockQuest",
  "EvolveBuild_Card_Reset_Num",
];
const candidateManifestIds = [
  ...constantIds([...weightNames, ...decisionConstantNames]),
  ...candidatePools.flatMap(({ manifest_record_ids: ids }) => ids),
];
const candidatePolicies = [
  {
    id: "galactic-baseballer.departure.candidate-policy.source-parameters",
    nameEn: "Departure candidate source parameters",
    nameZh: "启程篇候选源参数",
    evidenceQuality: "ExactStructured",
    mechanismQuality: "ExactRelationship",
    extra: {
      weight_vector: weightNames.map((name, ordinal) => ({
        ordinal,
        source_constant: name,
        weight: constant(name).value.IntValue,
      })),
      level_up_ability_names:
        constant("EvolveBuild_LevelUp_AbilityName").value.ArrayValue
          .map(({ StringValue }) => StringValue),
      reroll_maze_buff_id:
        String(constant("EvolveBuild_MazeBuffID_ReRoll").value.IntValue),
      exclusion_maze_buff_id:
        String(constant("EvolveBuild_MazeBuffID_LostCount").value.IntValue),
      reroll_count: constant("EvolveBuild_Reset_Num").value.IntValue,
      exclusion_count: constant("EvolveBuild_Remove_Num").value.IntValue,
      card_reroll_count: constant("EvolveBuild_Card_Reset_Num").value.IntValue,
      reroll_unlock_quest_id:
        String(constant("EvolveBuild_Reset_UnlockQuest").value.IntValue),
      exclusion_unlock_quest_id:
        String(constant("EvolveBuild_Remove_UnlockQuest").value.IntValue),
      skip_unlock_quest_id:
        String(constant("EvolveBuild_Skip_UnlockQuest").value.IntValue),
      weight_mapping_status:
        "Unspecified: exact vector retained; no card/category mapping inferred from ordinal alone",
    },
  },
  {
    id: "galactic-baseballer.departure.candidate-policy.deterministic-fallback",
    nameEn: "Departure deterministic candidate fallback",
    nameZh: "启程篇确定性候选回退",
    evidenceQuality: "ProjectPolicy",
    mechanismQuality: "PolicyBoundary",
    extra: {
      rng_label:
        "galactic-baseballer/{profile-id}/{activity-instance-id}/upgrade-candidate/{decision-ordinal}",
      candidate_order: "stable Starclock ID ascending before integer sampling",
      refresh_exclusion:
        "exclude the immediately displayed IDs once, then use the complete legal pool if exclusion empties it",
      no_legal_candidate:
        "emit explicit no-candidate outcome; consume no inventory resource; continue at the next declared boundary",
      rejected_alternatives: [
        "unbounded repeated random draws",
        "localized-name or hash-map iteration order",
        "silent replacement/downgrade of a maximum-level item",
      ],
      affected_fixture_ids: [
        "random-upgrade-candidates",
        "no-legal-candidate-failure-invariance",
      ],
      confidence: "Low",
      replacement_condition:
        "released structured logic or reproducible observation defines complete weights, offer order, refresh memory and every empty-pool branch",
    },
  },
].map((policy) => ({
  ...envelope({
    id: policy.id,
    kind: "CandidatePolicy",
    nameEn: policy.nameEn,
    nameZh: policy.nameZh,
    summaryEn:
      "Reference-only candidate generation, decision-resource and deterministic fallback contract.",
    summaryZh: "仅供资料使用的候选生成、决策资源与确定性回退契约。",
    manifestIds: candidateManifestIds,
    sourceRefs: policy.evidenceQuality === "ExactStructured"
      ? constantRefs(
        [...weightNames, ...decisionConstantNames],
        "exact candidate/decision constant",
      )
      : [localPolicySource([
        "gb.policy.upgrade-candidate-weight",
        "gb.policy.upgrade-candidate-order",
        "gb.policy.no-legal-candidate",
        "gb.policy.refresh-exclusion",
      ], "explicit deterministic fallback; not an observed parity claim")],
    tags: ["candidate-policy", "departure"],
    evidenceQuality: policy.evidenceQuality,
    mechanismQuality: policy.mechanismQuality,
  }),
  decision_order: policy.evidenceQuality === "ExactStructured" ? 0 : 1,
  ...policy.extra,
}));

const slotConstantNames = [
  "EvolveBuild_Slot_Weapon_Unlocked",
  "EvolveBuild_Slot_Weapon_Total",
  "EvolveBuild_Slot_Accessory_Unlocked",
  "EvolveBuild_Slot_Accessory_Total",
  "EvolveBuild_OriginStage_Weapon_Slot",
  "EvolveBuild_OriginStage_Accessory_Slot",
];
function slotPolicy(kind, scope, unlockedName, totalName) {
  return {
    ...envelope({
      id: `galactic-baseballer.departure.inventory-slot.${scope}.${kind.toLowerCase()}`,
      kind: "InventorySlotPolicy",
      nameEn: `Departure ${scope} ${kind} slots`,
      nameZh: `启程篇${scope === "Standard" ? "普通" : "起源关"}${kind === "Weapon" ? "武器" : "配饰"}槽位`,
      summaryEn: `Exact ${scope.toLowerCase()} ${kind.toLowerCase()} slot capacity.`,
      summaryZh: `精确${scope === "Standard" ? "普通" : "起源关"}${kind === "Weapon" ? "武器" : "配饰"}槽位容量。`,
      manifestIds: constantIds([unlockedName, totalName]),
      sourceRefs: constantRefs(
        [unlockedName, totalName],
        "exact released slot-capacity constant",
      ),
      tags: ["departure", kind.toLowerCase(), "slot"],
    }),
    slot_kind: kind,
    scope,
    initially_unlocked: constant(unlockedName).value.IntValue,
    total_capacity: constant(totalName).value.IntValue,
  };
}
const inventorySlots = [
  slotPolicy(
    "Weapon",
    "Standard",
    "EvolveBuild_Slot_Weapon_Unlocked",
    "EvolveBuild_Slot_Weapon_Total",
  ),
  slotPolicy(
    "Accessory",
    "Standard",
    "EvolveBuild_Slot_Accessory_Unlocked",
    "EvolveBuild_Slot_Accessory_Total",
  ),
  slotPolicy(
    "Weapon",
    "OriginStage",
    "EvolveBuild_OriginStage_Weapon_Slot",
    "EvolveBuild_OriginStage_Weapon_Slot",
  ),
  slotPolicy(
    "Accessory",
    "OriginStage",
    "EvolveBuild_OriginStage_Accessory_Slot",
    "EvolveBuild_OriginStage_Accessory_Slot",
  ),
];

const inventoryFamilyIds = [
  "weapon-acquisition-duplicate-upgrade",
  "accessory-acquisition-duplicate-upgrade",
  "slot-capacity-expansion-replacement",
  "no-legal-candidate-failure-invariance",
];
const inventoryManifestIds = [
  ...constantIds(slotConstantNames),
  ...inventoryFamilyIds,
];
const operationSpecs = [
  ["acquire-new", "AcquireNew", "insert a new item only into an unlocked empty slot"],
  ["upgrade-duplicate", "UpgradeDuplicate", "increment the owned item's level by one when below its exact maximum"],
  ["reject-max-duplicate", "RejectMaximumDuplicate", "exclude or reject a duplicate already at maximum level without mutation"],
  ["reject-full-new", "RejectFullInventoryNew", "reject a new item when no unlocked empty slot or explicit replacement decision exists"],
  ["expand-slot", "ExpandSlot", "increase unlocked capacity by one without exceeding the exact total capacity"],
];
const inventoryOperations = operationSpecs.map(
  ([suffix, operation, selectedPolicy], operationOrder) => ({
    ...envelope({
      id: `galactic-baseballer.departure.inventory-operation.${suffix}`,
      kind: "InventoryOperation",
      nameEn: `Departure inventory operation: ${operation}`,
      nameZh: `启程篇库存操作：${operation}`,
      summaryEn:
        "Deterministic ReferenceOnly inventory boundary preserving exact authored capacities and maximum levels.",
      summaryZh: "仅供资料使用的确定性库存边界，保留精确作者容量与最高等级。",
      manifestIds: inventoryManifestIds,
      sourceRefs: [
        ...constantRefs(slotConstantNames, "exact released slot constant"),
        localPolicySource(
          ["gb.policy.no-legal-candidate"],
          "inventory edge behavior selected explicitly; not an observed parity claim",
        ),
      ],
      tags: ["departure", "inventory-operation", "project-policy"],
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
    state_owner: "battle-local Galactic Baseballer inventory",
    failure_invariance: true,
    affected_fixture_ids: inventoryFamilyIds,
    confidence: "Low",
    replacement_condition:
      "released structured program evidence or reproducible observation proves the exact full/max/duplicate transition",
  }),
);

for (const rows of [
  levelThresholds,
  candidatePools,
  candidatePolicies,
  inventorySlots,
  inventoryOperations,
]) rows.sort((left, right) => left.id.localeCompare(right.id, "en"));
const outputs = new Map([
  ["level-thresholds.json", levelThresholds],
  ["candidate-pools.json", candidatePools],
  ["candidate-policies.json", candidatePolicies],
  ["inventory-slots.json", inventorySlots],
  ["inventory-operations.json", inventoryOperations],
]);
await mkdir(outputRoot, { recursive: true });
for (const [file, value] of outputs) {
  const target = path.join(outputRoot, file);
  const encoded = `${JSON.stringify(canonicalValue(value), null, 2)}\n`;
  if (check) {
    const existing = await readFile(target, "utf8");
    if (existing !== encoded)
      throw new Error(`Departure growth drift: ${target}`);
  } else {
    await writeFile(target, encoded);
  }
}
console.log(
  `Departure growth ${check ? "verified" : "wrote"}: `
  + `${levelThresholds.length} threshold, ${candidatePools.length} strategies, `
  + `${candidatePolicies.length} candidate policies, `
  + `${inventorySlots.length} slot policies, `
  + `${inventoryOperations.length} inventory operations`,
);
