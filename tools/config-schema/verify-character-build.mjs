import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { canonicalDecimalToMillionths, isCanonicalDecimal } from "./canonical-decimal.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const arguments_ = process.argv.slice(2);
assert(arguments_.every((argument) => argument === "--bless"), "usage: verify-character-build.mjs [--bless]");
const bless = arguments_.includes("--bless");
const toolPolicy = readJson(path.join(root, "policy/sora-toolchain.json"));
const fixture = path.join(root, "config/schema-fixtures/character-build");
const work = path.join(root, ".cache/character-build-schema-work");
const project = path.join(work, "config/schema-fixtures/character-build");
assert(path.relative(root, work).replaceAll("\\", "/") === ".cache/character-build-schema-work", "unexpected work path");

fs.rmSync(work, { recursive: true, force: true });
fs.mkdirSync(path.join(work, "config/schema"), { recursive: true });
fs.mkdirSync(project, { recursive: true });
fs.cpSync(path.join(root, "config/schema"), path.join(work, "config/schema"), { recursive: true });
prepareTomlSchemas(path.join(work, "config/schema"));
for (const name of ["project.toml", "data"]) fs.cpSync(path.join(fixture, name), path.join(project, name), { recursive: true });

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
assertSameFile(path.join(project, "generated/schema.lock"), path.join(direct, "schema.lock"), "direct schema lock differs");
assertSameTree(path.join(project, "generated/rust"), path.join(direct, "rust"), "direct Rust codegen differs");
assertSameFile(path.join(project, "generated/config.sora"), path.join(direct, "config.sora"), "direct binary export differs");
assertSameTree(path.join(project, "generated/debug-json"), path.join(direct, "debug-json"), "direct JSON export differs");

run(sora, ["--serial", "build", "--project", "./project.toml", "--clean"]);
formatRust(path.join(project, "generated/rust"));
assertMapsEqual(firstBuild, artifactHashes(path.join(project, "generated")), "second character/build schema build drifted");
verifyNegativeData();

