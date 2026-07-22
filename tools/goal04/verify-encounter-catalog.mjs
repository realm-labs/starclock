import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-encounter-catalog.json");
assert(policy.schema_revision === "starclock.goal04-encounter-catalog.v1", "unexpected encounter-catalog policy revision");
const domain = text("crates/starclock-mode-universe/src/encounter.rs");
const lowering = text("crates/starclock-mode-universe/src/encounter_lowering.rs");
const rule = text("crates/starclock-mode-universe/src/rule.rs");
const ruleLowering = text("crates/starclock-mode-universe/src/rule_lowering.rs");
const catalog = text("crates/starclock-mode-universe/src/catalog.rs");
for (const marker of ["EncounterGroupDefinition", "EncounterMemberDefinition", "EncounterPoolDefinition", "RoomContentBinding", "ContentPoolDefinition"])
  assert(domain.includes(`pub struct ${marker}`), `encounter domain omits ${marker}`);
assert(rule.includes("pub struct MechanicRuleDefinition") && rule.includes("pub struct MechanicParameter"), "mechanic contribution domain is incomplete");
assert(!/f32|f64|crate::generated|SoraConfig/.test(domain + rule), "encounter public domains leak transport or floating point");
for (const marker of [
  "Encounter group/member/wave/enemy denominator differs",
  "Content pool content key is unresolved for its kind",
  "Room content kind/group contract is inconsistent",
  "Content evidence provenance reference is unresolved"
]) assert(lowering.includes(marker), `encounter lowering omits ${marker}`);
assert(ruleLowering.includes("Mechanic rule content-record reference is unresolved"), "mechanic-to-content evidence reference is not validated");
assert(hexSequenceAppears(catalog, policy.definitions_digest), "encounter definition digest golden is absent");

const enemyReferences = new Set([
  ...debugValues("UniverseEncounterWaveEnemy", "enemy_variant_stable_key"),
  ...debugValues("UniverseDifficultyEnemy", "enemy_variant_stable_key")
]);
const referenceEnemies = new Set(json("content-reference/v4.4/enemy-variants.json").map((row) => row.id));
for (const key of enemyReferences) assert(referenceEnemies.has(key), `Universe enemy reference is absent from public reference catalog: ${key}`);
const goal01 = new Set(json("content-manifests/core-combat-v1/standard-v1.json").enemies.map((row) => row.id));
const overlap = [...enemyReferences].filter((key) => goal01.has(key)).length;
assert(enemyReferences.size === policy.counts.unique_enemy_variant_references, "unique enemy reference denominator differs");
assert(overlap === policy.counts.goal01_enemy_overlap, "Goal 01 enemy overlap differs");
assert(enemyReferences.size - overlap === policy.counts.phase4_enemy_overlay_required, "Phase 4 enemy overlay denominator differs");
assert(policy.shape.enemy_reference_catalog_closed && !policy.shape.enemy_runtime_overlay_compiled, "enemy reference/runtime coverage distinction collapsed");
const tests = (catalog.match(/^    #\[test\]$/gm) ?? []).length + (ruleLowering.match(/^    #\[test\]$/gm) ?? []).length;
assert(tests === policy.focused_tests, "encounter-focused test count differs");
const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P1-B6` \| `(InProgress|Complete)` \|/m.test(status), "G04-P1-B6 is not active or complete");

const evidence = {
  schema_revision: "starclock.goal04-encounter-catalog-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "encounter-pool-rule-evidence-catalog-lowered",
  definitions_digest: policy.definitions_digest,
  counts: policy.counts,
  shape: policy.shape,
  boundaries: {
    generated_rows_public: false,
    authoritative_float: false,
    every_internal_parent_order_content_rule_and_provenance_reference_validated: true,
    all_enemy_keys_exist_in_frozen_public_reference_catalog: true,
    goal01_representative_enemy_catalog_not_misreported_as_complete: true,
    runtime_enemy_overlay_deferred_to_declared_mechanic_partition: true,
    runtime_logic_implemented: false
  },
  new_registry_packages: policy.new_registry_packages,
  focused_tests: policy.focused_tests
};
const relative = "evidence/standard-universe-runtime-v1/catalog/encounter-definitions.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "encounter catalog evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "encounter catalog evidence is stale; run with --bless");
}
console.log(`Goal 04 encounter catalog verified (${policy.counts.encounter_groups} groups, ${policy.counts.enemy_slots} slots, ${enemyReferences.size} enemy keys; ${policy.definitions_digest.slice(0, 12)}).`);

function debugValues(table, field) { return json(`config/universe-generated/debug-json/${table}.json`).table.rows.map((row) => Object.values(row.values[field])[0]); }
function hexSequenceAppears(source, hex) { return [...source.matchAll(/0x([0-9a-f]{2})/g)].map((match) => match[1]).join("").includes(hex); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function assert(condition, message) { if (!condition) throw new Error(message); }
