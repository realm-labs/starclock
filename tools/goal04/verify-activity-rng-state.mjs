import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-activity-rng-state.json");
assert(policy.schema_revision === "starclock.goal04-activity-rng-state.v1", "unexpected Activity RNG/state policy revision");
const rng = text("crates/starclock-activity/src/activity_rng.rs");
const codec = text("crates/starclock-activity/src/codec.rs");
const transaction = text("crates/starclock-activity/src/transaction.rs");
const view = text("crates/starclock-activity/src/view.rs");
const tests = text("crates/starclock-activity/tests/activity_rng_state.rs");
const manifest = text("crates/starclock-activity/Cargo.toml");

assert(rng.includes(`ACTIVITY_RNG_REVISION: &str = "${policy.revisions.rng}"`), "Activity RNG revision differs");
assert(codec.includes(`ACTIVITY_STATE_CODEC_REVISION: &str = "${policy.revisions.state_codec}"`), "Activity state codec revision differs");
assert(codec.includes(`ACTIVITY_STATE_HASH_REVISION: &str = "${policy.revisions.state_hash}"`), "Activity state hash revision differs");
assert((rng.match(/^        Self::[A-Za-z]+,$/gm) ?? []).length === policy.bounds.labeled_streams, "Activity RNG label count differs");
assert(rng.includes(`const MAX_REJECTIONS: u32 = ${numberLiteral(policy.bounds.maximum_rejections)};`), "Activity RNG rejection budget differs");
for (const marker of [
  "ChaCha8Rng", "threshold = upper.wrapping_neg() % upper", "purpose == 0",
  "pub fn choose_weighted", "pub fn snapshots", "ActivityRngContext"
]) assert(rng.includes(marker), `Activity RNG omits ${marker}`);
for (const marker of [
  "b\"SCAS\"", "to_le_bytes()", "pub fn canonical_state_bytes", "pub fn state_hash",
  "writer.u64(self.command_sequence)", "writer.u32(self.node_visits.len() as u32)",
  "writer.u32(self.edge_traversals.len() as u32)", "let snapshots = rng.snapshots()"
]) assert((codec + transaction).includes(marker), `canonical Activity state omits ${marker}`);
for (const marker of ["pub struct ActivityPlayerView", "pub struct ActivityDebugView", "pub fn player_view", "pub fn debug_view"])
  assert((transaction + view).includes(marker), `Activity read views omit ${marker}`);
assert(!/derive\([^\n]*Clone[^\n]*\)\]\s*pub struct ActivityRngStreams/.test(rng), "authoritative Activity RNG must not implement Clone");
assert(!/derive\([^\n]*Clone[^\n]*\)\]\s*pub struct ActivityTransactionState/.test(transaction), "authoritative Activity state must not implement Clone");
assert(!/f32|f64|HashMap/.test(rng + codec + transaction + view), "Activity RNG/state uses float or unordered map");
assert(manifest.includes("rand.workspace = true"), "Activity crate does not bind the pinned workspace rand dependency");
assert((tests.match(/^#\[test\]$/gm) ?? []).length === policy.focused_tests, "Activity RNG/state focused test count differs");
assert(tests.includes(policy.goldens.first_graph_raw.replace(/\B(?=(\d{3})+(?!\d))/g, "_")), "Activity RNG golden differs");
const stateGolden = tests.match(/initial\.bytes\(\),\s*\[([\s\S]*?)\]\s*\)/);
assert(stateGolden !== null, "Activity state hash golden assertion is missing");
const stateGoldenHex = stateGolden[1].split(",").map((value) => value.trim()).filter(Boolean).map((value) => Number.parseInt(value, 10).toString(16).padStart(2, "0")).join("");
assert(stateGoldenHex === policy.goldens.initial_state_sha256, "Activity state hash golden differs");
for (const marker of [
  "perturbation_isolated", "canonical_v2_state_bytes_and_hash", "CauseMismatch",
  "player_view_is_visibility_filtered"
]) assert(tests.includes(marker), `Activity RNG/state tests omit ${marker}`);
const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P2-B4` \| `(InProgress|Complete)` \|/m.test(status), "G04-P2-B4 is not active or complete");

const evidence = {
  schema_revision: "starclock.goal04-activity-rng-state-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "labeled-rng-canonical-state-and-bounded-views-implemented",
  revisions: policy.revisions,
  bounds: policy.bounds,
  shape: policy.shape,
  goldens: policy.goldens,
  compatibility: {
    goal01_state_hash_goldens_unchanged: true,
    goal01_manifest_members_preserved: true,
    battle_checkpoint_fields_reserved_until: "G04-P2-B6",
    authoritative_float: false,
    generated_rows_public: false
  },
  source_sha256: {
    rng: sha256("crates/starclock-activity/src/activity_rng.rs"),
    codec: sha256("crates/starclock-activity/src/codec.rs"),
    transaction: sha256("crates/starclock-activity/src/transaction.rs"),
    view: sha256("crates/starclock-activity/src/view.rs"),
    tests: sha256("crates/starclock-activity/tests/activity_rng_state.rs")
  },
  focused_tests: policy.focused_tests,
  new_registry_packages: policy.new_registry_packages
};
const relative = "evidence/standard-universe-runtime-v1/activity/rng-state-codec-view.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "Activity RNG/state evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "Activity RNG/state evidence is stale; run with --bless");
}
console.log(`Goal 04 Activity RNG/state verified (${policy.bounds.labeled_streams} streams, ${policy.focused_tests} tests).`);

function numberLiteral(value) { return String(value).replace(/\B(?=(\d{3})+(?!\d))/g, "_"); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
