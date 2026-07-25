import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = json("policy/goal07-effect-state-runtime.json");
assert(
  policy.schema_revision === "starclock.goal07-effect-state-runtime.v1"
    && policy.batch === "G07-P1-B4",
  "effect/state runtime policy identity differs",
);
for (const [field, expected] of Object.entries({
  slot_reset_points: 9,
  effect_stack_policies: 8,
  duration_clocks: 8,
  tick_phases: 6,
  teardown_policies: 5,
  production_effect_definitions: 4,
  production_state_slots: 3,
  production_state_slot_resets: 2,
  production_character_resources: 46,
})) assert(policy[field] === expected, `${field} denominator differs`);
assert(
  Object.values(policy.contracts).every((value) => value === true),
  "effect/state runtime contract is incomplete",
);

const resets = json("config/generated/debug-json/StateSlotReset.json").table.rows;
assert(resets.length === policy.production_state_slot_resets, "state-slot reset count differs");
const probe = resets.find((row) =>
  row.values.state_slot_id?.Integer === 24_003
  && row.values.sequence?.Integer === 1
)?.values;
assert(
  probe?.reset_point?.String === "BattleStart",
  "formal BattleStart reset probe differs",
);

const state = text("crates/starclock-combat/src/rule/state.rs");
assert(
  state.includes("every_declared_lifecycle_reset_point_is_executable"),
  "complete slot-reset golden is missing",
);
const effects = text("crates/starclock-combat/src/resolver/effect_boundary.rs")
  + text("crates/starclock-combat/src/resolver/effect_operation.rs")
  + text("crates/starclock-combat/src/resolver/operation.rs");
for (const marker of [
  "EffectTickPhase",
  "checked_mul_integer(i64::from(effect.stacks))",
  "settle_effects_at_wave_end",
  "settle_effects_at_battle_end",
  "refresh_effect_stacks",
]) assert(effects.includes(marker), `effect runtime marker is missing: ${marker}`);
const lowerer = text("crates/starclock-data/src/modifier_lower.rs");
assert(
  lowerer.includes('"source_effect_stacks"')
    && lowerer.includes("source_stack_slot"),
  "source-effect stack binding is not lowered",
);
const tests = text("crates/starclock-combat/tests/effect_resource_pipeline.rs")
  + text("crates/starclock-combat/tests/catalog_contract.rs")
  + text("crates/starclock-data/src/catalog_modifier_tests.rs");
for (const marker of [
  "kafka_style_detonation_retains_source_snapshot_duration_and_stacks",
  "source_effect_stack_slots_require_one_effect_owner",
  "production_state_slot_reset_survives_excel_and_sora_lowering",
]) assert(tests.includes(marker), `effect/state golden is missing: ${marker}`);

const status = text("docs/goals/07-standard-universe-mechanics-completion-status.md");
assert(
  status.includes("| `G07-P1-B4` | `Complete` |"),
  "G07-P1-B4 is not complete",
);
console.log(
  "Goal 07 P1-B4 verified "
    + "(9 resets, 8 stack policies, 8 clocks, 46 character resources).",
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
