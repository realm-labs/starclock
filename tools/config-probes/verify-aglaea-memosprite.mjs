import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
assert(args.every((argument) => argument === "--bless"), "usage: verify-aglaea-memosprite.mjs [--bless]");
const bless = args.includes("--bless");
const fixture = path.join(root, "config/probes/v1a/aglaea-memosprite");
const work = path.join(root, ".cache/aglaea-memosprite-probe-verify");
assert(path.relative(root, work).replaceAll("\\", "/") === ".cache/aglaea-memosprite-probe-verify", "unexpected Aglaea work path");

const policy = readJson(path.join(root, "policy/sora-toolchain.json"));
const sora = path.join(root, policy.install_root, "bin", process.platform === "win32" ? "sora.exe" : "sora");
assert(fs.existsSync(sora), `Sora ${policy.version} is not installed`);
assert(capture(sora, ["--version"]).stdout.trim() === `sora ${policy.version}`, "wrong Sora version");
const sourcePayload = path.join(root, ".cache/content-reference/turnbasedgamedata/Config/ConfigAbility/Avatar/Avatar_Aglaea_00_Ability.json");
assert(fs.existsSync(sourcePayload), "prepared Aglaea source payload is absent");
const sourcePayloadSha256 = sha256(sourcePayload);
assert(sourcePayloadSha256 === "e26fa36f13e9b0eccda98cb8537e68cbe39e792b7100af5b3bf6e6c4a08f746b", "prepared Aglaea source payload drifted");

fs.rmSync(work, { recursive: true, force: true });
fs.mkdirSync(work, { recursive: true });
const first = generate("first");
const second = generate("second");
assertSameFile(first.bundle, second.bundle, "independent Aglaea probe bundles differ");
assertSameTree(first.debug, second.debug, "independent Aglaea probe debug exports differ");

const counts = Object.fromEntries(fs.readdirSync(first.debug).filter((file) => file.endsWith(".json")).sort().map((file) => [path.basename(file, ".json"), readJson(path.join(first.debug, file)).table.rows.length]));
const expectedCounts = { ConfigManifest: 1, ContentEvidenceBinding: 6, ContentIdentity: 6, EvidenceRecord: 5, Operation: 3, Program: 1, ProgramStep: 3, Selector: 2, SourceRecord: 1 };
assert(Object.keys(counts).length === 80, "Aglaea probe does not export all production tables");
for (const [name, count] of Object.entries(counts)) assert(count === (expectedCounts[name] ?? 0), `${name} probe row count differs`);

const golden = {
  schema_revision: "starclock.v1a-aglaea-memosprite-probe.v1",
  sora_cli_version: policy.version,
  reference_pack_sha256: "0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a",
  source_payload_sha256: sourcePayloadSha256,
  summon_skill_text_sha256: "85caf7fd51a5fe97c32c5f3f6120c7b73af62ebac713dcc29b936dd77ed98962",
  memosprite_talent_text_sha256: "af709d10048b253540fe9439ce44beff539202c1336ebd7aee9a8e00c7b3d371",
  joint_action_text_sha256: "4f5ddbdb234c2db9b384f699d59edf0a0b6af33841c1e426071ccb6b652b9b26",
  ultimate_text_sha256: "ee583a92ce0fa22ec5ed4e3c1c3253f913b4efabdd312ee8639fb6a2c2c1f3ce",
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
  console.log(`Blessed Aglaea memosprite probe (${golden.bundle_sha256}).`);
} else {
  assert(fs.existsSync(bundle) && fs.existsSync(expected), "Aglaea memosprite probe is not blessed");
  assertSameFile(first.bundle, bundle, "committed Aglaea memosprite bundle drifted");
  assert(JSON.stringify(readJson(expected)) === JSON.stringify(golden), "Aglaea memosprite golden drifted");
  console.log(`Aglaea memosprite probe verified (${golden.bundle_sha256}).`);
}

function generate(name) { const data = path.join(work, name, "data"); const out = path.join(work, name, "out"); run("cargo", ["run", "--manifest-path", "tools/workbook-bootstrap/Cargo.toml", "--locked", "--quiet", "--", "config/generated/templates", "config/probes/v1a/aglaea-memosprite/rows", path.relative(root, data)]); fs.mkdirSync(out, { recursive: true }); const bundle = path.join(out, "config.sora"); const debug = path.join(out, "debug-json"); run(sora, ["--serial", "export", "--format", "binary", "--project", "config/project.toml", "--data-root", data, "--out", bundle]); run(sora, ["--serial", "export", "--format", "json-debug", "--project", "config/project.toml", "--data-root", data, "--out", debug]); return { bundle, debug }; }
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
