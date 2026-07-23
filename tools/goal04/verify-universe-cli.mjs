import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const artifactOnly = process.env.STARCLOCK_ARTIFACT_CHECK_ONLY === "1";
const policy = json("policy/goal04-universe-cli.json");
assert(policy.schema_revision === "starclock.goal04-universe-cli.v1", "unexpected Universe CLI policy revision");
const main = text("crates/starclock-cli/src/main.rs");
const universe = text("crates/starclock-cli/src/universe_v1.rs");
const tests = text("crates/starclock-cli/tests/universe_cli.rs");
const manifest = text("crates/starclock-cli/Cargo.toml");

for (const marker of [
  'group == "universe" && command == "run"',
  'group == "universe" && command == "coverage"',
  'group == "universe" && scope == "config"',
  "is_universe_replay"
]) assert(main.includes(marker), `CLI router omits ${marker}`);
for (const marker of [
  `const CLI_REVISION: &str = "${policy.cli_revision}"`,
  "record_baseline_run",
  "encode_standard_universe_trace",
  "verify_standard_universe_replay",
  policy.contracts.battle_executor,
  policy.bundle_sha256
]) assert(universe.includes(marker) || tests.includes(marker), `Universe CLI implementation omits ${marker}`);
for (const marker of [
  "universe_configuration_and_coverage_are_machine_readable",
  "universe_run_round_trips_a_canonical_replay_and_detects_corruption",
  "universe_cli_keeps_usage_and_unknown_content_exit_classes_distinct",
  policy.golden.final_state_hash
]) assert(tests.includes(marker), `Universe CLI integration test omits ${marker}`);
assert(manifest.includes('starclock-mode-universe = { path = "../starclock-mode-universe" }'), "CLI does not depend on the Universe adapter crate");

if (!artifactOnly)
  execFileSync("cargo", ["test", "-p", "starclock-cli", "--test", "universe_cli", "--all-features"], { cwd: root, stdio: "inherit" });

const sources = [
  "crates/starclock-cli/Cargo.toml",
  "crates/starclock-cli/src/main.rs",
  "crates/starclock-cli/src/universe_v1.rs",
  "crates/starclock-cli/tests/universe_cli.rs",
  "tools/workspace/verify-dependencies.mjs"
];
const evidence = {
  schema_revision: "starclock.goal04-universe-cli-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "production-bundle-diagnostics-complete-baseline-run-and-canonical-replay-verification-are-cli-accessible",
  revision: policy.cli_revision,
  bundle_sha256: policy.bundle_sha256,
  catalog: policy.catalog,
  golden: policy.golden,
  contracts: policy.contracts,
  source_sha256: Object.fromEntries(sources.map((relative) => [relative, sha256(relative)])),
  policy_sha256: sha256("policy/goal04-universe-cli.json"),
  new_registry_packages: []
};
const relativeEvidence = "evidence/standard-universe-runtime-v1/interfaces/universe-cli.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relativeEvidence)), { recursive: true });
  fs.writeFileSync(path.join(root, relativeEvidence), output);
} else {
  assert(fs.existsSync(path.join(root, relativeEvidence)), "Universe CLI evidence is missing; run with --bless");
  assert(text(relativeEvidence).replaceAll("\r\n", "\n") === output, "Universe CLI evidence is stale; run with --bless");
}
console.log(`Goal 04 Universe CLI verified (${policy.golden.actions} actions, ${policy.golden.nested_battles} battles, ${policy.golden.encoded_bytes} bytes).`);

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
