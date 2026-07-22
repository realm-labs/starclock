import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-battle-settlement.json");
assert(policy.schema_revision === "starclock.goal04-battle-settlement.v1", "unexpected battle-settlement policy revision");
const source = text("crates/starclock-activity/src/battle_settlement.rs");
const projection = text("crates/starclock-activity/src/projection.rs");
const preparation = text("crates/starclock-activity/src/battle_preparation.rs");
const transaction = text("crates/starclock-activity/src/transaction.rs");
const view = text("crates/starclock-activity/src/view.rs");
const replay = text("crates/starclock-replay/src/activity.rs");
const tests = text("crates/starclock-activity/tests/battle_settlement.rs");

assert(new RegExp(`const MAX_COMPLETED_ACTIVITY_BATTLES: usize = ${numberLiteral(policy.bounds.maximum_completed_battles)};`).test(source), "completed-battle bound differs from policy");
assert(projection.includes("fields.len() > 100"), "projection-field bound differs from policy");
for (const marker of [
  "pub struct ActivityBattleResultContract", "pub struct ActivityBattleHandoff",
  "pub struct ActivityBattleStartRequest", "pub struct ActivityBattleResultSubmission",
  "pub enum HpCarryPolicy", "pub enum EnergyCarryPolicy", "pub enum LifeCarryPolicy",
  "pub enum PresenceCarryPolicy", "pub enum MetricSettlementPolicy",
  "ResultIdentityMismatch", "ResultDigestMismatch", "ResultProjectionMismatch",
  "FaultOutcomeMismatch", "ParticipantMaximumMismatch", "AmbiguousOutcomeEdge",
  "InvalidCarryPolicy",
  "pub fn start_pending_battle", "pub fn submit_pending_battle_result"
]) assert(source.includes(marker), `battle settlement omits ${marker}`);
assert(/rng\s*\.snapshots\(\)/.test(source) && source.includes("ActivityRngLabel::Battle"), "battle seed does not use the independent Battle stream");
assert(!source.includes("choose_index(") && !source.includes("choose_weighted("), "battle handoff consumes an Activity RNG draw");
assert(source.includes("working.carry.insert") && transaction.includes("self.carry.encode"), "participant carry is not committed and canonically encoded");
assert(view.includes("participant_carry: Box<[ActivityParticipantCarryState]>") && view.includes("pub fn participant_carry"), "bounded player view omits participant carry");
assert(preparation.includes("pub(crate) fn mark_settled") && preparation.includes("is_settled"), "settled attempt cannot be replaced for a later battle");
assert(replay.includes("ProjectedValue::ParticipantState(state)") && replay.includes("ParticipantBattleState::new"), "replay codec omits projected participant state");
assert(!/f32|f64|HashMap/.test(source + projection + transaction + view), "settlement uses float or unordered state");
assert((tests.match(/^#\[test\]$/gm) ?? []).length === policy.focused_tests, "battle-settlement focused test count differs");
for (const marker of [
  "verified_result_projects_metrics_and_exact_participant_carry",
  "loss_preserves_defeat_and_departure",
  "stale_forged_and_incompatible_results_preserve_bytes_and_rng",
  "settled_attempt_can_enter_the_next_battle_with_the_carry_ledger",
  "contract_rejects_undeclared_participants_and_wrong_metric_slots"
]) assert(tests.includes(marker), `battle-settlement tests omit ${marker}`);
const contractGolden = arrayGolden(tests, /handoff\.contract_digest\(\)\.bytes\(\),\s*\[([\s\S]*?)\]\s*\)/);
const seedGolden = arrayGolden(tests, /handoff\.identity\(\)\.seed\(\)\.bytes\(\),\s*\[([\s\S]*?)\]\s*\)/);
const stateGolden = arrayGolden(tests, /settlement\.state_hash\(\)\.bytes\(\),\s*\[([\s\S]*?)\]\s*\)/);
assert(contractGolden === policy.goldens.settlement_contract_sha256, "settlement-contract golden differs");
assert(seedGolden === policy.goldens.first_battle_seed_sha256, "battle-seed golden differs");
assert(stateGolden === policy.goldens.settled_activity_state_sha256, "settled-state golden differs");
const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P2-B6` \| `(InProgress|Complete)` \|/m.test(status), "G04-P2-B6 is not active or complete");

const evidence = {
  schema_revision: "starclock.goal04-battle-settlement-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "verified-battle-results-project-typed-metrics-and-participant-carry-atomically",
  bounds: policy.bounds,
  shape: policy.shape,
  goldens: policy.goldens,
  compatibility: {
    goal01_one_battle_hash_goldens_unchanged: true,
    generated_rows_public: false,
    authoritative_float: false,
    activity_constructs_combat_internals: false
  },
  source_sha256: {
    settlement: sha256("crates/starclock-activity/src/battle_settlement.rs"),
    projection: sha256("crates/starclock-activity/src/projection.rs"),
    preparation: sha256("crates/starclock-activity/src/battle_preparation.rs"),
    transaction: sha256("crates/starclock-activity/src/transaction.rs"),
    view: sha256("crates/starclock-activity/src/view.rs"),
    replay: sha256("crates/starclock-replay/src/activity.rs"),
    tests: sha256("crates/starclock-activity/tests/battle_settlement.rs")
  },
  focused_tests: policy.focused_tests,
  new_registry_packages: policy.new_registry_packages
};
const relative = "evidence/standard-universe-runtime-v1/activity/battle-settlement.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "battle-settlement evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "battle-settlement evidence is stale; run with --bless");
}
console.log(`Goal 04 battle settlement verified (${policy.focused_tests} tests, ${policy.bounds.maximum_completed_battles} completed-battle bound).`);

function arrayGolden(sourceText, pattern) {
  const match = sourceText.match(pattern);
  assert(match !== null, "expected Rust byte-array golden is missing");
  return match[1].split(",").map((value) => value.trim()).filter(Boolean).map((value) => Number.parseInt(value, 10).toString(16).padStart(2, "0")).join("");
}
function numberLiteral(value) { return String(value).replace(/\B(?=(\d{3})+(?!\d))/g, "_"); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
