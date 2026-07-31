import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = json("policy/goal07-selector-runtime.json");
assert(
  policy.schema_revision === "starclock.goal07-selector-runtime.v1"
    && policy.batch === "G07-P1-B2",
  "selector runtime policy identity differs",
);
assert(policy.origins === 8, "selector-origin denominator differs");
assert(policy.reference_points.length === 3, "selector reference denominator differs");
assert(policy.predicate_families.length === 7, "selector predicate denominator differs");
assert(policy.ordering_families.length === 8, "selector ordering denominator differs");
assert(policy.choice_families.length === 5, "selector choice denominator differs");
assert(policy.empty_pool_policies.length === 4, "empty-pool denominator differs");
assert(
  Object.values(policy.contracts).every((value) => value === true),
  "selector runtime contract is incomplete",
);

const generated = json("config/generated/debug-json/SelectorPredicate.json");
assert(
  generated.table.rows.length >= policy.production_selector_predicate_rows,
  "production selector-predicate rows regressed below the frozen denominator",
);
const predicate = generated.table.rows[0]?.values?.predicate?.Object;
assert(
  predicate?.type?.String === "FormationRange"
    && predicate.minimum_index?.Integer === 0
    && predicate.maximum_index?.Integer === 31,
  "production selector-predicate probe differs",
);

const selector = text("crates/starclock-combat/src/catalog/selector.rs");
for (const marker of [
  "pub enum RuleSelectorPredicate",
  "pub(crate) fn dependencies",
  "RuleSelectorPredicate::OwnedBy",
  "RuleSelectorPredicate::StatCompare",
]) assert(selector.includes(marker), `selector model marker is missing: ${marker}`);

const resolver = text("crates/starclock-combat/src/resolver/target.rs");
for (const marker of [
  "pub(super) fn ordered_rule_selectors",
  "RuleSelectorResolution::CancelRemaining",
  "choose_weighted",
  "selector.reference()",
  "RuleSelectorOrdering::EventOrder",
]) assert(resolver.includes(marker), `selector resolver marker is missing: ${marker}`);

const transaction = text("crates/starclock-combat/src/resolver/transaction.rs");
assert(
  transaction.includes("selector_event_snapshots")
    && transaction.includes("selector_action_snapshots"),
  "historical selector snapshots are not transaction-owned",
);
const lowerer = text("crates/starclock-data/src/selector_lower.rs");
assert(
  lowerer.includes("selector_predicate()")
    && lowerer.includes("weight_expression_id")
    && lowerer.includes("lower_predicate"),
  "Sora selector predicates/weights are not lowered",
);
const tests =
  text("crates/starclock-test-kit/tests/suites/core/combat/rule_selector_runtime.rs")
  + text("crates/starclock-test-kit/tests/suites/core/combat/catalog_contract.rs")
  + text("crates/starclock-test-kit/tests/suites/core/data/production_hit_plans.rs");
for (const marker of [
  "action_snapshot_selector_observes_pre_hit_life_after_lethal_damage",
  "empty_pool_policies_have_distinct_runtime_control_flow",
  "selector_rng_order_and_dependency_contracts_fail_closed",
  "FormationRange",
]) assert(tests.includes(marker), `selector golden is missing: ${marker}`);

const status = text(
  "docs/goals/07-standard-universe-mechanics-completion-status.md",
);
assert(
  status.includes("| `G07-P1-B2` | `Complete` |"),
  "G07-P1-B2 is not complete",
);
const nextBatch = status.match(/^\| Next unblocked batch \| (.+) \|$/mu)?.[1];
assert(
  nextBatch === "None"
    || /^`G07-(?:P1-B[3-6]|P[2-5]-M\d+-S\d+|P[67]-B\d+)`$/u
      .test(nextBatch ?? ""),
  "next batch regressed before G07-P1-B3",
);

console.log(
  "Goal 07 P1-B2 verified "
    + "(3 references, 7 predicates, 8 orders, 5 choices, 4 empty policies).",
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
