#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { canonical, sha256 } from "./lib/common.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const sourceRoot = path.resolve(valueAfter("--source-cache")
  ?? path.join(root, ".cache/content-reference/turnbasedgamedata"));
execFileSync(process.execPath, [
  "tools/currency-wars-reference/generate-pack.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/currency-wars-v1");
const schema = json(path.join(
  root,
  "content-manifests/currency-wars-v1/normalized-schema.json",
));
const manifest = json(path.join(
  root,
  "content-manifests/currency-wars-v1/content-manifest.json",
));
const fixtureContract = json(path.join(
  root,
  "content-manifests/currency-wars-v1/fixture-contract.json",
));
const p2Contracts = schema.files.filter(({ phase }) => phase === "P2-B6");
const rowsByFile = new Map();
for (const contract of schema.files) {
  const target = path.join(outputRoot, contract.file);
  if (!fs.existsSync(target)) continue;
  const rows = json(target);
  rowsByFile.set(contract.file, rows);
  assert(unique(rows.map(({ id }) => id)), `${contract.file} duplicate ID`);
  if (contract.phase !== "P2-B6") continue;
  assert(rows.every((row) =>
    schema.common_envelope.required_fields.every((field) =>
      Object.hasOwn(row, field))), `${contract.file} common envelope drift`);
  assert(rows.every((row) =>
    contract.required_domain_fields.every((field) =>
      Object.hasOwn(row, field))), `${contract.file} domain contract drift`);
}
assert(p2Contracts.every(({ file }) => rowsByFile.has(file)),
  "P2-B6 output missing");

const sources = rowsByFile.get("sources.json");
const sourceKeys = new Set(sources.map((row) => canonical([
  row.repository,
  row.revision,
  row.path,
  row.locator,
  row.sha256,
  row.evidence_quality,
])));
assert(sourceKeys.size === sources.length, "source receipt duplicate");
for (const [file, rows] of rowsByFile)
  for (const row of rows)
    for (const ref of row.source_refs ?? [])
      assert(sourceKeys.has(canonical([
        ref.repository,
        ref.revision,
        ref.path,
        ref.locator,
        ref.sha256,
        ref.evidence_quality,
      ])), `${file}/${row.id} has unresolved source receipt`);

const coverage = rowsByFile.get("coverage.json");
const manifestRows = Object.values(manifest.categories)
  .flatMap(({ records }) => records);
assert(coverage.length === 19250
  && manifestRows.length === coverage.length,
"manifest coverage denominator drift");
const coverageKeys = new Set(coverage.map((row) =>
  `${row.manifest_category}\0${row.manifest_record_id}`));
for (const [category, value] of Object.entries(manifest.categories))
  for (const record of value.records)
    assert(coverageKeys.has(`${category}\0${record.id}`),
      `missing coverage ${category}/${record.id}`);
assert(coverage.filter(({ state }) => state === "DataReady").length === 18524
  && coverage.filter(({ state }) => state === "Excluded").length === 726,
"DataReady/exclusion coverage drift");

const mechanicManifest = manifest.categories.mechanic_rules.records;
const exactMechanics = mechanicManifest.filter(({ ownership }) =>
  ownership !== "EvidenceOnly");
const mechanicSources = rowsByFile.get("mechanic-source-files.json");
const mechanicRules = rowsByFile.get("mechanic-rules.json");
assert(exactMechanics.length === 2367
  && mechanicSources.length === exactMechanics.length
  && mechanicRules.length === exactMechanics.length,
"mechanic source/rule closure drift");
assert(mechanicRules.every(({ runtime_lowered: lowered,
  ordered_operations: operations }) =>
  lowered === false
    && operations.length === 1
    && operations[0].interpretation === "DeferredToLaterRuntimeGoal"),
"runtime exclusion drift");

const families = rowsByFile.get("semantic-fixture-families.json");
const fixtures = rowsByFile.get("review-fixtures.json");
assert(families.length === 28 && fixtures.length === 28,
  "semantic fixture count drift");
const familyById = new Map(families.map((row) => [row.id, row]));
for (const required of fixtureContract.required_families) {
  const id = `currency-wars.fixture-family.${required.id}`;
  const family = familyById.get(id);
  assert(family
    && JSON.stringify(family.must_cover) === JSON.stringify(required.must_cover),
  `fixture family drift ${required.id}`);
  assert(fixtures.some(({ family_id: familyId }) => familyId === id),
    `fixture case missing ${required.id}`);
}
const gaps = rowsByFile.get("research-gaps.json");
assert(gaps.length === 12
  && gaps.every((row) =>
    row.coverage_state === "Researched"
      && row.evidence_quality === "ProjectPolicy"
      && row.replacement_condition),
"research-gap policy drift");

const indexRows = rowsByFile.get("pack-index.json");
assert(indexRows.length > 1
  && indexRows.every(({ pack_digest: digest }) =>
    digest === indexRows[0].pack_digest),
"pack-index chunk drift");
const index = indexRows[0];
assert(indexRows.slice(1).every(({ file_digests: digests }) =>
  digests.length === 0), "pack file digests must appear only in chunk zero");
const digestEntries = [];
for (const entry of index.file_digests) {
  const bytes = fs.readFileSync(path.join(outputRoot, entry.file));
  assert(String(bytes.length) === entry.bytes
    && sha256(bytes) === entry.sha256,
  `${entry.file} pack digest drift`);
  digestEntries.push(`${entry.file}\0${entry.sha256}`);
}
assert(sha256(digestEntries.join("\n")) === index.pack_digest,
  "pack digest drift");
const allStableIds = indexRows.flatMap(({ stable_id_index: ids }) => ids);
const indexedIds = new Set(allStableIds.map(({ id }) => id));
assert(indexedIds.size === allStableIds.length,
  "stable-ID index duplicate");
for (const row of coverage)
  for (const id of row.normalized_record_ids)
    assert(indexedIds.has(id), `${row.id} unresolved normalized ID ${id}`);
const packManifest = rowsByFile.get("manifest.json")[0];
assert(packManifest.record_counts["pack-index.json"]
  === String(indexRows.length), "pack-index manifest count drift");
assert(packManifest.content_manifest_sha256 === sha256(fs.readFileSync(path.join(
  root,
  "content-manifests/currency-wars-v1/content-manifest.json",
))), "content manifest binding drift");

console.log(
  `Currency Wars pack verified (${coverage.length} obligations; ` +
  `18524/18524 eligible DataReady; 726 explicit exclusions; ` +
  `${mechanicRules.length} unlowered mechanic dossiers; ` +
  `${fixtures.length} semantic fixture families; ${index.pack_digest}).`,
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
function unique(values) {
  return new Set(values).size === values.length;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
