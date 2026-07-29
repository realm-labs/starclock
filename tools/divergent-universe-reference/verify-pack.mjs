#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sourceCache = valueAfter("--source-cache");
const finalizeArgs = [
  "tools/divergent-universe-reference/finalize-pack.mjs",
  "--check",
  "--root",
  root,
];
if (sourceCache) finalizeArgs.push("--source-cache", path.resolve(sourceCache));
execFileSync(process.execPath, finalizeArgs, { cwd: root, stdio: "inherit" });

const referenceRoot = path.join(
  root,
  "content-reference/divergent-universe-v1",
);
const schema = rootJson(
  "content-manifests/divergent-universe-v1/normalized-schema.json",
);
const manifest = rootJson(
  "content-manifests/divergent-universe-v1/content-manifest.json",
);
const fixtureContract = rootJson(
  "content-manifests/divergent-universe-v1/fixture-contract.json",
);
const expectedFiles = schema.files.map(({ file }) => file).sort();
const actualFiles = fs.readdirSync(referenceRoot)
  .filter((file) => file.endsWith(".json")).sort();
assert(expectedFiles.length === 80, "normalized file denominator drift");
assert(equal(expectedFiles, actualFiles), "normalized output file set drift");

const allRows = new Map();
for (const file of expectedFiles) {
  const rows = reference(file);
  assert(Array.isArray(rows), `${file} must contain a row array`);
  for (const row of rows) {
    assert(row.id && !allRows.has(row.id),
      `${file} duplicate global stable ID ${row.id}`);
    assert(
      row.schema_revision === "starclock.divergent-universe-row.v1"
        && row.kind
        && row.name_en
        && row.name_zh_cn
        && row.summary_en
        && row.summary_zh_cn
        && ["DivergentUniverse", "Shared", "OtherMode", "Excluded"].includes(
          row.ownership,
        )
        && ["Cataloged", "Researched", "DataReady", "Blocked", "Excluded"].includes(
          row.coverage_state,
        )
        && row.evidence_quality
        && Array.isArray(row.source_refs)
        && row.source_refs.length >= 1
        && Array.isArray(row.tags),
      `${file} common envelope drift for ${row.id}`,
    );
    allRows.set(row.id, { file, row });
  }
}
assert(allRows.size === 27091, "global normalized row denominator drift");

const sources = reference("sources.json");
assert(sources.length === 7624, "source registry denominator drift");
const sourceIds = new Set(sources.map(({ source_id: id }) => id));
assert(sourceIds.size === sources.length, "duplicate source registry ID");
for (const file of expectedFiles)
  for (const row of reference(file)) {
    for (const source of row.source_refs)
      assert(
        sourceIds.has(source.source_id)
          && source.game_version === "4.4"
          && /^[0-9a-f]{64}$/u.test(source.sha256),
        `${file}/${row.id} unresolved or invalid source ${source.source_id}`,
      );
    for (const sourceId of row.evidence_refs ?? [])
      assert(sourceIds.has(sourceId),
        `${file}/${row.id} unresolved evidence ref ${sourceId}`);
  }

const mechanicSources = reference("mechanic-source-files.json");
const mechanicRules = reference("mechanic-rules.json");
assert(mechanicSources.length === 669 && mechanicRules.length === 669,
  "mechanic source/rule denominator drift");
const ruleById = new Map(mechanicRules.map((row) => [row.id, row]));
for (const source of mechanicSources) {
  const rule = ruleById.get(source.consumer_rule_ids[0]);
  assert(
    source.operation_types.length >= 1
      && source.operation_occurrence_count >= source.operation_types.length
      && source.disposition === "ReferenceOnlyNotLowered"
      && source.consumer_rule_ids.length === 1
      && source.runtime_lowered === false
      && rule
      && rule.source_file_id === source.id
      && rule.source_id === source.source_id
      && rule.ordered_operations.length === source.operation_types.length
      && rule.fixture_ids.length === 1
      && allRows.has(rule.fixture_ids[0])
      && rule.runtime_lowered === false,
    `mechanic source/rule closure drift for ${source.source_id}`,
  );
}

const coverage = reference("coverage.json");
const obligations = Object.entries(manifest.categories).flatMap(
  ([category, value]) => value.records.map((record) => ({
    category,
    id: String(record.id),
    locator: record.source,
    evidenceSha256: record.evidence_sha256,
  })),
);
assert(obligations.length === 6215 && coverage.length === 6215,
  "coverage denominator drift");
assert(equal(
  coverage.map((row) =>
    `${row.manifest_category}\0${row.manifest_record_id}`).sort(),
  obligations.map((row) => `${row.category}\0${row.id}`).sort(),
), "manifest exact-once coverage drift");
const obligationByKey = new Map(obligations.map((row) =>
  [`${row.category}\0${row.id}`, row]));
for (const row of coverage) {
  const obligation = obligationByKey.get(
    `${row.manifest_category}\0${row.manifest_record_id}`,
  );
  assert(
    obligation
      && row.state === "DataReady"
      && row.source_locator === obligation.locator
      && row.source_evidence_sha256 === obligation.evidenceSha256
      && row.normalized_record_ids.length >= 1
      && row.normalized_record_ids.every((id) => allRows.has(id))
      && row.blocking_gap_ids.length === 0,
    `coverage closure drift for ${row.source_id}`,
  );
}

