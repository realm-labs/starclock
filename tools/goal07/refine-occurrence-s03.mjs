#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const write = process.argv.slice(2).includes("--write");
assert(
  process.argv.slice(2).every((argument) => argument === "--write"),
  "usage: node tools/goal07/refine-occurrence-s03.mjs [--write]",
);

const packRoot = path.join(root, "content-reference", "standard-universe-v1");
const choicesPath = path.join(packRoot, "occurrence-choices.json");
const indexPath = path.join(packRoot, "pack-index.json");
const reviewPath = path.join(
  root,
  "evidence",
  "standard-universe-mechanics-complete-v1",
  "source-reviews",
  "G07-P4-M13-S03.json",
);
const originalChoices = fs.readFileSync(choicesPath, "utf8");
const choices = JSON.parse(originalChoices);
const blessings = json("blessings.json");
const curios = json("curios.json");
const byId = new Map(choices.map((choice) => [choice.id, choice]));

const blessingRefs = (rarity) =>
  blessings
    .filter((blessing) => rarity === undefined || blessing.rarity === rarity)
    .map((blessing) => blessing.id)
    .sort(compare);
const preservationTwoStar = blessings
  .filter(
    (blessing) =>
      blessing.rarity === 2 && blessing.path_id === "universe.path.preservation",
  )
  .map((blessing) => blessing.id)
  .sort(compare);
const curioRefs = (polarity) =>
  curios
    .filter((curio) => curio.pool_tags.includes(`polarity:${polarity}`))
    .map((curio) => curio.id)
    .sort(compare);
const blessingPool = {
  all: "universe.blessing-pool.all",
  oneStar: "universe.blessing-pool.rarity.1",
  threeStar: "universe.blessing-pool.rarity.3",
  preservationTwoStar: "universe.blessing-pool.path.preservation.rarity.2",
};

assert(blessingRefs().length === 162, "expected 162 Blessings");
assert(blessingRefs(1).length === 72, "expected 72 one-star Blessings");
assert(blessingRefs(3).length === 27, "expected 27 three-star Blessings");
assert(preservationTwoStar.length === 7, "expected seven two-star Preservation Blessings");
assert(curioRefs("positive").length === 46, "expected 46 normal Curios");
assert(curioRefs("negative").length === 15, "expected 15 negative Curios");

setOutcome("universe.occurrence.2.variant.10101.choice.01", {
  kinds: ["Enhance"],
  targets: ["Blessing"],
  numeric_literals: ["2"],
  parameter_refs: [blessingPool.all],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.2.variant.10101.choice.02", {
  kinds: ["Obtain"],
  targets: ["Blessing"],
  numeric_literals: ["1"],
  parameter_refs: [blessingPool.preservationTwoStar],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.19.variant.11601.choice.01", {
  kinds: ["Obtain", "Obtain"],
  targets: ["Blessing", "Curio"],
  numeric_literals: ["1", "1"],
  parameter_refs: [blessingPool.threeStar, ...curioRefs("negative")],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.19.variant.11601.choice.02", {
  kinds: ["Obtain"],
  targets: ["CosmicFragments"],
  numeric_literals: ["100"],
});

for (const occurrence of [20, 21]) {
  setCosmicGraph(occurrence, 9);
}
setCosmicGraph(22, 2);

const sourceReview = {
  schema_revision: "starclock.goal07-occurrence-source-review.v1",
  partition_id: "G07-P4-M13-S03",
  reviewed_on: "2026-07-28",
  frozen_source_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  sources: [
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Nomadic_Miners_(Occurrence)",
      section: "Possible Outcomes — Simulated Universe",
      evidence_sha256:
        "dfca95c82dc65feaaa22a4df450541221d350d4b9a2895cac47c780c3cb5d782",
      facts: ["enhance-random-blessings:2", "preservation-two-star-blessing:1"],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Kindling_of_the_Self-Annihilator",
      section: "Possible Outcomes — Simulated Universe",
      evidence_sha256:
        "bc734c63a07794fb9676959cc3ac9ce0affeca5bf3669a6105833892762c9f63",
      facts: ["three-star-blessing:1:negative-curio:1", "cosmic-fragments:100"],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Cosmic_Merchant_(I)",
      section: "Possible Outcomes — Simulated Universe",
      evidence_sha256:
        "d265d01bb0d25146f6e47ac9d2ce26f33df95cf5c4ea0a38e7081344ea0f5703",
      facts: ["one-star-blessing:1:cost:100", "negative-curio:1:cost:200"],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Cosmic_Con_Job_(II)",
      section: "Possible Outcomes — Simulated Universe",
      evidence_sha256:
        "2247914285c9c440e5ec26053ff17bf506bec6e57afb7558077bccee9d0947a1",
      facts: ["normal-curio:1:cost:100", "one-to-three-star-blessing:1:cost:100"],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Cosmic_Altruist_(III)",
      section: "Possible Outcomes — Simulated Universe",
      evidence_sha256:
        "de971d6ec2b9637f84288a98ecf552c796520757b661e868525b6db05562833f",
      facts: ["enhance-random-blessings:3:cost:10", "three-star-blessing:1:cost:10"],
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
  console.log("Refined Goal 07 Occurrence S03 source evidence and pack index.");
} else {
  assert(originalChoices === encodedChoices, "Occurrence S03 normalized choices drifted");
  assert(fs.readFileSync(reviewPath, "utf8") === encodedReview, "Occurrence S03 source review drifted");
  assert(fs.readFileSync(indexPath, "utf8") === encodedIndex, "Occurrence S03 pack index drifted");
  console.log("Goal 07 Occurrence S03 source refinement is stable.");
}

function setCosmicGraph(occurrence, count) {
  const prefix = `universe.occurrence.${occurrence}.variant.11701.choice.`;
  const definitions = [
    {
      kinds: ["Obtain", "Consume"],
      targets: ["Blessing", "CosmicFragments"],
      numeric_literals: ["1", "100"],
      parameter_refs: [blessingPool.oneStar],
      unspecified_random_policy: "StableUniformOrderedCandidates",
    },
    {
      kinds: ["Obtain", "Consume"],
      targets: ["Curio", "CosmicFragments"],
      numeric_literals: ["1", "200"],
      parameter_refs: curioRefs("negative"),
      unspecified_random_policy: "StableUniformOrderedCandidates",
    },
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
  for (let index = 0; index < count; index += 1) {
    setOutcome(`${prefix}${String(index + 1).padStart(2, "0")}`, definitions[index]);
  }
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
