import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
assert(args.every((argument) => argument === "--bless"), "usage: verify-elation-probes.mjs [--bless]");
const bless = args.includes("--bless");
const work = path.join(root, ".cache/elation-probes-verify");
assert(path.relative(root, work).replaceAll("\\", "/") === ".cache/elation-probes-verify", "unexpected Elation work path");

const policy = readJson(path.join(root, "policy/sora-toolchain.json"));
const sora = path.join(root, policy.install_root, "bin", process.platform === "win32" ? "sora.exe" : "sora");
assert(fs.existsSync(sora), `Sora ${policy.version} is not installed`);
assert(capture(sora, ["--version"]).stdout.trim() === `sora ${policy.version}`, "wrong Sora version");

const probes = [
  {
    name: "trailblazer-elation",
    schemaRevision: "starclock.v1a-trailblazer-elation-probe.v1",
    payload: ".cache/content-reference/turnbasedgamedata/Config/ConfigAbility/Avatar/Avatar_PlayerBoy_40_Ability.json",
    payloadSha256: "b9d0e61c263aa32ebc36bb0b6c1a823b5fae70f7728da240e8ca5d83c568eb49",
    textSha256: {
      ultimate: "f30d262673ee24ce7c5944da20d4052295baf1444013082cdbd48912ab715dd5",
      elation_skill: "41803edd31826ff48192c41e274f847d90d75e1ec7292f1a281b97b9fa3b6518",
      passive: "2c95faba307b47cba7317308d45004ab21753c4a87a2db2d88ecf56462088bc0",
      fixture_contract: "33fce667cca4acf676074af7f9f4f512674f19d327635d268864150ad6b7635a",
    },
    counts: { Ability: 2, AbilityPhase: 2, ConfigManifest: 1, ContentEvidenceBinding: 4, ContentIdentity: 4, Effect: 1, EvidenceRecord: 4, Operation: 4, Program: 1, ProgramStep: 4, Selector: 2, SourceRecord: 1, ValueExpression: 2 },
  },
  {
    name: "yao-guang-elation",
    schemaRevision: "starclock.v1a-yao-guang-elation-probe.v1",
    payload: ".cache/content-reference/turnbasedgamedata/Config/ConfigAbility/Avatar/Avatar_YaoGuang_00_Ability.json",
    payloadSha256: "6f604605ed512ec5f9f7b9cfb048c3ee896b18ee2073b0b0513b8d65f83d017c",
    textSha256: {
      elation_damage: "9edca7b273bf822d09bbabe4ca7d94df617a74c66f0e64f6b1c681e1dfcd2662",
      ultimate: "0e2c95f9423c1a42ff615df6292d35c870fffffacde875f42ce145a14c2440e5",
      passive: "10a04a1c62ad3721cd31a1cbe0daeb329035e093f70a44264f3c00eca3c1dbc5",
      fixture_contract: "2d1f35de96098c655bf19a263984f3ac3c516e9609ef383311545100f91048a0",
    },
    counts: { Ability: 2, AbilityPhase: 2, ConfigManifest: 1, ContentEvidenceBinding: 4, ContentIdentity: 4, EvidenceRecord: 4, LinkedUnitDefinition: 1, Operation: 4, Program: 1, ProgramStep: 4, Selector: 3, SourceRecord: 1, ValueExpression: 2 },
  },
];

fs.rmSync(work, { recursive: true, force: true });
fs.mkdirSync(work, { recursive: true });
for (const probe of probes) verifyProbe(probe);

function verifyProbe(probe) {
  const fixture = path.join(root, "config/probes/v1a", probe.name);
  const sourcePayload = path.join(root, probe.payload);
  assert(fs.existsSync(sourcePayload), `prepared ${probe.name} source payload is absent`);
  const sourcePayloadSha256 = sha256(sourcePayload);
  assert(sourcePayloadSha256 === probe.payloadSha256, `prepared ${probe.name} source payload drifted`);

  const first = generate(probe.name, "first");
  const second = generate(probe.name, "second");
  assertSameFile(first.bundle, second.bundle, `independent ${probe.name} bundles differ`);
  assertSameTree(first.debug, second.debug, `independent ${probe.name} debug exports differ`);
  const counts = Object.fromEntries(fs.readdirSync(first.debug).filter((file) => file.endsWith(".json")).sort().map((file) => [path.basename(file, ".json"), readJson(path.join(first.debug, file)).table.rows.length]));
  assert(Object.keys(counts).length === 82, `${probe.name} does not export all production tables`);
  for (const [name, count] of Object.entries(counts)) assert(count === (probe.counts[name] ?? 0), `${probe.name} ${name} row count differs`);

  const golden = {
    schema_revision: probe.schemaRevision,
    sora_cli_version: policy.version,
    reference_pack_sha256: "0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a",
    source_payload_sha256: sourcePayloadSha256,
    source_text_sha256: probe.textSha256,
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
    console.log(`Blessed ${probe.name} probe (${golden.bundle_sha256}).`);
  } else {
    assert(fs.existsSync(bundle) && fs.existsSync(expected), `${probe.name} probe is not blessed`);
    assertSameFile(first.bundle, bundle, `committed ${probe.name} bundle drifted`);
    assert(JSON.stringify(readJson(expected)) === JSON.stringify(golden), `${probe.name} golden drifted`);
    console.log(`Verified ${probe.name} probe (${golden.bundle_sha256}).`);
  }
}

function generate(probe, name) { const data = path.join(work, probe, name, "data"); const out = path.join(work, probe, name, "out"); run("cargo", ["run", "--manifest-path", "tools/workbook-bootstrap/Cargo.toml", "--locked", "--quiet", "--", "config/generated/templates", `config/probes/v1a/${probe}/rows`, path.relative(root, data)]); fs.mkdirSync(out, { recursive: true }); const bundle = path.join(out, "config.sora"); const debug = path.join(out, "debug-json"); run(sora, ["--serial", "export", "--format", "binary", "--project", "config/project.toml", "--data-root", data, "--out", bundle]); run(sora, ["--serial", "export", "--format", "json-debug", "--project", "config/project.toml", "--data-root", data, "--out", debug]); return { bundle, debug }; }
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
