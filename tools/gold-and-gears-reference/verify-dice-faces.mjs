#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const decimalPattern =
  /^(0|-?(?:[1-9][0-9]*(?:\.[0-9]*[1-9])?|0\.[0-9]*[1-9]))$/u;
execFileSync(
  process.execPath,
  ["tools/gold-and-gears-reference/import-dice-faces.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);

const normalizedRoot = "content-reference/gold-and-gears-v1";
const expected = {
  "dice-slots.json": 6,
  "dice-faces.json": 80,
  "dice-face-tags.json": 10,
};
const data = new Map();
for (const [file, count] of Object.entries(expected)) {
  const rows = json(`${normalizedRoot}/${file}`);
  assert(Array.isArray(rows) && rows.length === count, `${file} count drift`);
  data.set(file, rows);
}
const rows = [...data.values()].flat();
assert(unique(rows.map(({ id }) => id)), "dice-face pack contains duplicate IDs");
for (const row of rows) {
  assert(row.schema_revision === "starclock.gold-and-gears-row.v1",
    `${row.id} row revision drift`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof row[field] === "string" && row[field].trim() !== "",
      `${row.id} has empty ${field}`);
  assert(row.ownership === "GoldAndGears", `${row.id} ownership drift`);
  assert(row.coverage_state === "DataReady", `${row.id} is not DataReady`);
  assert(row.evidence_quality === "ExactStructured",
    `${row.id} evidence quality drift`);
  assert(Array.isArray(row.source_refs) && row.source_refs.length > 0,
    `${row.id} provenance drift`);
  assert(JSON.stringify(row.tags) === JSON.stringify([...row.tags].sort()),
    `${row.id} tags are not canonical`);
  for (const source of row.source_refs) {
    for (const field of [
      "source_id", "repository", "revision", "path", "locator", "sha256",
      "access_date", "evidence_quality",
    ])
      assert(typeof source[field] === "string" && source[field] !== "",
        `${row.id} source ref omits ${field}`);
    assert(/^[0-9a-f]{64}$/u.test(source.sha256),
      `${row.id} source digest drift`);
    if (source.evidence_quality === "ProjectPolicy")
      assert(source.note?.length > 0 && source.replacement_condition?.length > 0,
        `${row.id} tag policy is not replaceable`);
  }
}

const slots = data.get("dice-slots.json");
const slotById = new Map(slots.map((slot) => [slot.id, slot]));
assert(JSON.stringify(slots.map(({ base_max_rarity: rarity }) => rarity))
  === JSON.stringify([3, 3, 2, 2, 1, 1]), "base slot rarity drift");
assert(JSON.stringify(slots.map(({ upgraded_max_rarity: rarity }) => rarity))
  === JSON.stringify([3, 3, 3, 2, 2, 2]), "upgraded slot rarity drift");
for (const slot of slots)
  assert(slot.slot_index === Number(slot.source_id)
    && /^[0-9]+$/u.test(slot.base_name_text_hash)
    && /^[0-9]+$/u.test(slot.upgraded_name_text_hash),
  `${slot.id} slot identity drift`);

const tags = data.get("dice-face-tags.json");
const tagById = new Map(tags.map((tag) => [tag.id, tag]));
const expectedTagCodes = [
  "ActionPoint",
  "BlockChange",
  "Buff",
  "BuffProMax",
  "Coin",
  "Mark",
  "Miracle",
  "Move",
  "Replicate",
  "SpecialType",
];
assert(JSON.stringify(tags.map(({ mechanical_code: code }) => code).sort())
  === JSON.stringify(expectedTagCodes), "filter-tag code closure drift");
for (const tag of tags)
  assert(tag.mapping_evidence_quality === "ProjectPolicy"
    && tag.mapping_replacement_condition.length > 0
    && tag.source_refs.some(({ evidence_quality: quality }) =>
      quality === "ProjectPolicy"),
  `${tag.id} mapping policy drift`);

const dice = json(`${normalizedRoot}/dice-definitions.json`);
const diceById = new Map(dice.map((definition) => [definition.id, definition]));
const faces = data.get("dice-faces.json");
const faceBySourceId = new Map(faces.map((face) => [face.source_id, face]));
const rarityCounts = new Map();
const stageCounts = new Map();
const eligibilityCounts = new Map();
const noTargetIds = [];
for (const face of faces) {
  assert([1, 2, 3].includes(face.rarity), `${face.id} rarity drift`);
  assert([1, 2, 3].includes(face.activation_stage),
    `${face.id} activation stage drift`);
  rarityCounts.set(face.rarity, (rarityCounts.get(face.rarity) ?? 0) + 1);
  stageCounts.set(
    face.activation_stage,
    (stageCounts.get(face.activation_stage) ?? 0) + 1,
  );
  eligibilityCounts.set(
    face.allowed_dice_ids.length,
    (eligibilityCounts.get(face.allowed_dice_ids.length) ?? 0) + 1,
  );
  assert(face.parameters.every((parameter) => decimalPattern.test(parameter)),
    `${face.id} parameter drift`);
  assert(face.allowed_slot_ids.length === 6
    && unique(face.allowed_slot_ids)
    && face.allowed_slot_ids.every((slotId) => slotById.has(slotId)),
  `${face.id} slot closure drift`);
  assert(face.mechanical_tag_codes.length >= 1
    && unique(face.mechanical_tag_codes)
    && face.filter_tag_ids.length === face.mechanical_tag_codes.length
    && face.filter_tag_ids.every((tagId) => tagById.has(tagId)),
  `${face.id} tag closure drift`);
  for (const tagId of face.filter_tag_ids)
    assert(face.mechanical_tag_codes.includes(tagById.get(tagId).mechanical_code),
      `${face.id} tag join drift`);
  assert(face.tag_mapping_evidence_quality === "ProjectPolicy",
    `${face.id} tag mapping quality drift`);
  assert([3, 12].includes(face.allowed_dice_ids.length)
    && unique(face.allowed_dice_ids)
    && face.allowed_dice_ids.every((diceId) => diceById.has(diceId)),
  `${face.id} dice eligibility drift`);
  assert(face.universal_dice_eligibility
    === (face.allowed_dice_ids.length === 12),
  `${face.id} universal flag drift`);
  if (face.no_legal_target_behavior === "NoEffect") {
    noTargetIds.push(face.source_id);
    assert(face.no_legal_target_evidence_quality === "ExactStructured"
      && face.summary_en.includes("will not take effect when no"),
    `${face.id} no-target evidence drift`);
  } else
    assert(face.no_legal_target_behavior === "FailClosed"
      && face.no_legal_target_evidence_quality === "ProjectPolicy",
    `${face.id} fail-closed no-target policy drift`);
  assert(face.target_resolution_policy.policy_id
    === "dice-face-target-resolution-v1"
    && face.target_resolution_policy.candidate_order
      === "stable-node-or-content-id-ascending"
    && face.target_resolution_policy.unpublished_empty_set_behavior
      === "FailClosed",
  `${face.id} target-resolution policy drift`);
}
assert(JSON.stringify([...rarityCounts.entries()].sort((left, right) =>
  left[0] - right[0]))
  === JSON.stringify([[1, 27], [2, 28], [3, 25]]),
"face rarity distribution drift");
assert(JSON.stringify([...stageCounts.entries()].sort((left, right) =>
  left[0] - right[0]))
  === JSON.stringify([[1, 53], [2, 13], [3, 14]]),
"face activation-stage distribution drift");
assert(JSON.stringify([...eligibilityCounts.entries()].sort((left, right) =>
  left[0] - right[0]))
  === JSON.stringify([[3, 40], [12, 40]]),
"face dice-eligibility distribution drift");
assert(JSON.stringify(noTargetIds.sort())
  === JSON.stringify(["2058", "2070", "2071"]),
"released no-legal-target closure drift");

for (const definition of dice) {
  assert(definition.default_surface_ids.length === slots.length,
    `${definition.id} default loadout size drift`);
  definition.default_surface_ids.forEach((faceId, index) => {
    const face = faceBySourceId.get(faceId);
    const slot = slots[index];
    assert(face, `${definition.id} default face ${faceId} does not resolve`);
    assert(face.allowed_slot_ids.includes(slot.id),
      `${definition.id} face ${faceId} rejects slot ${slot.slot_index}`);
    assert(face.rarity <= slot.base_max_rarity,
      `${definition.id} face ${faceId} exceeds base slot rarity`);
    assert(face.allowed_dice_ids.includes(definition.id),
      `${definition.id} face ${faceId} rejects dice branch`);
  });
  for (const faceId of [
    ...definition.suggestive_surface_ids,
    ...definition.recommended_surface_ids,
  ]) {
    const face = faceBySourceId.get(faceId);
    assert(face && face.allowed_dice_ids.includes(definition.id),
      `${definition.id} recommendation ${faceId} is ineligible`);
  }
}
const dataInflation = diceById.get("gold-gears.custom-dice.403");
assert(dataInflation.effect_parts[0].text_en.includes("cheat attempt")
  && dataInflation.effect_parts[1].text_en.includes("reroll attempt")
  && dataInflation.effect_parts[1].text_en.includes("will not expire"),
"Data Inflation reroll/cheat source semantics drift");

const manifest = json(
  "content-manifests/gold-and-gears-v1/content-manifest.json",
);
for (const [file, categoryId] of [
  ["dice-slots.json", "dice_slots"],
  ["dice-faces.json", "dice_faces"],
  ["dice-face-tags.json", "dice_face_tags"],
]) {
  const actual = data.get(file).map(({ source_id: sourceId }) => sourceId).sort();
  const required = manifest.categories[categoryId].records
    .map(({ id }) => id).sort();
  assert(JSON.stringify(actual) === JSON.stringify(required),
    `${file} manifest exact-once drift`);
}

console.log(
  "Gold and Gears dice faces verified (6 slots; 80 faces; 10 replaceable " +
  "filter-tag joins; 12 legal defaults; 3 exact no-target no-op cases).",
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
