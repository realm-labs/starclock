import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const root = path.resolve(process.argv[2] ?? "content-reference/v4.4");
const failures = [];

const characters = read("characters.json");
const abilities = read("character-abilities.json");
const traces = read("character-traces.json");
const eidolons = read("character-eidolons.json");
const lightCones = read("light-cones.json");
const enemyTemplates = read("enemy-templates.json");
const enemyVariants = read("enemy-variants.json");
const enemyAbilities = read("enemy-abilities.json");
const encounters = read("encounters.json");
const packIndex = read("pack-index.json");
const manifest = read("manifest.json");

expectCount("characters", characters, 88);
expectCount("character abilities", abilities, 583);
expectCount("character Traces", traces, 1618);
expectCount("character Eidolons", eidolons, 528);
expectCount("Light Cones", lightCones, 165);
expectCount("enemy templates", enemyTemplates, 613);
expectCount("enemy variants", enemyVariants, 2591);
expectCount("enemy abilities", enemyAbilities, 3611);
expectCount("ordinary encounter candidates", encounters, 1471);

const abilityIds = ids(abilities, "character ability");
const traceIds = ids(traces, "Trace");
const eidolonIds = ids(eidolons, "Eidolon");
const characterIds = ids(characters, "character");
const lightConeIds = ids(lightCones, "Light Cone");
const enemyIds = ids(enemyTemplates, "enemy template");
const enemyVariantIds = ids(enemyVariants, "enemy variant");
const enemyAbilityIds = ids(enemyAbilities, "enemy ability");

void lightConeIds;

for (const character of characters) {
  expectReferences(character, "ability_ids", abilityIds);
  expectReferences(character, "trace_ids", traceIds);
  expectReferences(character, "eidolon_ids", eidolonIds);
  if (character.eidolon_ids.length !== 6) fail(`${character.id} has ${character.eidolon_ids.length} Eidolons instead of 6`);
  const ranks = character.eidolon_ids
    .map((id) => eidolons.find((entry) => entry.id === id)?.rank)
    .sort((a, b) => a - b);
  if (JSON.stringify(ranks) !== JSON.stringify([1, 2, 3, 4, 5, 6])) fail(`${character.id} does not contain exactly E1-E6`);
  if (!character.behavior_summary_en || !character.engine_contract_en) fail(`${character.id} lacks a mechanic contract`);
  if (!character.promotions.length) fail(`${character.id} lacks promotion data`);
}

for (const ability of abilities) {
  if (!characterIds.has(ability.character_id)) fail(`${ability.id} references missing character ${ability.character_id}`);
  if (!ability.mechanism_quality) fail(`${ability.id} lacks mechanism_quality`);
  verifyTextEvidence(ability);
}

for (const trace of traces) {
  if (!characterIds.has(trace.character_id)) fail(`${trace.id} references missing character ${trace.character_id}`);
  verifyTextEvidence(trace);
}

for (const eidolon of eidolons) {
  if (!characterIds.has(eidolon.character_id)) fail(`${eidolon.id} references missing character ${eidolon.character_id}`);
  verifyTextEvidence(eidolon);
}

for (const cone of lightCones) {
  if (cone.passive.superimpositions.length !== 5) fail(`${cone.id} does not contain exactly S1-S5`);
  const ranks = cone.passive.superimpositions.map((entry) => entry.rank);
  if (JSON.stringify(ranks) !== JSON.stringify([1, 2, 3, 4, 5])) fail(`${cone.id} has invalid Superimposition order`);
  if (!cone.promotions.length) fail(`${cone.id} lacks promotion data`);
  verifyTextEvidence(cone.passive, cone.id);
}

for (const enemy of enemyTemplates) {
  expectReferences(enemy, "ability_ids", enemyAbilityIds);
  if (!enemy.source_character_config) fail(`${enemy.id} lacks source character-config evidence`);
}

for (const variant of enemyVariants) {
  if (!enemyIds.has(variant.enemy_id)) fail(`${variant.id} references missing enemy ${variant.enemy_id}`);
}

