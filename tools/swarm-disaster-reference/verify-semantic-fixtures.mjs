#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { canonical, sha256 } from "./lib/common.mjs";

const args = process.argv.slice(2);
const write = args.includes("--write");
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const packRoot = path.join(root, "content-reference", "swarm-disaster-v1");
const debugRoot = path.join(
  root,
  "config",
  "swarm-disaster-generated",
  "debug-json",
);
const evidencePath = path.join(
  root,
  "evidence",
  "swarm-disaster-reference-v1",
  "semantic-fixture-audit.json",
);
const contract = json(
  path.join(root, "content-manifests/swarm-disaster-v1/fixture-contract.json"),
);
const schema = json(
  path.join(root, "content-manifests/swarm-disaster-v1/normalized-schema.json"),
);
const fixtures = pack("review-fixtures.json");
const rules = pack("mechanic-rules.json");
const gaps = pack("research-gaps.json");
const sources = pack("sources.json");
const sourceById = new Map(sources.map((row) => [row.id, row]));
const records = collectPackRows();
const fixtureByFamily = index(fixtures, "family_id", "fixture family");
const ruleByFamily = index(rules, "family_id", "rule family");
const fixtureById = index(fixtures, "id", "fixture");
const contractByFamily = index(
  contract.required_families,
  "id",
  "contract family",
);
const familyResults = [];
let operationsExecuted = 0;
let factsExecuted = 0;
let selectedRecordOccurrences = 0;

assert(
  fixtures.length === 23
    && rules.length === 23
    && contract.required_families.length === 23,
  "semantic family denominator differs",
);
assert(
  fixtureByFamily.size === fixtures.length
    && ruleByFamily.size === rules.length
    && contractByFamily.size === contract.required_families.length,
  "semantic family is not exact-once",
);

for (const family of contract.required_families) {
  const fixture = fixtureByFamily.get(family.id);
  const rule = ruleByFamily.get(family.id);
  assert(fixture && rule, `${family.id} rule/fixture closure differs`);
  executeFixture(family, fixture, rule);
}
assert(
  operationsExecuted === 85
    && factsExecuted === 108
    && selectedRecordOccurrences === 76,
  "semantic execution denominator differs",
);

const gapResults = verifyReplacementConditions();
verifySoraExport();

const report = {
  schema_revision: "starclock.swarm-disaster-semantic-fixture-audit.v1",
  goal_id: "swarm-disaster-reference-v1",
  snapshot: "Version 4.4",
  fixture_contract_sha256: sha256(fs.readFileSync(
    path.join(root, "content-manifests/swarm-disaster-v1/fixture-contract.json"),
  )),
  normalized_schema_sha256: sha256(fs.readFileSync(
    path.join(root, "content-manifests/swarm-disaster-v1/normalized-schema.json"),
  )),
  denominators: {
    mechanic_families: fixtures.length,
    mechanic_rules: rules.length,
    semantic_fixtures: fixtures.length,
    selected_record_occurrences: selectedRecordOccurrences,
    ordered_operations_executed: operationsExecuted,
    expected_facts_executed: factsExecuted,
    policy_fixtures: fixtures.filter(
      ({ fixture_evidence_quality: quality }) => quality === "ProjectPolicy",
    ).length,
    exact_fixtures: fixtures.filter(
      ({ fixture_evidence_quality: quality }) => quality === "ExactStructured",
    ).length,
    replacement_boundaries: gaps.length,
    affected_record_bindings: gaps.reduce(
      (sum, gap) => sum + gap.affected_records.length,
      0,
    ),
    affected_fixture_bindings: gaps.reduce(
      (sum, gap) => sum + gap.affected_fixture_ids.length,
      0,
    ),
  },
  checks: {
    fixture_families_exact_once: true,
    source_records_resolve_and_are_data_ready: true,
    operation_order_matches_frozen_contract: true,
    expected_facts_executed: true,
    unresolved_behavior_explicit: true,
    runtime_execution_remains_forbidden: true,
    sora_rules_match_normalized_rows: true,
    sora_fixtures_match_normalized_rows: true,
    sora_research_gaps_match_normalized_rows: true,
    sora_affected_bindings_match_normalized_rows: true,
    every_policy_has_rejected_alternatives: true,
    every_policy_has_rationale: true,
    every_policy_has_affected_fixtures: true,
    every_policy_has_field_confidence: true,
    every_policy_has_replacement_condition: true,
    blocking_boundaries: 0,
  },
  families: familyResults,
  replacement_boundaries: gapResults,
};
writeOrCheckEvidence(report);
console.log(
  `Swarm Disaster semantic fixtures ${write ? "written" : "verified"}: ` +
  `${fixtures.length} families, ${operationsExecuted} ordered operations, ` +
  `${factsExecuted} expected facts, ${gaps.length} replaceable boundaries.`,
);

