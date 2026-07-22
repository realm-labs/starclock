import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-activity-state.json");
assert(policy.schema_revision === "starclock.goal04-activity-state.v1", "unexpected Activity state policy revision");
const state = text("crates/starclock-activity/src/state_definition.rs");
const slot = text("crates/starclock-activity/src/slot.rs");
const spec = text("crates/starclock-activity/src/spec.rs");
const tests = text("crates/starclock-activity/tests/state_definition.rs");
const boundaryTests = text("crates/starclock-activity/tests/activity_boundary.rs");

for (const marker of [
  "pub struct ActivityScopePath",
  "pub enum ActivityScopeIdentity",
  "pub enum SlotCarryPolicy",
  "pub struct ActivityInventoryDefinition",
  "pub struct ActivityModifierDefinition",
  "pub enum ActivityModifierOwner",
  "pub struct ActivityStateDefinition"
]) assert(state.includes(marker), `Activity state model omits ${marker}`);
for (const marker of [
  "BoundedCounterMap",
  "pub fn new_with_policy",
  "MAX_SLOT_COLLECTION_ENTRIES",
  "SnapshotBeforeOwnerExit"
]) assert(slot.includes(marker), `typed slot model omits ${marker}`);
assert(!/HashMap|f32|f64/.test(state + slot), "Activity state uses an unbounded map or authoritative float");
for (const [constant, value] of [
  ["MAX_ACTIVITY_STATE_SLOTS", policy.bounds.maximum_slots],
  ["MAX_ACTIVITY_INVENTORIES", policy.bounds.maximum_inventories],
  ["MAX_ACTIVITY_MODIFIERS", policy.bounds.maximum_modifiers],
  ["MAX_INVENTORY_ENTRIES", policy.bounds.maximum_inventory_entries],
  ["MAX_INVENTORY_STACK", policy.bounds.maximum_stack]
]) assert(new RegExp(`const ${constant}: [^=]+ = ${numberLiteral(value)};`).test(state), `${constant} differs from policy`);
assert(new RegExp(`const MAX_SLOT_COLLECTION_ENTRIES: [^=]+ = ${numberLiteral(policy.bounds.maximum_slot_collection_entries)};`).test(slot), "slot collection bound differs");
assert(state.includes("slots.sort_by_key") && state.includes("inventories.sort_by_key") && state.includes("modifiers.sort_by_key"), "state definitions do not canonicalize order");
assert(state.includes("MissingModifierInventory") && state.includes("binary_search_by_key"), "modifier inventory ownership is not reference-closed");
assert(spec.includes("state: ActivityStateDefinition") && spec.includes("ActivityStateDefinition::new(slots"), "legacy Activity spec is not backed by the generic state definition");
assert((tests.match(/^#\[test\]$/gm) ?? []).length === policy.focused_tests, "Activity state focused test count differs");
for (const legacy of ["194, 66, 33, 96", "237, 237, 61, 51", "89, 89, 16, 147"])
  assert(boundaryTests.includes(legacy), `legacy one-battle hash golden changed: ${legacy}`);
const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P2-B2` \| `(InProgress|Complete)` \|/m.test(status), "G04-P2-B2 is not active or complete");

const evidence = {
  schema_revision: "starclock.goal04-activity-state-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "typed-scoped-state-contracts-implemented",
  bounds: policy.bounds,
  shape: policy.shape,
  compatibility: {
    goal01_slot_constructor_retained: true,
    goal01_state_hash_goldens_unchanged: true,
    mutable_command_transaction_store_deferred: "G04-P2-B3",
    generated_rows_public: false,
    authoritative_float: false
  },
  source_sha256: {
    state_definition: sha256("crates/starclock-activity/src/state_definition.rs"),
    slot: sha256("crates/starclock-activity/src/slot.rs"),
    tests: sha256("crates/starclock-activity/tests/state_definition.rs")
  },
  focused_tests: policy.focused_tests,
  new_registry_packages: policy.new_registry_packages
};
const relative = "evidence/standard-universe-runtime-v1/activity/state-definitions.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "Activity state evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "Activity state evidence is stale; run with --bless");
}
console.log(`Goal 04 Activity state verified (${policy.bounds.maximum_slots} slots, ${policy.bounds.maximum_inventories} inventories, ${policy.bounds.maximum_modifiers} modifiers).`);

function numberLiteral(value) { return String(value).replace(/\B(?=(\d{3})+(?!\d))/g, "_"); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
