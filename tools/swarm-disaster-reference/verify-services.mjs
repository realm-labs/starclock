#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/swarm-disaster-reference/import-services.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const read = (name) => JSON.parse(fs.readFileSync(path.join(
  root,
  "content-reference/swarm-disaster-v1",
  name,
), "utf8"));
const manifest = JSON.parse(fs.readFileSync(path.join(
  root,
  "content-manifests/swarm-disaster-v1/content-manifest.json",
), "utf8"));
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
function unique(values) {
  return values.length === new Set(values).size;
}
function exactOnce(rows, category, identity) {
  const actual = rows.map(identity).sort();
  const required = manifest.categories[category].records
    .map(({ id }) => id).sort();
  assert(JSON.stringify(actual) === JSON.stringify(required),
    `${category} exact-once mismatch`);
}

const services = read("services.json");
const currencies = read("currencies.json");
const adventures = read("adventure-outcomes.json");
const rules = read("service-rules.json");
const beacons = read("beacons.json");
assert(services.length === 15, "shared-service count drift");
assert(currencies.length === 1, "currency count drift");
assert(adventures.length === 6, "Adventure-outcome count drift");
assert(rules.length === 19, "service-rule count drift");
for (const rows of [services, currencies, adventures, rules])
  assert(unique(rows.map(({ id }) => id)), "duplicate service-pack ID");
exactOnce(
  services,
  "shared_services",
  ({ shared_service_id: id }) => id,
);
exactOnce(
  adventures,
  "adventure_outcomes",
  ({ source_id: id }) => id,
);

const serviceIds = new Set([
  ...services.map(({ id }) => id),
  ...beacons.map(({ id }) => id),
]);
for (const service of services)
  assert(service.ownership === "Shared"
    && service.eligibility.unresolved_offer_behavior === "FailClosed"
    && service.price_policy.insufficient_resource === "RejectWithoutMutation",
  `${service.id} service policy drift`);
assert(currencies[0].resource_id === "universe.currency.cosmic-fragments"
  && currencies[0].initial_value === "50"
  && currencies[0].cap_policy.scope === "ActivityRun",
"Cosmic Fragments lifecycle drift");
for (const adventure of adventures)
  assert(["RogueCaptureMonster", "RogueDestroyProp"].includes(
    adventure.adventure_type,
  )
    && adventure.offered_result.accepted_values.join(",")
      === "Tier1,Tier2,Tier3"
    && adventure.offered_result.input_simulation === "Excluded"
    && adventure.reward_program.unresolved_payload === "RejectWithoutMutation",
  `${adventure.id} abstract outcome drift`);
for (const rule of rules)
  assert(serviceIds.has(rule.service_id)
    && rule.conditions.length > 0
    && rule.ordered_operations.length > 0,
  `${rule.id} service binding drift`);
assert(services.filter(({ service_kind: kind }) =>
  kind === "BlessingShop").length === 5,
"Blessing-shop count drift");
assert(services.filter(({ service_kind: kind }) =>
  kind === "CurioShop").length === 4,
"Curio-shop count drift");
assert(rules.filter(({ service_id: id }) =>
  id.startsWith("swarm-disaster.beacon.")).length === 4,
"Beacon service-rule closure drift");
const typeCounts = Object.fromEntries(Object.entries(
  Object.groupBy(adventures, ({ adventure_type: type }) => type),
).map(([type, rows]) => [type, rows.length]));
assert(JSON.stringify(typeCounts) === JSON.stringify({
  RogueCaptureMonster: 3,
  RogueDestroyProp: 3,
}), "Adventure type distribution drift");

console.log(
  "Swarm Disaster service verification passed: 15 shared services, one " +
  "currency, four beacon rules and six abstract Adventure outcomes.",
);
