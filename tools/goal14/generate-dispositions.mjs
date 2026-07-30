#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const check = process.argv.includes("--check");
assert(process.argv.slice(2).every((value) => value === "--check"),
  "usage: generate-dispositions.mjs [--check]");

const policyPath = "policy/goal14-runtime-dispositions.json";
const policy = json(policyPath);
assert(policy.schema_revision === "starclock.goal14-runtime-disposition-policy.v1",
  "unsupported Goal 14 disposition policy");
assert(policy.goal_id === "gold-and-gears-runtime-v1", "Goal 14 identity drift");
for (const input of Object.values(policy.inputs))
  assert(sha256(input.path) === input.sha256, `disposition input drift: ${input.path}`);

const manifest = json(policy.inputs.source_manifest.path);
const rules = json(policy.inputs.mechanic_rules.path);
const fixtures = json(policy.inputs.semantic_fixtures.path);
const categoryNames = Object.keys(manifest.categories);
assert(equal(new Set(categoryNames), new Set(Object.keys(policy.source_category_targets))),
  "source-category disposition policy is not exact");

const sourceObligations = [];
for (const categoryName of categoryNames) {
  const category = manifest.categories[categoryName];
  const target = policy.source_category_targets[categoryName];
  assert(category.count === category.records.length,
    `${categoryName}: manifest category count drift`);
  for (const record of category.records) {
    const disposition = sourceDisposition(categoryName, record.ownership);
    sourceObligations.push({
      id: `${categoryName}:${record.id}`,
      category: categoryName,
      source_id: record.id,
      ownership: record.ownership,
      source_evidence_sha256: record.evidence_sha256,
      target_runtime_disposition: disposition,
      catalog_batch: target.catalog_batch,
      execution_batch: target.execution_batch,
      current_state: "Pending",
    });
  }
}
sourceObligations.sort(byId);
assert(unique(sourceObligations) === policy.expected.source_obligations,
  "source obligation exact-once assignment drift");

const fixtureIds = new Set(fixtures.map(({ id }) => id));
const ruleAssignments = rules.map((rule) => {
  const partition = policy.rule_partitions[rule.family_id];
  assert(partition !== undefined, `unassigned rule family ${rule.family_id}`);
  assert(rule.execution_disposition === "ReferenceOnly",
    `${rule.id}: Goal 08 rule is no longer reference-only`);
  assert(rule.runtime_handler_id === "", `${rule.id}: Goal 08 admitted a runtime handler`);
  for (const fixtureId of rule.fixture_ids)
    assert(fixtureIds.has(fixtureId), `${rule.id}: unknown fixture ${fixtureId}`);
  return {
    id: rule.id,
    family_id: rule.family_id,
    owner_id: rule.owner_id,
    ownership: rule.ownership,
    evidence_quality: rule.evidence_quality,
    policy_bound: rule.policy_bound,
    target_executor: rule.ownership === "Shared"
      ? "ReleasedSharedExecutor"
      : partition.gold_executor,
    target_accuracy: rule.policy_bound ? "VersionedProjectPolicy" : "ExactPublic",
    implementation_batch: partition.batch,
    fixture_ids: [...rule.fixture_ids].sort(compare),
    native_handler_id: null,
    current_state: "Pending",
  };
}).sort(byId);
assert(unique(ruleAssignments) === policy.expected.mechanic_rules,
  "mechanic-rule exact-once assignment drift");

const fixtureAssignments = fixtures.map((fixture) => ({
  id: fixture.id,
  family_id: fixture.family_id,
  ownership: fixture.ownership,
  evidence_quality: fixture.fixture_evidence_quality,
  source_record_ids: [...fixture.source_record_ids].sort(compare),
  ordered_operation_count: fixture.ordered_operations.length,
  expected_fact_count: fixture.expected_facts.length,
  execution_batch: "G14-P5-B1",
  target_runtime_disposition: "ProductionSemanticFixture",
  current_state: "Pending",
})).sort(byId);
assert(unique(fixtureAssignments) === policy.expected.semantic_fixture_families,
  "semantic-fixture exact-once assignment drift");

const partitions = Object.entries(policy.rule_partitions)
  .map(([familyId, partition], ordinal) => {
    const assigned = ruleAssignments.filter(({ family_id: family }) => family === familyId);
    assert(assigned.length === partition.expected_rules,
      `${partition.batch}: expected ${partition.expected_rules} rules, got ${assigned.length}`);
    return {
      id: partition.batch,
      ordinal,
      family_id: familyId,
      expected_rules: partition.expected_rules,
      rule_ids: assigned.map(({ id }) => id),
      gold_executor: partition.gold_executor,
      shared_rule_count: assigned.filter(({ ownership }) => ownership === "Shared").length,
      gold_rule_count: assigned.filter(({ ownership }) => ownership === "GoldAndGears").length,
      exact_public_count: assigned.filter(({ target_accuracy }) =>
        target_accuracy === "ExactPublic").length,
      project_policy_count: assigned.filter(({ target_accuracy }) =>
        target_accuracy === "VersionedProjectPolicy").length,
      fixture_ids: [...new Set(assigned.flatMap(({ fixture_ids: ids }) => ids))].sort(compare),
      dependencies: ordinal === 0
        ? ["G14-P4-B5"]
        : [Object.values(policy.rule_partitions)[ordinal - 1].batch],
    };
  });