function executeFixture(family, fixture, rule) {
  assert(fixture.family_id === family.id && rule.family_id === family.id,
    `${family.id} identity differs`);
  assert(
    canonical(fixture.source_record_ids)
      === canonical(fixture.input.selected_record_ids),
    `${family.id} selected record list differs`,
  );
  assert(
    fixture.input.family_id === family.id
      && fixture.input.deterministic_seed === "0"
      && fixture.input.external_outcome_only
        === (family.id === "service-and-adventure"),
    `${family.id} fixture input contract differs`,
  );
  assert(
    canonical(fixture.preconditions) === canonical([
      { fact: "runtime_loading", value: "ForbiddenReferenceOnly" },
      { fact: "source_records_data_ready", value: true },
    ]),
    `${family.id} fixture preconditions differ`,
  );
  const selected = fixture.source_record_ids.map((id) => {
    const record = records.get(id);
    assert(record, `${family.id} selected record ${id} does not resolve`);
    assert(record.coverage_state === "DataReady",
      `${family.id} selected record ${id} is not DataReady`);
    return record;
  });
  assert(selected.length > 0, `${family.id} fixture has no selected records`);
  selectedRecordOccurrences += selected.length;

  assert(
    fixture.ordered_operations.length === family.must_cover.length
      && rule.program.length === family.must_cover.length,
    `${family.id} operation denominator differs`,
  );
  const expectedFacts = [];
  for (let index = 0; index < family.must_cover.length; index += 1) {
    const fact = family.must_cover[index];
    const operation = fixture.ordered_operations[index];
    const ruleStep = rule.program[index];
    assert(
      operation.sequence === index + 1
        && operation.fact === fact
        && ruleStep.sequence === index + 1
        && ruleStep.source_fact === fact
        && ruleStep.operation === operation.operation
        && ruleStep.unresolved_behavior === operation.unresolved_behavior
        && ["FailClosed", "NotApplicable"].includes(
          operation.unresolved_behavior,
        ),
      `${family.id} ordered operation ${index + 1} differs`,
    );
    expectedFacts.push({
      path: `must_cover.${slug(fact)}`,
      operator: "reviewed",
      value: true,
    });
    operationsExecuted += 1;
  }
  expectedFacts.push({
    path: "source_record_count",
    operator: "equals",
    value: String(selected.length),
  });
  assert(
    canonical(fixture.expected_facts) === canonical(expectedFacts),
    `${family.id} expected-fact program differs`,
  );
  const state = {
    must_cover: Object.fromEntries(
      family.must_cover.map((fact) => [slug(fact), true]),
    ),
    source_record_count: String(selected.length),
  };
  for (const fact of fixture.expected_facts) {
    assert(["reviewed", "equals"].includes(fact.operator),
      `${family.id} has unsupported operator ${fact.operator}`);
    const actual = project(state, fact.path);
    assert(actual === fact.value,
      `${family.id} expected ${fact.path}=${JSON.stringify(fact.value)}`);
    factsExecuted += 1;
  }

  const evidenceIds = fixture.source_refs.map(({ source_id: id }) => id);
  assert(canonical(evidenceIds) === canonical(fixture.evidence_refs),
    `${family.id} fixture evidence list differs`);
  const quality = aggregateQuality(evidenceIds.map((id) => {
    const source = sourceById.get(id);
    assert(source, `${family.id} evidence ${id} does not resolve`);
    return source.evidence_quality;
  }));
  assert(
    quality === fixture.fixture_evidence_quality
      && quality === fixture.evidence_quality,
    `${family.id} fixture evidence quality differs`,
  );
  if (quality === "ProjectPolicy")
    assert(fixture.note?.trim() && fixture.replacement_condition?.trim(),
      `${family.id} policy fixture is not replaceable`);
  else
    assert(
      quality === "ExactStructured"
        && fixture.note === undefined
        && fixture.replacement_condition === undefined,
      `${family.id} exact fixture carries policy metadata`,
    );
  assert(
    canonical(rule.fixture_ids) === canonical([fixture.id])
      && rule.execution_disposition === "ReferenceOnly"
      && rule.runtime_handler_id === ""
      && rule.triggers.length > 0
      && rule.state_slots.length > 0
      && rule.state_slots.every(({ owner }) =>
        ["Activity", "Battle"].includes(owner)),
    `${family.id} reference-only rule boundary differs`,
  );
  familyResults.push({
    family_id: family.id,
    rule_id: rule.id,
    fixture_id: fixture.id,
    selected_record_count: selected.length,
    ordered_operation_count: fixture.ordered_operations.length,
    expected_fact_count: fixture.expected_facts.length,
    evidence_quality: quality,
    domain: rule.domain,
    execution_disposition: rule.execution_disposition,
    result: "Passed",
  });
}

