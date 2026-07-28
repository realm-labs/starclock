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
  "evidence/standard-universe-mechanics-complete-v1/source-reviews/G07-P4-M13-S11.json",
);
const choices = JSON.parse(fs.readFileSync(choicesPath, "utf8"));
const byId = new Map(choices.map((choice) => [choice.id, choice]));
const marker = (value) => `universe.occurrence-s11.${value}`;

for (const occurrence of [54, 55]) {
  const prefix = `universe.occurrence.${occurrence}.variant.14401.choice`;
  if (occurrence === 55) {
    setOutcome(`${prefix}.01`, {
      kinds: ["Discard", "Lose", "Special"],
      targets: ["Curio", "HP"],
      numeric_literals: ["1", "30%"],
      parameter_refs: ["universe.occurrence-s10.beauty-bug-feed-curio"],
      chance_percentages: ["70", "30"],
      unspecified_random_policy: "StableUniformOrderedCandidates",
      costs: [{ kind: "Discard", targets: ["Curio"] }],
    });
  }
  setOutcome(`${prefix}.02`, {
    kinds: ["Discard", "Lose", "Special"],
    targets: ["Blessing", "HP"],
    numeric_literals: ["1", "30%"],
    parameter_refs: [marker("beauty-bug-feed-blessing")],
    chance_percentages: ["70", "30"],
    unspecified_random_policy: "StableUniformOrderedCandidates",
    costs: [{ kind: "Discard", targets: ["Blessing"] }],
  });
  setOutcome(`${prefix}.03`, {
    kinds: ["Consume", "Lose", "Special"],
    targets: ["CosmicFragments", "HP"],
    numeric_literals: ["100", "30%"],
    parameter_refs: [marker("beauty-bug-feed-fragments")],
    chance_percentages: ["70", "30"],
    unspecified_random_policy: "StableUniformOrderedCandidates",
    costs: [{ kind: "Consume", targets: ["CosmicFragments"] }],
  });
  setOutcome(`${prefix}.04`, {
    kinds: ["Obtain"],
    targets: ["Curio"],
    numeric_literals: ["5"],
    parameter_refs: [
      marker("beauty-bug-heartfelt-gift"),
      "universe.curio-pool.polarity.positive",
    ],
    unspecified_random_policy: "StableUniformOrderedCandidates",
  });
  setOutcome(`${prefix}.05`, {
    kinds: ["Obtain"],
    targets: ["Blessing"],
    numeric_literals: ["1"],
    parameter_refs: [
      marker("beauty-bug-life-favor"),
      "universe.blessing-pool.rarity.3",
    ],
    unspecified_random_policy: "StableUniformOrderedCandidates",
  });
  setOutcome(`${prefix}.06`, {
    kinds: ["Special"],
    parameter_refs: [marker("beauty-bug-refuse")],
  });
}

setOutcome("universe.occurrence.56.variant.14501.choice.01", {
  kinds: ["Discard", "Obtain"],
  targets: ["Curio"],
  numeric_literals: ["1", "2"],
  parameter_refs: [
    marker("ace-trash-exchange"),
    "universe.curio-pool.polarity.positive",
  ],
  unspecified_random_policy: "StableUniformOrderedCandidates",
  costs: [{ kind: "Discard", targets: ["Curio"] }],
});
setOutcome("universe.occurrence.56.variant.14501.choice.02", {
  kinds: ["Special"],
  parameter_refs: [marker("ace-trash-leave")],
});

setOutcome("universe.occurrence.6.variant.10401.choice.01", {
  kinds: ["Restore", "Lose"],
  targets: ["HP"],
  numeric_literals: ["100%", "20%"],
  parameter_refs: [marker("shopping-doughnuts")],
  chance_percentages: ["80", "20"],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.6.variant.10401.choice.02", {
  kinds: ["Discard", "Obtain"],
  targets: ["Blessing"],
  numeric_literals: ["1", "1"],
  parameter_refs: [
    marker("shopping-lotus"),
    "universe.blessing-pool.rarity.1",
    "universe.blessing-pool.rarity.2",
  ],
  chance_percentages: ["80", "20"],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.6.variant.10401.choice.03", {
  kinds: ["Obtain"],
  targets: ["Curio"],
  numeric_literals: ["1"],
  parameter_refs: [
    marker("shopping-mechanical-box"),
    "universe.curio-pool.polarity.positive",
    "universe.curio-pool.polarity.negative",
  ],
  chance_percentages: ["80", "20"],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.6.variant.10401.choice.04", {
  kinds: ["Special"],
  parameter_refs: [marker("shopping-leave")],
});

