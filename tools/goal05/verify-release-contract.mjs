import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const hasRoot = Boolean(process.argv[2] && !process.argv[2].startsWith("--"));
const root = path.resolve(hasRoot ? process.argv[2] : ".");
const args = process.argv.slice(hasRoot ? 3 : 2);
const release = args.includes("--release");
const bless = args.includes("--bless");
const requireClean = args.includes("--require-clean");
const artifactOnly = process.env.STARCLOCK_ARTIFACT_CHECK_ONLY === "1";
assert(args.every((arg) => ["--release", "--bless", "--require-clean"].includes(arg)),
  "unknown Goal 05 release verifier argument");
assert(release, "Goal 05 verifier requires --release");
assert(!bless || release, "--bless is release-only");
assert(!requireClean || release, "--require-clean is release-only");

const policy = json("policy/goal05-release-contract.json");
assert(policy.schema_revision === "starclock.goal05-release-contract.v1",
  "unsupported Goal 05 release contract");
assert(policy.goal_id === "standard-universe-end-to-end-v1", "Goal 05 ID differs");
assert(policy.state === "Released", "Goal 05 release contract is not Released");

const status = text("docs/goals/05-standard-universe-end-to-end-status.md");
const plan = text("docs/goals/05-standard-universe-end-to-end.md");
const batchIds = [...status.matchAll(/^\| `(G05-P[0-5]-B\d+)` \|/gmu)].map((match) => match[1]);
const completeBatchIds = [...status.matchAll(
  /^\| `(G05-P[0-5]-B\d+)` \| `Complete` \|/gmu,
)].map((match) => match[1]);
assert(batchIds.length === policy.planned_batches, "Goal 05 batch denominator differs");
assert(new Set(batchIds).size === batchIds.length, "Goal 05 batch ledger contains duplicates");
assert(completeBatchIds.length === policy.planned_batches, "not every Goal 05 batch is Complete");
assert(new Set(batchIds.map((id) => id.slice(4, 6))).size === policy.planned_phases,
  "Goal 05 phase denominator differs");
assert(status.includes("| Final state | Complete |"), "Goal 05 terminal state is not Complete");
assert(!plan.includes("- [ ]"), "Goal 05 terminal checklist is incomplete");

for (const file of [
  "docs/goals/05-standard-universe-end-to-end.md",
  "docs/goals/05-standard-universe-end-to-end-status.md",
  "docs/goals/05-standard-universe-end-to-end-prompt.md",
  "docs/27-standard-universe-end-to-end-integration.md",
  "docs/28-standard-universe-integration-coverage.md",
  "policy/goal05-seeded-matrix.json",
  "policy/goal05-integration-coverage.json",
  "policy/goal05-hardening.json",
]) assert(fileExists(file), `Goal 05 release input is missing ${file}`);

for (const collection of [
  policy.evidence_files,
  policy.manifest_files,
  policy.documentation_files,
]) {
  assert(Array.isArray(collection) && new Set(collection).size === collection.length,
    "Goal 05 release inventory is absent or contains duplicates");
  for (const file of collection) assert(fileExists(file), `Goal 05 release reference is missing ${file}`);
}

const snapshots = json("policy/release-snapshots.json");
const snapshotIds = new Set(snapshots.goals.map((goal) => goal.goal_id));
for (const goalId of policy.required_prior_contracts)
  assert(snapshotIds.has(goalId), `required prior release snapshot is missing ${goalId}`);

const dispositionsPath =
  "content-manifests/standard-universe-end-to-end-v1/integration-dispositions.json";
const matrixPath = "evidence/standard-universe-end-to-end-v1/coverage/seeded-matrix.json";
const hardeningPath = "evidence/standard-universe-end-to-end-v1/hardening/native-hardening.json";
const dispositions = json(dispositionsPath);
const matrix = json(matrixPath);
const hardening = json(hardeningPath);
const expectedStates = new Set(["Integrated", "Metadata", "Policy", "RetainedApproximation"]);

assert(dispositions.records.length === policy.terminal_denominators.content_records,
  "Goal 05 content denominator differs");
assert(dispositions.rules.length === policy.terminal_denominators.rule_bindings,
  "Goal 05 rule denominator differs");
