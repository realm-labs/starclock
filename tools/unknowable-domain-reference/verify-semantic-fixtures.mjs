#!/usr/bin/env node

import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const write = args.includes("--write");
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--")) ?? ".",
);
const packRoot = path.join(root, "content-reference", "unknowable-domain-v1");
const contractPath = path.join(
  root,
  "content-manifests",
  "unknowable-domain-v1",
  "fixture-contract.json",
);
const schemaPath = path.join(
  root,
  "content-manifests",
  "unknowable-domain-v1",
  "normalized-schema.json",
);
const outputPath = path.join(
  root,
  "evidence",
  "unknowable-domain-reference-v1",
  "semantic-fixture-results.json",
);

const contract = json(contractPath);
const schema = json(schemaPath);
const valuesByFile = new Map(
  schema.files.map(({ file }) => [file, json(path.join(packRoot, file))]),
);
const fixtures = valuesByFile.get("review-fixtures.json");
const normalizedFamilies = valuesByFile.get("semantic-fixture-families.json");
const gaps = valuesByFile.get("research-gaps.json");
const rules = valuesByFile.get("mechanic-rules.json");
const sources = valuesByFile.get("sources.json");
const sourceById = uniqueMap(sources, ({ source_id: sourceId }) => sourceId, "source");
const contentById = new Map();
for (const [file, value] of valuesByFile) {
  if (!Array.isArray(value) || ["manifest.json", "pack-index.json"].includes(file)) {
    continue;
  }
  for (const row of value) {
    assert(!contentById.has(row.id), `${file}/${row.id}: duplicate stable ID`);
    contentById.set(row.id, { file, row });
  }
}
const fixtureById = uniqueMap(fixtures, ({ id }) => id, "fixture");
const gapByFamily = uniqueMap(gaps, ({ source_id: familyId }) => familyId, "gap family");
const normalizedFamilyById = uniqueMap(
  normalizedFamilies,
  ({ source_id: familyId }) => familyId,
  "normalized family",
);
const requiredFamilyById = uniqueMap(
  contract.required_families,
  ({ id }) => id,
  "required family",
);

assert(fixtures.length === 24, "semantic fixture denominator differs");
assert(requiredFamilyById.size === 24, "required family denominator differs");
assert(normalizedFamilyById.size === 24, "normalized family denominator differs");
assert(gapByFamily.size === 24, "research-gap family denominator differs");
const fixturesByFamily = Object.groupBy(fixtures, ({ family_id: familyId }) => familyId);
for (const family of requiredFamilyById.values()) {
  const matches = fixturesByFamily[family.id] ?? [];
  assert(
    matches.length === family.minimum_cases,
    `${family.id}: expected exactly ${family.minimum_cases} fixture(s)`,
  );
}
assert(
  Object.keys(fixturesByFamily).every((id) => requiredFamilyById.has(id)),
  "fixture uses a family outside the frozen contract",
);

