#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/swarm-disaster-reference/import-paths.mjs", "--check"],
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

const paths = read("paths.json");
const resonances = read("resonances.json");
const boosts = read("path-boosts.json");
const interplays = read("resonance-interplays.json");
const bonuses = read("bonuses.json");
assert(paths.length === 8, "Path count drift");
assert(resonances.length === 32, "Resonance count drift");
assert(boosts.length === 8, "Path boost count drift");
assert(interplays.length === 16, "Interplay count drift");
assert(bonuses.length === 6, "Trailblaze Bonus count drift");
for (const rows of [paths, resonances, boosts, interplays, bonuses])
  assert(unique(rows.map(({ id }) => id)), "duplicate Path-system ID");

exactOnce(paths, "paths", ({ shared_path_id: id }) => id);
exactOnce(resonances, "resonances", ({ shared_resonance_id: id }) => id);
exactOnce(boosts, "path_boosts", ({ source_id: id }) => id);
exactOnce(interplays, "resonance_interplays", ({ source_id: id }) => id);
exactOnce(bonuses, "trailblaze_bonuses", ({ source_id: id }) => id);

const pathIds = new Set(paths.map(({ shared_path_id: id }) => id));
const resonanceIds = new Set(resonances.map(({
  shared_resonance_id: id,
}) => id));
assert(!pathIds.has("universe.path.erudition"),
  "Erudition leaked into Swarm Disaster");
const propagation = paths.find(({ shared_path_id: id }) =>
  id === "universe.path.propagation");
assert(propagation?.propagation_unlock.is_propagation
  && propagation.propagation_unlock.required_unlock_id
    === "swarm-disaster.pathstrider-unlock.1000008",
"Propagation unlock binding drift");
for (const pathRow of paths) {
  assert(pathRow.selectable && pathRow.ownership === "Shared"
    && pathRow.formation_ids.length === 3
    && resonanceIds.has(pathRow.resonance_id)
    && pathRow.formation_ids.every((id) => resonanceIds.has(id)),
  `${pathRow.id} Path binding drift`);
  const owned = resonances.filter(({ path_id: id }) =>
    id === pathRow.shared_path_id);
  assert(owned.length === 4
    && owned.filter(({ kind }) => kind === "Resonance").length === 1
    && owned.filter(({ kind }) => kind === "Formation").length === 3,
  `${pathRow.id} Resonance closure drift`);
}
for (const boost of boosts)
  assert(pathIds.has(boost.path_id)
    && boost.effect_program.operation === "AddMazeBuff"
    && /^StageAbility_6412[0-7]0$/u.test(
      boost.effect_program.stage_ability,
    )
    && boost.application_boundary === "AfterPathSelectionAtRunStart",
  `${boost.id} effect binding drift`);
for (const interplay of interplays)
  assert(pathIds.has(interplay.main_path_id)
    && pathIds.has(interplay.sub_path_id)
    && interplay.main_path_id !== interplay.sub_path_id
    && interplay.thresholds.main_path_blessings === "3"
    && interplay.thresholds.sub_path_blessings === "3"
    && interplay.thresholds.counting_policy
      === "DistinctOwnedBlessingIdentity"
    && interplay.effect_program.parameters.every(({ value }) => value !== ""),
  `${interplay.id} threshold/effect drift`);
for (const pathId of pathIds)
  assert(interplays.filter(({ main_path_id: id }) => id === pathId).length
    === 2,
  `${pathId} does not have two Interplays`);

assert(JSON.stringify(bonuses.map(({ source_id: id }) => id))
  === JSON.stringify(["101", "102", "103", "104", "105", "106"]),
"Trailblaze Bonus identity drift");
assert(bonuses.find(({ source_id: id }) => id === "101")
  .effect_program.operations[0].value === "150",
"Fragmented Universe value drift");
assert(bonuses.find(({ source_id: id }) => id === "104")
  .effect_program.operations.map(({ operation }) => operation).join(",")
    === "SpendCosmicFragments,GrantRandomCurios",
"Orderly Universe atomic program drift");
assert(bonuses.find(({ source_id: id }) => id === "105")
  .effect_program.operations.map(({ count }) => count).join(",") === "2,1",
"Hungry Universe grant counts drift");
assert(bonuses.find(({ source_id: id }) => id === "106")
  .effect_program.operations.map(({ value, count }) => value ?? count)
    .join(",") === "-2,3",
"Bloodthirsty Universe program drift");

console.log(
  "Swarm Disaster Path-system verification passed: eight Paths, 32 shared " +
  "Resonances/Formations, eight boosts, 16 Interplays and six bonuses.",
);
