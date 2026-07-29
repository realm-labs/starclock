#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile, stat } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const packRoot = path.join(
  root,
  "content-reference/anomaly-arbitration-v1",
);
const manifestPath = path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/content-manifest.json",
);
const schemaPath = path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/normalized-schema.json",
);
const fixturePath = path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/fixture-contract.json",
);
const inventoryPath = path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/source-inventory.json",
);
const [manifestBytes, schemaBytes, fixtureBytes, inventoryBytes] =
  await Promise.all([
    readFile(manifestPath),
    readFile(schemaPath),
    readFile(fixturePath),
    readFile(inventoryPath),
  ]);
const manifest = JSON.parse(manifestBytes);
const schema = JSON.parse(schemaBytes);
const fixtureContract = JSON.parse(fixtureBytes);

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function digest(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

execFileSync(process.execPath, [
  path.join(root, "tools/anomaly-arbitration-reference/build-pack.mjs"),
  "--check",
], { stdio: "inherit" });

const documents = new Map();
for (const contract of schema.files) {
  const bytes = await readFile(path.join(packRoot, contract.file));
  const document = JSON.parse(bytes);
  documents.set(contract.file, { bytes, document, contract });
  assert(document.schema_revision
    === "starclock.anomaly-arbitration-normalized-file.v1"
    && document.goal_id === "anomaly-arbitration-reference-v1"
    && document.profile === "anomaly-arbitration-v1"
    && document.file === contract.file
    && document.record_kind === contract.record_kind
    && Array.isArray(document.records),
  `${contract.file} normalized file envelope drift`);
  const ids = new Set();
  for (const record of document.records) {
    assert(!ids.has(record.id), `${contract.file} duplicate ${record.id}`);
    ids.add(record.id);
    for (const field of schema.common_envelope.required_fields)
      assert(record[field] !== undefined, `${record.id} lacks ${field}`);
    assert(record.kind === contract.record_kind
      && record.name_en && record.name_zh_cn
      && record.summary_en && record.summary_zh_cn
      && record.coverage_state === "DataReady"
      && record.runtime_executable === false,
    `${record.id} authoring/runtime boundary drift`);
    for (const source of record.source_refs) {
      for (const field of schema.types.source_ref.required_fields)
        assert(source[field] !== undefined && source[field] !== "",
          `${record.id} source lacks ${field}`);
      assert(/^[0-9a-f]{64}$/u.test(source.sha256),
        `${record.id} source digest drift`);
    }
  }
}
assert(documents.size === 37, "normalized file denominator drift");

const manifestRows = Object.entries(manifest.categories).flatMap(
  ([category, { records = [] }]) => records.map((record) => ({
    category,
    record,
    manifestId: `${category}:${record.id}`,
  })),
);
assert(manifestRows.length === 392, "manifest denominator drift");
const coverage = documents.get("coverage.json").document.records;
assert(coverage.length === 392, "coverage row count drift");
const expectedManifestIds = manifestRows.map(({ manifestId }) => manifestId)
  .sort(compareText);
const coveredManifestIds = coverage.map(
  ({ manifest_category: category, manifest_record_id: id }) =>
    `${category}:${id}`,
).sort(compareText);
assert(JSON.stringify(coveredManifestIds)
  === JSON.stringify(expectedManifestIds),
"coverage exact-once identity drift");
assert(coverage.every((row) => row.required === 1
  && row.accounted === 1 && row.data_ready === 1
  && row.coverage_percent === "100"
  && row.normalized_record_ids.length > 0),
"coverage readiness drift");

const rules = documents.get("mechanic-rules.json").document.records;
const fixtures = documents.get("review-fixtures.json").document.records;
assert(rules.length === 18 && fixtures.length === 23,
  "rule/fixture denominator drift");
const requiredFamilies = new Set(fixtureContract.required_families.map(
  ({ id }) => id,
));
assert(JSON.stringify([...new Set(rules.map(({ family_id: id }) => id))]
  .sort(compareText)) === JSON.stringify([...requiredFamilies].sort(compareText)),
"mechanic-rule family drift");
const sourceIds = new Set(documents.get("sources.json").document.records.map(
  ({ id }) => id,
));
for (const family of fixtureContract.required_families) {
  const cases = fixtures.filter(
    ({ family_id: familyId }) => familyId === family.id,
  );
  assert(cases.length >= family.minimum_cases,
    `${family.id} fixture minimum drift`);
  for (const fixture of cases) {
    assert(fixture.source_record_ids.length > 0
      && fixture.ordered_operations.length > 0
      && fixture.expected_facts.length > 0
      && fixture.evidence_refs.length > 0
      && fixture.evidence_refs.every((id) => sourceIds.has(id))
      && fixture.executable_runtime_fixture === false,
    `${fixture.id} fixture contract drift`);
    const facts = new Set(fixture.ordered_operations.map(({ fact }) => fact));
    for (const required of family.must_cover)
      assert(facts.has(required), `${fixture.id} lacks ${required}`);
  }
}

const reconciliation =
  documents.get("reconciliation.json").document.records;
const shared = manifestRows.filter(
  ({ record }) => record.ownership === "Shared",
);
assert(reconciliation.length === 316 && shared.length === 316,
  "shared reconciliation denominator drift");
const expectedSharedKeys = shared.map(({ record }) =>
  `${record.source_path}\0${record.row_locator}\0${record.evidence_sha256}`)
  .sort(compareText);
const actualSharedKeys = reconciliation.map((row) =>
  `${row.source_path}\0${row.row_locator}\0${row.evidence_sha256}`)
  .sort(compareText);
assert(JSON.stringify(actualSharedKeys) === JSON.stringify(expectedSharedKeys)
  && reconciliation.every((row) =>
    row.conflict_state === "None"
    && row.peer_match_state === "AbsentFromCommittedPeerManifest"),
"shared reconciliation identity/state drift");

const gaps = documents.get("research-gaps.json").document.records;
assert(gaps.length === 9
  && gaps.every((row) => row.blocking === false
    && row.owner_batch && row.replacement_condition
    && row.affected_record_ids.length > 0),
"research-gap closure drift");
const receipt = documents.get("manifest.json").document.records[0];
assert(receipt.content_manifest_sha256 === digest(manifestBytes)
  && receipt.source_inventory_sha256 === digest(inventoryBytes)
  && receipt.normalized_schema_sha256 === digest(schemaBytes)
  && receipt.fixture_contract_sha256 === digest(fixtureBytes)
  && receipt.content_manifest_obligations === 392
  && receipt.normalized_file_count === 37
  && receipt.bundle_state === "Candidate",
"manifest receipt drift");

const index = documents.get("pack-index.json").document.records;
const indexedNames = schema.files.map(({ file }) => file)
  .filter((name) => name !== "pack-index.json");
assert(index.length === 36
  && JSON.stringify(index.map(({ file_name: name }) => name))
    === JSON.stringify(indexedNames),
"pack index file ordering drift");
for (const row of index) {
  const { bytes, document } = documents.get(row.file_name);
  const metadata = await stat(path.join(packRoot, row.file_name));
  assert(row.sha256 === digest(bytes)
    && row.byte_count === metadata.size
    && row.row_count === document.records.length,
  `${row.file_name} pack receipt drift`);
}

const fileDigest = digest(documents.get("pack-index.json").bytes);
console.log(
  `Anomaly Arbitration pack verified: 37 files, 392/392 DataReady, `
    + `18 rule families, 23 fixtures, 316 shared receipts, `
    + `pack-index=${fileDigest}.`,
);