const fixtureResults = [];
let operationCount = 0;
let assertionCount = 0;
let fixtureInputBindings = 0;
let fixtureEvidenceBindings = 0;
for (const fixture of fixtures) {
  const family = requiredFamilyById.get(fixture.family_id);
  const normalizedFamily = normalizedFamilyById.get(fixture.family_id);
  const gap = gapByFamily.get(fixture.family_id);
  assert(family, `${fixture.id}: required family does not resolve`);
  assert(normalizedFamily, `${fixture.id}: normalized family does not resolve`);
  assert(gap, `${fixture.id}: research gap does not resolve`);
  auditFixtureShape(fixture, family, normalizedFamily, gap);

  const records = fixture.source_record_ids.map((id) => {
    const value = contentById.get(id);
    assert(value, `${fixture.id}: source record ${id} does not resolve`);
    assert(value.row.coverage_state === "DataReady", `${fixture.id}: ${id} is not DataReady`);
    fixtureInputBindings += 1;
    return value.row;
  });
  const dedicatedPolicySourceId =
    `source.goal10.project-policy.semantic-fixture-${fixture.family_id}`;
  const policySource = sourceById.get(dedicatedPolicySourceId);
  assert(policySource, `${fixture.id}: dedicated policy source does not resolve`);
  assert(
    policySource.evidence_quality === "ProjectPolicy",
    `${fixture.id}: dedicated policy source is not ProjectPolicy`,
  );

  const assertions = fixture.expected_facts.map((expected, index) => {
    const operation = fixture.ordered_operations[index];
    assert(operation.fact === expected.fact, `${fixture.id}: operation/assertion fact differs`);
    assert(
      expected.assertion === "ExactOrExplicitlyPolicyBound",
      `${fixture.id}: unsupported assertion`,
    );
    assert(expected.runtime_claim === false, `${fixture.id}: runtime claim leaked`);
    return {
      ordinal: index + 1,
      fact: expected.fact,
      assertion: expected.assertion,
      disposition: "ExplicitlyPolicyBound",
      policy_source_id: dedicatedPolicySourceId,
      runtime_claim: false,
      result: "pass",
    };
  });
  const actual = {
    candidate_lane: fixture.preconditions.candidate_lane,
    runtime_loading: fixture.preconditions.runtime_loading,
    source_record_ids: records.map(({ id }) => id),
    candidate_order: fixture.input.candidate_order,
    unresolved_behavior: fixture.input.unresolved_behavior,
    reviewed_facts: assertions.map(({ fact, disposition }) => ({ fact, disposition })),
  };
  operationCount += fixture.ordered_operations.length;
  assertionCount += assertions.length;
  fixtureResults.push({
    fixture_id: fixture.id,
    family_id: fixture.family_id,
    evidence_quality: fixture.fixture_evidence_quality,
    source_record_count: records.length,
    evidence_binding_count: fixture.evidence_refs.length,
    operation_count: fixture.ordered_operations.length,
    assertion_count: assertions.length,
    assertions,
    policy_source_ids: fixture.source_refs
      .filter(({ evidence_quality: value }) => value === "ProjectPolicy")
      .map(({ source_id: sourceId }) => sourceId),
    trace_sha256: sha256(
      canonical({
        preconditions: fixture.preconditions,
        input: fixture.input,
        ordered_operations: fixture.ordered_operations,
        actual,
      }),
    ),
    runtime_executable: false,
    result: "pass",
  });
}
assert(operationCount === 93, "ordered-operation denominator differs");
assert(assertionCount === 93, "assertion denominator differs");
assert(fixtureInputBindings === 66, "fixture input-binding denominator differs");
assert(fixtureEvidenceBindings === 126, "fixture evidence-binding denominator differs");

assert(rules.length === 41, "mechanic-rule denominator differs");
const ruleFamilyCounts = {};
for (const rule of rules) {
  assert(requiredFamilyById.has(rule.family_id), `${rule.id}: unknown fixture family`);
  assert(
    rule.runtime_lowered === false && rule.fixture_ids.length === 1,
    `${rule.id}: reference-only execution boundary differs`,
  );
  const sourceFile = contentById.get(rule.source_file_id);
  assert(
    sourceFile?.file === "mechanic-source-files.json",
    `${rule.id}: mechanic source does not resolve`,
  );
  const fixture = fixtureById.get(rule.fixture_ids[0]);
  assert(fixture, `${rule.id}: fixture does not resolve`);
  assert(
    fixture.family_id === rule.family_id,
    `${rule.id}: fixture family does not match rule family`,
  );
  ruleFamilyCounts[rule.family_id] = (ruleFamilyCounts[rule.family_id] ?? 0) + 1;
}

const policySources = sources.filter(
  ({ evidence_quality: value }) => value === "ProjectPolicy",
);
assert(policySources.length === 29, "ProjectPolicy source denominator differs");
const affectedByPolicySource = new Map(
  policySources.map(({ source_id: sourceId }) => [sourceId, []]),
);
let policyQualityRows = 0;
for (const { file, row } of contentById.values()) {
  if (row.evidence_quality === "ProjectPolicy") policyQualityRows += 1;
  for (const sourceRef of row.source_refs) {
    const affected = affectedByPolicySource.get(sourceRef.source_id);
    if (!affected) continue;
    const source = sourceById.get(sourceRef.source_id);
    assert(sourceRef.note === source.note, `${file}/${row.id}: policy note differs`);
    assert(
      sourceRef.replacement_condition === source.replacement_condition,
      `${file}/${row.id}: policy replacement condition differs`,
    );
    affected.push({ file, id: row.id });
  }
}
assert(policyQualityRows === 178, "ProjectPolicy row denominator differs");

