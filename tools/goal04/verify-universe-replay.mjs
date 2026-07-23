import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const artifactOnly = process.env.STARCLOCK_ARTIFACT_CHECK_ONLY === "1";
const policy = json("policy/goal04-universe-replay.json");
assert(policy.schema_revision === "starclock.goal04-universe-replay.v1", "unexpected Universe replay policy revision");
const replay = text("crates/starclock-mode-universe/src/universe_replay.rs");
const runner = text("crates/starclock-mode-universe/src/baseline_runner.rs");
const payload = text("crates/starclock-replay/src/activity.rs");
const tests = text("crates/starclock-mode-universe/tests/encounter_runtime.rs");

for (const marker of [
  `STANDARD_UNIVERSE_REPLAY_ACTION_VERSION: u16 = ${policy.action_payload_version}`,
  `MAX_STANDARD_UNIVERSE_REPLAY_ACTIONS: u32 = ${policy.limits.max_actions.toLocaleString("en-US").replaceAll(",", "_")}`,
  "record_baseline_run",
  "encode_standard_universe_trace",
  "verify_standard_universe_replay",
  "NestedStartDivergence",
  "NestedEndDivergence",
  "StateDivergence",
  "DecisionDivergence"
]) assert(replay.includes(marker), `Universe replay omits ${marker}`);
for (const marker of [
  "encode_battle_result_payload",
  "decode_battle_result_payload",
  "encode_controller_diagnostic_payload",
  "decode_controller_diagnostic_payload",
  "encode_nested_battle_start_payload",
  "decode_nested_battle_end_payload"
]) assert(payload.includes(marker), `shared replay payload boundary omits ${marker}`);
assert(runner.includes(policy.controller_revision), "baseline runner revision differs");
for (const marker of [
  "complete_run_replay_verifies_and_reports_the_first_divergence",
  policy.golden.replay_sha256,
  `verified.action_count(), ${policy.golden.actions}`,
  `verified.diagnostic_count(), ${policy.golden.controller_diagnostics}`,
  `verified.nested_battle_count(), ${policy.golden.nested_battles}`
]) assert(tests.includes(marker), `Universe replay golden test omits ${marker}`);

if (!artifactOnly) {
  execFileSync("cargo", ["test", "-p", "starclock-replay", "--lib"], { cwd: root, stdio: "inherit" });
  execFileSync("cargo", ["test", "-p", "starclock-mode-universe", "--test", "encounter_runtime", "complete_run_replay_verifies_and_reports_the_first_divergence"], { cwd: root, stdio: "inherit" });
}

const sources = [
  "crates/starclock-replay/src/activity.rs",
  "crates/starclock-mode-universe/src/baseline_controller.rs",
  "crates/starclock-mode-universe/src/baseline_runner.rs",
  "crates/starclock-mode-universe/src/universe_replay.rs",
  "crates/starclock-mode-universe/tests/encounter_runtime.rs"
];
const evidence = {
  schema_revision: "starclock.goal04-universe-replay-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "complete-baseline-run-round-trips-with-controller-and-nested-battle-first-divergence-evidence",
  action_payload_version: policy.action_payload_version,
  limits: policy.limits,
  golden: policy.golden,
  contracts: policy.contracts,
  source_sha256: Object.fromEntries(sources.map((relative) => [relative, sha256(relative)])),
  policy_sha256: sha256("policy/goal04-universe-replay.json"),
  new_registry_packages: []
};
const relativeEvidence = "evidence/standard-universe-runtime-v1/interfaces/universe-replay.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relativeEvidence)), { recursive: true });
  fs.writeFileSync(path.join(root, relativeEvidence), output);
} else {
  assert(fs.existsSync(path.join(root, relativeEvidence)), "Universe replay evidence is missing; run with --bless");
  assert(text(relativeEvidence).replaceAll("\r\n", "\n") === output, "Universe replay evidence is stale; run with --bless");
}
console.log(`Goal 04 Universe replay verified (${policy.golden.actions} actions, ${policy.golden.nested_battles} battles, ${policy.golden.encoded_bytes} bytes).`);

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
