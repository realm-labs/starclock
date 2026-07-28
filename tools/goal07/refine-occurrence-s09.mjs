import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const packRoot = path.join(root, "content-reference/standard-universe-v1");
const choicesPath = path.join(packRoot, "occurrence-choices.json");
const indexPath = path.join(packRoot, "pack-index.json");
const reviewPath = path.join(
  root,
  "evidence/standard-universe-mechanics-complete-v1/source-reviews/G07-P4-M13-S09.json",
);
const choices = JSON.parse(fs.readFileSync(choicesPath, "utf8"));
const byId = new Map(choices.map((choice) => [choice.id, choice]));
const marker = (value) => `universe.occurrence-s09.${value}`;
const rarity = (value) => `universe.blessing-pool.rarity.${value}`;
const positiveCurios = "universe.curio-pool.polarity.positive";
const negativeCurios = "universe.curio-pool.polarity.negative";
const lottoCurios = ["universe.curio.63", "universe.curio.107"];

setOutcome("universe.occurrence.43.variant.12601.choice.01", {
  kinds: ["Consume", "Repair"],
  targets: ["CosmicFragments", "Curio"],
  numeric_literals: ["50", "1"],
  parameter_refs: [marker("repair-one-destroyed")],
  costs: [{ kind: "Consume", targets: ["CosmicFragments"] }],
});
setOutcome("universe.occurrence.43.variant.12601.choice.02", {
  kinds: ["Consume", "Repair"],
  targets: ["CosmicFragments", "Curio"],
  numeric_literals: ["100", "61"],
  parameter_refs: [marker("repair-all-destroyed")],
  costs: [{ kind: "Consume", targets: ["CosmicFragments"] }],
});
setOutcome("universe.occurrence.43.variant.12601.choice.03", {
  kinds: ["Special"],
  targets: [],
});

setOutcome("universe.occurrence.44.variant.12701.choice.01", {
  kinds: ["Discard", "Obtain"],
  targets: ["Blessing", "Blessing"],
  numeric_literals: ["2", "4"],
  parameter_refs: [marker("showman-two-to-four"), rarity(2)],
  unspecified_random_policy: "StableUniformOrderedCandidates",
  costs: [{ kind: "Discard", targets: ["Blessing"] }],
});
setOutcome("universe.occurrence.44.variant.12701.choice.02", {
  kinds: ["Discard", "Obtain"],
  targets: ["Blessing", "Blessing"],
  numeric_literals: ["2", "2"],
  parameter_refs: [marker("showman-two-to-two-three"), rarity(2), rarity(3)],
  unspecified_random_policy: "StableUniformOrderedCandidates",
  costs: [{ kind: "Discard", targets: ["Blessing"] }],
});

setOutcome("universe.occurrence.45.variant.12801.choice.01", {
  kinds: ["Consume", "Repair", "Obtain"],
  targets: ["CosmicFragments", "Curio", "Curio"],
  numeric_literals: ["100", "1", "1"],
  parameter_refs: [marker("double-lottery-buy"), ...lottoCurios],
  unspecified_random_policy: "StableUniformOrderedCandidates",
  costs: [{ kind: "Consume", targets: ["CosmicFragments"] }],
});
setOutcome("universe.occurrence.45.variant.12801.choice.02", {
  kinds: ["Consume", "Repair"],
  targets: ["CosmicFragments", "Curio"],
  numeric_literals: ["50", "2"],
  parameter_refs: [marker("double-lottery-repair"), ...lottoCurios],
  costs: [{ kind: "Consume", targets: ["CosmicFragments"] }],
});
setOutcome("universe.occurrence.45.variant.12801.choice.03", {
  kinds: ["Special"],
  targets: [],
});

