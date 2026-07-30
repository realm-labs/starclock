#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/swarm-disaster-reference/import-audience-dice.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
function unique(values) {
  return values.length === new Set(values).size;
}

const normalizedRoot = "content-reference/swarm-disaster-v1";
const paths = json(`${normalizedRoot}/audience-paths.json`);
const dice = json(`${normalizedRoot}/audience-dice.json`);
const manifest = json(
  "content-manifests/swarm-disaster-v1/content-manifest.json",
);
assert(paths.length === 8, `expected 8 Audience Paths, found ${paths.length}`);
assert(dice.length === 8, `expected 8 Audience Dice, found ${dice.length}`);
assert(unique(paths.map(({ id }) => id)), "duplicate Audience Path ID");
assert(unique(dice.map(({ id }) => id)), "duplicate Audience Die ID");

const expectedPathIds = new Set(manifest.categories.audience_paths.records
  .map(({ id }) => `swarm-disaster.audience-path.${id}`));
const expectedDiceIds = new Set(manifest.categories.audience_dice.records
  .map(({ id }) => `swarm-disaster.audience-die.${id}`));
const pathIds = new Set(paths.map(({ id }) => id));
const diceIds = new Set(dice.map(({ id }) => id));
const inheritedPaths = new Set(json(
  "content-reference/standard-universe-v1/paths.json",
).map(({ id }) => id));
for (const row of paths) {
  assert(expectedPathIds.delete(row.id), `${row.id} manifest mismatch`);
  assert(inheritedPaths.has(row.path_id), `${row.id} path does not resolve`);
  assert(diceIds.has(row.audience_die_id), `${row.id} die does not resolve`);
  assert(row.initial_effects.length === 1
    && row.passive_effects.length === 1,
  `${row.id} effect-slot mismatch`);
  assert(row.evidence_quality === "ProjectPolicy"
    && row.coverage_state === "DataReady",
  `${row.id} evidence/coverage drift`);
}
for (const row of dice) {
  assert(expectedDiceIds.delete(row.id), `${row.id} manifest mismatch`);
  assert(inheritedPaths.has(row.path_id), `${row.id} path does not resolve`);
  assert(pathIds.has(row.audience_path_id),
    `${row.id} Audience Path does not resolve`);
  assert(row.face_ids.length >= 5 && row.face_ids.length <= 6,
    `${row.id} face count drift`);
  assert(unique(row.face_ids), `${row.id} duplicate face reference`);
  assert(
    row.roll_policy.candidate_order === "AuthoredSortThenStableFaceId"
      && row.roll_policy.control_rule_source === "G09-P1-B5",
    `${row.id} roll policy drift`,
  );
}
assert(expectedPathIds.size === 0, "Audience Path exact-once mismatch");
assert(expectedDiceIds.size === 0, "Audience Die exact-once mismatch");
assert(dice.flatMap(({ face_ids: values }) => values).length === 42,
  "Audience Die face-reference total drift");

console.log(
  "Swarm Disaster Audience Dice verification passed: 8 Paths, 8 dice, " +
  "42 exact face references and all inherited Path IDs resolve.",
);
