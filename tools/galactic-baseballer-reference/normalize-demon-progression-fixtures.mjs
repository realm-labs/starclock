#!/usr/bin/env node

import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const check = process.argv.includes("--check");
const root = path.resolve(".");
const fragmentRoot = path.join(
  root,
  "content-reference",
  "galactic-baseballer-v1",
  "fragments",
);
const profileId = "galactic-baseballer.demon-king.v3_3";
const rowRevision = "starclock.galactic-baseballer-row.v1";
const read = async (file) =>
  JSON.parse(await readFile(path.join(fragmentRoot, file), "utf8"));
const strategies = await read("demon-adventure-strategies.json");
const currencies = await read("demon-currencies.json");
const progression = await read("demon-progression.json");
const shops = await read("demon-shop-upgrades.json");

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
  evidenceQuality = "ProjectPolicy",
  mechanismQuality = "PolicyBoundary",
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

const reputation = currencies.find(({ id }) => id.endsWith("cosmic-reputation"));
const raccoonGold = currencies.find(({ id }) => id.endsWith("raccoon-gold"));
const treasureRows = progression.filter(({ progression_kind: kind }) =>
  kind === "TreasureGroup" || kind === "TreasureCandidatePool");
const rankRows = progression.filter(({ progression_kind: kind }) =>
  kind === "CosmicReputationRank");
const accessorySlot = shops.find(({ source_numeric_id: id }) =>
  id === "3113815");
const surpriseWindfall = strategies.find(({ source_numeric_id: id }) =>
  id === "3113703");
if ([reputation, raccoonGold, accessorySlot, surpriseWindfall].some(
  (value) => value === undefined,
)) throw new Error("Demon King progression fixture source missing");

function approximation({
  id,
  nameEn,
  nameZh,
  sourceRows,
  fieldPath,
  unavailableFact,
  knownReleasedFacts,
  selectedPolicy,
  rejectedAlternatives,
  rationale,
  fixtureIds,
  confidence,
  replacementCondition,
  evidenceQuality = "ProjectPolicy",
}) {
  return {
    ...envelope({
      id,
      kind: "Approximation",
      nameEn,
      nameZh,
      summaryEn:
        "Explicit replaceable boundary; this record is not an observed-parity claim.",
      summaryZh: "显式可替换边界；本记录不声称已观察到玩法一致性。",
      manifestIds: sourceRows.flatMap(({ manifest_record_ids: ids }) => ids),
      sourceRefs: sourceRows.flatMap(({ source_refs: refs }) => refs),
      tags: ["approximation", "demon-king", "replaceable"],
      evidenceQuality,
    }),
    field_path: fieldPath,
    unavailable_fact: unavailableFact,
    known_released_facts: knownReleasedFacts,
    selected_policy: selectedPolicy,
    rejected_alternatives: rejectedAlternatives,
    rationale,
    affected_fixture_ids: fixtureIds,
    confidence,
    replacement_condition: replacementCondition,
  };
}

