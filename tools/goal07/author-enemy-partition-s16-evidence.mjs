#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const partitionId = "G07-P5-M15-S16";
const write = process.argv.includes("--write");
assert(
  process.argv.slice(2).every((argument) => argument === "--write"),
  "usage: author-enemy-partition-s16-evidence.mjs [--write]",
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
  [
    "enemy.memory-zone-meme-shell-of-faded-rage.elite.variant.01",
    "3013010",
    [44, 50, 66, 75, 84],
    "https://honkai-star-rail.fandom.com/wiki/Memory_Zone_Meme_%22Shell_of_Faded_Rage%22",
  ],
  [
    "enemy.memory-zone-meme-something-in-the-mirror.minionlv2.variant.02",
    "301201001",
    [54],
    "https://honkai-star-rail.fandom.com/wiki/Memory_Zone_Meme_%22Something_In_The_Mirror%22",
  ],
  [
    "enemy.searing-prowler.elite.variant.01",
    "1023010",
    [21, 29, 44, 61, 66, 75, 84],
    "https://honkai-star-rail.fandom.com/wiki/Searing_Prowler",
  ],
  [
    "enemy.senior-staff-team-leader-bug.elite.variant.01",
    "8033011",
    [47, 53, 69, 78, 87],
    "https://honkai-star-rail.fandom.com/wiki/Senior_Staff:_Team_Leader_(Bug)",
  ],
  [
    "enemy.senior-staff-team-leader.elite.variant.01",
    "8033010",
    [50, 66, 75, 84],
    "https://honkai-star-rail.fandom.com/wiki/Senior_Staff:_Team_Leader",
  ],
  [
    "enemy.silvermane-cannoneer.minionlv2.variant.01",
    "1002030",
    [47, 55, 57],
    "https://honkai-star-rail.fandom.com/wiki/Silvermane_Cannoneer",
  ],
  [
    "enemy.silvermane-gunner.minionlv2.variant.01",
    "1002050",
    [47, 55, 57],
    "https://honkai-star-rail.fandom.com/wiki/Silvermane_Gunner",
  ],
  [
    "enemy.silvermane-lieutenant-bug.elite.variant.01",
    "1003011",
    [32, 53, 64, 69, 78, 87],
    "https://honkai-star-rail.fandom.com/wiki/Silvermane_Lieutenant_(Bug)",
  ],
  [
    "enemy.silvermane-soldier.minionlv2.variant.01",
    "1002040",
    [54],
    "https://honkai-star-rail.fandom.com/wiki/Silvermane_Soldier",
  ],
  [
    "enemy.stormbringer-bug.elite.variant.01",
    "8003051",
    [24, 39, 64, 69, 78, 87],
    "https://honkai-star-rail.fandom.com/wiki/Stormbringer_(Bug)",
  ],
  [
    "enemy.stormbringer.elite.variant.01",
    "8003050",
    [50, 66, 75, 84],
    "https://honkai-star-rail.fandom.com/wiki/Stormbringer",
  ],
  [
    "enemy.the-ascended.elite.variant.01",
    "2023030",
    [50, 66, 75, 84],
    "https://honkai-star-rail.fandom.com/wiki/The_Ascended",
  ],
].map(([id, source, levels, publicUrl]) => ({
  id,
  source,
  levels,
  accuracy: "ExactPublic",
  publicUrl,
}));

assert(
  JSON.stringify(specs.map(({ id }) => id)) ===
    JSON.stringify(partition.enemy_variant_ids),
  `${partitionId}: evidence specification differs from frozen variants`,
);

