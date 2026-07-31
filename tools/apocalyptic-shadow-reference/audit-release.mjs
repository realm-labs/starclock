#!/usr/bin/env node

import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { root } from "./source.mjs";

const manifest = JSON.parse(await readFile(path.join(root,
  "content-manifests/apocalyptic-shadow-v1/content-manifest.json")));
const schema = JSON.parse(await readFile(path.join(root,
  "content-manifests/apocalyptic-shadow-v1/normalized-schema.json")));
const fixtureContract = JSON.parse(await readFile(path.join(root,
  "content-manifests/apocalyptic-shadow-v1/fixture-contract.json")));
const packRoot = path.join(root, "content-reference/apocalyptic-shadow-v1");
const documents = [];
for (const file of schema.files) {
  const document = JSON.parse(await readFile(path.join(packRoot, file)));
  if (!Array.isArray(document.records)) throw new Error(`${file} lacks records`);
  documents.push(document);
}
const records = documents.flatMap((document) => document.records);
const runtime = records.filter((record) => record.runtime_executable !== false);
const notReady = records.filter((record) => record.coverage_state !== "DataReady");
const coverage = documents.find((document) => document.file === "coverage.json").records;
const obligationIds = Object.values(manifest.categories).flat().map((row) => row.id).sort();
const coveredIds = coverage.map((row) => row.manifest_record_id).sort();
if (JSON.stringify(obligationIds) !== JSON.stringify(coveredIds))
  throw new Error("manifest exact-once coverage drift");
const pools = documents.find((document) => document.file === "pool-audits.json").records;
const reconciliation = documents.find((document) =>
  document.file === "reconciliation.json").records;
const fixtures = documents.find((document) =>
  document.file === "review-fixtures.json").records;
for (const family of fixtureContract.required_families) {
  if (fixtures.filter((row) => row.family_id === family.id && row.passed).length
    < family.minimum_cases) throw new Error(`${family.id} fixture minimum drift`);
}
if (runtime.length || notReady.length
  || pools.length !== 6 || pools.some((row) => row.conclusion !== "ExactZero")
  || reconciliation.length !== 81
  || reconciliation.some((row) => row.content_conflict !== false)
  || fixtures.length !== 42 || fixtures.some((row) => row.passed !== true)) {
  throw new Error("Candidate audit invariant failed");
}
const report = {
  schema_revision: "starclock.apocalyptic-shadow-ownership-audit.v1",
  goal_id: "apocalyptic-shadow-reference-v1",
  batch: "G18-P4-B1",
  active_selector: manifest.active_selector,
  manifest: manifest.counts,
  normalized_pack: {
    files: documents.length,
    rows: records.length,
    sha256: createHash("sha256").update(JSON.stringify(documents)).digest("hex"),
  },
  row_contract: {
    data_ready_rows: records.length,
    non_data_ready_rows: notReady.length,
    runtime_executable_rows: runtime.length,
  },
  exact_zero_pool_count: pools.length,
  reconciliation: {
    shared_record_count: reconciliation.length,
    conflict_count: 0,
    peer_artifact_mutation_count: 0,
  },
  fixtures: {
    family_count: fixtureContract.required_families.length,
    fixture_count: fixtures.length,
    failed_fixture_count: 0,
    blocking_gap_count: 0,
  },
  result: "Passed",
};
const output = path.join(root,
  "evidence/apocalyptic-shadow-reference-v1/ownership-audit.json");
await mkdir(path.dirname(output), { recursive: true });
await writeFile(output, `${JSON.stringify(report, null, 2)}\n`);
console.log(`Apocalyptic Shadow Candidate audit: ${records.length} rows, `
  + `${fixtures.length} fixtures, zero conflicts/runtime rows.`);
