#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const root = path.resolve(".");
const sourceCache = option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/galactic-baseballer-source");
const manifestPath = path.join(
  root,
  "content-manifests",
  "galactic-baseballer-v1",
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

execFileSync(process.execPath, [
  path.join("tools", "galactic-baseballer-reference", "manifest.mjs"),
  "--check",
  "--source-cache",
  sourceCache,
], {
  cwd: root,
  stdio: "inherit",
});

const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
assert(
  manifest.schema_revision
    === "starclock.galactic-baseballer-content-manifest.v1",
  "manifest schema revision drift",
);
assert(
  manifest.goal_id === "galactic-baseballer-reference-v1",
  "manifest Goal ID drift",
);
assert(manifest.snapshot.game_version === "4.4", "game version drift");
assert(
  JSON.stringify(manifest.profiles) === JSON.stringify([
    "galactic-baseballer.departure.v2_2",
    "galactic-baseballer.demon-king.v3_3",
  ]),
  "profile set/order drift",
);

const expectedCategories = {
  profiles: 2,
  profile_stages: 13,
  stage_periods: 113,
  weapon_collections: 87,
  weapon_levels: 379,
  weapon_types: 5,
  accessory_levels: 563,
  synthesis_materials: 27,
  upgrade_cards: 67,
  upgrade_card_types: 4,
  offer_box_groups: 5,
  offer_box_items: 10,
  mode_constants: 146,
  shop_progression: 30,
  content_tags: 8,
  tutorial_entries: 40,
  enemy_collection_locators: 6,
  reward_locators: 21,
  presentation_locators: 4,
  config_programs: 35,
  shared_stage_configs: 22,
  infinite_stage_groups: 22,
  infinite_waves: 74,
  infinite_monster_groups: 74,
  enemy_variants: 88,
  enemy_templates: 70,
  enemy_skills: 287,
  enemy_statuses: 10,
  semantic_fixture_families: 20,
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
  assert(
    category.records.length === expectedCount,
    `category record count mismatch: ${categoryId}`,
  );
  const ids = category.records.map(({ id }) => id);
  assert(
    ids.every((id, index) => index === 0 || ids[index - 1] < id),
    `category records are not uniquely sorted: ${categoryId}`,
  );
  for (const record of category.records) {
    assert(
      typeof record.source_path === "string" && record.source_path.length > 0,
      `missing source path: ${categoryId}/${record.id}`,
    );
    assert(
      typeof record.row_locator === "string" && record.row_locator.length > 0,
      `missing row locator: ${categoryId}/${record.id}`,
    );
    assert(
      /^[0-9a-f]{64}$/u.test(record.evidence_sha256),
      `invalid evidence digest: ${categoryId}/${record.id}`,
    );
    assert(
      ["ExactStructured", "ProjectPolicy"].includes(record.evidence_quality),
      `invalid evidence quality: ${categoryId}/${record.id}`,
    );
    assert(
      ["Departure", "DemonKing", "SharedBase", "Shared"]
        .includes(record.ownership),
      `invalid ownership: ${categoryId}/${record.id}`,
    );
    assert(
      typeof record.selector === "string" && record.selector.length > 0,
      `missing selector: ${categoryId}/${record.id}`,
    );
  }
}

const expectedRecords = Object.values(expectedCategories)
  .reduce((sum, count) => sum + count, 0);
assert(manifest.counts.records === expectedRecords, "record count drift");
assert(
  manifest.counts.data_ready_required === expectedRecords - 25,
  "DataReady denominator drift",
);
assert(manifest.counts.evidence_only === 25, "EvidenceOnly count drift");
assert(
  manifest.reconciliation.profile_table_rows.departure === 697
    && manifest.reconciliation.profile_table_rows.demon_king === 831,
  "profile source row count drift",
);
assert(
  manifest.reconciliation.dedicated_tables === 29
    && manifest.reconciliation.config_programs === 35,
  "dedicated source denominator drift",
);
assert(
  manifest.reconciliation.explicit_shared_stage_id === 4140116,
  "explicit shared stage proof drift",
);
assert(
  JSON.stringify(manifest.reconciliation.recursive_counts) === JSON.stringify({
    stage_configs: 22,
    infinite_stage_groups: 22,
    infinite_waves: 74,
    infinite_monster_groups: 74,
    enemy_variants: 88,
    enemy_templates: 70,
    enemy_skills: 287,
    enemy_statuses: 10,
  }),
  "recursive shared count drift",
);
assert(
  manifest.source_augmentation.records.length === 3,
  "P0-B3 source augmentation drift",
);
assert(
  manifest.exclusions_and_replacement_boundaries.legacy_stage_references.length
    === 3,
  "legacy stage boundary drift",
);
assert(
  manifest.exclusions_and_replacement_boundaries.unresolved_enemy_effect_ids
    .length === 9,
  "unresolved enemy effect boundary drift",
);
assert(
  manifest.counts.replacement_boundaries === 12,
  "replacement boundary count drift",
);
assert(
  manifest.categories.reward_locators.records.every(
    ({ data_status: status }) => status === "EvidenceOnly",
  )
    && manifest.categories.presentation_locators.records.every(
      ({ data_status: status }) => status === "EvidenceOnly",
    ),
  "account/presentation exclusion drift",
);

console.log(
  `Galactic Baseballer manifest verified: ${expectedRecords} exact obligations`,
);
