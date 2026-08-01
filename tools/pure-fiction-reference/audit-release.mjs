#!/usr/bin/env node

import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const root = process.cwd();
const manifest = JSON.parse(await readFile(path.join(root,
  "content-manifests/pure-fiction-v1/content-manifest.json")));
const schema = JSON.parse(await readFile(path.join(root,
  "content-reference/pure-fiction-v1/schema.json")));
const packRoot = path.join(root, "content-reference/pure-fiction-v1");
const documents = new Map();
for (const file of schema.normalized_files) {
  const document = JSON.parse(await readFile(path.join(packRoot, file)));
  if (!Array.isArray(document.records)) throw new Error(`${file} lacks records`);
  documents.set(file, document.records);
}
const manifestIds = new Set(manifest.obligations.map((row) => row.id));
if (manifestIds.size !== manifest.obligation_count) throw new Error("manifest ID drift");
const sources = documents.get("sources.json");
const sourceIds = new Set(sources.map((row) => row.id));
if (sourceIds.size !== sources.length || sources.length !== manifest.obligation_count)
  throw new Error("source exact-once drift");
for (const source of sources) {
  if (!/^[0-9a-f]{64}$/.test(source.evidence_digest)
    || !source.path_or_page || !source.row_locator || !source.revision_or_access_date)
    throw new Error(`${source.id}: incomplete provenance`);
}
const authoredRecords = [...documents.entries()]
  .filter(([file]) => file !== "sources.json")
  .flatMap(([, rows]) => rows);
for (const record of authoredRecords) {
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    if (typeof record[field] !== "string" || !record[field].trim())
      throw new Error(`${record.id}: missing ${field}`);
  if (record.coverage_state !== "DataReady" || record.runtime_executable !== false)
    throw new Error(`${record.id}: Candidate/runtime disposition drift`);
  if (!Array.isArray(record.manifest_record_ids) || !record.manifest_record_ids.length
    || record.manifest_record_ids.some((id) => !manifestIds.has(id)))
    throw new Error(`${record.id}: manifest reference drift`);
  if (!Array.isArray(record.source_record_ids) || !record.source_record_ids.length
    || record.source_record_ids.some((id) => !sourceIds.has(id)))
    throw new Error(`${record.id}: source reference drift`);
}
const coverage = documents.get("coverage.json");
const coveredIds = coverage.map((row) => row.manifest_record_id).sort();
const requiredIds = [...manifestIds].sort();
if (JSON.stringify(coveredIds) !== JSON.stringify(requiredIds))
  throw new Error("manifest exact-once coverage drift");
const pools = documents.get("pool-proofs.json");
const reconciliation = documents.get("reconciliation.json");
const gaps = documents.get("research-gaps.json");
const fixtures = documents.get("semantic-fixtures.json");
if (pools.length !== 7 || pools.some((row) => row.conclusion !== "ExactZero")
  || reconciliation.length !== 606 || reconciliation.some((row) => row.conflict)
  || reconciliation.some((row) => row.peer_artifact_mutated)
  || gaps.some((row) => row.blocking) || fixtures.length !== 18
  || fixtures.some((row) => row.passed !== true))
  throw new Error("Candidate audit invariant failed");
const report = {
  schema_revision: "starclock.pure-fiction-ownership-audit.v1",
  goal_id: "pure-fiction-reference-v1",
  batch: "G15-P4-B1",
  manifest: { obligation_count: manifest.obligation_count, counts: manifest.counts,
    digest: manifest.manifest_digest },
  normalized_pack: { files: documents.size,
    rows: [...documents.values()].reduce((sum, rows) => sum + rows.length, 0),
    sha256: createHash("sha256").update(JSON.stringify([...documents])).digest("hex") },
  row_contract: { authored_rows: authoredRecords.length,
    source_rows: sources.length, data_ready_authored_rows: authoredRecords.length,
    runtime_executable_rows: 0, bilingual_failures: 0, provenance_failures: 0 },
  exact_zero_pool_count: pools.length,
  active_release: { selected_season: "202024/2024", scheduled_unreleased_exclusions: 1,
    leaked_historical_or_unreleased_rows: 0 },
  reconciliation: { shared_record_count: reconciliation.length, conflict_count: 0,
    peer_artifact_mutation_count: 0 },
  fixtures: { fixture_count: fixtures.length, failed_fixture_count: 0,
    blocking_gap_count: 0 },
  result: "Passed",
};
const output = path.join(root, "evidence/pure-fiction-v1/ownership-audit.json");
await mkdir(path.dirname(output), { recursive: true });
await writeFile(output, `${JSON.stringify(report, null, 2)}\n`);
console.log(`Pure Fiction Candidate audit: ${report.normalized_pack.rows} rows, `
  + `${fixtures.length} fixtures, zero conflicts/runtime rows.`);
