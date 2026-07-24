#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const check = process.argv.includes("--check");
assert(process.argv.slice(2).every((value) => value === "--check"),
  "usage: audit-retained.mjs [--check]");
const policyPath = "policy/goal07-retained-audit.json";
const policy = json(policyPath);
assert(policy.schema_revision === "starclock.goal07-retained-audit-policy.v1",
  "unsupported Goal 07 audit policy");
for (const input of Object.values(policy.inputs))
  assert(sha256(input.path) === input.sha256, `audit input drift: ${input.path}`);

const runtime = json(policy.inputs.runtime_dispositions.path);
const goal05 = json(policy.inputs.goal05_dispositions.path);
const goal05Records = new Map(goal05.records.map((row) => [row.id, row]));
const goal05Rules = new Map(goal05.rules.map((row) => [row.id, row]));
const goal05Fixtures = new Map(goal05.fixtures.map((row) => [row.id, row]));
const referenceRoot = "content-reference/standard-universe-v1";
const referenceFiles = new Map();
for (const row of runtime.records) {
  if (!referenceFiles.has(row.source_file))
    referenceFiles.set(row.source_file, byId(json(`${referenceRoot}/${row.source_file}`)));
}
const ruleReferences = byId(json(`${referenceRoot}/mechanic-rules.json`));
const fixtureReferences = byId(json(`${referenceRoot}/review-fixtures.json`));

const records = runtime.records.map((row) => {
  const inherited = required(goal05Records, row.id, "Goal 05 record");
  const reference = required(required(referenceFiles, row.source_file, "reference file"),
    row.id, "reference record");
  const targetRuntime = recordRuntime(row, reference);
  return {
    kind: "content-record",
    id: row.id,
    milestone: goal07Milestone(row.partition),
    mechanic_family: required(policy.milestone_families, row.partition, "milestone family"),
    source_category: row.source_category,
    inherited_state: inherited.integration_state,
    prior_typed_disposition: row.disposition,
    shared_primitive_requirement: row.target,
    native_review_candidate: row.disposition === "StaticNativeHandler",
    intended_runtime_disposition: targetRuntime,
    native_fallback_if_ir_insufficient:
      row.disposition === "StaticNativeHandler" ? "ExecutableNative" : null,
    intended_accuracy_disposition:
      targetRuntime === "Metadata" ? "NotApplicable" : "ExactPublic",
    evidence_state: evidenceState(reference),
    evidence_gaps: evidenceGaps("record", inherited.integration_state, row, reference),
    linked_rule_ids: [...row.linked_rule_ids].sort(),
    linked_fixture_ids: [...row.linked_fixture_ids].sort(),
  };
}).sort(byStableId);

const rules = runtime.rules.map((row) => {
  const inherited = required(goal05Rules, row.id, "Goal 05 rule");
  const reference = required(ruleReferences, row.id, "reference rule");
  return {
    kind: "mechanic-rule",
    id: row.id,
    source_record_id: row.source_record_id,
    milestone: goal07Milestone(row.partition),
    mechanic_family: required(policy.milestone_families, row.partition, "milestone family"),
    rule_kind: row.rule_kind,
    inherited_state: inherited.integration_state,
    prior_typed_disposition: row.disposition,
    shared_primitive_requirement: `${row.target}:${row.rule_kind}`,
    native_review_candidate: row.disposition === "StaticNativeHandler",
    intended_runtime_disposition: "ExecutableRuleIr",
    native_fallback_if_ir_insufficient:
      row.disposition === "StaticNativeHandler" ? "ExecutableNative" : null,
    intended_accuracy_disposition: "ExactPublic",
    evidence_state: evidenceState(reference),
    evidence_gaps: evidenceGaps("rule", inherited.integration_state, row, reference),
  };
}).sort(byStableId);

const fixtures = runtime.fixtures.map((row) => {
  const inherited = required(goal05Fixtures, row.id, "Goal 05 fixture");
  const reference = required(fixtureReferences, row.id, "reference fixture");
  return {
    kind: "semantic-fixture",
    id: row.id,
    milestone: goal07Milestone(row.partition),
    mechanic_family: row.mechanic_family,
    inherited_state: inherited.integration_state,
    shared_primitive_requirement: `production-harness:${row.harness}`,
    native_review_candidate: false,
    intended_runtime_disposition: "Metadata",
    intended_accuracy_disposition: "NotApplicable",
    evidence_state: evidenceState(reference),
    evidence_gaps: ["MissingProductionExecution"],
    production_exit: "ProductionExecuted",
    input_ids: [...row.input_ids].sort(),
  };
}).sort(byStableId);

