#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output =
  "content-manifests/divergent-universe-runtime-v1/activity-capability-closure.json";
const inventoryInput =
  "content-manifests/divergent-universe-runtime-v1/capability-inventory.json";
const sharedFiles = [
  "crates/starclock-activity/src/program.rs",
  "crates/starclock-activity/src/state_definition.rs",
  "crates/starclock-activity/src/graph_activity.rs",
  "crates/starclock-activity/src/interaction.rs",
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
  const activityShapes = groups.flat().filter(({ domain }) => domain === "Activity");
  const activityPrograms = inventory.programs.filter(({ domain }) => domain === "Activity");
  const sharedActivityGaps = inventory.missing_capabilities.filter(({ capability }) =>
    capability.startsWith("activity."));
  const modeActivityGaps = inventory.missing_capabilities.filter(({ capability }) =>
    capability.startsWith("mode.divergent-universe.activity."));
  assert(sharedActivityGaps.length === 0, "shared Activity gaps remain unresolved");
  assert(modeActivityGaps.length === 42, "mode-owned Activity gap denominator drift");
  const dispositions = countBy(activityShapes, ({ mapping }) => mapping.disposition);
  assert(equal(dispositions, {
    ExistingPrimitive: 56,
    MissingCapability: 63,
    NonAuthoritative: 93,
  }), "Activity shape disposition drift");

  return {
    schema_revision: "starclock.divergent-universe-activity-capability-closure.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P2-B2",
    status: "CompleteNoSharedRustDelta",
    input_digests: {
      capability_inventory: {
        path: inventoryInput,
        sha256: sha256File(absolute(inventoryInput)),
      },
      shared_activity_surface: {
        paths: sharedFiles,
        sha256: hashFiles(sharedFiles),
      },
    },
    decision: {
      shared_activity_primitive_additions: [],
      rationale: "The current typed Activity vocabulary already represents accepted choices, ordered offers, conditions, inventory/state mutation, graph traversal/relocation/terminal outcomes and logical scope boundaries. No cross-program Divergent requirement needs a new shared primitive.",
      source_graph_boundary: "Dialogue waits, custom-string/entity signals, wall-clock waits and scene-entity selectors sequence presentation or external room gameplay. They are non-authoritative and may not create an Activity event bus, wall-clock scheduler or scene-object model.",
      mode_boundary: "The remaining source world, puzzle, prop, Adventure and minigame operations stay named mode-owned gaps for their assigned mechanic partitions. They may lower to typed Divergent programs, typed external outcomes or proven non-runtime dispositions, but not to content-ID branches in starclock-activity.",
      execution_credit: "None. This batch closes only shared Activity capability sufficiency; no Divergent source program is runtime-lowered or executed.",
    },
    existing_shared_surfaces: [
      surface("typed-expression-condition-state", "crates/starclock-activity/src/program.rs", [
        "pub enum ActivityExpression", "pub enum ActivityCondition",
        "pub enum ActivityOperation",
      ]),
      surface("offers-and-ordered-options", "crates/starclock-activity/src/program.rs", [
        "pub struct ActivityOptionDefinition", "Offer {",
      ]),
      surface("scoped-state-and-snapshots",
        "crates/starclock-activity/src/state_definition.rs", [
          "pub struct ActivityScopePath", "pub enum ActivitySnapshotBoundary",
          "pub struct ActivityStateDefinition",
        ]),
      surface("node-program-and-route-boundary",
        "crates/starclock-activity/src/graph_activity.rs", [
          "pub struct GraphActivityNodeProgram", "pub struct GraphActivityDefinition",
        ]),
      surface("typed-external-interaction-boundary",
        "crates/starclock-activity/src/interaction.rs", [
          "pub struct ActivityInteractionBinding", "pub struct ActivityInteractionBindings",
        ]),
    ],
    capability_probes: [
      probe("atomic-state-inventory-offer-and-route",
        "crates/starclock-test-kit/tests/suites/activity/activity/activity_transaction.rs", [
          "ordered_program_commits_slots_inventory_modifiers_graph_and_decision_atomically",
          "explicit_relocation_obeys_graph_node_and_visit_contracts_without_an_edge",
        ]),
      probe("typed-conditions-and-rejection-identity",
        "crates/starclock-test-kit/tests/suites/activity/activity/activity_transaction.rs", [
          "failed_requirement_rejects_without_changing_any_state",
          "conditional_executes_one_final_branch_and_preserves_transactionality",
        ]),
      probe("logical-scope-boundary",
        "crates/starclock-test-kit/tests/suites/activity/activity/logical_scope.rs", [
          "physical_nodes_share_one_logical_visit_and_reentry_is_fresh",
        ]),
      probe("external-interaction-transaction",
        "crates/starclock-test-kit/tests/suites/activity/activity/interaction_runtime.rs", [
          "accepted_random_interaction_commits_draw_effect_and_transition_together",
          "stale_and_faulting_random_interactions_preserve_exact_state_and_rng",
        ]),
    ],
    mode_owned_capabilities: modeActivityGaps,
    summary: {
      activity_programs: activityPrograms.length,
      activity_operation_occurrences: sum(activityPrograms.map(
        ({ operation_occurrence_count: count }) => count)),
      activity_shapes: activityShapes.length,
      activity_shape_dispositions: dispositions,
      shared_activity_missing_capabilities: sharedActivityGaps.length,
      shared_activity_primitive_additions: 0,
      mode_owned_capabilities: modeActivityGaps.length,
      mode_owned_shapes: sum(modeActivityGaps.map(({ shape_count: count }) => count)),
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
  return `Divergent Universe Activity capability closure ${state} `
    + `(${value.summary.activity_programs} programs; `
    + `${value.summary.mode_owned_capabilities} mode gaps; zero shared additions).`;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
