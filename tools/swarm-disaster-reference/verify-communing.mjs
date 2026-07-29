#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/swarm-disaster-reference/import-communing.mjs", "--check"],
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

const choices = read("communing-choices.json");
const cabinets = read("pathstrider-cabinets.json");
const dimensions = read("communing-dimensions.json");
const adjustments = read("communing-point-adjustments.json");
assert(choices.length === 21, "Communing choice count drift");
assert(cabinets.length === 31, "cabinet count drift");
assert(dimensions.length === 7, "dimension count drift");
assert(adjustments.length === 55, "point-adjustment count drift");
for (const rows of [choices, cabinets, dimensions, adjustments])
  assert(unique(rows.map(({ id }) => id)), "duplicate Communing ID");

const exact = (category, prefix, rows) => {
  const expected = new Set(manifest.categories[category].records
    .map(({ id }) => `${prefix}${id}`));
  for (const row of rows)
    assert(expected.delete(row.id), `${row.id} manifest mismatch`);
  assert(expected.size === 0, `${category} exact-once mismatch`);
};
exact("communing_choices", "swarm-disaster.communing-choice.", choices);
exact(
  "pathstrider_cabinets",
  "swarm-disaster.pathstrider-cabinet.",
  cabinets,
);
exact(
  "communing_dimensions",
  "swarm-disaster.communing-dimension.",
  dimensions,
);
const cabinetIds = new Set(cabinets.map(({ id }) => id));
const dimensionIds = new Set(dimensions.map(({ id }) => id));
for (const choice of choices)
  assert(choice.point_deltas.length === 0
    && choice.ordered_operations[0].operation === "IncrementAeonChoiceCounter",
  `${choice.id} invented a point delta`);
for (const cabinet of cabinets) {
  assert(cabinet.prerequisite_ids.every((id) => cabinetIds.has(id)),
    `${cabinet.id} prerequisite does not resolve`);
  assert(cabinet.unlocks_cabinet_ids.every((id) => cabinetIds.has(id)),
    `${cabinet.id} outgoing unlock does not resolve`);
  assert(cabinet.point_deltas.every(({ dimension_id: id }) =>
    dimensionIds.has(id)), `${cabinet.id} dimension does not resolve`);
}
for (const dimension of dimensions)
  assert(dimension.max_points === "20"
    && dimension.carry_policy === "PersistentAcrossRuns",
  `${dimension.id} maximum/carry drift`);
for (const adjustment of adjustments)
  assert(dimensionIds.has(adjustment.dimension_id)
    && adjustment.delta !== "0"
    && adjustment.clamp_policy === "ClampToDimensionMaximumAfterOperation",
  `${adjustment.id} operation drift`);

console.log(
  "Swarm Disaster Communing verification passed: 21 choices, 31 cabinets, " +
  "7 dimensions and 55 exact point adjustments.",
);
