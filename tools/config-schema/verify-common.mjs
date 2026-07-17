import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { canonicalDecimalToMillionths, isCanonicalDecimal } from "./canonical-decimal.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const arguments_ = process.argv.slice(2);
assert(arguments_.every((argument) => argument === "--bless"), "usage: verify-common.mjs [--bless]");
const bless = arguments_.includes("--bless");
const policy = readJson(path.join(root, "policy/config-schema.json"));
const toolPolicy = readJson(path.join(root, "policy/sora-toolchain.json"));
assert(policy.schema_revision === "starclock.config-schema-policy.v1", "unexpected config schema policy revision");
assert(policy.transport_id.type === "i32" && policy.transport_id.minimum === 1 && policy.transport_id.maximum === 2_147_483_647, "transport ID policy changed");
assert(policy.canonical_decimal.raw_scale === 1_000_000 && policy.canonical_decimal.maximum_fractional_digits === 6, "decimal scale policy changed");

const fixture = path.join(root, "config/schema-fixtures/common");
const expected = path.join(fixture, "expected");
const work = path.join(root, ".cache/common-schema-work");
const project = path.join(work, "config/schema-fixtures/common");
assert(path.relative(root, work).replaceAll("\\", "/") === ".cache/common-schema-work", "unexpected common-schema work path");

fs.rmSync(work, { recursive: true, force: true });
fs.mkdirSync(path.join(work, "config/schema"), { recursive: true });
fs.mkdirSync(project, { recursive: true });
fs.cpSync(path.join(root, "config/schema"), path.join(work, "config/schema"), { recursive: true });
for (const name of ["project.toml", "schema", "data"]) {
  const source = path.join(fixture, name);
  const target = path.join(project, name);
  fs.cpSync(source, target, { recursive: true });
}

const sora = resolveSora(toolPolicy);
assert(capture(sora, ["--version"]).stdout.trim() === `sora ${toolPolicy.version}`, "installed Sora version differs from policy");
run(sora, ["--serial", "check", "--project", "./project.toml"]);
run(sora, ["--serial", "build", "--project", "./project.toml", "--clean"]);
formatRust(path.join(project, "generated/rust"));
verifySchemaLock(path.join(project, "generated/schema.lock"));
verifyFixtureOutput(path.join(project, "generated/debug-json"));
const firstBuild = artifactHashes(path.join(project, "generated"));

const direct = path.join(project, "direct");
run(sora, ["--serial", "schema-lock", "--project", "./project.toml", "--out", "direct/schema.lock"]);
run(sora, ["--serial", "excel-template", "--project", "./project.toml", "--out", "direct/excel"]);
assertTemplateList(path.join(direct, "excel"));
run(sora, ["--serial", "gen", "--target", "rust", "--project", "./project.toml", "--out", "direct/rust", "--format-code", "never"]);
formatRust(path.join(direct, "rust"));
run(sora, ["--serial", "export", "--format", "binary", "--project", "./project.toml", "--data-root", "data", "--out", "direct/config.sora"]);
run(sora, ["--serial", "export", "--format", "json-debug", "--project", "./project.toml", "--data-root", "data", "--out", "direct/debug-json"]);
assertSameFile(path.join(project, "generated/schema.lock"), path.join(direct, "schema.lock"), "direct schema lock differs from configured build");
assertSameTree(path.join(project, "generated/rust"), path.join(direct, "rust"), "direct Rust codegen differs from configured build");
assertSameFile(path.join(project, "generated/config.sora"), path.join(direct, "config.sora"), "direct binary export differs from configured build");
assertSameTree(path.join(project, "generated/debug-json"), path.join(direct, "debug-json"), "direct diagnostic export differs from configured build");

run(sora, ["--serial", "build", "--project", "./project.toml", "--clean"]);
formatRust(path.join(project, "generated/rust"));
assertMapsEqual(firstBuild, artifactHashes(path.join(project, "generated")), "second common-schema build drifted");
verifyCanonicalDecimalPolicy();
verifyNegativeData();

