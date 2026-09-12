#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const inventoryPath = path.join(
  root, "content-manifests/divergent-universe-runtime-v1/capability-inventory.json",
);
const sourceCacheIndex = process.argv.indexOf("--source-cache");
const sourceCache = sourceCacheIndex === -1
  ? ".cache/content-reference/turnbasedgamedata"
  : required(process.argv[sourceCacheIndex + 1], "--source-cache value");

execFileSync("node", [
  "tools/divergent-universe-runtime/generate-capability-inventory.mjs",
  "--check", "--source-cache", sourceCache,
], { cwd: root, stdio: "inherit" });

const inventory = JSON.parse(fs.readFileSync(inventoryPath, "utf8"));
assert(inventory.schema_revision
  === "starclock.divergent-universe-capability-inventory.v1",
"capability inventory schema drift");
assert(inventory.batch === "G22-P2-B1", "capability inventory batch drift");
assert(inventory.programs.length === 669, "mechanic inventory denominator drift");
assert(unique(inventory.programs.map(({ mechanic_id: id }) => id)) === 669,
  "mechanic inventory is not exact-once");
assert(inventory.summary.executable_programs === 663
  && inventory.summary.metadata_only_programs === 3
  && inventory.summary.excluded_programs === 3,
"executable/metadata/excluded program accounting drift");
assert(inventory.summary.unique_source_files === 669, "source file denominator drift");
assert(equalJson(inventory.summary.source_scopes, {
  Activity: 662, Battle: 3, CrossBattle: 1, EvidenceLayout: 3,
}), "source scope denominator drift");
assert(equalJson(inventory.summary.domains, { Activity: 663, Metadata: 6 }),
  "runtime domain denominator drift");
assert(inventory.summary.operation_types === 180
  && inventory.summary.operation_occurrences === 7_055,
"source operation audit denominator drift");
for (const field of [
  "expression_shapes", "selector_shapes", "trigger_shapes", "operation_shapes",
  "state_shapes", "lifecycle_shapes", "record_shapes",
]) assert(inventory.summary[field] > 0, `empty required shape inventory: ${field}`);
assert(inventory.postfix_opcode_bytes.length > 0
  && inventory.postfix_opcode_bytes.every(({ semantic_status: status }) =>
    status === "UnresolvedExactByte"),
"unverified postfix opcode semantics were claimed");

const shapeGroups = {
  expressions: inventory.expression_shapes,
  selectors: inventory.selector_shapes,
  triggers: inventory.trigger_shapes,
  operations: inventory.operation_shapes,
  states: inventory.state_shapes,
  lifecycles: inventory.lifecycle_shapes,
};
const recordShapes = inventory.record_shapes;
const allShapes = [...Object.values(shapeGroups).flat(), ...recordShapes];
assert(unique(allShapes.map(({ shape_id: id }) => id)) === allShapes.length,
  "capability shape identity is not unique");
assert(allShapes.every(({ mapping }) =>
  ["ExistingPrimitive", "MissingCapability", "NonAuthoritative"]
    .includes(mapping.disposition)
  && Array.isArray(mapping.existing_support)
  && (mapping.disposition !== "MissingCapability" || mapping.missing_capability !== null)),
"capability shape has no terminal support mapping");

const shapeById = new Map(allShapes.map((shape) => [shape.shape_id, shape]));
const programMembership = new Map(allShapes.map(({ shape_id: id }) => [id, new Set()]));
for (const program of inventory.programs) {
  assert(shapeById.has(program.record_shape_id),
    `program record shape missing: ${program.mechanic_id}`);
  const assigned = [];
  for (const [group, ids] of Object.entries(program.extracted_shape_ids)) {
    assert(Object.hasOwn(shapeGroups, group),
      `unknown program shape group ${group}: ${program.mechanic_id}`);
    assert(ids.length === program.extracted_shape_counts[group],
      `program shape count drift for ${group}: ${program.mechanic_id}`);
    assert(unique(ids) === ids.length,
      `duplicate program shape for ${group}: ${program.mechanic_id}`);
    for (const id of ids) {
      assert(shapeGroups[group].some(({ shape_id: shapeId }) => shapeId === id),
        `program references wrong or missing ${group} shape: ${program.mechanic_id}`);
      programMembership.get(id).add(program.mechanic_id);
      assigned.push(id);
    }
  }
  const digest = crypto.createHash("sha256")
    .update([...new Set(assigned)].sort().join("\n")).digest("hex");
  assert(digest === program.extracted_shape_set_sha256,
    `program shape-set digest drift: ${program.mechanic_id}`);
  assert(program.operation_occurrence_count > 0,
    `program has no audited operation occurrence: ${program.mechanic_id}`);
}
for (const shapes of Object.values(shapeGroups)) {
  for (const shape of shapes)
    assert(programMembership.get(shape.shape_id).size === shape.program_count,
      `shape program-count drift: ${shape.shape_id}`);
}

