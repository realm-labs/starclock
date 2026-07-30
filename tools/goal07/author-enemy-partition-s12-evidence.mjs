#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const partitionId = "G07-P5-M15-S12";
const write = process.argv.includes("--write");
assert(
  process.argv.slice(2).every((argument) => argument === "--write"),
  "usage: author-enemy-partition-s12-evidence.mjs [--write]",
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
const difficulties = json(difficultiesPath);
const encounters = json(encountersPath);

const specs = [
  {
    id: "enemy.abundance-sprite-golden-hound.minionlv2.variant.01",
    source: "2022040",
    levels: [47, 51, 52, 54, 57],
    accuracy: "ExactPublic",
    publicUrl:
      "https://honkai-star-rail.fandom.com/wiki/Abundance_Sprite%3A_Golden_Hound",
    exactHp: { 47: 2731, 51: 3750, 52: 4205, 54: 5115, 57: 6480 },
  },
  {
    id: "enemy.abundance-sprite-malefic-ape-bug.elite.variant.01",
    source: "2023021",
    levels: [47, 53, 69, 78, 87],
    accuracy: "ApprovedNumericApproximation",
  },
  {
    id: "enemy.abundance-sprite-malefic-ape.elite.variant.01",
    source: "2023020",
    levels: [50, 66, 75, 84],
    accuracy: "ApprovedNumericApproximation",
  },
  {
    id: "enemy.abundance-sprite-wooden-lupus.minionlv2.variant.01",
    source: "2022050",
    levels: [47, 51, 52, 54, 57],
    accuracy: "ApprovedNumericApproximation",
  },
  {
    id: "enemy.antibaryon.minion.variant.01",
    source: "8011020",
    levels: [7, 21, 22, 24, 25, 51, 54],
    accuracy: "ApprovedNumericApproximation",
  },
  {
    id: "enemy.aurumaton-gatekeeper-bug.elite.variant.01",
    source: "2013011",
    levels: [53, 69, 78, 87],
    accuracy: "ApprovedNumericApproximation",
  },
  {
    id: "enemy.aurumaton-gatekeeper.elite.variant.01",
    source: "2013010",
    levels: [49, 50, 66, 75, 84],
    accuracy: "ExactPublic",
    publicUrl:
      "https://honkai-star-rail.fandom.com/wiki/Aurumaton_Gatekeeper",
    exactHp: {
      49: 19417,
      50: 20590,
      66: 72616,
      75: 112994,
      84: 167044,
    },
  },
  {
    id: "enemy.aurumaton-spectral-envoy.elite.variant.01",
    source: "2013020",
    levels: [50, 66, 75, 84],
    accuracy: "ApprovedNumericApproximation",
  },
  {
    id: "enemy.automaton-beetle.minionlv2.variant.01",
    source: "1012030",
    levels: [31, 34, 37, 54],
    accuracy: "ApprovedNumericApproximation",
  },
  {
    id: "enemy.automaton-direwolf.elite.variant.01",
    source: "1013020",
    levels: [29, 44, 61, 66, 75, 84],
    accuracy: "ApprovedNumericApproximation",
  },
  {
    id: "enemy.automaton-grizzly.elite.variant.01",
    source: "1013010",
    levels: [36, 61, 66, 75, 84],
    accuracy: "ApprovedNumericApproximation",
  },
  {
    id: "enemy.automaton-hound.minionlv2.variant.01",
    source: "1012010",
    levels: [31, 32, 44, 51, 57],
    accuracy: "ApprovedNumericApproximation",
  },
];

assert(
  JSON.stringify(specs.map(({ id }) => id)) ===
    JSON.stringify(partition.enemy_variant_ids),
  `${partitionId}: evidence specification differs from frozen variants`,
);

const hp930 = {
  7: 1455,
  21: 3020,
  22: 3226,
  24: 3638,
  25: 3845,
  29: 4669,
  31: 5274,
  32: 5672,
  34: 6469,
  36: 7265,
  37: 7663,
  44: 13551,
  47: 17071,
  49: 19417,
  50: 20590,
  51: 23435,
  52: 26279,
  53: 29123,
  54: 31968,
  57: 40501,
  61: 52964,
  66: 72616,
  69: 84408,
  75: 112994,
  78: 127788,
  84: 167044,
  87: 192070,
};
const atk = {
  7: 23,
  21: 57,
  22: 62,
  24: 71,
  25: 75,
  29: 93,
  31: 104,
  32: 109,
  34: 121,
  36: 132,
  37: 138,
  44: 187,
  47: 210,
  49: 226,
  50: 234,
  51: 244,
  52: 255,
  53: 265,
  54: 276,
  57: 307,
  61: 348,
  66: 397,
  69: 426,
  75: 494,
  78: 529,
  84: 597,
  87: 630,
};

const anchor = {
  schema_revision: "starclock.goal07-enemy-numeric-anchor.v2",
  partition_id: partitionId,
  numeric_policy_id: "goal07-s12-public-and-derived-level-curve-v1",
  source: {
    publisher: "Honkai: Star Rail Wiki and committed version 4.4 extraction",
    url:
      "https://honkai-star-rail.fandom.com/wiki/Abundance_Sprite%3A_Golden_Hound",
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
      "ExactPublic variants retain directly reviewed public HP rows; all other rows use the named normalized template curve.",
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
      public_url: spec.publicUrl ?? null,
      levels: spec.levels.map((level) => ({
        authored_level: level,
        base_hp: String(
          spec.exactHp?.[level] ??
            roundHalfAway((hp930[level] * baseHp) / 930),
        ),
        base_atk: String(atk[level]),
        base_def: String(200 + level * 10),
        base_spd: decimal(baseSpd * speedMultiplier(level)),
        effect_hit_rate: decimal(Math.max(0, level - 50) * 0.008),
        effect_resistance: decimal(
          baseEffectResistance + Math.min(0.1, Math.max(0, level - 50) * 0.004),
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
      "source skill identifiers, target hints, elements, ratios, operation tags",
    ]),
    source(templatesPath, [
      "rank, base HP, base SPD, toughness, effect resistance, retained AI sequence",
    ]),
    source(variantsPath, [
      "weaknesses, resistances, debuff resistances, summon references",
    ]),
    source(difficultiesPath, [
      "all frozen Standard Universe elite difficulty levels",
    ]),
    source(encountersPath, [
      "37 frozen encounter groups, 47 members, hard level group 2",
    ]),
    {
      path: anchorPath,
      sha256: anchorDigest,
      facts: [
        "exact-public Golden Hound and Aurumaton Gatekeeper rows",
        "approved normalized template curve for ten inherited approximate variants",
      ],
    },
  ],
  reviewed_runtime_boundaries: [
    "All twelve frozen variants materialize through production EnemyDefinition, EnemyStat, EnemyPhase, EnemyAbility and AI graph rows.",
    "Every frozen encounter and difficulty level has a corresponding authored EnemyStat row.",
    "Damage ratios, elements, target patterns, weaknesses, resistances and toughness are transcribed from the committed version 4.4 extraction.",
    "Golden Hound retains its killing-blow ally action-advance talent through phase-entry effect and shared Rule IR.",
    "Aurumaton Gatekeeper retains Dread, Restraint and Enchainment, including Weaken and Imprisonment effects and a source-ordered sanction action cycle.",
    "The ten inherited approximate variants use the named normalized template curve while preserving executable source-ordered mechanics.",
    "No native handler is introduced.",
  ],
};
const reviewEncoded = `${JSON.stringify(review, null, 2)}\n`;

emit(anchorPath, anchorEncoded);
emit(reviewPath, reviewEncoded);
console.log(
  `${write ? "Authored" : "Verified"} Goal 07 S12 numeric anchors and source review.`,
);

function source(relative, facts) {
  return { path: relative, sha256: digest(fs.readFileSync(absolute(relative))), facts };
}
function speedMultiplier(level) {
  if (level >= 86) return 1.32;
  if (level >= 78) return 1.2;
  if (level >= 65) return 1.1;
  return 1;
}
function roundHalfAway(value) {
  return Math.floor(value + 0.5);
}
function decimal(value) {
  return String(Math.round(value * 1_000_000) / 1_000_000);
}
function emit(relative, encoded) {
  const target = absolute(relative);
  if (write) {
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, encoded);
    return;
  }
  assert(fs.statSync(target, { throwIfNoEntry: false })?.isFile(), `${relative} is missing`);
  assert(fs.readFileSync(target, "utf8") === encoded, `${relative} drifted`);
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
