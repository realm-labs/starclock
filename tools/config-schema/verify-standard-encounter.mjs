import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { isCanonicalDecimal } from "./canonical-decimal.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const arguments_ = process.argv.slice(2);
assert(arguments_.every((argument) => argument === "--bless"), "usage: verify-standard-encounter.mjs [--bless]");
const bless = arguments_.includes("--bless");
const toolPolicy = readJson(path.join(root, "policy/sora-toolchain.json"));
const fixture = path.join(root, "config/schema-fixtures/standard-encounter");
const baseFixture = path.join(root, "config/schema-fixtures/character-build");
const ruleFixture = path.join(root, "config/schema-fixtures/rule-ir");
const work = path.join(root, ".cache/standard-encounter-schema-work");
const project = path.join(work, "config/schema-fixtures/standard-encounter");
assert(path.relative(root, work).replaceAll("\\", "/") === ".cache/standard-encounter-schema-work", "unexpected work path");

fs.rmSync(work, { recursive: true, force: true });
fs.mkdirSync(path.join(work, "config/schema"), { recursive: true });
fs.mkdirSync(project, { recursive: true });
fs.cpSync(path.join(root, "config/schema"), path.join(work, "config/schema"), { recursive: true });
prepareTomlSchemas(path.join(work, "config/schema"));
fs.copyFileSync(path.join(fixture, "project.toml"), path.join(project, "project.toml"));
fs.cpSync(path.join(baseFixture, "data"), path.join(project, "data"), { recursive: true });
composeOverlay(ruleFixture);
composeOverlay(fixture);

const sora = resolveSora(toolPolicy);
assert(capture(sora, ["--version"]).stdout.trim() === `sora ${toolPolicy.version}`, "installed Sora version differs from policy");
run(sora, ["--serial", "check", "--project", "./project.toml"]);
run(sora, ["--serial", "build", "--project", "./project.toml", "--clean"]);
formatRust(path.join(project, "generated/rust"));
const schema = verifySchemaLock(path.join(project, "generated/schema.lock"));
verifyFixtureOutput(path.join(project, "generated/debug-json"));
const firstBuild = artifactHashes(path.join(project, "generated"));

const direct = path.join(project, "direct");
run(sora, ["--serial", "schema-lock", "--project", "./project.toml", "--out", "direct/schema.lock"]);
run(sora, ["--serial", "excel-template", "--project", "./project.toml", "--out", "direct/excel"]);
assertTemplateList(path.join(direct, "excel"), schema.tables.map((table) => table.name));
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
assertMapsEqual(firstBuild, artifactHashes(path.join(project, "generated")), "second Standard encounter build drifted");
verifyNegativeData();

const actualFiles = artifactFiles(path.join(project, "generated"));
const actualMap = Object.fromEntries(actualFiles.map((relative) => [relative, sha256(path.join(project, "generated", relative))]));
const outputDigest = digestFileMap(actualMap);
const baseDigest = readJson(path.join(ruleFixture, "expected-manifest.json")).output_digest;
if (bless) {
  fs.writeFileSync(path.join(fixture, "expected-manifest.json"), `${JSON.stringify({
    schema_revision: "starclock.standard-encounter-schema-golden.v1",
    sora_cli_version: toolPolicy.version,
    base_rule_ir_digest: baseDigest,
    output_digest: outputDigest,
    files: actualMap
  }, null, 2)}\n`);
  console.log(`Blessed Standard encounter schema golden (${actualFiles.length} files; ${outputDigest}).`);
} else {
  const manifest = readJson(path.join(fixture, "expected-manifest.json"));
  assert(manifest.schema_revision === "starclock.standard-encounter-schema-golden.v1", "unexpected Standard encounter golden revision");
  assert(manifest.sora_cli_version === toolPolicy.version, "Standard encounter golden uses another Sora version");
  assert(manifest.base_rule_ir_digest === baseDigest, "Rule IR base fixture digest drifted");
  assert(JSON.stringify(actualMap) === JSON.stringify(manifest.files), "Standard encounter golden bytes drifted");
  assert(outputDigest === manifest.output_digest, "Standard encounter golden digest drifted");
  console.log(`Standard encounter schema golden verified (${actualFiles.length} files; ${outputDigest}).`);
}

