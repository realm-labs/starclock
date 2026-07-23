import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-universe-encounter-runtime.json");
assert(policy.schema_revision === "starclock.goal04-universe-encounter-runtime.v1", "unexpected encounter-runtime policy revision");
const topology = text("crates/starclock-mode-universe/src/topology.rs");
const slots = text("crates/starclock-mode-universe/src/encounter_slot.rs");
const overlay = text("crates/starclock-mode-universe/src/battle_overlay.rs");
const runtime = text("crates/starclock-mode-universe/src/runtime.rs");
const activity = text("crates/starclock-activity/src/graph_activity.rs");
const tests = text("crates/starclock-mode-universe/tests/encounter_runtime.rs");

for (const marker of [
  "ResolvedRoomContent", "ActivityRngLabel::Encounter", "ROOM_DRAW_PURPOSE",
  "MEMBER_DRAW_PURPOSE", "ActivityDecisionKind::Encounter", "ActivityDecisionKind::Reward",
  "ActivityEdgeCondition::BattleOutcome", "source_group_id"
]) assert(topology.includes(marker), `encounter micrograph omits ${marker}`);
for (const marker of ["Mandatory", "Optional", "Sequential", "OneOf", "skip_optional", "can_exit"])
  assert(slots.includes(marker), `encounter-slot model omits ${marker}`);
for (const marker of ["UniverseEncounterOverlay", "DuplicateBattleSpec", "participant_lock_digest", "binding_for_spec"])
  assert(overlay.includes(marker), `battle overlay omits ${marker}`);
for (const marker of ["engage_encounter", "choose_preparation_option", "start_pending_battle", "submit_pending_battle_result"])
  assert(runtime.includes(marker) && activity.includes(marker), `Activity battle chain omits ${marker}`);
assert(!/position_[xy]|f32|f64|HashMap/.test(topology + slots + overlay + runtime), "encounter runtime contains spatial coordinates, floats or unordered state");
assert((tests.match(/^#\[test\]$/gm) ?? []).length === 2, "encounter integration test count differs");
assert((slots.match(/^    #\[test\]$/gm) ?? []).length === 1, "encounter-slot unit test count differs");
assert(tests.includes("encounter_resolution_preparation_handoff_and_reward_return_are_one_deterministic_chain"), "encounter end-to-end test is missing");
assert(tests.includes("baseline_runner_uses_offered_options_and_executes_nested_battles_to_terminal"), "baseline complete-run encounter test is missing");
assert(tests.includes("overlay.bindings().len(), 173"), "all 173 encounter members are not represented by the overlay fixture");
const overlayGolden = arrayGolden(tests, /overlay\.digest\(\)\.bytes\(\),\s*\[([\s\S]*?)\]\s*\)/);
const rewardGolden = arrayGolden(tests, /settled\.state_hash\(\)\.bytes\(\),\s*\[([\s\S]*?)\]\s*\)/);
assert(overlayGolden === policy.goldens.resolved_overlay_sha256, "encounter overlay golden differs");
assert(rewardGolden === policy.goldens.won_reward_state_sha256, "post-battle reward-state golden differs");
const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P3-B3` \| `(InProgress|Complete)` \|/m.test(status), "G04-P3-B3 is not active or complete");

const evidence = {
  schema_revision: "starclock.goal04-universe-encounter-runtime-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "room-member-preparation-battle-result-reward-chain-is-deterministic-and-spatial-free",
  counts: policy.counts,
  policies: policy.policies,
  goldens: policy.goldens,
  deferred_scope: policy.scope,
  compatibility: {
    generated_rows_public: false,
    mutable_mode_engine_added: false,
    invalid_command_mutates_state_or_rng: false,
    complete_catalog_cloned_per_run: false
  },
  source_sha256: Object.fromEntries([
    "crates/starclock-mode-universe/src/topology.rs",
    "crates/starclock-mode-universe/src/encounter_slot.rs",
    "crates/starclock-mode-universe/src/battle_overlay.rs",
    "crates/starclock-mode-universe/src/runtime.rs",
    "crates/starclock-activity/src/graph_activity.rs",
    "crates/starclock-mode-universe/tests/encounter_runtime.rs"
  ].map((relative) => [relative, sha256(relative)])),
  focused_tests: policy.focused_tests,
  new_registry_packages: policy.new_registry_packages
};
const relative = "evidence/standard-universe-runtime-v1/profile/universe-encounter-runtime.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "Universe encounter-runtime evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "Universe encounter-runtime evidence is stale; run with --bless");
}
console.log(`Goal 04 Universe encounter runtime verified (${policy.counts.rooms} rooms, ${policy.counts.encounter_members} members, ${policy.counts.enemy_slots} enemy slots).`);

function arrayGolden(sourceText, pattern) {
  const match = sourceText.match(pattern);
  assert(match !== null, "expected Rust byte-array golden is missing");
  return match[1].split(",").map((value) => value.trim()).filter(Boolean).map((value) => Number.parseInt(value, 10).toString(16).padStart(2, "0")).join("");
}
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
