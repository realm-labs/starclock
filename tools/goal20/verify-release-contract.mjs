#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const hasRoot = Boolean(process.argv[2] && !process.argv[2].startsWith("--"));
const root = path.resolve(hasRoot ? process.argv[2] : ".");
const options = process.argv.slice(hasRoot ? 3 : 2);
assert(options.every((option) => ["--release", "--bless", "--require-clean"].includes(option)),
  "usage: verify-release-contract.mjs [root] --release [--bless] [--require-clean]");
assert(options.includes("--release"), "Goal 20 verifier requires --release");
const bless = options.includes("--bless");
const requireClean = options.includes("--require-clean");
const policyPath = "policy/goal20-release-contract.json";
const evidencePath = "evidence/swarm-disaster-runtime-v1/release/release-evidence.json";
const policy = currentJson(policyPath);
const snapshots = currentJson("policy/release-snapshots.json");
const snapshot = snapshots.goals.find(({ goal_id: goalId }) => goalId === policy.goal_id);
const completionCommit = snapshot?.completion_commit;

assert(policy.schema_revision === "starclock.goal20-release-contract.v1" &&
  policy.goal_id === "swarm-disaster-runtime-v1" && policy.state === "Released",
"Goal 20 release identity drift");
assert(policy.planned_phases === 9 && policy.planned_fixed_batches === 39 &&
  policy.planned_mechanic_partitions === 12 && policy.planned_total_batches === 51 &&
  policy.release_batch === "G20-P8-B4", "Goal 20 phase or batch denominator drift");
assert(Object.values(policy.contracts).every((value) => value === true),
  "Goal 20 release contracts must all be enabled");
for (const collection of [policy.policy_files, policy.evidence_files, policy.documentation_files]) {
  assert(Array.isArray(collection) && collection.length > 0 &&
    new Set(collection).size === collection.length, "Goal 20 release inventory is invalid");
  for (const relative of collection)
    assert(sourceExists(relative), `Goal 20 release input is missing ${relative}`);
}

if (snapshot) {
  assert(capture("git", ["cat-file", "-t", completionCommit]) === "commit",
    "Goal 20 completion commit is unavailable");
  assert(capture("git", ["show", "-s", "--format=%T", completionCommit]) === snapshot.completion_tree,
    "Goal 20 completion tree differs from its snapshot");
  assert(snapshot.status_path === "docs/goals/20-swarm-disaster-runtime-status.md" &&
    snapshot.release_policy_path === policyPath && snapshot.release_evidence_path === evidencePath &&
    snapshot.evidence_schema_revision === "starclock.goal20-release-evidence.v1",
  "Goal 20 snapshot paths drift");
}

const status = sourceText("docs/goals/20-swarm-disaster-runtime-status.md");
const batchIds = [...status.matchAll(/^\| `(G20-P\d+-(?:B|M)\d+)` \|/gmu)].map((match) => match[1]);
const completed = [...status.matchAll(
  /^\| `(G20-P\d+-(?:B|M)\d+)` \| `Complete` \|/gmu,
)].map((match) => match[1]);
assert(batchIds.length === 51 && new Set(batchIds).size === 51 && completed.length === 51,
  "Goal 20 batch ledger is incomplete");
assert(status.includes("| State | `ReleasePendingSnapshot` |") &&
  status.includes("| Active batch | None |") &&
  status.includes("| Next unblocked batch | None |") &&
  status.includes("| `G20-P8-B4` | `Complete` |") &&
  (status.match(/^- \[ \]/gmu) ?? []).length === 1 &&
  status.includes("- [ ] Register the P8-B4 completion snapshot after hosted native CI succeeds."),
"Goal 20 status or terminal checklist is incomplete");

