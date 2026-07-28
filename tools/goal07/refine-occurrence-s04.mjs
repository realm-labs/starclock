#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const write = process.argv.slice(2).includes("--write");
assert(
  process.argv.slice(2).every((argument) => argument === "--write"),
  "usage: node tools/goal07/refine-occurrence-s04.mjs [--write]",
);

const packRoot = path.join(root, "content-reference", "standard-universe-v1");
const choicesPath = path.join(packRoot, "occurrence-choices.json");
const indexPath = path.join(packRoot, "pack-index.json");
const reviewPath = path.join(
  root,
  "evidence",
  "standard-universe-mechanics-complete-v1",
  "source-reviews",
  "G07-P4-M13-S04.json",
);
const originalChoices = fs.readFileSync(choicesPath, "utf8");
const choices = JSON.parse(originalChoices);
const blessings = json("blessings.json");
const curios = json("curios.json");
const byId = new Map(choices.map((choice) => [choice.id, choice]));

const curioRefs = (polarity) =>
  curios
    .filter((curio) => curio.pool_tags.includes(`polarity:${polarity}`))
    .map((curio) => curio.id)
    .sort(compare);
const blessingPool = {
  all: "universe.blessing-pool.all",
  twoStar: "universe.blessing-pool.rarity.2",
  threeStar: "universe.blessing-pool.rarity.3",
};

assert(
  blessings.filter((blessing) => blessing.rarity === 2).length === 63,
  "expected 63 two-star Blessings",
);
assert(curioRefs("positive").length === 46, "expected 46 normal Curios");
assert(curioRefs("negative").length === 15, "expected 15 negative Curios");
assert(curios.length === 61, "expected 61 complete Curios");

const cosmicDefinitions = [
  { kinds: ["Special"], targets: [] },
  {
    kinds: ["Obtain", "Consume"],
    targets: ["Curio", "CosmicFragments"],
    numeric_literals: ["1", "100"],
    parameter_refs: curioRefs("positive"),
    unspecified_random_policy: "StableUniformOrderedCandidates",
  },
  {
    kinds: ["Obtain", "Consume"],
    targets: ["Blessing", "CosmicFragments"],
    numeric_literals: ["1", "100"],
    parameter_refs: [blessingPool.all],
    unspecified_random_policy: "StableUniformOrderedCandidates",
  },
  { kinds: ["Special"], targets: [] },
  {
    kinds: ["Enhance", "Consume"],
    targets: ["Blessing", "CosmicFragments"],
    numeric_literals: ["3", "10"],
    parameter_refs: [blessingPool.all],
    unspecified_random_policy: "StableUniformOrderedCandidates",
  },
  {
    kinds: ["Obtain", "Consume"],
    targets: ["Blessing", "CosmicFragments"],
    numeric_literals: ["1", "10"],
    parameter_refs: [blessingPool.threeStar],
    unspecified_random_policy: "StableUniformOrderedCandidates",
  },
  { kinds: ["Special"], targets: [] },
];
for (let index = 0; index < cosmicDefinitions.length; index += 1) {
  setOutcome(
    `universe.occurrence.22.variant.11701.choice.${String(index + 3).padStart(2, "0")}`,
    cosmicDefinitions[index],
  );
}

setOutcome("universe.occurrence.23.variant.11801.choice.01", {
  kinds: ["Obtain", "Obtain"],
  targets: ["CosmicFragments", "Curio"],
  numeric_literals: ["300", "1"],
  parameter_refs: curioRefs("negative"),
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.23.variant.11801.choice.02", {
  kinds: ["Obtain"],
  targets: ["CosmicFragments"],
  numeric_literals: ["100"],
});

const sal = {
  kinds: ["Obtain", "Lose", "Special"],
  targets: ["Curio", "HP", "Character"],
  numeric_literals: ["1", "20%"],
  parameter_refs: curioRefs("positive"),
  unspecified_random_policy: "StableUniformOrderedCandidates",
};
const leo = {
  kinds: ["Discard", "Obtain", "Special"],
  targets: ["Curio", "Blessing", "Character"],
  numeric_literals: ["1", "1"],
  parameter_refs: [blessingPool.twoStar],
  unspecified_random_policy: "StableUniformOrderedCandidates",
};
const reset = {
  kinds: ["Obtain", "Special"],
  targets: ["CosmicFragments", "Character"],
  numeric_literals: ["100"],
};
const saleoDefinitions = [sal, leo, sal, reset, leo, reset];
for (const occurrence of [24, 25]) {
  for (let index = 0; index < saleoDefinitions.length; index += 1) {
    setOutcome(
      `universe.occurrence.${occurrence}.variant.11901.choice.${String(index + 1).padStart(2, "0")}`,
      saleoDefinitions[index],
    );
  }
}
for (let index = 0; index < 3; index += 1) {
  setOutcome(
    `universe.occurrence.26.variant.11901.choice.${String(index + 1).padStart(2, "0")}`,
    saleoDefinitions[index],
  );
}

