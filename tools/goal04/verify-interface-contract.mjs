import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] ?? ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-interface-contract.json");
assert(policy.schema_revision === "starclock.goal04-interface-contract.v1", "unexpected interface policy revision");

const document = text("docs/standard-universe-runtime-interface-contract.md");
for (const value of [
  policy.activity.api_revision,
  ...policy.activity.command_kinds,
  ...policy.activity.boundary_kinds,
  ...policy.public_mode_types,
  ...Object.values(policy.revisions).map(String),
  ...policy.activity_rng_streams,
  ...policy.configuration_components
]) assert(document.includes(value), `interface document omits ${value}`);
for (const marker of ["Activity::apply", "byte relabeling is forbidden", "No public signature names a generated Sora module"])
  assert(document.includes(marker), `interface document omits ${marker}`);

const combat = text("crates/starclock-combat/src/lib.rs");
const replay = text("crates/starclock-replay/src/format.rs");
const rng = text("crates/starclock-combat/src/rng/mod.rs");
assert(combat.includes(`STATE_HASH_REVISION: &str = "${policy.revisions.legacy_state_hash}"`), "legacy state-hash revision differs");
assert(combat.includes(`NUMERIC_POLICY_REVISION: &str = "${policy.revisions.numeric_policy}"`), "numeric revision differs");
assert(rng.includes(`RNG_ALGORITHM_REVISION: &str = "${policy.revisions.combat_rng}"`), "combat RNG revision differs");
assert(replay.includes(`REPLAY_FORMAT_VERSION: u32 = ${policy.revisions.replay_format}`), "replay format differs");
assert(replay.includes(`REPLAY_SCHEMA_VERSION: u32 = ${policy.revisions.legacy_replay_schema}`), "legacy replay schema differs");

const goal = text("docs/goals/04-standard-universe-runtime.md");
for (const marker of ["Activity::apply(ActivityCommand)", "Generated Sora types", "normalized JSON and `.xlsx` are never runtime inputs", "G04-P0-B2"])
  assert(goal.includes(marker), `Goal 04 plan omits ${marker}`);
const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P0-B2` \| `(InProgress|Complete)` \|/m.test(status), "G04-P0-B2 is not active or complete");

const evidence = {
  schema_revision: "starclock.goal04-interface-contract-evidence.v1",
  goal_id: policy.goal_id,
  result: "frozen",
  contract_sha256: sha256("docs/standard-universe-runtime-interface-contract.md"),
  activity: policy.activity,
  public_mode_types: policy.public_mode_types,
  revisions: policy.revisions,
  activity_rng_streams: policy.activity_rng_streams,
  configuration_components: policy.configuration_components,
  compatibility: {
    legacy_battle_and_one_battle_replays_retained: true,
    new_activity_bytes_use_v2_codec: true,
    cross_revision_relabeling_allowed: false,
    generated_rows_in_public_api: false
  }
};
const relative = "evidence/standard-universe-runtime-v1/foundation/interface-contract.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "interface evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "interface evidence is stale; run with --bless");
}
console.log(`Goal 04 interface contract verified (${policy.activity.command_kinds.length} commands, ${policy.activity.decision_kinds.length} decisions, Activity state ${policy.revisions.activity_state_codec}).`);

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