assert(partitions.length === policy.expected.rule_partitions,
  "rule partition count drift");
assert(new Set(partitions.flatMap(({ rule_ids: ids }) => ids)).size
  === policy.expected.mechanic_rules, "partition rule assignment is not exact-once");

const dispositionSummary = countBy(sourceObligations, "target_runtime_disposition");
const ownershipSummary = countBy(sourceObligations, "ownership");
assert(ownershipSummary.GoldAndGears === policy.expected.gold_owned
  && ownershipSummary.Shared === policy.expected.shared,
"source ownership denominator drift");

const dispositions = {
  schema_revision: "starclock.gold-and-gears-runtime-dispositions.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  generated_on: "2026-07-30",
  policy_sha256: sha256(policyPath),
  inputs: Object.fromEntries(Object.entries(policy.inputs)
    .map(([name, input]) => [name, input.sha256])),
  summary: {
    source_obligations: sourceObligations.length,
    source_ownership: ownershipSummary,
    source_target_dispositions: dispositionSummary,
    mechanic_rules: ruleAssignments.length,
    rule_ownership: countBy(ruleAssignments, "ownership"),
    rule_target_executors: countBy(ruleAssignments, "target_executor"),
    rule_target_accuracy: countBy(ruleAssignments, "target_accuracy"),
    semantic_fixture_families: fixtureAssignments.length,
    fixture_evidence_quality: countBy(fixtureAssignments, "evidence_quality"),
    rule_partitions: partitions.length,
    native_handlers_admitted: 0,
  },
  source_obligations: sourceObligations,
  mechanic_rules: ruleAssignments,
  semantic_fixtures: fixtureAssignments,
};
const partitionManifest = {
  schema_revision: "starclock.gold-and-gears-rule-partitions.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  generated_on: "2026-07-30",
  policy_sha256: dispositions.policy_sha256,
  mechanic_rules_sha256: policy.inputs.mechanic_rules.sha256,
  summary: {
    partitions: partitions.length,
    rules: partitions.reduce((sum, partition) => sum + partition.expected_rules, 0),
    shared_rules: partitions.reduce((sum, partition) => sum + partition.shared_rule_count, 0),
    gold_rules: partitions.reduce((sum, partition) => sum + partition.gold_rule_count, 0),
    native_handlers_admitted: 0,
  },
  partitions,
};

const dispositionPath =
  "content-manifests/gold-and-gears-runtime-v1/runtime-dispositions.json";
const partitionPath =
  "content-manifests/gold-and-gears-runtime-v1/rule-partitions.json";
const dispositionText = encode(dispositions);
const partitionText = encode(partitionManifest);
const evidence = {
  schema_revision: "starclock.gold-and-gears-disposition-summary.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  generated_on: "2026-07-30",
  result: "Pass",
  source_obligations: sourceObligations.length,
  source_ownership: ownershipSummary,
  source_target_dispositions: dispositionSummary,
  mechanic_rules: ruleAssignments.length,
  rule_target_executors: dispositions.summary.rule_target_executors,
  rule_target_accuracy: dispositions.summary.rule_target_accuracy,
  semantic_fixture_families: fixtureAssignments.length,
  rule_partitions: partitions.map(({ id, family_id: family, expected_rules: count }) => ({
    id,
    family,
    count,
  })),
  exact_once_gaps: 0,
  exact_once_duplicates: 0,
  native_handlers_admitted: 0,
  runtime_dispositions_sha256: digest(dispositionText),
  rule_partitions_sha256: digest(partitionText),
  policy_sha256: dispositions.policy_sha256,
};
const evidencePath =
  "evidence/gold-and-gears-runtime-v1/foundation/disposition-summary.json";

writeOrCheck(dispositionPath, dispositionText);
writeOrCheck(partitionPath, partitionText);
writeOrCheck(evidencePath, encode(evidence));
console.log(
  `Goal 14 dispositions ${check ? "verified" : "generated"} ` +
  `(${sourceObligations.length} obligations; ${ruleAssignments.length} rules; ` +
  `${fixtureAssignments.length} fixtures; ${partitions.length} partitions).`,
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
  for (const entry of entries)
    counts[entry[field]] = (counts[entry[field]] ?? 0) + 1;
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
