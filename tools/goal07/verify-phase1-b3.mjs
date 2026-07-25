import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = json("policy/goal07-modifier-runtime.json");
assert(
  policy.schema_revision === "starclock.goal07-modifier-runtime.v1"
    && policy.batch === "G07-P1-B3",
  "modifier runtime policy identity differs",
);
for (const [field, expected] of Object.entries({
  stat_stages: 5,
  formula_stages: 16,
  aggregation_policies: 9,
  snapshot_policies: 9,
  production_modifier_definitions: 1556,
  production_modifier_filters: 151,
  production_formula_stage_modifiers: 268,
  production_authored_comparator_groups: 1,
})) assert(policy[field] === expected, `${field} denominator differs`);
assert(
  Object.values(policy.contracts).every((value) => value === true),
  "modifier runtime contract is incomplete",
);

const groups = json("config/generated/debug-json/ModifierStackingGroup.json")
  .table.rows;
const probe = groups.find((row) => row.values.id?.Integer === 970001)?.values;
assert(
  probe?.stable_key?.String === "goal07.probe.modifier.strongest-comparator"
    && probe.aggregation?.String === "StrongestByComparator"
    && probe.comparator_expression_id?.Integer === 24801,
  "formal comparator probe differs",
);

const resolver = text("crates/starclock-combat/src/modifier/resolve.rs");
for (const marker of [
  "pub fn query_formula",
  "fn aggregate_group",
  "registry checked comparator",
  "fn apply_bounds",
]) assert(resolver.includes(marker), `modifier resolver marker is missing: ${marker}`);

const snapshots = text(
  "crates/starclock-combat/src/resolver/modifier_snapshot.rs",
);
for (const marker of [
  "pub(crate) fn initialize_battle",
  "SnapshotPolicy::OnApplication",
  "SnapshotPolicy::OnActionStart",
  "capture_stats",
]) assert(snapshots.includes(marker), `snapshot marker is missing: ${marker}`);

const formula = text("crates/starclock-combat/src/resolver/operation_formula.rs");
for (const marker of [
  "struct FormulaInputs",
  "fn damage",
  "fn healing",
  "fn shield",
]) assert(formula.includes(marker), `formula bridge marker is missing: ${marker}`);

const tests =
  text("crates/starclock-combat/tests/modifier_pipeline.rs")
  + text("crates/starclock-combat/tests/damage_lifecycle.rs")
  + text("crates/starclock-data/src/catalog_modifier_tests.rs");
for (const marker of [
  "strongest_comparator_is_authored_and_weaker_instance_remains_available",
  "application_action_phase_hit_and_stack_snapshots_change_damage",
  "production_rows_execute_formula_stage_and_authored_comparator",
]) assert(tests.includes(marker), `modifier golden is missing: ${marker}`);

const status = text(
  "docs/goals/07-standard-universe-mechanics-completion-status.md",
);
assert(
  status.includes("| `G07-P1-B3` | `Complete` |"),
  "G07-P1-B3 is not complete",
);
console.log(
  "Goal 07 P1-B3 verified "
    + "(1556 definitions, 268 formula modifiers, 9 snapshots).",
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
