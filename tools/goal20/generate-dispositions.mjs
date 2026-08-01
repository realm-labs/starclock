#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const check = process.argv.includes("--check");
assert(process.argv.slice(2).every((value) => value === "--check"),
  "usage: generate-dispositions.mjs [--check]");
const policyPath = "policy/goal20-runtime-dispositions.json";
const policy = json(policyPath);
assert(policy.schema_revision === "starclock.goal20-runtime-disposition-policy.v1",
  "unsupported Goal 20 disposition policy");
assert(policy.goal_id === "swarm-disaster-runtime-v1", "Goal 20 identity drift");
for (const input of Object.values(policy.inputs))
  assert(sha256(input.path) === input.sha256, `disposition input drift: ${input.path}`);

const manifest = json(policy.inputs.source_manifest.path);
const rules = json(policy.inputs.mechanic_rules.path);
const fixtures = json(policy.inputs.semantic_fixtures.path);
const boundaries = json(policy.inputs.policy_boundaries.path);
const categoryNames = Object.keys(manifest.categories);
assert(equal(new Set(categoryNames), new Set(Object.keys(policy.source_category_targets))),
  "source-category disposition policy is not exact");

const sourceObligations = [];
for (const categoryName of categoryNames) {
  const category = manifest.categories[categoryName];
  const target = policy.source_category_targets[categoryName];
  assert(category.count === category.records.length, `${categoryName}: count drift`);
  for (const record of category.records) {
    sourceObligations.push({
      id: `${categoryName}:${record.id}`,
      category: categoryName,
      source_id: record.id,
      ownership: record.ownership,
      source_evidence_sha256: record.evidence_sha256,
      target_runtime_disposition: sourceDisposition(categoryName, record.ownership),
      catalog_batch: target.catalog_batch,
      execution_batch: target.execution_batch,
      current_state: "Pending",
    });
  }
}
sourceObligations.sort(byId);
assert(unique(sourceObligations) === policy.expected.source_obligations,
  "source obligation exact-once assignment drift");

const fixtureById = new Map(fixtures.map((fixture) => [fixture.id, fixture]));
assert(fixtureById.size === policy.expected.semantic_fixture_families,
  "fixture ID denominator drift");
const familyToPartition = new Map();
for (const [batch, partition] of Object.entries(policy.rule_partitions)) {
  for (const family of partition.families) {
    assert(!familyToPartition.has(family), `duplicate rule family ${family}`);
    familyToPartition.set(family, { batch, executor: partition.executor });
  }
}

const ruleAssignments = rules.map((rule) => {
  const partition = familyToPartition.get(rule.family_id);
  assert(partition !== undefined, `unassigned rule family ${rule.family_id}`);
  assert(rule.execution_disposition === "ReferenceOnly",
    `${rule.id}: Goal 09 rule is no longer reference-only`);
  assert(rule.runtime_handler_id === "", `${rule.id}: Goal 09 admitted a handler`);
  for (const fixtureId of rule.fixture_ids)
    assert(fixtureById.has(fixtureId), `${rule.id}: unknown fixture ${fixtureId}`);
  return {
    id: rule.id,
    family_id: rule.family_id,
    owner_id: rule.id.replace(".mechanic-rule.", ".owner."),
    ownership: rule.ownership,
    evidence_quality: rule.evidence_quality,
    target_executor: partition.executor,
    target_accuracy: rule.evidence_quality === "ProjectPolicy"
      ? "VersionedProjectPolicy"
      : "ExactStructured",
    implementation_batch: partition.batch,
    fixture_ids: [...rule.fixture_ids].sort(compare),
    native_handler_id: null,
    current_state: "Pending",
  };
}).sort(byId);
assert(unique(ruleAssignments) === policy.expected.mechanic_rules,
  "mechanic-rule exact-once assignment drift");
assert(equal(new Set(rules.map(({ family_id: family }) => family)),
  new Set(policy.fixture_family_owners ? Object.keys(policy.fixture_family_owners) : [])),
"fixture owner policy does not cover every mechanic family");

