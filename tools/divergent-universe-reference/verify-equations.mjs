#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sourceRoot = path.resolve(
  valueAfter("--source-cache")
    ?? path.join(root, ".cache/content-reference/turnbasedgamedata"),
);
execFileSync(process.execPath, [
  "tools/divergent-universe-reference/import-equations.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });
const outputRoot = path.join(root, "content-reference/divergent-universe-v1");
const fileNames = [
  "equations.json",
  "equation-recipes.json",
  "equation-categories.json",
  "equation-offers.json",
  "equation-progress.json",
  "equation-expansion-states.json",
  "equation-effects.json",
  "equation-replacement-rules.json",
];
const data = Object.fromEntries(fileNames.map((file) =>
  [file, json(path.join(outputRoot, file))]));
const expected = {
  "equations.json": 80,
  "equation-recipes.json": 80,
  "equation-categories.json": 4,
  "equation-offers.json": 136,
  "equation-progress.json": 80,
  "equation-expansion-states.json": 160,
  "equation-effects.json": 25,
  "equation-replacement-rules.json": 4,
};
for (const [file, count] of Object.entries(expected)) {
  assert(data[file].length === count, `${file} row count drift`);
  assert(unique(data[file].map(({ id }) => id)), `${file} duplicate IDs`);
  assert(data[file].every(validEnvelope), `${file} invalid envelope`);
}

const manifest = json(path.join(
  root,
  "content-manifests/divergent-universe-v1/content-manifest.json",
));
const expectedSources = new Map();
for (const categoryId of [
  "equations",
  "equation_displays",
  "equation_randomizers",
  "equation_keywords",
  "equation_keyword_params",
])
  for (const record of manifest.categories[categoryId].records)
    expectedSources.set(record.source, {
      digest: record.evidence_sha256,
      categoryId,
    });
const actualSources = new Map();
for (const rows of Object.values(data))
  for (const row of rows)
    for (const ref of row.source_refs)
      if (expectedSources.has(`${ref.path}#${ref.locator}`))
        actualSources.set(`${ref.path}#${ref.locator}`, ref.sha256);
assert(actualSources.size === 330,
  "Equation manifest obligations are not all accounted");
for (const [locator, expectedSource] of expectedSources)
  assert(actualSources.get(locator) === expectedSource.digest,
    `${expectedSource.categoryId}/${locator} receipt drift`);

const equations = data["equations.json"];
const recipes = new Map(data["equation-recipes.json"].map((row) =>
  [row.id, row]));
const statesByEquation = Map.groupBy(
  data["equation-expansion-states.json"],
  (row) => row.equation_id,
);
assert(equations.every((row) =>
  row.category
    && recipes.has(row.recipe_id)
    && statesByEquation.get(row.id)?.length === 2
    && row.effect_ids.length === 1
    && row.runtime_lowered === false),
"Equation definition/recipe/state closure drift");
assert(JSON.stringify(Object.fromEntries(
  [...Map.groupBy(equations, (row) => row.category)]
    .map(([category, rows]) => [category, rows.length]).sort(),
)) === JSON.stringify({ Epic: 24, Legendary: 16, PathEcho: 8, Rare: 32 }),
"Equation category distribution drift");
assert(equations.every((row) =>
  ["121", "122", "124", "125", "126", "127", "128", "129"]
    .includes(row.main_path_type_id)
    && (!row.sub_path_type_id
      || ["121", "122", "124", "125", "126", "127", "128", "129"]
        .includes(row.sub_path_type_id))),
"Equation Path selector closure drift");
assert([...recipes.values()].every((row) =>
  row.main_path_count > 0
    && row.sub_path_count >= 0
    && row.contribution_unit === "OwnedBlessingIdentity"),
"Equation recipe contribution boundary drift");

const offers = data["equation-offers.json"];
assert(offers.every((row) =>
  row.coverage_state === "Researched"
    && row.candidate_ids.length === 0
    && row.weight_program === "Unspecified"
    && row.no_legal_candidate === "Unspecified"),
"unpublished Equation offer set/weight was invented");
const effects = data["equation-effects.json"];
assert(effects.filter(({ current_path: current }) => current).length === 23,
  "current-path Equation keyword count drift");
assert(effects.filter(({ parameters }) => parameters.length > 0).length === 9,
  "Equation keyword parameter closure drift");
assert(effects.every(({ runtime_lowered: lowered }) => lowered === false),
  "Equation keyword was runtime-lowered");

const transitions = data["equation-replacement-rules.json"];
assert(transitions.every((row) =>
  row.evidence_quality === "ProjectPolicy"
    && row.no_legal_candidate === "RejectWithoutMutation"
    && row.source_refs.some((ref) => ref.replacement_condition)),
"Equation transition policy/fallback drift");

const digest = crypto.createHash("sha256");
for (const file of fileNames.sort())
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Divergent Universe Equations verified ` +
  `(${Object.values(data).flat().length} rows; 330 manifest receipts; ` +
  `digest ${digest.digest("hex")}).`,
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

function validEnvelope(row) {
  return row.schema_revision === "starclock.divergent-universe-row.v1"
    && row.name_en
    && row.name_zh_cn
    && row.summary_en
    && row.summary_zh_cn
    && row.source_refs.length > 0
    && row.source_refs.every((ref) => /^[0-9a-f]{64}$/u.test(ref.sha256));
}

function unique(values) {
  return new Set(values).size === values.length;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
