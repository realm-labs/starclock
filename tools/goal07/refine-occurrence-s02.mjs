#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const write = process.argv.slice(2).includes("--write");
assert(
  process.argv.slice(2).every((argument) => argument === "--write"),
  "usage: node tools/goal07/refine-occurrence-s02.mjs [--write]",
);

const packRoot = path.join(root, "content-reference", "standard-universe-v1");
const choicesPath = path.join(packRoot, "occurrence-choices.json");
const blessingsPath = path.join(packRoot, "blessings.json");
const curiosPath = path.join(packRoot, "curios.json");
const pathsPath = path.join(packRoot, "paths.json");
const indexPath = path.join(packRoot, "pack-index.json");
const reviewPath = path.join(
  root,
  "evidence",
  "standard-universe-mechanics-complete-v1",
  "source-reviews",
  "G07-P4-M13-S02.json",
);
const originalChoices = fs.readFileSync(choicesPath, "utf8");
const choices = JSON.parse(originalChoices);
const blessings = JSON.parse(fs.readFileSync(blessingsPath, "utf8"));
const curios = JSON.parse(fs.readFileSync(curiosPath, "utf8"));
const paths = JSON.parse(fs.readFileSync(pathsPath, "utf8"));
const byId = new Map(choices.map((choice) => [choice.id, choice]));

const blessingRefs = (rarity) =>
  blessings
    .filter((blessing) => blessing.rarity === rarity)
    .map((blessing) => blessing.id)
    .sort(compare);
const curioRefs = (polarity) =>
  curios
    .filter((curio) => curio.pool_tags.includes(`polarity:${polarity}`))
    .map((curio) => curio.id)
    .sort(compare);
const pathRefs = paths.map((entry) => entry.id).sort(compare);
const enemyRefs = [
  "enemy.trot-deuce.minionlv2.variant.01",
  "enemy.trot-prime.minionlv2.variant.01",
  "enemy.trot-tri.minionlv2.variant.01",
].sort(compare);

assert(blessingRefs(2).length === 63, "expected 63 two-star Blessings");
assert(blessingRefs(3).length === 27, "expected 27 three-star Blessings");
assert(curioRefs("positive").length === 46, "expected 46 normal Curios");
assert(curioRefs("negative").length === 15, "expected 15 negative Curios");
assert(pathRefs.length === 9, "expected nine Standard Universe paths");

setOutcome("universe.occurrence.12.variant.10901.choice.07", {
  kinds: ["Obtain"],
  targets: ["CosmicFragments"],
  numeric_literals: ["100"],
});
setOutcome("universe.occurrence.12.variant.10901.choice.08", {
  kinds: ["Obtain"],
  targets: ["Curio"],
  numeric_literals: ["1"],
  parameter_refs: ["universe.curio.60"],
});
setOutcome("universe.occurrence.12.variant.10901.choice.09", {
  kinds: ["Obtain"],
  targets: ["CosmicFragments"],
  numeric_literals: ["100"],
});

setOutcome("universe.occurrence.13.variant.11001.choice.01", {
  kinds: ["Obtain", "Lose"],
  targets: ["Blessing", "HP"],
  numeric_literals: ["1", "30%"],
  parameter_refs: blessingRefs(2),
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.13.variant.11001.choice.02", {
  kinds: ["Obtain", "Lose"],
  targets: ["Blessing", "HP"],
  numeric_literals: ["1", "80%"],
  parameter_refs: blessingRefs(3),
  unspecified_random_policy: "StableUniformOrderedCandidates",
});

setTransition("universe.occurrence.14.variant.11101.choice.01");
setTransition("universe.occurrence.14.variant.11101.choice.02");
setOutcome("universe.occurrence.14.variant.11101.choice.03", {
  kinds: ["Obtain", "Obtain"],
  targets: ["Blessing", "Curio"],
  numeric_literals: ["1", "1"],
  parameter_refs: [...blessingRefs(3), "universe.curio.59"],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.14.variant.11101.choice.04", {
  kinds: ["Obtain", "Lose"],
  targets: ["Blessing", "HP"],
  numeric_literals: ["1", "50%"],
  parameter_refs: blessingRefs(2),
  unspecified_random_policy: "StableUniformOrderedCandidates",
});

setOutcome("universe.occurrence.15.variant.11201.choice.01", {
  kinds: ["Obtain"],
  targets: ["Blessing"],
  parameter_refs: pathRefs,
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.15.variant.11201.choice.02", {
  kinds: ["Obtain"],
  targets: ["CosmicFragments"],
  numeric_literals: ["2000"],
});
setOutcome("universe.occurrence.15.variant.11201.choice.03", {
  kinds: ["Obtain", "Obtain"],
  targets: ["Blessing", "CosmicFragments"],
  numeric_literals: ["1", "2000"],
  parameter_refs: pathRefs,
  unspecified_random_policy: "StableUniformOrderedCandidates",
});

setOutcome("universe.occurrence.16.variant.11301.choice.01", {
  kinds: ["Battle"],
  targets: [],
  numeric_literals: ["48"],
  parameter_refs: [
    ...enemyRefs,
    "universe.occurrence-battle.reward.defeated-enemy-blessing",
    "universe.occurrence-battle.stage.80212011",
  ],
});
setTransition("universe.occurrence.16.variant.11301.choice.02");

