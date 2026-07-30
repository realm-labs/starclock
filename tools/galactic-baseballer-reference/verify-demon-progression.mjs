#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const sourceCache = option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/galactic-baseballer-source");
const fragmentRoot = path.join(
  root,
  "content-reference",
  "galactic-baseballer-v1",
  "fragments",
);

function option(name) {
  const index = process.argv.indexOf(name);
  if (index === -1) return undefined;
  const value = process.argv[index + 1];
  if (value === undefined || value.startsWith("--"))
    throw new Error(`${name} requires a path`);
  return value;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

execFileSync(process.execPath, [
  path.join(
    "tools",
    "galactic-baseballer-reference",
    "cache-public-revisions.mjs",
  ),
  "--cache",
  path.join(sourceCache, "public-revisions"),
  "--offline",
], { cwd: root, stdio: "inherit" });
for (const script of [
  "normalize-demon-growth-strategies.mjs",
  "normalize-demon-progression.mjs",
]) {
  execFileSync(process.execPath, [
    path.join("tools", "galactic-baseballer-reference", script),
    "--check",
    "--source-cache",
    sourceCache,
  ], { cwd: root, stdio: "inherit" });
}
execFileSync(process.execPath, [
  path.join(
    "tools",
    "galactic-baseballer-reference",
    "normalize-demon-progression-fixtures.mjs",
  ),
  "--check",
], { cwd: root, stdio: "inherit" });

const read = async (file) =>
  JSON.parse(await readFile(path.join(fragmentRoot, file), "utf8"));
const thresholds = await read("demon-level-thresholds.json");
const strategies = await read("demon-adventure-strategies.json");
const pools = await read("demon-candidate-pools.json");
const policies = await read("demon-candidate-policies.json");
const slots = await read("demon-inventory-slots.json");
const operations = await read("demon-inventory-operations.json");
const currencies = await read("demon-currencies.json");
const progression = await read("demon-progression.json");
const shops = await read("demon-shop-upgrades.json");
const unlocks = await read("demon-unlocks.json");
const approximations = await read("demon-progression-approximations.json");
const rules = await read("demon-progression-mechanic-rules.json");
const fixtures = await read("demon-progression-review-fixtures.json");

assert(thresholds.length === 1, "Demon King threshold count drift");
assert(
  thresholds[0].experience_threshold === "40"
    && thresholds[0].wave_multiplier === "0.27"
    && thresholds[0].level_scaling_parameters.join(",") === "0.14,1,12"
    && [
      thresholds[0].experience_awards.normal_1,
      thresholds[0].experience_awards.normal_2,
      thresholds[0].experience_awards.elite,
      thresholds[0].experience_awards.boss,
    ].join(",") === "2,4,8,0"
    && thresholds[0].experience_awards.special
      === "UnspecifiedNoDemonKingConstant",
  "Demon King experience boundary drift",
);

assert(strategies.length === 56, "Adventure Strategy count drift");
const strategyTypeCounts = Object.fromEntries(
  ["General", "Growth", "Power", "DemonKing"].map((type) => [
    type,
    strategies.filter(({ source_type: rowType }) => rowType === type).length,
  ]),
);
assert(
  JSON.stringify(strategyTypeCounts) === JSON.stringify({
    General: 15,
    Growth: 22,
    Power: 18,
    DemonKing: 1,
  }),
  "strategy type partition drift",
);
assert(
  strategies.every(({ maximum_level: maximum, program_summary: summary }) =>
    maximum === 1
      && summary.ability_names.length >= 1
      && summary.operation_types.length >= 1
      && /^[0-9a-f]{64}$/u.test(summary.program_fragment_sha256)),
  "strategy level/program closure drift",
);
assert(
  strategies.find(({ source_numeric_id: id }) => id === "3113799")
    .public_unlock_condition
    === "Reach the Demon King Challenge phase in the Demon King's Den",
  "Ultimate Bat strategy unlock cross-check drift",
);

assert(pools.length === 4, "strategy candidate pool count drift");
assert(
  pools.map(({ candidate_ids: ids }) => ids.length).sort((a, b) => a - b)
    .join(",") === "1,15,18,22",
  "strategy candidate pool size drift",
);
assert(
  new Set(pools.flatMap(({ candidate_ids: ids }) => ids)).size === 56,
  "strategy pool exact-once membership drift",
);
assert(policies.length === 2, "candidate policy count drift");
const sourcePolicy = policies.find(({ decision_order: order }) => order === 0);
const fallbackPolicy = policies.find(({ decision_order: order }) => order === 1);
assert(
  sourcePolicy.weight_vector.map(({ weight }) => weight).join(",")
    === "18,6,3,3,7,6,2,0,2,0,7,7,7,7,7"
    && sourcePolicy.reroll_count === 3
    && sourcePolicy.exclusion_count === 2
    && sourcePolicy.strategy_reroll_count === 1
    && sourcePolicy.weight_mapping_status.startsWith("Unspecified"),
  "Demon King candidate source parameters drift",
);
assert(
  fallbackPolicy.evidence_quality === "ProjectPolicy"
    && fallbackPolicy.rng_label.includes("strategy-candidate")
    && fallbackPolicy.rejected_alternatives.length >= 2,
  "Demon King candidate fallback drift",
);

assert(slots.length === 4, "Demon King slot policy count drift");
const slotVector = slots.map((row) =>
  `${row.scope}:${row.slot_kind}:${row.initially_unlocked}/${row.total_capacity}`)
  .sort();
assert(
  slotVector.join(",") === [
    "OriginStage:Accessory:4/4",
    "OriginStage:Weapon:3/3",
    "Standard:Accessory:4/6",
    "Standard:Weapon:4/5",
  ].join(","),
  "Demon King slot capacity drift",
);
assert(
  operations.length === 5
    && operations.every(({ failure_invariance: invariant }) => invariant),
  "Demon King inventory operation drift",
);

assert(currencies.length === 2, "Demon King currency count drift");
const gold = currencies.find(({ id }) => id.endsWith("raccoon-gold"));
const reputation = currencies.find(({ id }) => id.endsWith("cosmic-reputation"));
assert(
  gold.source_item_id === "281027"
    && gold.maximum_balance === 500000
    && Object.values(gold.enemy_income).join(",") === "5,5,20,200"
    && gold.chest_probability_vector.map(({ value }) => value).join(",")
      === "0.6,0.3,0.1"
    && reputation.offering_type === 8
    && reputation.rank_count === 20
    && reputation.account_reward_payloads_imported === false,
  "Demon King currency boundary drift",
);

assert(progression.length === 35, "Demon King progression row count drift");
const ranks = progression.filter(({ progression_kind: kind }) =>
  kind === "CosmicReputationRank");
const groups = progression.filter(({ progression_kind: kind }) =>
  kind === "TreasureGroup");
const treasurePools = progression.filter(({ progression_kind: kind }) =>
  kind === "TreasureCandidatePool");
assert(
  ranks.length === 20 && groups.length === 5 && treasurePools.length === 10,
  "progression family counts drift",
);
assert(
  ranks.find(({ rank }) => rank === 20).cumulative_cost === 97500
    && ranks.every(({ account_reward_disposition: disposition }) =>
      disposition === "EvidenceOnlyNotImported"),
  "Cosmic Reputation rank closure drift",
);
assert(
  treasurePools.reduce(
    (count, { candidate_entries: entries }) => count + entries.length,
    0,
  ) === 100,
  "treasure candidate entry denominator drift",
);
assert(
  groups.every(({ box_item_pool_ids: ids }) =>
    ids.every(({ source_box_item_id: sourceId }) =>
      treasurePools.some(({ source_box_item_id: poolId }) =>
        poolId === sourceId))),
  "treasure group reference closure drift",
);

assert(shops.length === 60, "Cosmic Store level count drift");
assert(
  new Set(shops.map(({ source_numeric_id: id }) => id)).size === 16,
  "Cosmic Store definition count drift",
);
assert(
  shops.filter(({ shop_type: type }) => type === "AddMazeBuff").length === 54
    && shops.filter(({ shop_type: type }) =>
      type === "InitWeaponLevel").length === 5
    && shops.filter(({ shop_type: type }) =>
      type === "AddAccessorySlot").length === 1
    && shops.reduce((sum, { cost }) => sum + cost, 0) === 75600,
  "Cosmic Store type/price closure drift",
);
const shopManifestIds = new Set(shops.flatMap(
  ({ manifest_record_ids: ids }) => ids.filter((id) =>
    id.includes("EvoBdSCShopConfig")),
));
const shopMazeManifestIds = new Set(shops.flatMap(
  ({ manifest_record_ids: ids }) => ids.filter((id) =>
    id.includes("EvoBdSCMazeBuff")),
));
const tagManifestIds = new Set(shops.flatMap(
  ({ manifest_record_ids: ids }) => ids.filter((id) =>
    id.includes("EvoBdSCTagConfig")),
));
assert(
  shopManifestIds.size === 16
    && shopMazeManifestIds.size === 54
    && tagManifestIds.size === 4,
  "Cosmic Store source exact closure drift",
);

assert(unlocks.length === 30, "Demon King unlock row count drift");
assert(
  unlocks.filter(({ unlock_kind: kind }) =>
    kind === "TutorialLocator").length === 20
    && unlocks.filter(({ disposition }) =>
      disposition === "EvidenceOnlyAccountReward").length === 1
    && unlocks.filter(({ disposition }) =>
      disposition === "EvidenceOnlyPresentationLocator").every(
        ({ runtime_effect: effect }) => effect === false,
      ),
  "unlock/tutorial disposition drift",
);
const tutorialManifestIds = new Set(unlocks.flatMap(
  ({ manifest_record_ids: ids }) => ids.filter((id) =>
    id.includes("EvoBdSCTutorial")),
));
assert(tutorialManifestIds.size === 20, "tutorial exact closure drift");

assert(approximations.length === 4, "progression approximation count drift");
for (const row of approximations) {
  assert(
    row.unavailable_fact.length > 20
      && row.known_released_facts.length > 20
      && row.selected_policy.length > 20
      && row.rejected_alternatives.length >= 2
      && row.rationale.length > 20
      && row.affected_fixture_ids.length >= 1
      && ["Low", "Medium", "High"].includes(row.confidence)
      && row.replacement_condition.length > 20,
    `progression approximation contract drift: ${row.id}`,
  );
}
assert(
  rules.length === 2
    && rules.map(({ family_id: id }) => id).sort().join(",")
      === "adventure-strategy,galactic-store-progression",
  "progression rule families drift",
);
assert(fixtures.length === 6, "progression fixture count drift");
for (const rule of rules) {
  assert(
    rule.fixture_ids.every((id) =>
      fixtures.some(({ id: fixtureId }) => fixtureId === id)),
    `progression rule fixture link drift: ${rule.id}`,
  );
}
const rejectedStore = fixtures.find(({ id }) =>
  id.endsWith("galactic-store-progression.rejected"));
assert(
  rejectedStore.expected_facts.state_byte_identical === true
    && rejectedStore.expected_facts.balance === 5999,
  "store rejection invariance drift",
);

const arsenalWeaponLevels = await read("demon-weapon-levels.json");
const arsenalAccessoryLevels = await read("demon-accessory-levels.json");
const arsenalMazeIds = new Set(
  [...arsenalWeaponLevels, ...arsenalAccessoryLevels].flatMap(
    ({ manifest_record_ids: ids }) => ids.filter((id) =>
      id.includes("EvoBdSCMazeBuff")),
  ),
);
const strategyMazeIds = new Set(strategies.flatMap(
  ({ manifest_record_ids: ids }) => ids.filter((id) =>
    id.includes("EvoBdSCMazeBuff")),
));
assert(
  arsenalMazeIds.size === 198
    && strategyMazeIds.size === 56
    && shopMazeManifestIds.size === 54
    && new Set([
      ...arsenalMazeIds,
      ...strategyMazeIds,
      ...shopMazeManifestIds,
    ]).size === 308,
  "Demon King MazeBuff responsibility closure drift",
);

console.log(
  "Demon King progression verified: 56 strategies, 4 pools, 2 currencies, "
  + "20 reputation ranks, 5/10 treasure groups/pools, 60 shop levels, "
  + "30 unlocks, 4 approximation boundaries and 6 fixtures",
);
