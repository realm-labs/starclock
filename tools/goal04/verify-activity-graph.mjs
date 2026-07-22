import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-activity-graph.json");
assert(policy.schema_revision === "starclock.goal04-activity-graph.v1", "unexpected Activity graph policy revision");

const graph = text("crates/starclock-activity/src/graph.rs");
const codec = text("crates/starclock-activity/src/codec.rs");
const spec = text("crates/starclock-activity/src/spec.rs");
const aggregate = text("crates/starclock-activity/src/aggregate.rs");
const tests = text("crates/starclock-activity/tests/graph_definition.rs");
const boundaryTests = text("crates/starclock-activity/tests/activity_boundary.rs");

for (const marker of [
  "pub struct ActivityGraphDefinition",
  "pub struct ActivityNodeDefinition",
  "pub struct ActivityEdgeDefinition",
  "pub enum ActivityNodeKind",
  "pub enum ActivityTerminalOutcome",
  "Abandoned = 2",
  "pub enum ActivityEdgeCondition",
  "ActivityGraphDefinitionError::CannotReachTerminal"
]) assert(graph.includes(marker), `Activity graph implementation omits ${marker}`);
for (const [constant, value] of [
  ["MAX_ACTIVITY_NODES", policy.bounds.maximum_nodes],
  ["MAX_ACTIVITY_EDGES", policy.bounds.maximum_edges],
  ["MAX_ACTIVITY_TOTAL_VISITS", policy.bounds.maximum_total_visits],
  ["MAX_NODE_VISITS", policy.bounds.maximum_node_visits],
  ["MAX_EDGE_TRAVERSALS", policy.bounds.maximum_edge_traversals]
]) assert(new RegExp(`const ${constant}: [^=]+ = ${numberLiteral(value)};`).test(graph), `${constant} differs from policy`);

assert(codec.includes("pub(crate) struct ActivityV2Writer"), "Activity v2 writer is absent");
assert(codec.includes("to_le_bytes()"), "Activity v2 writer is not explicitly little-endian");
assert(graph.includes(`b"${policy.graph_codec}"`), "graph digest domain differs");
assert(graph.includes(`*b"${policy.graph_codec_magic}"`), "graph digest magic differs");
assert(graph.includes("nodes.sort_by_key") && graph.includes("edges.sort_by_key"), "graph identity does not canonicalize authored order");
assert(graph.includes("validate_reachability") && graph.includes("CannotReachTerminal"), "graph reachability closure is absent");
assert(spec.includes("graph: ActivityGraphDefinition") && spec.includes("let graph = flow.into_graph()"), "one-battle spec is not backed by the generic graph");
assert(aggregate.includes("spec.graph().entry()") && aggregate.includes("ActivityEdgeCondition::BattleOutcome"), "one-battle execution does not traverse the generic graph");
assert((tests.match(/^#\[test\]$/gm) ?? []).length === policy.focused_tests, "Activity graph focused test count differs");
assert(decimalGolden(tests) === policy.representative_graph_digest, "representative graph digest golden differs");
for (const legacy of [
  "194, 66, 33, 96",
  "237, 237, 61, 51",
  "89, 89, 16, 147"
]) assert(boundaryTests.includes(legacy), `legacy one-battle hash golden changed: ${legacy}`);

const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P2-B1` \| `(InProgress|Complete)` \|/m.test(status), "G04-P2-B1 is not active or complete");
const evidence = {
  schema_revision: "starclock.goal04-activity-graph-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "validated-immutable-activity-graph",
  graph_codec: {
    domain: policy.graph_codec,
    magic: policy.graph_codec_magic,
    version: policy.graph_codec_version,
    byte_order: "little-endian",
    representative_digest: policy.representative_graph_digest
  },
  bounds: policy.bounds,
  shape: policy.shape,
  compatibility: {
    goal01_one_battle_profile_compiles_to_generic_graph: true,
    goal01_state_hash_goldens_unchanged: true,
    graph_activity_state_machine_deferred: "G04-P2-B2/G04-P2-B3",
    generated_rows_public: false
  },
  source_sha256: {
    graph: sha256("crates/starclock-activity/src/graph.rs"),
    tests: sha256("crates/starclock-activity/tests/graph_definition.rs")
  },
  focused_tests: policy.focused_tests,
  new_registry_packages: policy.new_registry_packages
};
const relative = "evidence/standard-universe-runtime-v1/activity/graph-definition.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "Activity graph evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "Activity graph evidence is stale; run with --bless");
}
console.log(`Goal 04 Activity graph verified (${policy.bounds.maximum_nodes} nodes, ${policy.bounds.maximum_edges} edges; ${policy.representative_graph_digest.slice(0, 12)}).`);

function numberLiteral(value) { return String(value).replace(/\B(?=(\d{3})+(?!\d))/g, "_"); }
function decimalGolden(source) {
  const match = source.match(/left\.digest\(\)\.bytes\(\),\s*\[([\s\S]*?)\]\s*\)/);
  assert(match, "representative graph decimal golden is absent");
  const bytes = [...match[1].matchAll(/\d+/g)].map((item) => Number(item[0]));
  assert(bytes.length === 32 && bytes.every((value) => value >= 0 && value <= 255), "representative graph golden is not 32 bytes");
  return Buffer.from(bytes).toString("hex");
}
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