const encounterMembers = tableRows(policy.inputs.encounter_members.path)
  .map((values) => ({
    kind: "encounter-member",
    id: `universe.encounter-member.${integer(values.id)}`,
    milestone: "G07-P5-M15",
    mechanic_family: policy.milestone_families["G04-P4-M15"],
    inherited_state: "ExecutableProjection",
    shared_primitive_requirement: "encounter-member-wave-phase-carry",
    native_review_candidate: false,
    intended_runtime_disposition: "ExecutableShared",
    intended_accuracy_disposition: "ExactPublic",
    evidence_state: "StructuredPublicEvidence",
    evidence_gaps: ["MissingMechanismCorrectEnemyAndCarryFixture"],
    source_stage_id: string(values.source_stage_id),
    source_rogue_monster_id: string(values.source_rogue_monster_id),
  }))
  .sort(byStableId);

const enemyKeys = new Set();
for (const input of [
  policy.inputs.encounter_wave_enemies.path,
  policy.inputs.difficulty_enemies.path,
])
  for (const values of tableRows(input))
    enemyKeys.add(string(values.enemy_variant_stable_key));
const coreEnemyKeys = new Set(tableRows(policy.inputs.core_enemy_variants.path)
  .map((values) => string(values.mechanically_distinct_key)));
const enemyReference = byId(json("content-reference/v4.4/enemy-variants.json"));
const enemyVariants = [...enemyKeys].sort().map((id) => {
  const exact = coreEnemyKeys.has(id);
  const reference = required(enemyReference, id, "enemy reference");
  return {
    kind: "enemy-variant",
    id,
    milestone: "G07-P5-M15",
    mechanic_family: policy.milestone_families["G04-P4-M15"],
    inherited_state: exact ? "ExactDefinition" : "ApproximateProxy",
    shared_primitive_requirement: "enemy-definition-ai-skills-phases-summons",
    native_review_candidate: false,
    intended_runtime_disposition: "ExecutableShared",
    intended_accuracy_disposition:
      exact ? "ExactPublic" : "ApprovedNumericApproximation",
    mechanism_target: "ExactPublic",
    evidence_state: reference.quality === "ExactStructured"
      ? "StructuredPublicEvidence"
      : "PublicEvidence",
    evidence_gaps: exact
      ? ["MissingGoal07ProductionEnemyFixture"]
      : ["MissingMechanismCorrectDefinitionAndAi", "MissingNamedNumericApproximationPolicy"],
    source_monster_id: reference.source_monster_id,
  };
});

assert(records.length === policy.denominators.records, "record denominator drift");
assert(rules.length === policy.denominators.rules, "rule denominator drift");
assert(fixtures.length === policy.denominators.fixtures, "fixture denominator drift");
assert(enemyVariants.length === policy.denominators.enemy_variants,
  "enemy variant denominator drift");
assert(encounterMembers.length === policy.denominators.encounter_members,
  "encounter member denominator drift");
assert(records.filter(({ inherited_state: state }) => state === "RetainedApproximation").length
  === policy.denominators.retained_records, "retained record denominator drift");
assert(rules.filter(({ inherited_state: state }) => state === "RetainedApproximation").length
  === policy.denominators.retained_rules, "retained rule denominator drift");
assert(enemyVariants.filter(({ inherited_state: state }) => state === "ApproximateProxy").length
  === policy.denominators.approximate_enemy_proxies, "enemy proxy denominator drift");

const report = {
  schema_revision: "starclock.goal07-retained-audit.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  generated_on: "2026-07-25",
  source_sha256: Object.fromEntries(Object.values(policy.inputs)
    .map(({ path: inputPath, sha256: digest }) => [inputPath, digest])),
  policy_sha256: sha256(policyPath),
  summary: {
    records: summary(records),
    rules: summary(rules),
    fixtures: summary(fixtures),
    enemy_variants: summary(enemyVariants),
    encounter_members: summary(encounterMembers),
    native_review_candidate_entries: [...records, ...rules]
      .filter(({ native_review_candidate: candidate }) => candidate).length,
    native_review_candidate_rules: rules
      .filter(({ native_review_candidate: candidate }) => candidate).length,
    project_policy_evidence_gaps: [...records, ...rules, ...fixtures]
      .filter(({ evidence_gaps: gaps }) =>
        gaps.includes("ProjectPolicyMechanismRequiresReplacementOrJustification")).length,
  },
  records,
  rules,
  fixtures,
  enemy_variants: enemyVariants,
  encounter_members: encounterMembers,
};
const manifestRelative =
  "content-manifests/standard-universe-mechanics-complete-v1/retained-audit.json";