function composeOverlay(sourceFixture) {
  const overlay = path.join(sourceFixture, "data-overlay");
  for (const entry of fs.readdirSync(overlay, { withFileTypes: true }).sort((left, right) => left.name.localeCompare(right.name))) {
    assert(entry.isFile(), `unexpected overlay directory ${entry.name}`);
    const source = path.join(overlay, entry.name);
    if (entry.name.endsWith(".fragment.toml")) {
      const targetName = entry.name.replace(".fragment.toml", ".toml");
      const target = path.join(project, "data", targetName);
      assert(fs.existsSync(target), `fragment target ${targetName} is absent`);
      fs.appendFileSync(target, fs.readFileSync(source));
    } else {
      fs.copyFileSync(source, path.join(project, "data", entry.name));
    }
  }
}

function verifySchemaLock(file) {
  const schema = readJson(file).schema;
  assert(schema.package === "starclock_standard_encounter_schema_fixture", "schema lock package differs");
  const tables = new Map(schema.tables.map((table) => [table.name, table]));
  for (const name of expectedNewTables()) assert(tables.has(name), `schema lock lacks table ${name}`);
  const projection = schema.unions.find((union) => union.name === "BattleResultProjectionFieldNode");
  assertVariantSet(projection, ["Outcome", "FinalStateHash", "EventDigest", "TerminalFault", "Metric"]);
  for (const table of schema.tables) {
    assert(!/^(Challenge|Universe|Shop|Reward|Account|Season|Clock|Score)/.test(table.name), `out-of-scope table ${table.name}`);
    for (const field of table.fields) verifyField(`${table.name}.${field.name}`, field);
  }
  for (const union of schema.unions) for (const variant of union.variants) for (const field of variant.fields) verifyField(`${union.name}.${variant.name}.${field.name}`, field);
  assertRef(tables, "EnemyVariant", "template_id", "EnemyTemplate");
  assertRef(tables, "AiCandidate", "ability_id", "EnemyAbility");
  assertRef(tables, "WaveSlot", "enemy_variant_id", "EnemyVariant");
  assertRef(tables, "BattleBinding", "encounter_id", "Encounter");
  assertRef(tables, "BattleParticipantSlot", "character_id", "ContentIdentity");
  assertRef(tables, "StandardScenario", "battle_binding_id", "BattleBinding");
  for (const table of ["ActivityDefinition", "ActivitySection", "ActivityNode", "ActivityEdge", "ActivitySlot", "ParticipantPolicy", "BattleResultProjection", "BattleResultProjectionField", "BattleBinding", "BattleBindingRule", "BattleParticipantSlot"])
    assert(tables.has(table), `missing minimum Activity seam ${table}`);
  return schema;
}

function verifyField(name, field) {
  const encoded = JSON.stringify(field.ty);
  assert(!encoded.includes("F32") && !encoded.includes("F64"), `${name} exposes authoritative floating point`);
  if (field.name.endsWith("_decimal")) assert((field.ty === "String" || field.ty?.Optional === "String") && JSON.stringify(field.length) === "[1,32]", `${name} violates decimal transport policy`);
}