const terminal = policy.terminal_denominators;
const completeness = sourceJson(policy.evidence_files[0]);
const matrix = sourceJson(policy.evidence_files[1]);
const replay = sourceJson(policy.evidence_files[2]);
const baseline = sourceJson(policy.evidence_files[3]);
const cli = sourceJson(policy.evidence_files[4]);
const agent = sourceJson(policy.evidence_files[5]);
const mcp = sourceJson(policy.evidence_files[6]);
const hardening = sourceJson(policy.evidence_files[7]);
const performance = sourceJson(policy.evidence_files[8]);
const audits = sourceJson(policy.evidence_files[9]);
const generated = sourceJson("policy/generated-drift.json");

assert(completeness.result === "Pass" &&
  completeness.production_runtime.source_obligations === terminal.source_obligations &&
  completeness.production_runtime.mechanic_rules === terminal.mechanic_rules &&
  completeness.production_runtime.semantic_fixtures === terminal.semantic_fixture_families &&
  completeness.production_runtime.native_handlers === terminal.native_handlers &&
  completeness.production_runtime.runtime_json_file_reads === 0,
"Goal 20 completeness denominator drift");
assert(completeness.source_exact_once.gaps === 0 && completeness.source_exact_once.duplicates === 0 &&
  completeness.rule_exact_once.gaps === 0 && completeness.rule_exact_once.duplicates === 0 &&
  completeness.rule_exact_once.orphan_rules === 0 && completeness.fixture_exact_once.gaps === 0 &&
  completeness.fixture_exact_once.duplicates === 0, "Goal 20 exact-once release drift");
assert(matrix.result === "Pass" && matrix.execution.completed_runs === terminal.seeded_complete_runs &&
  matrix.execution.fresh_factory_verified_runs === terminal.seeded_complete_runs &&
  matrix.execution.total_executor_invocations === terminal.nested_battle_executions &&
  matrix.execution.runtime_json_reads === 0, "Goal 20 seeded matrix drift");
assert(replay.result === "Pass" &&
  replay.representative_complete_run.activity_actions === terminal.representative_activity_actions &&
  replay.representative_complete_run.accepted_battle_commands === terminal.representative_battle_commands &&
  replay.representative_complete_run.replay_bytes === terminal.representative_replay_bytes &&
  replay.representative_complete_run.real_nested_battles === 12,
"Goal 20 representative replay drift");
assert(baseline.result === "Pass" && baseline.contract.exact_offered_commands_only === true &&
  cli.result === "Pass" && cli.representative_run.fresh_verification_passed === true &&
  agent.result === "Pass" && agent.representative_session.external_actions === terminal.agent_external_actions &&
  mcp.result === "Pass" && mcp.representative_transport_session.external_actions === terminal.agent_external_actions,
"Goal 20 controller or external-surface parity drift");
assert(hardening.result === "cross-platform-native-contract-and-local-macos-arm64-vectors-frozen" &&
  hardening.corpora.rng_domains === terminal.rng_domains &&
  hardening.corpora.invalid_actions === terminal.invalid_actions &&
  hardening.corpora.malformed_replay_cases === terminal.malformed_replays,
"Goal 20 hardening denominator drift");
assert(performance.result === "seven-frozen-workloads-with-stable-runner-and-broad-ci-budgets" &&
  performance.samples.length === terminal.stable_runner_samples &&
  performance.medians.length === terminal.performance_workloads &&
  performance.samples.every((sample) => sample.rows.length === terminal.performance_workloads),
"Goal 20 performance evidence drift");
assert(audits.result.includes("audits-pass") &&
  audits.dependency_license.reviewed_registry_packages === terminal.reviewed_registry_packages &&
  audits.architecture_native_source.admitted_native_handlers === terminal.native_handlers &&
  audits.completeness.policy_boundaries === terminal.policy_boundaries &&
  audits.candidate.sora_tables === terminal.sora_tables && audits.candidate.workbook_rows === terminal.workbook_rows &&
  generated.checks.length === terminal.generated_drift_checks_at_release &&
  generated.checks.filter((check) => check.requires === "source-cache").length === terminal.source_cache_checks,
"Goal 20 release-audit evidence drift");

