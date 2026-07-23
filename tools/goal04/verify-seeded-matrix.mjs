import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-seeded-matrix.json");
assert(policy.schema_revision === "starclock.goal04-seeded-matrix.v1", "unexpected seeded-matrix policy revision");
const source = "crates/starclock-agent-api/examples/g04_universe_seed_matrix.rs";
const stdout = execFileSync("cargo", ["run", "--quiet", "--release", "-p", "starclock-agent-api", "--example", "g04_universe_seed_matrix"], {
  cwd: root, encoding: "utf8", maxBuffer: 16 * 1024 * 1024, stdio: ["ignore", "pipe", "inherit"]
});
const matrix = JSON.parse(stdout.trim());
assert(matrix.schema_revision === policy.matrix_revision, "matrix revision drift");
assert(matrix.executor_revision === policy.contracts.battle_executor, "nested executor identity drift");
for (const [field, expected] of [
  ["worlds", policy.coverage.worlds], ["difficulties", policy.coverage.difficulties],
  ["distinct_path_options", policy.coverage.path_options], ["complete_runs", policy.coverage.complete_runs]
]) assert(matrix.coverage[field] === expected, `${field} coverage drift`);
assert(matrix.coverage.first_seed === policy.first_seed, "first seed drift");
assert(matrix.runs.length === policy.coverage.complete_runs, "run denominator drift");
const worlds = new Set();
const difficulties = new Set();
const paths = new Set();
const seeds = new Set();
let nestedBattles = 0;
for (const [index, run] of matrix.runs.entries()) {
  assert(run.ordinal === index, `run ${index} ordinal drift`);
  assert(run.seed === policy.first_seed + index, `run ${index} seed drift`);
  assert(run.terminal === "completed", `run ${index} did not complete`);
  assert(Number(run.external_actions) > 0 && Number(run.replay_actions) >= Number(run.external_actions), `run ${index} action counts are invalid`);
  assert(Number(run.nested_battles) >= 0 && run.encoded_bytes > 0, `run ${index} lacks replay evidence`);
  assert(/^[0-9a-f]{64}$/.test(run.final_state_hash) && /^[0-9a-f]{64}$/.test(run.replay_sha256), `run ${index} hash is invalid`);
  assert(run.decision_kinds.length > 0 && run.action_kinds.length > 0, `run ${index} lacks decision diagnostics`);
  worlds.add(run.world);
  difficulties.add(`${run.world}/${run.difficulty_index}`);
  paths.add(run.path_option_id);
  seeds.add(run.seed);
  nestedBattles += Number(run.nested_battles);
}
assert(worlds.size === policy.coverage.worlds, "World coverage is incomplete");
assert(difficulties.size === policy.coverage.difficulties, "difficulty coverage is incomplete");
assert(paths.size === policy.coverage.path_options, "Path coverage is incomplete");
assert(seeds.size === policy.coverage.complete_runs, "seeds are not unique");
assert(nestedBattles > 0, "the complete matrix executed no nested battles");
assert(matrix.failures.length === policy.coverage.failure_cases, "failure denominator drift");
assert(equal(matrix.failures.map((entry) => entry.case), policy.failure_cases), "failure-case order drift");
for (const failure of matrix.failures)
  assert(failure.code === "invalid_request" && failure.committed === false, `${failure.case} did not fail inertly`);

const evidence = {
  schema_revision: "starclock.goal04-seeded-matrix-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: matrix.result,
  matrix,
  contracts: policy.contracts,
  source_sha256: { [source]: sha256(source) },
  policy_sha256: sha256("policy/goal04-seeded-matrix.json"),
  new_registry_packages: []
};
const relativeEvidence = "evidence/standard-universe-runtime-v1/hardening/seeded-matrix.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relativeEvidence)), { recursive: true });
  fs.writeFileSync(path.join(root, relativeEvidence), output);
} else {
  assert(fs.existsSync(path.join(root, relativeEvidence)), "seeded matrix evidence is missing; run with --bless");
  assert(text(relativeEvidence).replaceAll("\r\n", "\n") === output, "seeded matrix evidence is stale; run with --bless");
}
console.log(`Goal 04 seeded matrix verified (${worlds.size} Worlds, ${paths.size} Paths, ${matrix.runs.length} complete runs).`);

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