const approximations = [
  approximation({
    id:
      "galactic-baseballer.demon-king.approximation.reputation-rank-costs",
    nameEn: "Cosmic Reputation rank costs",
    nameZh: "银河声望等级费用",
    sourceRows: [reputation, ...rankRows],
    fieldPath: "progression.cosmic_reputation.rank_costs",
    unavailableFact:
      "A released structured Offering table for all 20 Demon King rank costs is absent from the frozen structured family.",
    knownReleasedFacts:
      "OfferingType=8 and maximum=500000 are exact; pinned public revision 324128 lists 20 costs ending at cumulative 97500.",
    selectedPolicy:
      "Retain the pinned public revision's 3000x5, 4500x5 and 6000x10 costs as ReferenceOnly cross-check data; import no reward payload.",
    rejectedAlternatives: [
      "infer rank costs from Departure values",
      "derive costs from the value of account rewards",
      "drop reputation ranks because their reward payload is excluded",
    ],
    rationale:
      "The ranks are requested progression facts, but public community evidence must remain visibly lower-quality than structured rows.",
    fixtureIds: [
      "galactic-baseballer.demon-king.fixture.cosmic-reputation-rank-20",
    ],
    confidence: "Medium",
    replacementCondition:
      "A pinned released structured Offering definition provides every Demon King rank cost.",
    evidenceQuality: "ApproximateFromReleasedText",
  }),
  approximation({
    id: "galactic-baseballer.demon-king.approximation.treasure-selection",
    nameEn: "Demon King's Treasure selection",
    nameZh: "魔王宝藏选择",
    sourceRows: treasureRows,
    fieldPath: "progression.treasure.selection",
    unavailableFact:
      "The released tables expose ordered entry lists but not selection weights, group timing or duplicate-entry semantics.",
    knownReleasedFacts:
      "Five groups point to ten pools; every pool has ten authored entry positions and repeated IDs are preserved.",
    selectedPolicy:
      "Use labeled project integer sampling over eligible authored entry ordinals; repeated positions remain distinct and stable.",
    rejectedAlternatives: [
      "deduplicate repeated stable IDs before sampling",
      "treat authored array order as descending hidden weight",
      "use generic shuffle or collection iteration order",
    ],
    rationale:
      "Entry-position sampling preserves all source information while making the hidden distribution explicitly replaceable.",
    fixtureIds: [
      "galactic-baseballer.demon-king.fixture.treasure-selection",
    ],
    confidence: "Low",
    replacementCondition:
      "Released logic defines treasure group timing, weights, count and duplicate handling.",
  }),
  approximation({
    id:
      "galactic-baseballer.demon-king.approximation.shop-transaction-atomicity",
    nameEn: "Cosmic Store transaction atomicity",
    nameZh: "银河商店交易原子性",
    sourceRows: [accessorySlot],
    fieldPath: "shop.purchase.transaction",
    unavailableFact:
      "The exact rejection event ordering for insufficient balance, wrong current level and maximum level is not published.",
    knownReleasedFacts:
      "All 60 price steps, levels, shop types and effects are exact structured facts.",
    selectedPolicy:
      "Validate current level and balance before deducting currency; commit cost, level and effect as one ordered operation.",
    rejectedAlternatives: [
      "deduct currency before validating the current level",
      "partially apply the effect on an insufficient-balance rejection",
      "silently clamp purchases at maximum level",
    ],
    rationale:
      "Pre-validation preserves rejected-command byte identity and makes the transaction auditable.",
    fixtureIds: [
      "galactic-baseballer.demon-king.fixture.galactic-store-progression.success",
      "galactic-baseballer.demon-king.fixture.galactic-store-progression.rejected",
    ],
    confidence: "Low",
    replacementCondition:
      "Released transaction logic or reproducible observations close every rejection branch and event order.",
  }),
  approximation({
    id:
      "galactic-baseballer.demon-king.approximation.chest-probability-ordinal",
    nameEn: "Chest probability ordinal mapping",
    nameZh: "宝箱概率序号映射",
    sourceRows: [raccoonGold],
    fieldPath: "currencies.raccoon_gold.chest_probability_vector",
    unavailableFact:
      "The semantic names and application timing of the three probability ordinals are not exposed by the constant table.",
    knownReleasedFacts:
      "The exact vectors are 0.6,0.3,0.1 and -0.15,0,0.15.",
    selectedPolicy:
      "Retain both vectors by ordinal and do not assign chest-tier names or floating draws.",
    rejectedAlternatives: [
      "name the ordinals small, medium and large from array position",
      "convert the decimals into host floating probability draws",
      "reorder the vector by value",
    ],
    rationale:
      "Ordinal retention is lossless and avoids inventing a mapping not present in released structured facts.",
    fixtureIds: [
      "galactic-baseballer.demon-king.fixture.chest-probability-vector",
    ],
    confidence: "Low",
    replacementCondition:
      "Released logic binds every ordinal to a named chest state and sampling boundary.",
  }),
];

function approximationSource(id) {
  const record = approximations.find(({ id: recordId }) => recordId === id);
  if (record === undefined) throw new Error(`approximation missing: ${id}`);
  return {
    source_id: `source.goal16.approximation.${digest(record).slice(0, 16)}`,
    repository_or_url: "starclock",
    revision_or_access_date: rowRevision,
    game_version: "4.4",
    path_or_page:
      "content-reference/galactic-baseballer-v1/fragments/demon-progression-approximations.json",
    locator: id,
    sha256: digest(record),
    evidence_quality: record.evidence_quality,
    mechanism_quality: "PolicyBoundary",
    note: "explicit replaceable Goal 16 boundary",
    replacement_condition: record.replacement_condition,
  };
}

function rule({
  familyId,
  order,
  nameEn,
  nameZh,
  sourceRows,
  approximationIds,
  trigger,
  owner,
  preconditions,
  operations,
  fixtureIds,
}) {
  return {
    ...envelope({
      id: `galactic-baseballer.demon-king.rule.${familyId}`,
      kind: "MechanicRule",
      nameEn,
      nameZh,
      summaryEn:
        "ReferenceOnly rule binding exact progression facts and explicit deterministic boundaries.",
      summaryZh: "仅供资料使用的规则，绑定精确进度事实与显式确定性边界。",
      manifestIds: [
        familyId,
        ...sourceRows.flatMap(({ manifest_record_ids: ids }) => ids),
      ],
      sourceRefs: [
        ...sourceRows.flatMap(({ source_refs: refs }) => refs),
        ...approximationIds.map(approximationSource),
      ],
      tags: ["demon-king", familyId, "mechanic-rule"],
    }),
    family_id: familyId,
    rule_order: order,
    trigger_point: trigger,
    state_owner: owner,
    preconditions,
    ordered_operations: operations.map((operation, operationOrder) => ({
      operation_order: operationOrder,
      operation,
    })),
    fixture_ids: fixtureIds,
    runtime_executable: false,
  };
}