assert(dispositions.fixtures.length === policy.terminal_denominators.semantic_fixtures,
  "Goal 05 fixture denominator differs");
for (const entry of [...dispositions.records, ...dispositions.rules, ...dispositions.fixtures])
  assert(expectedStates.has(entry.integration_state), `unsupported integration state ${entry.integration_state}`);

const recordStates = counts(dispositions.records);
const ruleStates = counts(dispositions.rules);
const fixtureStates = counts(dispositions.fixtures);
assert(recordStates.Integrated === policy.terminal_denominators.integrated_records,
  "integrated content denominator differs");
assert(recordStates.Policy === policy.terminal_denominators.policy_records,
  "policy content denominator differs");
assert(recordStates.RetainedApproximation === policy.terminal_denominators.approximate_records,
  "approximate content denominator differs");
assert(ruleStates.Integrated === policy.terminal_denominators.integrated_rules,
  "integrated rule denominator differs");
assert(ruleStates.RetainedApproximation === policy.terminal_denominators.approximate_rules,
  "approximate rule denominator differs");
assert(fixtureStates.Metadata === policy.terminal_denominators.semantic_fixtures,
  "fixture metadata denominator differs");
assert(JSON.stringify(dispositions.summary.records) === JSON.stringify(recordStates),
  "record disposition summary is stale");
assert(JSON.stringify(dispositions.summary.rules) === JSON.stringify(ruleStates),
  "rule disposition summary is stale");
assert(JSON.stringify(dispositions.summary.fixtures) === JSON.stringify(fixtureStates),
  "fixture disposition summary is stale");

const coverage = matrix.matrix.coverage;
const assembly = matrix.matrix.battle_assembly;
for (const [field, policyField] of [
  ["worlds", "worlds"],
  ["difficulties", "difficulties"],
  ["distinct_path_options", "paths"],
  ["complete_runs", "complete_seeded_runs"],
  ["nested_battles", "real_nested_battles"],
  ["battle_commands", "real_battle_commands"],
  ["external_outcome_actions", "atomic_external_outcomes"],
]) assert(coverage[field] === policy.terminal_denominators[policyField],
  `seeded matrix ${field} denominator differs`);
for (const [field, policyField] of [
  ["encounter_members", "encounter_members"],
  ["enemy_variants", "enemy_variants"],
  ["exact_enemy_definitions", "exact_enemy_definitions"],
  ["approximate_enemy_proxies", "approximate_enemy_proxies"],
]) assert(assembly[field] === policy.terminal_denominators[policyField],
  `battle assembly ${field} denominator differs`);
assert(coverage.battle_commands === coverage.battle_state_records,
  "not every accepted battle command has a state record");
assert(matrix.matrix.runs.length === policy.terminal_denominators.complete_seeded_runs,
  "seeded matrix row denominator differs");
assert(matrix.matrix.runs.every((run) => run.terminal === "completed"),
  "seeded matrix contains an incomplete run");
assert(assembly.initial_declared_rule_bindings === 0
  && assembly.initial_materialized_rule_bindings === 0,
  "release fixture unexpectedly grants unowned mechanics");

assert(hardening.local_execution.elapsed_ms
  <= json("policy/goal05-hardening.json").wall_budget_seconds * 1_000,
  "Goal 05 hardening exceeds its wall budget");
for (const [field, expected] of Object.entries(json("policy/goal05-hardening.json").corpora))
  assert(hardening.corpora[field] === expected, `hardening corpus differs for ${field}`);
assert(hardening.native_profiles.length === 3, "native execution profile denominator differs");
assert(hardening.compile_only_profiles.length === 3, "compile-only profile denominator differs");
assert(hardening.compile_only_profiles.every((profile) => profile.runtime_claims === 0),
  "compile-only profile makes a runtime claim");
assert(hardening.contracts.committed_local_report_is_not_hosted_ci_proof === true,
  "local hardening evidence is mislabeled as hosted proof");