for (const ability of enemyAbilities) {
  if (!enemyIds.has(ability.enemy_id)) fail(`${ability.id} references missing enemy ${ability.enemy_id}`);
  if (!ability.mechanism_quality) fail(`${ability.id} lacks mechanism_quality`);
  verifyTextEvidence(ability);
}

for (const encounter of encounters) {
  for (const wave of encounter.waves) {
    for (const slot of wave.slots) {
      if (!slot.enemy_variant_id || !enemyVariantIds.has(slot.enemy_variant_id)) {
        fail(`${encounter.id} has unresolved enemy variant ${slot.source_monster_id}`);
      }
    }
  }
}

verifyPackIndex(packIndex);
verifySourceManifest(manifest);

if (failures.length) {
  for (const failure of failures) process.stderr.write(`ERROR: ${failure}\n`);
  process.exit(1);
}

process.stdout.write(
  [
    "Content reference verification passed.",
    `pack=${packIndex.pack_sha256}`,
    `characters=${characters.length}`,
    `light_cones=${lightCones.length}`,
    `enemy_templates=${enemyTemplates.length}`,
    `enemy_variants=${enemyVariants.length}`,
    `enemy_abilities=${enemyAbilities.length}`,
    `ordinary_encounters=${encounters.length}`,
  ].join(" ") + "\n",
);

function verifyPackIndex(index) {
  const actualFiles = fs.readdirSync(root)
    .filter((name) => name.endsWith(".json") && name !== "pack-index.json")
    .sort();
  const indexedFiles = index.files.map((entry) => entry.name);
  if (JSON.stringify(actualFiles) !== JSON.stringify(indexedFiles)) fail("pack-index file list does not match generated JSON files");
  for (const entry of index.files) {
    const actual = sha256File(path.join(root, entry.name));
    if (actual !== entry.sha256) fail(`pack-index hash mismatch for ${entry.name}`);
  }
  const digestInput = index.files.map((entry) => `${entry.name}\0${entry.sha256}\n`).join("");
  const digest = sha256Text(digestInput);
  if (digest !== index.pack_sha256) fail(`pack digest mismatch: expected ${index.pack_sha256}, got ${digest}`);
}

function verifySourceManifest(value) {
  const expected = new Map([
    ["dimbreath-turnbasedgamedata", "fd978d6ef09f941fba644c731ab54abd6f7c3568"],
    ["mar-7th-star-rail-res", "7b349e39ee0f6f3bf814567995829b99c95e7a93"],
  ]);
  for (const repository of value.repositories ?? []) {
    const revision = expected.get(repository.id);
    if (!revision) fail(`unexpected source repository ${repository.id}`);
    else if (repository.revision !== revision) fail(`source revision mismatch for ${repository.id}`);
    expected.delete(repository.id);
  }
  for (const missing of expected.keys()) fail(`missing source repository ${missing}`);
}

function verifyTextEvidence(record, label = record.id) {
  const evidence = record.source_text;
  if (!evidence) return;
  if (evidence.emitted !== false) fail(`${label} emits source mechanic text`);
  if (evidence.sha256 && !/^[0-9a-f]{64}$/.test(evidence.sha256)) fail(`${label} has invalid source-text SHA-256`);
}

function expectReferences(record, field, allowed) {
  for (const id of record[field] ?? []) {
    if (!allowed.has(id)) fail(`${record.id}.${field} references missing ${id}`);
  }
}

function ids(records, label) {
  const result = new Set();
  for (const record of records) {
    if (!record.id) fail(`${label} record lacks id`);
    else if (result.has(record.id)) fail(`duplicate ${label} id ${record.id}`);
    else result.add(record.id);
  }
  return result;
}

function expectCount(label, records, expected) {
  if (records.length !== expected) fail(`${label}: expected ${expected}, got ${records.length}`);
}

function read(name) {
  return JSON.parse(fs.readFileSync(path.join(root, name), "utf8"));
}

function sha256File(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function sha256Text(value) {
  return crypto.createHash("sha256").update(value, "utf8").digest("hex");
}

function fail(message) {
  failures.push(message);
}
