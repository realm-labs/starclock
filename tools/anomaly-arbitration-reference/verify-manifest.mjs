#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const root = path.resolve(".");
const sourceCache = option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/content-reference");
const fallbackSourceCache = option("--fallback-source-cache")
  ?? process.env.STARCLOCK_FALLBACK_SOURCE_CACHE;
const manifestPath = path.join(
  root,
  "content-manifests",
  "anomaly-arbitration-v1",
  "content-manifest.json",
);

function option(name) {
  const index = args.indexOf(name);
  if (index === -1) return undefined;
  if (args[index + 1] === undefined || args[index + 1].startsWith("--"))
    throw new Error(`${name} requires a path`);
  return args[index + 1];
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const generatorArgs = [
  path.join("tools", "anomaly-arbitration-reference", "manifest.mjs"),
  "--check",
  "--source-cache",
  sourceCache,
];
if (fallbackSourceCache !== undefined)
  generatorArgs.push("--fallback-source-cache", fallbackSourceCache);
execFileSync(process.execPath, generatorArgs, {
  cwd: root,
  stdio: "inherit",
});

const encoded = await readFile(manifestPath);
const manifest = JSON.parse(encoded);
assert(
  manifest.schema_revision
    === "starclock.anomaly-arbitration-content-manifest.v1",
  "manifest schema revision drift",
);
assert(
  manifest.goal_id === "anomaly-arbitration-reference-v1",
  "manifest goal ID drift",
);
assert(manifest.snapshot.game_version === "4.4", "game version drift");
assert(
  manifest.snapshot.source_revision
    === "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  "structured source revision drift",
);
assert(
  manifest.active_period_selector.group_id === 8
    && manifest.active_period_selector.name_zh === "尘世卷中"
    && manifest.active_period_selector.name_en === "Enwreathed by the World",
  "active Version 4.4 period selector drift",
);
assert(
  manifest.active_period_selector.released_cross_checks.length === 2
    && manifest.active_period_selector.released_cross_checks.every(
      ({ evidence_quality: quality }) => quality === "Observed",
    ),
  "released active-period cross-check drift",
);

const expectedCategories = {
  profiles: 1,
  active_periods: 1,
  stage_definitions: 4,
  boss_difficulty_definitions: 1,
  stage_configs: 5,
  battle_targets: 7,
  stage_traits: 8,
  quadrant_options: 3,
  battle_events: 3,
  mode_constants: 14,
  terminal_outcomes: 4,
  participant_policies: 4,
  record_progress_lifecycles: 6,
  king_state_transitions: 6,
  clock_rules: 9,
  objective_aggregations: 5,
  semantic_fixture_families: 18,
  enemy_variants: 27,
  enemy_templates: 26,
  enemy_skills: 115,
  enemy_statuses: 52,
  config_programs: 73,
  blessings: 0,
  curios: 0,
  occurrences: 0,
  gameplay_services: 0,
  currencies: 0,
  random_content_pools: 0,
};
assert(
  JSON.stringify(Object.keys(manifest.categories))
    === JSON.stringify(Object.keys(expectedCategories)),
  "category set/order drift",
);
for (const [categoryId, expectedCount] of Object.entries(expectedCategories)) {
  const category = manifest.categories[categoryId];
  assert(category.id === categoryId, `category ID mismatch: ${categoryId}`);
  assert(category.count === expectedCount, `category count drift: ${categoryId}`);
  assert(category.records.length === expectedCount,
    `category record count mismatch: ${categoryId}`);
  const ids = category.records.map(({ id }) => id);
  assert(
    ids.every((id, index) => index === 0 || ids[index - 1] < id),
    `category records are not uniquely sorted: ${categoryId}`,
  );
  for (const record of category.records) {
    assert(typeof record.source_path === "string" && record.source_path.length > 0,
      `missing source path: ${categoryId}/${record.id}`);
    assert(typeof record.row_locator === "string" && record.row_locator.length > 0,
      `missing row locator: ${categoryId}/${record.id}`);
    assert(/^[0-9a-f]{64}$/u.test(record.evidence_sha256),
      `invalid evidence digest: ${categoryId}/${record.id}`);
    assert(["ExactStructured", "ProjectPolicy"].includes(record.evidence_quality),
      `invalid evidence quality: ${categoryId}/${record.id}`);
    assert(["AnomalyArbitration", "Shared"].includes(record.ownership),
      `invalid active ownership: ${categoryId}/${record.id}`);
    assert(typeof record.selector === "string" && record.selector.length > 0,
      `missing selector proof: ${categoryId}/${record.id}`);
  }
}

assert(
  manifest.counts.categories === 28
    && manifest.counts.records === 392
    && manifest.counts.ownership.AnomalyArbitration === 76
    && manifest.counts.ownership.Shared === 316,
  "active exact-once count drift",
);
assert(
  Object.values(manifest.categories).reduce(
    (sum, { count }) => sum + count,
    0,
  ) === manifest.counts.records,
  "category denominator does not reconcile",
);
assert(
  manifest.exclusions.historical_period_count === 77
    && manifest.exclusions.account_reward_count === 13
    && manifest.exclusions.excluded_constant_count === 15
    && manifest.exclusions.presentation_count === 1
    && manifest.counts.exclusions === 106,
  "historical/account/presentation exclusion count drift",
);
assert(
  manifest.exclusions.historical_period_rows.every((record) =>
    record.ownership === "ExcludedHistoricalPeriod"
    && record.reachability === "Excluded"),
  "historical rows escaped the exclusion boundary",
);
assert(
  manifest.exclusions.empty_reward_override_table.count === 0,
  "empty reward override proof drift",
);

const zeroFamilies = [
  "blessings",
  "curios",
  "occurrences",
  "gameplay_services",
  "currencies",
  "random_content_pools",
];
assert(manifest.counts.zero_categories === zeroFamilies.length,
  "zero-category denominator drift");
for (const family of zeroFamilies) {
  const proof = manifest.zero_pool_proofs[family];
  assert(proof.count === 0, `nonzero audited pool: ${family}`);
  assert(/^[0-9a-f]{64}$/u.test(proof.selector_closure_sha256),
    `invalid zero selector digest: ${family}`);
  assert(proof.replacement_condition.includes("released active selector"),
    `zero proof lacks replacement condition: ${family}`);
}

assert(
  manifest.counter_groups.profile_period_entry.required === 20
    && manifest.counter_groups.stages_and_difficulties.required === 10
    && manifest.counter_groups.targets_traits_quadrants_events.required === 21
    && manifest.counter_groups.encounters_and_enemies.required === 293
    && manifest.counter_groups.audited_empty_pools.required === 0
    && manifest.counter_groups.participant_and_records.required === 10
    && manifest.counter_groups.king_and_clocks.required === 15
    && manifest.counter_groups.objective_aggregation.required === 12
    && manifest.counter_groups.semantic_fixture_families.required === 18,
  "counter-group denominator drift",
);
assert(
  manifest.categories.config_programs.records.every(({ id }) =>
    !id.includes("Rogue")),
  "unselected Rogue program leaked into the active config closure",
);
assert(
  manifest.categories.stage_configs.records.every(
    ({ row_locator: locator }) =>
      ["30508011", "30508012", "30508013", "30508021", "30508022"]
        .some((stageId) => locator.includes(stageId)),
  ),
  "historical StageConfig row leaked into the active denominator",
);

console.log(
  "Anomaly Arbitration manifest verified: " +
  `${manifest.counts.records} active records, ` +
  `${manifest.counts.exclusions} exclusions, ` +
  `${manifest.counts.zero_categories} exact-zero pools; SHA-256 ` +
  `${createHash("sha256").update(encoded).digest("hex")}.`,
);
