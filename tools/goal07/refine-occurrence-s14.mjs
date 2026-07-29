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
  "evidence/standard-universe-mechanics-complete-v1/source-reviews/G07-P4-M13-S14.json",
);
const choices = JSON.parse(fs.readFileSync(choicesPath, "utf8"));
const byId = new Map(choices.map((choice) => [choice.id, choice]));
const marker = (value) => `universe.occurrence-s14.${value}`;

for (const ordinal of [17, 18]) {
  setOutcome(
    `universe.occurrence.77.variant.19501.choice.${String(ordinal).padStart(2, "0")}`,
    {
      kinds: ["Special"],
      parameter_refs: [marker("mirror-no-effect")],
    },
  );
}
for (let ordinal = 1; ordinal <= 18; ordinal += 1) {
  setOutcome(
    `universe.occurrence.78.variant.19501.choice.${String(ordinal).padStart(2, "0")}`,
    {
      kinds: ["Special"],
      parameter_refs: [marker("mirror-no-effect")],
    },
  );
}

setOutcome("universe.occurrence.8.variant.10601.choice.01", {
  kinds: ["Obtain"],
  targets: ["Blessing"],
  numeric_literals: ["1"],
  parameter_refs: ["universe.blessing-pool.path.elation.rarity.2"],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.8.variant.10601.choice.02", {
  kinds: ["Obtain"],
  targets: ["Blessing"],
  numeric_literals: ["1"],
  parameter_refs: ["universe.blessing-pool.path.hunt.rarity.2"],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.8.variant.10601.choice.03", {
  kinds: ["Restore"],
  targets: ["HP"],
  numeric_literals: ["100%"],
});

setOutcome("universe.occurrence.9.variant.10701.choice.01", {
  kinds: ["Obtain"],
  targets: ["CosmicFragments"],
  numeric_literals: ["200"],
});
setOutcome("universe.occurrence.9.variant.10701.choice.02", {
  kinds: ["Lose", "Obtain"],
  targets: ["HP", "Blessing"],
  numeric_literals: ["20%", "2"],
  parameter_refs: ["universe.blessing-pool.rarity.1"],
  unspecified_random_policy: "StableUniformOrderedCandidates",
  costs: [{ kind: "Lose", targets: ["HP"] }],
});

const review = {
  schema_revision: "starclock.goal07-occurrence-source-review.v1",
  partition_id: "G07-P4-M13-S14",
  reviewed_on: "2026-07-28",
  frozen_source_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  sources: [
    source(
      "https://honkai-star-rail.fandom.com/wiki/Mirror_of_Transcendence_%28III%29",
      "Possible Outcomes — Simulated Universe",
      "6af249295195088f4313ff6a427ce7c5a253b1eb236e9f34a9ff13bae7095196",
      [
        "requirement:three-random-wishes-current-run",
        "any-choice:no-gameplay-effect",
        "dialogue:all-three-candles-lit:leave",
      ],
    ),
    source(
      "https://honkai-star-rail.fandom.com/wiki/Interactive_Arts",
      "Possible Outcomes — Simulated Universe",
      "d1a397954ed6d2751070893e18371ad4a7beea07e9262fb0df2e58902ad0ac74",
      [
        "musical:obtain-one-two-star-elation-blessing",
        "action:obtain-one-two-star-hunt-blessing",
        "please-let-me-live:heal-all-characters-100-percent-maximum-hp",
      ],
    ),
    source(
      "https://honkai-star-rail.fandom.com/wiki/Pixel_World",
      "Possible Outcomes — Simulated Universe",
      "ccb0aaf559375cbd614541b4dcf92f20a936ad388c284370dd50269d084dd9eb",
      [
        "left-pipe:obtain-200-cosmic-fragments",
        "right-bricks:lose-20-percent-current-hp-all-characters",
        "right-bricks:obtain-two-random-one-star-blessings",
      ],
    ),
    raw("Event0010601/Opt0010601.json", "cb7df52b7275b36e6b3cfa6538302aa68d116d37cf399bb3bbda5c958105eddb"),
    raw("Event0010701/Opt0010701.json", "88286d772e64a092031aea2e3034a7862c8a0d97855bfb7bf3ea7f616d2847cb"),
    raw("Event0019501/Opt0019501.json", "eda05c56d009484e90ef4c4df4d57a33825a8edaef00cae4e5c69813e7fd52c2"),
    raw("Event0019502/Opt0019502.json", "e17eaef6bdcb021f2df6fdae92926ad338826783ac96ab48a44b4a2bde32ecc5"),
    raw("Event0019503/Opt0019503.json", "cb3251ade4018bd38cc1aa5c7853eb83c6882bcdad68cf6ecca8cb52a14844f9"),
    raw("Event0019504/Opt0019504.json", "249ba9ffaaa00e660f2e640162a366e81746fe6057179b0d457e585fb556372a"),
    raw("Event0019505/Opt0019505.json", "801dceb0b1fb481e1e67a07487c30b2b8cee3442599426e16c1d0366adf0eb1c"),
    raw("Event0019506/Opt0019506.json", "ad4f9596569e12371eca40cbffa6875b354e437de33d293ef171be19be1fd244"),
    raw("Event0019507/Opt0019507.json", "249ba9ffaaa00e660f2e640162a366e81746fe6057179b0d457e585fb556372a"),
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
  console.log("Wrote Goal 07 Occurrence S14 source refinement.");
} else {
  assert.equal(fs.readFileSync(choicesPath, "utf8"), encodedChoices);
  assert.equal(fs.readFileSync(reviewPath, "utf8"), encodedReview);
  assertPackIndex();
  console.log("Goal 07 Occurrence S14 source refinement is stable.");
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