const sourceReview = {
  schema_revision: "starclock.goal07-occurrence-source-review.v1",
  partition_id: "G07-P4-M13-S04",
  reviewed_on: "2026-07-28",
  frozen_source_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  sources: [
    {
      url: "https://game8.co/games/Honkai-Star-Rail/archives/477632",
      section: "Societal Dreamscape Possible Outcomes",
      evidence_sha256:
        "4bd0fb506bc2c82a36bcefd6178cab263dcc9f428dda8246add548575f2f5346",
      facts: [
        "negative-curio:1:cosmic-fragments:300",
        "cosmic-fragments:100",
      ],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Saleo_%28I%29",
      section: "Possible Outcomes — Simulated Universe",
      evidence_sha256:
        "400728699e11a9271aca5b4cd323d1c200fc6e4c17ed83c05032970fe4ead82f",
      facts: [
        "normal-curio:1:current-hp-loss:20-percent:transition:sal-ii",
        "discard-curio:1:two-star-blessing:1:transition:leo-iii",
      ],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Sal_%28II%29",
      section: "Possible Outcomes — Simulated Universe",
      evidence_sha256:
        "48087d9e930f7180d9e3c16ce5bc874e308683f2b444ca76b263434e6fcf604e",
      facts: [
        "normal-curio:1:current-hp-loss:20-percent",
        "cosmic-fragments:100:transition:saleo-i",
      ],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Leo_%28III%29",
      section: "Possible Outcomes — Simulated Universe",
      evidence_sha256:
        "f596a721de7cd42e7ec37e3e9a3ec52d1553c851ea7a1c1f7ae8a029cd507bb5",
      facts: [
        "discard-curio:1:two-star-blessing:1",
        "cosmic-fragments:100:transition:saleo-i",
      ],
    },
    {
      url: "https://raw.githubusercontent.com/Dimbreath/turnbasedgamedata/fd978d6ef09f941fba644c731ab54abd6f7c3568/Config/Level/Rogue/RogueDialogue/Event0011801/Opt0011801.json",
      section: "OptionList",
      evidence_sha256:
        "6db3f3150dfed35878c642a95ee4984e5188a850284fd7ea67648d9560cdedaa",
      facts: ["option-11801-desc-value:300", "option-11802-desc-value:100"],
    },
    {
      url: "https://raw.githubusercontent.com/Dimbreath/turnbasedgamedata/fd978d6ef09f941fba644c731ab54abd6f7c3568/Config/Level/Rogue/RogueDialogue/Event0011901/Opt0011901.json",
      section: "OptionList",
      evidence_sha256:
        "a5868b50e81bfcd23ea3854b02203ea0e6c5a6acefc5554b1b402d4f2d23b217",
      facts: ["option-11901-desc-value:20"],
    },
  ],
};

const encodedChoices = `${JSON.stringify(choices, null, 2)}\n`;
const encodedReview = `${JSON.stringify(sourceReview, null, 2)}\n`;
const encodedIndex = `${JSON.stringify(buildIndex(), null, 2)}\n`;
if (write) {
  fs.writeFileSync(choicesPath, encodedChoices);
  fs.mkdirSync(path.dirname(reviewPath), { recursive: true });
  fs.writeFileSync(reviewPath, encodedReview);
  fs.writeFileSync(indexPath, encodedIndex);
  console.log("Refined Goal 07 Occurrence S04 source evidence and pack index.");
} else {
  assert(originalChoices === encodedChoices, "Occurrence S04 normalized choices drifted");
  assert(fs.readFileSync(reviewPath, "utf8") === encodedReview, "Occurrence S04 source review drifted");
  assert(fs.readFileSync(indexPath, "utf8") === encodedIndex, "Occurrence S04 pack index drifted");
  console.log("Goal 07 Occurrence S04 source refinement is stable.");
}

function setOutcome(id, patch) {
  const choice = byId.get(id);
  assert(choice, `missing ${id}`);
  choice.costs = [];
  choice.outcomes[0] = {
    kinds: patch.kinds,
    targets: patch.targets,
    numeric_literals: patch.numeric_literals ?? [],
    parameter_refs: patch.parameter_refs ?? [],
    chance_percentages: [],
    unspecified_random_policy: patch.unspecified_random_policy ?? "",
  };
}

function buildIndex() {
  const existing = JSON.parse(fs.readFileSync(indexPath, "utf8"));
  const files = existing.files
    .map(({ file, rows }) => {
      const bytes =
        file === "occurrence-choices.json"
          ? Buffer.from(encodedChoices)
          : fs.readFileSync(path.join(packRoot, file));
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
function json(name) {
  return JSON.parse(fs.readFileSync(path.join(packRoot, name), "utf8"));
}
function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}
function compare(left, right) {
  return left.localeCompare(right);
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