const snapshotIds = new Set(snapshots.goals.map(({ goal_id: goalId }) => goalId));
for (const goalId of policy.required_prior_contracts)
  assert(snapshotIds.has(goalId), `Goal 20 prior release snapshot is missing ${goalId}`);

const report = {
  schema_revision: "starclock.goal20-release-evidence.v1",
  goal_id: policy.goal_id,
  released_on: policy.released_on,
  result: "complete",
  completion_commit: "this file's containing G20-P8-B4 commit",
  completion: {
    phases: policy.planned_phases,
    fixed_batches: policy.planned_fixed_batches,
    mechanic_partitions: policy.planned_mechanic_partitions,
    total_batches: policy.planned_total_batches,
    release_batch: policy.release_batch,
  },
  coverage: {
    source_obligations: terminal.source_obligations,
    mechanic_rules: terminal.mechanic_rules,
    semantic_fixture_families: terminal.semantic_fixture_families,
    policy_boundaries: terminal.policy_boundaries,
    sora_tables: terminal.sora_tables,
    workbook_rows: terminal.workbook_rows,
    exact_once_gaps: 0,
    exact_once_duplicates: 0,
    orphan_rules: 0,
    native_handlers: terminal.native_handlers,
    runtime_json_file_reads: 0,
  },
  production_matrix: {
    complete_runs: matrix.execution.completed_runs,
    fresh_factory_verified_runs: matrix.execution.fresh_factory_verified_runs,
    primary_nested_battles: matrix.execution.primary_nested_battles,
    fresh_verification_nested_battles: matrix.execution.fresh_verification_nested_battles,
    total_executor_invocations: matrix.execution.total_executor_invocations,
    difficulties: matrix.frozen_matrix.difficulties,
    paths: matrix.frozen_matrix.paths,
    audience_dice: matrix.frozen_matrix.audience_dice,
    reachable_faces: matrix.frozen_matrix.reachable_faces,
    countdown_disarray_boundaries: matrix.frozen_matrix.countdown_disarray_boundaries,
    policy_probes: matrix.frozen_matrix.policy_probes,
  },
  representative_replay: replay.representative_complete_run,
  surfaces: {
    baseline_controller_revision: baseline.contract.controller_revision,
    cli_revision: cli.contract.cli_revision,
    agent_revision: agent.contract.interface_revision,
    mcp_revision: mcp.contract.mcp_revision,
    external_actions: agent.representative_session.external_actions,
    final_state_hash: agent.representative_session.final_state_hash,
    replay_sha256: agent.representative_session.replay_sha256,
  },
  hardening: {
    rng_domains: hardening.corpora.rng_domains,
    invalid_actions: hardening.corpora.invalid_actions,
    malformed_replays: hardening.corpora.malformed_replay_cases,
    activity_command_sha256: hardening.goldens.activity_command_sha256,
    battle_command_sha256: hardening.goldens.battle_command_sha256,
    battle_event_state_sha256: hardening.goldens.battle_event_state_sha256,
    activity_state_sha256: hardening.goldens.activity_state_sha256,
    rng_domain_sha256: hardening.goldens.rng_domain_sha256,
  },
  performance: {
    workloads: performance.medians.length,
    stable_runner_samples: performance.samples.length,
    warm_assembly_hits: terminal.warm_assembly_hits,
    concurrent_sessions: terminal.concurrent_sessions,
  },
  audits: {
    reviewed_registry_packages: audits.dependency_license.reviewed_registry_packages,
    protected_reference_roots: audits.protected_reference_roots.length,
    generated_drift_checks: generated.checks.length,
    source_cache_checks: terminal.source_cache_checks,
    clean_checkout_command: audits.clean_checkout_command,
  },
  native: {
    required_native_profiles: policy.native_profiles,
    compile_only_profiles: policy.compile_only_profiles,
    compile_only_runtime_claims: 0,
    hosted_evidence_path: policy.hosted_native_evidence,
    registration: "separate snapshot commit after successful hosted CI",
  },
  policy_files_sha256: sourceHashes(policy.policy_files),
  evidence_files_sha256: sourceHashes(policy.evidence_files),
  documentation_files_sha256: sourceHashes(policy.documentation_files),
  prior_contracts: policy.required_prior_contracts,
  clean_worktree_command: policy.clean_worktree_command,
};
const output = `${JSON.stringify(report, null, 2)}\n`;
if (bless) {
  assert(!snapshot, "Goal 20 release evidence cannot be re-blessed after snapshot registration");
  fs.mkdirSync(path.dirname(absolute(evidencePath)), { recursive: true });
  fs.writeFileSync(absolute(evidencePath), output);
} else {
  assert(currentExists(evidencePath), `${evidencePath} is missing; run with --bless`);
  const existing = snapshot ? sourceText(evidencePath) : currentText(evidencePath);
  assert(existing.replaceAll("\r\n", "\n") === output, "Goal 20 release evidence is stale");
}

