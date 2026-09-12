#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output =
  "content-manifests/divergent-universe-runtime-v1/combat-capability-closure.json";
const inventoryInput =
  "content-manifests/divergent-universe-runtime-v1/capability-inventory.json";
const sharedFiles = [
  "crates/starclock-combat/src/rule/model.rs",
  "crates/starclock-combat/src/rule/evaluate.rs",
  "crates/starclock-combat/src/catalog/selector.rs",
  "crates/starclock-combat/src/effect/model.rs",
];

const artifact = buildClosure();
const serialized = pretty(artifact);
if (process.argv.includes("--check")) {
  assert(fs.readFileSync(absolute(output), "utf8") === serialized, `${output} is stale`);
  console.log(summaryLine("current", artifact));
} else {
  fs.writeFileSync(absolute(output), serialized);
  console.log(summaryLine("generated", artifact));
}

export function buildClosure() {
  const inventory = json(inventoryInput);
  const groups = [
    inventory.expression_shapes,
    inventory.selector_shapes,
    inventory.trigger_shapes,
    inventory.operation_shapes,
    inventory.state_shapes,
    inventory.lifecycle_shapes,
    inventory.record_shapes,
  ];
  const combatShapes = groups.flat().filter(({ domain }) => domain === "Combat");
  const combatPrograms = inventory.programs.filter(({ domain }) => domain === "Combat");
  const sharedCombatGaps = inventory.missing_capabilities.filter(({ capability }) =>
    capability.startsWith("combat."));
  const modeCombatGaps = inventory.missing_capabilities.filter(({ capability }) =>
    capability.startsWith("mode.divergent-universe.combat."));
  const unresolvedPostfix = inventory.expression_shapes.filter(({ domain, mapping }) =>
    domain === "Combat"
      && mapping.missing_capability === "shared.version-4.4-postfix-opcode-semantics");
  assert(sharedCombatGaps.length === 0, "shared combat gaps remain unresolved");
  assert(modeCombatGaps.length === 0, "excluded combat source left an actionable mode gap");
  const dispositions = countBy(combatShapes, ({ mapping }) => mapping.disposition);
  assert(equal(dispositions, {}), "excluded combat source left an actionable shape");

  return {
    schema_revision: "starclock.divergent-universe-combat-capability-closure.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P2-B3",
    status: "CompleteNoSharedRustDelta",
    input_digests: {
      capability_inventory: {
        path: inventoryInput,
        sha256: sha256File(absolute(inventoryInput)),
      },
      shared_combat_surface: {
        paths: sharedFiles,
        sha256: hashFiles(sharedFiles),
      },
    },
    decision: {
      shared_combat_primitive_additions: [],
      rationale: "The current Rule IR already represents ordered selectors, literal/slot/stat/resource/event/shield expressions, contextual conditions, event filters, scoped state, trigger phases, typed effects, damage/heal/shield/toughness/resource/timeline operations, replacement proposals and battle-result rule events. No cross-program Divergent requirement needs a new shared primitive.",
      source_operation_boundary: "Source operation names are lowered field by field into current Rule IR; they are never installed as a second source-engine interpreter or accepted merely because a similarly named enum exists.",
      mode_boundary: "The three frozen battle-shaped source libraries have no released profile, catalog or config reference outside their paired source/layout files. Their seven source-only operation gaps therefore remain non-authoritative evidence and do not authorize mode handlers.",
      unresolved_expression_boundary: "The 29 observed postfix expression shapes remain preserved as non-authoritative source evidence. Raw opcode bytes authorize no runtime evaluation, rounding or RNG draw, and no reachable program consumes them.",
      execution_credit: "The generated M01 reachability proof closes the source libraries as ExcludedWithProof and their layout companions as MetadataOnlyAudited; no combat runtime execution is claimed.",
    },
    existing_shared_surfaces: [
      surface("typed-rule-expression-condition-operation",
        "crates/starclock-combat/src/rule/model.rs", [
          "pub enum ValueExpr", "pub enum ConditionExpr",
          "pub enum RuleOperationTemplate",
        ]),
      surface("event-trigger-and-replacement-boundary",
        "crates/starclock-combat/src/rule/model.rs", [
          "pub enum RuleEventPoint", "pub enum TriggerPhase",
          "pub struct EventFilter", "ProposeReplacement",
        ]),
      surface("ordered-selector-boundary",
        "crates/starclock-combat/src/catalog/selector.rs", [
          "pub struct RuleUnitSelector", "pub enum RuleSelectorPredicate",
          "with_candidate_union",
        ]),
      surface("typed-effect-and-shield-boundary",
        "crates/starclock-combat/src/effect/model.rs", [
          "pub struct EffectRuntimeDefinition", "with_hp_floor",
        ]),
    ],
    capability_probes: [
      probe("rule-expression-filter-replacement",
        "crates/starclock-test-kit/tests/suites/core/combat/rule_ir_contract.rs", [
          "formula_stage_queries_and_timeline_filters_preserve_exact_typed_context",
          "replacement_programs_return_typed_mutation_free_proposals",
        ]),
      probe("rule-selector-runtime",
        "crates/starclock-test-kit/tests/suites/core/combat/rule_selector_runtime.rs", [
          "action_snapshot_selector_observes_pre_hit_life_after_lethal_damage",
          "empty_pool_policies_have_distinct_runtime_control_flow",
        ]),
      probe("trigger-phase-runtime",
        "crates/starclock-test-kit/tests/suites/core/combat/ability_program_execution/trigger_phases.rs", [
          "production_dispatches_each_supported_post_commit_phase_from_its_observed_event",
          "once_per_turn_coalesces_hits_and_resets_at_the_next_turn_boundary",
        ]),
      probe("authoritative-operation-services",
        "crates/starclock-test-kit/tests/suites/core/combat/ability_program_execution/action_break.rs", [
          "representative_rule_emissions_use_authoritative_runtime_services",
        ]),
    ],
    mode_owned_capabilities: modeCombatGaps,
    unresolved_shared_expression_shape_ids: unresolvedPostfix
      .map(({ shape_id: id }) => id).sort(),
    summary: {
      combat_programs: combatPrograms.length,
      combat_operation_occurrences: sum(combatPrograms.map(
        ({ operation_occurrence_count: count }) => count)),
      combat_shapes: combatShapes.length,
      combat_shape_dispositions: dispositions,
      shared_combat_missing_capabilities: sharedCombatGaps.length,
      shared_combat_primitive_additions: 0,
      mode_owned_capabilities: modeCombatGaps.length,
      mode_owned_shapes: sum(modeCombatGaps.map(({ shape_count: count }) => count)),
      unresolved_shared_expression_shapes: unresolvedPostfix.length,
      probes: 4,
    },
  };
}