const actualFiles = artifactFiles(path.join(project, "generated"));
const actualMap = Object.fromEntries(actualFiles.map((relative) => [relative, sha256(path.join(project, "generated", relative))]));
const outputDigest = digestFileMap(actualMap);
if (bless) {
  const manifest = {
    schema_revision: "starclock.character-build-schema-golden.v1",
    sora_cli_version: toolPolicy.version,
    output_digest: outputDigest,
    files: actualMap
  };
  fs.writeFileSync(path.join(fixture, "expected-manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  console.log(`Blessed character/build schema golden (${actualFiles.length} files; ${outputDigest}).`);
} else {
  const manifest = readJson(path.join(fixture, "expected-manifest.json"));
  assert(manifest.schema_revision === "starclock.character-build-schema-golden.v1", "unexpected golden revision");
  assert(manifest.sora_cli_version === toolPolicy.version, "golden uses another Sora version");
  assert(JSON.stringify(actualMap) === JSON.stringify(manifest.files), "character/build golden bytes drifted");
  assert(outputDigest === manifest.output_digest, "character/build golden digest drifted");
  console.log(`Character/build schema golden verified (${actualFiles.length} files; ${outputDigest}).`);
}

function verifySchemaLock(file) {
  const schema = readJson(file).schema;
  assert(schema.package === "starclock_character_build_schema_fixture", "schema lock package differs");
  const tableNames = new Set(schema.tables.map((table) => table.name));
  for (const name of expectedTables()) assert(tableNames.has(name), `schema lock lacks table ${name}`);
  const union = schema.unions.find((candidate) => candidate.name === "BuildPatch");
  assert(union?.tag === "type", "BuildPatch is not a tagged union");
  assert(JSON.stringify(union.variants.map((variant) => variant.name)) === JSON.stringify([
    "AddRule", "RemoveRule", "AddModifier", "AddAbility", "ReplaceAbility", "PatchAbility",
    "AdjustAbilityLevel", "AdjustResourceDefinition", "AdjustStateSlot", "AddTag"
  ]), "BuildPatch variants are not the reviewed closed set");

  const authoritativeFields = [
    ...schema.tables.flatMap((table) => table.fields.map((field) => [`${table.name}.${field.name}`, field])),
    ...schema.unions.flatMap((item) => item.variants.flatMap((variant) => variant.fields.map((field) => [`${item.name}.${variant.name}.${field.name}`, field])))
  ];
  for (const [name, field] of authoritativeFields) {
    const encodedType = JSON.stringify(field.ty);
    assert(!encodedType.includes("F32") && !encodedType.includes("F64"), `${name} exposes an authoritative float`);
    if (field.name.endsWith("_decimal")) {
      assert(field.ty === "String", `${name} must transport a string`);
      assert(JSON.stringify(field.length) === "[1,32]", `${name} has the wrong decimal source length`);
    }
  }
  const tables = new Map(schema.tables.map((table) => [table.name, table]));
  for (const [tableName, fieldName, target] of [
    ["Character", "id", "ContentIdentity"], ["Ability", "id", "ContentIdentity"],
    ["TraceNode", "id", "ContentIdentity"], ["Eidolon", "id", "ContentIdentity"],
    ["LightCone", "id", "ContentIdentity"], ["CharacterAbilityBinding", "ability_id", "Ability"],
    ["TracePatch", "trace_id", "TraceNode"], ["EidolonPatch", "eidolon_id", "Eidolon"],
    ["LightConeSuperimposition", "light_cone_id", "LightCone"]
  ]) {
    const field = tables.get(tableName).fields.find((candidate) => candidate.name === fieldName);
    assert(field?.ty?.Ref?.table === target, `${tableName}.${fieldName} is not a typed ${target} reference`);
  }
  const prerequisites = tables.get("TraceNode").fields.find((field) => field.name === "prerequisite_trace_ids");
  assert(prerequisites.ty.List.Ref.table === "TraceNode", "Trace prerequisites are not typed self-references");
}

function verifyFixtureOutput(directory) {
  const identities = rows(directory, "ContentIdentity");
  assert(identities.length === 16, "fixture identity count differs");
  for (const row of identities) {
    assert(value(row, "enabled") === false, "synthetic identity became enabled");
    assert(value(row, "release_state") === "ProjectFixture", "synthetic identity gained a release state");
    assert(value(row, "coverage_state") === "Disabled", "synthetic identity entered coverage");
  }
  assertCompleteRanks(rows(directory, "Eidolon").map((row) => value(row, "rank")), 6, "Eidolon");
  const superimpositionRows = rows(directory, "LightConeSuperimposition");
  assertSuperimpositionRanks(superimpositionRows.map((row) => value(row, "rank")), superimpositionRows.map((row) => value(row, "constant_across_ranks")));
  expectAssertion(() => assertCompleteRanks([1, 2, 3, 4, 5], 6, "negative Eidolon"), "missing E6 completeness was accepted");
  expectAssertion(() => assertSuperimpositionRanks([1, 2, 3, 4], [false, false, false, false]), "missing S5 completeness was accepted");
  assertSuperimpositionRanks([1], [true]);

  const traceRows = rows(directory, "TraceNode");
  assert(JSON.stringify(value(traceRows[0], "prerequisite_trace_ids")) === "[]", "root Trace gained a prerequisite");
  assert(JSON.stringify(value(traceRows[1], "prerequisite_trace_ids")) === "[5]", "Trace self-reference differs");
  const hitRows = rows(directory, "HitPlanHit");
  assert(hitRows.map((row) => value(row, "sequence")).join(",") === "1,2", "hit ordering differs");
  for (const field of ["damage_ratio_decimal", "toughness_ratio_decimal"]) {
    const values = hitRows.map((row) => value(row, field));
    assert(values.every(isCanonicalDecimal), `${field} is not canonical`);
    assert(values.reduce((sum, source) => sum + canonicalDecimalToMillionths(source), 0n) === 1_000_000n, `${field} does not sum to one`);
  }
  assert(rows(directory, "CharacterAbilityBinding").some((row) => value(row, "slot") === "Technique"), "Technique binding is absent");
  assert(rows(directory, "TracePatch").every((row) => typeof value(row, "patch").type === "string"), "Trace patch union did not export");
}

function verifyNegativeData() {
  expectDataFailure("TraceNode.toml", (source) => source.replace("prerequisite_trace_ids = [5]", "prerequisite_trace_ids = [999]"));
  expectDataFailure("Eidolon.toml", (source) => source.replace("id = 12\ncharacter_id = 1\nrank = 6", "id = 12\ncharacter_id = 1\nrank = 5"));
  expectDataFailure("CharacterAbilityBinding.toml", (source) => source.replace("ability_id = 3", "ability_id = 999"));
}

function expectDataFailure(name, mutate) {
  const file = path.join(project, "data", name);
  const original = fs.readFileSync(file, "utf8");
  const changed = mutate(original);
  assert(changed !== original, `negative fixture ${name} did not mutate`);
  try {
    fs.writeFileSync(file, changed);
    const result = spawnSync(sora, ["--serial", "export", "--format", "json-debug", "--project", "./project.toml", "--data-root", "data", "--out", `negative/${path.parse(name).name}`], { cwd: project, encoding: "utf8" });
    if (result.error) throw result.error;
    assert(result.status !== 0, `negative fixture ${name} unexpectedly passed`);
  } finally {
    fs.writeFileSync(file, original);
  }
}

function assertCompleteRanks(actual, maximum, label) {
  const sorted = [...actual].sort((left, right) => left - right);
  assert(JSON.stringify(sorted) === JSON.stringify(Array.from({ length: maximum }, (_, index) => index + 1)), `${label} ranks are incomplete`);
}

function assertSuperimpositionRanks(ranks, constantFlags) {
  assert(ranks.length === constantFlags.length && ranks.length > 0, "superimposition rows are empty or misaligned");
  if (constantFlags.every(Boolean)) {
    assert(JSON.stringify(ranks) === "[1]", "constant superimposition must be one S1 row");
  } else {
    assert(constantFlags.every((flag) => !flag), "superimposition mixes constant and scalable rows");
    assertCompleteRanks(ranks, 5, "Light Cone superimposition");
  }
}

function assertTemplateList(directory) {
  const actual = fs.readdirSync(directory, { withFileTypes: true }).filter((entry) => entry.isFile()).map((entry) => entry.name).sort();
  const expected = expectedTables().map((name) => `${name}.xlsx`).sort();
  assert(JSON.stringify(actual) === JSON.stringify(expected), `Excel template list differs: ${actual.join(", ")}`);
}

function expectedTables() {
  return [
    "SourceRecord", "EvidenceRecord", "ContentIdentity", "ContentEvidenceBinding", "ConfigManifest",
    "Ability", "AbilityResourceDelta", "AbilityLevelParameter", "AbilityPhase", "HitPlan", "HitPlanHit", "AbilityHitPlanBinding",
    "Character", "CharacterStat", "CharacterResource", "CharacterAbilityBinding", "TraceNode", "TracePatch", "Eidolon", "EidolonPatch",
    "LightCone", "LightConeStat", "LightConeSuperimposition"
  ];
}

function rows(directory, name) { return readJson(path.join(directory, `${name}.json`)).table.rows; }
function value(row, name) {
  const encoded = row.values[name];
  if ("Integer" in encoded) return encoded.Integer;
  if ("String" in encoded) return encoded.String;
  if ("Bool" in encoded) return encoded.Bool;
  if ("List" in encoded) return encoded.List.map((item) => "Integer" in item ? item.Integer : item.String);
  if ("Object" in encoded) return Object.fromEntries(Object.entries(encoded.Object).map(([key, item]) => [key, "Integer" in item ? item.Integer : "String" in item ? item.String : item.Bool]));
  throw new Error(`unsupported diagnostic value for ${name}`);
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
function formatRust(directory) { run("rustfmt", ["--edition", "2024", ...walk(directory).filter((file) => file.endsWith(".rs"))]); }
function artifactFiles(directory) { return walk(directory).map((file) => path.relative(directory, file).replaceAll("\\", "/")).sort(); }
function artifactHashes(directory) { return new Map(artifactFiles(directory).map((relative) => [relative, sha256(path.join(directory, relative))])); }
function walk(directory) {
  assert(fs.existsSync(directory), `missing directory ${directory}`);
  return fs.readdirSync(directory, { withFileTypes: true }).sort((left, right) => left.name.localeCompare(right.name)).flatMap((entry) => {
    const target = path.join(directory, entry.name);
    return entry.isDirectory() ? walk(target) : [target];
  });
}
function assertSameFile(left, right, message) { assert(fs.readFileSync(left).equals(fs.readFileSync(right)), message); }
function assertSameTree(left, right, message) {
  const leftFiles = artifactFiles(left);
  const rightFiles = artifactFiles(right);
  assert(JSON.stringify(leftFiles) === JSON.stringify(rightFiles), `${message}: file lists differ`);
  for (const relative of leftFiles) assertSameFile(path.join(left, relative), path.join(right, relative), `${message}: ${relative}`);
}
function assertMapsEqual(left, right, message) { assert(JSON.stringify([...left]) === JSON.stringify([...right]), message); }
function sha256(file) { return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex"); }
function digestFileMap(files) { return crypto.createHash("sha256").update(Object.entries(files).map(([name, digest]) => `${name}\0${digest}\n`).join(""), "utf8").digest("hex"); }
function readJson(file) { return JSON.parse(fs.readFileSync(file, "utf8")); }
function relativeCommand(command) { return path.relative(root, command).replaceAll("\\", "/") || command; }
function expectAssertion(action, message) {
  let failed = false;
  try { action(); } catch { failed = true; }
  assert(failed, message);
}
function assert(condition, message) { if (!condition) throw new Error(message); }

function prepareTomlSchemas(directory) {
  for (const file of walk(directory).filter((candidate) => candidate.endsWith(".toml"))) {
    const source = fs.readFileSync(file, "utf8");
    fs.writeFileSync(file, source.replaceAll('format = "xlsx"', 'format = "toml"').replace(/file = "([A-Za-z0-9_-]+)\.xlsx"/g, 'file = "$1.toml"'));
  }
}
