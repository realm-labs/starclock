import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-activity-transaction.json");
assert(policy.schema_revision === "starclock.goal04-activity-transaction.v1", "unexpected Activity transaction policy revision");
const program = text("crates/starclock-activity/src/program.rs");
const transaction = text("crates/starclock-activity/src/transaction.rs");
const tests = text("crates/starclock-activity/tests/activity_transaction.rs");
const boundaryTests = text("crates/starclock-activity/tests/activity_boundary.rs");

for (const marker of [
  "pub enum ActivityExpression", "pub enum ActivityCondition", "pub enum ActivityOperation",
  "pub struct ActivityOptionDefinition", "pub struct ActivityProgramDefinition",
  "pub fn validate_against", "pub enum ActivityProgramBindingError"
]) assert(program.includes(marker), `Activity program model omits ${marker}`);
for (const marker of [
  "pub struct ActivityCause", "pub enum ActivityTransactionEventKind",
  "pub enum ActivityFault", "pub enum ActivityTransactionOutcome",
  "let mut working = self.clone()", "faulted.terminal = Some(ActivityTerminalOutcome::Faulted)",
  "edge_traversals", "node_visits", "maximum_total_visits"
]) assert(transaction.includes(marker), `Activity transaction executor omits ${marker}`);
for (const [constant, value] of [
  ["MAX_ACTIVITY_PROGRAM_OPERATIONS", policy.bounds.maximum_operations],
  ["MAX_ACTIVITY_PROGRAM_DEPTH", policy.bounds.maximum_program_depth],
  ["MAX_ACTIVITY_OPTIONS", policy.bounds.maximum_options]
]) assert(new RegExp(`const ${constant}: [^=]+ = ${numberLiteral(value)};`).test(program), `${constant} differs from policy`);
assert(program.includes("(pair[0].priority, pair[0].id) >= (pair[1].priority, pair[1].id)"), "option ordering contract is absent");
assert(!/f32|f64|HashMap/.test(program + transaction), "Activity transaction uses float or unordered map");
assert((tests.match(/^#\[test\]$/gm) ?? []).length === policy.focused_tests, "Activity transaction focused test count differs");
for (const marker of [
  "failed_requirement_rejects_without_changing_any_state",
  "internal_fault_discards_partial_work_and_commits_only_faulted_settlement",
  "graph_visit_and_edge_budgets_are_authoritative_transaction_limits"
]) assert(tests.includes(marker), `Activity transaction tests omit ${marker}`);
for (const legacy of ["194, 66, 33, 96", "237, 237, 61, 51", "89, 89, 16, 147"])
  assert(boundaryTests.includes(legacy), `legacy one-battle hash golden changed: ${legacy}`);
const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P2-B3` \| `(InProgress|Complete)` \|/m.test(status), "G04-P2-B3 is not active or complete");

const evidence = {
  schema_revision: "starclock.goal04-activity-transaction-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "typed-program-and-transaction-runtime-implemented",
  bounds: policy.bounds,
  shape: policy.shape,
  compatibility: {
    goal01_state_hash_goldens_unchanged: true,
    graph_state_hash_and_rng_deferred: "G04-P2-B4",
    battle_handoff_deferred: "G04-P2-B5",
    generated_rows_public: false,
    authoritative_float: false
  },
  source_sha256: {
    program: sha256("crates/starclock-activity/src/program.rs"),
    transaction: sha256("crates/starclock-activity/src/transaction.rs"),
    tests: sha256("crates/starclock-activity/tests/activity_transaction.rs")
  },
  focused_tests: policy.focused_tests,
  new_registry_packages: policy.new_registry_packages
};
const relative = "evidence/standard-universe-runtime-v1/activity/transaction-runtime.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "Activity transaction evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "Activity transaction evidence is stale; run with --bless");
}
console.log(`Goal 04 Activity transaction verified (${policy.bounds.maximum_operations} ops, ${policy.bounds.maximum_options} options, ${policy.focused_tests} tests).`);

function numberLiteral(value) { return String(value).replace(/\B(?=(\d{3})+(?!\d))/g, "_"); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