function verifyReplacementConditions() {
  const policySources = sources.filter(({ evidence_quality: quality }) =>
    ["ProjectPolicy", "ApproximateFromReleasedText"].includes(quality));
  const gapBySource = index(gaps, "policy_source_id", "research-gap source");
  assert(
    gaps.length === 31
      && policySources.length === gaps.length
      && gapBySource.size === gaps.length,
    "replacement-boundary denominator differs",
  );
  const allowedConfidence = new Set([
    "DeterministicPolicyNotObservedParity",
    "ReleasedTextCrossCheck",
  ]);
  const results = [];
  for (const source of policySources) {
    const gap = gapBySource.get(source.id);
    assert(gap, `${source.id} lacks a research-gap row`);
    assert(
      gap.state === "PolicyBound"
        && gap.gap_state === "PolicyBound"
        && gap.blocking === false
        && gap.coverage_state === "DataReady"
        && gap.evidence_quality === source.evidence_quality
        && gap.known_facts.trim()
        && gap.selected_policy.trim()
        && gap.note === source.note
        && gap.replacement_condition === source.replacement_condition
        && gap.replacement_condition.trim(),
      `${gap.id} replacement contract differs`,
    );
    assert(
      Array.isArray(gap.rejected_alternatives)
        && gap.rejected_alternatives.length === 2
        && new Set(gap.rejected_alternatives).size === 2
        && gap.rejected_alternatives.every((value) => value.trim()),
      `${gap.id} rejected alternatives differ`,
    );
    assert(gap.rationale?.trim(), `${gap.id} rationale is missing`);
    assert(allowedConfidence.has(gap.confidence),
      `${gap.id} confidence label differs`);
    assert(
      source.evidence_quality === "ProjectPolicy"
        ? gap.confidence === "DeterministicPolicyNotObservedParity"
        : gap.confidence === "ReleasedTextCrossCheck",
      `${gap.id} confidence does not match evidence quality`,
    );
    assert(
      Array.isArray(gap.affected_fixture_ids)
        && gap.affected_fixture_ids.length > 0
        && new Set(gap.affected_fixture_ids).size
          === gap.affected_fixture_ids.length,
      `${gap.id} affected fixture list differs`,
    );
    for (const id of gap.affected_fixture_ids)
      assert(fixtureById.has(id), `${gap.id} has unknown affected fixture ${id}`);
    for (const ref of gap.affected_records)
      assert(records.has(ref.id) && records.get(ref.id).__file === ref.file,
        `${gap.id} has unknown affected record ${ref.file}/${ref.id}`);
    results.push({
      research_gap_id: gap.id,
      policy_source_id: source.id,
      evidence_quality: source.evidence_quality,
      confidence: gap.confidence,
      affected_record_count: gap.affected_records.length,
      affected_fixture_ids: gap.affected_fixture_ids,
      replacement_condition_sha256: sha256(gap.replacement_condition),
      result: "Passed",
    });
  }
  return results;
}