const fixtureAssignments = fixtures.map((fixture) => ({
  id: fixture.id,
  family_id: fixture.family_id,
  ownership: fixture.ownership,
  evidence_quality: fixture.fixture_evidence_quality,
  source_record_ids: [...fixture.source_record_ids].sort(compare),
  ordered_operation_count: fixture.ordered_operations.length,
  expected_fact_count: fixture.expected_facts.length,
  execution_batch: "G20-P5-B1",
  implementation_owner_batch: policy.fixture_family_owners[fixture.family_id],
  target_runtime_disposition: "ProductionSemanticFixture",
  current_state: "Pending",
})).sort(byId);
assert(unique(fixtureAssignments) === policy.expected.semantic_fixture_families,
  "semantic-fixture exact-once assignment drift");

const boundaryAssignments = boundaries.map((boundary) => {
  assert(boundary.state === "PolicyBound" && boundary.blocking === false,
    `${boundary.id}: boundary is not a nonblocking inherited policy`);
  const fixtureIds = [...boundary.affected_fixture_ids].sort(compare);
  const implementationBatches = [...new Set(fixtureIds.map((fixtureId) => {
    const fixture = fixtureById.get(fixtureId);
    assert(fixture !== undefined, `${boundary.id}: unknown fixture ${fixtureId}`);
    const owner = policy.fixture_family_owners[fixture.family_id];
    assert(owner !== undefined, `${boundary.id}: unowned family ${fixture.family_id}`);
    return owner;
  }))].sort(compare);
  return {
    id: boundary.id,
    field: boundary.field,
    policy_source_id: boundary.policy_source_id,
    confidence: boundary.confidence,
    selected_policy: boundary.selected_policy,
    replacement_condition: boundary.replacement_condition,
    affected_fixture_ids: fixtureIds,
    affected_record_count: boundary.affected_records.length,
    affected_records_sha256: digest(encode(boundary.affected_records)),
    implementation_batches: implementationBatches,
    target_accuracy: "VersionedProjectPolicy",
    current_state: "InheritedPolicy",
  };
}).sort(byId);
assert(unique(boundaryAssignments) === policy.expected.policy_boundaries,
  "policy-boundary exact-once assignment drift");

const partitions = Object.entries(policy.rule_partitions).map(([batch, partition], ordinal) => {
  const familySet = new Set(partition.families);
  const assigned = ruleAssignments.filter(({ family_id: family }) => familySet.has(family));
  assert(assigned.length === partition.families.length,
    `${batch}: one-rule-per-family denominator drift`);
  return {
    id: batch,
    ordinal,
    family_ids: [...partition.families],
    expected_rules: assigned.length,
    rule_ids: assigned.map(({ id }) => id).sort(compare),
    executor: partition.executor,
    exact_structured_count: assigned.filter(({ target_accuracy: accuracy }) =>
      accuracy === "ExactStructured").length,
    project_policy_count: assigned.filter(({ target_accuracy: accuracy }) =>
      accuracy === "VersionedProjectPolicy").length,
    fixture_ids: [...new Set(assigned.flatMap(({ fixture_ids: ids }) => ids))].sort(compare),
    dependencies: ordinal === 0
      ? ["G20-P4-B5"]
      : [Object.keys(policy.rule_partitions)[ordinal - 1]],
  };
});
assert(partitions.length === policy.expected.rule_partitions,
  "rule partition count drift");
assert(new Set(partitions.flatMap(({ rule_ids: ids }) => ids)).size
  === policy.expected.mechanic_rules, "partition assignment is not exact-once");

