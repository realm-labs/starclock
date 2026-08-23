#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const inventoryPath = path.join(
  root, "content-manifests/currency-wars-runtime-v1/capability-inventory.json",
);
const sourceCacheIndex = process.argv.indexOf("--source-cache");
const sourceCache = sourceCacheIndex === -1
  ? ".cache/content-reference/turnbasedgamedata"
  : process.argv[sourceCacheIndex + 1];

execFileSync("node", [
  "tools/currency-wars-runtime/generate-capability-inventory.mjs",
  "--check", "--source-cache", sourceCache,
], { cwd: root, stdio: "inherit" });

const inventory = JSON.parse(fs.readFileSync(inventoryPath, "utf8"));
assert(inventory.batch === "G21-P2-B1", "capability inventory batch drift");
assert(inventory.programs.length === 2_367, "mechanic inventory denominator drift");
assert(unique(inventory.programs.map(({ mechanic_id: id }) => id)) === 2_367,
  "mechanic inventory is not exact-once");
assert(inventory.summary.executable_programs + inventory.summary.metadata_only_programs === 2_367,
  "executable/metadata program accounting drift");
assert(inventory.summary.unique_source_files === 995, "source file denominator drift");
assert(inventory.summary.configuration_types > 0
  && inventory.summary.configuration_type_shapes > 0
  && inventory.summary.expression_shapes > 0
  && inventory.summary.selector_shapes > 0
  && inventory.summary.trigger_shapes > 0
  && inventory.summary.state_shapes > 0
  && inventory.summary.lifecycle_shapes > 0,
"one or more required capability inventories are empty");
assert(inventory.postfix_opcode_bytes.length > 0
  && inventory.postfix_opcode_bytes.every(({ semantic_status: status }) =>
    status === "UnresolvedExactByte"),
"unverified postfix opcode semantics were claimed");

const shapeGroups = [
  inventory.configuration_type_shapes,
  inventory.expression_shapes,
  inventory.selector_shapes,
  inventory.trigger_shapes,
  inventory.state_shapes,
  inventory.lifecycle_shapes,
  inventory.record_shapes,
];
const shapes = shapeGroups.flat();
assert(unique(shapes.map(({ shape_id: id }) => id)) === shapes.length,
  "capability shape identity is not unique");
assert(shapes.every(({ mapping }) =>
  ["ExistingPrimitive", "MissingCapability", "NonAuthoritative"]
    .includes(mapping.disposition)
  && Array.isArray(mapping.existing_support)
  && (mapping.disposition !== "MissingCapability" || mapping.missing_capability !== null)),
"capability shape has no terminal support mapping");

const shapeIds = new Set(shapes.map(({ shape_id: id }) => id));
for (const program of inventory.programs) {
  assert(shapeIds.has(program.record_shape_id),
    `program record shape missing: ${program.mechanic_id}`);
  const assigned = Object.values(program.extracted_shape_counts)
    .reduce((total, count) => total + count, 0);
  assert(/^[0-9a-f]{64}$/.test(program.extracted_shape_set_sha256),
    `program shape-set digest is invalid: ${program.mechanic_id}`);
  if (program.target_execution === "MetadataOnly")
    assert(assigned === 0,
      `metadata-only program acquired executable shapes: ${program.mechanic_id}`);
}

const missing = new Set(inventory.missing_capabilities.map(({ capability }) => capability));
assert(missing.has("shared.version-4.4-postfix-opcode-semantics"),
"shared-capability gap inventory is incomplete");
assert([...missing].every((capability) => !capability.startsWith("activity.")),
  "P2-B2 left a shared Activity capability unresolved");
assert([...missing].every((capability) => !capability.startsWith("combat.")),
  "P2-B3 left a shared combat capability unresolved");
assert([...missing].every((capability) => !capability.startsWith("build.")),
  "P2-B4 left a shared Build capability unresolved");
verifyActivityCapabilitySurface();
verifyCombatCapabilitySurface();
verifyBuildCapabilitySurface();

console.log(
  `Currency Wars capability inventory verified (${inventory.summary.configuration_types} types; `
    + `${inventory.summary.postfix_opcode_sequences} postfix sequences; `
    + `${inventory.summary.missing_capabilities} named gaps).`,
);

function unique(values) {
  return new Set(values).size;
}

function verifyCombatCapabilitySurface() {
  const selector = fs.readFileSync(
    path.join(root, "crates/starclock-combat/src/catalog/selector.rs"), "utf8",
  );
  const effect = fs.readFileSync(
    path.join(root, "crates/starclock-combat/src/effect/model.rs"), "utf8",
  );
  const rule = fs.readFileSync(
    path.join(root, "crates/starclock-combat/src/rule/model.rs"), "utf8",
  );
  const required = [
    [selector, "with_candidate_union"],
    [effect, "with_hp_floor"],
    [rule, "ModifySkillPointMaximum"],
  ];
  for (const [source, fragment] of required)
    assert(source.includes(fragment), `shared combat capability is missing: ${fragment}`);
}

function verifyActivityCapabilitySurface() {
  const program = fs.readFileSync(
    path.join(root, "crates/starclock-activity/src/program.rs"), "utf8",
  );
  const required = [
    "pub enum ActivityComparison",
    "CounterEntryCount(ActivitySlotId)",
    "OrderedIdSetCount(ActivitySlotId)",
    "InventoryEntryCount(ActivityInventoryId)",
    "ModifierStacks(ActivityModifierId)",
    "SetCounter {",
    "RemoveOrderedId {",
    "SetInventoryCount {",
    "SetModifierStacks {",
  ];
  for (const fragment of required)
    assert(program.includes(fragment), `shared Activity capability is missing: ${fragment}`);
}

function verifyBuildCapabilitySurface() {
  const contribution = fs.readFileSync(
    path.join(root, "crates/starclock-build/src/contribution.rs"), "utf8",
  );
  const spec = fs.readFileSync(
    path.join(root, "crates/starclock-build/src/spec.rs"), "utf8",
  );
  const compiler = fs.readFileSync(
    path.join(root, "crates/starclock-build/src/compiler.rs"), "utf8",
  );
  const required = [
    [contribution, "pub enum BuildContributionApplicability"],
    [contribution, "pub struct BuildContributionDefinition"],
    [spec, "with_contributions"],
    [compiler, "apply_contributions"],
    [compiler, "BuildSourceOwner::Contribution"],
  ];
  for (const [source, fragment] of required)
    assert(source.includes(fragment), `shared Build capability is missing: ${fragment}`);
}

function assert(condition, message) {
  if (!condition)
    throw new Error(message);
}
