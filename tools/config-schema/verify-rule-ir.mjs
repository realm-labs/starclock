import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { isCanonicalDecimal } from "./canonical-decimal.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const arguments_ = process.argv.slice(2);
assert(arguments_.every((argument) => argument === "--bless"), "usage: verify-rule-ir.mjs [--bless]");
const bless = arguments_.includes("--bless");
const toolPolicy = readJson(path.join(root, "policy/sora-toolchain.json"));
const fixture = path.join(root, "config/schema-fixtures/rule-ir");
const baseFixture = path.join(root, "config/schema-fixtures/character-build");
const work = path.join(root, ".cache/rule-ir-schema-work");
const project = path.join(work, "config/schema-fixtures/rule-ir");
assert(path.relative(root, work).replaceAll("\\", "/") === ".cache/rule-ir-schema-work", "unexpected work path");

fs.rmSync(work, { recursive: true, force: true });
fs.mkdirSync(path.join(work, "config/schema"), { recursive: true });
fs.mkdirSync(project, { recursive: true });
fs.cpSync(path.join(root, "config/schema"), path.join(work, "config/schema"), { recursive: true });
prepareTomlSchemas(path.join(work, "config/schema"));
fs.copyFileSync(path.join(fixture, "project.toml"), path.join(project, "project.toml"));
fs.cpSync(path.join(baseFixture, "data"), path.join(project, "data"), { recursive: true });
composeOverlay();

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
assertMapsEqual(firstBuild, artifactHashes(path.join(project, "generated")), "second Rule IR schema build drifted");
verifyNegativeData();

