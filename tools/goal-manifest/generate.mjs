import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const EXPECTED_REFERENCE_PACK =
  "0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a";
const GOAL_ID = "core-combat-v1";
const SNAPSHOT = "4.4";
const GENERATED_ON = "2026-07-17";
const MANIFEST_SCHEMA = "starclock-goal-manifest-v1";

const repoRoot = path.resolve(process.cwd());
const referenceRoot = path.join(repoRoot, "content-reference", "v4.4");
const outputRoot = path.join(repoRoot, "content-manifests", GOAL_ID);
const checkOnly = process.argv.includes("--check");

const referenceIndex = readJson(path.join(referenceRoot, "pack-index.json"));
if (referenceIndex.pack_sha256 !== EXPECTED_REFERENCE_PACK) {
  throw new Error(
    `reference pack mismatch: expected ${EXPECTED_REFERENCE_PACK}, got ${referenceIndex.pack_sha256}`,
  );
}
verifyReferencePack(referenceIndex);

const characters = readJson(path.join(referenceRoot, "characters.json"));
const lightCones = readJson(path.join(referenceRoot, "light-cones.json"));
const enemyTemplates = readJson(path.join(referenceRoot, "enemy-templates.json"));
const enemyVariants = readJson(path.join(referenceRoot, "enemy-variants.json"));
const encounters = readJson(path.join(referenceRoot, "encounters.json"));

const characterById = byId(characters);
const lightConeById = byId(lightCones);
const enemyTemplateById = byId(enemyTemplates);
const enemyVariantById = byId(enemyVariants);
const encounterById = byId(encounters);

const v1bCharacterIds = [
  "character.aglaea",
  "character.asta",
  "character.clara",
  "character.firefly",
  "character.kafka",
  "character.silver-wolf-lv-999",
];

const standardEncounterSelections = [
  {
    id: "encounter.cocoon.0001",
    archetypes: ["BasicSingleWave"],
    note: "One public single wave with ordinary single-target enemy actions.",
  },
  {
    id: "encounter.mainline.0541",
    archetypes: [
      "DeterministicMultiWave",
      "SummonerOrReplaceableEntity",
      "CrowdControlAndDot",
      "DefeatRevivalOrReturn",
      "TargetInvalidation",
      "PostActionWaveAdvancement",
    ],
    note: "Three public waves with DoT, Mara revival, and a final Shape Shifter summoner.",
  },
  {
    id: "encounter.farmelement.0008",
    archetypes: ["EliteWithAdds", "CrowdControlAndDot", "TargetInvalidation"],
    note: "Aurumaton Gatekeeper with two Dragonfish adds and authored crowd control.",
  },
  {
    id: "encounter.mainline.0276",
    archetypes: ["MultiPhaseBoss", "CrowdControlAndDot"],
    note: "Cocolia, Mother of Deception with explicit phase-changing abilities.",
  },
  {
    id: "encounter.mainline.0755",
    archetypes: ["ToughnessLayerRouting", "MultiPhaseBoss"],
    note: "Great Septimus with three authored Toughness layers and phase-tagged abilities.",
  },
  {
    id: "encounter.mainline.1253",
    archetypes: [
      "DeterministicMultiWave",
      "EliteWithAdds",
      "SummonerOrReplaceableEntity",
      "CrowdControlAndDot",
      "DefeatRevivalOrReturn",
      "TargetInvalidation",
      "PostActionWaveAdvancement",
    ],
    note: "Three public waves with Burn, control, revive/return, self-destruction, and summoning.",
  },
];

