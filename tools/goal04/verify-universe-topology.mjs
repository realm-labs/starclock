import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-universe-topology.json");
assert(policy.schema_revision === "starclock.goal04-universe-topology.v1", "unexpected Universe-topology policy revision");
const topology = text("crates/starclock-mode-universe/src/topology.rs");
const entry = text("crates/starclock-mode-universe/src/entry.rs");
const runtime = text("crates/starclock-activity/src/graph_activity.rs");
const program = text("crates/starclock-activity/src/program.rs");
const transaction = text("crates/starclock-activity/src/transaction.rs");
const tests = text("crates/starclock-mode-universe/tests/topology_runtime.rs");

for (const marker of [
  "pub struct DomainHubDefinition", "pub struct DomainRouteDefinition",
  "ActivityBootstrapSelection::new", "ActivityRngLabel::Graph",
  "ActivityDecisionKind::Checkpoint", "ActivityDecisionKind::Route",
  "ActivityExpression::CounterValue", "ActivityOperation::AddCounter",
  "room_is_eligible", "GraphActivityDefinition::new"
]) assert(topology.includes(marker), `topology compiler omits ${marker}`);
for (const marker of [
  "pub struct GraphActivityDefinition", "pub struct GraphActivity",
  "pub fn start(", "pub fn choose_option(", "fn pump(",
  "new_with_initial_values"
]) assert(runtime.includes(marker) || transaction.includes(marker), `generic graph Activity omits ${marker}`);
assert(entry.includes("runtime: Arc<GraphActivityDefinition>") && entry.includes("GraphActivity::start"), "compiled entry does not retain/start the generic graph definition");
assert(program.includes("OptionalId = 4") && program.includes("CounterValue"), "typed program IR cannot express Path selection and hub gates");
assert(!/position_[xy]|f32|f64|HashMap/.test(topology + runtime), "topology runtime contains spatial coordinates, floats or unordered state");
assert((tests.match(/^#\[test\]$/gm) ?? []).length === policy.focused_tests, "topology focused test count differs");
for (const marker of [
  "all_topologies_compile_to_bounded_spatial_free_hubs",
  "start_draws_one_topology_and_offers_nine_paths_without_leaking_private_state",
  "mandatory_interaction_consumption_gates_routes_and_a_seeded_graph_terminates",
  "stale_and_unoffered_hub_commands_preserve_exact_state_and_rng",
  "topology_draw_is_reproducible_for_the_same_seed_and_identity"
]) assert(tests.includes(marker), `topology tests omit ${marker}`);
for (const [needle, expected] of [["runtime.graph().nodes().len(),", policy.counts.activity_nodes], ["runtime.graph().edges().len(),", policy.counts.activity_edges], ["compiled.domain_hubs().len(),", policy.counts.domain_hubs]])
  assert(tests.includes(`${needle} ${expected}`), `topology count assertion differs for ${needle}`);
const graphGolden = arrayGolden(tests, /runtime\.graph\(\)\.digest\(\)\.bytes\(\),\s*\[([\s\S]*?)\]\s*\)/);
const stateGolden = arrayGolden(tests, /view\.state_hash\(\)\.bytes\(\),\s*\[([\s\S]*?)\]\s*\)/);
assert(graphGolden === policy.goldens.graph_sha256, "topology graph golden differs");
assert(stateGolden === policy.goldens.seed7_initial_state_sha256, "topology initial-state golden differs");
const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P3-B2` \| `(InProgress|Complete)` \|/m.test(status), "G04-P3-B2 is not active or complete");

const evidence = {
  schema_revision: "starclock.goal04-universe-topology-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "all-source-topologies-compile-to-bounded-spatial-free-domain-hubs",
  counts: policy.counts,
  policies: policy.policies,
  goldens: policy.goldens,
  deferred_scope: policy.scope,
  compatibility: {
    generated_rows_public: false,
    mutable_mode_engine_added: false,
    coordinates_hashed: false,
    invalid_command_mutates_state_or_rng: false
  },
  source_sha256: {
    topology: sha256("crates/starclock-mode-universe/src/topology.rs"),
    entry: sha256("crates/starclock-mode-universe/src/entry.rs"),
    graph_activity: sha256("crates/starclock-activity/src/graph_activity.rs"),
    program: sha256("crates/starclock-activity/src/program.rs"),
    transaction: sha256("crates/starclock-activity/src/transaction.rs"),
    tests: sha256("crates/starclock-mode-universe/tests/topology_runtime.rs")
  },
  focused_tests: policy.focused_tests,
  new_registry_packages: policy.new_registry_packages
};
const relative = "evidence/standard-universe-runtime-v1/profile/universe-topology.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "Universe-topology evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "Universe-topology evidence is stale; run with --bless");
}
console.log(`Goal 04 Universe topology verified (${policy.counts.topology_candidates} templates, ${policy.counts.domain_hubs} hubs, ${policy.counts.activity_edges} edges).`);

function arrayGolden(sourceText, pattern) {
  const match = sourceText.match(pattern);
  assert(match !== null, "expected Rust byte-array golden is missing");
  return match[1].split(",").map((value) => value.trim()).filter(Boolean).map((value) => Number.parseInt(value, 10).toString(16).padStart(2, "0")).join("");
}
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