const actualFiles = artifactFiles(path.join(project, "generated"));
const actualMap = Object.fromEntries(actualFiles.map((relative) => [relative, sha256(path.join(project, "generated", relative))]));
const outputDigest = digestFileMap(actualMap);
if (bless) {
  const manifest = {
    schema_revision: "starclock.rule-ir-schema-golden.v1",
    sora_cli_version: toolPolicy.version,
    base_character_build_digest: readJson(path.join(baseFixture, "expected-manifest.json")).output_digest,
    output_digest: outputDigest,
    files: actualMap
  };
  fs.writeFileSync(path.join(fixture, "expected-manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  console.log(`Blessed Rule IR schema golden (${actualFiles.length} files; ${outputDigest}).`);
} else {
  const manifest = readJson(path.join(fixture, "expected-manifest.json"));
  assert(manifest.schema_revision === "starclock.rule-ir-schema-golden.v1", "unexpected Rule IR golden revision");
  assert(manifest.sora_cli_version === toolPolicy.version, "Rule IR golden uses another Sora version");
  assert(manifest.base_character_build_digest === readJson(path.join(baseFixture, "expected-manifest.json")).output_digest, "Rule IR base fixture digest drifted");
  assert(JSON.stringify(actualMap) === JSON.stringify(manifest.files), "Rule IR golden bytes drifted");
  assert(outputDigest === manifest.output_digest, "Rule IR golden digest drifted");
  console.log(`Rule IR schema golden verified (${actualFiles.length} files; ${outputDigest}).`);
}

function composeOverlay() {
  const overlay = path.join(fixture, "data-overlay");
  for (const entry of fs.readdirSync(overlay, { withFileTypes: true }).sort((left, right) => left.name.localeCompare(right.name))) {
    assert(entry.isFile(), `unexpected Rule IR overlay directory ${entry.name}`);
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
  assert(schema.package === "starclock_rule_ir_schema_fixture", "schema lock package differs");
  const tables = new Map(schema.tables.map((table) => [table.name, table]));
  for (const name of expectedTables()) assert(tables.has(name), `schema lock lacks table ${name}`);
  const unions = new Map(schema.unions.map((union) => [union.name, union]));
  assertVariantSet(unions.get("EventPattern"), ["Battle", "Wave", "Turn", "Action", "Hit", "Damage", "HpChanged", "HealApplied", "ShieldChanged", "ToughnessChanged", "WeaknessBroken", "Effect", "ResourceChanged", "TimelineChanged", "Unit", "PresenceChanged", "EncounterTransition", "RuleStateChanged", "DecisionRequested", "FaultRaised", "InformationalRule"]);
  assertVariantSet(unions.get("ProgramStepNode"), ["Operation", "If", "ForEach"]);
  assertVariantSet(unions.get("ValueExpressionNode"), ["IntegerLiteral", "ScalarLiteral", "RatioLiteral", "ProbabilityLiteral", "StableIdLiteral", "BooleanLiteral", "ReadStateSlot", "AbilityParameter", "ReadResource", "QueryStat", "ReadEventProperty", "SelectorCount", "SelectorSum", "CheckedBinary", "Clamp", "Negate", "Choose", "Convert"]);
  assertVariantSet(unions.get("ConditionExpressionNode"), ["Constant", "Compare", "All", "Any", "Not", "HasTag", "LifePresence", "ResourceBounds", "EffectExists", "HasWeakness", "IsBroken", "SelectorCardinality", "EventPropertyCompare"]);
  assertVariantSet(unions.get("OperationPayload"), expectedOperationVariants());
  assertVariantSet(unions.get("SelectorPredicateNode"), ["FormationRange", "HasMark", "HasWeakness", "HasEffect", "HasTag", "OwnedBy", "StatCompare"]);
  assertVariantSet(unions.get("ModifierFilterNode"), ["AbilityTag", "DamageTag", "Element", "Action", "Life", "Presence", "Source", "Target"]);

  const allFields = [
    ...schema.tables.flatMap((table) => table.fields.map((field) => [`${table.name}.${field.name}`, field])),
    ...schema.unions.flatMap((union) => union.variants.flatMap((variant) => variant.fields.map((field) => [`${union.name}.${variant.name}.${field.name}`, field])))
  ];
  for (const [name, field] of allFields) {
    const encoded = JSON.stringify(field.ty);
    assert(!encoded.includes("F32") && !encoded.includes("F64"), `${name} exposes an authoritative float`);
    if (field.name.endsWith("_decimal")) {
      assert((field.ty === "String" || field.ty?.Optional === "String") && JSON.stringify(field.length) === "[1,32]", `${name} violates decimal transport policy`);
    }
  }
  assertRef(tables, "RuleDefinition", "id", "ContentIdentity");
  assertRef(tables, "StateSlot", "initial_expression_id", "ValueExpression");
  assertRef(tables, "RuleTrigger", "condition_id", "ConditionExpression");
  assertRef(tables, "RuleTrigger", "program_id", "Program");
  assertRef(tables, "ModifierDefinition", "value_expression_id", "ValueExpression");
  assertRef(tables, "EffectModifierBinding", "modifier_id", "ModifierDefinition");
  assertRef(tables, "ProgramStep", "program_id", "Program");
  const iterations = unions.get("ProgramStepNode").variants.find((variant) => variant.name === "ForEach").fields.find((field) => field.name === "maximum_iterations");
  assert(JSON.stringify(iterations.range) === "[1,64]", "structured iteration is not statically bounded");
}

function verifyFixtureOutput(directory) {
  const identities = rows(directory, "ContentIdentity");
  assert(identities.length === 24, "composed fixture identity count differs");
  for (const row of identities) {
    assert(value(row, "enabled") === false, "synthetic identity became enabled");
    assert(value(row, "release_state") === "ProjectFixture", "synthetic identity gained a release state");
    assert(value(row, "coverage_state") === "Disabled", "synthetic identity entered coverage");
  }
  for (const table of expectedTables()) verifyCanonicalDecimals(rows(directory, table), table);
  const model = readModel(directory);
  validateModel(model);
  assert(JSON.stringify(model.steps.get(19).map((step) => step.sequence)) === "[1,2]", "main program ordering differs");
  assert(JSON.stringify(model.steps.get(20).map((step) => step.sequence)) === "[1,2,3]", "branch program ordering differs");
  assert(JSON.stringify(model.steps.get(21).map((step) => step.sequence)) === "[1,2]", "iteration body ordering differs");
  assert(model.reachableOperations.has(1) && model.reachableOperations.has(7), "typed operation or replacement proposal is unreachable");
  assert(!model.reachableOperations.has(6), "disabled native-handler fixture became reachable");
  const handler = rows(directory, "NativeHandler")[0];
  assert(value(handler, "enabled") === false, "synthetic native handler became enabled");
  assert(value(handler, "ir_insufficiency_reason").length > 0 && value(handler, "removal_condition").length > 0, "native-handler audit fields are empty");
  const modifier = rows(directory, "ModifierDefinition")[0];
  assert((value(modifier, "source_rule_id") === undefined) !== (value(modifier, "source_effect_id") === undefined), "modifier must have exactly one source definition");
  const captures = rows(directory, "SnapshotCapture");
  assert(captures.length === 2 && JSON.stringify(captures.map((row) => value(row, "owner_identity_id"))) === "[22,23]", "effect/modifier snapshot captures differ");
  for (const capture of captures) {
    assert(value(capture, "capture_kind") === "Stat" && value(capture, "stat") === "Atk", "typed snapshot capture payload differs");
    assert(value(capture, "resource_kind") === undefined && value(capture, "state_slot_id") === undefined && value(capture, "expression_id") === undefined, "snapshot capture mixes payload kinds");
  }

  const badDomain = cloneModel(model);
  badDomain.programs.set(20, "Activity");
  expectAssertion(() => validateModel(badDomain), "cross-domain nested program was accepted");
  const badCycle = cloneModel(model);
  badCycle.steps.set(20, [{ sequence: 1, step: { type: "If", condition_id: 1, then_program_id: 19 } }]);
  expectAssertion(() => validateModel(badCycle), "cyclic program graph was accepted");
  const badReplacement = cloneModel(model);
  badReplacement.operations.set(7, { domain: "Battle", payload: { type: "Damage" } });
  expectAssertion(() => validateModel(badReplacement), "replacement trigger emitted an ordinary mutation");
  const badExpressionCycle = cloneModel(model);
  badExpressionCycle.expressions.set(5, { type: "CheckedBinary", left_expression_id: 6, right_expression_id: 3 });
  expectAssertion(() => validateModel(badExpressionCycle), "cyclic expression graph was accepted");
}

function readModel(directory) {
  const programs = new Map(rows(directory, "Program").map((row) => [value(row, "id"), value(row, "domain")]));
  const steps = groupRows(rows(directory, "ProgramStep"), "program_id", (row) => ({ sequence: value(row, "sequence"), step: value(row, "step") }));
  const operations = new Map(rows(directory, "Operation").map((row) => [value(row, "id"), { domain: value(row, "domain"), payload: value(row, "payload") }]));
  const rules = new Map(rows(directory, "RuleDefinition").map((row) => [value(row, "id"), value(row, "domain")]));
  const triggers = rows(directory, "RuleTrigger").map((row) => ({ rule: value(row, "rule_id"), phase: value(row, "phase"), program: value(row, "program_id") }));
  const selectors = new Map(rows(directory, "Selector").map((row) => [value(row, "id"), value(row, "domain")]));
  const slots = rows(directory, "StateSlot").map((row) => ({ rule: value(row, "rule_id"), scope: value(row, "owner_scope") }));
  const expressions = new Map(rows(directory, "ValueExpression").map((row) => [value(row, "id"), value(row, "node")]));
  const conditions = new Map(rows(directory, "ConditionExpression").map((row) => [value(row, "id"), value(row, "node")]));
  return { programs, steps, operations, rules, triggers, selectors, slots, expressions, conditions, reachableOperations: new Set() };
}

function validateModel(model) {
  model.reachableOperations = new Set();
  assertAcyclicPrograms(model.programs, model.steps);
  assertAcyclicExpressionGraph(model.expressions, model.conditions);
  for (const trigger of model.triggers) {
    const domain = model.rules.get(trigger.rule);
    assert(domain === model.programs.get(trigger.program), "trigger program crosses rule domain");
    const reachable = collectOperations(trigger.program, domain, model, new Set());
    for (const operation of reachable) model.reachableOperations.add(operation);
    if (trigger.phase === "Replace") {
      for (const operation of reachable) assert(model.operations.get(operation).payload.type === "ProposeReplacement", "replacement trigger contains an ordinary mutation");
    }
  }
  for (const slot of model.slots) {
    const domain = model.rules.get(slot.rule);
    const battleScopes = new Set(["Battle", "Wave", "Turn", "Action", "Hit"]);
    assert((domain === "Battle") === battleScopes.has(slot.scope), "state-slot scope crosses rule domain");
  }
}

function assertAcyclicExpressionGraph(expressions, conditions) {
  const visiting = new Set();
  const visited = new Set();
  function visit(key) {
    assert(!visiting.has(key), "expression/condition graph is cyclic");
    if (visited.has(key)) return;
    const [kind, rawId] = key.split(":");
    const id = Number(rawId);
    const node = kind === "v" ? expressions.get(id) : conditions.get(id);
    assert(node !== undefined, `${kind === "v" ? "expression" : "condition"} ${id} is unresolved`);
    visiting.add(key);
    for (const child of expressionGraphChildren(node)) visit(child);
    visiting.delete(key);
    visited.add(key);
  }
  for (const id of expressions.keys()) visit(`v:${id}`);
  for (const id of conditions.keys()) visit(`c:${id}`);
}

function expressionGraphChildren(node) {
  const result = [];
  for (const [name, child] of Object.entries(node)) {
    if (name.endsWith("_expression_id")) result.push(`v:${child}`);
    else if (name === "condition_id") result.push(`c:${child}`);
    else if (name === "condition_ids") for (const id of child) result.push(`c:${id}`);
  }
  return result;
}

function assertAcyclicPrograms(programs, steps) {
  const visiting = new Set();
  const visited = new Set();
  function visit(id) {
    assert(programs.has(id), `program ${id} is unresolved`);
    assert(!visiting.has(id), "program graph is cyclic");
    if (visited.has(id)) return;
    visiting.add(id);
    for (const { step } of steps.get(id) ?? []) {
      for (const child of childPrograms(step)) {
        assert(programs.get(child) === programs.get(id), "structured program child crosses domain");
        visit(child);
      }
    }
    visiting.delete(id);
    visited.add(id);
  }
  for (const id of programs.keys()) visit(id);
}

function collectOperations(program, domain, model, visiting) {
  assert(!visiting.has(program), "program traversal encountered a cycle");
  visiting.add(program);
  const result = new Set();
  for (const { step } of model.steps.get(program) ?? []) {
    if (step.type === "Operation") {
      const operation = model.operations.get(step.operation_id);
      assert(operation?.domain === domain, "operation crosses program domain");
      result.add(step.operation_id);
    } else {
      for (const child of childPrograms(step)) for (const operation of collectOperations(child, domain, model, visiting)) result.add(operation);
    }
  }
  visiting.delete(program);
  return result;
}

function childPrograms(step) {
  if (step.type === "If") return [step.then_program_id, ...(step.else_program_id === undefined ? [] : [step.else_program_id])];
  if (step.type === "ForEach") return [step.body_program_id];
  return [];
}

function cloneModel(model) {
  return {
    programs: new Map(model.programs),
    steps: new Map([...model.steps].map(([key, rows_]) => [key, rows_.map((row) => ({ sequence: row.sequence, step: { ...row.step } }))])),
    operations: new Map([...model.operations].map(([key, operation]) => [key, { domain: operation.domain, payload: { ...operation.payload } }])),
    rules: new Map(model.rules), triggers: model.triggers.map((trigger) => ({ ...trigger })),
    selectors: new Map(model.selectors), slots: model.slots.map((slot) => ({ ...slot })),
    expressions: new Map([...model.expressions].map(([key, node]) => [key, { ...node }])),
    conditions: new Map([...model.conditions].map(([key, node]) => [key, { ...node }])), reachableOperations: new Set()
  };
}

function verifyNegativeData() {
  expectDataFailure("Operation.toml", (source) => source.replace("amount_expression_id = 8", "amount_expression_id = 999"));
  expectDataFailure("RuleTrigger.toml", (source) => source.replace("sequence = 2\nevent = { type = \"HpChanged\" }", "sequence = 1\nevent = { type = \"HpChanged\" }"));
  expectDataFailure("EffectModifierBinding.toml", (source) => source.replace("modifier_id = 23", "modifier_id = 999"));
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

function expectedOperationVariants() {
  return ["Damage", "TrueDamage", "Heal", "Shield", "ConsumeHp", "RedirectDamage", "ReduceToughness", "CreateToughnessLayer", "RemoveToughnessLayer", "Break", "SuperBreak", "ApplyEffect", "RemoveEffect", "DetonateDot", "RefreshEffect", "TransferEffect", "ModifyEffect", "ModifyResource", "ModifyStateSlot", "AdvanceAction", "DelayAction", "QueueAction", "CancelAction", "GrantExtraTurn", "Summon", "Despawn", "Transform", "ReplaceAbility", "SetField", "ChangePresence", "AddWeakness", "RemoveWeakness", "ResistanceOverride", "RequestDecision", "EmitRuleEvent", "RequestEncounterTransition", "ProposeReplacement", "InvokeNativeHandler"];
}
function expectedTables() {
  return [
    "SourceRecord", "EvidenceRecord", "ContentIdentity", "ContentEvidenceBinding", "ConfigManifest",
    "Ability", "AbilityResourceDelta", "AbilityLevelParameter", "AbilityPhase", "HitPlan", "HitPlanHit", "AbilityHitPlanBinding",
    "Character", "CharacterStat", "CharacterResource", "CharacterAbilityBinding", "TraceNode", "TracePatch", "Eidolon", "EidolonPatch",
    "LightCone", "LightConeStat", "LightConeSuperimposition",
    "NativeHandler", "RuleDefinition", "RuleSourceTag", "StateSlot", "StateSlotReset", "EventFilter", "RuleTrigger",
    "Selector", "SelectorPredicate", "ModifierStackingGroup", "ModifierDefinition", "SnapshotCapture", "ModifierFilter", "ValueExpression", "ConditionExpression",
    "Effect", "EffectTag", "EffectModifierBinding", "EffectRuleBinding", "EffectGrantedAbility", "Program", "ProgramStep", "Operation", "LinkedUnitDefinition", "CountdownDefinition", "OperationNativeArgument"
  ];
}
function assertVariantSet(union, expected) { assert(union?.tag === "type" && JSON.stringify(union.variants.map((variant) => variant.name)) === JSON.stringify(expected), `${union?.name ?? "missing union"} variants differ`); }
function assertRef(tables, tableName, fieldName, target) {
  const field = tables.get(tableName).fields.find((candidate) => candidate.name === fieldName);
  assert(field?.ty?.Ref?.table === target, `${tableName}.${fieldName} is not a typed ${target} reference`);
}
function verifyCanonicalDecimals(tableRows, table) {
  for (const row of tableRows) inspect(row.values, table);
  function inspect(value_, key) {
    if (!value_ || typeof value_ !== "object") return;
    for (const [name, child] of Object.entries(value_)) {
      if (name.endsWith("_decimal") && child?.String !== undefined) assert(isCanonicalDecimal(child.String), `${key}.${name} is not canonical`);
      inspect(child, `${key}.${name}`);
    }
  }
}
function groupRows(tableRows, key, map) {
  const groups = new Map();
  for (const row of tableRows) {
    const id = value(row, key);
    if (!groups.has(id)) groups.set(id, []);
    groups.get(id).push(map(row));
  }
  return groups;
}
function rows(directory, name) { return readJson(path.join(directory, `${name}.json`)).table.rows; }
function value(row, name) { const encoded = row.values[name]; return encoded === undefined ? undefined : decode(encoded); }
function decode(encoded) {
  if ("Integer" in encoded) return encoded.Integer;
  if ("String" in encoded) return encoded.String;
  if ("Bool" in encoded) return encoded.Bool;
  if ("List" in encoded) return encoded.List.map(decode);
  if ("Object" in encoded) return Object.fromEntries(Object.entries(encoded.Object).map(([key, child]) => [key, decode(child)]));
  throw new Error(`unsupported diagnostic value ${JSON.stringify(encoded)}`);
}
function assertTemplateList(directory) {
  const actual = fs.readdirSync(directory, { withFileTypes: true }).filter((entry) => entry.isFile()).map((entry) => entry.name).sort();
  const expected = expectedTables().map((name) => `${name}.xlsx`).sort();
  assert(JSON.stringify(actual) === JSON.stringify(expected), `Rule IR Excel template list differs: ${actual.join(", ")}`);
}
function resolveSora(tool) {
  const binary = path.join(root, tool.install_root, "bin", process.platform === "win32" ? "sora.exe" : "sora");
  assert(fs.existsSync(binary), `Sora ${tool.version} is not installed; run ${tool.install_command}`);
  return binary;
}
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