function verifyFixtureOutput(directory) {
  const identities = rows(directory, "ContentIdentity");
  assert(identities.length === 40, "composed fixture must contain 40 identities");
  assert(identities.every((row) => !value(row, "enabled") && value(row, "release_state") === "ProjectFixture" && value(row, "coverage_state") === "Disabled"), "fixture identities must remain disabled");
  for (const table of expectedNewTables()) for (const row of rows(directory, table)) verifyCanonicalDecimals(row.values, table);

  const variants = rows(directory, "EnemyVariant").map((row) => value(row, "id"));
  assert(JSON.stringify(variants) === "[26,40]", "fixture enemy variants differ");
  assert(rows(directory, "EnemyToughnessLayer").some((row) => value(row, "kind") === "ExoToughness"), "fixture lacks Exo-Toughness boundary");
  assert(rows(directory, "EnemyLink").some((row) => value(row, "kind") === "Summon"), "fixture lacks linked summon boundary");
  verifyAi(rows(directory, "AiGraph"), rows(directory, "AiState"), rows(directory, "AiCandidate"), rows(directory, "AiTransition"));
  verifyActivity(rows(directory, "ActivityDefinition"), rows(directory, "ActivityNode"), rows(directory, "ActivityEdge"));

  const participants = rows(directory, "BattleParticipantSlot").map((row) => ({
    team: value(row, "team_index"), formation: value(row, "formation_index"), character: value(row, "character_id"),
    spec: value(row, "resolved_spec_sha256"), build: value(row, "build_digest_sha256")
  }));
  verifyParticipants(participants);

  const fields = rows(directory, "BattleResultProjectionField").map((row) => value(row, "field").type);
  verifyProjection(fields);
  const activitySlot = rows(directory, "ActivitySlot").map(decodedRow)[0];
  verifyActivitySlot(activitySlot);
  const profile = rows(directory, "StandardProfile").map((row) => Object.fromEntries(["player_team_count", "maximum_party_size", "has_global_clock", "has_score", "has_seasonal_rules"].map((name) => [name, value(row, name)])))[0];
  verifyStandard(profile);
  assert(rows(directory, "EncounterWave").length === 1 && rows(directory, "WaveSlot").length === 1, "fixture must remain a one-wave handoff");
  assert(rows(directory, "StandardScenario").length === 1, "fixture must contain one Standard scenario");

  expectAssertion(() => verifyStandard({ ...profile, has_score: true }), "score-enabled Standard profile passed");
  const nodes = rows(directory, "ActivityNode").map(decodedRow);
  const edges = rows(directory, "ActivityEdge").map(decodedRow);
  expectAssertion(() => verifyActivity([{ values: { id: { Integer: 31 }, entry_node_id: { Integer: 1 } } }], nodes, edges.filter((edge) => edge.values.target_node_id.Integer !== 4)), "unreachable terminal passed");
  const cycleEdge = { values: { source_node_id: { Integer: 2 }, target_node_id: { Integer: 1 }, maximum_traversals: { Integer: 1 } } };
  expectAssertion(() => verifyActivity([{ values: { id: { Integer: 31 }, entry_node_id: { Integer: 1 } } }], nodes, [...edges, cycleEdge]), "Activity cycle passed");
  const states = rows(directory, "AiState").map(decodedRow);
  expectAssertion(() => verifyAi(rows(directory, "AiGraph"), states.map((row) => value(row, "id") === 1 ? { ...row, values: { ...row.values, mandatory_fallback_ability_id: undefined } } : row), rows(directory, "AiCandidate"), rows(directory, "AiTransition")), "AI state without fallback passed");
  expectAssertion(() => verifyActivitySlot({ ...activitySlot, values: { ...activitySlot.values, owner_scope: { String: "Battle" } } }), "battle-scoped Activity slot passed");
  expectAssertion(() => verifyProjection([...fields, "Metric"]), "undeclared Standard projection field passed");
  expectAssertion(() => verifyParticipants(participants.map((row, index) => index === 3 ? { ...row, character: 38 } : row)), "duplicate participant passed");
  expectAssertion(() => verifyParticipants(participants.map((row, index) => index === 0 ? { ...row, build: "invalid" } : row)), "invalid participant lock passed");
}

function verifyStandard(profile) {
  assert(profile.player_team_count === 1 && profile.maximum_party_size === 4, "Standard party bounds differ");
  assert(!profile.has_global_clock && !profile.has_score && !profile.has_seasonal_rules, "challenge semantics entered Standard profile");
}

function verifyProjection(fields) {
  assert(JSON.stringify(fields) === JSON.stringify(["Outcome", "FinalStateHash", "EventDigest", "TerminalFault"]), "Standard projection is not exactly declared");
}

function verifyParticipants(participants) {
  assert(participants.length === 4, "fixture must prove a full Standard party");
  assert(JSON.stringify(participants.map((row) => row.formation)) === "[0,1,2,3]", "participant formation ordering differs");
  assert(new Set(participants.map((row) => row.character)).size === 4, "participants violate uniqueness");
  assert(participants.every((row) => row.team === 0 && /^[0-9a-f]{64}$/.test(row.spec) && /^[0-9a-f]{64}$/.test(row.build)), "participant lock data differs");
}

function verifyActivitySlot(slot) {
  const scope = value(slot, "owner_scope");
  assert(["Activity", "Section", "Node", "Attempt"].includes(scope), "Activity slot crosses into battle scope");
  if (["Node", "Attempt"].includes(scope)) assert(value(slot, "node_id") !== undefined, `${scope} slot lacks node owner`);
  if (["Section", "Node", "Attempt"].includes(scope)) assert(value(slot, "section_id") !== undefined, `${scope} slot lacks section owner`);
}

