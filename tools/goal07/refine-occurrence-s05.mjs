#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const write = process.argv.slice(2).includes("--write");
assert(
  process.argv.slice(2).every((argument) => argument === "--write"),
  "usage: node tools/goal07/refine-occurrence-s05.mjs [--write]",
);

const packRoot = path.join(root, "content-reference", "standard-universe-v1");
const choicesPath = path.join(packRoot, "occurrence-choices.json");
const indexPath = path.join(packRoot, "pack-index.json");
const reviewPath = path.join(
  root,
  "evidence",
  "standard-universe-mechanics-complete-v1",
  "source-reviews",
  "G07-P4-M13-S05.json",
);
const originalChoices = fs.readFileSync(choicesPath, "utf8");
const choices = JSON.parse(originalChoices);
const curios = json("curios.json");
const paths = json("paths.json");
const byId = new Map(choices.map((choice) => [choice.id, choice]));

const pathRefs = paths
  .map((value) => ({
    id: value.id,
    display: Number(value.source_ids.find((source) => Number(source) >= 120)),
  }))
  .sort((left, right) => left.display - right.display)
  .map((value) => value.id);
const errorCodeRefs = curios
  .filter(
    (curio) =>
      curio.name_en.endsWith(" Code") &&
      curio.state_ids.some((state) => state.endsWith(".repairing")),
  )
  .map((curio) => curio.id)
  .sort(compare);
const blessingPool = {
  all: "universe.blessing-pool.all",
  oneStar: "universe.blessing-pool.rarity.1",
  twoStar: "universe.blessing-pool.rarity.2",
  threeStar: "universe.blessing-pool.rarity.3",
};
const progressive = "universe.occurrence-progressive.key.13501";

assert(pathRefs.length === 9, "expected nine source-ordered Paths");
assert(errorCodeRefs.length === 6, "expected six repairable Error Code Curios");

const history = [
  [3, blessingPool.oneStar],
  [2, blessingPool.twoStar],
  [1, blessingPool.threeStar],
];
for (const offset of [0, 4]) {
  for (let index = 0; index < history.length; index += 1) {
    const [quantity, pool] = history[index];
    setOutcome(
      `universe.occurrence.3.variant.10201.choice.${String(offset + index + 1).padStart(2, "0")}`,
      {
        kinds: ["Enhance"],
        targets: ["Blessing"],
        numeric_literals: [String(quantity)],
        parameter_refs: [...pathRefs, pool],
        unspecified_random_policy: "StableUniformOrderedCandidates",
      },
    );
  }
  setOutcome(
    `universe.occurrence.3.variant.10201.choice.${String(offset + 4).padStart(2, "0")}`,
    { kinds: ["Special"], targets: [] },
  );
}

const reset = {
  kinds: ["Obtain", "Special"],
  targets: ["CosmicFragments", "Character"],
  numeric_literals: ["100"],
};
const leo = {
  kinds: ["Discard", "Obtain", "Special"],
  targets: ["Curio", "Blessing", "Character"],
  numeric_literals: ["1", "1"],
  parameter_refs: [blessingPool.twoStar],
  unspecified_random_policy: "StableUniformOrderedCandidates",
};
setOutcome("universe.occurrence.26.variant.11901.choice.04", reset);
setOutcome("universe.occurrence.26.variant.11901.choice.05", leo);
setOutcome("universe.occurrence.26.variant.11901.choice.06", reset);

setOutcome("universe.occurrence.27.variant.12001.choice.01", {
  kinds: ["Discard", "Obtain"],
  targets: ["Curio", "CosmicFragments"],
  numeric_literals: ["1", "200"],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.27.variant.12001.choice.02", {
  kinds: ["Special"],
  targets: [],
});

setOutcome("universe.occurrence.28.variant.12101.choice.01", {
  kinds: ["Obtain"],
  targets: ["Curio"],
  numeric_literals: ["1"],
  parameter_refs: errorCodeRefs,
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.28.variant.12101.choice.02", {
  kinds: ["Special"],
  targets: [],
});

for (const variant of [13301, 13302]) {
  setOutcome(`universe.occurrence.29.variant.${variant}.choice.01`, {
    kinds: ["Lose"],
    targets: ["CosmicFragments"],
    numeric_literals: ["50%"],
  });
  setOutcome(`universe.occurrence.29.variant.${variant}.choice.02`, {
    kinds: ["Special"],
    targets: [],
  });
}

setOutcome("universe.occurrence.30.variant.13501.choice.01", {
  kinds: ["Obtain", "Battle", "Special"],
  targets: ["Blessing"],
  numeric_literals: ["1"],
  parameter_refs: [blessingPool.all, progressive],
  chance_percentages: [
    "56",
    "30",
    "14",
    "32",
    "60",
    "8",
    "8",
    "90",
    "2",
    "0",
    "100",
    "0",
  ],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.30.variant.13501.choice.02", {
  kinds: ["Special"],
  targets: [],
  parameter_refs: [progressive],
});

