import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const arguments_ = process.argv.slice(2);
assert(arguments_.every((argument) => argument === "--bless"), "usage: verify.mjs [--bless]");
const bless = arguments_.includes("--bless");
const work = path.join(root, ".cache/config-production-verify");
const projectRoot = path.join(work, "config");
const expectedFile = path.join(root, "config/production-golden.json");
assert(path.relative(root, work).replaceAll("\\", "/") === ".cache/config-production-verify", "unexpected verification work path");

run("node", ["tools/config-production/generate-bootstrap-policy.mjs"]);
const toolPolicy = readJson(path.join(root, "policy/sora-toolchain.json"));
const sora = path.join(root, toolPolicy.install_root, "bin", process.platform === "win32" ? "sora.exe" : "sora");
assert(fs.existsSync(sora), `Sora ${toolPolicy.version} is not installed; run ${toolPolicy.install_command}`);
assert(capture(sora, ["--version"]).stdout.trim() === `sora ${toolPolicy.version}`, "installed Sora version differs from policy");
verifyProductionSchemaSources();
verifyNoOverwrite();

fs.rmSync(work, { recursive: true, force: true });
fs.mkdirSync(projectRoot, { recursive: true });
for (const name of ["project.toml", "schema", "data"]) fs.cpSync(path.join(root, "config", name), path.join(projectRoot, name), { recursive: true });
run(sora, ["--serial", "check", "--project", "config/project.toml"], work);
run(sora, ["--serial", "build", "--project", "config/project.toml", "--clean"], work);
formatRust(path.join(projectRoot, "generated/rust"));
verifyGeneratedOutput(path.join(projectRoot, "generated"));
verifyTemplateList(path.join(projectRoot, "generated/templates"));
verifyReadOnlySync();
verifyBootstrapReproduction();

const stable = artifactMap(path.join(projectRoot, "generated"));
const outputDigest = digestFileMap(stable);
if (bless) {
  copyGenerated(path.join(projectRoot, "generated"), path.join(root, "config/generated"));
  fs.writeFileSync(expectedFile, `${JSON.stringify({
    schema_revision: "starclock.production-config-golden.v1",
    sora_cli_version: toolPolicy.version,
    reference_pack_sha256: "0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a",
    goal_manifest_sha256: "e2188c7844d678253c98d569db017dbad7101541cf502aba4c2eb80c0435bf19",
    identity_count: 4519,
    enabled_identity_count: 4370,
    table_count: 82,
    output_digest: outputDigest,
    files: stable,
  }, null, 2)}\n`);
  console.log(`Blessed production config golden (${Object.keys(stable).length} files; ${outputDigest}).`);
} else {
  const expected = readJson(expectedFile);
  assert(expected.schema_revision === "starclock.production-config-golden.v1", "unexpected production golden revision");
  assert(expected.sora_cli_version === toolPolicy.version, "production golden uses another Sora version");
  assert(expected.reference_pack_sha256 === "0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a", "production golden reference digest differs");
  assert(expected.goal_manifest_sha256 === "e2188c7844d678253c98d569db017dbad7101541cf502aba4c2eb80c0435bf19", "production golden goal digest differs");
  assert(JSON.stringify(stable) === JSON.stringify(expected.files) && outputDigest === expected.output_digest, "production config golden drifted");
  assertMapsEqual(new Map(Object.entries(stable)), new Map(Object.entries(artifactMap(path.join(root, "config/generated")))), "committed production generated artifacts drifted");
  console.log(`Production config golden verified (${Object.keys(stable).length} files; ${outputDigest}).`);
}

function verifyProductionSchemaSources() {
  for (const file of walk(path.join(root, "config/schema")).filter((candidate) => candidate.endsWith(".toml"))) {
    const source = fs.readFileSync(file, "utf8");
    assert(!source.includes('format = "toml"') && !/file = "[A-Za-z0-9_-]+\.toml"/.test(source), `${path.relative(root, file)} retains a non-production table source`);
    assert((source.match(/format = "xlsx"/g) ?? []).length === (source.match(/\[\[tables\]\]/g) ?? []).length, `${path.relative(root, file)} lacks an xlsx source for a table`);
  }
}

function verifyNoOverwrite() {
  const before = hashTree(path.join(root, "config/data"));
  const result = spawnSync("node", ["tools/config-production/bootstrap.mjs", "--output", "config/data"], { cwd: root, encoding: "utf8" });
  if (result.error) throw result.error;
  assert(result.status !== 0 && `${result.stdout}\n${result.stderr}`.includes("refusing to overwrite"), "bootstrap did not reject the designer workbook root");
  assertMapsEqual(before, hashTree(path.join(root, "config/data")), "failed bootstrap mutated designer workbooks");
}

