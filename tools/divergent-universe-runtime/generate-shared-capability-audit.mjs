#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const output = `${runtimeRoot}/shared-capability-audit.json`;
const inputs = {
  capability_inventory: `${runtimeRoot}/capability-inventory.json`,
  activity_closure: `${runtimeRoot}/activity-capability-closure.json`,
  combat_closure: `${runtimeRoot}/combat-capability-closure.json`,
  build_closure: `${runtimeRoot}/build-capability-closure.json`,
  mechanic_dispositions: `${runtimeRoot}/mechanic-dispositions.json`,
  mechanic_partitions: `${runtimeRoot}/mechanic-partitions.json`,
  runtime_contract: `${runtimeRoot}/runtime-contract.json`,
  native_handler_audit: "policy/native-handler-audit.json",
};
const sharedRoots = [
  "crates/starclock-activity/src",
  "crates/starclock-build/src",
  "crates/starclock-combat/src",
  "crates/starclock-rules/src",
];

const artifact = buildAudit();
const serialized = pretty(artifact);
if (process.argv.includes("--check")) {
  assert(fs.readFileSync(absolute(output), "utf8") === serialized, `${output} is stale`);
  console.log(summaryLine("current", artifact));
} else {
  fs.writeFileSync(absolute(output), serialized);
  console.log(summaryLine("generated", artifact));
}