function verifyActivity(activities, nodes, edges) {
  const entry = value(activities[0], "entry_node_id");
  const nodeIds = new Set(nodes.map((row) => value(row, "id")));
  const adjacency = new Map([...nodeIds].map((id) => [id, []]));
  for (const edge of edges) adjacency.get(value(edge, "source_node_id"))?.push(value(edge, "target_node_id"));
  const reached = new Set();
  const active = new Set();
  function visit(id) {
    assert(nodeIds.has(id), `Activity graph references missing node ${id}`);
    assert(!active.has(id), "Activity graph contains a cycle");
    if (reached.has(id)) return;
    reached.add(id); active.add(id);
    for (const target of adjacency.get(id)) visit(target);
    active.delete(id);
  }
  visit(entry);
  assert(reached.size === nodeIds.size, "Activity graph contains an unreachable node");
  const terminals = nodes.filter((row) => value(row, "kind") === "Terminal");
  assert(terminals.length === 3 && terminals.every((row) => value(row, "terminal_outcome") !== undefined), "Activity terminal outcomes differ");
  assert(nodes.filter((row) => value(row, "kind") === "Battle").length === 1, "fixture must contain one battle node");
}

function verifyAi(graphs, states, candidates, transitions) {
  const stateIds = new Set(states.map((row) => value(row, "id")));
  const initial = value(graphs[0], "initial_state_id");
  assert(stateIds.has(initial), "AI initial state is absent");
  assert(states.every((row) => value(row, "mandatory_fallback_ability_id") !== undefined), "AI state lacks mandatory fallback");
  assert(candidates.every((row) => stateIds.has(value(row, "state_id"))), "AI candidate has unreachable state");
  assert(transitions.every((row) => stateIds.has(value(row, "state_id")) && stateIds.has(value(row, "target_state_id"))), "AI transition has missing state");
  const automatic = transitions.filter((row) => value(row, "timing") === "AutomaticBeforeDecision");
  assert(automatic.every((row) => value(row, "state_id") !== value(row, "target_state_id")), "automatic transition self-cycle passed");
}

function verifyNegativeData() {
  expectDataFailure("WaveSlot.toml", (source) => source.replace("enemy_variant_id = 26", "enemy_variant_id = 999"));
  expectDataFailure("AiCandidate.toml", (source) => `${source}\n[[rows]]\nid = 2\nstable_key = "fixture.ai.candidate.duplicate"\nstate_id = 1\nsequence = 1\nability_id = 27\ncondition_id = 3\ntarget_selector_id = 35\npriority = 1\nselection = "FirstLegal"\nno_target_fallback = "Fault"\n`);
  expectDataFailure("BattleBinding.toml", (source) => source.replace("projection_id = 1", "projection_id = 999"));
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
  } finally { fs.writeFileSync(file, original); }
}