setOutcome("universe.occurrence.17.variant.11401.choice.01", {
  kinds: ["Obtain"],
  targets: ["Curio"],
  numeric_literals: ["1"],
  parameter_refs: curioRefs("negative"),
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setTransition("universe.occurrence.17.variant.11401.choice.02");

setOutcome("universe.occurrence.18.variant.11501.choice.01", {
  kinds: ["Obtain"],
  targets: ["Curio"],
  numeric_literals: ["1"],
  parameter_refs: curioRefs("positive"),
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.18.variant.11501.choice.02", {
  kinds: ["Obtain"],
  targets: ["CosmicFragments"],
  numeric_literals: ["150"],
});

const sourceReview = {
  schema_revision: "starclock.goal07-occurrence-source-review.v1",
  partition_id: "G07-P4-M13-S02",
  reviewed_on: "2026-07-27",
  frozen_source_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  sources: [
    {
      url: "turnbasedgamedata/Config/Level/Rogue/RogueDialogue/Event0010902/Opt0010902.json",
      section: "OnSelect option values",
      evidence_sha256:
        "2341a600454980e2fc76bfbbb97ab81cdeb3443e107548f970461cec530b01d5",
      facts: ["named-curio:universe.curio.60", "cosmic-fragments:100"],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Statue",
      section: "Possible Outcomes",
      evidence_sha256:
        "ceb0484fe896bce8542935c2ba2c7c71b8908c8a48d15fd9c1866f0e17937e3d",
      facts: ["two-star-blessing:1:current-hp-loss:30%", "three-star-blessing:1:current-hp-loss:80%"],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Insect_Nest",
      section: "Possible Outcomes",
      evidence_sha256:
        "68b04363f24c0e352a0722a4fc71fb694c31a79abf3ae3086680e8111af96a04",
      facts: [
        "negative-curio:universe.curio.59:three-star-blessing:1",
        "two-star-blessing:1:current-hp-loss:50%",
      ],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Ruan_Mei_(Occurrence)",
      section: "Possible Outcomes",
      evidence_sha256:
        "3cb0107e4a1ee05eace7a29f219ca77c015d2e92052bd06a52bcc6ae6c09b347",
      facts: ["all-blessings:one-uniform-path", "cosmic-fragments:2000"],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Three_Little_Pigs",
      section: "Possible Outcomes",
      evidence_sha256:
        "af08d1346adede1e0a52a54aad890a98d67f46a23c4fdc1f83ba757e1dc2630d",
      facts: [
        "stage:80212011:level:48",
        "enemies:trot-prime,trot-deuce,trot-tri",
        "reward:one-blessing-per-defeated-trotter",
      ],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Unending_Darkness",
      section: "Possible Outcomes",
      evidence_sha256:
        "7c09ab434ab6d5159168e38079030a8be05c5212cd7fce753fe5c20622505c4d",
      facts: ["random-negative-curio:1"],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/The_Architects",
      section: "Possible Outcomes",
      evidence_sha256:
        "b78a2311f9906379fd936874c71f28a320fed0f6fccc8b3f29b48c2cde395fac",
      facts: ["random-normal-curio:1", "cosmic-fragments:150"],
    },
  ],
};

const encodedChoices = `${JSON.stringify(choices, null, 2)}\n`;
const encodedReview = `${JSON.stringify(sourceReview, null, 2)}\n`;
if (write) {
  fs.writeFileSync(choicesPath, encodedChoices);
  fs.mkdirSync(path.dirname(reviewPath), { recursive: true });
  fs.writeFileSync(reviewPath, encodedReview);
  fs.writeFileSync(indexPath, `${JSON.stringify(buildIndex(), null, 2)}\n`);
  console.log("Refined Goal 07 Occurrence S02 source evidence and pack index.");
} else {
  assert(originalChoices === encodedChoices, "Occurrence S02 normalized choices drifted");
  assert(
    fs.readFileSync(reviewPath, "utf8") === encodedReview,
    "Occurrence S02 source review drifted",
  );
  assert(
    fs.readFileSync(indexPath, "utf8") === `${JSON.stringify(buildIndex(), null, 2)}\n`,
    "Occurrence S02 pack index drifted",
  );
  console.log("Goal 07 Occurrence S02 source refinement is stable.");
}

function setTransition(id) {
  setOutcome(id, { kinds: ["Special"], targets: [] });
}

function setOutcome(id, patch) {
  const choice = required(id);
  choice.costs = [];
  choice.outcomes[0] = {
    kinds: patch.kinds,
    targets: patch.targets,
    numeric_literals: patch.numeric_literals ?? [],
    parameter_refs: patch.parameter_refs ?? [],
    chance_percentages: [],
    unspecified_random_policy: patch.unspecified_random_policy ?? "",
  };
  choice.note =
    "Exact public option/result evidence is lowered to executable Standard Universe state changes; ordered candidate references freeze every random pool.";
}

function required(id) {
  const value = byId.get(id);
  assert(value, `${id}: normalized choice is missing`);
  assert(value.outcomes?.length === 1, `${id}: expected one normalized outcome`);
  return value;
}

function buildIndex() {
  const existing = JSON.parse(fs.readFileSync(indexPath, "utf8"));
  const files = existing.files
    .map(({ file, rows }) => {
      const bytes = fs.readFileSync(path.join(packRoot, file));
      const parsed = JSON.parse(bytes);
      return {
        file,
        bytes: bytes.length,
        rows: Array.isArray(parsed) ? parsed.length : rows,
        sha256: sha256(bytes),
      };
    })
    .sort((left, right) => compare(left.file, right.file));
  return {
    schema: existing.schema,
    files,
    pack_sha256: sha256(
      Buffer.from(
        files.map(({ file, sha256: digest }) => `${file}\0${digest}`).join("\n"),
      ),
    ),
  };
}

function compare(left, right) {
  return left.localeCompare(right, "en");
}

function sha256(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
