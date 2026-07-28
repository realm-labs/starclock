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
  "evidence/standard-universe-mechanics-complete-v1/source-reviews/G07-P4-M13-S12.json",
);
const choices = JSON.parse(fs.readFileSync(choicesPath, "utf8"));
const byId = new Map(choices.map((choice) => [choice.id, choice]));
const marker = (value) => `universe.occurrence-s12.${value}`;
const s11 = (value) => `universe.occurrence-s11.${value}`;
const cuckooCurios = [
  "universe.curio.65",
  "universe.curio.66",
  "universe.curio.67",
  "universe.curio.70",
  "universe.curio.71",
  "universe.curio.108",
];

const mirrorPrefix = "universe.occurrence.62.variant.19501.choice";
for (const ordinal of ["04", "08", "10"]) {
  setOutcome(`${mirrorPrefix}.${ordinal}`, {
    kinds: ["Battle"],
    numeric_literals: ["1"],
    parameter_refs: [
      marker("mirror-rescue"),
      "universe.occurrence-battle.stage.mirror-curse-of-captivity",
      "universe.occurrence-battle.level.48",
      "universe.occurrence-battle.wave.1",
      "enemy.everwinter-shadewalker.minionlv2.variant.01",
      "enemy.everwinter-shadewalker.minionlv2.variant.01",
      "enemy.guardian-shadow.elite.variant.01",
      "universe.occurrence-battle.reward.fixed-blessings.2",
    ],
  });
}
for (const ordinal of ["05", "11", "14", "16"]) {
  setOutcome(`${mirrorPrefix}.${ordinal}`, {
    kinds: ["Special"],
    numeric_literals: ["1"],
    parameter_refs: [s11("mirror-light-candle")],
  });
}
for (const ordinal of ["06", "12", "15", "17"]) {
  setOutcome(`${mirrorPrefix}.${ordinal}`, {
    kinds: ["Special"],
    parameter_refs: [s11("mirror-leave")],
  });
}
for (const ordinal of ["07", "09"]) {
  setOutcome(`${mirrorPrefix}.${ordinal}`, {
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
      s11("mirror-random-wish"),
      "universe.blessing-pool.rarity.2",
      "universe.blessing-pool.rarity.3",
      "universe.curio-pool.polarity.positive",
    ],
    chance_percentages: ["10", "20", "5", "20", "10", "10", "10", "15"],
    unspecified_random_policy: "StableUniformOrderedCandidates",
  });
}
for (const ordinal of ["13", "18"]) {
  setOutcome(`${mirrorPrefix}.${ordinal}`, {
    kinds: ["Special"],
    parameter_refs: [marker("mirror-all-candles-lit")],
  });
}

for (const occurrence of [63, 64]) {
  const prefix = `universe.occurrence.${occurrence}.variant.19601.choice`;
  setOutcome(`${prefix}.01`, {
    kinds: ["Obtain"],
    targets: ["Curio"],
    numeric_literals: ["1"],
    parameter_refs: [marker("cuckoo-acquire-one"), ...cuckooCurios],
    unspecified_random_policy: "StableUniformOrderedCandidates",
  });
  setOutcome(`${prefix}.02`, {
    kinds: ["Obtain"],
    targets: ["Curio"],
    numeric_literals: ["2"],
    parameter_refs: [marker("cuckoo-acquire-two"), ...cuckooCurios],
    unspecified_random_policy: "StableUniformOrderedCandidates",
  });
  setOutcome(`${prefix}.03`, {
    kinds: ["Obtain", "Obtain"],
    targets: ["Curio", "Blessing"],
    numeric_literals: ["1", "1"],
    parameter_refs: [
      marker("cuckoo-acquire-one-rarity-two-blessing"),
      ...cuckooCurios,
      "universe.blessing-pool.rarity.2",
    ],
    unspecified_random_policy: "StableUniformOrderedCandidates",
  });
  setOutcome(`${prefix}.04`, {
    kinds: ["Discard"],
    targets: ["Curio"],
    parameter_refs: [marker("cuckoo-discard-all"), ...cuckooCurios],
    costs: [{ kind: "Discard", targets: ["Curio"] }],
  });
  setOutcome(`${prefix}.05`, {
    kinds: ["Obtain", "Obtain"],
    targets: ["Curio", "Blessing"],
    numeric_literals: ["1", "1"],
    parameter_refs: [
      marker("cuckoo-acquire-one-rarity-three-blessing"),
      ...cuckooCurios,
      "universe.blessing-pool.rarity.3",
    ],
    unspecified_random_policy: "StableUniformOrderedCandidates",
  });
  setOutcome(`${prefix}.06`, {
    kinds: ["Discard", "Obtain"],
    targets: ["Curio", "Curio"],
    parameter_refs: [
      marker("cuckoo-exchange-all-for-curios"),
      ...cuckooCurios,
      "universe.curio-pool.polarity.positive",
    ],
    unspecified_random_policy: "StableUniformOrderedCandidates",
  });
}
setOutcome("universe.occurrence.63.variant.19601.choice.07", {
  kinds: ["Discard", "Obtain"],
  targets: ["Curio", "Blessing"],
  parameter_refs: [
    marker("cuckoo-exchange-all-for-blessings"),
    ...cuckooCurios,
    "universe.blessing-pool.rarity.1",
    "universe.blessing-pool.rarity.2",
    "universe.blessing-pool.rarity.3",
  ],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});

