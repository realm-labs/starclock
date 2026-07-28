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
  "evidence/standard-universe-mechanics-complete-v1/source-reviews/G07-P4-M13-S08.json",
);
const choices = JSON.parse(fs.readFileSync(choicesPath, "utf8"));
const byId = new Map(choices.map((choice) => [choice.id, choice]));
const allPaths = [
  "universe.path.preservation",
  "universe.path.remembrance",
  "universe.path.nihility",
  "universe.path.abundance",
  "universe.path.hunt",
  "universe.path.destruction",
  "universe.path.elation",
  "universe.path.propagation",
  "universe.path.erudition",
];
const rarity = (value) => `universe.blessing-pool.rarity.${value}`;
const marker = (value) => `universe.occurrence-s08.${value}`;
const level = "universe.occurrence-battle.level.60";
const fixedReward = "universe.occurrence-battle.reward.fixed-blessings.1";

setOutcome("universe.occurrence.39.variant.12201.choice.09", {
  kinds: ["Obtain"],
  targets: ["Blessing"],
  numeric_literals: ["1"],
  parameter_refs: [
    ...allPaths,
    rarity(3),
    "universe.occurrence-s07.current-path-blessings",
  ],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.39.variant.12201.choice.10", {
  kinds: ["Obtain"],
  targets: ["CosmicFragments"],
  numeric_literals: ["400"],
});
setOutcome("universe.occurrence.39.variant.12201.choice.11", {
  kinds: ["Obtain"],
  targets: ["Curio"],
  numeric_literals: ["1"],
  parameter_refs: ["universe.curio.113"],
});

for (const [choice, blessingRarity, quantity] of [
  [1, 1, 3],
  [2, 2, 2],
  [3, 3, 1],
  [5, 1, 3],
  [6, 2, 2],
  [7, 3, 1],
]) {
  setOutcome(`universe.occurrence.4.variant.10201.choice.0${choice}`, {
    kinds: ["Enhance"],
    targets: ["Blessing"],
    numeric_literals: [`${quantity}`],
    parameter_refs: [
      ...allPaths,
      rarity(blessingRarity),
      marker("history-best-path"),
    ],
    unspecified_random_policy: "StableUniformOrderedCandidates",
  });
}
for (const choice of [4, 8]) {
  setOutcome(`universe.occurrence.4.variant.10201.choice.0${choice}`, {
    kinds: ["Special"],
    targets: [],
  });
}

setOutcome("universe.occurrence.40.variant.12301.choice.01", {
  kinds: ["Special"],
  targets: [],
  numeric_literals: ["10"],
  parameter_refs: [marker("cosmic-crescendo")],
  chance_percentages: [
    "26",
    "13",
    "13",
    "13",
    "4",
    "4",
    "3",
    "3",
    "3",
    "3",
    "3",
    "3",
    "3",
    "2",
    "2",
    "2",
  ],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.40.variant.12301.choice.02", {
  kinds: ["Special"],
  targets: [],
});

for (const [choice, kind] of [
  [1, "yu-add-sugar"],
  [2, "yu-add-toothpaste"],
]) {
  setOutcome(`universe.occurrence.41.variant.12401.choice.0${choice}`, {
    kinds: ["Special"],
    targets: [],
    parameter_refs: [marker(kind)],
  });
}
for (const [choice, kind, chances] of [
  [3, "yu-sugar-vigorous", ["70", "20", "10"]],
  [4, "yu-sugar-gentle", ["50", "50"]],
  [5, "yu-toothpaste-vigorous", ["80", "20"]],
  [6, "yu-toothpaste-gentle", ["50", "30", "20"]],
]) {
  setOutcome(`universe.occurrence.41.variant.12401.choice.0${choice}`, {
    kinds: ["Special"],
    targets: [],
    parameter_refs: [
      marker(kind),
      "universe.curio.121",
      "universe.curio.122",
    ],
    chance_percentages: chances,
  });
}
setOutcome("universe.occurrence.41.variant.12401.choice.07", {
  kinds: ["Battle"],
  targets: [],
  parameter_refs: [
    marker("yu-resolve-elite"),
    "universe.occurrence-battle.stage.yu-qingtu-elite",
    level,
    "enemy.frigid-prowler.elite.variant.01",
    fixedReward,
    rarity(3),
  ],
});
setOutcome("universe.occurrence.41.variant.12401.choice.08", {
  kinds: ["Lose"],
  targets: ["CosmicFragments"],
  numeric_literals: ["50%"],
  parameter_refs: [marker("yu-resolve-thief")],
  costs: [{ kind: "Lose", targets: ["CosmicFragments"] }],
});
setOutcome("universe.occurrence.41.variant.12401.choice.09", {
  kinds: ["Battle"],
  targets: [],
  parameter_refs: [
    marker("yu-resolve-thief"),
    "universe.occurrence-battle.stage.yu-qingtu-thief",
    level,
    "enemy.frigid-prowler.elite.variant.01",
    fixedReward,
  ],
});