const scenarios = [
  scenario(
    "scenario.standard-v1.basic-single-wave",
    "encounter.cocoon.0001",
    "104729",
    ["SingleTarget", "Blast", "AoE", "Bounce", "Support"],
    [
      build("character.asta", "light-cone.chorus", 0, 1),
      build("character.dan-heng", "light-cone.adversarial", 0, 1),
      build("character.march-7th.preservation", "light-cone.amber", 0, 1),
      build("character.natasha", "light-cone.cornucopia", 0, 1),
    ],
  ),
  scenario(
    "scenario.standard-v1.multi-wave-dot-revival",
    "encounter.mainline.0541",
    "209759",
    ["SingleTarget", "Blast", "AoE", "Bounce", "Support"],
    [
      build("character.kafka", "light-cone.hidden-shadow", 0, 1),
      build("character.sampo", "light-cone.loop", 0, 1),
      build("character.asta", "light-cone.meshing-cogs", 0, 1),
      build("character.bailu", "light-cone.fine-fruit", 0, 1),
    ],
  ),
  scenario(
    "scenario.standard-v1.elite-control-counter",
    "encounter.farmelement.0008",
    "314159",
    ["SingleTarget", "Blast", "AoE", "Support"],
    [
      build("character.clara", "light-cone.mutual-demise", 0, 1),
      build("character.tingyun", "light-cone.mediation", 0, 1),
      build("character.march-7th.preservation", "light-cone.defense", 0, 1),
      build("character.lynx", "light-cone.multiplication", 0, 1),
    ],
  ),
  scenario(
    "scenario.standard-v1.cocolia-phase-change",
    "encounter.mainline.0276",
    "419431",
    ["SingleTarget", "Blast", "AoE", "Bounce", "Support", "Enhance"],
    [
      build("character.firefly", "light-cone.collapsing-sky", 6, 5),
      build("character.ruan-mei", "light-cone.meshing-cogs", 6, 5),
      build("character.trailblazer.harmony", "light-cone.chorus", 6, 5),
      build("character.gallagher", "light-cone.cornucopia", 6, 5),
    ],
  ),
  scenario(
    "scenario.standard-v1.layered-toughness",
    "encounter.mainline.0755",
    "524287",
    ["SingleTarget", "Blast", "AoE", "Bounce", "Support", "Enhance"],
    [
      build("character.firefly", "light-cone.shattered-home", 0, 1),
      build("character.fugue", "light-cone.void", 0, 1),
      build("character.ruan-mei", "light-cone.mediation", 0, 1),
      build("character.gallagher", "light-cone.multiplication", 0, 1),
    ],
  ),
  scenario(
    "scenario.standard-v1.target-invalidation-and-return",
    "encounter.mainline.1253",
    "629137",
    ["SingleTarget", "Blast", "AoE", "Bounce", "Support"],
    [
      build("character.herta", "light-cone.data-bank", 6, 5),
      build("character.himeko", "light-cone.passkey", 6, 5),
      build("character.asta", "light-cone.chorus", 6, 5),
      build("character.bailu", "light-cone.fine-fruit", 6, 5),
    ],
  ),
];

const characterEntries = [...characters]
  .sort(compareId)
  .map((record) => ({
    id: record.id,
    reference_id: record.id,
    inclusion_state: "Required",
    implementation_state: "Pending",
    release_state: "Released",
    enabled: true,
    reference_quality: record.quality,
  }));

const lightConeEntries = [...lightCones]
  .sort(compareId)
  .map((record) => ({
    id: record.id,
    reference_id: record.id,
    inclusion_state: "Required",
    implementation_state: "Pending",
    release_state: "Released",
    enabled: true,
    reference_quality: record.quality,
  }));

const selectedEncounters = standardEncounterSelections.map((selection) => {
  const reference = required(encounterById, selection.id, "encounter");
  return {
    id: selection.id,
    reference_id: selection.id,
    inclusion_state: "Required",
    implementation_state: "Pending",
    archetypes: [...selection.archetypes].sort(),
    note: selection.note,
    source_stage_id: reference.source_stage_id,
  };
});

const selectedVariantIds = new Set();
for (const selection of standardEncounterSelections) {
  const encounter = required(encounterById, selection.id, "encounter");
  for (const wave of encounter.waves) {
    for (const slot of wave.slots) selectedVariantIds.add(slot.enemy_variant_id);
  }
}

const selectedEnemies = [...selectedVariantIds]
  .sort()
  .map((variantId) => {
    const variant = required(enemyVariantById, variantId, "enemy variant");
    const template = required(enemyTemplateById, variant.enemy_id, "enemy template");
    return {
      id: variant.id,
      variant_reference_id: variant.id,
      template_reference_id: template.id,
      inclusion_state: "Required",
      implementation_state: "Pending",
      reference_quality: variant.quality,
      template_reference_quality: template.quality,
    };
  });

const remainingCharacterIds = characterEntries
  .map((entry) => entry.id)
  .filter((id) => !v1bCharacterIds.includes(id));

const partitions = {
  schema_revision: MANIFEST_SCHEMA,
  goal_id: GOAL_ID,
  snapshot: SNAPSHOT,
  generated_on: GENERATED_ON,
  reference_pack_sha256: EXPECTED_REFERENCE_PACK,
  character_v1b: {
    batch_id: "G01-P7-V1B",
    ids: [...v1bCharacterIds].sort(),
  },
  character_partitions: partition(remainingCharacterIds, 8, "G01-P7-C"),
  light_cone_partitions: partition(
    lightConeEntries.map((entry) => entry.id),
    16,
    "G01-P7-L",
  ),
};

const header = {
  schema_revision: MANIFEST_SCHEMA,
  goal_id: GOAL_ID,
  snapshot: SNAPSHOT,
  generated_on: GENERATED_ON,
  reference_pack_sha256: EXPECTED_REFERENCE_PACK,
};