if (snapshot) verifyHostedEvidence(snapshot);
if (requireClean) {
  assert(snapshot, "Goal 20 completion snapshot is not registered");
  assert(capture("git", ["status", "--porcelain"]) === "", "Goal 20 worktree is not clean");
}
console.log(`Goal 20 release verified (51 batches${snapshot ? ", snapshot and hosted native evidence" : ", pre-snapshot"}${requireClean ? ", clean" : ""}).`);

function verifyHostedEvidence(releaseSnapshot) {
  const hosted = currentJson(policy.hosted_native_evidence);
  assert(hosted.schema_revision === "starclock.goal20-hosted-native-ci.v1" &&
    hosted.goal_id === policy.goal_id && hosted.result === "pass" &&
    hosted.completion_commit === releaseSnapshot.completion_commit && hosted.run.conclusion === "success",
  "Goal 20 hosted native evidence identity drift");
  const expected = [...policy.native_profiles, ...policy.compile_only_profiles].sort();
  const actual = hosted.profiles.map(({ profile }) => profile).sort();
  assert(JSON.stringify(actual) === JSON.stringify(expected) &&
    hosted.profiles.every((entry) => entry.evidence_origin === "hosted-ci" &&
      entry.commit === releaseSnapshot.completion_commit && /^[0-9a-f]{64}$/u.test(entry.artifact_sha256)),
  "Goal 20 hosted CI profile receipts drift");
  for (const profile of policy.native_profiles)
    assert(hosted.profiles.find((entry) => entry.profile === profile)?.execution_mode === "native",
      `${profile}: hosted native execution receipt is missing`);
  for (const profile of policy.compile_only_profiles)
    assert(hosted.profiles.find((entry) => entry.profile === profile)?.execution_mode === "compile-only",
      `${profile}: hosted compile-only receipt is missing`);
}
function sourceHashes(files) { return Object.fromEntries(files.map((file) => [file, sourceSha256(file)])); }
function sourceSha256(relative) { return digest(Buffer.from(sourceText(relative))); }
function sourceExists(relative) {
  if (!completionCommit) return currentExists(relative);
  try { capture("git", ["cat-file", "-e", `${completionCommit}:${relative}`]); return true; } catch { return false; }
}
function sourceText(relative) {
  return completionCommit ? capture("git", ["show", `${completionCommit}:${relative}`], false) : currentText(relative);
}
function sourceJson(relative) { return JSON.parse(sourceText(relative)); }
function currentJson(relative) { return JSON.parse(currentText(relative)); }
function currentText(relative) { return fs.readFileSync(absolute(relative), "utf8"); }
function currentExists(relative) { return fs.statSync(absolute(relative), { throwIfNoEntry: false })?.isFile(); }
function absolute(relative) { return path.join(root, relative); }
function digest(value) { return crypto.createHash("sha256").update(value).digest("hex"); }
function capture(command, args, trim = true) {
  const value = execFileSync(command, args, { cwd: root, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
  return trim ? value.trim() : value;
}
function assert(condition, message) { if (!condition) throw new Error(message); }
