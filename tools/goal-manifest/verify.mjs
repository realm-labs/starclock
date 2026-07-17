import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const EXPECTED_REFERENCE_PACK =
  "0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a";
const EXPECTED_ARCHETYPES = new Set([
  "BasicSingleWave",
  "DeterministicMultiWave",
  "EliteWithAdds",
  "SummonerOrReplaceableEntity",
  "CrowdControlAndDot",
  "MultiPhaseBoss",
  "ToughnessLayerRouting",
  "DefeatRevivalOrReturn",
  "TargetInvalidation",
  "PostActionWaveAdvancement",
]);
const EXPECTED_TARGET_SHAPES = new Set([
  "SingleTarget",
  "Blast",
  "AoE",
  "Bounce",
  "Support",
  "Enhance",
]);

const repoRoot = path.resolve(process.cwd());
const manifestRoot = path.join(repoRoot, "content-manifests", "core-combat-v1");
const referenceRoot = path.join(repoRoot, "content-reference", "v4.4");
const failures = [];

const characterManifest = readManifest("released-character-forms.json");
const lightConeManifest = readManifest("released-light-cones.json");
const standardManifest = readManifest("standard-v1.json");
const partitions = readManifest("partitions.json");
const index = readManifest("manifest-index.json");

const referenceCharacters = readReference("characters.json");
const referenceLightCones = readReference("light-cones.json");
const referenceTemplates = readReference("enemy-templates.json");
const referenceVariants = readReference("enemy-variants.json");
const referenceEncounters = readReference("encounters.json");
const referencePackIndex = readReference("pack-index.json");

if (referencePackIndex.pack_sha256 !== EXPECTED_REFERENCE_PACK) {
  fail(`reference pack digest is ${referencePackIndex.pack_sha256}`);
}
verifyReferencePack(referencePackIndex);
for (const manifest of [characterManifest, lightConeManifest, standardManifest, partitions]) {
  if (manifest.reference_pack_sha256 !== EXPECTED_REFERENCE_PACK) {
    fail(`${manifest.kind ?? "partitions"} is not bound to the required reference pack`);
  }
}

verifyEntries(
  "character",
  characterManifest.entries,
  referenceCharacters,
  88,
);
verifyEntries(
  "Light Cone",
  lightConeManifest.entries,
  referenceLightCones,
  165,
);

const characterIds = ids(referenceCharacters);
const lightConeIds = ids(referenceLightCones);
const characterPathById = new Map(referenceCharacters.map((entry) => [entry.id, entry.path]));
const lightConePathById = new Map(referenceLightCones.map((entry) => [entry.id, entry.path]));
const templateIds = ids(referenceTemplates);
const variantById = new Map(referenceVariants.map((entry) => [entry.id, entry]));
const encounterById = new Map(referenceEncounters.map((entry) => [entry.id, entry]));

const standardEnemyIds = new Set();
for (const enemy of standardManifest.enemies) {
  if (standardEnemyIds.has(enemy.id)) fail(`duplicate standard enemy ${enemy.id}`);
  standardEnemyIds.add(enemy.id);
  const variant = variantById.get(enemy.variant_reference_id);
  if (!variant) fail(`${enemy.id} has missing variant reference`);
  else if (variant.enemy_id !== enemy.template_reference_id) {
    fail(`${enemy.id} template reference does not match its variant`);
  }
  if (!templateIds.has(enemy.template_reference_id)) {
    fail(`${enemy.id} has missing template reference`);
  }
  verifyRequiredPending(enemy, `standard enemy ${enemy.id}`);
}