const rules = [
  rule({
    familyId: "adventure-strategy",
    order: 16,
    nameEn: "Demon King Adventure Strategy",
    nameZh: "魔王篇冒险策略",
    sourceRows: [surpriseWindfall],
    approximationIds: [],
    trigger: "accepted Adventure Strategy candidate",
    owner: "battle-local strategy set and profile Raccoon Gold balance",
    preconditions: {
      strategy_id: surpriseWindfall.id,
      not_already_selected: true,
    },
    operations: [
      "resolve the exact level-one strategy",
      "install its exact structural binding",
      "apply the authored 2000 Raccoon Gold parameter",
    ],
    fixtureIds: [
      "galactic-baseballer.demon-king.fixture.adventure-strategy",
    ],
  }),
  rule({
    familyId: "galactic-store-progression",
    order: 17,
    nameEn: "Demon King Cosmic Store progression",
    nameZh: "魔王篇银河商店进度",
    sourceRows: [accessorySlot, raccoonGold],
    approximationIds: [
      "galactic-baseballer.demon-king.approximation.shop-transaction-atomicity",
    ],
    trigger: "accepted persistent Cosmic Store purchase",
    owner: "profile-persistent Demon King progression",
    preconditions: {
      upgrade_id: accessorySlot.id,
      current_level: 0,
      required_balance: accessorySlot.cost,
    },
    operations: [
      "validate exact current level and maximum",
      "validate Raccoon Gold balance",
      "deduct exact price",
      "advance persistent upgrade level",
      "project the accessory-slot effect into future battle assembly",
    ],
    fixtureIds: [
      "galactic-baseballer.demon-king.fixture.galactic-store-progression.success",
      "galactic-baseballer.demon-king.fixture.galactic-store-progression.rejected",
    ],
  }),
];

function fixture({
  id,
  familyId,
  nameEn,
  nameZh,
  sourceRows,
  approximationIds = [],
  trigger,
  owner,
  preconditions,
  input,
  operations,
  expected,
}) {
  const refs = [
    ...sourceRows.flatMap(({ source_refs: sourceRefs }) => sourceRefs),
    ...approximationIds.map(approximationSource),
  ];
  return {
    ...envelope({
      id,
      kind: "SemanticReviewFixture",
      nameEn,
      nameZh,
      summaryEn: "Concrete ReferenceOnly progression review case.",
      summaryZh: "具体的仅供资料使用进度审查案例。",
      manifestIds: sourceRows.flatMap(({ manifest_record_ids: ids }) => ids),
      sourceRefs: refs,
      tags: ["demon-king", familyId, "semantic-review"],
    }),
    family_id: familyId,
    source_record_ids: sourceRows.map(({ id: sourceId }) => sourceId),
    trigger_point: trigger,
    state_owner: owner,
    preconditions,
    input,
    ordered_operations: operations.map((operation, operationOrder) => ({
      operation_order: operationOrder,
      operation,
    })),
    expected_facts: expected,
    evidence_refs: refs.map(({ source_id: sourceId }) => sourceId),
    runtime_executable: false,
  };
}

const firstTreasurePool = progression.find(({ id }) =>
  id.endsWith("treasure-pool.1"));
