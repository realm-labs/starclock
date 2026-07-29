#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/swarm-disaster-reference/import-dice-faces.mjs", "--check"],
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

const faces = read("dice-faces.json");
const rarities = read("dice-rarities.json");
const targets = read("dice-target-rules.json");
const controls = read("dice-roll-controls.json");
assert(faces.length === 42, "dice-face count drift");
assert(rarities.length === 3, "dice-rarity count drift");
assert(targets.length === 42, "dice-target count drift");
assert(controls.length === 4, "dice-control count drift");
for (const rows of [faces, rarities, targets, controls])
  assert(unique(rows.map(({ id }) => id)), "duplicate dice partition ID");

const expectedFaces = new Set(manifest.categories.dice_faces.records
  .map(({ id }) => `swarm-disaster.dice-face.${id}`));
const expectedRarities = new Set(manifest.categories.dice_rarities.records
  .map(({ id }) => `swarm-disaster.dice-rarity.${id}`));
const rarityIds = new Set(rarities.map(({ id }) => id));
const targetIds = new Set(targets.map(({ id }) => id));
for (const face of faces) {
  assert(expectedFaces.delete(face.id), `${face.id} manifest mismatch`);
  assert(rarityIds.has(face.rarity_id), `${face.id} rarity does not resolve`);
  assert(targetIds.has(face.target_rule_id), `${face.id} target does not resolve`);
  assert(face.effect_program.length === 1, `${face.id} effect count drift`);
}
for (const rarity of rarities)
  assert(expectedRarities.delete(rarity.id), `${rarity.id} manifest mismatch`);
assert(expectedFaces.size === 0, "dice-face exact-once mismatch");
assert(expectedRarities.size === 0, "dice-rarity exact-once mismatch");
for (const target of targets)
  assert(target.ordering === "StableDomainThenNodeId"
    && target.no_legal_target === "NoOp"
    && target.evidence_quality === "ProjectPolicy",
  `${target.id} target policy drift`);
assert(new Set(controls.map(({ operation }) => operation)).size === 4,
  "dice-control operation mismatch");
for (const control of controls)
  assert(control.result_order === "AuthoredSortThenStableFaceId"
    && control.fallback_policy.startsWith("Reject"),
  `${control.id} control policy drift`);

console.log(
  "Swarm Disaster dice-face verification passed: 42 faces, 3 rarities, " +
  "42 target rules and 4 fail-closed controls.",
);