function verifySoraExport() {
  const debugFixtures = debugRows("SwarmDisasterReviewFixture");
  const debugRules = debugRows("SwarmDisasterMechanicRule");
  const debugGaps = debugRows("SwarmDisasterResearchGap");
  const debugAffected = debugRows("SwarmDisasterResearchGapAffected");
  assert(
    debugFixtures.length === fixtures.length
      && debugRules.length === rules.length
      && debugGaps.length === gaps.length
      && debugAffected.length === 5_560,
    "Sora semantic/evidence row denominator differs",
  );
  const exportedFixtures = index(
    debugFixtures,
    "stable_key",
    "Sora fixture",
  );
  const exportedRules = index(debugRules, "stable_key", "Sora rule");
  const exportedGaps = index(debugGaps, "stable_key", "Sora research gap");
  for (const fixture of fixtures) {
    const row = exportedFixtures.get(fixture.id);
    assert(
      row
        && row.family_id === fixture.family_id
        && canonical(row.source_record_ids)
          === canonical(fixture.source_record_ids)
        && row.preconditions_json === canonical(fixture.preconditions)
        && row.input_json === canonical(fixture.input)
        && row.ordered_operations_json === canonical(
          fixture.ordered_operations,
        )
        && row.expected_facts_json === canonical(fixture.expected_facts)
        && canonical(row.evidence_refs) === canonical(fixture.evidence_refs)
        && row.fixture_evidence_quality === fixture.fixture_evidence_quality,
      `${fixture.id} Sora fixture export differs`,
    );
  }
  for (const rule of rules) {
    const row = exportedRules.get(rule.id);
    assert(
      row
        && row.family_id === rule.family_id
        && row.state_slots_json === canonical(rule.state_slots)
        && row.program_json === canonical(rule.program)
        && row.execution_disposition === "ReferenceOnly"
        && row.runtime_handler_id === null,
      `${rule.id} Sora rule export differs`,
    );
  }
  const gapNumericId = new Map();
  for (const gap of gaps) {
    const row = exportedGaps.get(gap.id);
    assert(
      row
        && row.policy_source_id === gap.policy_source_id
        && canonical(row.rejected_alternatives)
          === canonical(gap.rejected_alternatives)
        && row.rationale === gap.rationale
        && canonical(row.affected_fixture_ids)
          === canonical(gap.affected_fixture_ids)
        && row.confidence === gap.confidence
        && row.replacement_condition === gap.replacement_condition,
      `${gap.id} Sora research-gap export differs`,
    );
    gapNumericId.set(row.id, gap);
  }
  const expectedAffected = gaps.flatMap((gap) =>
    gap.affected_records.map((ref, ordinal) => ({
      gap,
      ordinal,
      file: ref.file,
      record_stable_key: ref.id,
    })));
  assert(expectedAffected.length === debugAffected.length,
    "research-gap affected-binding denominator differs");
  for (let index_ = 0; index_ < expectedAffected.length; index_ += 1) {
    const expected = expectedAffected[index_];
    const actual = debugAffected[index_];
    assert(
      gapNumericId.get(actual.research_gap_id)?.id === expected.gap.id
        && actual.ordinal === expected.ordinal
        && actual.file === expected.file
        && actual.record_stable_key === expected.record_stable_key,
      `${expected.gap.id}/${expected.ordinal} Sora affected binding differs`,
    );
  }
}

function collectPackRows() {
  const result = new Map();
  for (const contract_ of schema.files) {
    const value = pack(contract_.file);
    for (const row of Array.isArray(value) ? value : [value]) {
      if (typeof row?.id !== "string") continue;
      assert(!result.has(row.id), `duplicate pack record ${row.id}`);
      Object.defineProperty(row, "__file", {
        value: contract_.file,
        enumerable: false,
      });
      result.set(row.id, row);
    }
  }
  return result;
}

function aggregateQuality(values) {
  if (values.includes("ProjectPolicy")) return "ProjectPolicy";
  if (values.includes("ApproximateFromReleasedText"))
    return "ApproximateFromReleasedText";
  if (values.includes("Observed")) return "Observed";
  if (values.includes("ExactPublicText")) return "ExactPublicText";
  return "ExactStructured";
}

function debugRows(table) {
  return json(path.join(debugRoot, `${table}.json`)).table.rows
    .map(({ values }) => Object.fromEntries(Object.entries(values)
      .map(([key, value]) => [key, unwrap(value)])));
}

function unwrap(value) {
  if (value === "Null") return null;
  if (Object.hasOwn(value, "String")) return value.String;
  if (Object.hasOwn(value, "Integer")) return value.Integer;
  if (Object.hasOwn(value, "Bool")) return value.Bool;
  if (Object.hasOwn(value, "List")) return value.List.map(unwrap);
  throw new Error(`unsupported Sora debug value ${JSON.stringify(value)}`);
}

function project(value, dottedPath) {
  let current = value;
  for (const segment of dottedPath.split(".")) {
    assert(current && typeof current === "object"
      && Object.hasOwn(current, segment),
    `fixture result lacks ${dottedPath}`);
    current = current[segment];
  }
  return current;
}

function slug(value) {
  return value.toLowerCase().replaceAll(/[^a-z0-9]+/gu, "-")
    .replaceAll(/^-|-$/gu, "");
}

function index(values, field, label) {
  const result = new Map();
  for (const value of values) {
    assert(!result.has(value[field]),
      `${label} contains duplicate ${value[field]}`);
    result.set(value[field], value);
  }
  return result;
}

function writeOrCheckEvidence(value) {
  const encoded = `${JSON.stringify(value, null, 2)}\n`;
  if (write) {
    fs.mkdirSync(path.dirname(evidencePath), { recursive: true });
    fs.writeFileSync(evidencePath, encoded);
    return;
  }
  assert(fs.readFileSync(evidencePath, "utf8") === encoded,
    "semantic-fixture evidence has generated drift");
}

function pack(file) {
  return json(path.join(packRoot, file));
}

function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
