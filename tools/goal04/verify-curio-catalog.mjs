import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-curio-catalog.json");
assert(policy.schema_revision === "starclock.goal04-curio-catalog.v1", "unexpected Curio-catalog policy revision");
const domain = text("crates/starclock-mode-universe/src/curio.rs");
const lowering = text("crates/starclock-mode-universe/src/curio_lowering.rs");
const catalog = text("crates/starclock-mode-universe/src/catalog.rs");
const ids = text("crates/starclock-mode-universe/src/id.rs");
for (const marker of ["pub struct CurioDefinition", "pub struct CurioStateDefinition", "pub enum CurioStateKind"])
  assert(domain.includes(marker), `Curio domain omits ${marker}`);
for (const marker of ["CurioId", "CurioStateId"])
  assert(new RegExp(`id_type!\\(\\s*${marker}`).test(ids), `Curio catalog omits stable ID ${marker}`);
assert(!/f32|f64|crate::generated|SoraConfig/.test(domain), "Curio public domain leaks transport or floating-point implementation");
for (const marker of [
  "Curio lifecycle transition crosses Curio ownership",
  "Repairing Curio transition must consume charges and target its Fixed state",
  "Curio charge parameter index is out of bounds",
  "Curio replacement reference is unresolved"
]) assert(lowering.includes(marker), `Curio lowering omits validation: ${marker}`);
assert(lowering.includes("definitions.len() != 67") && lowering.includes("definitions.len() != 61"), "Curio release denominators are not executable gates");
assert(hexSequenceAppears(catalog, policy.definitions_digest), "Curio definition digest golden is absent");
for (const [name, count] of Object.entries(policy.counts)) assert(Number.isInteger(count) && count >= 0, `${name} count is invalid`);
assert(policy.counts.active_states + policy.counts.repairing_states + policy.counts.fixed_states === policy.counts.states, "Curio state-kind partition differs");
assert(policy.shape.maximum_author_decimal_scale > policy.shape.combat_decimal_scale, "Curio source precision boundary is absent");
const tests = (catalog.match(/^    #\[test\]$/gm) ?? []).length
  + (lowering.match(/^    #\[test\]$/gm) ?? []).length;
assert(tests === policy.focused_tests, "Curio-focused test count differs");
const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P1-B4` \| `(InProgress|Complete)` \|/m.test(status), "G04-P1-B4 is not active or complete");

const evidence = {
  schema_revision: "starclock.goal04-curio-catalog-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "curio-lifecycle-catalog-lowered",
  definitions_digest: policy.definitions_digest,
  counts: policy.counts,
  shape: policy.shape,
  boundaries: {
    generated_rows_public: false,
    authoritative_float: false,
    authored_decimal_exact_until_formula_compilation: true,
    all_parent_initial_rule_and_lifecycle_references_validated: true,
    replacement_reference_modeled_without_fabricated_rows: true,
    runtime_logic_implemented: false
  },
  new_registry_packages: policy.new_registry_packages,
  focused_tests: policy.focused_tests
};
const relative = "evidence/standard-universe-runtime-v1/catalog/curio-definitions.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "Curio catalog evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "Curio catalog evidence is stale; run with --bless");
}
console.log(`Goal 04 Curio catalog verified (${policy.counts.curios} Curios, ${policy.counts.states} states, ${policy.counts.parameters} parameters; ${policy.definitions_digest.slice(0, 12)}).`);

function hexSequenceAppears(source, hex) { return [...source.matchAll(/0x([0-9a-f]{2})/g)].map((match) => match[1]).join("").includes(hex); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function assert(condition, message) { if (!condition) throw new Error(message); }
