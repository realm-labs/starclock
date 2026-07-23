import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-activity-agent-api.json");
assert(policy.schema_revision === "starclock.goal04-activity-agent-api.v1", "unexpected Activity agent API policy revision");
const facade = text("crates/starclock-agent-api/src/lib.rs");
const action = text("crates/starclock-agent-api/src/activity_action.rs");
const observation = text("crates/starclock-agent-api/src/activity_observation.rs");
const session = text("crates/starclock-agent-api/src/activity_session.rs");
const reference = text("crates/starclock-agent-api/src/activity_reference.rs");
const replay = text("crates/starclock-mode-universe/src/universe_replay.rs");
const tests = text("crates/starclock-agent-api/tests/activity_session_loop.rs");

for (const marker of ["pub mod activity_action", "pub mod activity_observation", "pub mod activity_session"])
  assert(facade.includes(marker), `agent facade omits ${marker}`);
for (const marker of [
  `MAX_OFFERED_ACTIVITY_ACTIONS: usize = ${policy.limits.offered_actions}`,
  "starclock-agent-activity-action-v1",
  "state_hash.bytes()",
  "InvalidActionToken"
]) assert(action.includes(marker), `Activity action boundary omits ${marker}`);
for (const marker of [
  `ACTIVITY_AGENT_INTERFACE_REVISION: &str = "${policy.interface_revision}"`,
  "AgentActivityObservation",
  "ActivityObservationContext",
  "view.pending_battle().is_some()"
]) assert(observation.includes(marker), `Activity observation boundary omits ${marker}`);
for (const marker of [
  `ACTIVITY_AGENT_CONTROLLER_REVISION: &str = "${policy.controller_revision}"`,
  `MAX_ACTIVITY_ACTIONS_PER_SETTLEMENT: usize = ${policy.limits.accepted_actions_per_settlement}`,
  "PlayActivityActionRequest",
  "idempotency",
  "settle_automatic_battles",
  "encode_standard_universe_trace",
  "verify_standard_universe_replay_with_controller"
]) assert(session.includes(marker), `Activity session boundary omits ${marker}`);
assert(reference.includes(policy.battle_executor), "reference encounter executor identity is missing");
assert(replay.includes("expected_controller_revision"), "Universe replay cannot truthfully bind non-baseline controllers");
for (const marker of [
  "activity_session_exposes_only_tokens_settles_battles_and_round_trips_replay",
  policy.golden.replay_sha256,
  policy.golden.final_state_hash,
  `assert_eq!(external_steps, ${policy.golden.external_actions})`,
  `assert_eq!(replay.bytes().len(), ${policy.golden.encoded_bytes.toLocaleString("en-US").replaceAll(",", "_")})`
]) assert(tests.includes(marker), `Activity agent golden omits ${marker}`);

execFileSync("cargo", ["test", "-p", "starclock-agent-api", "--lib"], { cwd: root, stdio: "inherit" });
execFileSync("cargo", ["test", "-p", "starclock-agent-api", "--test", "standard_session_loop"], { cwd: root, stdio: "inherit" });
execFileSync("cargo", ["test", "-p", "starclock-agent-api", "--test", "activity_session_loop"], { cwd: root, stdio: "inherit" });

const sources = [
  "crates/starclock-agent-api/Cargo.toml",
  "crates/starclock-agent-api/src/lib.rs",
  "crates/starclock-agent-api/src/activity_action.rs",
  "crates/starclock-agent-api/src/activity_observation.rs",
  "crates/starclock-agent-api/src/activity_reference.rs",
  "crates/starclock-agent-api/src/activity_session.rs",
  "crates/starclock-agent-api/tests/activity_session_loop.rs",
  "crates/starclock-mode-universe/src/universe_replay.rs",
  "tools/workspace/verify-dependencies.mjs"
];
const evidence = {
  schema_revision: "starclock.goal04-activity-agent-api-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "owned-opaque-token-activity-session-settles-nested-battles-and-exports-a-freshly-verifiable-replay",
  revisions: { interface: policy.interface_revision, controller: policy.controller_revision },
  limits: policy.limits,
  golden: policy.golden,
  contracts: policy.contracts,
  source_sha256: Object.fromEntries(sources.map((relative) => [relative, sha256(relative)])),
  policy_sha256: sha256("policy/goal04-activity-agent-api.json"),
  new_registry_packages: []
};
const relativeEvidence = "evidence/standard-universe-runtime-v1/interfaces/activity-agent-api.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relativeEvidence)), { recursive: true });
  fs.writeFileSync(path.join(root, relativeEvidence), output);
} else {
  assert(fs.existsSync(path.join(root, relativeEvidence)), "Activity agent API evidence is missing; run with --bless");
  assert(text(relativeEvidence).replaceAll("\r\n", "\n") === output, "Activity agent API evidence is stale; run with --bless");
}
console.log(`Goal 04 Activity agent API verified (${policy.golden.external_actions} external actions, ${policy.golden.nested_battles} battles, ${policy.golden.encoded_bytes} bytes).`);

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
