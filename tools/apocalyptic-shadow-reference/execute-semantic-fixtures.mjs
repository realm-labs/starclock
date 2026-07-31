#!/usr/bin/env node

import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { root } from "./source.mjs";

const document = JSON.parse(await readFile(path.join(root,
  "content-reference/apocalyptic-shadow-v1/review-fixtures.json")));
const contract = JSON.parse(await readFile(path.join(root,
  "content-manifests/apocalyptic-shadow-v1/fixture-contract.json")));
const results = document.records.map((fixture) => ({
  fixture_id: fixture.fixture_id,
  family_id: fixture.family_id,
  passed: fixture.passed === true
    && JSON.stringify(fixture.expected) === JSON.stringify(fixture.actual),
  blocking_gap_ids: fixture.blocking_gap_ids,
}));
for (const family of contract.required_families) {
  if (results.filter((row) => row.family_id === family.id && row.passed).length
    < family.minimum_cases) throw new Error(`${family.id} minimum not met`);
}
const report = {
  schema_revision: "starclock.apocalyptic-shadow-semantic-results.v1",
  goal_id: "apocalyptic-shadow-reference-v1",
  batch: "G18-P4-B1",
  family_count: contract.required_families.length,
  fixture_count: results.length,
  passed_fixture_count: results.filter((row) => row.passed).length,
  failed_fixture_count: results.filter((row) => !row.passed).length,
  blocking_gap_count: results.flatMap((row) => row.blocking_gap_ids).length,
  results,
};
if (report.failed_fixture_count || report.blocking_gap_count)
  throw new Error("semantic fixture execution failed");
const output = path.join(root,
  "evidence/apocalyptic-shadow-reference-v1/semantic-fixture-results.json");
await mkdir(path.dirname(output), { recursive: true });
await writeFile(output, `${JSON.stringify(report, null, 2)}\n`);
console.log(`Apocalyptic Shadow semantic fixtures: ${results.length} passed.`);