const ownershipSummary = countBy(sourceObligations, "ownership");
assert(ownershipSummary.SwarmDisaster === policy.expected.swarm_owned
  && ownershipSummary.Shared === policy.expected.shared,
"source ownership denominator drift");
const dispositions = {
  schema_revision: "starclock.swarm-disaster-runtime-dispositions.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  generated_on: "2026-08-01",
  policy_sha256: sha256(policyPath),
  inputs: Object.fromEntries(Object.entries(policy.inputs)
    .map(([name, input]) => [name, input.sha256])),
  summary: {
    source_obligations: sourceObligations.length,
    source_ownership: ownershipSummary,
    source_target_dispositions: countBy(sourceObligations, "target_runtime_disposition"),
    mechanic_rules: ruleAssignments.length,
    rule_target_executors: countBy(ruleAssignments, "target_executor"),
    rule_target_accuracy: countBy(ruleAssignments, "target_accuracy"),
    semantic_fixture_families: fixtureAssignments.length,
    fixture_evidence_quality: countBy(fixtureAssignments, "evidence_quality"),
    policy_boundaries: boundaryAssignments.length,
    policy_current_state: countBy(boundaryAssignments, "current_state"),
    rule_partitions: partitions.length,
    native_handlers_admitted: 0,
  },
  source_obligations: sourceObligations,
  mechanic_rules: ruleAssignments,
  semantic_fixtures: fixtureAssignments,
  policy_boundaries: boundaryAssignments,
};
const partitionManifest = {
  schema_revision: "starclock.swarm-disaster-rule-partitions.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  generated_on: "2026-08-01",
  policy_sha256: dispositions.policy_sha256,
  mechanic_rules_sha256: policy.inputs.mechanic_rules.sha256,
  summary: {
    partitions: partitions.length,
    rules: partitions.reduce((sum, partition) => sum + partition.expected_rules, 0),
    native_handlers_admitted: 0,
  },
  partitions,
};

const dispositionPath =
  "content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json";
const partitionPath =
  "content-manifests/swarm-disaster-runtime-v1/rule-partitions.json";
const dispositionText = encode(dispositions);
const partitionText = encode(partitionManifest);
const evidence = {
  schema_revision: "starclock.swarm-disaster-disposition-summary.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  generated_on: "2026-08-01",
  result: "Pass",
  source_obligations: sourceObligations.length,
  source_ownership: ownershipSummary,
  source_target_dispositions: dispositions.summary.source_target_dispositions,
  mechanic_rules: ruleAssignments.length,
  rule_target_accuracy: dispositions.summary.rule_target_accuracy,
  semantic_fixture_families: fixtureAssignments.length,
  policy_boundaries: boundaryAssignments.length,
  inherited_policy_boundaries: boundaryAssignments.length,
  rule_partitions: partitions.map(({ id, family_ids: families, expected_rules: count }) => ({
    id, families, count,
  })),
  exact_once_gaps: 0,
  exact_once_duplicates: 0,
  native_handlers_admitted: 0,
  runtime_dispositions_sha256: digest(dispositionText),
  rule_partitions_sha256: digest(partitionText),
  policy_sha256: dispositions.policy_sha256,
};
const evidencePath =
  "evidence/swarm-disaster-runtime-v1/foundation/disposition-summary.json";
writeOrCheck(dispositionPath, dispositionText);
writeOrCheck(partitionPath, partitionText);
writeOrCheck(evidencePath, encode(evidence));
console.log(
  `Goal 20 dispositions ${check ? "verified" : "generated"} ` +
  `(${sourceObligations.length} obligations; ${ruleAssignments.length} rules; ` +
  `${fixtureAssignments.length} fixtures; ${boundaryAssignments.length} policies; ` +
  `${partitions.length} partitions).`,
);

function sourceDisposition(category, ownership) {
  if (category === "adventure_outcomes") return "ExternalOutcome";
  if (category === "semantic_fixture_families") return "Metadata";
  return ownership === "Shared" ? "SharedIntegrated" : "Integrated";
}
function unique(entries) {
  return new Set(entries.map(({ id }) => id)).size;
}
function countBy(entries, field) {
  const counts = {};
  for (const entry of entries) counts[entry[field]] = (counts[entry[field]] ?? 0) + 1;
  return Object.fromEntries(Object.entries(counts).sort(([left], [right]) =>
    compare(left, right)));
}
function byId(left, right) {
  return compare(left.id, right.id);
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
function equal(left, right) {
  return left.size === right.size && [...left].every((value) => right.has(value));
}
function writeOrCheck(relative, value) {
  const file = path.join(root, relative);
  if (check) {
    assert(fs.statSync(file, { throwIfNoEntry: false })?.isFile(),
      `${relative} is missing; run without --check`);
    assert(fs.readFileSync(file, "utf8") === value, `${relative} has generated drift`);
    return;
  }
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, value);
}
function encode(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}
function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function sha256(relative) {
  return digest(fs.readFileSync(path.join(root, relative)));
}
function digest(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
