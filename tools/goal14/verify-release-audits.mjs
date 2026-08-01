#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const hasRoot = Boolean(process.argv[2] && !process.argv[2].startsWith("--"));
const root = path.resolve(hasRoot ? process.argv[2] : ".");
const options = process.argv.slice(hasRoot ? 3 : 2);
assert(options.every((option) => option === "--bless"),
  "usage: verify-release-audits.mjs [root] [--bless]");
const bless = options.includes("--bless");
const artifactOnly = process.env.STARCLOCK_ARTIFACT_CHECK_ONLY === "1";
const policyPath = "policy/goal14-release-audits.json";
const evidencePath = "evidence/gold-and-gears-runtime-v1/audits/release-audits.json";
const policy = json(policyPath);
const foundation = json("policy/goal14-foundation.json");
const completeness = json(
  "evidence/gold-and-gears-runtime-v1/foundation/runtime-completeness.json",
);
const generated = json("policy/generated-drift.json");

assert(policy.schema_revision === "starclock.goal14-release-audits.v1" &&
  policy.goal_id === "gold-and-gears-runtime-v1" && policy.batch === "G14-P8-B3",
"Goal 14 release-audit policy identity drift");
assert(Object.values(policy.contracts).every((value) => value === true),
  "every Goal 14 release-audit contract must be enabled");
if (!artifactOnly) {
  for (const command of policy.required_verifiers)
    execFileSync(command[0], command.slice(1), { cwd: root, stdio: "inherit" });
}

const snapshots = json("policy/release-snapshots.json");
const byGoal = new Map(snapshots.goals.map((entry) => [entry.goal_id, entry]));
assert(equal(policy.required_prior_goals,
  foundation.required_snapshots.map(({ goal_id: goalId }) => goalId)),
"Goal 14 prior-goal inventory differs from its frozen foundation");
const prior = policy.required_prior_goals.map((goalId) => {
  const frozen = foundation.required_snapshots.find(({ goal_id: id }) => id === goalId);
  const current = byGoal.get(goalId);
  assert(current?.completion_commit === frozen.completion_commit &&
    current?.completion_tree === frozen.completion_tree,
  `${goalId}: current completion snapshot differs from Goal 14 foundation`);
  const releaseEvidence = json(current.release_evidence_path);
  assert(releaseEvidence.schema_revision === current.evidence_schema_revision,
    `${goalId}: release evidence schema drift`);
  return {
    goal_id: goalId,
    completion_commit: current.completion_commit,
    completion_tree: current.completion_tree,
    release_evidence_sha256: sha256(current.release_evidence_path),
  };
});

assert(foundation.protected_roots.length === policy.denominators.protected_reference_roots,
  "Goal 14 protected-root denominator drift");
const protectedRoots = foundation.protected_roots.map((entry) => {
  const currentTree = capture("git", ["rev-parse", `HEAD:${entry.path}`]);
  assert(currentTree === entry.tree, `${entry.path}: protected Goal 08 tree drift`);
  return { path: entry.path, tree: currentTree };
});

const expected = policy.denominators;
assert(completeness.result === "Pass" &&
  completeness.production_runtime.source_obligations === expected.source_obligations &&
  completeness.production_runtime.mechanic_rules === expected.mechanic_rules &&
  completeness.production_runtime.semantic_fixtures === expected.semantic_fixture_families &&
  completeness.production_runtime.native_handlers === expected.admitted_native_handlers &&
  completeness.production_runtime.runtime_json_file_reads === 0,
"Goal 14 runtime completeness audit drift");
assert(completeness.source_exact_once.gaps === 0 &&
  completeness.source_exact_once.duplicates === 0 &&
  completeness.rule_exact_once.gaps === 0 && completeness.rule_exact_once.duplicates === 0 &&
  completeness.rule_exact_once.orphan_rules === 0 &&
  completeness.fixture_exact_once.gaps === 0 && completeness.fixture_exact_once.duplicates === 0,
"Goal 14 exact-once release audit has gaps, duplicates or orphans");
const reference = foundation.reference_input;
assert(reference.source_obligations === expected.source_obligations &&
  reference.mechanic_rules === expected.mechanic_rules &&
  reference.semantic_fixture_families === expected.semantic_fixture_families &&
  reference.sora_tables === expected.sora_tables && reference.workbook_rows === expected.workbook_rows,
"Goal 14 frozen source denominator drift");
assert(sha256(reference.candidate_bundle.path) === reference.candidate_bundle.sha256,
  "Goal 14 Candidate bundle drift");

