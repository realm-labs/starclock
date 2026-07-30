#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/swarm-disaster-reference/import-blessings.mjs", "--check"],
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

const blessings = read("blessings.json");
const levels = read("blessing-levels.json");
const memberships = read("pool-membership.json");
const paths = read("paths.json");
const resonances = read("resonances.json");
assert(blessings.length === 144, "Blessing count drift");
assert(levels.length === 288, "Blessing-level count drift");
assert(memberships.length === 184, "pool-membership count drift");
for (const rows of [blessings, levels, memberships])
  assert(unique(rows.map(({ id }) => id)), "duplicate Blessing-pool ID");

exactOnce(
  blessings,
  "blessings",
  ({ shared_blessing_id: id }) => id,
);
exactOnce(
  levels,
  "blessing_levels",
  ({ shared_blessing_level_id: id }) => id,
);
const pathIds = new Set(paths.map(({ shared_path_id: id }) => id));
const blessingIds = new Set(blessings.map(({ id }) => id));
const levelIds = new Set(levels.map(({ id }) => id));
for (const blessing of blessings)
  assert(pathIds.has(blessing.path_id)
    && blessing.ownership === "Shared"
    && blessing.level_ids.length === 2
    && blessing.level_ids.every((id) => levelIds.has(id))
    && blessing.pool_rules.base_integer_weight === "1",
  `${blessing.id} shared pool closure drift`);
for (const level of levels)
  assert(blessingIds.has(level.blessing_id)
    && ["1", "2"].includes(level.level)
    && level.parameter_values.every(({ value }) => value !== "")
    && level.effect_program.modifier_name.length > 0,
  `${level.id} authored level drift`);
for (const pathId of pathIds) {
  const owned = blessings.filter(({ path_id: id }) => id === pathId);
  assert(owned.length === 18, `${pathId} Blessing count drift`);
  const counts = ["1", "2", "3"].map((rarity) =>
    owned.filter((row) => row.rarity === rarity).length);
  assert(JSON.stringify(counts) === JSON.stringify([8, 7, 3]),
    `${pathId} rarity distribution drift`);
}
for (const blessing of blessings)
  assert(levels.filter(({ blessing_id: id }) => id === blessing.id).length
    === 2,
  `${blessing.id} authored-level closure drift`);

assert(memberships.filter(({ member_kind: kind }) =>
  kind === "Path").length === 8, "Path membership count drift");
assert(memberships.filter(({ member_kind: kind }) =>
  kind === "Resonance" || kind === "Formation").length === 32,
"Resonance membership count drift");
assert(memberships.filter(({ member_kind: kind }) =>
  kind === "Blessing").length === 144,
"Blessing membership count drift");
const memberIds = new Set([
  ...pathIds,
  ...resonances.map(({ shared_resonance_id: id }) => id),
  ...blessings.map(({ shared_blessing_id: id }) => id),
]);
for (const member of memberships)
  assert(memberIds.has(member.member_id)
    && member.eligibility.rule.length > 0
    && member.weight_policy.selection.length > 0,
  `${member.id} membership binding drift`);
assert(![...memberIds].some((id) => id.includes("erudition")
  || id.startsWith("universe.blessing.6128")),
"Erudition leaked into Swarm Blessing pool");

console.log(
  "Swarm Disaster Blessing pool verification passed: 144 shared Blessings, " +
  "288 exact levels and 184 explicit pool memberships.",
);