const coveredArchetypes = new Set();
const selectedEncounterIds = new Set();
for (const entry of standardManifest.encounters) {
  const encounter = encounterById.get(entry.reference_id);
  if (!encounter) fail(`missing standard encounter ${entry.reference_id}`);
  if (selectedEncounterIds.has(entry.id)) fail(`duplicate standard encounter ${entry.id}`);
  selectedEncounterIds.add(entry.id);
  verifyRequiredPending(entry, `standard encounter ${entry.id}`);
  for (const archetype of entry.archetypes) coveredArchetypes.add(archetype);
  if (encounter) {
    for (const wave of encounter.waves) {
      for (const slot of wave.slots) {
        if (!standardEnemyIds.has(slot.enemy_variant_id)) {
          fail(`${entry.id} slot ${slot.slot} omits enemy ${slot.enemy_variant_id}`);
        }
      }
    }
  }
}
expectSet("standard archetypes", coveredArchetypes, EXPECTED_ARCHETYPES);

const coveredTargetShapes = new Set();
for (const entry of standardManifest.scenarios) {
  verifyRequiredPending(entry, `standard scenario ${entry.id}`);
  if (!selectedEncounterIds.has(entry.encounter_id)) {
    fail(`${entry.id} references an encounter outside standard-v1`);
  }
  if (!/^\d+$/.test(entry.seed)) fail(`${entry.id} seed is not a canonical unsigned integer string`);
  if (entry.builds.length !== 4) fail(`${entry.id} must bind exactly four player builds`);
  for (const shape of entry.required_target_shapes) coveredTargetShapes.add(shape);
  for (const build of entry.builds) {
    if (!characterIds.has(build.form_id)) fail(`${entry.id} has missing form ${build.form_id}`);
    if (!lightConeIds.has(build.light_cone_id)) {
      fail(`${entry.id} has missing Light Cone ${build.light_cone_id}`);
    }
    if (characterPathById.get(build.form_id) !== lightConePathById.get(build.light_cone_id)) {
      fail(`${entry.id} has path-mismatched build ${build.form_id} / ${build.light_cone_id}`);
    }
    if (![0, 6].includes(build.eidolon)) fail(`${entry.id} has unsupported Eidolon fixture`);
    if (![1, 5].includes(build.superimposition)) {
      fail(`${entry.id} has unsupported Superimposition fixture`);
    }
  }
}
expectSet("target shapes", coveredTargetShapes, EXPECTED_TARGET_SHAPES);

const v1bIds = new Set(partitions.character_v1b.ids);
if (partitions.character_v1b.batch_id !== "G01-P7-V1B") fail("V1B batch ID changed");
expectSet(
  "V1B membership",
  v1bIds,
  new Set([
    "character.aglaea",
    "character.asta",
    "character.clara",
    "character.firefly",
    "character.kafka",
    "character.silver-wolf-lv-999",
  ]),
);

verifyPartitions(
  "character",
  partitions.character_partitions,
  "G01-P7-C",
  8,
  new Set([...characterIds].filter((id) => !v1bIds.has(id))),
);
verifyPartitions(
  "Light Cone",
  partitions.light_cone_partitions,
  "G01-P7-L",
  16,
  lightConeIds,
);

verifyManifestIndex(index);

if (failures.length) {
  for (const failure of failures) process.stderr.write(`ERROR: ${failure}\n`);
  process.exit(1);
}

process.stdout.write(
  [
    "Goal manifest verification passed.",
    `reference_pack=${EXPECTED_REFERENCE_PACK}`,
    `manifest=${index.manifest_sha256}`,
    `characters=${characterManifest.entries.length}`,
    `light_cones=${lightConeManifest.entries.length}`,
    `standard_enemies=${standardManifest.enemies.length}`,
    `standard_encounters=${standardManifest.encounters.length}`,
    `standard_scenarios=${standardManifest.scenarios.length}`,
    `character_partitions=${partitions.character_partitions.length}`,
    `light_cone_partitions=${partitions.light_cone_partitions.length}`,
  ].join(" ") + "\n",
);