const manifestText = encode(report);
const evidence = {
  schema_revision: "starclock.goal07-retained-audit-summary.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "complete",
  audited: {
    records: records.length,
    rules: rules.length,
    fixtures: fixtures.length,
    enemy_variants: enemyVariants.length,
    encounter_members: encounterMembers.length,
  },
  inherited_debt: {
    retained_records: policy.denominators.retained_records,
    retained_rules: policy.denominators.retained_rules,
    approximate_enemy_proxies: policy.denominators.approximate_enemy_proxies,
  },
  target_runtime_dispositions: countBy(
    [...records, ...rules, ...fixtures, ...enemyVariants, ...encounterMembers],
    "intended_runtime_disposition",
  ),
  target_accuracy_dispositions: countBy(
    [...records, ...rules, ...fixtures, ...enemyVariants, ...encounterMembers],
    "intended_accuracy_disposition",
  ),
  native_review_candidate_entries: report.summary.native_review_candidate_entries,
  native_review_candidate_rules: report.summary.native_review_candidate_rules,
  project_policy_evidence_gaps: report.summary.project_policy_evidence_gaps,
  audit_sha256: digest(manifestText),
  policy_sha256: report.policy_sha256,
};
const evidenceRelative =
  "evidence/standard-universe-mechanics-complete-v1/phase0/retained-audit-summary.json";
writeOrCheck(manifestRelative, manifestText);
writeOrCheck(evidenceRelative, encode(evidence));
console.log(
  `Goal 07 retained audit ${check ? "verified" : "generated"} ` +
  `(${records.length} records, ${rules.length} rules, ${fixtures.length} fixtures, ` +
  `${enemyVariants.length} enemies, ${encounterMembers.length} members; ` +
  `${report.summary.native_review_candidate_rules} rule candidates require native review).`,
);

function recordRuntime(row, reference) {
  if (row.disposition === "DataOnlyMetadata") return "Metadata";
  if (row.disposition === "ExplicitPolicy") return "SelectionPolicy";
  if (row.source_category === "occurrence-choices"
    && reference.mechanism_quality === "ProjectPolicy")
    return "ExternalDecision";
  return row.linked_rule_ids.length > 0 ? "ExecutableRuleIr" : "ExecutableShared";
}
function evidenceState(reference) {
  if (reference.mechanism_quality === "ProjectPolicy")
    return "ProjectPolicyRequiresReplacementOrJustification";
  if (reference.quality === "ExactStructured") return "StructuredPublicEvidence";
  if (reference.quality === "ExactPublicText") return "PublicTextEvidence";
  return "EvidenceReviewRequired";
}
function evidenceGaps(kind, inheritedState, row, reference) {
  const gaps = [];
  if (kind === "rule" && inheritedState !== "Integrated")
    gaps.push(row.disposition === "StaticNativeHandler"
      ? "MissingExecutableLoweringAndNativeAdmissionReview"
      : "TypedPlanNotProductionBehavior");
  if (kind === "record" && inheritedState === "RetainedApproximation")
    gaps.push(row.disposition === "StaticNativeHandler"
      ? "MissingExecutableLoweringAndNativeAdmissionReview"
      : "TypedPlanNotProductionBehavior");
  if (inheritedState === "Policy") gaps.push("MissingDeterministicSelectionProof");
  if (reference.mechanism_quality === "ProjectPolicy")
    gaps.push("ProjectPolicyMechanismRequiresReplacementOrJustification");
  return gaps.length === 0 ? ["MissingGoal07ExactOnceProductionDisposition"] : gaps;
}
function goal07Milestone(partition) {
  return partition.replace("G04-P4-", "G07-P2-")
    .replace("G07-P2-M11", "G07-P3-M11")
    .replace("G07-P2-M12", "G07-P3-M12")
    .replace("G07-P2-M13", "G07-P4-M13")
    .replace("G07-P2-M14", "G07-P4-M14")
    .replace("G07-P2-M15", "G07-P5-M15");
}
function summary(entries) {
  return {
    total: entries.length,
    inherited_states: countBy(entries, "inherited_state"),
    runtime_targets: countBy(entries, "intended_runtime_disposition"),
    accuracy_targets: countBy(entries, "intended_accuracy_disposition"),
    milestones: countBy(entries, "milestone"),
  };
}
function countBy(entries, field) {
  const counts = {};
  for (const entry of entries)
    counts[entry[field]] = (counts[entry[field]] ?? 0) + 1;
  return Object.fromEntries(Object.entries(counts).sort(([left], [right]) =>
    left.localeCompare(right)));
}
function tableRows(relative) {
  return json(relative).table.rows.map(({ values }) => values);
}
function integer(value) {
  assert(value && Number.isInteger(value.Integer), "expected encoded integer");
  return value.Integer;
}
function string(value) {
  assert(value && typeof value.String === "string", "expected encoded string");
  return value.String;
}
function byId(entries) {
  return new Map(entries.map((entry) => [entry.id, entry]));
}
function required(collection, key, label) {
  const value = collection instanceof Map ? collection.get(key) : collection[key];
  assert(value !== undefined, `${label} is missing ${key}`);
  return value;
}
function byStableId(left, right) {
  return left.id.localeCompare(right.id);
}
function writeOrCheck(relative, value) {
  const file = path.join(root, relative);
  if (check) {
    assert(fs.statSync(file, { throwIfNoEntry: false })?.isFile(),
      `${relative} is missing; run without --check`);
    assert(fs.readFileSync(file, "utf8") === value,
      `${relative} has generated drift`);
  } else {
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, value);
  }
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
