#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/gold-and-gears-reference/import-services.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);

const services = json("content-reference/gold-and-gears-v1/services.json");
const adventures = json(
  "content-reference/gold-and-gears-v1/adventure-outcomes.json",
);
const beacons = json("content-reference/gold-and-gears-v1/beacons.json");
const rooms = json("content-reference/gold-and-gears-v1/rooms.json");
const standard = json("content-reference/standard-universe-v1/services.json")
  .filter(({ id }) => !id.includes(".trailblaze-bonus."));
const manifest = json(
  "content-manifests/gold-and-gears-v1/content-manifest.json",
);

assert(services.length === 15, "shared-service count drift");
assert(adventures.length === 8, "Adventure-outcome count drift");
assert(beacons.length === 6, "Beacon closure drift");
const allRows = [...services, ...adventures];
assert(unique(allRows.map(({ id }) => id)), "duplicate service-pack ID");
for (const row of allRows) {
  assert(row.schema_revision === "starclock.gold-and-gears-row.v1"
    && row.ownership === "Shared"
    && row.coverage_state === "DataReady"
    && row.evidence_quality === "ExactStructured",
  `${row.id} common envelope drift`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof row[field] === "string" && row[field].trim() !== "",
      `${row.id} has empty ${field}`);
  assert(JSON.stringify(row.tags) === JSON.stringify([...row.tags].sort()),
    `${row.id} tags are not canonical`);
  assert(Array.isArray(row.source_refs) && row.source_refs.length > 0,
    `${row.id} has no provenance`);
  for (const source of row.source_refs)
    assert(/^[0-9a-f]{64}$/u.test(source.sha256)
      && source.source_id && source.repository && source.revision
      && source.path && source.locator && source.access_date
      && source.evidence_quality,
    `${row.id} source ref drift`);
}

const standardById = new Map(standard.map((row) => [row.id, row]));
for (const service of services) {
  const inherited = standardById.get(service.id);
  assert(inherited !== undefined
    && service.source_mode_owner === "Standard"
    && service.service_kind === inherited.kind
    && JSON.stringify(service.parameters) === JSON.stringify(inherited.parameters)
    && JSON.stringify(service.inherited_rule_ids)
      === JSON.stringify(inherited.rule_ids)
    && service.selection_policy.unresolved_pool_behavior === "FailClosed",
  `${service.id} inherited service drift`);
}
assert(services.filter(({ service_kind: kind }) => kind === "BlessingShop")
  .length === 5,
"Blessing-shop count drift");
assert(services.filter(({ service_kind: kind }) => kind === "CurioShop")
  .length === 4,
"Curio-shop count drift");
for (const shop of services.filter(({ service_kind: kind }) =>
  kind === "BlessingShop"))
  assert(JSON.stringify(shop.gold_gears_offer_rule.inventory)
    === JSON.stringify([
      { rarity: "1", unit_cost: "100", base_stock: "3" },
      { rarity: "2", unit_cost: "200", base_stock: "2" },
      { rarity: "3", unit_cost: "300", base_stock: "1" },
    ]),
  `${shop.id} Blessing inventory drift`);
for (const shop of services.filter(({ service_kind: kind }) =>
  kind === "CurioShop"))
  assert(JSON.stringify(shop.gold_gears_offer_rule.inventory)
    === JSON.stringify([
      { slot: "1", unit_cost: "150" },
      { slot: "2", unit_cost: "150" },
      { slot: "3", unit_cost: "300" },
    ]),
  `${shop.id} Curio inventory drift`);
assert(services.find(({ service_kind: kind }) => kind === "Reviver")
  .gold_gears_offer_rule.unit_cost === "80",
"Gold and Gears Reviver cost drift");

const roomIds = new Set(rooms.map(({ id }) => id));
const typeCounts = Object.fromEntries(Object.entries(
  Object.groupBy(adventures, ({ adventure_type: type }) => type),
).map(([type, rows]) => [type, rows.length]));
assert(JSON.stringify(typeCounts) === JSON.stringify({
  RogueCaptureMonster: 3,
  RogueDestroyProp: 3,
  RogueEscapeLaser: 1,
  RogueTurntable: 1,
}), "Adventure type distribution drift");
const expectedThresholds = new Map([
  ["RogueCaptureMonster", ["2000", "3600"]],
  ["RogueDestroyProp", ["15", "30"]],
  ["RogueTurntable", ["2", "3"]],
  ["RogueEscapeLaser", ["4", "6"]],
]);
for (const adventure of adventures) {
  assert(roomIds.has(adventure.room_id)
    && adventure.rewards_are_cumulative
    && adventure.reward_tiers.length === 3
    && adventure.downloader_service_id === "universe.service.downloader"
    && adventure.reward_selection_policy.policy_id
      === "adventure-reward-selection-v1"
    && adventure.reward_selection_policy.unresolved_pool_behavior
      === "FailClosed",
  `${adventure.id} room/reward closure drift`);
  assert(JSON.stringify(adventure.objective_thresholds
    .map(({ minimum_value: value }) => value))
    === JSON.stringify(expectedThresholds.get(adventure.adventure_type)),
  `${adventure.id} objective threshold drift`);
  assert(adventure.reward_tiers[0].minimum_value === "100"
    && adventure.reward_tiers[0].maximum_value === "150"
    && adventure.reward_tiers[1].rarity === "2"
    && adventure.reward_tiers[2].operation === "OfferCurioChoice",
  `${adventure.id} tier reward drift`);
}

for (const [category, rows] of [
  ["shared_services", services],
  ["adventure_outcomes", adventures],
]) {
  const identity = category === "shared_services" ? "id" : "source_id";
  const actual = rows.map((row) => row[identity]).sort();
  const required = manifest.categories[category].records
    .map(({ id }) => id).sort();
  assert(JSON.stringify(actual) === JSON.stringify(required),
    `${category} manifest exact-once drift`);
}

console.log(
  "Gold and Gears services verified (15 shared services; 6 prior beacons; " +
  "8 Adventure rooms across 4 challenge types and 3 cumulative tiers).",
);

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function unique(values) {
  return new Set(values).size === values.length;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