for (const [choice, stage, enemy, blessingRarity, curioCount] of [
  [
    1,
    "beast-horde-young",
    "enemy.automaton-grizzly.elite.variant.01",
    2,
    0,
  ],
  [
    2,
    "beast-horde-adult",
    "enemy.frigid-prowler.elite.variant.01",
    3,
    1,
  ],
]) {
  setOutcome(`universe.occurrence.42.variant.12501.choice.0${choice}`, {
    kinds: curioCount ? ["Battle", "Obtain"] : ["Battle"],
    targets: curioCount ? ["Character", "Curio"] : [],
    numeric_literals: curioCount ? ["1", `${curioCount}`] : [],
    parameter_refs: [
      `universe.occurrence-battle.stage.${stage}`,
      level,
      enemy,
      fixedReward,
      rarity(blessingRarity),
    ],
  });
}

const review = {
  schema_revision: "starclock.goal07-occurrence-source-review.v1",
  partition_id: "G07-P4-M13-S08",
  reviewed_on: "2026-07-28",
  frozen_source_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  sources: [
    source(
      "Knights_of_Beauty_to_the_Rescue",
      "e99027dae3e292c00fe63aa6af83c84dae7bcf0d5e2f4ba7fbdf33fad3fe6418",
      [
        "holvisio:current-path-three-star-blessing-1",
        "galahad-icahn:cosmic-fragments-400",
        "galahad-icahn:cavity-system-model-1",
      ],
    ),
    source(
      "History_Fictionologists_%28II%29",
      "66b820efaf7a4e1405816220ffd9019e7c015558d65539841de12bdf3a1deb5a",
      [
        "greatest-owned-path:one-star-enhance-3",
        "greatest-owned-path:two-star-enhance-2",
        "greatest-owned-path:three-star-enhance-1",
        "leave:no-change",
      ],
    ),
    source(
      "Cosmic_Crescendo",
      "23e83401555eafbdb7a47eee4df43760f30fa5d58488c96d1adbc7f539be96c8",
      [
        "listen:effects-10",
        "ordered-effect-weights:26-13-13-13-4-4-3-3-3-3-3-3-3-2-2-2",
        "leave:no-change",
      ],
    ),
    source(
      "Genius_Society_55_Yu_Qingtu",
      "7d9e4b7b4431c005ca69b7df3af9ea25f56df3e46a222aeed3859aa9b1ca5adf",
      [
        "sugar-vigorous:pinkest-70:hp-loss-20:thief-10",
        "sugar-gentle:pinkest-50:thief-50",
        "toothpaste-vigorous:thalan-80:elite-20",
        "toothpaste-gentle:thalan-50:hp-loss-30:elite-20",
        "thief-sleep:fragments-loss-50-percent",
      ],
    ),
    source(
      "Beast_Horde%3A_Voracious_Catastrophe",
      "c6e2894933ecaa4bbb77dde81f04eca6fc8fe52c56df91c98f6fff0f885fc577",
      [
        "young-beasts:battle:two-star-blessing-1",
        "adult-beast:battle:three-star-blessing-1:curio-1",
      ],
    ),
    ...[
      ["10201", "eff8fc55698de48d85c38caa4054148d8f63fe40a5cdfbf193494995b874e726"],
      ["10202", "eff8fc55698de48d85c38caa4054148d8f63fe40a5cdfbf193494995b874e726"],
      ["12201", "920f08a26c51ffb2c9394fd46973c6ab45df4c712588091d65ee0d95a777e9c7"],
      ["12301", "e81d95b2b570414d7c7d397115f80c66782b6b701475c0da8f3cec79f5460376"],
      ["12401", "3395357b583c3871c8a6f998aa9fd95a53dad1fd316958bfba78a923a6243571"],
      ["12501", "3a551b6f302001894203b825ae8de8adad5814dcae059852b8afd72f031545eb"],
    ].map(([option, digest]) => ({
      url: `https://gitlab.com/Dimbreath/turnbasedgamedata/-/raw/fd978d6ef09f941fba644c731ab54abd6f7c3568/Config/Level/Rogue/RogueDialogue/Event00${option}/Opt00${option}.json`,
      section: "OptionList",
      evidence_sha256: digest,
      facts: [`source-option:${option}`],
    })),
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
  console.log("Refined Goal 07 Occurrence S08 source evidence and pack index.");
} else {
  assert.equal(fs.readFileSync(choicesPath, "utf8"), encodedChoices);
  assert.equal(fs.readFileSync(reviewPath, "utf8"), encodedReview);
  assert.equal(fs.readFileSync(indexPath, "utf8"), encodedIndex);
  console.log("Goal 07 Occurrence S08 source refinement is stable.");
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