setOutcome("universe.occurrence.60.variant.19301.choice.01", {
  kinds: ["Consume", "Obtain", "Obtain"],
  targets: ["CosmicFragments", "Blessing", "CosmicFragments"],
  numeric_literals: ["50", "1", "100"],
  parameter_refs: [
    marker("universal-dancer-fortune"),
    "universe.blessing-pool.rarity.3",
  ],
  chance_percentages: ["30", "70", "65", "35", "100", "0"],
  unspecified_random_policy: "StableUniformOrderedCandidates",
  costs: [{ kind: "Consume", targets: ["CosmicFragments"] }],
});
setOutcome("universe.occurrence.60.variant.19301.choice.02", {
  kinds: ["Special"],
  parameter_refs: [marker("universal-dancer-refuse")],
});

setOutcome("universe.occurrence.62.variant.19501.choice.01", {
  kinds: ["Special"],
  numeric_literals: ["1"],
  parameter_refs: [marker("mirror-light-candle")],
});
setOutcome("universe.occurrence.62.variant.19501.choice.02", {
  kinds: ["Special"],
  parameter_refs: [marker("mirror-leave")],
});
setOutcome("universe.occurrence.62.variant.19501.choice.03", {
  kinds: ["Obtain", "Obtain", "Obtain", "Obtain", "Obtain", "Obtain", "Enhance", "Obtain"],
  targets: [
    "CosmicFragments",
    "CosmicFragments",
    "CosmicFragments",
    "Blessing",
    "Blessing",
    "Curio",
    "Blessing",
    "Blessing",
  ],
  numeric_literals: ["50", "150", "300", "1", "1", "2", "3", "2"],
  parameter_refs: [
    marker("mirror-random-wish"),
    "universe.blessing-pool.rarity.2",
    "universe.blessing-pool.rarity.3",
    "universe.curio-pool.polarity.positive",
  ],
  chance_percentages: ["10", "20", "5", "20", "10", "10", "10", "15"],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});

const review = {
  schema_revision: "starclock.goal07-occurrence-source-review.v1",
  partition_id: "G07-P4-M13-S11",
  reviewed_on: "2026-07-28",
  frozen_source_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  sources: [
    source(
      "https://honkai-star-rail.fandom.com/wiki/Loneliness%2C_Cosmic_Beauty_Bugs%2C_Simulated_Universe_%28I%29",
      "Possible Outcomes — Simulated Universe",
      "a5309f2f1f835d2e1e14082218ef422380080632ce07445404faa21b8b69db31",
      [
        "feed-blessing-or-100-fragments:discard-cost",
        "feed-success-70:unlock-part-two",
        "feed-failure-30:all-allies-current-hp-minus-30-percent",
      ],
    ),
    source(
      "https://honkai-star-rail.fandom.com/wiki/Loneliness%2C_Cosmic_Beauty_Bugs%2C_Simulated_Universe_%28II%29",
      "Possible Outcomes — Simulated Universe",
      "b353f9c0b936aae2f298e11958338a380653c23b03c5efe994deb191fb22e748",
      ["heartfelt-gift:five-random-curios", "life-favor:select-one-three-star-blessing", "refuse:no-change"],
    ),
    source(
      "https://honkai-star-rail.fandom.com/wiki/Ace_Trash_Digger",
      "Possible Outcomes — Simulated Universe",
      "198aeb225bc3465dcfe0fbe564fa5b020bd0f7a740c476dc37cffd7f4a5b91ed",
      ["exchange:discard-one-curio:obtain-two-random-normal-curios", "leave:no-change"],
    ),
    source(
      "https://honkai-star-rail.fandom.com/wiki/Shopping_Channel",
      "Possible Outcomes — Simulated Universe",
      "3a51d51dd8c82966d6c11975d0751fd6e138ffb202a7d6bb350889e41a2df38f",
      [
        "doughnuts:80-heal-to-full:20-current-hp-minus-20-percent",
        "lotus:80-one-star-to-two-star:20-two-star-to-one-star",
        "mechanical-box:80-normal-curio:20-negative-curio",
        "leave:no-change",
      ],
    ),
    source(
      "https://honkai-star-rail.fandom.com/wiki/Insights_from_the_Universal_Dancer",
      "Possible Outcomes — Simulated Universe",
      "dca8be0a9a9bea18aa9b598d83e32ff0d2b1cf7e94a62bc543b5519c357269df",
      [
        "fortune-cost:50-fragments",
        "success-chance:30-65-100:select-one-three-star-blessing",
        "failure:obtain-100-fragments:retry",
        "refuse:no-change",
      ],
    ),
    source(
      "https://honkai-star-rail.fandom.com/wiki/Mirror_of_Transcendence_%28I%29",
      "Possible Outcomes — Simulated Universe",
      "fd01834738902de1d602af8dc684651f3b355dc96462c26e125403d59f28a5cf",
      [
        "light-candle:advance-stage",
        "random-wish:10-20-5-20-10-10-10-15-weighted-outcomes",
        "three-random-wishes:unlock-part-three",
        "leave:no-change",
      ],
    ),
    raw("Event0010401/Opt0010401.json", "3fe81243aa4223edb42da8e98fb1f61c903a27e413335e2d9b594f57111b830b"),
    raw("Event0014401/Opt0014401.json", "d0c013975c82138328a9d498a02c969ec2d065ff9c9174f79dc7ab508322082f"),
    raw("Event0014402/Opt0014402.json", "21fafd17582729d95bc874a17aabdd8455efe0fe877f9546d5e0f50b6b13e644"),
    raw("Event0014501/Opt0014501.json", "f9c9582b9f9532b147d8628e570460d40fed2ee5ae3539987289e3dae412ca3c"),
    raw("Event0019301/Opt0019301.json", "2358bc6c2954962ffef15a701cc15b5e06392ef4b31228ec9f7c3370b544f35a"),
    raw("Event0019501/Opt0019501.json", "eda05c56d009484e90ef4c4df4d57a33825a8edaef00cae4e5c69813e7fd52c2"),
  ],
};

