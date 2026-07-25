import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = json("policy/goal07-action-reaction-break-runtime.json");
assert(
  policy.schema_revision === "starclock.goal07-action-reaction-break-runtime.v1"
    && policy.batch === "G07-P1-B5",
  "action/reaction runtime policy identity differs",
);
for (const [field, expected] of Object.entries({
  action_origins: 11,
  reaction_boundaries: 4,
  reaction_tiers: 4,
  maximum_reactions_per_command: 256,
  wave_transition_policies: 4,
  enemy_phase_transition_models: 3,
  production_operation_probes: 3,
  state_codec_version: 4,
  event_payload_version: 3,
})) assert(policy[field] === expected, `${field} denominator differs`);
assert(policy.state_hash_revision === "sha256-v5", "state hash revision differs");
assert(
  Object.values(policy.contracts).every((value) => value === true),
  "action/reaction runtime contract is incomplete",
);

const operations = json("config/generated/debug-json/Operation.json").table.rows;
for (const id of [24_701, 24_702, 24_703]) {
  assert(
    operations.some((row) => row.values.id?.Integer === id),
    `formal operation probe ${id} is missing`,
  );
}
const combat = text("crates/starclock-combat/src/lib.rs");
const codec = text("crates/starclock-combat/src/codec/state.rs");
const replay = text("crates/starclock-replay/src/battle_event.rs");
assert(combat.includes('STATE_HASH_REVISION: &str = "sha256-v6"'), "current combat hash revision differs");
assert(codec.includes("STATE_CODEC_VERSION: u16 = 5"), "current combat state codec differs");
assert(replay.includes("BATTLE_EVENT_PAYLOAD_VERSION: u16 = 4"), "current event payload differs");
for (const version of [
  "BATTLE_EVENT_PAYLOAD_VERSION_V1",
  "BATTLE_EVENT_PAYLOAD_VERSION_V2",
  "BATTLE_EVENT_PAYLOAD_VERSION_V3",
]) {
  assert(replay.includes(version), `historical event codec is missing: ${version}`);
}
const runtime = [
  "crates/starclock-combat/src/resolver/program_timeline.rs",
  "crates/starclock-combat/src/resolver/operation_break.rs",
  "crates/starclock-combat/src/resolver/program_break.rs",
  "crates/starclock-combat/src/resolver/lifecycle.rs",
  "crates/starclock-combat/src/resolver/settle.rs",
].map(text).join("\n");
for (const marker of [
  "ExtraTurnGranted",
  "ActionGaugeChanged",
  "execute_force_break",
  "seed_observed_reduction",
  "execute_boundary_program",
]) assert(runtime.includes(marker), `runtime marker is missing: ${marker}`);
const tests = [
  "crates/starclock-combat/tests/ability_program_execution/action_break.rs",
  "crates/starclock-combat/tests/enemy_orchestration.rs",
  "crates/starclock-combat/tests/damage_lifecycle.rs",
  "crates/starclock-combat/src/reaction/queue.rs",
  "crates/starclock-data/src/operation_lower.rs",
].map(text).join("\n");
for (const marker of [
  "representative_rule_emissions_use_authoritative_runtime_services",
  "phase_transition_is_transactional_and_applies_every_carry_family",
  "nondefault_wave_boundaries_emit_at_the_authored_lifecycle_point",
  "semantic_tiers_cannot_be_inverted_by_authored_priority",
  "goal07_action_break_probes_survive_excel_sora_and_typed_lowering",
]) assert(tests.includes(marker), `action/reaction golden is missing: ${marker}`);
const status = text("docs/goals/07-standard-universe-mechanics-completion-status.md");
assert(status.includes("| `G07-P1-B5` | `Complete` |"), "G07-P1-B5 is not complete");
console.log(
  "Goal 07 P1-B5 verified " +
  "(11 origins, 4 boundaries, Break hooks, historical SCBS v4, current SCBS v5).",
);

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
