#!/usr/bin/env node

import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const check = process.argv.includes("--check");
const packRoot = path.join(
  root,
  "content-reference/anomaly-arbitration-v1",
);
const output = path.join(
  root,
  "evidence/anomaly-arbitration-reference-v1/semantic-fixture-results.json",
);
const contract = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/fixture-contract.json",
)));
const schema = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/normalized-schema.json",
)));

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function canonical(value) {
  return JSON.stringify(value, Object.keys(value).sort());
}

function digest(value) {
  return createHash("sha256").update(
    JSON.stringify(value, null, 0),
  ).digest("hex");
}

const documents = new Map();
const rows = new Map();
for (const file of schema.files) {
  const document = JSON.parse(await readFile(path.join(packRoot, file.file)));
  documents.set(file.file, document.records);
  for (const row of document.records) {
    assert(!rows.has(row.id), `duplicate stable row ${row.id}`);
    rows.set(row.id, row);
  }
}
const sources = new Map(
  documents.get("sources.json").map((row) => [row.id, row]),
);
const rules = new Map(
  documents.get("mechanic-rules.json").map((row) => [row.family_id, row]),
);
const fixtures = documents.get("review-fixtures.json");
const contractFamilies = new Map(
  contract.required_families.map((family) => [family.id, family]),
);
assert(contractFamilies.size === 18 && rules.size === 18,
  "semantic family denominator drift");

const results = [];
let operationCount = 0;
for (const fixture of fixtures) {
  const family = contractFamilies.get(fixture.family_id);
  const rule = rules.get(fixture.family_id);
  assert(family && rule, `${fixture.id}: unknown semantic family`);
  assert(rule.fixture_ids.includes(fixture.id),
    `${fixture.id}: mechanic rule does not select fixture`);
  assert(fixture.executable_runtime_fixture === false
    && fixture.runtime_executable === false,
  `${fixture.id}: reference fixture became executable`);
  assert(fixture.preconditions.profile === "anomaly-arbitration-v1"
    && fixture.preconditions.game_version === "4.4"
    && fixture.preconditions.source_records_data_ready === true
    && fixture.input.kind === "ReferenceReview",
  `${fixture.id}: deterministic precondition/input drift`);
  const sourceRows = fixture.source_record_ids.map((id) => {
    const row = rows.get(id);
    assert(row, `${fixture.id}: unresolved source record ${id}`);
    assert(row.coverage_state === "DataReady" && !row.runtime_executable,
      `${fixture.id}/${id}: source record readiness drift`);
    return row;
  });
  const evidenceRows = fixture.evidence_refs.map((id) => {
    const row = sources.get(id);
    assert(row, `${fixture.id}: unresolved evidence ${id}`);
    assert(row.coverage_state === "DataReady" && !row.runtime_executable,
      `${fixture.id}/${id}: evidence readiness drift`);
    return row;
  });
  const operations = fixture.ordered_operations;
  const expected = fixture.expected_facts;
  assert(operations.length === expected.length
    && operations.every((operation, index) =>
      operation.order === index + 1
        && operation.operation === "VerifyDeclaredFact"
        && operation.fact === expected[index].fact)
    && expected.every((fact) =>
      fact.scope === fixture.family_id
        && fact.expected === "CoveredByEvidenceOrExplicitPolicyBoundary"),
  `${fixture.id}: ordered operation/expectation drift`);
  const operationFacts = operations.map(({ fact }) => fact);
  const requiredFacts = rule.required_fact_order.map(({ fact }) => fact);
  assert(canonical(operationFacts) === canonical(requiredFacts)
    && family.must_cover.every((fact) => operationFacts.includes(fact)),
  `${fixture.id}: required fact order/coverage drift`);
  assert(rule.source_record_ids.every((id) =>
    fixture.source_record_ids.includes(id)
      || fixtures.some((candidate) =>
        candidate.family_id === fixture.family_id
          && candidate.source_record_ids.includes(id))),
  `${fixture.id}: family source closure drift`);
  const hasExactEvidence = evidenceRows.some((row) =>
    ["ExactStructured", "ExactPublicText", "Observed"].includes(
      row.source_evidence_quality,
    ));
  const hasPolicyBoundary = evidenceRows.some((row) =>
    ["ApproximateFromReleasedText", "ProjectPolicy"].includes(
      row.source_evidence_quality,
    ) || row.source_mechanism_quality === "PolicyBoundary");
  assert(hasExactEvidence || hasPolicyBoundary,
    `${fixture.id}: no admissible evidence mode`);
  operationCount += operations.length;
  results.push({
    fixture_id: fixture.id,
    family_id: fixture.family_id,
    case: fixture.input.case,
    source_record_count: sourceRows.length,
    evidence_ref_count: evidenceRows.length,
    ordered_operation_count: operations.length,
    exact_evidence_present: hasExactEvidence,
    policy_boundary_present: hasPolicyBoundary,
    trace_sha256: digest({
      preconditions: fixture.preconditions,
      input: fixture.input,
      operations,
      expected,
      source_record_ids: fixture.source_record_ids,
      evidence_refs: fixture.evidence_refs,
    }),
    result: "Passed",
  });
}
assert(results.length === 23, "fixture case denominator drift");
for (const family of contract.required_families) {
  assert(results.filter((row) => row.family_id === family.id).length
    >= family.minimum_cases,
  `${family.id}: fixture minimum drift`);
}

const gaps = documents.get("research-gaps.json");
const gapResults = gaps.map((gap) => {
  assert(gap.blocking === false
    && gap.owner_batch
    && gap.replacement_condition
    && gap.evidence_boundary
    && gap.selected_policy === "PreserveUnavailableWithoutInventingParity"
    && gap.affected_record_ids.length > 0,
  `${gap.id}: replacement contract drift`);
  for (const id of gap.affected_record_ids) {
    const affected = rows.get(id);
    assert(affected && affected.coverage_state === "DataReady"
      && !affected.runtime_executable,
    `${gap.id}: unresolved or executable affected record ${id}`);
  }
  assert(gap.source_refs.every((source) =>
    [...sources.values()].some((row) => row.source_id === source.source_id)),
  `${gap.id}: unresolved evidence source`);
  return {
    gap_id: gap.id,
    owner: gap.owner_batch,
    affected_record_count: gap.affected_record_ids.length,
    blocking: false,
    replacement_condition_present: true,
    exact_parity_claim: false,
    result: "Passed",
  };
});
assert(gapResults.length === 9, "research-gap denominator drift");

const report = {
  schema_revision: "starclock.anomaly-arbitration-semantic-results.v1",
  goal_id: "anomaly-arbitration-reference-v1",
  game_version: "4.4",
  role: contract.fixture_role,
  family_count: contractFamilies.size,
  fixture_count: results.length,
  ordered_operation_count: operationCount,
  passed_fixture_count: results.length,
  failed_fixture_count: 0,
  replacement_check_count: gapResults.length,
  blocking_gap_count: 0,
  fixtures: results,
  replacement_checks: gapResults,
};
const encoded = `${JSON.stringify(report, null, 2)}\n`;
if (check) {
  const current = await readFile(output, "utf8");
  assert(current === encoded, "semantic fixture result drift");
} else {
  await writeFile(output, encoded);
}
console.log(
  `Anomaly Arbitration semantic fixtures passed: ${results.length} cases, `
    + `${operationCount} ordered facts, 18 families, `
    + `${gapResults.length} replacement checks.`,
);
