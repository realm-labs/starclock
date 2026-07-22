import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-universe-run-runtime.json");
assert(policy.schema_revision === "starclock.goal04-universe-run-runtime.v1", "unexpected run-runtime policy revision");
const runtime = text("crates/starclock-mode-universe/src/run_runtime.rs");
const entry = text("crates/starclock-mode-universe/src/entry.rs");
const facade = text("crates/starclock-mode-universe/src/runtime.rs");
const topology = text("crates/starclock-mode-universe/src/topology.rs");
const activity = text("crates/starclock-activity/src/graph_activity.rs");
const tests = text("crates/starclock-mode-universe/tests/run_runtime.rs");

for (const marker of [
  "RunRuntimeCatalog", "OccurrenceRuntimeChoice", "ServiceRuntimeDefinition",
  "AbilityTreeRuleContribution", "CosmicFragments", "credit_fragments", "spend_fragments"
]) assert(runtime.includes(marker), `run runtime omits ${marker}`);
for (const marker of [
  "RUN_RUNTIME_REVISION", "COSMIC_FRAGMENTS_SLOT", "EXTERNAL_OUTCOME_SLOT",
  "run_runtime: Arc<RunRuntimeCatalog>", "abstract_interactions"
]) assert(entry.includes(marker), `entry identity/state omits ${marker}`);
for (const marker of ["cosmic_fragments", "ability_tree_contributions", "submit_external_outcome"])
  assert(facade.includes(marker), `runtime facade omits ${marker}`);
for (const marker of ["AbstractInteractionBinding", "ActivityDecisionKind::ExternalOutcome", "content_formation"])
  assert(topology.includes(marker), `topology interaction seam omits ${marker}`);
assert(activity.includes("pub fn submit_external_outcome("), "generic graph Activity omits external-outcome mutation boundary");
assert(!/f32|f64|HashMap/.test(runtime + entry + topology), "run runtime contains floats or unordered state");
assert((tests.match(/^#\[test\]$/gm) ?? []).length === policy.focused_tests, "run-runtime focused test count differs");
for (const marker of [
  "all_occurrence_service_and_ability_inputs_compile_to_typed_runtime",
  "cosmic_fragment_credit_and_spend_are_checked_atomic_activity_operations",
  "noncombat_rooms_accept_only_offered_external_outcomes_without_granting_battle_rewards"
]) assert(tests.includes(marker), `run-runtime tests omit ${marker}`);
const numericTests = tests.replace(/(?<=\d)_(?=\d)/g, "");
assert(numericTests.includes(`runtime.occurrence_choices().len(), ${policy.counts.occurrence_choices}`), "Occurrence-choice denominator differs");
assert(numericTests.includes(`runtime.services().len(), ${policy.counts.services}`), "service denominator differs");
const topologyTests = text("crates/starclock-mode-universe/tests/topology_runtime.rs").replace(/(?<=\d)_(?=\d)/g, "");
assert(topologyTests.includes(`compiled.abstract_interactions().len(), ${policy.counts.abstract_interaction_bindings}`), "abstract-interaction denominator differs");
const runtimeGolden = arrayGolden(tests, /runtime\.digest\(\),\s*\[([\s\S]*?)\]\s*\)/);
const abilityGolden = arrayGolden(tests, /abilities\.digest\(\),\s*\[([\s\S]*?)\]\s*\)/);
const entryGolden = arrayGolden(text("crates/starclock-mode-universe/tests/entry_compilation.rs"), /base\.identity\(\)\.definition_digest\(\)\.bytes\(\),\s*\[([\s\S]*?)\]\s*\)/);
const graphGolden = arrayGolden(text("crates/starclock-mode-universe/tests/topology_runtime.rs"), /runtime\.graph\(\)\.digest\(\)\.bytes\(\),\s*\[([\s\S]*?)\]\s*\)/);
assert(runtimeGolden === policy.goldens.runtime_catalog_sha256, "run runtime-catalog golden differs");
assert(abilityGolden === policy.goldens.representative_ability_contribution_sha256, "Ability Tree contribution golden differs");
assert(entryGolden === policy.goldens.entry_definition_sha256, "run-aware entry golden differs");
assert(graphGolden === policy.goldens.activity_graph_sha256, "run-aware graph golden differs");
const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P3-B7` \| `(InProgress|Complete)` \|/m.test(status), "G04-P3-B7 is not active or complete");

const evidence = {
  schema_revision: "starclock.goal04-universe-run-runtime-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "typed-run-inputs-fragments-and-external-interactions-are-deterministic",
  counts: policy.counts,
  policies: policy.policies,
  goldens: policy.goldens,
  deferred_scope: policy.scope,
  compatibility: {
    generated_rows_public: false,
    mutable_mode_engine_added: false,
    authoritative_float: false,
    noncombat_room_grants_battle_reward: false,
    arbitrary_external_outcome_accepted: false
  },
  source_sha256: Object.fromEntries([
    "crates/starclock-activity/src/graph_activity.rs",
    "crates/starclock-mode-universe/src/run_runtime.rs",
    "crates/starclock-mode-universe/src/entry.rs",
    "crates/starclock-mode-universe/src/runtime.rs",
    "crates/starclock-mode-universe/src/topology.rs",
    "crates/starclock-mode-universe/tests/run_runtime.rs",
    "crates/starclock-mode-universe/tests/entry_compilation.rs",
    "crates/starclock-mode-universe/tests/topology_runtime.rs"
  ].map((relative) => [relative, sha256(relative)])),
  focused_tests: policy.focused_tests,
  new_registry_packages: policy.new_registry_packages
};
const relative = "evidence/standard-universe-runtime-v1/profile/universe-run-runtime.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "Universe run-runtime evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "Universe run-runtime evidence is stale; run with --bless");
}
console.log(`Goal 04 Universe run runtime verified (${policy.counts.occurrence_choices} choices, ${policy.counts.services} services, ${policy.counts.abstract_interaction_bindings} interaction bindings).`);

function arrayGolden(sourceText, pattern) {
  const match = sourceText.match(pattern);
  assert(match !== null, "expected Rust byte-array golden is missing");
  return match[1].split(",").map((value) => value.trim()).filter(Boolean).map((value) => Number.parseInt(value, 10).toString(16).padStart(2, "0")).join("");
}
function text(relativePath) { return fs.readFileSync(path.join(root, relativePath), "utf8"); }
function json(relativePath) { return JSON.parse(text(relativePath)); }
function sha256(relativePath) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relativePath))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