const write = process.argv.includes("--write");
const encodedChoices = `${JSON.stringify(choices, null, 2)}\n`;
const encodedReview = `${JSON.stringify(review, null, 2)}\n`;
if (write) {
  fs.writeFileSync(choicesPath, encodedChoices);
  fs.mkdirSync(path.dirname(reviewPath), { recursive: true });
  fs.writeFileSync(reviewPath, encodedReview);
  refreshPackIndex();
  console.log("Wrote Goal 07 Occurrence S11 source refinement.");
} else {
  assert.equal(fs.readFileSync(choicesPath, "utf8"), encodedChoices);
  assert.equal(fs.readFileSync(reviewPath, "utf8"), encodedReview);
  assertPackIndex();
  console.log("Goal 07 Occurrence S11 source refinement is stable.");
}

function setOutcome(id, value) {
  const choice = byId.get(id);
  assert(choice, `missing ${id}`);
  choice.outcomes = [{
    kinds: value.kinds,
    targets: value.targets ?? [],
    numeric_literals: value.numeric_literals ?? [],
    parameter_refs: value.parameter_refs ?? [],
    chance_percentages: value.chance_percentages ?? [],
    unspecified_random_policy: value.unspecified_random_policy ?? "",
  }];
  choice.costs = value.costs ?? [];
  choice.mechanism_quality = "ExactStructured";
  choice.quality_overrides = [];
  choice.note = value.note ?? "";
}

function source(url, section, evidence_sha256, facts) {
  return { url, section, evidence_sha256, facts };
}

function raw(relative, evidence_sha256) {
  return source(
    `https://gitlab.com/Dimbreath/turnbasedgamedata/-/raw/fd978d6ef09f941fba644c731ab54abd6f7c3568/Config/Level/Rogue/RogueDialogue/${relative}`,
    "OptionList",
    evidence_sha256,
    [`source-option:${relative.match(/Opt00(\d+)/)?.[1]}`],
  );
}

function refreshPackIndex() {
  const index = JSON.parse(fs.readFileSync(indexPath, "utf8"));
  const entry = index.files.find((value) => value.file === "occurrence-choices.json");
  assert(entry);
  const bytes = fs.readFileSync(choicesPath);
  entry.bytes = bytes.length;
  entry.rows = choices.length;
  entry.sha256 = crypto.createHash("sha256").update(bytes).digest("hex");
  index.files.sort((left, right) => left.file.localeCompare(right.file));
  index.pack_sha256 = crypto.createHash("sha256").update(
    index.files.map((value) => `${value.file}\0${value.sha256}`).join("\n"),
  ).digest("hex");
  fs.writeFileSync(indexPath, `${JSON.stringify(index, null, 2)}\n`);
}

function assertPackIndex() {
  const index = JSON.parse(fs.readFileSync(indexPath, "utf8"));
  const entry = index.files.find((value) => value.file === "occurrence-choices.json");
  const bytes = fs.readFileSync(choicesPath);
  assert.equal(entry.bytes, bytes.length);
  assert.equal(entry.rows, choices.length);
  assert.equal(entry.sha256, crypto.createHash("sha256").update(bytes).digest("hex"));
  assert.equal(
    index.pack_sha256,
    crypto.createHash("sha256").update(
      [...index.files]
        .sort((left, right) => left.file.localeCompare(right.file))
        .map((value) => `${value.file}\0${value.sha256}`)
        .join("\n"),
    ).digest("hex"),
  );
}
