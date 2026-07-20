import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
assert(args.every((argument) => argument === "--bless"), "usage: verify-kafka-dot.mjs [--bless]");
const bless = args.includes("--bless");
const fixture = path.join(root, "config/probes/v1a/kafka-dot");
const work = path.join(root, ".cache/kafka-dot-probe-verify");
assert(path.relative(root, work).replaceAll("\\", "/") === ".cache/kafka-dot-probe-verify", "unexpected Kafka work path");

const policy = readJson(path.join(root, "policy/sora-toolchain.json"));
const sora = path.join(root, policy.install_root, "bin", process.platform === "win32" ? "sora.exe" : "sora");
assert(fs.existsSync(sora), `Sora ${policy.version} is not installed`);
assert(capture(sora, ["--version"]).stdout.trim() === `sora ${policy.version}`, "wrong Sora version");

fs.rmSync(work, { recursive: true, force: true });
fs.mkdirSync(work, { recursive: true });
const first = generate("first");
const second = generate("second");
assertSameFile(first.bundle, second.bundle, "independent Kafka probe bundles differ");
assertSameTree(first.debug, second.debug, "independent Kafka probe debug exports differ");

const counts = Object.fromEntries(fs.readdirSync(first.debug).filter((file) => file.endsWith(".json")).sort().map((file) => [path.basename(file, ".json"), readJson(path.join(first.debug, file)).table.rows.length]));
const expectedCounts = { ConfigManifest: 1, ContentEvidenceBinding: 9, ContentIdentity: 9, Effect: 1, EvidenceRecord: 4, Operation: 7, Program: 3, ProgramStep: 7, RuleDefinition: 1, Selector: 3, SourceRecord: 1, StateSlot: 1, StateSlotReset: 1, ValueExpression: 15 };
assert(Object.keys(counts).length === 80, "Kafka probe does not export all production tables");
for (const [name, count] of Object.entries(counts)) assert(count === (expectedCounts[name] ?? 0), `${name} Kafka probe row count differs`);

const golden = {
  schema_revision: "starclock.v1a-kafka-dot-probe.v1",
  sora_cli_version: policy.version,
  reference_pack_sha256: "0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a",
  source_payload_sha256: "1e5818890690c13d6cdf5c23d2af962383bd3f5588b3fe7376d0ed86376d59e3",
  skill_text_sha256: "b5949a8ece4b820b09fb1c5c5b0b3c21f301ccb140d8a9a087b24602ef55ecd9",
  ultimate_text_sha256: "db74faed79ab0239c580c7690fb1c3e26483cbf9280a34f80a65bda2e672799f",
  talent_text_sha256: "459f96d9ff2695ebaab77a3983b153e0c425043cfd9fe80cc15fd8cf53f6c926",
  observation_sha256: "9d65d647f562badfec8104e962748061373baff3057343bba6fd833d38be18bc",
  table_count: 80,
  populated_table_count: Object.values(counts).filter((count) => count > 0).length,
  identity_count: counts.ContentIdentity,
  production_coverage_credit: 0,
  bundle_sha256: sha256(first.bundle),
  debug_digest: digestFileMap(artifactMap(first.debug)),
};
const bundle = path.join(fixture, "config.sora");
const expected = path.join(fixture, "golden.json");
if (bless) {
  fs.copyFileSync(first.bundle, bundle);
  fs.writeFileSync(expected, `${JSON.stringify(golden, null, 2)}\n`);
  console.log(`Blessed Kafka DoT probe (${golden.bundle_sha256}).`);
} else {
  assert(fs.existsSync(bundle) && fs.existsSync(expected), "Kafka DoT probe is not blessed");
  assertSameFile(first.bundle, bundle, "committed Kafka DoT bundle drifted");
  assert(JSON.stringify(readJson(expected)) === JSON.stringify(golden), "Kafka DoT golden drifted");
  console.log(`Kafka DoT probe verified (${golden.bundle_sha256}).`);
}

function generate(name) {
  const data = path.join(work, name, "data");
  const out = path.join(work, name, "out");
  run("cargo", ["run", "--manifest-path", "tools/workbook-bootstrap/Cargo.toml", "--locked", "--quiet", "--", "config/generated/templates", "config/probes/v1a/kafka-dot/rows", path.relative(root, data)]);
  fs.mkdirSync(out, { recursive: true });
  const bundle = path.join(out, "config.sora");
  const debug = path.join(out, "debug-json");
  run(sora, ["--serial", "export", "--format", "binary", "--project", "config/project.toml", "--data-root", data, "--out", bundle]);
  run(sora, ["--serial", "export", "--format", "json-debug", "--project", "config/project.toml", "--data-root", data, "--out", debug]);
  return { bundle, debug };
}
function artifactMap(directory) { return Object.fromEntries(walk(directory).map((file) => [path.relative(directory, file).replaceAll("\\", "/"), sha256(file)])); }
function digestFileMap(files) { return crypto.createHash("sha256").update(Object.entries(files).map(([name, digest]) => `${name}\0${digest}\n`).join(""), "utf8").digest("hex"); }
function walk(directory) { return fs.readdirSync(directory, { withFileTypes: true }).sort((left, right) => left.name.localeCompare(right.name)).flatMap((entry) => { const target = path.join(directory, entry.name); return entry.isDirectory() ? walk(target) : [target]; }); }
function assertSameFile(left, right, message) { assert(fs.readFileSync(left).equals(fs.readFileSync(right)), message); }
function assertSameTree(left, right, message) { assert(JSON.stringify(artifactMap(left)) === JSON.stringify(artifactMap(right)), message); }
function sha256(file) { return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex"); }
function readJson(file) { return JSON.parse(fs.readFileSync(file, "utf8")); }
function run(command, arguments_) { const env = command === "cargo" ? { ...process.env, CARGO_TARGET_DIR: path.join(root, ".cache/workbook-bootstrap-target") } : process.env; const result = spawnSync(command, arguments_, { cwd: root, stdio: "inherit", env }); if (result.error) throw result.error; assert(result.status === 0, `${command} ${arguments_.join(" ")} exited with ${result.status}`); }
function capture(command, arguments_) { const result = spawnSync(command, arguments_, { cwd: root, encoding: "utf8" }); if (result.error) throw result.error; assert(result.status === 0, `${command} ${arguments_.join(" ")} exited with ${result.status}`); return result; }
function assert(condition, message) { if (!condition) throw new Error(message); }