const hp930 = {
  21: 3020,
  24: 3638,
  29: 4669,
  32: 5672,
  39: 8460,
  44: 13551,
  47: 17071,
  50: 20590,
  53: 29123,
  54: 31968,
  55: 34812,
  57: 40501,
  61: 52964,
  64: 64755,
  66: 72616,
  69: 84408,
  75: 112994,
  78: 127788,
  84: 167044,
  87: 192070,
};
const atk = {
  21: 57,
  24: 71,
  29: 93,
  32: 109,
  39: 150,
  44: 187,
  47: 210,
  50: 234,
  53: 265,
  54: 276,
  55: 286,
  57: 307,
  61: 348,
  64: 377,
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
  numeric_policy_id: "goal07-s16-public-level-curve-v1",
  source: {
    publisher: "Honkai: Star Rail Wiki and committed version 4.4 extraction",
    url: "https://honkai-star-rail.fandom.com/wiki/Silvermane_Lieutenant",
    accessed_on: "2026-07-29",
    game_version: "4.4",
    confidence: "PerVariantExactPublicCurve",
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
      "All variants use the reviewed public monster curve scaled by their committed version 4.4 template base.",
  },
  variants: specs.map((spec) => {
    const variant = variants.find(
      ({ source_monster_id }) => String(source_monster_id) === spec.source,
    );
    const template = templates.find(({ id }) => id === variant?.enemy_id);
    assert(template && variant, `${spec.id}: retained source definition is missing`);
    const baseHp = Number(template.base_stats.hp);
    const baseSpd = Number(template.base_stats.spd);
    const baseEffectResistance = Number(template.base_stats.effect_res ?? 0);
    return {
      enemy_variant_id: spec.id,
      source_monster_id: spec.source,
      combat_rank: template.rank,
      accuracy_disposition: spec.accuracy,
      numeric_policy_id: "goal07-exact-public-per-level-v1",
      public_url: spec.publicUrl,
      levels: spec.levels.map((level) => ({
        authored_level: level,
        base_hp: String(Math.floor((hp930[level] * baseHp) / 930 + 0.5)),
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
  numeric_policy_ids: ["goal07-exact-public-per-level-v1"],
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
    source(abilitiesPath, ["source skill identifiers, target hints, elements, ratios and operation tags"]),
    source(templatesPath, ["rank, base stats, toughness and retained AI sequence"]),
    source(variantsPath, ["weaknesses, resistances, debuff resistance and summon references"]),
    source(difficultiesPath, ["all frozen Standard Universe elite difficulty levels"]),
    source(encountersPath, ["3 frozen encounter members and every referenced encounter level"]),
    {
      path: anchorPath,
      sha256: anchorDigest,
      facts: [
        "public monster HP, ATK, DEF, SPD, effect-hit and effect-resistance level curve",
        "per-variant version 4.4 template bases and every frozen authored level",
      ],
    },
  ],
  reviewed_runtime_boundaries: [
    "All twelve variants materialize through production enemy definitions, stats, phases, abilities and AI graphs.",
    "Every frozen encounter and difficulty level has a corresponding authored EnemyStat row.",
    "Damage ratios, elements, targets, weaknesses, resistances and toughness follow the committed version 4.4 extraction.",
    "Searing Prowler retains Burn immunity, Melt, High-Temperature Operation and its Fire attack cycle.",
    "Senior Staff Team Leaders retain Performance Boost, training state and formation-linked personnel summons.",
    "Silvermane Cannoneer retains 130% plus 100% blast Barrage and 220% plus 150% Covering Support follow-up semantics.",
    "Silvermane Lieutenant retains Reinforcement, Rallying and damage-triggered 600% Shield Reflect.",
    "Stormbringer variants retain Wind Shear, Wind-Twisting Crossbow and Storm Cyclone state boundaries.",
    "The Ascended retains its charge into Black Prana's Snare and Wind Shear attacks.",
    "The two Memory Zone Memes retain rage, shell and mirror-transformation boundaries.",
    "No native handler is introduced.",
  ],
};

emit(anchorPath, anchorEncoded);
emit(reviewPath, `${JSON.stringify(review, null, 2)}\n`);
console.log(`${write ? "Authored" : "Verified"} Goal 07 S16 numeric anchors and source review.`);

function source(relative, facts) {
  return { path: relative, sha256: digest(fs.readFileSync(absolute(relative))), facts };
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
    assert(fs.statSync(target, { throwIfNoEntry: false })?.isFile(), `${relative} is missing`);
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
