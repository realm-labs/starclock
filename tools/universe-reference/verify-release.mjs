import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const hasRoot = Boolean(process.argv[2] && !process.argv[2].startsWith("--"));
const root = path.resolve(hasRoot ? process.argv[2] : ".");
const options = process.argv.slice(hasRoot ? 3 : 2);
assert(options.every((option) => ["--bless", "--require-clean"].includes(option)), "usage: verify-release.mjs [root] [--bless] [--require-clean]");
const bless = options.includes("--bless");
const requireClean = options.includes("--require-clean");

const status = text("docs/goals/03-standard-universe-reference-data-status.md");
assert(status.includes("| State | `Complete` |"), "Goal 03 state is not Complete");
assert(status.includes("| Next unblocked batch | None |"), "Goal 03 still has a next batch");
assert((status.match(/^\| Phase [0-4].*\| `Complete` \|/gm) ?? []).length === 5, "not every Goal 03 phase is Complete");
assert((status.match(/^\| `G03-P[0-4]-B[0-9]+` \| `Complete` \|/gm) ?? []).length === 28, "not every Goal 03 batch is Complete");
assert(!status.includes("- [ ]"), "Goal 03 terminal checklist has unchecked items");
assert(status.includes("| Completion commit | This row's containing commit (`G03-P4-B4`) |"), "Goal 03 completion record is missing");
assert(text("docs/goals/README.md").includes("no universe runtime | Complete |"), "Goal index does not mark Goal 03 Complete");
const boundary = text("docs/23-standard-simulated-universe-reference.md");
const normalizedBoundary = boundary.replace(/\s+/g, " ");
for (const marker of ["config/universe-project.toml", "config/universe-generated/", "not the runtime `config/generated/config.sora`", "[Goal 04](goals/04-standard-universe-runtime.md) now provides that deliberate domain conversion"])
  assert(normalizedBoundary.includes(marker), `public boundary omits ${marker}`);

run("node", ["tools/universe-reference/audit-release.mjs", "."]);
run("node", ["tools/universe-reference/verify-fixtures.mjs", "."]);
run("node", ["tools/goal-hardening/verify-release-contract.mjs"]);
run("node", ["tools/agent-control/verify-goal02-release-contract.mjs"]);

const manifest = json("content-manifests/standard-universe-v1/content-manifest.json");
const coverage = json("content-reference/standard-universe-v1/coverage.json");
const pack = json("content-reference/standard-universe-v1/pack-index.json");
const schema = json("config/universe-generated/schema.lock").schema;
const debugRoot = path.join(root, "config", "universe-generated", "debug-json");
const universeRows = schema.tables.reduce((sum, table) => sum + json(path.join("config/universe-generated/debug-json", `${table.name}.json`)).table.rows.length, 0);
const fixtures = json("content-reference/standard-universe-v1/review-fixtures.json");
const rules = json("content-reference/standard-universe-v1/mechanic-rules.json");
const sources = json("content-reference/standard-universe-v1/sources.json");
assert(fs.readdirSync(debugRoot).filter((name) => name.endsWith(".json")).length === 49, "Universe debug table count differs");
assert(schema.tables.length === 49 && universeRows === 13793, "Universe schema/row denominator differs");
assert(coverage.required === 2201 && coverage.data_ready === 2201 && coverage.coverage_percent === "100", "Universe coverage differs");
assert(rules.length === 786 && fixtures.length === 78 && sources.length === 2645, "Universe rule/fixture/source denominator differs");

const commits = {
  workbooks: "4937812e79ee3d90d489a36481a7c9c5cb4f8f53",
  release_audit: "18e7fbc194db192d14b582907549b5950e76f794",
  semantic_fixtures: "0e6f365026ab8f62eadd814a242e5e2e6cfea920",
  pipeline_hardening: "6a2797640b61ce9d0c8c4a5a000c809e0364c977",
};
for (const commit of Object.values(commits)) run("git", ["cat-file", "-e", `${commit}^{commit}`]);

