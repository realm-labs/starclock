import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const check = process.argv.includes("--check");
const auditPath =
  "content-manifests/standard-universe-mechanics-complete-v1/retained-audit.json";
const partitionPath =
  "content-manifests/standard-universe-mechanics-complete-v1/content-partitions.json";
const policyPath = "policy/goal07-evidence-and-approximation.json";
const outputPath =
  "content-manifests/standard-universe-mechanics-complete-v1/evidence-and-approximation-register.json";
const summaryPath =
  "evidence/standard-universe-mechanics-complete-v1/phase0/baseline-summary.json";

const audit = json(auditPath);
const partitions = json(partitionPath);
const policy = json(policyPath);
const ownerById = new Map();
for (const partition of partitions.partitions) {
  for (const field of [
    "record_ids",
    "rule_ids",
    "fixture_ids",
    "enemy_variant_ids",
    "encounter_member_ids",
  ]) {
    for (const id of partition[field]) {
      assert(!ownerById.has(id), `duplicate partition owner for ${id}`);
      ownerById.set(id, partition.id);
    }
  }
}

const projectDecisions = audit.records
  .filter((row) =>
    row.evidence_gaps.includes(
      "ProjectPolicyMechanismRequiresReplacementOrJustification",
    ))
  .map((row) => ({
    id: row.id,
    milestone: row.milestone,
    partition: requiredOwner(row.id),
    resolution: policy.project_policy_resolution.resolution,
    terminal_runtime_disposition:
      policy.project_policy_resolution.terminal_runtime_disposition,
    required_contract:
      "explicit ordered legal outcomes plus validated replay-recorded result command",
    status: "RegisteredForPartitionImplementation",
  }))
  .sort(byId);

const numericApproximations = audit.enemy_variants
  .filter((row) =>
    row.intended_accuracy_disposition === "ApprovedNumericApproximation")
  .map((row) => ({
    id: row.id,
    source_monster_id: row.source_monster_id,
    milestone: row.milestone,
    partition: requiredOwner(row.id),
    policy_id: policy.enemy_numeric_policy.id,
    mechanism_target: row.mechanism_target,
    numeric_status: "PendingPerVariantInputs",
    mechanic_status: "PendingExactMechanicImplementation",
    required_evidence_fields:
      policy.enemy_numeric_policy.required_per_variant_evidence,
  }))
  .sort(byId);

const gapCounts = {};
for (const family of [
  "records",
  "rules",
  "fixtures",
  "enemy_variants",
  "encounter_members",
]) {
  gapCounts[family] = {};
  for (const row of audit[family]) {
    for (const gap of row.evidence_gaps) {
      gapCounts[family][gap] = (gapCounts[family][gap] ?? 0) + 1;
    }
  }
}

assert(
  projectDecisions.length === policy.project_policy_resolution.expected_records,
  "project-policy record denominator differs",
);
assert(
  numericApproximations.length === policy.enemy_numeric_policy.expected_variants,
  "numeric-approximation denominator differs",
);

const register = {
  schema_revision: "starclock.goal07-evidence-and-approximation-register.v1",
  goal_id: "standard-universe-mechanics-complete-v1",
  batch: "G07-P0-B4",
  generated_on: "2026-07-25",
  source_sha256: {
    [auditPath]: sha256(auditPath),
    [partitionPath]: sha256(partitionPath),
    [policyPath]: sha256(policyPath),
  },
  summary: {
    project_policy_records: projectDecisions.length,
    numeric_approximation_candidates: numericApproximations.length,
    insufficient_mechanic_evidence_blocks_partition: true,
    unregistered_public_evidence_gap_categories: 0,
  },
  gap_counts: gapCounts,
  project_policy_records: projectDecisions,
  enemy_numeric_approximations: numericApproximations,
};
const registerText = `${JSON.stringify(register, null, 2)}\n`;
const registerSha = digest(Buffer.from(registerText));
const summary = {
  schema_revision: "starclock.goal07-phase0-baseline-summary.v1",
  goal_id: "standard-universe-mechanics-complete-v1",
  batch: "G07-P0-B4",
  result: "complete",
  batches: {
    fixed: partitions.summary.fixed_batches,
    generated_content: partitions.summary.generated_batches,
    total: partitions.summary.total_batches,
  },
  evidence_and_approximation: {
    external_decision_records: projectDecisions.length,
    numeric_approximation_candidates: numericApproximations.length,
    mechanic_approximation_allowed: false,
  },
  performance_workloads: json("policy/goal07-performance.json").workloads.length,
  dependency_baseline: {
    new_registry_packages: json("policy/goal07-dependency-baseline.json")
      .new_registry_packages,
    cargo_lock_sha256: sha256("Cargo.lock"),
  },
  release_state: json("policy/goal07-release-contract.json").state,
  register_sha256: registerSha,
};
const summaryText = `${JSON.stringify(summary, null, 2)}\n`;

writeOrCheck(outputPath, registerText);
writeOrCheck(summaryPath, summaryText);
console.log(
  `Goal 07 Phase 0 registers ${check ? "verified" : "generated"} `
    + `(${projectDecisions.length} external decisions, `
    + `${numericApproximations.length} numeric candidates).`,
);

function requiredOwner(id) {
  const owner = ownerById.get(id);
  assert(owner, `missing partition owner for ${id}`);
  return owner;
}
function byId(left, right) {
  return left.id.localeCompare(right.id, "en");
}
function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function sha256(relative) {
  return digest(fs.readFileSync(path.join(root, relative)));
}
function digest(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}
function writeOrCheck(relative, expected) {
  const absolute = path.join(root, relative);
  if (check) {
    assert(fs.existsSync(absolute), `missing generated file ${relative}`);
    assert(fs.readFileSync(absolute, "utf8") === expected, `${relative} drifted`);
    return;
  }
  fs.mkdirSync(path.dirname(absolute), { recursive: true });
  fs.writeFileSync(absolute, expected);
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
