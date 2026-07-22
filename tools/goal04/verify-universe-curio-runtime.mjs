import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-universe-curio-runtime.json");
assert(policy.schema_revision === "starclock.goal04-universe-curio-runtime.v1", "unexpected Curio-runtime policy revision");
const runtime = text("crates/starclock-mode-universe/src/curio_runtime.rs");
const entry = text("crates/starclock-mode-universe/src/entry.rs");
const facade = text("crates/starclock-mode-universe/src/runtime.rs");
const tests = text("crates/starclock-mode-universe/tests/curio_runtime.rs");

for (const marker of [
  "CurioRuntimeCatalog", "CurioRuntimeBindings", "CurioContributionSet",
  "acquisition_option", "consume_charge_operations", "repair_operations",
  "replacement_operations", "teardown_operations", "contributions_from_owned"
]) assert(runtime.includes(marker), `Curio runtime omits ${marker}`);
for (const marker of [
  "CURIO_RUNTIME_REVISION", "CURIO_STATE_SLOT", "CURIO_CHARGE_SLOT", "CURIO_INVENTORY"
]) assert(entry.includes(marker), `entry identity/state omits ${marker}`);
assert(facade.includes("curio_contributions"), "runtime facade omits Curio contributions");
assert(!/f32|f64|HashMap/.test(runtime + entry), "Curio runtime contains floats or unordered state");
assert((tests.match(/^#\[test\]$/gm) ?? []).length === policy.focused_tests, "Curio focused test count differs");

const catalogGolden = arrayGolden(tests, /runtime\.digest\(\),\s*\[([\s\S]*?)\]\s*\)/);
const contributionGolden = arrayGolden(tests, /initial_contribution\.digest\(\),\s*\[([\s\S]*?)\]\s*\)/);
const entryGolden = arrayGolden(text("crates/starclock-mode-universe/tests/entry_compilation.rs"), /base\.identity\(\)\.definition_digest\(\)\.bytes\(\),\s*\[([\s\S]*?)\]\s*\)/);
assert(catalogGolden === policy.goldens.runtime_catalog_sha256, "Curio runtime-catalog golden differs");
assert(contributionGolden === policy.goldens.representative_repairing_contribution_sha256, "Curio contribution golden differs");
assert(entryGolden === policy.goldens.entry_definition_sha256, "Curio-aware entry golden differs");
const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P3-B6` \| `(InProgress|Complete)` \|/m.test(status), "G04-P3-B6 is not active or complete");

const evidence = {
  schema_revision: "starclock.goal04-universe-curio-runtime-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "curio-ownership-charge-repair-replacement-teardown-and-contributions-are-deterministic",
  counts: policy.counts,
  policies: policy.policies,
  goldens: policy.goldens,
  deferred_scope: policy.scope,
  compatibility: {
    generated_rows_public: false,
    mutable_mode_engine_added: false,
    mutations_use_activity_operations: true,
    authoritative_float: false,
    scattered_curio_id_branching: false
  },
  source_sha256: Object.fromEntries([
    "crates/starclock-mode-universe/src/curio_runtime.rs",
    "crates/starclock-mode-universe/src/entry.rs",
    "crates/starclock-mode-universe/src/runtime.rs",
    "crates/starclock-mode-universe/tests/curio_runtime.rs",
    "crates/starclock-mode-universe/tests/encounter_runtime.rs",
    "crates/starclock-mode-universe/tests/entry_compilation.rs",
    "crates/starclock-mode-universe/tests/topology_runtime.rs"
  ].map((relative) => [relative, sha256(relative)])),
  focused_tests: policy.focused_tests,
  new_registry_packages: policy.new_registry_packages
};
const relative = "evidence/standard-universe-runtime-v1/profile/universe-curio-runtime.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "Universe Curio-runtime evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "Universe Curio-runtime evidence is stale; run with --bless");
}
console.log(`Goal 04 Universe Curio runtime verified (${policy.counts.curios} Curios, ${policy.counts.states} states).`);

function arrayGolden(sourceText, pattern) {
  const match = sourceText.match(pattern);
  assert(match !== null, "expected Rust byte-array golden is missing");
  return match[1].split(",").map((value) => value.trim()).filter(Boolean).map((value) => Number.parseInt(value, 10).toString(16).padStart(2, "0")).join("");
}
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