const evidence = {
  schema_revision: "starclock.standard-universe-reference-release.v1",
  goal_id: "standard-universe-reference-v1",
  result: "complete",
  snapshot: { game_version: "4.4", access_date: "2026-07-22", profile: "standard-main-world" },
  commits,
  content: {
    manifest_rows: Object.values(manifest.categories).reduce((sum, category) => sum + category.count, 0),
    data_ready: coverage.data_ready,
    rules: rules.length,
    fixtures: fixtures.length,
    fixture_facts: fixtures.reduce((sum, fixture) => sum + fixture.expected_facts.length, 0),
    provenance_rows: sources.length,
    pack_files: pack.files.length,
  },
  authoring: {
    adapter: "openpyxl==3.1.5",
    schema_export_authority: "sora-cli==0.3.0",
    tables: schema.tables.length,
    workbook_rows: universeRows,
    workbook_semantic_sha256: "d54c6c1bea0fef51dc844aadfe77270976d288358b36c77b391b16ffba383390",
    workbooks: {
      "Universe.xlsx": sha256("config/data/Universe.xlsx"),
      "UniverseBindings.xlsx": sha256("config/data/UniverseBindings.xlsx"),
      "UniverseEvidence.xlsx": sha256("config/data/UniverseEvidence.xlsx"),
    },
  },
  digests: {
    source_manifest_sha256: sha256("content-manifests/standard-universe-v1/content-manifest.json"),
    normalized_pack_sha256: pack.pack_sha256,
    pack_index_file_sha256: sha256("content-reference/standard-universe-v1/pack-index.json"),
    coverage_file_sha256: sha256("content-reference/standard-universe-v1/coverage.json"),
    universe_staging_bundle_sha256: sha256("config/universe-generated/config.sora"),
    preserved_core_runtime_bundle_sha256: sha256("config/generated/config.sora"),
  },
  runtime_boundary: {
    universe_bundle_role: "StagingAndReviewOnly",
    universe_runtime_lowering: false,
    json_runtime_path: false,
    core_bundle_unchanged: true,
  },
  acceptance: {
    repository_command: "node tools/repository-check/run.mjs --with-source-cache",
    elapsed_seconds: 296,
    generated_artifact_checks: 28,
    goal01_release_contract: "pass",
    goal02_release_contract: "pass",
    visual_review_pages: 49,
    clean_verification_command: "node tools/universe-reference/verify-release.mjs . --require-clean",
  },
};

assert(evidence.content.manifest_rows === 1935, "manifest denominator differs");
assert(evidence.digests.normalized_pack_sha256 === "8a6ea40d777be0c007290dc4af82080c6bc8abd56d5b3e133309dea66e9eb5dd", "pack digest differs");
assert(evidence.digests.universe_staging_bundle_sha256 === "0d94d25bf93392fb65cca1d2879a36170f70262d3dab5a92d5b634fab19f3b04", "Universe bundle digest differs");
assert(evidence.digests.preserved_core_runtime_bundle_sha256 === "abd84f70461675337092d12377db53f08b4562114fa90aa0b37ad869e9270440", "core runtime bundle changed");

const relative = "evidence/standard-universe-reference-v1/release/release-evidence.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "Goal 03 release evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "Goal 03 release evidence is stale; run with --bless");
}
if (requireClean) assert(capture("git", ["status", "--porcelain"]).trim() === "", "Goal 03 worktree is not clean");
console.log(`Goal 03 release verified (${schema.tables.length} tables, ${universeRows} rows, ${coverage.data_ready} DataReady, ${fixtures.length} fixtures${requireClean ? ", clean" : ""}).`);

function run(command, arguments_) { execFileSync(command, arguments_, { cwd: root, stdio: "ignore" }); }
function capture(command, arguments_) { return execFileSync(command, arguments_, { cwd: root, encoding: "utf8" }); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