const rank20 = rankRows.find(({ rank }) => rank === 20);
const fixtures = [
  fixture({
    id: "galactic-baseballer.demon-king.fixture.adventure-strategy",
    familyId: "adventure-strategy",
    nameEn: "Surprise Windfall strategy",
    nameZh: "意外之财策略",
    sourceRows: [surpriseWindfall, raccoonGold],
    trigger: "accepted Adventure Strategy candidate",
    owner: "battle-local strategy set and profile Raccoon Gold balance",
    preconditions: { balance: 100, strategy_absent: true },
    input: { strategy_id: surpriseWindfall.id },
    operations: [
      "resolve exact strategy and parameter",
      "install strategy",
      "credit 2000 Raccoon Gold",
    ],
    expected: {
      strategy_installed: true,
      balance: 2100,
      currency_delta: 2000,
    },
  }),
  fixture({
    id:
      "galactic-baseballer.demon-king.fixture.galactic-store-progression.success",
    familyId: "galactic-store-progression",
    nameEn: "Cosmic Store accessory slot purchase",
    nameZh: "银河商店配饰槽位购买",
    sourceRows: [accessorySlot, raccoonGold],
    approximationIds: [
      "galactic-baseballer.demon-king.approximation.shop-transaction-atomicity",
    ],
    trigger: "accepted persistent Cosmic Store purchase",
    owner: "profile-persistent Demon King progression",
    preconditions: { balance: 6000, current_level: 0 },
    input: { shop_upgrade_id: accessorySlot.id },
    operations: rules[1].ordered_operations.map(({ operation }) => operation),
    expected: {
      accepted: true,
      balance: 0,
      current_level: 1,
      accessory_slot_delta_for_future_battles: 1,
    },
  }),
  fixture({
    id:
      "galactic-baseballer.demon-king.fixture.galactic-store-progression.rejected",
    familyId: "galactic-store-progression",
    nameEn: "Cosmic Store insufficient balance",
    nameZh: "银河商店余额不足",
    sourceRows: [accessorySlot, raccoonGold],
    approximationIds: [
      "galactic-baseballer.demon-king.approximation.shop-transaction-atomicity",
    ],
    trigger: "rejected persistent Cosmic Store purchase",
    owner: "profile-persistent Demon King progression",
    preconditions: { balance: 5999, current_level: 0 },
    input: { shop_upgrade_id: accessorySlot.id },
    operations: [
      "validate exact current level",
      "detect insufficient balance",
      "reject before deduction or effect",
    ],
    expected: {
      accepted: false,
      state_byte_identical: true,
      balance: 5999,
      current_level: 0,
    },
  }),
  fixture({
    id: "galactic-baseballer.demon-king.fixture.treasure-selection",
    familyId: "random-upgrade-candidates",
    nameEn: "Demon King's Treasure candidate ordinal",
    nameZh: "魔王宝藏候选序号",
    sourceRows: [firstTreasurePool],
    approximationIds: [
      "galactic-baseballer.demon-king.approximation.treasure-selection",
    ],
    trigger: "treasure candidate selection",
    owner: "battle-local treasure decision",
    preconditions: { source_box_item_id: "1" },
    input: { labeled_integer_sampled_ordinal: 0 },
    operations: [
      "preserve all authored entry positions",
      "sample an integer ordinal with the fixture label",
      "resolve the selected stable item ID",
    ],
    expected: {
      selected_ordinal: 0,
      selected_stable_id:
        firstTreasurePool.candidate_entries[0].stable_id,
    },
  }),
  fixture({
    id: "galactic-baseballer.demon-king.fixture.cosmic-reputation-rank-20",
    familyId: "galactic-store-progression",
    nameEn: "Cosmic Reputation rank 20 cost",
    nameZh: "银河声望等级 20 费用",
    sourceRows: [reputation, rank20],
    approximationIds: [
      "galactic-baseballer.demon-king.approximation.reputation-rank-costs",
    ],
    trigger: "reference progression audit",
    owner: "profile-persistent account progression",
    preconditions: { prior_rank: 19 },
    input: { inspect_rank: 20 },
    operations: [
      "read pinned rank row",
      "verify incremental and cumulative cost",
      "leave account reward payload excluded",
    ],
    expected: {
      incremental_cost: 6000,
      cumulative_cost: 97500,
      account_reward_payload_imported: false,
    },
  }),
  fixture({
    id: "galactic-baseballer.demon-king.fixture.chest-probability-vector",
    familyId: "random-upgrade-candidates",
    nameEn: "Chest probability ordinal preservation",
    nameZh: "宝箱概率序号保留",
    sourceRows: [raccoonGold],
    approximationIds: [
      "galactic-baseballer.demon-king.approximation.chest-probability-ordinal",
    ],
    trigger: "reference currency audit",
    owner: "immutable Raccoon Gold definition",
    preconditions: {},
    input: { inspect_vectors: true },
    operations: [
      "read exact vectors",
      "retain source ordinal",
      "assign no semantic tier name",
    ],
    expected: {
      probability_values: ["0.6", "0.3", "0.1"],
      step_values: ["-0.15", "0", "0.15"],
      semantic_ordinal_names_assigned: false,
    },
  }),
];

for (const rows of [approximations, rules, fixtures])
  rows.sort((left, right) => left.id.localeCompare(right.id, "en"));
const outputs = new Map([
  ["demon-progression-approximations.json", approximations],
  ["demon-progression-mechanic-rules.json", rules],
  ["demon-progression-review-fixtures.json", fixtures],
]);
await mkdir(fragmentRoot, { recursive: true });
for (const [file, value] of outputs) {
  const target = path.join(fragmentRoot, file);
  const encoded = `${JSON.stringify(value, null, 2)}\n`;
  if (check) {
    const existing = await readFile(target, "utf8");
    if (existing !== encoded)
      throw new Error(`Demon King progression fixture drift: ${target}`);
  } else {
    await writeFile(target, encoded);
  }
}

console.log(
  `Demon King progression fixtures ${check ? "verified" : "wrote"}: `
  + `${approximations.length} boundaries, ${rules.length} rules, `
  + `${fixtures.length} fixtures`,
);