setOutcome("universe.occurrence.46.variant.12901.choice.01", {
  kinds: ["Enhance"],
  targets: ["Blessing"],
  numeric_literals: ["500"],
  parameter_refs: [marker("ruan-enhance-all")],
});
setOutcome("universe.occurrence.46.variant.12901.choice.02", {
  kinds: ["Obtain"],
  targets: ["Curio"],
  numeric_literals: ["10"],
  parameter_refs: [marker("ruan-curios-ten"), positiveCurios],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.46.variant.12901.choice.03", {
  kinds: ["Enhance", "Obtain"],
  targets: ["Blessing", "Curio"],
  numeric_literals: ["500", "10"],
  parameter_refs: [marker("ruan-both"), positiveCurios],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});

for (const [choice, stage] of [
  [1, "third"],
  [3, "second"],
  [5, "first"],
]) {
  setOutcome(`universe.occurrence.47.variant.13001.choice.${pad(choice)}`, {
    kinds: ["Consume"],
    targets: ["CosmicFragments"],
    numeric_literals: ["40"],
    parameter_refs: [marker(`perfect-pay-${stage}`)],
    costs: [{ kind: "Consume", targets: ["CosmicFragments"] }],
  });
}
for (const [choice, stage] of [
  [2, "third"],
  [4, "second"],
  [6, "first"],
]) {
  setOutcome(`universe.occurrence.47.variant.13001.choice.${pad(choice)}`, {
    kinds: ["Special"],
    targets: [],
    parameter_refs: [marker(`perfect-leave-${stage}`)],
  });
}
for (const [choice, kind, stage, chances] of [
  [7, "clay", "second", ["50", "50"]],
  [8, "popular", "second", ["40", "60"]],
  [9, "clay", "third", ["50", "50"]],
  [10, "popular", "third", ["40", "60"]],
  [11, "clay", "first", ["50", "50"]],
]) {
  setOutcome(`universe.occurrence.47.variant.13001.choice.${pad(choice)}`, {
    kinds: ["Obtain"],
    targets: ["Curio"],
    numeric_literals: ["1"],
    parameter_refs: [
      marker(`perfect-${kind}-${stage}`),
      positiveCurios,
      ...(kind === "clay" ? [negativeCurios] : []),
    ],
    chance_percentages: chances,
    unspecified_random_policy: "StableUniformOrderedCandidates",
  });
}

const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
const review = {
  schema_revision: "starclock.goal07-occurrence-source-review.v1",
  partition_id: "G07-P4-M13-S09",
  reviewed_on: "2026-07-28",
  frozen_source_revision: revision,
  sources: [
    source(
      "The_Curio_Fixer",
      "17df1af69d20e0883a56aacf73ee77fe09f9181648b71767682309a1d4e9217a",
      [
        "repair-one:fragments-50:destroyed-curio-1",
        "repair-all:fragments-100",
        "leave:no-change",
      ],
    ),
    source(
      "Showman%27s_Sleight",
      "499f73f0d56b217b63eebef5608ba5390267013a5149763303d04ea4e05acdbb",
      [
        "left:discard-two-two-star:obtain-four-two-star",
        "right:discard-two-two-star:obtain-two-three-star",
      ],
    ),
    source(
      "The_Double_Lottery_Experience",
      "03a9b72351de8cc85e12e45a6e30410c5ef872ba06aff06c4071ce3d690573ed",
      [
        "vote:fragments-100:repair-one-lotto:obtain-one-lotto",
        "double-delight:fragments-50:repair-all-lottos",
        "refuse:no-change",
      ],
    ),
    source(
      "Ruan_Mei_%28II%29",
      "6a0322477918070db9cdbeaa86334489f8ccb69a984986ebcde559aa04395de4",
      [
        "worship:enhance-up-to-500-blessings",
        "steal:normal-curios-10",
        "ruan-mei-downloaded:both",
      ],
    ),
    source(
      "The_%2APerfect%2A_Grand_Challenge%21",
      "d801dafab8bd525816a365dc884cd9073e6bec8c5bf2e1ed166a137addf9310c",
      [
        "attempts:maximum-3:fragments-40-each",
        "clay-doll:normal-curio-50:negative-curio-50",
        "popular-toy:normal-curio-40:nothing-60",
        "leave:end",
      ],
    ),
    ...[
      ["12601", "417039e37c2b845018c5a2889d72bf27b8e78813ebc508706522b67badb54839"],
      ["12701", "ae55121259cfc5c3035a5f6526296d2958dc01616d414cd5b7566ffe73755227"],
      ["12801", "d39207859a522c58d292120eb8b6ac94effd560bded7c12a83589e436ddca623"],
      ["12901", "abdaa95b382e1f6c7c828ba1ff577e0b731e4368c0d74724874e5fb403e6b686"],
      ["13001", "110823c0e966fa22d2d0fea17fdc31f53687c44d0131990447b42511fe5e1dc3"],
    ].map(([option, digest]) => ({
      url: `https://gitlab.com/Dimbreath/turnbasedgamedata/-/raw/${revision}/Config/Level/Rogue/RogueDialogue/Event00${option}/Opt00${option}.json`,
      section: "OptionList",
      evidence_sha256: digest,
      facts: [`source-option:${option}`],
    })),
    {
      url: `https://gitlab.com/Dimbreath/turnbasedgamedata/-/raw/${revision}/Config/Level/Rogue/RogueDialogue/Event0013001/Act0013001.json`,
      section: "OnStartSequece",
      evidence_sha256: "8a2e5e03e397f6aad05a74d0875490001ed474cbe9a0c1a7478832a85bfaaf16",
      facts: [
        "perfect-graph:first-13001-13002:second-13005-13006:third-13009-13010",
        "perfect-results:first-13003-13004:second-13007-13008:third-13011-13012",
      ],
    },
  ],
};