const outputs = new Map([
  [
    "released-character-forms.json",
    json({ ...header, kind: "ReleasedCharacterCombatForms", entries: characterEntries }),
  ],
  [
    "released-light-cones.json",
    json({ ...header, kind: "ReleasedLightCones", entries: lightConeEntries }),
  ],
  [
    "standard-v1.json",
    json({
      ...header,
      kind: "StandardV1",
      profile: {
        id: "profile.standard-v1",
        inclusion_state: "Required",
        implementation_state: "Pending",
        implicit_clock: false,
        implicit_score: false,
        implicit_season_rules: false,
        default_wave_transition: "AfterAction",
      },
      enemies: selectedEnemies,
      encounters: selectedEncounters,
      scenarios,
    }),
  ],
  ["partitions.json", json(partitions)],
]);

const indexedFiles = [...outputs.entries()]
  .sort(([left], [right]) => compareText(left, right))
  .map(([name, content]) => ({ name, sha256: sha256Text(content) }));
const digestInput = indexedFiles
  .map((entry) => `${entry.name}\0${entry.sha256}\n`)
  .join("");
const manifestIndex = {
  schema_revision: "starclock-goal-manifest-index-v1",
  goal_id: GOAL_ID,
  reference_pack_sha256: EXPECTED_REFERENCE_PACK,
  files: indexedFiles,
  manifest_sha256: sha256Text(digestInput),
};
outputs.set("manifest-index.json", json(manifestIndex));

if (checkOnly) {
  const failures = [];
  for (const [name, expected] of outputs) {
    const file = path.join(outputRoot, name);
    if (!fs.existsSync(file)) failures.push(`${name} is missing`);
    else if (fs.readFileSync(file, "utf8") !== expected) failures.push(`${name} has drift`);
  }
  const expectedNames = [...outputs.keys()].sort();
  const actualNames = fs.existsSync(outputRoot)
    ? fs.readdirSync(outputRoot).filter((name) => name.endsWith(".json")).sort()
    : [];
  if (JSON.stringify(actualNames) !== JSON.stringify(expectedNames)) {
    failures.push("generated JSON file list has drift");
  }
  if (failures.length) {
    for (const failure of failures) process.stderr.write(`ERROR: ${failure}\n`);
    process.exit(1);
  }
  process.stdout.write(
    `Goal manifest generation check passed. manifest=${manifestIndex.manifest_sha256}\n`,
  );
} else {
  fs.mkdirSync(outputRoot, { recursive: true });
  for (const [name, content] of outputs) {
    fs.writeFileSync(path.join(outputRoot, name), content, "utf8");
  }
  process.stdout.write(
    `Generated ${outputs.size} files in ${path.relative(repoRoot, outputRoot)}. manifest=${manifestIndex.manifest_sha256}\n`,
  );
}

function scenario(id, encounterId, seed, targetShapes, builds) {
  return {
    id,
    encounter_id: encounterId,
    inclusion_state: "Required",
    implementation_state: "Pending",
    seed,
    controller: "baseline-v1",
    required_target_shapes: [...targetShapes].sort(),
    builds,
  };
}

function build(formId, lightConeId, eidolon, superimposition) {
  return {
    form_id: formId,
    character_level: 80,
    character_promotion: 6,
    ability_level_profile: eidolon === 6 ? "MaximumGameLegal" : "StandardMaximum",
    trace_profile: "AllBattleRelevant",
    eidolon,
    light_cone_id: lightConeId,
    light_cone_level: 80,
    light_cone_promotion: 6,
    superimposition,
  };
}

function partition(ids, size, prefix) {
  const sorted = [...ids].sort();
  const result = [];
  for (let index = 0; index < sorted.length; index += size) {
    result.push({
      batch_id: `${prefix}${String(result.length + 1).padStart(2, "0")}`,
      ids: sorted.slice(index, index + size),
    });
  }
  return result;
}

function byId(records) {
  return new Map(records.map((record) => [record.id, record]));
}

function required(index, id, label) {
  const value = index.get(id);
  if (!value) throw new Error(`missing ${label} reference ${id}`);
  return value;
}

function compareId(left, right) {
  return compareText(left.id, right.id);
}

function compareText(left, right) {
  if (left < right) return -1;
  if (left > right) return 1;
  return 0;
}

function verifyReferencePack(index) {
  for (const entry of index.files) {
    const actual = crypto
      .createHash("sha256")
      .update(fs.readFileSync(path.join(referenceRoot, entry.name)))
      .digest("hex");
    if (actual !== entry.sha256) {
      throw new Error(`reference pack hash mismatch for ${entry.name}`);
    }
  }
  const digestInput = index.files.map((entry) => `${entry.name}\0${entry.sha256}\n`).join("");
  if (sha256Text(digestInput) !== EXPECTED_REFERENCE_PACK) {
    throw new Error("reference pack aggregate digest mismatch");
  }
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function json(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function sha256Text(value) {
  return crypto.createHash("sha256").update(value, "utf8").digest("hex");
}