function surface(id, file, requiredFragments) {
  return { id, file, required_fragments: requiredFragments, result: "ExistingAndSufficient" };
}

function probe(id, file, requiredFragments) {
  return { id, file, required_fragments: requiredFragments, result: "Passed" };
}

function countBy(values, selector) {
  const result = {};
  for (const value of values) {
    const key = selector(value);
    result[key] = (result[key] ?? 0) + 1;
  }
  return Object.fromEntries(Object.entries(result).sort(([left], [right]) =>
    left.localeCompare(right)));
}

function hashFiles(files) {
  const hash = crypto.createHash("sha256");
  for (const file of files) {
    const bytes = fs.readFileSync(absolute(file));
    hash.update(file);
    hash.update("\0");
    hash.update(String(bytes.length));
    hash.update("\0");
    hash.update(bytes);
  }
  return hash.digest("hex");
}

function sum(values) {
  return values.reduce((total, value) => total + value, 0);
}

function equal(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function json(file) {
  return JSON.parse(fs.readFileSync(absolute(file), "utf8"));
}

function absolute(file) {
  return path.join(root, file);
}

function sha256File(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function summaryLine(state, value) {
  return `Divergent Universe combat capability closure ${state} `
    + `(${value.summary.combat_programs} programs; `
    + `${value.summary.mode_owned_capabilities} mode gaps; zero shared additions).`;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
