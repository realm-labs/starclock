import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-path-catalog.json");
assert(policy.schema_revision === "starclock.goal04-path-catalog.v1", "unexpected Path-catalog policy revision");
const domain = text("crates/starclock-mode-universe/src/path.rs");
const lowering = text("crates/starclock-mode-universe/src/path_lowering.rs");
const catalog = text("crates/starclock-mode-universe/src/catalog.rs");
const ids = text("crates/starclock-mode-universe/src/id.rs");
for (const marker of ["pub struct PathDefinition", "pub struct BlessingDefinition", "pub struct BlessingLevelDefinition", "pub struct ResonanceDefinition", "pub struct ExactParameter"])
  assert(domain.includes(marker), `Path domain omits ${marker}`);
for (const marker of ["PathId", "BlessingId", "BlessingLevelId", "ResonanceId"])
  assert(new RegExp(`id_type!\\(\\s*${marker}`).test(ids), `Path catalog omits stable ID ${marker}`);
assert(domain.includes("coefficient: i64") && domain.includes("scale: u8"), "exact authored decimal atom differs");
assert(!/f32|f64|crate::generated|SoraConfig/.test(domain), "Path public domain leaks transport or floating-point implementation");
assert(lowering.includes("maximum precision validated") && lowering.includes("parameter decimal precision is invalid"), "bounded exact-decimal validation is absent");
assert(lowering.includes("get_by_stable_key(key)") && lowering.includes("Path-family mechanic rule key is unresolved"), "mechanic-rule reference validation is absent");
assert(hexSequenceAppears(catalog, policy.definitions_digest), "Path definition digest golden is absent");
for (const [name, count] of Object.entries(policy.counts)) assert(Number.isInteger(count) && count > 0, `${name} count is invalid`);
assert(policy.shape.maximum_author_decimal_scale > policy.shape.combat_decimal_scale, "source precision policy no longer records explicit formula-boundary rounding");
const tests = (catalog.match(/^    #\[test\]$/gm) ?? []).length
  + (text("crates/starclock-mode-universe/src/lowering.rs").match(/^    #\[test\]$/gm) ?? []).length
  + (lowering.match(/^    #\[test\]$/gm) ?? []).length;
assert(tests === policy.focused_tests, "Path-focused test count differs");
const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P1-B3` \| `(InProgress|Complete)` \|/m.test(status), "G04-P1-B3 is not active or complete");

const evidence = {
  schema_revision: "starclock.goal04-path-catalog-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "path-blessing-resonance-catalog-lowered",
  definitions_digest: policy.definitions_digest,
  counts: policy.counts,
  shape: policy.shape,
  boundaries: {
    generated_rows_public: false,
    authoritative_float: false,
    authored_decimal_exact_until_formula_compilation: true,
    all_parent_order_and_rule_references_validated: true,
    runtime_logic_implemented: false
  },
  new_registry_packages: policy.new_registry_packages,
  focused_tests: policy.focused_tests
};
const relative = "evidence/standard-universe-runtime-v1/catalog/path-definitions.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "Path catalog evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "Path catalog evidence is stale; run with --bless");
}
console.log(`Goal 04 Path catalog verified (${policy.counts.paths} Paths, ${policy.counts.blessings} Blessings, ${policy.counts.resonances + policy.counts.formations} Resonance rows; ${policy.definitions_digest.slice(0, 12)}).`);

function hexSequenceAppears(source, hex) { return [...source.matchAll(/0x([0-9a-f]{2})/g)].map((match) => match[1]).join("").includes(hex); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function assert(condition, message) { if (!condition) throw new Error(message); }
