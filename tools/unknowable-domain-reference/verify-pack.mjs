#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const referenceRoot = path.join(
  root,
  "content-reference/unknowable-domain-v1",
);
execFileSync(
  process.execPath,
  ["tools/unknowable-domain-reference/finalize-pack.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);

const schema = json(
  "content-manifests/unknowable-domain-v1/normalized-schema.json",
);
const sourceManifest = json(
  "content-manifests/unknowable-domain-v1/content-manifest.json",
);
const expectedFiles = schema.files.map(({ file }) => file).sort();
const actualFiles = fs.readdirSync(referenceRoot)
  .filter((file) => file.endsWith(".json"))
  .sort();
assert(expectedFiles.length === 65, "normalized schema file count drift");
assert(equal(expectedFiles, actualFiles), "normalized output file set drift");

const allRows = new Map();
for (const file of expectedFiles) {
  const fileRows = reference(file);
  assert(Array.isArray(fileRows), `${file} must contain an array`);
  for (const row of fileRows) {
    assert(row.id && !allRows.has(row.id), `${file} duplicate global row ID`);
    assert(
      row.schema_revision === "starclock.unknowable-domain-row.v1"
        && row.kind
        && row.name_en
        && row.name_zh_cn
        && row.summary_en
        && row.summary_zh_cn
        && ["UnknowableDomain", "Shared"].includes(row.ownership)
        && row.coverage_state === "DataReady"
        && row.evidence_quality
        && Array.isArray(row.source_refs)
        && Array.isArray(row.tags),
      `${file} common envelope drift for ${row.id}`,
    );
    allRows.set(row.id, row);
  }
}

const sources = reference("sources.json");
assert(sources.length === 4473, "source registry denominator drift");
const sourceIds = new Set(sources.map(({ source_id: id }) => id));
assert(sourceIds.size === sources.length, "duplicate source registry ID");
for (const [file, rows] of expectedFiles.map((file) =>
  [file, reference(file)])) {
  for (const row of rows) {
    for (const ref of row.source_refs) {
      assert(
        sourceIds.has(ref.source_id),
        `${file}/${row.id} has unresolved source ${ref.source_id}`,
      );
      assert(
        ref.game_version === "4.4" && /^[0-9a-f]{64}$/u.test(ref.sha256),
        `${file}/${row.id} source evidence drift`,
      );
    }
    for (const sourceId of row.evidence_refs ?? [])
      assert(
        sourceIds.has(sourceId),
        `${file}/${row.id} has unresolved evidence source ${sourceId}`,
      );
  }
}

const mechanicSources = reference("mechanic-source-files.json");
const mechanicRules = reference("mechanic-rules.json");
assert(
  mechanicSources.length === 41 && mechanicRules.length === 41,
  "mechanic source/rule denominator drift",
);
const ruleById = new Map(mechanicRules.map((row) => [row.id, row]));
for (const source of mechanicSources) {
  assert(
    source.kind === "MechanicSourceFile"
      && source.operation_types.length >= 1
      && source.operation_occurrence_count >= source.operation_types.length
      && source.consumer_rule_ids.length === 1
      && source.runtime_lowered === false,
    `mechanic source boundary drift for ${source.source_id}`,
  );
  const rule = ruleById.get(source.consumer_rule_ids[0]);
  assert(
    rule
      && rule.kind === "UnknowableMechanicRule"
      && rule.source_file_id === source.id
      && rule.source_id === source.source_id
      && equal(
        rule.ordered_operations.map(({ operation_type: type }) => type),
        source.operation_types.map(({ operation_type: type }) => type),
      )
      && rule.fixture_ids.length === 1
      && rule.runtime_lowered === false,
    `mechanic rule closure drift for ${source.source_id}`,
  );
  assert(
    allRows.has(rule.fixture_ids[0]),
    `mechanic rule fixture missing for ${source.source_id}`,
  );
}

const coverage = reference("coverage.json");
const obligations = Object.entries(sourceManifest.categories).flatMap(
  ([category, value]) => value.records.map((record) => ({
    category,
    id: String(record.id),
    locator: record.source,
    evidenceSha256: record.evidence_sha256,
  })),
);
assert(
  obligations.length === 5377 && coverage.length === obligations.length,
  "coverage denominator drift",
);
const expectedCoverage = obligations.map(({ category, id }) =>
  `${category}\0${id}`).sort();
const actualCoverage = coverage.map((row) =>
  `${row.manifest_category}\0${row.manifest_record_id}`).sort();
assert(equal(expectedCoverage, actualCoverage), "manifest exact-once drift");
const obligationByKey = new Map(obligations.map((row) =>
  [`${row.category}\0${row.id}`, row]));
for (const row of coverage) {
  const obligation = obligationByKey.get(
    `${row.manifest_category}\0${row.manifest_record_id}`,
  );
  assert(
    obligation
      && row.kind === "ReferenceCoverage"
      && row.state === "DataReady"
      && row.source_locator === obligation.locator
      && row.source_evidence_sha256 === obligation.evidenceSha256
      && row.data_ids.length >= 1
      && row.data_ids.every((id) => allRows.has(id))
      && row.blocking_gap_ids.length === 0,
    `coverage closure drift for ${row.source_id}`,
  );
}

const contract = json(
  "content-manifests/unknowable-domain-v1/fixture-contract.json",
);
const families = reference("semantic-fixture-families.json");
const fixtures = reference("review-fixtures.json");
const gaps = reference("research-gaps.json");
assert(
  families.length === 24 && fixtures.length === 24 && gaps.length === 24,
  "semantic family/fixture/gap denominator drift",
);
const requiredFamilies = new Map(contract.required_families.map((row) =>
  [row.id, row]));
const fixtureByFamily = new Map(fixtures.map((row) => [row.family_id, row]));
const gapByFamily = new Map(gaps.map((row) => [row.source_id, row]));
assert(requiredFamilies.size === 24, "fixture contract denominator drift");
for (const family of families) {
  const requiredFamily = requiredFamilies.get(family.source_id);
  const fixture = fixtureByFamily.get(family.source_id);
  const gap = gapByFamily.get(family.source_id);
  assert(
    requiredFamily
      && equal(family.must_cover, requiredFamily.must_cover)
      && family.minimum_cases === requiredFamily.minimum_cases
      && family.selected_source_record_ids.length >= 1
      && family.selected_source_record_ids.every((id) => allRows.has(id))
      && family.runtime_executable === false,
    `semantic family drift for ${family.source_id}`,
  );
  assert(
    fixture
      && fixture.source_record_ids.length >= 1
      && fixture.source_record_ids.every((id) => allRows.has(id))
      && fixture.expected_facts.length === family.must_cover.length
      && fixture.fixture_evidence_quality === "ProjectPolicy"
      && fixture.runtime_executable === false,
    `review fixture drift for ${family.source_id}`,
  );
  assert(
    gap
      && gap.state === "PolicyBound"
      && gap.blocking === false
      && gap.owner === "G10-P4-B2"
      && gap.affected_data_ids.length >= 1
      && gap.affected_data_ids.every((id) => allRows.has(id))
      && gap.replacement_condition,
    `research gap drift for ${family.source_id}`,
  );
}

const receipts = reference("reconciliation-receipts.json");
assert(receipts.length === 155, "ownership receipt denominator drift");
assert(
  receipts.every((row) =>
    row.kind === "OwnershipReconciliationReceipt"
      && ["Goal08", "Goal09"].includes(row.checkpoint_goal)
      && row.source_path
      && row.row_locator
      && /^[0-9a-f]{64}$/u.test(row.evidence_sha256)
      && row.outcome !== "Conflict"
      && row.blocking === false),
  "ownership reconciliation conflict or locator drift",
);
const checkpoints = new Map([
  ["Goal08", "2f7b3ccf699c52c2738136b8636d140e053bb2eb"],
  ["Goal09", "9bd2ad285de4c10e7ab060f00bf078855923a09c"],
]);
assert(receipts.every((row) =>
  row.checkpoint_commit === checkpoints.get(row.checkpoint_goal)),
"ownership checkpoint drift");

const manifestRows = reference("manifest.json");
assert(manifestRows.length === 1, "reference manifest cardinality drift");
const manifest = manifestRows[0];
assert(
  manifest.frozen_source_obligations === 5377
    && manifest.data_ready_source_obligations === 5377
    && manifest.coverage_percent === "100"
    && manifest.normalized_file_count === 65
    && manifest.mechanic_source_count === 41
    && manifest.mechanic_rule_count === 41
    && manifest.semantic_fixture_family_count === 24
    && manifest.research_gap_count === 24
    && manifest.blocking_research_gap_count === 0
    && manifest.reconciliation_receipt_count === 155
    && manifest.runtime_loading === "ForbiddenReferenceOnly"
    && manifest.candidate_quality === true,
  "reference manifest summary drift",
);

const indexRows = reference("pack-index.json");
assert(indexRows.length === 1, "pack index cardinality drift");
const index = indexRows[0];
assert(
  index.file_digests.length === 64
    && index.runtime_loading === "ForbiddenReferenceOnly",
  "pack index boundary drift",
);
for (const entry of index.file_digests) {
  const bytes = fs.readFileSync(path.join(referenceRoot, entry.file));
  assert(
    entry.bytes === bytes.length && entry.sha256 === sha256(bytes),
    `pack file digest drift for ${entry.file}`,
  );
}
const expectedPackDigest = sha256(index.file_digests.map(
  ({ file, sha256: digest }) => `${file}\0${digest}`,
).join("\n"));
assert(index.pack_digest === expectedPackDigest, "pack digest drift");

const evidence = fs.readFileSync(path.join(
  root,
  "evidence/unknowable-domain-reference-v1/phase2-pack-boundary.md",
), "utf8");
for (const phrase of [
  "5,377 frozen obligations",
  "41 mechanic source files",
  "4,473 source-evidence rows",
  "24 nonblocking research gaps",
  "155 ownership receipts",
  "65 normalized files",
  "`ForbiddenReferenceOnly`",
])
  assert(evidence.includes(phrase), `Phase 2 evidence omits ${phrase}`);

console.log(
  "Unknowable Domain Phase 2 pack verified (5,377/5,377 DataReady; " +
  "41 mechanic sources/rules; 4,473 sources; 24 fixtures/gaps; " +
  "155 ownership receipts; 65 normalized files; runtime forbidden).",
);

function json(relative) {
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
