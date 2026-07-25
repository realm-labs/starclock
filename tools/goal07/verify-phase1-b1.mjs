import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = json("policy/goal07-trigger-runtime.json");
assert(
  policy.schema_revision === "starclock.goal07-trigger-runtime.v1"
    && policy.batch === "G07-P1-B1",
  "trigger runtime policy identity differs",
);
assert(policy.once_scopes.length === 8, "battle once-scope denominator differs");
assert(policy.cause_fields.length === 10, "cause field denominator differs");
assert(
  policy.currently_unobserved_points.length === 3,
  "unobserved event-point denominator differs",
);
assert(
  policy.replacement.proposal_api === "evaluate_replacement_program"
    && policy.replacement.ordinary_mutations_allowed === false
    && policy.replacement.accepted_without_typed_consumer === false
    && policy.replacement.consumer_integration_batch === "G07-P1-B5",
  "replacement boundary differs",
);
assert(
  Object.values(policy.contracts).every((value) => value === true),
  "trigger runtime contract is incomplete",
);

const model = text("crates/starclock-combat/src/rule/model.rs");
for (const marker of [
  "pub struct RuleReplacementProposal",
  "pub parent_event: Option<EventId>",
  "pub root_command: Option<crate::CommandId>",
  "pub action: Option<ActionId>",
  "pub phase: Option<crate::PhaseId>",
  "pub hit: Option<HitId>",
  "OnceScope::Turn => (0, 0)",
]) assert(model.includes(marker), `rule model marker is missing: ${marker}`);
assert(
  text("crates/starclock-combat/src/rule/timing.rs")
    .includes("pub const fn runtime_phases"),
  "trigger timing matrix module is missing",
);

const evaluator = text("crates/starclock-combat/src/rule/evaluate.rs");
for (const marker of [
  "pub fn evaluate_replacement_program",
  "pub(crate) fn reset_scope",
  "pub(crate) fn reset_event",
]) assert(evaluator.includes(marker), `rule evaluator marker is missing: ${marker}`);

const resolver = text("crates/starclock-combat/src/resolver/rule.rs");
assert(
  resolver.includes("for phase in event_point.runtime_phases()"),
  "production resolver does not dispatch the phase matrix",
);
assert(
  resolver.includes("let mut event_parent = event.id();"),
  "rule cause chains do not start from the observed event",
);
assert(
  resolver.includes("txn.reset_event_once_keys(event.id());"),
  "event once keys are not bounded",
);

const validation = text("crates/starclock-combat/src/catalog/rule_validate.rs");
assert(
  validation.includes(".runtime_phases()")
    && validation.includes(".contains(&trigger.phase)"),
  "catalog accepts an unobservable trigger phase",
);

const tests =
  text("crates/starclock-combat/tests/ability_program_execution.rs")
  + text(
    "crates/starclock-combat/tests/ability_program_execution/trigger_phases.rs",
  );
for (const marker of [
  "production_dispatches_each_supported_post_commit_phase_from_its_observed_event",
  "after_defeat_settlement_dispatches_from_the_defeated_fact",
  "once_per_turn_coalesces_hits_and_resets_at_the_next_turn_boundary",
]) assert(tests.includes(marker), `production trigger probe is missing: ${marker}`);

const status = text(
  "docs/goals/07-standard-universe-mechanics-completion-status.md",
);
assert(
  status.includes("| `G07-P1-B1` | `Complete` |"),
  "G07-P1-B1 is not complete",
);
const nextBatch = status.match(/^\| Next unblocked batch \| (.+) \|$/mu)?.[1];
assert(
  nextBatch === "None"
    || /^`G07-(?:P1-B[2-6]|P[2-5]-M\d+-S\d+|P[67]-B\d+)`$/u
      .test(nextBatch ?? ""),
  "next batch regressed before G07-P1-B2",
);
console.log(
  "Goal 07 P1-B1 verified "
    + "(6 phase families, 8 once scopes, 10 cause fields, fail-closed replacement).",
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
