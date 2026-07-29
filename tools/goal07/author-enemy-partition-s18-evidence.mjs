#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const partitionId = "G07-P5-M15-S18";
const write = process.argv.includes("--write");
assert(
  process.argv.slice(2).every((argument) => argument === "--write"),
  "usage: author-enemy-partition-s18-evidence.mjs [--write]",
);

const goalRoot = "evidence/standard-universe-mechanics-complete-v1";
const anchorPath = `${goalRoot}/sources/${partitionId}-numeric-anchors.json`;
const reviewPath = `${goalRoot}/source-reviews/${partitionId}.json`;
const templatesPath = "content-reference/v4.4/enemy-templates.json";
const variantsPath = "content-reference/v4.4/enemy-variants.json";
const abilitiesPath = "content-reference/v4.4/enemy-abilities.json";
const difficultiesPath =
  "content-reference/standard-universe-v1/world-difficulties.json";
const encountersPath =
  "content-reference/standard-universe-v1/encounter-groups.json";
const manifestPath =
  "content-manifests/standard-universe-mechanics-complete-v1/content-partitions.json";

const manifest = json(manifestPath);
const partition = manifest.partitions.find(({ id }) => id === partitionId);
assert(partition, `${partitionId}: frozen partition is missing`);
const templates = json(templatesPath);
const variants = json(variantsPath);

const specs = [
  {
    id: "enemy.voidranger-trampler.elite.variant.01",
    source: "8013010",
    levels: [11, 15, 29, 44, 61, 66, 75, 84],
    accuracy: "ExactPublic",
    publicUrl:
      "https://honkai-star-rail.fandom.com/wiki/Voidranger%3A_Trampler",
  },
  {
    id: "enemy.windspawn.minion.variant.01",
    source: "8001050",
    levels: [51, 52, 54, 57],
    accuracy: "ApprovedNumericApproximation",
    publicUrl: "https://honkai-star-rail.fandom.com/wiki/Windspawn",
  },
  {
    id: "enemy.wraith-warden.minionlv2.variant.01",
    source: "2002030",
    levels: [54],
    accuracy: "ApprovedNumericApproximation",
    publicUrl: "https://honkai-star-rail.fandom.com/wiki/Wraith_Warden",
  },
];

assert(
  JSON.stringify(specs.map(({ id }) => id)) ===
    JSON.stringify(partition.enemy_variant_ids),
  `${partitionId}: evidence specification differs from frozen variants`,
);

const hp930 = {
  11: 1770,
  15: 2234,
  29: 4669,
  44: 13551,
  51: 23435,
  52: 26279,
  54: 31968,
  57: 40501,
  61: 52964,
  66: 72616,
  75: 112994,
  84: 167044,
};
const atk = {
  11: 29,
  15: 40,
  29: 93,
  44: 187,
  51: 244,
  52: 255,
  54: 276,
  57: 307,
  61: 348,
  66: 397,
  75: 494,
  84: 597,
};

const anchor = {
  schema_revision: "starclock.goal07-enemy-numeric-anchor.v2",
  partition_id: partitionId,
  numeric_policy_id: "goal07-s18-public-and-derived-level-curve-v1",
  source: {
    publisher: "Honkai: Star Rail Wiki and committed version 4.4 extraction",
    url: "https://honkai-star-rail.fandom.com/wiki/Voidranger%3A_Trampler",
    accessed_on: "2026-07-29",
    game_version: "4.4",
    confidence: "PerVariantExactOrApprovedDerivedCurve",
  },
  derivation: {
    hp_reference_base: "930",
    hp_rounding: "NearestTiesAway",
    atk_reference_base: "18",
    def_reference_base: "210",
    speed_breakpoints: [
      { level_minimum: 1, multiplier: "1" },
      { level_minimum: 65, multiplier: "1.1" },
      { level_minimum: 78, multiplier: "1.2" },
      { level_minimum: 86, multiplier: "1.32" },
    ],
    exact_overrides:
      "The Trampler uses its reviewed public level curve; Windspawn and Wraith Warden use the named normalized template curve.",
  },
  variants: specs.map((spec) => {
    const template = templates.find(
      ({ source_template_id }) => String(source_template_id) === spec.source,
    );
    const variant = variants.find(
      ({ source_monster_id }) => String(source_monster_id) === spec.source,
    );
    assert(template && variant, `${spec.id}: retained source definition is missing`);
    const baseHp = Number(template.base_stats.hp);
    const baseDef = Number(template.base_stats.def);
    const baseSpd = Number(template.base_stats.spd);
    const baseEffectResistance = Number(template.base_stats.effect_res ?? 0);
    return {
      enemy_variant_id: spec.id,
      source_monster_id: spec.source,
      combat_rank: template.rank,
      accuracy_disposition: spec.accuracy,
      numeric_policy_id:
        spec.accuracy === "ExactPublic"
          ? "goal07-exact-public-per-level-v1"
          : "goal07-normalized-template-level-curve-v1",
      public_url: spec.publicUrl,
      levels: spec.levels.map((level) => ({
        authored_level: level,
        base_hp: String(Math.floor((hp930[level] * baseHp) / 930 + 0.5)),
        base_atk: String(atk[level]),
        base_def: String(
          Math.floor(((200 + level * 10) * baseDef) / 210 + 0.5),
        ),
        base_spd: decimal(baseSpd * speedMultiplier(level)),
        effect_hit_rate: decimal(Math.max(0, level - 50) * 0.008),
        effect_resistance: decimal(
          baseEffectResistance +
            Math.min(0.1, Math.max(0, level - 50) * 0.004),
        ),
      })),
    };
  }),
};