if (!artifactOnly) {
  run("node", ["tools/goal05/verify-integration-probes.mjs"]);
  run("node", ["tools/goal05/verify-seeded-matrix.mjs"]);
  run("node", ["tools/goal05/verify-integration-coverage.mjs"]);
  run("node", ["tools/goal05/run-native-hardening.mjs"]);
  run("node", ["tools/ci/verify-workflow.mjs"]);
}

const report = {
  schema_revision: "starclock.goal05-release-evidence.v1",
  goal_id: policy.goal_id,
  released_on: policy.released_on,
  result: "complete",
  policy_sha256: sha256("policy/goal05-release-contract.json"),
  completion: {
    phases: policy.planned_phases,
    batches: policy.planned_batches,
    release_batch: policy.release_batch,
  },
  runtime_revisions: policy.runtime_revisions,
  integration_coverage: {
    content_records: dispositions.records.length,
    rule_bindings: dispositions.rules.length,
    semantic_fixtures: dispositions.fixtures.length,
    record_states: recordStates,
    rule_states: ruleStates,
    fixture_states: fixtureStates,
    disposition_sha256: sha256(dispositionsPath),
  },
  real_execution: {
    worlds: coverage.worlds,
    paths: coverage.distinct_path_options,
    difficulties: coverage.difficulties,
    complete_runs: coverage.complete_runs,
    nested_battles: coverage.nested_battles,
    battle_commands: coverage.battle_commands,
    atomic_external_outcomes: coverage.external_outcome_actions,
    encounter_members: assembly.encounter_members,
    exact_enemy_definitions: assembly.exact_enemy_definitions,
    approximate_enemy_proxies: assembly.approximate_enemy_proxies,
    runtime_stat_policy: assembly.runtime_stat_policy,
    seeded_matrix_sha256: sha256(matrixPath),
  },
  hardening: {
    local_elapsed_ms: hardening.local_execution.elapsed_ms,
    wall_budget_seconds: json("policy/goal05-hardening.json").wall_budget_seconds,
    required_native_profiles: hardening.native_profiles.map((profile) => profile.id),
    compile_only_profiles: hardening.compile_only_profiles.map((profile) => profile.id),
    hosted_native_proof: "retained-ci-artifact-only",
    hardening_sha256: sha256(hardeningPath),
  },
  evidence_sha256: hashes(policy.evidence_files),
  manifest_sha256: hashes(policy.manifest_files),
  documentation_sha256: hashes(policy.documentation_files),
  prior_contracts: policy.required_prior_contracts,
  retained_boundaries: {
    dynamic_acquired_inventory_battle_rematerialization: "not-implemented",
    retained_approximate_content_records: recordStates.RetainedApproximation,
    retained_approximate_rules: ruleStates.RetainedApproximation,
    all_current_enemy_runtime_stats: "approximate",
  },
  clean_checkout_command: policy.clean_checkout_command,
};
const output = `${JSON.stringify(report, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, policy.release_evidence)), { recursive: true });
  fs.writeFileSync(path.join(root, policy.release_evidence), output);
} else {
  assert(fileExists(policy.release_evidence), "Goal 05 release evidence is missing; run with --bless");
  assert(text(policy.release_evidence).replaceAll("\r\n", "\n") === output,
    "Goal 05 release evidence is stale; run with --bless");
}
if (requireClean)
  assert(capture("git", ["status", "--porcelain"]) === "", "Goal 05 worktree is not clean");
console.log(`Goal 05 release verified (${policy.planned_batches} batches${requireClean ? ", clean" : ""}).`);

function counts(entries) {
  const result = {
    Integrated: 0,
    Metadata: 0,
    Policy: 0,
    RetainedApproximation: 0,
  };
  for (const entry of entries) result[entry.integration_state] += 1;
  return result;
}
function hashes(files) {
  return Object.fromEntries(files.map((file) => [file, sha256(file)]));
}
function run(command, commandArgs) {
  execFileSync(command, commandArgs, { cwd: root, stdio: "inherit" });
}
function capture(command, commandArgs) {
  return execFileSync(command, commandArgs, { cwd: root, encoding: "utf8" }).trim();
}
function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function fileExists(relative) {
  return fs.statSync(path.join(root, relative), { throwIfNoEntry: false })?.isFile();
}
function sha256(relative) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex");
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
