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
  "evidence/standard-universe-mechanics-complete-v1/source-reviews/G07-P4-M13-S07.json",
);
const choices = JSON.parse(fs.readFileSync(choicesPath, "utf8"));
const byId = new Map(choices.map((choice) => [choice.id, choice]));
const level = "universe.occurrence-battle.level.60";
const reward =
  "universe.occurrence-battle.reward.within-cycles.4.base.1.bonus.1";
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
const marker = (value) => `universe.occurrence-s07.${value}`;

const periodicEnemies = new Map([
  [
    13702,
    [
      "enemy.imaginary-weaver.minionlv2.variant.01",
      "enemy.imaginary-weaver.minionlv2.variant.01",
      "enemy.decaying-shadow.elite.variant.01",
    ],
  ],
  [
    13703,
    [
      "enemy.wraith-warden.minionlv2.variant.01",
      "enemy.wraith-warden.minionlv2.variant.01",
      "enemy.aurumaton-spectral-envoy.elite.variant.01",
    ],
  ],
  [
    13704,
    [
      "enemy.dreamjolt-troupes-mr-domescreen.minionlv2.variant.01",
      "enemy.dreamjolt-troupes-mr-domescreen.minionlv2.variant.01",
      "enemy.dreamjolt-troupes-beyond-overcooked.elite.variant.01",
    ],
  ],
  [
    13705,
    [
      "enemy.memory-zone-meme-heartbreaker.minionlv2.variant.01",
      "enemy.memory-zone-meme-shell-of-faded-rage.elite.variant.01",
    ],
  ],
]);
for (const [variant, enemies] of periodicEnemies) {
  setOutcome(`universe.occurrence.35.variant.${variant}.choice.01`, {
    kinds: ["Battle"],
    targets: [],
    parameter_refs: [
      "universe.occurrence-battle.stage.periodic-demon-lord",
      level,
      ...enemies,
      reward,
    ],
  });
}

setOutcome("universe.occurrence.36.variant.13801.choice.01", {
  kinds: ["Discard", "Obtain"],
  targets: ["Blessing", "Blessing"],
  numeric_literals: ["1", "1"],
  parameter_refs: [rarity(3), marker("exchange-rarity-3-for-3")],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.36.variant.13801.choice.02", {
  kinds: ["Discard", "Obtain"],
  targets: ["Blessing", "Blessing"],
  numeric_literals: ["1", "1"],
  parameter_refs: [marker("exchange-rarity-1-2-for-1-2-3")],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.36.variant.13801.choice.03", {
  kinds: ["Special"],
  targets: [],
});

for (const [choice, percent, blessingRarity] of [
  [1, 20, 2],
  [2, 80, 3],
]) {
  setOutcome(`universe.occurrence.37.variant.13901.choice.0${choice}`, {
    kinds: ["Lose", "Obtain"],
    targets: ["HP", "Blessing"],
    numeric_literals: [`${percent}%`, "1"],
    parameter_refs: [rarity(blessingRarity)],
    unspecified_random_policy: "StableUniformOrderedCandidates",
  });
}
setOutcome("universe.occurrence.37.variant.13901.choice.03", {
  kinds: ["Special"],
  targets: [],
});

for (const [choice, fragments, rarities] of [
  [1, 50, [1, 2]],
  [2, 100, [1, 2, 3]],
]) {
  setOutcome(`universe.occurrence.38.variant.14001.choice.0${choice}`, {
    kinds: ["Consume", "Obtain"],
    targets: ["CosmicFragments", "Blessing"],
    numeric_literals: [`${fragments}`, "1"],
    parameter_refs: rarities.map(rarity),
    unspecified_random_policy: "StableUniformOrderedCandidates",
  });
}
setOutcome("universe.occurrence.38.variant.14001.choice.03", {
  kinds: ["Special"],
  targets: [],
});

setOutcome("universe.occurrence.39.variant.12201.choice.01", {
  kinds: ["Enhance"],
  targets: ["Blessing"],
  numeric_literals: ["2"],
  parameter_refs: [rarity(3)],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.39.variant.12201.choice.02", {
  kinds: ["Obtain"],
  targets: ["Character"],
  numeric_literals: ["1"],
  parameter_refs: [marker("current-path-formation")],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.39.variant.12201.choice.03", {
  kinds: ["Obtain"],
  targets: ["Blessing"],
  numeric_literals: ["2"],
  parameter_refs: [
    ...allPaths,
    rarity(2),
    rarity(3),
    marker("current-path-blessings"),
  ],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.39.variant.12201.choice.04", {
  kinds: ["Obtain"],
  targets: ["Blessing"],
  numeric_literals: ["3"],
  parameter_refs: [
    ...allPaths,
    "universe.blessing-pool.all",
    marker("current-path-blessings"),
  ],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.39.variant.12201.choice.05", {
  kinds: ["Discard", "Obtain"],
  targets: ["Blessing", "Blessing"],
  numeric_literals: ["4", "4"],
  parameter_refs: [
    ...allPaths,
    marker("exchange-four-one-star-for-current-path"),
  ],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.39.variant.12201.choice.06", {
  kinds: ["Obtain"],
  targets: ["Curio"],
  numeric_literals: ["3"],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.39.variant.12201.choice.07", {
  kinds: ["Discard", "Obtain"],
  targets: ["Curio", "CosmicFragments"],
  numeric_literals: ["1", "50"],
  parameter_refs: [marker("all-curios-for-fragments-50")],
});
setOutcome("universe.occurrence.39.variant.12201.choice.08", {
  kinds: ["Discard"],
  targets: ["Curio"],
  numeric_literals: ["1"],
  parameter_refs: [65, 66, 67, 70, 71, 108].map(
    (id) => `universe.curio.${id}`,
  ),
  unspecified_random_policy: "StableUniformOrderedCandidates",
});

