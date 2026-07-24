import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal05-seeded-matrix.json");
assert(policy.schema_revision === "starclock.goal05-seeded-matrix.v1", "unexpected policy revision");
const source = "crates/starclock-agent-api/examples/g05_real_universe_seed_matrix.rs";
const stdout = execFileSync(
  "cargo",
  ["run", "--quiet", "--release", "-p", "starclock-agent-api", "--example", "g05_real_universe_seed_matrix"],
  { cwd: root, encoding: "utf8", maxBuffer: 32 * 1024 * 1024, stdio: ["ignore", "pipe", "inherit"] }
);
const matrix = JSON.parse(stdout.trim());
assert(matrix.schema_revision === policy.matrix_revision, "matrix revision drift");
assert(matrix.executor_revision === policy.contracts.battle_executor, "executor revision drift");
for (const [field, expected] of [
  ["worlds", policy.coverage.worlds],
  ["difficulties", policy.coverage.difficulties],
  ["distinct_path_options", policy.coverage.path_options],
  ["complete_runs", policy.coverage.complete_runs]
]) assert(matrix.coverage[field] === expected, `${field} coverage drift`);
for (const [field, expected] of [
  ["encounter_members", policy.coverage.encounter_members],
  ["enemy_variants", policy.coverage.enemy_variants],
  ["exact_enemy_definitions", policy.coverage.exact_enemy_definitions],
  ["approximate_enemy_proxies", policy.coverage.approximate_enemy_proxies],
  ["catalog_mechanic_rule_rows", policy.coverage.catalog_mechanic_rule_rows],
  ["initial_declared_rule_bindings", policy.coverage.initial_declared_rule_bindings]
]) assert(matrix.battle_assembly[field] === expected, `${field} assembly coverage drift`);
assert(matrix.coverage.first_seed === policy.first_seed, "first seed drift");
assert(matrix.battle_assembly.initial_materialized_rule_bindings === 0, "initial snapshot granted unowned rules");
assert(matrix.runs.length === policy.coverage.complete_runs, "run denominator drift");

const worlds = new Set();
const difficulties = new Set();
const paths = new Set();
const seeds = new Set();
let externalActions = 0;
let replayActions = 0;
let externalOutcomes = 0;
let nestedBattles = 0;
let battleCommands = 0;
let battleStates = 0;
for (const [index, run] of matrix.runs.entries()) {
  assert(run.ordinal === index, `run ${index} ordinal drift`);
  assert(run.seed === policy.first_seed + index, `run ${index} seed drift`);
  assert(run.terminal === "completed", `run ${index} did not complete`);
  assert(run.external_actions > 0 && run.replay_actions >= run.external_actions, `run ${index} action counts are invalid`);
  assert(run.nested_battles >= 0 && run.encoded_bytes > 0, `run ${index} lacks replay evidence`);
  assert(run.replay_components === 9, `run ${index} component denominator drift`);
  assert(run.battle_commands === run.battle_state_records, `run ${index} battle state stream is incomplete`);
  assert(run.nested_battles === 0 || run.battle_commands > 0, `run ${index} fabricated a battle projection`);
  assert(/^[0-9a-f]{64}$/.test(run.final_state_hash), `run ${index} final hash is invalid`);
  assert(/^[0-9a-f]{64}$/.test(run.replay_sha256), `run ${index} replay hash is invalid`);
  assert(run.decision_kinds.length > 0 && run.action_kinds.length > 0, `run ${index} lacks decision diagnostics`);
  worlds.add(run.world);
  difficulties.add(`${run.world}/${run.difficulty_index}`);
  paths.add(run.path_option_id);
  seeds.add(run.seed);
  externalActions += run.external_actions;
  replayActions += run.replay_actions;
  externalOutcomes += run.external_outcome_actions;
  nestedBattles += run.nested_battles;
  battleCommands += run.battle_commands;
  battleStates += run.battle_state_records;
}
assert(worlds.size === policy.coverage.worlds, "World coverage is incomplete");
assert(difficulties.size === policy.coverage.difficulties, "difficulty coverage is incomplete");
assert(paths.size === policy.coverage.path_options, "Path coverage is incomplete");
assert(seeds.size === policy.coverage.complete_runs, "seeds are not unique");
assert(nestedBattles > 0 && battleCommands > 0, "matrix executed no real nested combat");
assert(externalOutcomes > 0, "matrix traversed no atomic external outcomes");
for (const [field, actual] of [
  ["external_actions", externalActions],
  ["replay_actions", replayActions],
  ["external_outcome_actions", externalOutcomes],
  ["nested_battles", nestedBattles],
  ["battle_commands", battleCommands],
  ["battle_state_records", battleStates]
]) assert(matrix.coverage[field] === actual, `${field} aggregate drift`);
assert(matrix.failures.length === policy.coverage.failure_cases, "failure denominator drift");
assert(equal(matrix.failures.map((entry) => entry.case), policy.failure_cases), "failure order drift");
for (const failure of matrix.failures)
  assert(failure.code === "invalid_request" && failure.committed === false, `${failure.case} was not inert`);

const evidence = {
  schema_revision: "starclock.goal05-seeded-matrix-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: matrix.result,
  matrix,
  contracts: policy.contracts,
  source_sha256: { [source]: sha256(source) },
  policy_sha256: sha256("policy/goal05-seeded-matrix.json"),
  new_registry_packages: []
};
const relativeEvidence = "evidence/standard-universe-end-to-end-v1/coverage/seeded-matrix.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relativeEvidence)), { recursive: true });
  fs.writeFileSync(path.join(root, relativeEvidence), output);
} else {
  assert(fs.existsSync(path.join(root, relativeEvidence)), "matrix evidence is missing; run with --bless");
  assert(text(relativeEvidence).replaceAll("\r\n", "\n") === output, "matrix evidence is stale; run with --bless");
}
console.log(
  `Goal 05 real matrix verified (${worlds.size} Worlds, ${matrix.runs.length} runs, ` +
  `${nestedBattles} battles, ${battleCommands} combat commands, ${externalOutcomes} external outcomes).`
);

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex");
}
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