const encodedChoices = JSON.stringify(choices, null, 2) + "\n";
const encodedReview = JSON.stringify(review, null, 2) + "\n";
const encodedIndex = JSON.stringify(buildIndex(), null, 2) + "\n";
if (process.argv.includes("--write")) {
  fs.writeFileSync(choicesPath, encodedChoices);
  fs.mkdirSync(path.dirname(reviewPath), { recursive: true });
  fs.writeFileSync(reviewPath, encodedReview);
  fs.writeFileSync(indexPath, encodedIndex);
  console.log("Refined Goal 07 Occurrence S09 source evidence and pack index.");
} else {
  assert.equal(fs.readFileSync(choicesPath, "utf8"), encodedChoices);
  assert.equal(fs.readFileSync(reviewPath, "utf8"), encodedReview);
  assert.equal(fs.readFileSync(indexPath, "utf8"), encodedIndex);
  console.log("Goal 07 Occurrence S09 source refinement is stable.");
}

function setOutcome(id, patch) {
  const choice = byId.get(id);
  assert(choice, `missing ${id}`);
  choice.costs = patch.costs ?? [];
  choice.outcomes[0] = {
    kinds: patch.kinds,
    targets: patch.targets,
    numeric_literals: patch.numeric_literals ?? [],
    parameter_refs: patch.parameter_refs ?? [],
    chance_percentages: patch.chance_percentages ?? [],
    unspecified_random_policy: patch.unspecified_random_policy ?? "",
  };
}
function source(page, digest, facts) {
  return {
    url: `https://honkai-star-rail.fandom.com/wiki/${page}`,
    section: "Possible Outcomes — Simulated Universe",
    evidence_sha256: digest,
    facts,
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
      return {
        file,
        bytes: bytes.length,
        rows: Array.isArray(JSON.parse(bytes)) ? JSON.parse(bytes).length : rows,
        sha256: sha256(bytes),
      };
    })
    .sort((left, right) => left.file.localeCompare(right.file));
  return {
    schema: existing.schema,
    files,
    pack_sha256: sha256(
      Buffer.from(files.map(({ file, sha256: digest }) => `${file}\0${digest}`).join("\n")),
    ),
  };
}
function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}
function pad(value) {
  return value.toString().padStart(2, "0");
}