function expectedNewTables() {
  return [
    "EnemyTemplate", "EnemyVariant", "EnemyStat", "EnemyWeakness", "EnemyResistance", "EnemyDebuffResistance", "EnemyToughnessLayer", "EnemyAbility", "EnemyVariantAbility", "EnemyPhase", "EnemyLink",
    "AiGraph", "AiState", "AiCandidate", "AiTransition", "Encounter", "EncounterRuleBinding", "EncounterWave", "WaveSlot",
    "ActivityDefinition", "ActivitySection", "ActivityNode", "ActivityEdge", "ActivitySlot", "ActivitySlotReset", "ParticipantPolicy", "BattleResultProjection", "BattleResultProjectionField", "BattleBinding", "BattleBindingRule", "BattleParticipantSlot",
    "StandardProfile", "StandardScenario"
  ];
}
function assertVariantSet(union, expected) { assert(union?.tag === "type" && JSON.stringify(union.variants.map((variant) => variant.name)) === JSON.stringify(expected), `${union?.name ?? "missing union"} variants differ`); }
function assertRef(tables, tableName, fieldName, target) { const field = tables.get(tableName).fields.find((candidate) => candidate.name === fieldName); assert(field?.ty?.Ref?.table === target, `${tableName}.${fieldName} is not a typed ${target} reference`); }
function verifyCanonicalDecimals(values, table) { inspect(values, table); function inspect(value_, key) { if (!value_ || typeof value_ !== "object") return; for (const [name, child] of Object.entries(value_)) { if (name.endsWith("_decimal") && child?.String !== undefined) assert(isCanonicalDecimal(child.String), `${key}.${name} is not canonical`); inspect(child, `${key}.${name}`); } } }
function decodedRow(row) { return { ...row, values: { ...row.values } }; }
function rows(directory, name) { return readJson(path.join(directory, `${name}.json`)).table.rows; }
function value(row, name) { const encoded = row.values[name]; return encoded === undefined ? undefined : decode(encoded); }
function decode(encoded) { if ("Integer" in encoded) return encoded.Integer; if ("String" in encoded) return encoded.String; if ("Bool" in encoded) return encoded.Bool; if ("List" in encoded) return encoded.List.map(decode); if ("Object" in encoded) return Object.fromEntries(Object.entries(encoded.Object).map(([key, child]) => [key, decode(child)])); throw new Error(`unsupported diagnostic value ${JSON.stringify(encoded)}`); }
function assertTemplateList(directory, tables) { const actual = fs.readdirSync(directory, { withFileTypes: true }).filter((entry) => entry.isFile()).map((entry) => entry.name).sort(); const expected = tables.map((name) => `${name}.xlsx`).sort(); assert(JSON.stringify(actual) === JSON.stringify(expected), "Standard encounter Excel template list differs"); }
function resolveSora(tool) { const binary = path.join(root, tool.install_root, "bin", process.platform === "win32" ? "sora.exe" : "sora"); assert(fs.existsSync(binary), `Sora ${tool.version} is not installed; run ${tool.install_command}`); return binary; }
function run(command, args) { const result = spawnSync(command, args, { cwd: project, stdio: "inherit" }); if (result.error) throw result.error; assert(result.status === 0, `${relativeCommand(command)} ${args.join(" ")} exited with ${result.status}`); }
function capture(command, args) { const result = spawnSync(command, args, { cwd: project, encoding: "utf8" }); if (result.error) throw result.error; assert(result.status === 0, `${relativeCommand(command)} ${args.join(" ")} exited with ${result.status}: ${result.stderr}`); return result; }
function formatRust(directory) { run("rustfmt", ["--edition", "2024", ...walk(directory).filter((file) => file.endsWith(".rs"))]); }
function artifactFiles(directory) { return walk(directory).map((file) => path.relative(directory, file).replaceAll("\\", "/")).sort(); }
function artifactHashes(directory) { return new Map(artifactFiles(directory).map((relative) => [relative, sha256(path.join(directory, relative))])); }
function walk(directory) { assert(fs.existsSync(directory), `missing directory ${directory}`); return fs.readdirSync(directory, { withFileTypes: true }).sort((left, right) => left.name.localeCompare(right.name)).flatMap((entry) => { const target = path.join(directory, entry.name); return entry.isDirectory() ? walk(target) : [target]; }); }
function assertSameFile(left, right, message) { assert(fs.readFileSync(left).equals(fs.readFileSync(right)), message); }
function assertSameTree(left, right, message) { const leftFiles = artifactFiles(left); const rightFiles = artifactFiles(right); assert(JSON.stringify(leftFiles) === JSON.stringify(rightFiles), `${message}: file lists differ`); for (const relative of leftFiles) assertSameFile(path.join(left, relative), path.join(right, relative), `${message}: ${relative}`); }
function assertMapsEqual(left, right, message) { assert(JSON.stringify([...left]) === JSON.stringify([...right]), message); }
function sha256(file) { return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex"); }
function digestFileMap(files) { return crypto.createHash("sha256").update(Object.entries(files).map(([name, digest]) => `${name}\0${digest}\n`).join(""), "utf8").digest("hex"); }
function readJson(file) { return JSON.parse(fs.readFileSync(file, "utf8")); }
function relativeCommand(command) { return path.relative(root, command).replaceAll("\\", "/") || command; }
function expectAssertion(action, message) { let failed = false; try { action(); } catch { failed = true; } assert(failed, message); }
function assert(condition, message) { if (!condition) throw new Error(message); }

function prepareTomlSchemas(directory) {
  for (const file of walk(directory).filter((candidate) => candidate.endsWith(".toml"))) {
    const source = fs.readFileSync(file, "utf8");
    fs.writeFileSync(file, source.replaceAll('format = "xlsx"', 'format = "toml"').replace(/file = "([A-Za-z0-9_-]+)\.xlsx"/g, 'file = "$1.toml"'));
  }
}
