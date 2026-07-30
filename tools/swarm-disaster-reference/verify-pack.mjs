#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { canonical, sha256 } from "./lib/common.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const packRoot = path.join(root, "content-reference/swarm-disaster-v1");
execFileSync(
  process.execPath,
  ["tools/swarm-disaster-reference/finalize-pack.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);

const schema = json(
  "content-manifests/swarm-disaster-v1/normalized-schema.json",
);
const sourceManifest = json(
  "content-manifests/swarm-disaster-v1/content-manifest.json",
);
const fixtureContract = json(
  "content-manifests/swarm-disaster-v1/fixture-contract.json",
);
const values = new Map(schema.files.map(({ file }) => [
  file,
  json(`content-reference/swarm-disaster-v1/${file}`),
]));
const rules = values.get("mechanic-rules.json");
const sources = values.get("sources.json");
const coverage = values.get("coverage.json");
const gaps = values.get("research-gaps.json");
const fixtures = values.get("review-fixtures.json");
const receipts = values.get("reconciliation-receipts.json");
const manifest = values.get("manifest.json");
const packIndex = values.get("pack-index.json");

assert(rules.length === 23, "mechanic-rule count drift");
assert(sources.length === 8139, "source registry count drift");
assert(coverage.length === 6963, "coverage row count drift");
assert(gaps.length === 31, "research-gap count drift");
assert(fixtures.length === 23, "semantic fixture count drift");
assert(receipts.length === 609, "Goal 08 receipt count drift");

for (const contract of schema.files) {
  const value = values.get(contract.file);
  assert(value !== undefined, `${contract.file} is missing`);
  const allRows = Array.isArray(value) ? value : [value];
  const matchingRows = allRows.filter(({ kind }) =>
    kind === contract.record_kind);
  const contractRows = matchingRows.length > 0 ? matchingRows : allRows;
  for (const row of contractRows)
    for (const field of contract.required_domain_fields)
      assert(Object.hasOwn(row, field),
        `${contract.file}/${row.id ?? "root"} lacks ${field}`);
}

const fixtureByFamily = new Map(
  fixtures.map((row) => [row.family_id, row]),
);
const ruleByFamily = new Map(rules.map((row) => [row.family_id, row]));
assert(fixtureByFamily.size === fixtures.length, "duplicate fixture family");
assert(ruleByFamily.size === rules.length, "duplicate rule family");
for (const family of fixtureContract.required_families) {
  const fixture = fixtureByFamily.get(family.id);
  const rule = ruleByFamily.get(family.id);
  assert(fixture && rule, `${family.id} rule/fixture closure drift`);
  assert(fixture.source_record_ids.length > 0
    && fixture.preconditions.length > 0
    && fixture.ordered_operations.length === family.must_cover.length
    && fixture.expected_facts.length > family.must_cover.length
    && fixture.evidence_refs.length > 0,
  `${family.id} fixture contract drift`);
  assert(JSON.stringify(fixture.ordered_operations.map(({ sequence }) =>
    sequence))
    === JSON.stringify(
      Array.from({ length: family.must_cover.length }, (_, index) => index + 1),
    ),
  `${family.id} fixture operation order drift`);
  assert(JSON.stringify(fixture.ordered_operations.map(({ fact }) => fact))
    === JSON.stringify(family.must_cover),
  `${family.id} must-cover facts drift`);
  assert(rule.execution_disposition === "ReferenceOnly"
    && rule.runtime_handler_id === ""
    && rule.triggers.length > 0
    && rule.state_slots.length > 0
    && rule.program.length === family.must_cover.length
    && JSON.stringify(rule.fixture_ids) === JSON.stringify([fixture.id]),
  `${family.id} mechanic-rule contract drift`);
}

const rowIdsByFile = new Map();
for (const [file, value] of values) {
  if (!Array.isArray(value)) continue;
  assert(unique(value.map(({ id }) => id)), `${file} duplicate row ID`);
  rowIdsByFile.set(file, new Set(value.map(({ id }) => id)));
}
const coverageByObligation = new Map(coverage.map((row) => [
  `${row.manifest_category}\0${row.manifest_record_id}`,
  row,
]));
assert(coverageByObligation.size === coverage.length,
  "duplicate coverage obligation");
for (const [categoryId, category] of Object.entries(sourceManifest.categories))
  for (const record of category.records) {
    const row = coverageByObligation.get(`${categoryId}\0${record.id}`);
    assert(row
      && row.coverage_state === "DataReady"
      && row.source_locator === record.source
      && row.source_evidence_sha256 === record.evidence_sha256
      && row.normalized_refs.length > 0
      && row.blocking_gap_ids.length === 0,
    `${categoryId}/${record.id} coverage drift`);
    for (const ref of row.normalized_refs)
      assert(rowIdsByFile.get(ref.file)?.has(ref.id),
        `${row.id} unresolved normalized ref ${ref.file}/${ref.id}`);
  }

const sourceById = new Map(sources.map((row) => [row.id, row]));
assert(sourceById.size === sources.length, "duplicate source ID");
for (const [file, value] of values) {
  if (!Array.isArray(value) || file === "sources.json") continue;
  for (const row of value)
    for (const ref of row.source_refs ?? []) {
      const source = sourceById.get(ref.source_id);
      assert(source
        && source.repository === ref.repository
        && source.revision === ref.revision
        && source.path === ref.path
        && source.locator === ref.locator
        && source.sha256 === ref.sha256
        && source.evidence_quality === ref.evidence_quality,
      `${file}/${row.id} unresolved source ${ref.source_id}`);
    }
}

const policySourceIds = new Set(sources
  .filter(({ evidence_quality: quality }) =>
    ["ProjectPolicy", "ApproximateFromReleasedText"].includes(quality))
  .map(({ id }) => id));
assert(JSON.stringify([...policySourceIds].sort())
  === JSON.stringify(gaps.map(({ policy_source_id: id }) => id).sort()),
"policy source and research-gap sets differ");
for (const gap of gaps) {
  assert(gap.state === "PolicyBound"
    && gap.gap_state === "PolicyBound"
    && gap.blocking === false
    && gap.known_facts
    && gap.selected_policy
    && gap.replacement_condition
    && sourceById.has(gap.policy_source_id),
  `${gap.id} nonblocking research-gap contract drift`);
  for (const ref of gap.affected_records)
    assert(rowIdsByFile.get(ref.file)?.has(ref.id),
      `${gap.id} unresolved affected record ${ref.file}/${ref.id}`);
}

const goal08Object =
  "457d05f0e3a7b6fe3abb7e8f142f96fa271f5ecd:" +
  "content-manifests/gold-and-gears-v1/content-manifest.json";
const goal08Result = spawnSync("git", ["cat-file", "blob", goal08Object], {
  cwd: root,
  encoding: "utf8",
  maxBuffer: 16 * 1024 * 1024,
});
assert(goal08Result.status === 0, "Goal 08 checkpoint manifest unavailable");
const goal08 = JSON.parse(goal08Result.stdout);
const goldByIdentity = new Map();
for (const [categoryId, category] of Object.entries(goal08.categories))
  for (const record of category.records)
    goldByIdentity.set(`${record.source}\0${record.id}`, { categoryId, record });
const expectedOverlap = [];
for (const [categoryId, category] of Object.entries(sourceManifest.categories))
  for (const record of category.records) {
    const gold = goldByIdentity.get(`${record.source}\0${record.id}`);
    if (gold) expectedOverlap.push({ categoryId, record, gold });
  }
assert(expectedOverlap.length === receipts.length,
  "Goal 08 overlap receipt count drift");
for (const receipt of receipts) {
  const gold = goldByIdentity.get(
    `${receipt.source_path}${receipt.row_locator === receipt.swarm_record_id
      ? ""
      : `#${receipt.row_locator}`}\0${receipt.swarm_record_id}`,
  ) ?? [...goldByIdentity.values()].find(({ record }) =>
    record.id === receipt.swarm_record_id
    && record.evidence_sha256 === receipt.evidence_sha256);
  assert(gold
    && receipt.outcome === "MatchedSharedFact"
    && receipt.goal08_commit
      === "457d05f0e3a7b6fe3abb7e8f142f96fa271f5ecd"
    && receipt.evidence_sha256 === gold.record.evidence_sha256
    && receipt.goal08_category === gold.categoryId,
  `${receipt.id} Goal 08 reconciliation drift`);
}

assert(manifest.source_manifest_sha256 === sha256(fs.readFileSync(path.join(
  root,
  "content-manifests/swarm-disaster-v1/source-inventory.json",
)))
  && manifest.content_manifest_sha256 === sha256(fs.readFileSync(path.join(
    root,
    "content-manifests/swarm-disaster-v1/content-manifest.json",
  )))
  && manifest.frozen_source_obligations === 6963
  && manifest.data_ready_source_obligations === 6963
  && manifest.coverage_percent === "100"
  && manifest.mechanic_rule_count === 23
  && manifest.semantic_fixture_family_count === 23
  && manifest.research_gap_count === 31
  && manifest.blocking_research_gap_count === 0
  && manifest.reconciliation_receipt_count === 609
  && manifest.runtime_loading === "ForbiddenReferenceOnly"
  && manifest.candidate_quality === true,
"pack manifest drift");

const expectedIndexFiles = schema.files.map(({ file }) => file)
  .filter((file) => file !== "pack-index.json").sort();
assert(JSON.stringify(packIndex.map(({ file }) => file))
  === JSON.stringify(expectedIndexFiles),
"pack-index file set drift");
for (const entry of packIndex) {
  const bytes = fs.readFileSync(path.join(packRoot, entry.file));
  const value = values.get(entry.file);
  assert(entry.bytes === bytes.length
    && entry.rows === (Array.isArray(value) ? value.length : 1)
    && entry.sha256 === sha256(bytes),
  `${entry.file} pack-index drift`);
}
const expectedPackDigest = sha256(
  packIndex.map(({ file, sha256: digest }) =>
    `${file}\0${digest}`).join("\n"),
);
assert(packIndex.every(({ pack_sha256: digest }) =>
  digest === expectedPackDigest), "pack digest drift");

console.log(
  "Swarm Disaster pack verified (6,963/6,963 DataReady; 23 rules and " +
  "fixtures; 31 nonblocking gaps; 609 exact Goal 08 receipts; 64 files).",
);

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function unique(values) {
  return new Set(values).size === values.length;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