const sourceReview = {
  schema_revision: "starclock.goal07-occurrence-source-review.v1",
  partition_id: "G07-P4-M13-S05",
  reviewed_on: "2026-07-28",
  frozen_source_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  sources: [
    {
      url: "https://honkai-star-rail.fandom.com/wiki/History_Fictionologists_%28I%29",
      section: "Possible Outcomes and Gameplay Notes",
      evidence_sha256:
        "24b1839e367beea6c3abf3db372472a807ee8f134ad0fa68d6409a6cc9417445",
      facts: [
        "greatest-owned-path:one-star:enhance-3",
        "greatest-owned-path:two-star:enhance-2",
        "greatest-owned-path:three-star:enhance-1",
      ],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Bounty_Hunter",
      section: "Possible Outcomes — Simulated Universe",
      evidence_sha256:
        "1abc9fac1899b86684a0feb608142fc10c94e867dc8ce578722c70cf561aca10",
      facts: ["discard-curio:1:cosmic-fragments:200", "leave"],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Implement_of_Error",
      section: "Possible Outcomes — Simulated Universe",
      evidence_sha256:
        "8a691360126f941c357fe14167002a75d11af6d0883f40885296635bc1582ae5",
      facts: ["random-repairing-error-code-curio:1", "leave"],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/We_Are_Cowboys",
      section: "Possible Outcomes — Simulated Universe",
      evidence_sha256:
        "c39edbdd2ca89bfb38dd607c7eadb297dfcc488aaad9e4dbe6d3e82afd4b6ed2",
      facts: ["current-cosmic-fragments-loss:50-percent", "battle-transition"],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Nildis_%28Lightfish%29",
      section: "Possible Outcomes — Simulated Universe",
      evidence_sha256:
        "3cbcd6dae19ad6dcad038664de90851d430c64102fdeabdafacbaeb120dc701e",
      facts: [
        "attempt-1:blessing-56:battle-30:blank-14",
        "attempt-2:blessing-32:battle-60:blank-8",
        "attempt-3:blessing-8:battle-90:blank-2",
        "attempt-4:battle-100",
      ],
    },
    {
      url: "https://raw.githubusercontent.com/Dimbreath/turnbasedgamedata/fd978d6ef09f941fba644c731ab54abd6f7c3568/Config/Level/Rogue/RogueDialogue/Event0010201/Opt0010201.json",
      section: "OptionList and DynamicMap",
      evidence_sha256:
        "366a8f24aea3f7d6568550aa11e7794fef19c3af074d4d845003332021c951b8",
      facts: ["path-display-ids:120-through-128", "options:10201-through-10204"],
    },
    {
      url: "https://raw.githubusercontent.com/Dimbreath/turnbasedgamedata/fd978d6ef09f941fba644c731ab54abd6f7c3568/Config/Level/Rogue/RogueDialogue/Event0012001/Opt0012001.json",
      section: "OptionList",
      evidence_sha256:
        "afa354dbe3ab45782692a681acb32ac26f5e062e98529aa7607b78d20751ebd2",
      facts: ["option-12001-desc-value:200"],
    },
    {
      url: "https://raw.githubusercontent.com/Dimbreath/turnbasedgamedata/fd978d6ef09f941fba644c731ab54abd6f7c3568/Config/Level/Rogue/RogueDialogue/Event0012101/Opt0012101.json",
      section: "OptionList",
      evidence_sha256:
        "af263dd2f4a15fed1b2c7a9444ba73883d46bbec7079940f6a376813e5b61a07",
      facts: ["options:12101-and-12102"],
    },
    {
      url: "https://raw.githubusercontent.com/Dimbreath/turnbasedgamedata/fd978d6ef09f941fba644c731ab54abd6f7c3568/Config/Level/Rogue/RogueDialogue/Event0013301/Opt0013301.json",
      section: "OptionList",
      evidence_sha256:
        "9d97f90eede99aa2d859bf7e80201a649a6f3a4981058b709890340b41dde3ad",
      facts: ["option-13301-desc-value:50"],
    },
    {
      url: "https://raw.githubusercontent.com/Dimbreath/turnbasedgamedata/fd978d6ef09f941fba644c731ab54abd6f7c3568/Config/Level/Rogue/RogueDialogue/Event0013501/Opt0013501.json",
      section: "OptionList",
      evidence_sha256:
        "b98ac2300e1d500d0136d5bd7a0bbb39615be307e7db502689a2f6d66c1ad6b0",
      facts: ["option-13501:flip", "option-13506:give-up"],
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
  console.log("Refined Goal 07 Occurrence S05 source evidence and pack index.");
} else {
  assert(originalChoices === encodedChoices, "Occurrence S05 normalized choices drifted");
  assert(fs.readFileSync(reviewPath, "utf8") === encodedReview, "Occurrence S05 source review drifted");
  assert(fs.readFileSync(indexPath, "utf8") === encodedIndex, "Occurrence S05 pack index drifted");
  console.log("Goal 07 Occurrence S05 source refinement is stable.");
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
    chance_percentages: patch.chance_percentages ?? [],
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