const approximationResults = [];
let affectedPolicyBindingCount = 0;
for (const source of policySources) {
  assert(nonempty(source.note), `${source.source_id}: policy note is empty`);
  assert(
    nonempty(source.replacement_condition) &&
      source.replacement_condition.startsWith("Replace"),
    `${source.source_id}: replacement condition is not actionable`,
  );
  const affected = affectedByPolicySource.get(source.source_id);
  assert(affected.length > 0, `${source.source_id}: policy source is orphaned`);
  const affectedIds = new Set(affected.map(({ id }) => id));
  const linkedFixtureIds = fixtures
    .filter(
      (fixture) =>
        fixture.source_refs.some(({ source_id: sourceId }) => sourceId === source.source_id) ||
        fixture.source_record_ids.some((id) => affectedIds.has(id)),
    )
    .map(({ id }) => id)
    .sort();
  assert(linkedFixtureIds.length > 0, `${source.source_id}: no semantic fixture is linked`);
  affectedPolicyBindingCount += affected.length;
  approximationResults.push({
    policy_source_id: source.source_id,
    evidence_quality: source.evidence_quality,
    affected_record_count: affected.length,
    fixture_ids: linkedFixtureIds,
    note_sha256: sha256(source.note),
    replacement_condition: source.replacement_condition,
    replacement_condition_sha256: sha256(source.replacement_condition),
    result: "pass",
  });
}
assert(affectedPolicyBindingCount === 191, "policy affected-binding denominator differs");

for (const gap of gaps) {
  assert(
    gap.state === "PolicyBound" && gap.blocking === false,
    `${gap.id}: research gap is blocking`,
  );
  assert(gap.owner === "G10-P4-B2", `${gap.id}: research-gap owner differs`);
  const fixture = fixturesByFamily[gap.source_id]?.[0];
  assert(fixture, `${gap.id}: fixture does not resolve`);
  assert(
    canonical(gap.affected_data_ids) === canonical(fixture.source_record_ids),
    `${gap.id}: affected records differ from fixture inputs`,
  );
  const policySourceId = `source.goal10.project-policy.semantic-fixture-${gap.source_id}`;
  const source = sourceById.get(policySourceId);
  assert(source, `${gap.id}: policy source does not resolve`);
  assert(
    gap.replacement_condition === source.replacement_condition,
    `${gap.id}: replacement condition differs`,
  );
}

const report = {
  schema_revision: "starclock.unknowable-domain-semantic-fixture-results.v1",
  goal_id: "unknowable-domain-reference-v1",
  executed_at: "2026-07-29",
  result: "pass",
  inputs: {
    fixture_contract_sha256: sha256(fs.readFileSync(contractPath)),
    normalized_schema_sha256: sha256(fs.readFileSync(schemaPath)),
    fixture_data_sha256: sha256(
      fs.readFileSync(path.join(packRoot, "review-fixtures.json")),
    ),
    research_gap_data_sha256: sha256(
      fs.readFileSync(path.join(packRoot, "research-gaps.json")),
    ),
    mechanic_rule_data_sha256: sha256(
      fs.readFileSync(path.join(packRoot, "mechanic-rules.json")),
    ),
  },
  summary: {
    required_families: requiredFamilyById.size,
    executed_fixtures: fixtureResults.length,
    ordered_operations: operationCount,
    assertions: assertionCount,
    source_record_bindings: fixtureInputBindings,
    evidence_bindings: fixtureEvidenceBindings,
    mechanic_rules: rules.length,
    approximation_boundaries: policySources.length,
    replacement_conditions_verified: approximationResults.length,
    affected_policy_record_bindings: affectedPolicyBindingCount,
    policy_quality_rows: policyQualityRows,
    research_gaps: gaps.length,
    blocking_gaps: 0,
    runtime_executions: 0,
    failed_assertions: 0,
  },
  mechanic_rule_families: sortedObject(ruleFamilyCounts),
  fixtures: fixtureResults.sort((left, right) =>
    left.family_id.localeCompare(right.family_id),
  ),
  approximations: approximationResults.sort((left, right) =>
    left.policy_source_id.localeCompare(right.policy_source_id),
  ),
};
const encoded = `${JSON.stringify(report, null, 2)}\n`;
if (write) {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, encoded);
} else {
  assert(
    fs.existsSync(outputPath),
    "semantic fixture evidence is missing; run with --write",
  );
  assert(
    fs.readFileSync(outputPath, "utf8") === encoded,
    "semantic fixture evidence drifted",
  );
}
console.log(
  `Unknowable Domain semantic fixtures passed (${fixtureResults.length} families; ` +
    `${operationCount} ordered review operations; ${assertionCount} assertions; ` +
    `${approximationResults.length} replacement conditions; ` +
    `${affectedPolicyBindingCount} policy bindings; zero runtime executions).`,
);