const anchorEncoded = `${JSON.stringify(anchor, null, 2)}\n`;
const anchorDigest = digest(Buffer.from(anchorEncoded));
const review = {
  schema_revision: "starclock.goal07-enemy-source-review.v2",
  partition_id: partitionId,
  reviewed_on: "2026-07-29",
  mechanism_target: "ExactPublic",
  numeric_status: "ApprovedPerVariantInputs",
  mechanic_status: "ExecutableMechanismCorrect",
  numeric_policy_ids: [
    "goal07-exact-public-per-level-v1",
    "goal07-normalized-template-level-curve-v1",
  ],
  variants: anchor.variants.map((entry) => ({
    enemy_variant_id: entry.enemy_variant_id,
    source_monster_id: entry.source_monster_id,
    accuracy_disposition: entry.accuracy_disposition,
    numeric_policy_id: entry.numeric_policy_id,
    authored_levels: entry.levels.map(({ authored_level }) => authored_level),
    public_url: entry.public_url,
    numeric_evidence_path: anchorPath,
    numeric_evidence_sha256: anchorDigest,
  })),
  sources: [
    source(abilitiesPath, [
      "source skill identifiers, target hints, elements, ratios and operation tags",
    ]),
    source(templatesPath, [
      "rank, base stats, toughness and retained AI sequence",
    ]),
    source(variantsPath, [
      "weaknesses, resistances and Windspawn Wind Shear immunity",
    ]),
    source(difficultiesPath, [
      "all frozen Standard Universe Trampler difficulty levels",
    ]),
    source(encountersPath, [
      "three frozen encounter members and every referenced encounter level",
    ]),
    {
      path: anchorPath,
      sha256: anchorDigest,
      facts: [
        "exact-public Trampler rows at every frozen authored level",
        "approved normalized template rows for Windspawn and Wraith Warden",
      ],
    },
  ],
  reviewed_runtime_boundaries: [
    "All three variants materialize through production enemy definitions, stats, phases, abilities and AI graphs.",
    "The pre-existing exact Trampler identity is upgraded in place and covered at every S18 encounter and difficulty level.",
    "Trampler retains its five-action cycle, lock marker and 600% Quantum Entanglement strike.",
    "Windspawn retains 250% Wind damage, 80%-ATK Wind Shear for two turns and Wind Shear immunity.",
    "Wraith Warden retains its 250% Physical single-target attack.",
    "No native handler is introduced.",
  ],
};

emit(anchorPath, anchorEncoded);
emit(reviewPath, `${JSON.stringify(review, null, 2)}\n`);
console.log(
  `${write ? "Authored" : "Verified"} Goal 07 S18 numeric anchors and source review.`,
);

function source(relative, facts) {
  return {
    path: relative,
    sha256: digest(fs.readFileSync(absolute(relative))),
    facts,
  };
}
function speedMultiplier(level) {
  if (level >= 86) return 1.32;
  if (level >= 78) return 1.2;
  if (level >= 65) return 1.1;
  return 1;
}
function decimal(value) {
  return String(Math.round(value * 1_000_000) / 1_000_000);
}
function emit(relative, encoded) {
  const target = absolute(relative);
  if (write) {
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, encoded);
  } else {
    assert(
      fs.statSync(target, { throwIfNoEntry: false })?.isFile(),
      `${relative} is missing`,
    );
    assert(fs.readFileSync(target, "utf8") === encoded, `${relative} drifted`);
  }
}
function digest(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}
function json(relative) {
  return JSON.parse(fs.readFileSync(absolute(relative), "utf8"));
}
function absolute(relative) {
  return path.join(root, relative);
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
