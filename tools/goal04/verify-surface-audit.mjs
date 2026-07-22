import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const root = path.resolve(process.argv[2] ?? ".");
const bless = process.argv.includes("--bless");
assert(process.argv.slice(2).every((value, index) => index === 0 && value !== "--bless" || value === "--bless"), "usage: verify-surface-audit.mjs [root] [--bless]");

const policy = json("policy/goal04-surface-audit.json");
assert(policy.schema_revision === "starclock.goal04-surface-audit.v1", "unexpected audit policy revision");
const commit = resolveCommit(policy.baseline_commit);
const baseline = (relative) => capture("git", ["show", `${commit}:${relative}`]);

const crateNames = baselineWorkspaceCrates(commit);
assert(equal(crateNames, policy.baseline_surface.workspace_crates), "baseline workspace crate inventory differs");
assert(!crateNames.includes("starclock-mode-universe"), "baseline unexpectedly contains universe mode crate");

const aggregate = baseline("crates/starclock-activity/src/aggregate.rs");
for (const marker of ["pub enum ActivityCommand", "StartBattle {", "SubmitBattleResult {", "pub fn apply(", "BattleHandoff"])
  assert(aggregate.includes(marker), `baseline Activity omits ${marker}`);
assert(enumVariants(aggregate, "ActivityCommand").join(",") === policy.baseline_surface.activity_command_variants.join(","), "baseline Activity command variants differ");
assert(enumVariants(aggregate, "ActivityDecision").join(",") === policy.baseline_surface.activity_decision_variants.join(","), "baseline Activity decision variants differ");

const scope = baseline("crates/starclock-activity/src/scope.rs");
for (const marker of policy.baseline_surface.activity_scopes)
  assert(scope.includes(marker), `baseline Activity scope omits ${marker}`);

const replay = baseline("crates/starclock-replay/src/activity.rs");
for (const marker of ["NestedBattleBoundary", "Start(BattleResultIdentity)", "End(BattleResultDigest)", "ACTIVITY_COMMAND_PAYLOAD_VERSION: u16 = 1"])
  assert(replay.includes(marker), `baseline Activity replay omits ${marker}`);

const cli = baseline("crates/starclock-cli/src/main.rs");
for (const route of policy.baseline_surface.cli_route_families)
  assert(cli.includes(`group == "${route}"`), `baseline CLI omits ${route} route`);
assert(!cli.includes('group == "universe"'), "baseline unexpectedly has Universe CLI route");

const mcp = baseline("crates/starclock-mcp/src/tools.rs");
const mcpTools = [...mcp.matchAll(/name = "(starclock_[a-z_]+)"/g)].map((match) => match[1]);
assert(equal(mcpTools, policy.baseline_surface.mcp_tool_names), "baseline MCP tool inventory differs");

const coreSchema = json("config/generated/schema.lock").schema.tables.length;
const universeSchema = json("config/universe-generated/schema.lock").schema.tables.length;
assert(coreSchema === policy.prerequisites.core_tables, "core schema table count differs");
assert(universeSchema === policy.prerequisites.universe_tables, "Universe schema table count differs");
assert(sha256("config/generated/config.sora") === policy.prerequisites.core_bundle_sha256, "core bundle digest differs");
assert(sha256("config/universe-generated/config.sora") === policy.prerequisites.universe_bundle_sha256, "Universe bundle digest differs");

const coverage = json("content-reference/standard-universe-v1/coverage.json");
const rules = json("content-reference/standard-universe-v1/mechanic-rules.json");
const fixtures = json("content-reference/standard-universe-v1/review-fixtures.json");
assert(coverage.data_ready === policy.prerequisites.data_ready_records, "DataReady denominator differs");
assert(rules.length === policy.prerequisites.rule_bindings, "rule denominator differs");
assert(fixtures.length === policy.prerequisites.semantic_fixtures, "fixture denominator differs");

const auditDoc = text("docs/standard-universe-runtime-surface-audit.md");
for (const marker of ["not a claim", "Activity::apply", "bounded Activity micrograph", "G04-P5-B5"])
  assert(auditDoc.includes(marker), `surface audit document omits ${marker}`);
const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P0-B1` \| `(InProgress|Complete)` \|/m.test(status), "G04-P0-B1 is not active or complete");

const evidence = {
  schema_revision: "starclock.goal04-surface-audit-evidence.v1",
  goal_id: policy.goal_id,
  result: "baseline-audited",
  baseline_commit: commit,
  snapshot: policy.snapshot,
  prerequisites: {
    core_bundle_sha256: sha256("config/generated/config.sora"),
    universe_bundle_sha256: sha256("config/universe-generated/config.sora"),
    core_tables: coreSchema,
    universe_tables: universeSchema,
    data_ready_records: coverage.data_ready,
    rule_bindings: rules.length,
    semantic_fixtures: fixtures.length
  },
  baseline_surface: {
    workspace_crates: crateNames,
    activity_commands: policy.baseline_surface.activity_command_variants,
    activity_decisions: policy.baseline_surface.activity_decision_variants,
    activity_scopes: policy.baseline_surface.activity_scopes,
    nested_battle_boundaries: policy.baseline_surface.nested_battle_boundaries,
    cli_route_families: policy.baseline_surface.cli_route_families,
    mcp_tool_names: mcpTools
  },
  retained_invariants: policy.retained_invariants,
  gaps: policy.gaps,
  conclusion: "Goal 04 requires generic multi-node and multi-battle Activity evolution; no Universe runtime capability is claimed by this baseline."
};

const relative = "evidence/standard-universe-runtime-v1/foundation/surface-audit.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "surface audit evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "surface audit evidence is stale; run with --bless");
}
console.log(`Goal 04 surface audited at ${commit.slice(0, 12)} (${crateNames.length} crates, ${mcpTools.length} MCP tools, ${policy.gaps.length} owned gaps).`);

function baselineWorkspaceCrates(commit) {
  const files = capture("git", ["ls-tree", "-r", "--name-only", commit, "crates"])
    .split(/\r?\n/).filter((file) => /^crates\/[^/]+\/Cargo\.toml$/.test(file));
  return files.map((file) => {
    const cargo = capture("git", ["show", `${commit}:${file}`]);
    const name = cargo.match(/^name\s*=\s*"([^"]+)"/m)?.[1];
    assert(name, `cannot read package name from ${file}`);
    return name;
  }).sort();
}
function enumVariants(source, name) {
  const body = source.match(new RegExp(`pub enum ${name} \\{([\\s\\S]*?)^\\}`, "m"))?.[1];
  assert(body, `cannot find enum ${name}`);
  return [...body.matchAll(/^\s{4}([A-Z][A-Za-z0-9_]*)/gm)].map((match) => match[1]);
}
function resolveCommit(revision) { return capture("git", ["rev-parse", `${revision}^{commit}`]).trim(); }
function run(command, args) { return execFileSync(command, args, { cwd: root, encoding: "utf8" }); }
function capture(command, args) { return run(command, args); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