function auditFixtureShape(fixture, family, normalizedFamily, gap) {
  for (const field of contract.required_fields) {
    assert(Object.hasOwn(fixture, field), `${fixture.id}: missing ${field}`);
  }
  assert(fixture.coverage_state === "DataReady", `${fixture.id}: not DataReady`);
  assert(
    fixture.fixture_evidence_quality === fixture.evidence_quality,
    `${fixture.id}: fixture evidence quality differs`,
  );
  assert(
    fixture.fixture_evidence_quality === "ProjectPolicy",
    `${fixture.id}: review fixture is not explicitly policy-bound`,
  );
  assert(fixture.runtime_executable === false, `${fixture.id}: runtime execution leaked`);
  assert(normalizedFamily.runtime_executable === false, `${fixture.id}: family became runtime`);
  assert(
    canonical(normalizedFamily.must_cover) === canonical(family.must_cover),
    `${fixture.id}: normalized family facts differ from the contract`,
  );
  assert(
    fixture.source_record_ids.length > 0 &&
      canonical(fixture.source_record_ids) ===
        canonical([...fixture.source_record_ids].sort()) &&
      canonical(fixture.source_record_ids) ===
        canonical(normalizedFamily.selected_source_record_ids) &&
      canonical(fixture.source_record_ids) === canonical(gap.affected_data_ids),
    `${fixture.id}: source records are empty, unordered or differ from family closure`,
  );
  assert(
    fixture.preconditions.candidate_lane === "ReferenceCandidate" &&
      fixture.preconditions.runtime_loading === "Forbidden" &&
      fixture.preconditions.source_record_count === fixture.source_record_ids.length,
    `${fixture.id}: preconditions differ`,
  );
  assert(
    fixture.input.kind === "SemanticReview" &&
      fixture.input.candidate_order === "StableIdAscending" &&
      fixture.input.unresolved_behavior === "FailClosed",
    `${fixture.id}: deterministic review input differs`,
  );
  assert(
    fixture.ordered_operations.length === family.must_cover.length &&
      fixture.ordered_operations.every(
        ({ ordinal, operation, fact }, index) =>
          ordinal === index + 1 &&
          operation === "ReviewRequiredFact" &&
          fact === family.must_cover[index],
      ),
    `${fixture.id}: review operation trace differs`,
  );
  assert(
    fixture.expected_facts.length === family.must_cover.length &&
      fixture.expected_facts.every(
        ({ fact, assertion, runtime_claim: runtimeClaim }, index) =>
          fact === family.must_cover[index] &&
          assertion === "ExactOrExplicitlyPolicyBound" &&
          runtimeClaim === false,
      ),
    `${fixture.id}: expected facts differ`,
  );
  assert(
    fixture.evidence_refs.length > 0 &&
      canonical(fixture.evidence_refs) ===
        canonical(fixture.source_refs.map(({ source_id: sourceId }) => sourceId)),
    `${fixture.id}: evidence references differ from provenance`,
  );
  for (const id of fixture.evidence_refs) {
    assert(sourceById.has(id), `${fixture.id}: evidence ${id} does not resolve`);
    fixtureEvidenceBindings += 1;
  }
  const dedicatedPolicySource =
    `source.goal10.project-policy.semantic-fixture-${fixture.family_id}`;
  assert(
    fixture.evidence_refs.includes(dedicatedPolicySource),
    `${fixture.id}: dedicated policy source is missing`,
  );
}

function uniqueMap(values, keyOf, label) {
  const result = new Map();
  for (const value of values) {
    const key = keyOf(value);
    assert(!result.has(key), `duplicate ${label} ${key}`);
    result.set(key, value);
  }
  return result;
}

function sortedObject(value) {
  return Object.fromEntries(
    Object.entries(value).sort(([left], [right]) => left.localeCompare(right)),
  );
}

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
}

function nonempty(value) {
  return typeof value === "string" && value.trim().length > 0;
}

function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
