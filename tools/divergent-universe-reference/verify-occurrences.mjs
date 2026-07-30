#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sourceRoot = path.resolve(
  valueAfter("--source-cache")
    ?? path.join(root, ".cache/content-reference/turnbasedgamedata"),
);
execFileSync(process.execPath, [
  "tools/divergent-universe-reference/import-occurrences.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/divergent-universe-v1");
const occurrences = json(path.join(outputRoot, "occurrences.json"));
const variants = json(path.join(outputRoot, "occurrence-variants.json"));
const choices = json(path.join(outputRoot, "occurrence-choices.json"));
const manifest = json(path.join(
  root,
  "content-manifests/divergent-universe-v1/content-manifest.json",
));
assert(occurrences.length === 118, "Occurrence identity count drift");
assert(variants.length === 97, "Occurrence variant count drift");
assert(choices.length === 0, "missing Tourn3 graphs cannot yield choices");
assert(new Set([...occurrences, ...variants].map((row) => row.id)).size === 215,
  "Occurrence/variant IDs are not unique");
assert(exactOnce(
  occurrences.map((row) => row.source_id),
  manifest.categories.occurrences.records.map((row) => row.id),
), "Occurrence manifest exact-once drift");
assert(exactOnce(
  variants.map((row) => row.source_id),
  manifest.categories.occurrence_variants.records.map((row) => row.id),
), "Occurrence variant manifest exact-once drift");

const occurrenceIds = new Set(occurrences.map((row) => row.id));
const variantIds = new Set(variants.map((row) => row.id));
assert(occurrences.every((row) =>
  row.coverage_state === "Researched"
    && row.evidence_quality === "ProjectPolicy"
    && row.variant_ids.length === 1
    && row.variant_ids.every((id) => variantIds.has(id))
    && row.choice_ids.length === 0
    && row.unlock_rules.length > 0
    && row.unresolved_offer_behavior === "FailClosed"
    && row.runtime_lowered === false),
"Occurrence identity boundary drift");
assert(variants.every((row) =>
  row.coverage_state === "Researched"
    && row.evidence_quality === "ProjectPolicy"
    && row.occurrence_ids.length >= 1
    && row.occurrence_ids.every((id) => occurrenceIds.has(id))
    && row.occurrence_id === row.occurrence_ids[0]
    && row.graph_resolution === "MissingAtPinnedRevision"
    && row.choice_ids.length === 0
    && row.fallback === "RejectWithoutMutation"
    && row.runtime_lowered === false),
"Occurrence variant fail-closed boundary drift");
assert(variants.filter((row) => row.occurrence_ids.length === 1).length === 83
  && variants.filter((row) => row.occurrence_ids.length === 2).length === 7
  && variants.filter((row) => row.occurrence_ids.length === 3).length === 7,
"Occurrence multi-handbook variant distribution drift");

for (const variant of variants) {
  const result = spawnSync("git", [
    "cat-file",
    "-e",
    `fd978d6ef09f941fba644c731ab54abd6f7c3568:${variant.graph_path}`,
  ], { cwd: sourceRoot, stdio: "ignore" });
  assert(result.status !== 0,
    `${variant.id} graph unexpectedly exists at the fixed revision`);
}

const digest = crypto.createHash("sha256");
for (const file of [
  "occurrences.json",
  "occurrence-variants.json",
  "occurrence-choices.json",
])
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Divergent Universe Occurrences verified (118 identities; 97 variants; ` +
  `97 missing published graph paths; digest ${digest.digest("hex")}).`,
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}

function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function exactOnce(left, right) {
  return JSON.stringify([...left].sort())
    === JSON.stringify([...right].sort());
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