const missingByCapability = new Map();
for (const shape of allShapes) {
  if (shape.mapping.disposition !== "MissingCapability") continue;
  const ids = missingByCapability.get(shape.mapping.missing_capability) ?? new Set();
  ids.add(shape.shape_id);
  missingByCapability.set(shape.mapping.missing_capability, ids);
}
const publishedMissing = new Map(inventory.missing_capabilities.map((entry) =>
  [entry.capability, entry]));
assert(publishedMissing.size === inventory.missing_capabilities.length,
  "missing capability identity is not unique");
assert(publishedMissing.size === missingByCapability.size,
  "missing capability summary denominator drift");
for (const [capability, shapeIds] of missingByCapability) {
  const published = required(publishedMissing.get(capability),
    `missing capability summary ${capability}`);
  assert(equalJson(published.shape_ids, [...shapeIds].sort()),
    `missing capability shape coverage drift: ${capability}`);
}
assert(!publishedMissing.has("shared.version-4.4-postfix-opcode-semantics"),
  "unreachable postfix metadata was retained as a runtime capability gap");
assert([...publishedMissing].every(([capability]) => !capability.startsWith("activity.")),
  "G22-P2-B2 left a shared Activity capability unresolved");
assert([...publishedMissing].some(([capability]) =>
  capability.startsWith("mode.divergent-universe.activity.")),
"mode-owned Activity lowering gaps were lost");
assert([...publishedMissing].every(([capability]) => !capability.startsWith("combat.")),
  "G22-P2-B3 left a shared Combat capability unresolved");
assert([...publishedMissing].every(([capability]) =>
  !capability.startsWith("mode.divergent-universe.combat.")),
"unreachable mode-owned Combat lowering gap was retained");
assert([...publishedMissing].every(([capability]) => !capability.startsWith("build.")),
  "unsubstantiated Build capability gap was introduced");
verifySharedCapabilitySurfaces();

console.log(
  `Divergent Universe capability inventory verified (${inventory.summary.operation_types} `
    + `operation types; ${inventory.summary.postfix_opcode_sequences} postfix sequences; `
    + `${inventory.summary.missing_capabilities} named gaps).`,
);

function verifySharedCapabilitySurfaces() {
  const activity = fs.readFileSync(
    path.join(root, "crates/starclock-activity/src/program.rs"), "utf8",
  );
  const activityState = fs.readFileSync(
    path.join(root, "crates/starclock-activity/src/state_definition.rs"), "utf8",
  );
  const combat = fs.readFileSync(
    path.join(root, "crates/starclock-combat/src/rule/model.rs"), "utf8",
  );
  const selector = fs.readFileSync(
    path.join(root, "crates/starclock-combat/src/catalog/selector.rs"), "utf8",
  );
  const requiredFragments = [
    [activity, "pub enum ActivityExpression"],
    [activity, "pub enum ActivityCondition"],
    [activity, "pub enum ActivityOperation"],
    [activity, "pub struct ActivityProgramDefinition"],
    [activityState, "pub struct ActivityStateDefinition"],
    [combat, "pub struct StateSlotDef"],
    [combat, "pub enum RuleEventPoint"],
    [combat, "pub enum ValueExpr"],
    [combat, "pub enum ConditionExpr"],
    [combat, "pub enum RuleOperationTemplate"],
    [combat, "pub struct BattleRuleDefinition"],
    [selector, "pub struct RuleUnitSelector"],
  ];
  for (const [source, fragment] of requiredFragments)
    assert(source.includes(fragment), `mapped shared capability is missing: ${fragment}`);
}

function unique(values) {
  return new Set(values).size;
}

function equalJson(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function required(value, label) {
  assert(value !== undefined && value !== null, `missing ${label}`);
  return value;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
