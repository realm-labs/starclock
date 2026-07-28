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
  "evidence/standard-universe-mechanics-complete-v1/source-reviews/G07-P4-M13-S10.json",
);
const choices = JSON.parse(fs.readFileSync(choicesPath, "utf8"));
const byId = new Map(choices.map((choice) => [choice.id, choice]));
const marker = (value) => `universe.occurrence-s10.${value}`;

setOutcome("universe.occurrence.47.variant.13001.choice.12", {
  kinds: ["Obtain"],
  targets: ["Curio"],
  numeric_literals: ["1"],
  parameter_refs: [
    "universe.occurrence-s09.perfect-popular-first",
    "universe.curio-pool.polarity.positive",
  ],
  chance_percentages: ["40", "60"],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});

setOutcome("universe.occurrence.5.variant.10301.choice.01", {
  kinds: ["Obtain"],
  targets: ["Blessing"],
  numeric_literals: ["1"],
  parameter_refs: [
    "universe.blessing-pool.rarity.1",
    "universe.blessing-pool.rarity.2",
  ],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.5.variant.10301.choice.02", {
  kinds: ["Obtain"],
  targets: ["CosmicFragments"],
  numeric_literals: ["100"],
});

for (const occurrence of [52, 53]) {
  setBank(occurrence, 1, "deposit", 200, 100);
  setBank(occurrence, 2, "deposit", 400, 150);
  setBank(occurrence, 3, "deposit", 600, 200);
  setBank(occurrence, 4, "leave", 0);
  setBank(occurrence, 5, "withdraw", 200);
  setBank(occurrence, 6, "preserve", 200);
  setBank(occurrence, 7, "withdraw", 400);
  setBank(occurrence, 8, "preserve", 400);
  setBank(occurrence, 9, "withdraw", 600);
  setBank(occurrence, 10, "preserve", 600);
}

setOutcome("universe.occurrence.54.variant.14401.choice.01", {
  kinds: ["Discard", "Lose", "Special"],
  targets: ["Curio", "HP"],
  numeric_literals: ["1", "30"],
  parameter_refs: [marker("beauty-bug-feed-curio")],
  chance_percentages: ["70", "30"],
  unspecified_random_policy: "StableUniformOrderedCandidates",
  costs: [{ kind: "Discard", targets: ["Curio"] }],
});

const review = {
  schema_revision: "starclock.goal07-occurrence-source-review.v1",
  partition_id: "G07-P4-M13-S10",
  reviewed_on: "2026-07-28",
  frozen_source_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  sources: [
    source(
      "https://honkai-star-rail.fandom.com/wiki/Jim_Hulk_and_Jim_Hall",
      "Possible Outcomes — Simulated Universe",
      "cf83dbcae6c8fa95e487602cccd811429cdf554aae394ff132ac91dad805b966",
      ["collection:one-random-one-or-two-star-blessing", "walk-away:fragments-100"],
    ),
    source(
      "https://honkai-star-rail.fandom.com/wiki/Ka-ching%21_IPC_Banking_%28I%29",
      "Possible Outcomes — Simulated Universe",
      "39679b9b342cf8eb73b788be87342189ebbc5c9db488990dcd17cd11143532c4",
      [
        "deposit-100:withdraw-200",
        "deposit-150:withdraw-400",
        "deposit-200:withdraw-600",
        "leave:no-change",
      ],
    ),
    source(
      "https://honkai-star-rail.fandom.com/wiki/Ka-ching%21_IPC_Banking_%28II%29",
      "Possible Outcomes — Simulated Universe",
      "e39107940fed2c5598240e53b8dd0a2ccf6b80240a131d6651001e46fa59135c",
      [
        "withdraw:fragments-200-or-400-or-600:reset-to-part-one",
        "leave:stored-fragments-remain",
      ],
    ),
    source(
      "https://honkai-star-rail.fandom.com/wiki/Loneliness%2C_Cosmic_Beauty_Bugs%2C_Simulated_Universe_%28I%29",
      "Possible Outcomes — Simulated Universe",
      "3371c357ce92dcc7037d6e744c733ec17ea2b18587f76ec5ea0a8d2db7c2aa93",
      [
        "feed-curio:discard-one",
        "success-70:unlock-part-two",
        "failure-30:all-allies-current-hp-minus-30-percent",
      ],
    ),
    raw("Event0010301/Opt0010301.json", "1e8ce8c0d17b3c790d1fda6cfd8f7a738e62ffef175a25b76275817770420d83"),
    raw("Event0014301/Opt0014301.json", "519b08c82e1dda48ecd9c8802c3a6ceb06c719495dd5b6afb9e2242b444c0ffd"),
    raw("Event0014302/Opt0014302.json", "682a0cb794c28bdd1a2ee4183dcdf404cecce6c7dcd053f77c244864e7061245"),
    raw("Event0014303/Opt0014303.json", "378b117d1ef5d445107a10bf42b6f450152b71534a4915084712dd38903218c6"),
    raw("Event0014304/Opt0014304.json", "3f35288b7385740206fefb5324df866f7b9a8a2cd466d59e687ba2923fc5b99a"),
    raw("Event0014401/Opt0014401.json", "d0c013975c82138328a9d498a02c969ec2d065ff9c9174f79dc7ab508322082f"),
    raw("Event0014402/Opt0014402.json", "21fafd17582729d95bc874a17aabdd8455efe0fe877f9546d5e0f50b6b13e644"),
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
  console.log("Wrote Goal 07 Occurrence S10 source refinement.");
} else {
  assert.equal(fs.readFileSync(choicesPath, "utf8"), encodedChoices);
  assert.equal(fs.readFileSync(reviewPath, "utf8"), encodedReview);
  assertPackIndex();
  console.log("Goal 07 Occurrence S10 source refinement is stable.");
}

function setBank(occurrence, choice, action, amount, cost = 0) {
  const id =
    `universe.occurrence.${occurrence}.variant.14301.choice.${String(choice).padStart(2, "0")}`;
  const deposit = action === "deposit";
  const withdraw = action === "withdraw";
  setOutcome(id, {
    kinds: deposit ? ["Consume", "Special"] : withdraw ? ["Obtain", "Special"] : ["Special"],
    targets: deposit || withdraw ? ["CosmicFragments"] : ["Character"],
    numeric_literals: deposit ? [String(cost), String(amount)] : amount ? [String(amount)] : [],
    parameter_refs: [marker(`bank-${action}-${amount}`)],
    costs: [],
  });
}

function setOutcome(id, value) {
  const choice = byId.get(id);
  assert(choice, `missing ${id}`);
  choice.outcomes = [{
    kinds: value.kinds,
    targets: value.targets,
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