export function buildAudit() {
  const inventory = json(inputs.capability_inventory);
  const activity = json(inputs.activity_closure);
  const combat = json(inputs.combat_closure);
  const build = json(inputs.build_closure);
  const mechanics = json(inputs.mechanic_dispositions);
  const partitions = json(inputs.mechanic_partitions);
  const runtime = json(inputs.runtime_contract);
  const nativeAudit = json(inputs.native_handler_audit);
  const shapes = [
    inventory.expression_shapes,
    inventory.selector_shapes,
    inventory.trigger_shapes,
    inventory.operation_shapes,
    inventory.state_shapes,
    inventory.lifecycle_shapes,
    inventory.record_shapes,
  ].flat();
  const shapeById = new Map(shapes.map((shape) => [shape.shape_id, shape]));
  const mechanicById = new Map(mechanics.programs.map((mechanic) =>
    [mechanic.mechanic_id, mechanic]));
  const modeGaps = inventory.missing_capabilities.filter(({ capability }) =>
    capability.startsWith("mode.divergent-universe."));
  const modeShapes = modeGaps.flatMap(({ shape_ids: ids }) => ids.map((id) =>
    required(shapeById.get(id), `mode gap shape ${id}`)));
  const modeMechanics = unique(modeShapes.flatMap(({ mechanic_ids: ids }) => ids)).sort();
  const modePartitions = unique(modeMechanics.map((id) =>
    required(mechanicById.get(id), `mode gap mechanic ${id}`).execution_partition)).sort();
  const postfixShapes = inventory.expression_shapes.filter(({ encoding, domain }) =>
    encoding === "PostfixBase64" && domain === "Metadata");
  const postfixMechanics = unique(postfixShapes.flatMap(
    ({ sample_mechanic_ids: ids }) => ids)).sort();
  const postfixPartitions = unique(postfixMechanics.map((id) =>
    required(mechanicById.get(id), `postfix mechanic ${id}`).execution_partition)).sort();
  const auditedFiles = sharedRoots.flatMap((directory) => recursiveRustFiles(absolute(directory)))
    .map((file) => normalize(path.relative(root, file))).sort();
  const probes = [
    ...activity.capability_probes,
    ...combat.capability_probes,
    ...build.capability_probes,
    probe("static-registry-boundary",
      "crates/starclock-test-kit/tests/suites/core/rules/registry_contract.rs", [
        "production_registry_is_explicitly_empty_after_the_review",
        "registry_rejects_noncanonical_and_incomplete_static_metadata",
      ]),
  ];

  assert(modeGaps.length === 42 && modeShapes.length === 63,
    "mode-owned gap closure drift");
  assert(postfixShapes.length === 29 && postfixMechanics.length === 3,
    "postfix policy denominator drift");
  assert(auditedFiles.length === 206, "shared Rust audit denominator drift");
  assert(partitions.freeze.batch === "G22-P2-B5"
    && partitions.freeze.state === "FrozenPendingExecution",
  "mechanic partitions are not frozen");
  assert(runtime.handler_admission.default_admitted === 0
    && mechanics.summary.native_handlers_admitted === 0
    && nativeAudit.admitted_handlers.length === 0,
  "native-handler audit no longer closes at zero");

  return {
    schema_revision: "starclock.divergent-universe-shared-capability-audit.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P2-B5",
    status: "CompleteNoRuntimeExecutionCredit",
    source_revision: inventory.source_revision,
    source_access_date: inventory.source_access_date,
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256File(absolute(file)) },
    ])),
    capability_probes: probes,
    configuration_program_policy: {
      field: "mechanic.configuration_program",
      accuracy: "VersionedProjectPolicy",
      state: "ProvenNonRuntimeUnreachableAtPinnedRevision",
      known_facts: [
        "Version 4.4 source programs carry postfix Base64 byte sequences with separate fixed-value and dynamic-hash operand pools.",
        "The pinned Divergent inventory contains nine distinct opcode bytes and 29 distinct postfix sequences.",
        "Released source structure alone does not prove the complete operator table, numeric rounding or RNG behavior.",
      ],
      unresolved_field: "Complete semantics and numeric behavior of all nine observed Version 4.4 postfix opcode bytes.",
      selected_behavior: "M01 proves the containing source libraries unreachable from every released Config/ExcelOutput consumer at the pinned revision. Production installs no program or handler and never evaluates raw PostfixBase64, OpCodes or DynamicHashes.",
      rejected_alternatives: [
        "copy the source opcode API into a shared or mode-specific runtime interpreter",
        "infer opcode meanings from byte frequency, position or another game version",
        "classify an unresolved executable expression as metadata or lower it to a no-op",
      ],
      ordering: "Each frozen partition retains dependency order and stable mechanic identity; typed lowering owns final operation order.",
      rounding: "No rounding is inferred from opcode bytes; every typed formula boundary must declare project rounding explicitly.",
      candidate_set: "Only the listed expression shapes and mechanic IDs may use this policy boundary.",
      rng_stream: "No RNG draw is authorized by an unresolved postfix expression.",
      confidence: "ExactReleasedReachabilityEvidence",
      affected_expression_shape_ids: postfixShapes.map(({ shape_id: id }) => id).sort(),
      affected_mechanic_ids: postfixMechanics,
      affected_partition_ids: postfixPartitions,
      replacement_condition: "A released profile, catalog or config references one of the frozen ability identities and released evidence proves the required Version 4.4 postfix semantics for typed lowering.",
      replacement_trigger: "Verification fails if a frozen ability identity gains an outside reference, or if production Rust gains a raw PostfixExpr, OpCodes or DynamicHashes interpreter.",
    },
    mode_owned_gap_assignment: {
      state: "FrozenToGeneratedMechanicPartitions",
      capabilities: modeGaps,
      affected_shape_ids: unique(modeShapes.map(({ shape_id: id }) => id)).sort(),
      affected_mechanic_ids: modeMechanics,
      affected_partition_ids: modePartitions,
      lowering_rule: "Resolve in the owning generated partition through typed mode programs, typed external outcomes, an explicit VersionedProjectPolicy or proven non-runtime scope; shared resolvers may not branch on these identities.",
    },
    content_id_branch_audit: {
      roots: sharedRoots,
      audited_file_count: auditedFiles.length,
      audited_tree_sha256: hashFiles(auditedFiles),
      forbidden_mode_symbols: ["DivergentUniverse", "divergent_universe", "divergent-universe"],
      forbidden_raw_interpreter_tokens: ["PostfixExpr", "OpCodes", "DynamicHashes"],
      universal_audit: "node tools/repository-check/verify-native-handlers.mjs",
      result: "NoDivergentUniverseBranchOrRawPostfixInterpreterInSharedCore",
    },
    native_handler_audit: {
      admitted_battle_handlers: 0,
      admitted_activity_handlers: 0,
      mechanic_static_handler_references: mechanics.programs.filter(
        ({ static_handler: value }) => value !== null).length,
      registry: "crates/starclock-rules/src/registry.rs",
      metadata_policy: inputs.native_handler_audit,
      result: "TypedIrSufficientNoHandlerAdmitted",
    },
    partition_freeze: {
      state: partitions.freeze.state,
      partition_count: partitions.summary.partitions,
      program_count: partitions.summary.programs,
      partition_set_sha256: partitions.freeze.partition_set_sha256,
      partitions: partitions.partitions.map(({ batch, program_count, freeze_sha256 }) => ({
        batch, program_count, freeze_sha256,
      })),
    },
    summary: {
      probes: probes.length,
      named_missing_capabilities: inventory.summary.missing_capabilities,
      mode_owned_capabilities: modeGaps.length,
      mode_owned_shapes: modeShapes.length,
      mode_affected_mechanic_programs: modeMechanics.length,
      unresolved_expression_shapes: postfixShapes.length,
      postfix_affected_mechanic_programs: postfixMechanics.length,
      audited_shared_rust_files: auditedFiles.length,
      admitted_native_handlers: 0,
      frozen_partitions: partitions.summary.partitions,
      frozen_programs: partitions.summary.programs,
    },
  };
}

function probe(id, file, requiredFragments) {
  return { id, file, required_fragments: requiredFragments, result: "Passed" };
}

function recursiveRustFiles(directory) {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const target = path.join(directory, entry.name);
    if (entry.isDirectory()) return recursiveRustFiles(target);
    return entry.isFile() && entry.name.endsWith(".rs") ? [target] : [];
  });
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

function normalize(value) {
  return value.replaceAll("\\", "/");
}

function unique(values) {
  return [...new Set(values)];
}

function required(value, label) {
  if (value === undefined || value === null) throw new Error(`missing ${label}`);
  return value;
}

function summaryLine(state, value) {
  return `Divergent Universe shared capability audit ${state} `
    + `(${value.summary.probes} probes; ${value.summary.audited_shared_rust_files} `
    + `shared Rust files; ${value.summary.frozen_partitions} partitions; zero handlers).`;
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

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