const families = reference("semantic-fixture-families.json");
const fixtures = reference("review-fixtures.json");
const gaps = reference("research-gaps.json");
assert(families.length === 25 && fixtures.length === 25 && gaps.length === 25,
  "fixture/gap denominator drift");
const requiredFamilies = new Map(
  fixtureContract.required_families.map((row) => [row.id, row]),
);
const fixtureByFamily = new Map(fixtures.map((row) =>
  [row.family_id, row]));
const gapByFamily = new Map(gaps.map((row) => [row.source_id, row]));
for (const family of families) {
  const requiredFamily = requiredFamilies.get(family.source_id);
  const fixture = fixtureByFamily.get(family.source_id);
  const gap = gapByFamily.get(family.source_id);
  assert(
    requiredFamily
      && family.minimum_cases === requiredFamily.minimum_cases
      && equal(family.must_cover, requiredFamily.must_cover)
      && family.selected_source_record_ids.length >= 1
      && family.selected_source_record_ids.every((id) => allRows.has(id))
      && family.runtime_executable === false
      && fixture
      && fixture.source_record_ids.length >= 1
      && fixture.source_record_ids.every((id) => allRows.has(id))
      && fixture.expected_facts.length === family.must_cover.length
      && fixture.evidence_quality === "ProjectPolicy"
      && fixture.runtime_executable === false
      && gap
      && gap.state === "PolicyBound"
      && gap.blocking === false
      && gap.owner === "G11-P4-B2"
      && gap.affected_data_ids.every((id) => allRows.has(id))
      && gap.replacement_condition,
    `fixture or research-gap closure drift for ${family.source_id}`,
  );
}

const receipts = reference("reconciliation-receipts.json");
assert(
  receipts.length === 102 &&
    receipts.every(
      (receipt) =>
        receipt.ownership === "Shared" &&
        receipt.coverage_state === "DataReady" &&
        receipt.outcome === "MatchedShared" &&
        receipt.checkpoint_ownership === "SharedSourceEvidence" &&
        receipt.goal11_ownership === "SharedSourceEvidence" &&
        receipt.blocking === false,
    ) &&
    JSON.stringify(
      Object.fromEntries(
        Object.entries(
          Object.groupBy(receipts, ({ checkpoint_goal: goal }) => goal),
        ).map(([goal, rows]) => [goal, rows.length]),
      ),
    ) === '{"Goal08":53,"Goal09":45,"Goal10":4}',
  "reconciliation receipt closure drift",
);
const summaryRows = reference("manifest.json");
assert(summaryRows.length === 1, "reference manifest cardinality drift");
const summary = summaryRows[0];
assert(
  summary.frozen_source_obligations === 6215
    && summary.data_ready_source_obligations === 6215
    && summary.coverage_percent === "100"
    && summary.normalized_files.length === 80
    && Object.keys(summary.record_counts).length === 80
    && summary.mechanic_source_count === 669
    && summary.mechanic_rule_count === 669
    && summary.source_evidence_count === 7624
    && summary.semantic_fixture_family_count === 25
    && summary.reconciliation_receipt_count === 102
    && summary.nonblocking_research_gap_count === 25
    && summary.blocking_research_gap_count === 0
    && summary.runtime_loading === "ForbiddenReferenceOnly"
    && summary.candidate_quality === true,
  "reference manifest summary drift",
);

const indexRows = reference("pack-index.json");
assert(indexRows.length === 1, "pack index cardinality drift");
const index = indexRows[0];
assert(
  index.file_digests.length === 79
    && index.stable_id_index.length === 27090
    && index.runtime_loading === "ForbiddenReferenceOnly",
  "pack index denominator/boundary drift",
);
for (const entry of index.file_digests) {
  const bytes = fs.readFileSync(path.join(referenceRoot, entry.file));
  assert(entry.bytes === bytes.length && entry.sha256 === sha256(bytes),
    `pack file digest drift for ${entry.file}`);
}
assert(index.pack_digest === sha256(index.file_digests.map(
  ({ file, sha256: digest }) => `${file}\0${digest}`,
).join("\n")), "pack digest drift");
for (const entry of index.stable_id_index) {
  const indexed = allRows.get(entry.id);
  assert(indexed && indexed.file === entry.file,
    `stable ID index drift for ${entry.id}`);
}

const evidence = fs.readFileSync(path.join(
  root,
  "evidence/divergent-universe-reference-v1/phase2-pack-boundary.md",
), "utf8");
for (const phrase of [
  "6,215/6,215",
  "669 mechanic source files",
  "7,620 source-evidence rows",
  "25 semantic fixture families",
  "25 nonblocking research gaps",
  "80 normalized files",
  "`ForbiddenReferenceOnly`",
])
  assert(evidence.includes(phrase), `Phase 2 evidence omits ${phrase}`);

console.log(
  "Divergent Universe Phase 2 pack verified (6,215/6,215 DataReady " +
  "dispositions; 669 mechanic sources/rules; 7,624 sources; 102 " +
  "reconciliation receipts; 25 fixtures/gaps; 80 files; runtime forbidden).",
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}

function rootJson(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}

function reference(file) {
  return JSON.parse(fs.readFileSync(path.join(referenceRoot, file), "utf8"));
}

function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function equal(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
