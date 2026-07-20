import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
assert(args.every((argument) => argument === "--bless"), "usage: verify-clara-counter.mjs [--bless]");
const bless = args.includes("--bless");
const fixture = path.join(root, "config/probes/v1a/clara-counter");
const work = path.join(root, ".cache/clara-counter-probe-verify");
assert(path.relative(root, work).replaceAll("\\", "/") === ".cache/clara-counter-probe-verify", "unexpected Clara work path");

const policy = readJson(path.join(root, "policy/sora-toolchain.json"));
const sora = path.join(root, policy.install_root, "bin", process.platform === "win32" ? "sora.exe" : "sora");
assert(fs.existsSync(sora), `Sora ${policy.version} is not installed`);
assert(capture(sora, ["--version"]).stdout.trim() === `sora ${policy.version}`, "wrong Sora version");

fs.rmSync(work, { recursive: true, force: true });
fs.mkdirSync(work, { recursive: true });
const first = generate("first");
const second = generate("second");
assertSameFile(first.bundle, second.bundle, "independent Clara probe bundles differ");
assertSameTree(first.debug, second.debug, "independent Clara probe debug exports differ");

const counts = Object.fromEntries(fs.readdirSync(first.debug).filter((file) => file.endsWith(".json")).sort().map((file) => [path.basename(file, ".json"), readJson(path.join(first.debug, file)).table.rows.length]));
const expectedCounts = { Ability: 1, AbilityPhase: 1, ConditionExpression: 1, ConfigManifest: 1, ContentEvidenceBinding: 6, ContentIdentity: 6, EventFilter: 1, EvidenceRecord: 4, Operation: 2, Program: 1, ProgramStep: 2, RuleDefinition: 1, RuleTrigger: 1, Selector: 2, SourceRecord: 1, StateSlot: 1, ValueExpression: 3 };
assert(Object.keys(counts).length === 82, "Clara probe does not export all production tables");
for (const [name, count] of Object.entries(counts)) assert(count === (expectedCounts[name] ?? 0), `${name} Clara probe row count differs`);

const golden = {
  schema_revision: "starclock.v1a-clara-counter-probe.v1",
  sora_cli_version: policy.version,
  reference_pack_sha256: "0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a",
  talent_text_sha256: "b3227ad1ff5d956f96b89c0eefc1f7ea0e798e09abc966e948deb05dbe14cb10",
  ultimate_text_sha256: "d063025569b6ddb708c7f98725e943aa3cc87979736769b3d05a2ff9eadfe159",
  skill_text_sha256: "25c38d455922b5cb2bf9beb377fe2099c9bcbd44aadd7a4b46efe3b531f69e2d",
  fixture_contract_sha256: "d50deb87c40a97805e449a35a8d53b193e9a3cb88f9266e862fcbe1fceeee440",
  table_count: 82,
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
  console.log(`Blessed Clara counter probe (${golden.bundle_sha256}).`);
} else {
  assert(fs.existsSync(bundle) && fs.existsSync(expected), "Clara counter probe is not blessed");
  assertSameFile(first.bundle, bundle, "committed Clara counter bundle drifted");
  assert(JSON.stringify(readJson(expected)) === JSON.stringify(golden), "Clara counter golden drifted");
  console.log(`Clara counter probe verified (${golden.bundle_sha256}).`);
}

function generate(name) {
  const data = path.join(work, name, "data");
  const out = path.join(work, name, "out");
  run("cargo", ["run", "--manifest-path", "tools/workbook-bootstrap/Cargo.toml", "--locked", "--quiet", "--", "config/generated/templates", "config/probes/v1a/clara-counter/rows", path.relative(root, data)]);
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