const review = {
  schema_revision: "starclock.goal07-occurrence-source-review.v1",
  partition_id: "G07-P4-M13-S12",
  reviewed_on: "2026-07-28",
  frozen_source_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  sources: [
    source(
      "https://honkai-star-rail.fandom.com/wiki/Mirror_of_Transcendence_%28I%29",
      "Possible Outcomes — Simulated Universe",
      "fd01834738902de1d602af8dc684651f3b355dc96462c26e125403d59f28a5cf",
      [
        "duplicate-state-options:light-wish-battle-leave",
        "rescue-battle:bug-elite",
        "victory-reward:two-three-star-blessings",
        "victory-unlock:mirror-part-two",
      ],
    ),
    source(
      "https://honkai-star-rail.fandom.com/wiki/The_Cuckoo_Clock_Fanatic_%28I%29",
      "Possible Outcomes — Simulated Universe",
      "c83adc7fa5a730093d54790040a86e8d247cae290aedddb8960c8639e30bdd0e",
      [
        "accept:one-random-cuckoo-clock-negative-curio",
        "refuse:two-random-cuckoo-clock-negative-curios",
      ],
    ),
    source(
      "https://honkai-star-rail.fandom.com/wiki/The_Cuckoo_Clock_Fanatic_%28II%29",
      "Possible Outcomes — Simulated Universe",
      "e66fafc91fc591a42035389a52c69e2d771bcbdc2f21a33ce28a5436968f2904",
      [
        "accept-again:one-random-cuckoo-clock-negative-curio:one-two-star-blessing",
        "return:discard-all-cuckoo-clocks",
      ],
    ),
    raw("Event0019502/Opt0019502.json", "e17eaef6bdcb021f2df6fdae92926ad338826783ac96ab48a44b4a2bde32ecc5"),
    raw("Event0019503/Opt0019503.json", "cb3251ade4018bd38cc1aa5c7853eb83c6882bcdad68cf6ecca8cb52a14844f9"),
    raw("Event0019504/Opt0019504.json", "249ba9ffaaa00e660f2e640162a366e81746fe6057179b0d457e585fb556372a"),
    raw("Event0019505/Opt0019505.json", "801dceb0b1fb481e1e67a07487c30b2b8cee3442599426e16c1d0366adf0eb1c"),
    raw("Event0019506/Opt0019506.json", "ad4f9596569e12371eca40cbffa6875b354e437de33d293ef171be19be1fd244"),
    raw("Event0019507/Opt0019507.json", "249ba9ffaaa00e660f2e640162a366e81746fe6057179b0d457e585fb556372a"),
    raw("Event0019601/Opt0019601.json", "14ef1a944b23fe593d393bbb0c6efd1c866a265ddef027947287fbbe26ac12d7"),
    raw("Event0019602/Opt0019602.json", "a9eec45f3a137ea4b311647ce190d84bd60916dbbdc7cbd97ef12c5d623ce7a5"),
    raw("Event0019603/Opt0019603.json", "7c79da34b8f06e6489d878f589428636fde351730c667256b5ee542c2c6b47b2"),
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
  console.log("Wrote Goal 07 Occurrence S12 source refinement.");
} else {
  assert.equal(fs.readFileSync(choicesPath, "utf8"), encodedChoices);
  assert.equal(fs.readFileSync(reviewPath, "utf8"), encodedReview);
  assertPackIndex();
  console.log("Goal 07 Occurrence S12 source refinement is stable.");
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
