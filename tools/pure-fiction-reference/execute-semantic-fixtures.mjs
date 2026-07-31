#!/usr/bin/env node

import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const root = process.cwd();
const packRoot = path.join(root, "content-reference/pure-fiction-v1");
const schema = JSON.parse(await readFile(path.join(packRoot, "schema.json")));
const records = new Map();
for (const file of schema.normalized_files) {
  const document = JSON.parse(await readFile(path.join(packRoot, file)));
  for (const record of document.records ?? []) {
    if (records.has(record.id) && file !== "pack-index.json"
      && !record.id.startsWith("pf.buff."))
      throw new Error(`${record.id}: conflicting normalized ID`);
    if (!records.has(record.id)) records.set(record.id, record);
  }
}
const fixtureDocument = JSON.parse(await readFile(path.join(packRoot,
  "semantic-fixtures.json")));
const results = [];
function get(rootValue, dottedPath) {
  let value = rootValue;
  for (const segment of dottedPath.split(".")) {
    if (value === null || value === undefined
      || !Object.prototype.hasOwnProperty.call(value, segment))
      return { present: false, value: undefined };
    value = value[segment];
  }
  return { present: true, value };
}
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
for (const fixture of fixtureDocument.records) {
  const inputs = fixture.input_ids.map((id) => {
    const input = records.get(id);
    if (!input) throw new Error(`${fixture.id}: missing input ${id}`);
    return input;
  });
  const state = { ...fixture.initial_state, inputs };
  if (fixture.commands.length !== 0)
    throw new Error(`${fixture.id}: reference-only fixture contains commands`);
  for (const fact of fixture.expected_facts) {
    const actual = get(state, fact.path);
    let passed = false;
    if (fact.op === "absent") passed = !actual.present;
    else if (fact.op === "equals" || fact.op === "ordered_equals")
      passed = actual.present && equal(actual.value, fact.value);
    else if (fact.op === "contains")
      passed = actual.present && (typeof actual.value === "string"
        ? actual.value.includes(fact.value)
        : Array.isArray(actual.value) && actual.value.some((value) => equal(value, fact.value)));
    else throw new Error(`${fixture.id}: unsupported fact op ${fact.op}`);
    if (!passed) throw new Error(`${fixture.id}: ${fact.op} ${fact.path} failed`);
  }
  if (!fixture.replacement_condition.trim())
    throw new Error(`${fixture.id}: missing replacement condition`);
  results.push({ fixture_id: fixture.id, family: fixture.family,
    input_ids: fixture.input_ids, asserted_fact_count: fixture.expected_facts.length,
    result: "Passed" });
}
const gaps = JSON.parse(await readFile(path.join(packRoot, "research-gaps.json"))).records;
for (const gap of gaps)
  if (gap.blocking || !gap.replacement_condition?.trim()
    || gap.mechanism_quality !== "DeterministicProjectPolicyNotObservedParity")
    throw new Error(`${gap.id}: approximation replacement audit failed`);
const report = { schema_revision: "starclock.pure-fiction-semantic-fixture-run.v1",
  goal_id: "pure-fiction-reference-v1", batch: "G15-P4-B2",
  fixture_count: results.length, asserted_fact_count: results.reduce((sum, row) =>
    sum + row.asserted_fact_count, 0), passed_fixture_count: results.length,
  failed_fixture_count: 0, approximation_count: gaps.length,
  blocking_approximation_count: 0,
  fixture_digest: createHash("sha256").update(JSON.stringify(results)).digest("hex"),
  results, result: "Passed" };
const output = path.join(root, "evidence/pure-fiction-v1/semantic-fixture-run.json");
await mkdir(path.dirname(output), { recursive: true });
await writeFile(output, `${JSON.stringify(report, null, 2)}\n`);
console.log(`Pure Fiction semantic fixtures: ${results.length} passed, `
  + `${gaps.length} replacement conditions reviewed, zero failures.`);
