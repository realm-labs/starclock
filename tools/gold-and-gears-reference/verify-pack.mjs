#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const dataRoot = path.join(root, "content-reference/gold-and-gears-v1");
execFileSync(
  process.execPath,
  ["tools/gold-and-gears-reference/finalize-pack.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);

const schema = json(
  "content-manifests/gold-and-gears-v1/normalized-schema.json",
);
const sourceManifest = json(
  "content-manifests/gold-and-gears-v1/content-manifest.json",
);
const fixtureContract = json(
  "content-manifests/gold-and-gears-v1/fixture-contract.json",
);
const rules = json("content-reference/gold-and-gears-v1/mechanic-rules.json");
const sources = json("content-reference/gold-and-gears-v1/sources.json");
const coverage = json("content-reference/gold-and-gears-v1/coverage.json");
const gaps = json("content-reference/gold-and-gears-v1/research-gaps.json");
const fixtures = json("content-reference/gold-and-gears-v1/review-fixtures.json");
const manifest = json("content-reference/gold-and-gears-v1/manifest.json");
const packIndex = json("content-reference/gold-and-gears-v1/pack-index.json");

assert(rules.length === 1224, "mechanic-rule count drift");
assert(sources.length === 9082, "source-registry count drift");
assert(coverage.length === 42, "coverage-category count drift");
assert(fixtures.length === 18, "semantic-fixture count drift");
assert(gaps.length === 16 && gaps.every(({ blocking }) => !blocking),
  "research-gap blocking state drift");
assert(unique(rules.map(({ id }) => id)), "duplicate mechanic-rule ID");
assert(unique(sources.map(({ source_id: id }) => id)), "duplicate source ID");
assert(unique(fixtures.map(({ id }) => id)), "duplicate fixture ID");

const sourceIds = new Set(sources.map(({ source_id: id }) => id));
const fixtureIds = new Set(fixtures.map(({ id }) => id));
for (const contract of fixtureContract.required_families) {
  const matches = fixtures.filter(({ family_id: id }) => id === contract.id);
  assert(matches.length >= contract.minimum_cases,
    `${contract.id} fixture-family coverage drift`);
}
for (const fixture of fixtures) {
  for (const field of fixtureContract.required_fields)
    assert(fixture[field] !== undefined, `${fixture.id} missing ${field}`);
  assert(fixture.source_record_ids.length > 0
    && fixture.ordered_operations.length > 0
    && fixture.expected_facts.length > 0
    && fixture.evidence_refs.length > 0,
  `${fixture.id} fixture payload drift`);
  for (const sourceId of fixture.evidence_refs)
    assert(sourceIds.has(sourceId), `${fixture.id} missing source ${sourceId}`);
  if (["ProjectPolicy", "ApproximateFromReleasedText"].includes(
    fixture.fixture_evidence_quality,
  ))
    assert(fixture.note && fixture.replacement_condition,
      `${fixture.id} missing approximation boundary`);
}
for (const rule of rules) {
  assert(rule.execution_disposition === "ReferenceOnly"
    && rule.runtime_handler_id === ""
    && rule.fixture_ids.length === 1
    && fixtureIds.has(rule.fixture_ids[0]),
  `${rule.id} runtime/fixture boundary drift`);
  for (const ref of rule.source_refs)
    assert(sourceIds.has(ref.source_id),
      `${rule.id} has unresolved source ${ref.source_id}`);
}
for (const source of sources) {
  assert(source.source_id === source.id
    && /^[0-9a-f]{64}$/u.test(source.evidence_sha256)
    && source.repository_or_url
    && source.revision_or_access_date
    && source.relative_path_or_page
    && source.row_locator
    && source.access_date,
  `${source.source_id} source registry drift`);
  if (source.evidence_quality === "ProjectPolicy")
    assert(source.note && source.replacement_condition,
      `${source.source_id} policy boundary drift`);
}
for (const contract of schema.files) {
  if (["sources.json", "manifest.json", "pack-index.json"].includes(
    contract.file,
  )) continue;
  const value = json(`content-reference/gold-and-gears-v1/${contract.file}`);
  if (!Array.isArray(value)) continue;
  for (const row of value) {
    for (const field of [
      "id", "kind", "name_en", "name_zh_cn", "summary_en", "summary_zh_cn",
      "ownership", "coverage_state", "evidence_quality", "source_refs", "tags",
    ])
      assert(row[field] !== undefined, `${contract.file}/${row.id} missing ${field}`);
    for (const ref of row.source_refs)
      assert(sourceIds.has(ref.source_id),
        `${contract.file}/${row.id} has unresolved source ${ref.source_id}`);
  }
}
for (const gap of gaps)
  assert(gap.gap_state === "PolicyBound"
    && gap.affected_records.length > 0
    && sourceIds.has(gap.policy_source_id)
    && gap.note
    && gap.replacement_condition,
  `${gap.id} research-gap registry drift`);

const conundrum = json(
  "content-reference/gold-and-gears-v1/conundrum-levels.json",
);
const encounterIds = new Set(json(
  "content-reference/gold-and-gears-v1/encounter-groups.json",
).map(({ id }) => id));
const curioPools = new Set(json(
  "content-reference/gold-and-gears-v1/curios.json",
).map(({ selection_pool_id: id }) => id));
const auxiliaryTwo = conundrum.find(({ source_id: id }) => id === "202")
  .effect_contributions[0];
const auxiliaryFive = conundrum.find(({ source_id: id }) => id === "205")
  .effect_contributions[0];
assert(auxiliaryTwo.encounter_group_ids.length === 12
  && auxiliaryTwo.encounter_group_ids.every((id) => encounterIds.has(id)),
"Auxiliary +2 encounter closure drift");
assert(auxiliaryFive.pool_binding_state === "DataReady"
  && curioPools.has(auxiliaryFive.selection_pool_id)
  && auxiliaryFive.unresolved_pool_behavior === "FailClosed",
"Auxiliary +5 Curio closure drift");

const requiredTotal = coverage.reduce((sum, row) => sum + row.required, 0);
const readyTotal = coverage.reduce((sum, row) => sum + row.data_ready, 0);
assert(requiredTotal === 7913 && readyTotal === 7913,
  "source-obligation coverage drift");
for (const row of coverage)
  assert(row.required === row.accounted
    && row.accounted === row.data_ready
    && row.coverage_percent === "100"
    && row.blocking_gap_ids.length === 0,
  `${row.category_id} coverage drift`);
const coveredCategories = coverage.map(({ category_id: id }) => id).sort();
assert(JSON.stringify(coveredCategories)
  === JSON.stringify(Object.keys(sourceManifest.categories).sort()),
"coverage category identity drift");

assert(manifest.frozen_source_obligations === 7913
  && manifest.data_ready_source_obligations === 7913
  && manifest.coverage_percent === "100"
  && manifest.normalized_file_count === 51
  && manifest.mechanic_rule_count === rules.length
  && manifest.semantic_fixture_family_count === fixtures.length
  && manifest.research_gap_count === gaps.length
  && manifest.blocking_research_gap_count === 0
  && manifest.runtime_loading === "ForbiddenReferenceOnly",
"pack manifest drift");

const expectedFiles = schema.files.map(({ file }) => file).sort();
const actualFiles = fs.readdirSync(dataRoot)
  .filter((file) => file.endsWith(".json")).sort();
assert(JSON.stringify(actualFiles) === JSON.stringify(expectedFiles),
  "normalized file set drift");
assert(packIndex.files.length === 50, "pack-index file count drift");
for (const entry of packIndex.files) {
  const bytes = fs.readFileSync(path.join(dataRoot, entry.file));
  assert(bytes.length === entry.bytes
    && sha256(bytes) === entry.sha256,
  `${entry.file} pack-index drift`);
}
assert(packIndex.pack_sha256 === sha256(packIndex.files
  .map(({ file, sha256: digest }) => `${file}\0${digest}`).join("\n")),
"pack aggregate digest drift");

console.log(
  `Gold and Gears pack verified (${rules.length} rules; ${sources.length} ` +
  `sources; 7,913/7,913 DataReady; ${gaps.length} nonblocking gaps; ` +
  "18 fixture families; 51 normalized files).",
);

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function unique(values) {
  return new Set(values).size === values.length;
}
function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
