import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-universe-path-runtime.json");
assert(policy.schema_revision === "starclock.goal04-universe-path-runtime.v1", "unexpected Path-runtime policy revision");
const runtime = text("crates/starclock-mode-universe/src/path_runtime.rs");
const topology = text("crates/starclock-mode-universe/src/topology.rs");
const entry = text("crates/starclock-mode-universe/src/entry.rs");
const facade = text("crates/starclock-mode-universe/src/runtime.rs");
const tests = text("crates/starclock-mode-universe/tests/path_runtime.rs");

for (const marker of [
  "PathPassiveContribution", "PathContributionSet", "ResonanceRuleContribution",
  "ResonanceActionState", "can_activate", "formation_selection_options",
  "FORMATION_SELECTION_THRESHOLDS", "PathRuntimeCatalog"
]) assert(runtime.includes(marker), `Path runtime omits ${marker}`);
for (const marker of [
  "formation_node", "formation_inventory", "path_blessing_count_slot",
  "ActivityOperation::AddCounter", "ActivityDecisionKind::Choice"
]) assert(topology.includes(marker), `Path topology integration omits ${marker}`);
assert(entry.includes("PATH_RUNTIME_REVISION") && entry.includes("FORMATION_INVENTORY"), "entry identity/state omits Path runtime");
assert(facade.includes("path_contributions"), "runtime facade omits Path contributions");
assert(!/f32|f64|HashMap/.test(runtime + topology + entry), "Path runtime contains floats or unordered state");
assert((tests.match(/^#\[test\]$/gm) ?? []).length === policy.focused_tests, "Path focused test count differs");

const catalogGolden = arrayGolden(tests, /path_runtime\.digest\(\),\s*\[([\s\S]*?)\]\s*\)/);
const contributionGolden = arrayGolden(tests, /complete\.digest\(\),\s*\[([\s\S]*?)\]\s*\)/);
const entryGolden = arrayGolden(text("crates/starclock-mode-universe/tests/entry_compilation.rs"), /base\.identity\(\)\.definition_digest\(\)\.bytes\(\),\s*\[([\s\S]*?)\]\s*\)/);
const graphGolden = arrayGolden(text("crates/starclock-mode-universe/tests/topology_runtime.rs"), /runtime\.graph\(\)\.digest\(\)\.bytes\(\),\s*\[([\s\S]*?)\]\s*\)/);
assert(catalogGolden === policy.goldens.runtime_catalog_sha256, "Path runtime-catalog golden differs");
assert(contributionGolden === policy.goldens.representative_complete_contribution_sha256, "Path contribution golden differs");
assert(entryGolden === policy.goldens.entry_definition_sha256, "Path-aware entry golden differs");
assert(graphGolden === policy.goldens.activity_graph_sha256, "Path-aware graph golden differs");
const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P3-B5` \| `(InProgress|Complete)` \|/m.test(status), "G04-P3-B5 is not active or complete");

const evidence = {
  schema_revision: "starclock.goal04-universe-path-runtime-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "path-passive-resonance-energy-action-and-formation-selection-runtime-is-deterministic",
  counts: policy.counts,
  policies: policy.policies,
  goldens: policy.goldens,
  deferred_scope: policy.scope,
  compatibility: {
    generated_rows_public: false,
    mutable_mode_engine_added: false,
    formation_selection_uses_activity_apply: true,
    authoritative_float: false,
    scattered_path_id_branching: false
  },
  source_sha256: Object.fromEntries([
    "crates/starclock-mode-universe/src/path_runtime.rs",
    "crates/starclock-mode-universe/src/topology.rs",
    "crates/starclock-mode-universe/src/entry.rs",
    "crates/starclock-mode-universe/src/runtime.rs",
    "crates/starclock-mode-universe/tests/path_runtime.rs",
    "crates/starclock-mode-universe/tests/encounter_runtime.rs",
    "crates/starclock-mode-universe/tests/entry_compilation.rs",
    "crates/starclock-mode-universe/tests/topology_runtime.rs"
  ].map((relative) => [relative, sha256(relative)])),
  focused_tests: policy.focused_tests,
  new_registry_packages: policy.new_registry_packages
};
const relative = "evidence/standard-universe-runtime-v1/profile/universe-path-runtime.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "Universe Path-runtime evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "Universe Path-runtime evidence is stale; run with --bless");
}
console.log(`Goal 04 Universe Path runtime verified (${policy.counts.paths} Paths, ${policy.counts.resonances} Resonances, ${policy.counts.formations} Formations).`);

function arrayGolden(sourceText, pattern) {
  const match = sourceText.match(pattern);
  assert(match !== null, "expected Rust byte-array golden is missing");
  return match[1].split(",").map((value) => value.trim()).filter(Boolean).map((value) => Number.parseInt(value, 10).toString(16).padStart(2, "0")).join("");
}
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