function verifyReadOnlySync() {
  const before = hashTree(path.join(root, "config/data"));
  const result = capture(sora, ["--serial", "excel-sync", "--project", "config/project.toml", "--data-root", "config/data"], root);
  assertMapsEqual(before, hashTree(path.join(root, "config/data")), "read-only excel-sync mutated designer workbooks");
  assert(!`${result.stdout}\n${result.stderr}`.includes("add columns"), "designer workbooks need schema synchronization");
}

function verifyBootstrapReproduction() {
  const first = path.join(work, "bootstrap-a");
  const second = path.join(work, "bootstrap-b");
  run("node", ["tools/config-production/bootstrap.mjs", "--output", path.relative(root, first)], root);
  run("node", ["tools/config-production/bootstrap.mjs", "--output", path.relative(root, second)], root);
  const firstOut = path.join(work, "direct-a");
  const secondOut = path.join(work, "direct-b");
  for (const [data, out] of [[first, firstOut], [second, secondOut]]) {
    run(sora, ["--serial", "export", "--format", "binary", "--project", "config/project.toml", "--data-root", data, "--out", path.join(out, "config.sora")], work);
    run(sora, ["--serial", "export", "--format", "json-debug", "--project", "config/project.toml", "--data-root", data, "--out", path.join(out, "debug-json")], work);
  }
  assertSameFile(path.join(firstOut, "config.sora"), path.join(secondOut, "config.sora"), "two bootstrap exports differ");
  assertSameTree(path.join(firstOut, "debug-json"), path.join(secondOut, "debug-json"), "two bootstrap diagnostic exports differ");
  verifyBootstrapOutput(path.join(firstOut, "debug-json"));
}

function verifyGeneratedOutput(directory) {
  const schema = readJson(path.join(directory, "schema.lock")).schema;
  assert(schema.package === "starclock_production_config" && schema.tables.length === 82, "production schema lock differs");
  const debug = path.join(directory, "debug-json");
  const counts = new Map(schema.tables.map((table) => [table.name, rows(debug, table.name).length]));
  assert(counts.get("SourceRecord") === 2 && counts.get("EvidenceRecord") === 3, "production provenance counts differ");
  assert(counts.get("ContentIdentity") === 4519 && counts.get("ContentEvidenceBinding") === 4653 && counts.get("ConfigManifest") === 1, "production identity counts differ");
  for (const [name, expected] of Object.entries({
    Ability: 651, AbilityLevelParameter: 17606, AbilityResourceDelta: 513,
    AiGraph: 17, EnemyTemplate: 17, EnemyVariant: 17, Encounter: 6,
    StandardProfile: 1, StandardScenario: 6, HitPlan: 354,
    Character: 88, CharacterStat: 7568, CharacterResource: 46,
    CharacterAbilityBinding: 583, TraceNode: 1618, TracePatch: 894, Eidolon: 528, EidolonPatch: 412,
    Effect: 4, EffectGrantedAbility: 3, EffectModifierBinding: 1, ModifierDefinition: 966,
    ModifierStackingGroup: 14, ModifierFilter: 151,
    CountdownDefinition: 1, LinkedUnitDefinition: 1,
    Operation: 30, Program: 14, ProgramStep: 35, RuleDefinition: 19, RuleSourceTag: 0, Selector: 42,
    StateSlot: 3, ValueExpression: 1006, LightCone: 16, LightConeStat: 1376,
    LightConeSuperimposition: 265,
  })) assert(counts.get(name) === expected, `${name} production count differs`);
  const identities = rows(debug, "ContentIdentity");
  assert(identities.every((row) => value(row, "release_state") === "Released"), "production identities must be released");
  assert(identities.filter((row) => value(row, "enabled") === true).length === 4370, "production enabled identity count differs");
  const coverage = Object.groupBy(identities, (row) => value(row, "coverage_state"));
  assert((coverage.GoldenVerified?.length ?? 0) === 134 && (coverage.DataReady?.length ?? 0) === 4236, "released content coverage states differ");
  const rust = walk(path.join(directory, "rust")).filter((file) => file.endsWith(".rs")).map((file) => fs.readFileSync(file, "utf8")).join("\n");
  assert(!rust.includes("serde_json") && !rust.includes("json-debug"), "generated runtime reader gained a JSON path");
  const boundary = fs.readFileSync(path.join(root, "crates/starclock-data/src/bundle.rs"), "utf8");
  assert(boundary.includes("SoraBundle::parse") && !boundary.includes("serde_json") && !boundary.includes("read_to_string"), "starclock-data boundary does not exclusively load Sora binary bytes");
}

