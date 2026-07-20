import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
assert(args.every((argument) => argument === "--bless"), "usage: verify.mjs [--bless]");
const bless = args.includes("--bless");
const fixture = path.join(root, "config/catalog-fixtures/representative");
const work = path.join(root, ".cache/config-catalog-verify");
const expectedBundle = path.join(fixture, "config.sora");
const expectedGolden = path.join(fixture, "golden.json");
assert(path.relative(root, work).replaceAll("\\", "/") === ".cache/config-catalog-verify", "unexpected cache path");

const toolPolicy = readJson(path.join(root, "policy/sora-toolchain.json"));
const sora = path.join(root, toolPolicy.install_root, "bin", process.platform === "win32" ? "sora.exe" : "sora");
assert(fs.existsSync(sora), `Sora ${toolPolicy.version} is not installed`);
assert(capture(sora, ["--version"]).stdout.trim() === `sora ${toolPolicy.version}`, "wrong Sora version");

fs.rmSync(work, { recursive: true, force: true });
fs.mkdirSync(work, { recursive: true });
const first = generate("first");
const second = generate("second");
assertSameFile(first.bundle, second.bundle, "independent representative bundles differ");
assertSameTree(first.debug, second.debug, "independent representative debug exports differ");
verifyCounts(first.debug);

const bundleSha256 = sha256(first.bundle);
const debugFiles = artifactMap(first.debug);
const debugDigest = digestFileMap(debugFiles);
const golden = {
  schema_revision: "starclock.catalog-representative.v1",
  sora_cli_version: toolPolicy.version,
  schema_fingerprint: "efe787b52282426b",
  table_count: 82,
  populated_table_count: 13,
  identity_count: 3,
  ability_count: 1,
  hit_plan_count: 1,
  character_count: 1,
  bundle_sha256: bundleSha256,
  debug_digest: debugDigest,
  debug_files: debugFiles,
};

if (bless) {
  fs.copyFileSync(first.bundle, expectedBundle);
  fs.writeFileSync(expectedGolden, `${JSON.stringify(golden, null, 2)}\n`);
  console.log(`Blessed representative catalog fixture (${bundleSha256}).`);
} else {
  assert(fs.existsSync(expectedBundle) && fs.existsSync(expectedGolden), "representative fixture is not blessed");
  assertSameFile(first.bundle, expectedBundle, "committed representative bundle drifted");
  assert(JSON.stringify(readJson(expectedGolden)) === JSON.stringify(golden), "representative catalog golden drifted");
  console.log(`Representative catalog fixture verified (${bundleSha256}).`);
}

function generate(name) {
  const data = path.join(work, name, "data");
  const out = path.join(work, name, "out");
  run("cargo", ["run", "--manifest-path", "tools/workbook-bootstrap/Cargo.toml", "--locked", "--quiet", "--", "config/generated/templates", "config/catalog-fixtures/representative/rows", path.relative(root, data)]);
  fs.mkdirSync(out, { recursive: true });
  const bundle = path.join(out, "config.sora");
  const debug = path.join(out, "debug-json");
  run(sora, ["--serial", "export", "--format", "binary", "--project", "config/project.toml", "--data-root", data, "--out", bundle]);
  run(sora, ["--serial", "export", "--format", "json-debug", "--project", "config/project.toml", "--data-root", data, "--out", debug]);
  return { bundle, debug };
}

function verifyCounts(directory) {
  const files = fs.readdirSync(directory).filter((name) => name.endsWith(".json")).sort();
  assert(files.length === 82, "representative export does not contain all 82 tables");
  const counts = Object.fromEntries(files.map((file) => [path.basename(file, ".json"), readJson(path.join(directory, file)).table.rows.length]));
  const expected = {
    Ability: 1, AbilityHitPlanBinding: 1, AbilityPhase: 1, Character: 1,
    CharacterAbilityBinding: 1, CharacterStat: 2, ConfigManifest: 1,
    ContentEvidenceBinding: 3, ContentIdentity: 3, EvidenceRecord: 1,
    HitPlan: 1, HitPlanHit: 1, SourceRecord: 1,
  };
  for (const [name, count] of Object.entries(counts)) assert(count === (expected[name] ?? 0), `${name} row count differs`);
  assert(Object.values(counts).filter((count) => count > 0).length === 13, "populated table count differs");
}

function artifactMap(directory) { return Object.fromEntries(walk(directory).map((file) => [path.relative(directory, file).replaceAll("\\", "/"), sha256(file)])); }
function digestFileMap(files) { return crypto.createHash("sha256").update(Object.entries(files).map(([name, digest]) => `${name}\0${digest}\n`).join(""), "utf8").digest("hex"); }
function hashTree(directory) { return new Map(walk(directory).map((file) => [path.relative(directory, file).replaceAll("\\", "/"), sha256(file)])); }
function walk(directory) { return fs.readdirSync(directory, { withFileTypes: true }).sort((left, right) => left.name.localeCompare(right.name)).flatMap((entry) => { const target = path.join(directory, entry.name); return entry.isDirectory() ? walk(target) : [target]; }); }
function assertSameFile(left, right, message) { assert(fs.readFileSync(left).equals(fs.readFileSync(right)), message); }
function assertSameTree(left, right, message) { assert(JSON.stringify([...hashTree(left)]) === JSON.stringify([...hashTree(right)]), message); }
function sha256(file) { return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex"); }
function readJson(file) { return JSON.parse(fs.readFileSync(file, "utf8")); }
function run(command, arguments_) { const env = command === "cargo" ? { ...process.env, CARGO_TARGET_DIR: path.join(root, ".cache/workbook-bootstrap-target") } : process.env; const result = spawnSync(command, arguments_, { cwd: root, stdio: "inherit", env }); if (result.error) throw result.error; assert(result.status === 0, `${command} ${arguments_.join(" ")} exited with ${result.status}`); }
function capture(command, arguments_) { const result = spawnSync(command, arguments_, { cwd: root, encoding: "utf8" }); if (result.error) throw result.error; assert(result.status === 0, `${command} ${arguments_.join(" ")} exited with ${result.status}`); return result; }
function assert(condition, message) { if (!condition) throw new Error(message); }