const actualFiles = artifactFiles(path.join(project, "generated"));
if (bless) {
  fs.rmSync(expected, { recursive: true, force: true });
  for (const relative of actualFiles) {
    const destination = path.join(expected, relative);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.copyFileSync(path.join(project, "generated", relative), destination);
  }
  const files = Object.fromEntries(actualFiles.map((relative) => [relative, sha256(path.join(project, "generated", relative))]));
  const manifest = {
    schema_revision: "starclock.common-schema-golden.v1",
    sora_cli_version: toolPolicy.version,
    output_digest: digestFileMap(files),
    files
  };
  fs.writeFileSync(path.join(fixture, "expected-manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  console.log(`Blessed common schema golden (${actualFiles.length} files; ${manifest.output_digest}).`);
} else {
  const manifest = readJson(path.join(fixture, "expected-manifest.json"));
  assert(manifest.schema_revision === "starclock.common-schema-golden.v1", "unexpected common-schema golden revision");
  assert(manifest.sora_cli_version === toolPolicy.version, "common-schema golden uses another Sora version");
  const expectedFiles = artifactFiles(expected);
  assert(JSON.stringify(actualFiles) === JSON.stringify(expectedFiles), "common-schema golden file list drifted");
  const actualMap = Object.fromEntries(actualFiles.map((relative) => [relative, sha256(path.join(project, "generated", relative))]));
  const expectedMap = Object.fromEntries(expectedFiles.map((relative) => [relative, sha256(path.join(expected, relative))]));
  assert(JSON.stringify(actualMap) === JSON.stringify(expectedMap), "common-schema golden bytes drifted");
  assert(JSON.stringify(actualMap) === JSON.stringify(manifest.files), "common-schema manifest hashes drifted");
  assert(digestFileMap(actualMap) === manifest.output_digest, "common-schema output digest drifted");
  console.log(`Common schema golden verified (${actualFiles.length} files; ${manifest.output_digest}).`);
}

function verifySchemaLock(file) {
  const lock = readJson(file);
  const schema = lock.schema;
  assert(schema.package === "starclock_common_schema_fixture", "schema lock package differs");
  const enumNames = schema.enums.map((entry) => entry.name);
  for (const name of ["ContentKind", "ReleaseState", "CoverageState", "SourceCategory", "Confidence", "EvidenceKind", "FactQuality", "MechanismQuality"]) {
    assert(enumNames.includes(name), `schema lock lacks enum ${name}`);
  }
  const tables = new Map(schema.tables.map((table) => [table.name, table]));
  for (const name of ["SourceRecord", "EvidenceRecord", "ContentIdentity", "ContentEvidenceBinding", "ConfigManifest", "CanonicalDecimalFixture"]) {
    assert(tables.has(name), `schema lock lacks table ${name}`);
  }
  for (const table of tables.values()) {
    for (const field of table.fields) {
      const encodedType = JSON.stringify(field.ty);
      assert(!encodedType.includes("F32") && !encodedType.includes("F64"), `${table.name}.${field.name} exposes an authoritative float`);
      if (field.name === "id") {
        assert(field.ty === "I32" && JSON.stringify(field.range) === "[1,2147483647]", `${table.name}.id violates transport policy`);
      }
      if (field.name.endsWith(policy.canonical_decimal.field_suffix)) {
        assert(field.ty === "String", `${table.name}.${field.name} must transport a string`);
        assert(JSON.stringify(field.length) === "[1,32]", `${table.name}.${field.name} has the wrong source length`);
      }
    }
  }
  assert(tables.get("ContentIdentity").indexes.some((index) => index.name === "by_stable_key" && index.unique), "content stable keys are not unique");
  assert(tables.get("ContentEvidenceBinding").indexes.filter((index) => index.unique).length === 2, "fact evidence bindings lack unique identity/sequence and identity/fact constraints");
}

function verifyFixtureOutput(directory) {
  const decimalRows = readJson(path.join(directory, "CanonicalDecimalFixture.json")).table.rows;
  assert(decimalRows.length === 1, "canonical decimal fixture row is absent");
  const values = decimalRows[0].values;
  assert(values.ratio_decimal.String === "0.25", "ratio decimal changed");
  assert(values.signed_decimal.String === "-12.5", "signed decimal changed");
  assert(values.upper_precision_decimal.String === "123456.000001", "six-place decimal changed");
  const source = readJson(path.join(directory, "SourceRecord.json")).table.rows[0].values;
  const evidence = readJson(path.join(directory, "EvidenceRecord.json")).table.rows[0].values;
  assert(isSha256(source.evidence_sha256.String), "source evidence digest is not lowercase SHA-256");
  assert(isSha256(evidence.sha256.String), "evidence record digest is not lowercase SHA-256");
  const identities = readJson(path.join(directory, "ContentIdentity.json")).table.rows;
  assert(identities.length === 1 && identities[0].values.enabled.Bool === false, "synthetic identity must stay disabled");
  assert(identities[0].values.coverage_state.String === "Disabled", "synthetic identity entered production coverage");
}

function verifyCanonicalDecimalPolicy() {
  const accepted = new Map([
    ["0", 0n],
    ["0.25", 250_000n],
    ["-12.5", -12_500_000n],
    ["123456.000001", 123_456_000_001n],
    ["9223372036854.775807", 9_223_372_036_854_775_807n],
    ["-9223372036854.775808", -9_223_372_036_854_775_808n]
  ]);
  for (const [source, raw] of accepted) assert(canonicalDecimalToMillionths(source) === raw, `decimal ${source} mapped incorrectly`);
  for (const source of ["", "+1", "01", "-0", "1.", ".5", "1.0", "1.0000000", "1e-3", "1,5", "NaN", "∞", "9223372036854.775808", "-9223372036854.775809"]) {
    assert(!isCanonicalDecimal(source), `noncanonical decimal ${source} was accepted`);
  }
  assert(policy.canonical_decimal.grammar === "^-?(?:0|[1-9][0-9]*)(?:\\.[0-9]{0,5}[1-9])?$", "machine-readable decimal grammar differs from parser");
  assert(!isSha256("A".repeat(64)) && !isSha256("a".repeat(63)) && !isSha256("g".repeat(64)), "SHA-256 syntax check is too broad");
}

function assertTemplateList(directory) {
  const actual = fs.readdirSync(directory, { withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => entry.name)
    .sort();
  const expectedNames = [
    "CanonicalDecimalFixture.xlsx",
    "ConfigManifest.xlsx",
    "ContentEvidenceBinding.xlsx",
    "ContentIdentity.xlsx",
    "EvidenceRecord.xlsx",
    "SourceRecord.xlsx"
  ];
  assert(JSON.stringify(actual) === JSON.stringify(expectedNames), `common-schema Excel template list differs: ${actual.join(", ")}`);
}

function verifyNegativeData() {
  expectDataFailure("ContentIdentity.toml", (source) => source.replace('name_zh_cn = "通用模式测试夹具"', 'name_zh_cn = ""'), ["name_zh_cn", "outside"]);
  expectDataFailure("ContentEvidenceBinding.toml", (source) => source.replace("content_id = 1", "content_id = 999"), ["content_id", "reference"]);
  expectDataFailure("SourceRecord.toml", (source) => `${source}\n${source.replace("id = 1", "id = 2")}`, ["by_stable_key", "duplicate"]);
}

function expectDataFailure(name, mutate, needles) {
  const file = path.join(project, "data", name);
  const original = fs.readFileSync(file, "utf8");
  const changed = mutate(original);
  assert(changed !== original, `negative fixture ${name} did not mutate`);
  try {
    fs.writeFileSync(file, changed);
    const negativeOutput = path.join(project, "negative", `${path.parse(name).name}.json`);
    fs.rmSync(negativeOutput, { force: true });
    const result = spawnSync(sora, ["--serial", "export", "--format", "json-debug", "--project", "./project.toml", "--data-root", "data", "--out", negativeOutput], { cwd: project, encoding: "utf8" });
    if (result.error) throw result.error;
    assert(result.status !== 0, `negative fixture ${name} unexpectedly passed`);
    const report = `${result.stdout}\n${result.stderr}`;
    for (const needle of needles) assert(report.toLowerCase().includes(needle.toLowerCase()), `negative fixture ${name} did not report ${needle}: ${report}`);
  } finally {
    fs.writeFileSync(file, original);
  }
}

function resolveSora(tool) {
  const binary = path.join(root, tool.install_root, "bin", process.platform === "win32" ? "sora.exe" : "sora");
  assert(fs.existsSync(binary), `Sora ${tool.version} is not installed; run ${tool.install_command}`);
  return binary;
}
function run(command, args) {
  const result = spawnSync(command, args, { cwd: project, stdio: "inherit" });
  if (result.error) throw result.error;
  assert(result.status === 0, `${relativeCommand(command)} ${args.join(" ")} exited with ${result.status}`);
}
function capture(command, args) {
  const result = spawnSync(command, args, { cwd: project, encoding: "utf8" });
  if (result.error) throw result.error;
  assert(result.status === 0, `${relativeCommand(command)} ${args.join(" ")} exited with ${result.status}: ${result.stderr}`);
  return result;
}
function formatRust(directory) {
  run("rustfmt", ["--edition", "2024", ...walk(directory).filter((file) => file.endsWith(".rs"))]);
}
function artifactFiles(directory) { return walk(directory).map((file) => path.relative(directory, file).replaceAll("\\", "/")).sort(); }
function artifactHashes(directory) { return new Map(artifactFiles(directory).map((relative) => [relative, sha256(path.join(directory, relative))])); }
function walk(directory) {
  assert(fs.existsSync(directory), `missing directory ${directory}`);
  return fs.readdirSync(directory, { withFileTypes: true }).sort((left, right) => left.name.localeCompare(right.name)).flatMap((entry) => {
    const target = path.join(directory, entry.name);
    return entry.isDirectory() ? walk(target) : entry.isFile() ? [target] : [];
  });
}
function assertSameFile(left, right, message) { assert(sha256(left) === sha256(right), message); }
function assertSameTree(left, right, message) { assertMapsEqual(artifactHashes(left), artifactHashes(right), message); }
function assertMapsEqual(left, right, message) { assert(JSON.stringify([...left]) === JSON.stringify([...right]), message); }
function digestFileMap(files) {
  const canonical = Object.entries(files).sort(([left], [right]) => left.localeCompare(right)).map(([name, hash]) => `${name}\0${hash}\n`).join("");
  return crypto.createHash("sha256").update(canonical, "utf8").digest("hex");
}
function sha256(file) { return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex"); }
function isSha256(value) { return typeof value === "string" && /^[0-9a-f]{64}$/u.test(value); }
function readJson(file) { return JSON.parse(fs.readFileSync(file, "utf8")); }
function relativeCommand(command) { return path.isAbsolute(command) ? path.relative(root, command).replaceAll("\\", "/") : command; }
function assert(condition, message) { if (!condition) throw new Error(message); }
