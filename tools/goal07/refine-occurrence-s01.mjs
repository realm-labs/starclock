#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const write = process.argv.slice(2).includes("--write");
assert(
  process.argv.slice(2).every((argument) => argument === "--write"),
  "usage: node tools/goal07/refine-occurrence-s01.mjs [--write]",
);

const packRoot = path.join(root, "content-reference", "standard-universe-v1");
const choicesPath = path.join(packRoot, "occurrence-choices.json");
const sourcesPath = path.join(packRoot, "sources.json");
const indexPath = path.join(packRoot, "pack-index.json");
const reviewPath = path.join(
  root,
  "evidence",
  "standard-universe-mechanics-complete-v1",
  "source-reviews",
  "G07-P4-M13-S01.json",
);
const originalChoices = fs.readFileSync(choicesPath, "utf8");
const originalSources = fs.readFileSync(sourcesPath, "utf8");
const choices = JSON.parse(originalChoices);
const sources = JSON.parse(originalSources).filter(
  ({ id }) => id !== "source.supplemental.standard-su-occurrence-stephen-lloyd-idea",
);
const byId = new Map(choices.map((choice) => [choice.id, choice]));

const sourceReview = {
  schema_revision: "starclock.goal07-occurrence-source-review.v1",
  partition_id: "G07-P4-M13-S01",
  reviewed_on: "2026-07-27",
  frozen_pack_source_count: sources.length,
  sources: [
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Stephen_Lloyd%27s_Idea",
      section: "Possible Outcomes",
      evidence_sha256:
        "fb064c22fd19eb77bc9b2ee964aed6fc621c56e617fb542bafbd12c010513d87",
      facts: ["startup-capital:cosmic-fragments:150"],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Aha_Stuffed_Toy",
      section: "Possible Outcomes",
      evidence_sha256:
        "8a665585c95e53e45871c30102946660db63a25ff76c8574a712dda19e1ad912",
      facts: ["normal-inputs-do-not-change-result"],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/I.O.U._Dispenser_%28I%29",
      section: "Possible Outcomes",
      evidence_sha256:
        "45df4c73d4c8a05e1dc64f310c1de5d59d481b03e1bdb2ed0b1dc750e5cce854",
      facts: [
        "named-curio:universe.curio.60",
        "decline:cosmic-fragments:100",
      ],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/I.O.U._Dispenser_%28II%29",
      section: "Possible Outcomes",
      evidence_sha256:
        "d14514d9775feea723f6620fccdd897f6a3ac9555ce470861239ff36e83ae700",
      facts: [
        "named-curio:universe.curio.60",
        "decline:cosmic-fragments:100",
      ],
    },
  ],
};

const stephen = required("universe.occurrence.1.variant.40398.choice.01");
stephen.outcomes[0].numeric_literals = ["150"];
stephen.provenance_ids = stephen.provenance_ids.filter(
  (id) => id !== "source.supplemental.standard-su-occurrence-stephen-lloyd-idea",
);
stephen.note =
  "Choice/result text is exact public TextMap evidence; the supplemental public outcome table fixes the qualitative reward at 150 Cosmic Fragments.";

const s01IouChoices = [
  { occurrence: 11, curios: ["01", "05", "06", "08"], fragments: ["04", "07", "09"] },
  { occurrence: 12, curios: ["01", "05", "06"], fragments: ["04"] },
];
for (const { occurrence, curios, fragments } of s01IouChoices) {
  for (const choice of curios) {
    const row = required(
      `universe.occurrence.${occurrence}.variant.10901.choice.${choice}`,
    );
    row.outcomes[0].parameter_refs = ["universe.curio.60"];
    row.note =
      "Choice/result text names Angel-type I.O.U. Dispenser exactly; the stable Curio reference prevents fallback to a random Curio pool.";
  }
  for (const choice of fragments) {
    const row = required(
      `universe.occurrence.${occurrence}.variant.10901.choice.${choice}`,
    );
    row.outcomes[0].numeric_literals = ["100"];
    row.note =
      "Choice/result TextMap and dialogue option value identify an exact 100-Cosmic-Fragment result.";
  }
}

const encodedChoices = `${JSON.stringify(choices, null, 2)}\n`;
const encodedSources = `${JSON.stringify(sources, null, 2)}\n`;
const encodedReview = `${JSON.stringify(sourceReview, null, 2)}\n`;
if (write) {
  fs.writeFileSync(choicesPath, encodedChoices);
  fs.writeFileSync(sourcesPath, encodedSources);
  fs.mkdirSync(path.dirname(reviewPath), { recursive: true });
  fs.writeFileSync(reviewPath, encodedReview);
  fs.writeFileSync(indexPath, `${JSON.stringify(buildIndex(), null, 2)}\n`);
  console.log("Refined Goal 07 Occurrence S01 source evidence and pack index.");
} else {
  assert(originalChoices === encodedChoices, "Occurrence S01 normalized choices drifted");
  assert(originalSources === encodedSources, "Occurrence S01 supplemental source drifted");
  assert(
    fs.readFileSync(reviewPath, "utf8") === encodedReview,
    "Occurrence S01 source review drifted",
  );
  const expectedIndex = `${JSON.stringify(buildIndex(), null, 2)}\n`;
  assert(
    fs.readFileSync(indexPath, "utf8") === expectedIndex,
    "Occurrence S01 pack index drifted",
  );
  console.log("Goal 07 Occurrence S01 source refinement is stable.");
}

function required(id) {
  const value = byId.get(id);
  assert(value, `${id}: normalized choice is missing`);
  assert(value.outcomes?.length === 1, `${id}: expected one normalized outcome`);
  return value;
}

function buildIndex() {
  const existing = JSON.parse(fs.readFileSync(indexPath, "utf8"));
  const files = existing.files
    .map(({ file, rows }) => {
      const bytes = fs.readFileSync(path.join(packRoot, file));
      const parsed = JSON.parse(bytes);
      return {
        file,
        bytes: bytes.length,
        rows: Array.isArray(parsed) ? parsed.length : rows,
        sha256: sha256(bytes),
      };
    })
    .sort((left, right) => left.file.localeCompare(right.file, "en"));
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

function sha256(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