const baselineLock = capture("git", ["show", `${policy.dependency_baseline_commit}:Cargo.lock`]);
const currentLock = fs.readFileSync(absolute("Cargo.lock"), "utf8");
const baselineRegistry = registryPackages(baselineLock);
const currentRegistry = registryPackages(currentLock);
assert(equal(baselineRegistry, currentRegistry) &&
  currentRegistry.length === expected.reviewed_registry_packages,
"Goal 14 introduced or removed a registry package");
for (const review of policy.reviewed_manifest_delta) {
  assert(review.disposition.length >= 100 && exists(review.path),
    `${review.path}: manifest delta review is missing or weak`);
  const before = capture("git", ["show", `${policy.dependency_baseline_commit}:${review.path}`]);
  assert(before !== text(review.path), `${review.path}: reviewed manifest did not change`);
}
const dependencyPolicy = json("policy/dependency-and-tool-policy.json");
const grouped = (dependencyPolicy.package_groups ?? []).flatMap((group) => group.packages);
const licensed = [...dependencyPolicy.packages, ...grouped];
assert(licensed.every((entry) => typeof entry.license === "string" && entry.license.length > 0),
  "dependency license inventory is incomplete");

const sourcePolicy = json("policy/repository-checks.json");
const nativePolicy = json("policy/native-handler-audit.json");
const cacheChecks = generated.checks.filter((check) => check.requires === "source-cache").length;
assert(generated.checks.length === expected.generated_drift_checks &&
  cacheChecks === expected.source_cache_checks,
"Goal 14 generated-drift denominator drift");
assert(nativePolicy.v1a_reviews.length === 8,
  "Goal 14 native-handler scope denominator drift");

const report = {
  schema_revision: "starclock.goal14-release-audits-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "dependency-license-architecture-native-source-provenance-generated-prior-release-audits-pass",
  prior_releases: prior,
  protected_reference_roots: protectedRoots,
  completeness: {
    source_obligations: completeness.production_runtime.source_obligations,
    mechanic_rules: completeness.production_runtime.mechanic_rules,
    semantic_fixture_families: completeness.production_runtime.semantic_fixtures,
    policy_boundaries: expected.policy_boundaries,
    exact_once_gaps: 0,
    exact_once_duplicates: 0,
    orphan_rules: 0,
    runtime_json_file_reads: 0,
  },
  candidate: {
    sora_tables: reference.sora_tables,
    workbook_rows: reference.workbook_rows,
    bundle_sha256: reference.candidate_bundle.sha256,
    normalized_pack_sha256: reference.normalized_pack_sha256,
  },
  dependency_license: {
    baseline_commit: policy.dependency_baseline_commit,
    reviewed_registry_packages: currentRegistry.length,
    new_registry_packages: [],
    reviewed_manifests: policy.reviewed_manifest_delta,
    license_inventory_entries: licensed.length,
    baseline_cargo_lock_sha256: digest(baselineLock),
    current_cargo_lock_sha256: digest(currentLock),
  },
  architecture_native_source: {
    dependency_boundaries_passed: true,
    native_handler_scopes: nativePolicy.v1a_reviews.length,
    admitted_native_handlers: expected.admitted_native_handlers,
    maximum_handwritten_lines: sourcePolicy.rust_source.maximum_handwritten_lines,
    maximum_facade_lines: sourcePolicy.rust_source.maximum_facade_lines,
  },
  generated_drift: {
    checks: generated.checks.length,
    source_cache_checks: cacheChecks,
    cache_independent_checks: generated.checks.length - cacheChecks,
  },
  clean_checkout_command: policy.clean_checkout_command,
  source_cache_command: policy.source_cache_command,
  policy_sha256: sha256(policyPath),
  contracts: policy.contracts,
};
const output = `${JSON.stringify(report, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(absolute(evidencePath)), { recursive: true });
  fs.writeFileSync(absolute(evidencePath), output);
} else {
  assert(exists(evidencePath), `${evidencePath} is missing; run with --bless`);
  assert(text(evidencePath).replaceAll("\r\n", "\n") === output,
    `${evidencePath} is stale; run with --bless`);
}
console.log(`Goal 14 release audits verified (${prior.length} prior releases, ${protectedRoots.length} protected roots, ${currentRegistry.length} registry packages).`);

function registryPackages(lock) {
  return lock.split("[[package]]").slice(1).flatMap((block) => {
    const source = block.match(/^source = "([^"]+)"/mu)?.[1];
    if (!source?.startsWith("registry+")) return [];
    const name = block.match(/^name = "([^"]+)"/mu)?.[1];
    const version = block.match(/^version = "([^"]+)"/mu)?.[1];
    assert(name && version, "Cargo.lock registry package identity malformed");
    return [`${name}@${version}`];
  }).sort();
}
function capture(command, args) {
  return execFileSync(command, args, { cwd: root, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 }).trim();
}
function digest(value) { return crypto.createHash("sha256").update(value).digest("hex"); }
function sha256(relative) { return digest(fs.readFileSync(absolute(relative))); }
function absolute(relative) { return path.join(root, relative); }
function exists(relative) { return fs.statSync(absolute(relative), { throwIfNoEntry: false })?.isFile(); }
function text(relative) { return fs.readFileSync(absolute(relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
