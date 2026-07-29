#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/swarm-disaster-reference/import-communing-trail.mjs", "--check"],
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

const nodes = read("communing-trail-nodes.json");
const prerequisites = read("communing-trail-prerequisites.json");
const effects = read("communing-trail-effects.json");
const dimensions = new Set(read("communing-dimensions.json")
  .map(({ id }) => id));
assert(nodes.length === 63, "Communing Trail node count drift");
assert(prerequisites.length === 56, "prerequisite edge count drift");
assert(effects.length === 63, "Communing Trail effect count drift");
for (const rows of [nodes, prerequisites, effects])
  assert(unique(rows.map(({ id }) => id)), "duplicate Communing Trail ID");

const expected = new Set(manifest.categories.communing_trail_nodes.records
  .map(({ id }) => `swarm-disaster.communing-trail.${id}`));
const nodeIds = new Set(nodes.map(({ id }) => id));
const prerequisiteIds = new Set(prerequisites.map(({ id }) => id));
const effectIds = new Set(effects.map(({ id }) => id));
for (const node of nodes) {
  assert(expected.delete(node.id), `${node.id} manifest mismatch`);
  assert(dimensions.has(node.dimension_id),
    `${node.id} dimension does not resolve`);
  assert(node.prerequisite_ids.every((id) => prerequisiteIds.has(id)),
    `${node.id} prerequisite does not resolve`);
  assert(node.effect_ids.every((id) => effectIds.has(id)),
    `${node.id} effect does not resolve`);
}
assert(expected.size === 0, "Communing Trail exact-once mismatch");
for (const prerequisite of prerequisites)
  assert(nodeIds.has(prerequisite.node_id)
    && nodeIds.has(prerequisite.required_node_id),
  `${prerequisite.id} endpoint does not resolve`);
for (const effect of effects)
  assert(nodeIds.has(effect.node_id)
    && ["Activity", "Battle", "ActivityAndBattle"].includes(effect.domain)
    && effect.ordered_operations.length === 1
    && effect.battle_projection.enabled === (effect.domain !== "Activity"),
  `${effect.id} domain/projection drift`);
for (const dimensionId of dimensions) {
  const dimensionNodes = nodes.filter(({ dimension_id: id }) =>
    id === dimensionId);
  assert(dimensionNodes.length === 9,
    `${dimensionId} does not contain nine nodes`);
}

console.log(
  "Swarm Disaster Communing Trail verification passed: 63 nodes, " +
  "56 policy predecessor edges and 63 typed effects.",
);