function verifyBootstrapOutput(debug) {
  const identities = rows(debug, "ContentIdentity");
  assert(rows(debug, "SourceRecord").length === 2 && rows(debug, "EvidenceRecord").length === 3, "bootstrap provenance counts differ");
  assert(identities.length === 283 && rows(debug, "ContentEvidenceBinding").length === 283, "bootstrap identity counts differ");
  assert(identities.every((row) => value(row, "release_state") === "Released" && value(row, "enabled") === false), "bootstrap identities must remain released and disabled");
}

function verifyTemplateList(directory) {
  const templates = fs.readdirSync(directory, { withFileTypes: true }).filter((entry) => entry.isFile()).map((entry) => entry.name).sort();
  assert(templates.length === 82 && templates.every((name) => name.endsWith(".xlsx")), "production template file list differs");
  const data = fs.readdirSync(path.join(root, "config/data"), { withFileTypes: true }).filter((entry) => entry.isFile()).map((entry) => entry.name).sort();
  assert(JSON.stringify(data) === JSON.stringify(templates), "designer workbook layout differs from schema template layout");
}

function copyGenerated(source, destination) {
  fs.mkdirSync(destination, { recursive: true });
  for (const relative of stableFiles(source)) {
    const target = path.join(destination, relative);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.copyFileSync(path.join(source, relative), target);
  }
  const templateDestination = path.join(destination, "templates");
  fs.rmSync(templateDestination, { recursive: true, force: true });
  fs.cpSync(path.join(source, "templates"), templateDestination, { recursive: true });
}
function artifactMap(directory) { return Object.fromEntries(stableFiles(directory).map((relative) => [relative, sha256(path.join(directory, relative))])); }
function stableFiles(directory) { return walk(directory).map((file) => path.relative(directory, file).replaceAll("\\", "/")).filter((relative) => !relative.startsWith("templates/")).sort(); }
function rows(directory, name) { return readJson(path.join(directory, `${name}.json`)).table.rows; }
function value(row, name) { const encoded = row.values[name]; if ("Integer" in encoded) return encoded.Integer; if ("String" in encoded) return encoded.String; if ("Bool" in encoded) return encoded.Bool; throw new Error(`unsupported diagnostic value ${JSON.stringify(encoded)}`); }
function formatRust(directory) { run("rustfmt", ["--edition", "2024", ...walk(directory).filter((file) => file.endsWith(".rs"))], root); }
function hashTree(directory) { return new Map(walk(directory).map((file) => [path.relative(directory, file).replaceAll("\\", "/"), sha256(file)])); }
function walk(directory) { assert(fs.existsSync(directory), `missing directory ${directory}`); return fs.readdirSync(directory, { withFileTypes: true }).sort((left, right) => left.name.localeCompare(right.name)).flatMap((entry) => { const target = path.join(directory, entry.name); return entry.isDirectory() ? walk(target) : [target]; }); }
function assertSameFile(left, right, message) { assert(fs.readFileSync(left).equals(fs.readFileSync(right)), message); }
function assertSameTree(left, right, message) { assertMapsEqual(hashTree(left), hashTree(right), message); }
function assertMapsEqual(left, right, message) { assert(JSON.stringify([...left]) === JSON.stringify([...right]), message); }
function digestFileMap(files) { return crypto.createHash("sha256").update(Object.entries(files).map(([name, digest]) => `${name}\0${digest}\n`).join(""), "utf8").digest("hex"); }
function sha256(file) { return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex"); }
function readJson(file) { return JSON.parse(fs.readFileSync(file, "utf8")); }
function run(command, args, cwd = root) { const environment = command === "cargo" ? { ...process.env, CARGO_TARGET_DIR: path.join(root, ".cache/workbook-bootstrap-target") } : process.env; const result = spawnSync(command, args, { cwd, stdio: "inherit", env: environment }); if (result.error) throw result.error; assert(result.status === 0, `${command} ${args.join(" ")} exited with ${result.status}`); }
function capture(command, args, cwd = root) { const result = spawnSync(command, args, { cwd, encoding: "utf8" }); if (result.error) throw result.error; assert(result.status === 0, `${command} ${args.join(" ")} exited with ${result.status}: ${result.stderr}`); return result; }
function assert(condition, message) { if (!condition) throw new Error(message); }