const review = {
  schema_revision: "starclock.goal07-occurrence-source-review.v1",
  partition_id: "G07-P4-M13-S07",
  reviewed_on: "2026-07-28",
  frozen_source_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  sources: [
    source("Periodic_Demon_Lord", "e8a7515c4c43ad66b9e06edfc9a2ee655032266f8a6856fc020e19389345c974", [
      "variant-2:imaginary-weaver-2:decaying-shadow-1",
      "variant-3:wraith-warden-2:aurumaton-spectral-envoy-1",
      "variant-4:mr-domescreen-2:beyond-overcooked-1",
      "variant-5:heartbreaker-1:shell-of-faded-rage-1",
      "victory-within-cycles-4:additional-blessing-1",
    ]),
    source("Let%27s_Exchange_Gifts", "fe3f83d16f795a9b154398e72f7d561dc8b40b777eaf2c46c2e0bd25c9bcd438", [
      "reforge:discard-three-star-1:obtain-three-star-1",
      "exchange:discard-one-to-two-star-1:obtain-one-to-three-star-1",
      "leave:no-change",
    ]),
    source("Make_A_Wish", "883b9ac47d63cbe03abdfd29ac3507396dd7888cba93b687fb9a1f75b182841d", [
      "two-star-blessing:current-hp-loss-20-percent",
      "three-star-blessing:current-hp-loss-80-percent",
      "leave:no-change",
    ]),
    source("Robot_Sales_Terminal", "62ece87a017a9dbeafa85173481c7e6cca070586983f04b228428fe75c4d3e86", [
      "one-to-two-star-blessing:fragments-50",
      "one-to-three-star-blessing:fragments-100",
      "leave:no-change",
    ]),
    source("Knights_of_Beauty_to_the_Rescue", "e99027dae3e292c00fe63aa6af83c84dae7bcf0d5e2f4ba7fbdf33fad3fe6418", [
      "stilott:enhance-three-star-blessings-2",
      "abomins:current-path-resonance-formation-1",
      "argenti:current-path-two-to-three-star-blessings-2",
      "argenti:current-path-blessings-3",
      "will-garner:discard-one-star-blessings-4:current-path-blessings-4",
      "pomaine:random-curios-3",
      "pomaine:discard-all-curios:fragments-per-curio-50",
      "anoklay:discard-negative-cuckoo-curio-1",
    ]),
    ...[
      ["12201", "920f08a26c51ffb2c9394fd46973c6ab45df4c712588091d65ee0d95a777e9c7"],
      ["13702", "2ddaeaa1039dbb3a348cecf19d97275d7cd64c2a2fa8587b609bbf73173e56c4"],
      ["13703", "4ef47defb84a99e5f9e2117a1a73bb385981591d42787e6965c1fa408960012c"],
      ["13704", "4fa7c401de622362708a15dfa820e5c56e1feebf56e02704fea7cfdb7660127f"],
      ["13705", "480bcacdcda6cd26e23eb9a3ba7fce5ac6b4c56c4f3fb095908d1c790597e980"],
      ["13801", "1d541d7d75ce334677127eb39bd81fd57ac6328c8e297e6591d54cde14389397"],
      ["13901", "d292bd6b5ef0bd79771bf7b0303a07ce6fe2782eed1a4bb682c2c0b58e524270"],
      ["14001", "4744de296d062fc4403956594102c3495cbb044c4d6e6f539f71eaf487f16ddf"],
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
  console.log("Refined Goal 07 Occurrence S07 source evidence and pack index.");
} else {
  assert.equal(fs.readFileSync(choicesPath, "utf8"), encodedChoices);
  assert.equal(fs.readFileSync(reviewPath, "utf8"), encodedReview);
  assert.equal(fs.readFileSync(indexPath, "utf8"), encodedIndex);
  console.log("Goal 07 Occurrence S07 source refinement is stable.");
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