function verifyEntries(label, entries, references, expectedCount) {
  if (entries.length !== expectedCount) fail(`${label} count is ${entries.length}, expected ${expectedCount}`);
  const referenceIds = ids(references);
  const entryIds = new Set();
  for (const entry of entries) {
    if (entryIds.has(entry.id)) fail(`duplicate ${label} manifest ID ${entry.id}`);
    entryIds.add(entry.id);
    if (entry.reference_id !== entry.id || !referenceIds.has(entry.reference_id)) {
      fail(`${label} ${entry.id} has an invalid reference binding`);
    }
    verifyRequiredPending(entry, `${label} ${entry.id}`);
  }
  expectSet(`${label} manifest`, entryIds, referenceIds);
}

function verifyRequiredPending(entry, label) {
  if (entry.inclusion_state !== "Required") fail(`${label} is not Required`);
  if (entry.implementation_state !== "Pending") fail(`${label} is not Pending`);
}

function verifyPartitions(label, values, prefix, maxSize, expectedIds) {
  const actualIds = new Set();
  for (let index = 0; index < values.length; index += 1) {
    const partition = values[index];
    const expectedBatchId = `${prefix}${String(index + 1).padStart(2, "0")}`;
    if (partition.batch_id !== expectedBatchId) {
      fail(`${label} partition ${index + 1} is ${partition.batch_id}, expected ${expectedBatchId}`);
    }
    if (!partition.ids.length || partition.ids.length > maxSize) {
      fail(`${partition.batch_id} has invalid size ${partition.ids.length}`);
    }
    if (JSON.stringify(partition.ids) !== JSON.stringify([...partition.ids].sort())) {
      fail(`${partition.batch_id} membership is not in stable-ID order`);
    }
    for (const id of partition.ids) {
      if (actualIds.has(id)) fail(`${id} appears in more than one ${label} partition`);
      actualIds.add(id);
    }
  }
  expectSet(`${label} partitions`, actualIds, expectedIds);
}

function verifyManifestIndex(value) {
  if (value.reference_pack_sha256 !== EXPECTED_REFERENCE_PACK) {
    fail("manifest index reference-pack binding changed");
  }
  const actualFiles = fs
    .readdirSync(manifestRoot)
    .filter((name) => name.endsWith(".json") && name !== "manifest-index.json")
    .sort();
  const indexedFiles = value.files.map((entry) => entry.name);
  if (JSON.stringify(actualFiles) !== JSON.stringify(indexedFiles)) {
    fail("manifest-index file list does not match generated JSON files");
  }
  for (const entry of value.files) {
    const actual = sha256File(path.join(manifestRoot, entry.name));
    if (actual !== entry.sha256) fail(`manifest-index hash mismatch for ${entry.name}`);
  }
  const digestInput = value.files.map((entry) => `${entry.name}\0${entry.sha256}\n`).join("");
  const digest = sha256Text(digestInput);
  if (digest !== value.manifest_sha256) fail(`manifest digest mismatch: got ${digest}`);
}

function verifyReferencePack(value) {
  for (const entry of value.files) {
    const actual = sha256File(path.join(referenceRoot, entry.name));
    if (actual !== entry.sha256) fail(`reference pack hash mismatch for ${entry.name}`);
  }
  const digestInput = value.files.map((entry) => `${entry.name}\0${entry.sha256}\n`).join("");
  const digest = sha256Text(digestInput);
  if (digest !== EXPECTED_REFERENCE_PACK) fail(`reference pack aggregate digest is ${digest}`);
}

function expectSet(label, actual, expected) {
  const missing = [...expected].filter((value) => !actual.has(value));
  const extra = [...actual].filter((value) => !expected.has(value));
  if (missing.length || extra.length) {
    fail(`${label} differs; missing=[${missing.join(", ")}], extra=[${extra.join(", ")}]`);
  }
}

function ids(records) {
  return new Set(records.map((entry) => entry.id));
}

function readManifest(name) {
  return JSON.parse(fs.readFileSync(path.join(manifestRoot, name), "utf8"));
}

function readReference(name) {
  return JSON.parse(fs.readFileSync(path.join(referenceRoot, name), "utf8"));
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
