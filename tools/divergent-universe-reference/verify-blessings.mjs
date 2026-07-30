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
  "tools/divergent-universe-reference/import-blessings.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });
const outputRoot = path.join(root, "content-reference/divergent-universe-v1");
const fileNames = [
  "blessing-paths.json",
  "blessings.json",
  "blessing-levels.json",
  "blessing-groups.json",
  "blessing-rewrite-rules.json",
  "blessing-equation-contributions.json",
];
const data = Object.fromEntries(fileNames.map((file) =>
  [file, json(path.join(outputRoot, file))]));
const expected = {
  "blessing-paths.json": 8,
  "blessings.json": 414,
  "blessing-levels.json": 828,
  "blessing-groups.json": 118,
  "blessing-rewrite-rules.json": 416,
  "blessing-equation-contributions.json": 414,
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
  "blessing_paths",
  "blessings",
  "blessing_levels",
  "blessing_groups",
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
assert(actualSources.size === 954,
  "Blessing unique source receipts are not all accounted");
for (const [locator, expectedSource] of expectedSources)
  assert(actualSources.get(locator) === expectedSource.digest,
    `${expectedSource.categoryId}/${locator} receipt drift`);

const paths = data["blessing-paths.json"];
assert(paths.map(({ path_type_id: id }) => id).sort(compare).join(",")
  === "121,122,124,125,126,127,128,129",
"active Blessing Path selector drift");
const blessings = data["blessings.json"];
const levels = data["blessing-levels.json"];
const levelsByBlessing = Map.groupBy(levels, (row) => row.blessing_id);
assert(blessings.every((row) =>
  levelsByBlessing.get(row.id)?.length === 2
    && row.level_ids.every((id) =>
      levelsByBlessing.get(row.id).some((level) => level.id === id))),
"Blessing base/enhanced level closure drift");
assert(JSON.stringify(Object.fromEntries(
  [...Map.groupBy(blessings, (row) => row.category)]
    .map(([category, rows]) => [category, rows.length]).sort(),
)) === JSON.stringify({ Common: 184, Legendary: 69, Rare: 161 }),
"Blessing category distribution drift");
assert(JSON.stringify(Object.fromEntries(
  [...Map.groupBy(blessings, (row) => row.path_type_id)]
    .map(([type, rows]) => [type, rows.length]).sort(),
)) === JSON.stringify({
  121: 54,
  122: 54,
  124: 54,
  125: 54,
  126: 54,
  127: 54,
  128: 54,
  129: 36,
}), "Blessing Path distribution drift");
assert(levels.every((row) =>
  row.binding_key
    && row.parameters.length > 0
    && row.equation_contribution_identity
    && row.runtime_lowered === false),
"Blessing level binding/contribution boundary drift");

const groups = data["blessing-groups.json"];
assert(JSON.stringify(Object.fromEntries(
  [...Map.groupBy(groups, (row) => row.source_candidate_ids.length)]
    .map(([size, rows]) => [size, rows.length]),
)) === JSON.stringify({ 2: 37, 3: 34, 7: 25, 8: 22 }),
"Blessing group size distribution drift");
assert(groups.every((row) =>
  row.source_candidate_ids.length
    === row.resolved_mode_level_ids.length
      + row.resolved_subgroup_ids.length
      + row.unresolved_source_ids.length
    && row.weight_program === "Unspecified"),
"Blessing group exact-once candidate classification drift");
assert(groups.every((row) =>
  row.coverage_state === "DataReady"
    && row.evidence_quality === "ExactStructured"
    && row.unresolved_source_ids.length === 0),
"Blessing groups are not fully closed");
assert(groups.reduce((count, row) =>
  count + row.resolved_mode_level_ids.length, 0) === 351,
"Blessing direct terminal membership count drift");
assert(groups.reduce((count, row) =>
  count + row.resolved_subgroup_ids.length, 0) === 176,
"Blessing nested subgroup membership count drift");

const rewriteRows = data["blessing-rewrite-rules.json"];
assert(rewriteRows.filter(({ evidence_quality: quality }) =>
  quality === "ExactStructured").length === 414,
"Blessing enhancement transition count drift");
assert(rewriteRows.filter(({ evidence_quality: quality }) =>
  quality === "ProjectPolicy").length === 2,
"Blessing generic rewrite policy count drift");
assert(rewriteRows.every((row) =>
  row.no_legal_candidate === "RejectWithoutMutation"
    && row.runtime_lowered === false),
"Blessing rewrite fallback/runtime boundary drift");

const contributions = data["blessing-equation-contributions.json"];
const equations = json(path.join(outputRoot, "equations.json"));
const equationById = new Map(equations.map((row) => [row.id, row]));
assert(contributions.every((row) =>
  row.contribution === 1
    && row.base_and_enhanced_count_equally
    && row.equation_ids.length > 0
    && row.equation_ids.every((id) => {
      const equation = equationById.get(id);
      return equation
        && [equation.main_path_type_id, equation.sub_path_type_id]
          .includes(row.path_type_id);
    })),
"Blessing-to-Equation contribution closure drift");

const digest = crypto.createHash("sha256");
for (const file of fileNames.sort())
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Divergent Universe Blessings verified ` +
  `(${Object.values(data).flat().length.toLocaleString("en-US")} rows; ` +
  `1,368 manifest receipts; digest ${digest.digest("hex")}).`,
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

function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
