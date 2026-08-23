import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const arguments_ = process.argv.slice(2);
assert(arguments_.length === 0, "usage: verify.mjs");
const work = path.join(root, ".cache/config-production-verify");
const projectRoot = path.join(work, "config");
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
formatRust(path.join(projectRoot, "generated/core-rust"));
verifyGeneratedOutput(path.join(projectRoot, "generated"));
verifyTemplateList(path.join(projectRoot, "generated/templates"));
verifyReadOnlySync();
verifyBootstrapReproduction();

const stable = artifactMap(path.join(projectRoot, "generated"));
assertMapsEqual(new Map(Object.entries(stable)), new Map(Object.entries(artifactMap(path.join(root, "config/generated")))), "committed production generated artifacts drifted");
console.log(`Current production config verified (${Object.keys(stable).length} generated files).`);

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
  assert(schema.project_id === "starclock_production_config" && schema.tables.length === 82, "production schema lock differs");
  const debug = path.join(directory, "debug-json");
  const counts = new Map(schema.tables.map((table) => [table.name, rows(debug, table.name).length]));
  assert(counts.get("SourceRecord") === 20 && counts.get("EvidenceRecord") === 21, "production provenance counts differ");
  assert(counts.get("ContentIdentity") === 6807 && counts.get("ContentEvidenceBinding") === 7163 && counts.get("ConfigManifest") === 1, "production identity counts differ");
  for (const [name, expected] of Object.entries({
    Ability: 1021, AbilityLevelParameter: 18317, AbilityResourceDelta: 515,
    AiGraph: 120, EnemyTemplate: 90, EnemyVariant: 90, Encounter: 6,
    StandardProfile: 1, StandardScenario: 6, HitPlan: 354,
    Character: 90, CharacterStat: 7740, CharacterResource: 46,
    CharacterAbilityBinding: 599, TraceNode: 1654, TracePatch: 925, Eidolon: 540, EidolonPatch: 423,
    Effect: 144, EffectGrantedAbility: 3, EffectModifierBinding: 20, ModifierDefinition: 1595,
    ModifierStackingGroup: 45, ModifierFilter: 155,
    CountdownDefinition: 1, LinkedUnitDefinition: 43,
    Operation: 741, Program: 433, ProgramStep: 785, RuleDefinition: 196, RuleSourceTag: 0, Selector: 251,
    StateSlot: 3, ValueExpression: 2126, LightCone: 165, LightConeStat: 14190,
    LightConeSuperimposition: 2665,
  })) assert(counts.get(name) === expected, `${name} production count differs`);
  const identities = rows(debug, "ContentIdentity");
  assert(identities.every((row) => value(row, "release_state") === "Released"), "production identities must be released");
  assert(identities.filter((row) => value(row, "enabled") === true).length === 6807, "production enabled identity count differs");
  const coverage = Object.groupBy(identities, (row) => value(row, "coverage_state"));
  assert((coverage.GoldenVerified?.length ?? 0) === 1737 && (coverage.DataReady?.length ?? 0) === 5070, "released content coverage states differ");
  const rust = walk(path.join(directory, "core-rust")).filter((file) => file.endsWith(".rs")).map((file) => fs.readFileSync(file, "utf8")).join("\n");
  assert(!rust.includes("serde_json") && !rust.includes("json-debug"), "generated runtime reader gained a JSON path");
  const boundary = fs.readFileSync(path.join(root, "crates/starclock-data/src/bundle.rs"), "utf8");
  assert(boundary.includes("SoraBundle::parse") && !boundary.includes("serde_json") && !boundary.includes("read_to_string"), "starclock-data boundary does not exclusively load Sora binary bytes");
}

function verifyBootstrapOutput(debug) {
  const identities = rows(debug, "ContentIdentity");
  assert(rows(debug, "SourceRecord").length === 2 && rows(debug, "EvidenceRecord").length === 3, "bootstrap provenance counts differ");
  assert(identities.length === 285 && rows(debug, "ContentEvidenceBinding").length === 285, "bootstrap identity counts differ");
  assert(identities.every((row) => value(row, "release_state") === "Released" && value(row, "enabled") === false), "bootstrap identities must remain released and disabled");
}

function verifyTemplateList(directory) {
  const templates = fs.readdirSync(directory, { withFileTypes: true }).filter((entry) => entry.isFile()).map((entry) => entry.name).sort();
  assert(templates.length === 82 && templates.every((name) => name.endsWith(".xlsx")), "production template file list differs");
  const data = fs.readdirSync(path.join(root, "config/data"), { withFileTypes: true })
    .filter((entry) => entry.isFile() && !entry.name.startsWith("Universe"))
    .map((entry) => entry.name).sort();
  assert(JSON.stringify(data) === JSON.stringify(templates), "designer workbook layout differs from schema template layout");
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
function sha256(file) { return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex"); }
function readJson(file) { return JSON.parse(fs.readFileSync(file, "utf8")); }
function run(command, args, cwd = root) { const environment = command === "cargo" ? { ...process.env, CARGO_TARGET_DIR: path.join(root, ".cache/workbook-bootstrap-target") } : process.env; const result = spawnSync(command, args, { cwd, stdio: "inherit", env: environment }); if (result.error) throw result.error; assert(result.status === 0, `${command} ${args.join(" ")} exited with ${result.status}`); }
function capture(command, args, cwd = root) { const result = spawnSync(command, args, { cwd, encoding: "utf8" }); if (result.error) throw result.error; assert(result.status === 0, `${command} ${args.join(" ")} exited with ${result.status}: ${result.stderr}`); return result; }
function assert(condition, message) { if (!condition) throw new Error(message); }
