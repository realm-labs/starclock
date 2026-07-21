import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const bless = process.argv.slice(2).includes("--bless");
assert(process.argv.slice(2).every((argument) => argument === "--bless"), "unsupported argument");

const schemaLock = readJson("config/generated/schema.lock");
const productionGolden = readJson("config/production-golden.json");
const tables = new Map();
for (const tableSchema of schemaLock.schema.tables) {
  const diagnostic = readJson(`config/generated/debug-json/${tableSchema.name}.json`);
  assert(diagnostic.table.name === tableSchema.name, `${tableSchema.name}: diagnostic table-name mismatch`);
  tables.set(tableSchema.name, { schema: tableSchema, rows: diagnostic.table.rows });
}
assert(tables.size === 82, "production schema must contain exactly 82 audited tables");

const structs = new Map(schemaLock.schema.structs.map((entry) => [entry.name, entry]));
const unions = new Map(schemaLock.schema.unions.map((entry) => [entry.name, entry]));
const enums = new Map(schemaLock.schema.enums.map((entry) => [entry.name, new Set(entry.values)]));
const keySets = new Map();
let rowCount = 0;
for (const [name, table] of tables) {
  rowCount += table.rows.length;
  if (table.schema.key) {
    const values = table.rows.map((row) => primitive(row.values[table.schema.key]));
    assert(new Set(values.map(stable)).size === values.length, `${name}: duplicate primary key`);
    keySets.set(`${name}.${table.schema.key}`, new Set(values.map(stable)));
  }
}

let referenceCount = 0;
let uniqueIndexCount = 0;
for (const [name, table] of tables) {
  for (const [rowIndex, row] of table.rows.entries()) {
    for (const field of table.schema.fields) {
      validateType(field.ty, row.values[field.name], `${name}[${rowIndex}].${field.name}`);
    }
  }
  for (const index of table.schema.indexes.filter((entry) => entry.unique)) {
    uniqueIndexCount += 1;
    const seen = new Set();
    for (const [rowIndex, row] of table.rows.entries()) {
      const key = stable(index.fields.map((field) => primitive(row.values[field])));
      assert(!seen.has(key), `${name}.${index.name}: duplicate unique key at row ${rowIndex}`);
      seen.add(key);
    }
  }
}

const identities = decodedRows("ContentIdentity");
const identityById = new Map(identities.map((row) => [row.id, row]));
assert(identityById.size === identities.length, "ContentIdentity IDs are not unique");
const enabled = identities.filter((row) => row.enabled);
assert(enabled.length === productionGolden.enabled_identity_count, "enabled identity count differs from production golden");
assert(enabled.every((row) => ["DataReady", "GoldenVerified"].includes(row.coverage_state)), "an enabled identity is below DataReady");

const kindTables = {
  Ability: "Ability",
  Effect: "Effect",
  ModifierDefinition: "Modifier",
  Program: "Program",
  RuleDefinition: "Rule",
  Selector: "Selector",
  StateSlot: "StateSlot",
};
for (const [tableName, kind] of Object.entries(kindTables)) {
  for (const row of decodedRows(tableName)) {
    assert(identityById.get(row.id)?.content_kind === kind, `${tableName}.${row.id}: identity kind is not ${kind}`);
  }
}

const rules = decodedRows("RuleDefinition");
const rootedRules = new Set([
  ...decodedRows("Ability").map((row) => row.entry_rule_identity_id).filter(Number.isInteger),
  ...decodedRows("LightCone").map((row) => row.passive_rule_identity_id),
]);
assert(rootedRules.size === rules.length, "rule root count differs from RuleDefinition count");
for (const rule of rules) assert(rootedRules.has(rule.id), `RuleDefinition.${rule.id}: unreachable from an ability or Light Cone root`);
const sourceDigests = collectSourceDigests(path.join(root, "content-reference", "v4.4"));
for (const rule of rules) {
  assert(sourceDigests.has(rule.source_digest_sha256), `RuleDefinition.${rule.id}: source digest is absent from the prepared reference pack`);
  assert(identityById.has(rule.source_definition_identity_id), `RuleDefinition.${rule.id}: source identity is absent`);
}

const nativeRows = decodedRows("NativeHandler");
assert(nativeRows.length === 0, "production NativeHandler table is not empty");
assert(rules.every((row) => row.native_handler_id === null), "a production rule binds a native handler");
const nativeOperations = decodedRows("Operation").filter((row) => row.payload?.type === "InvokeNativeHandler");
assert(nativeOperations.length === 0, "a production operation invokes a native handler");

const triggers = decodedRows("RuleTrigger");
const onceScopes = enums.get("OnceScope");
assert(onceScopes, "OnceScope enum is missing");
const onceCounts = Object.fromEntries([...onceScopes].sort().map((scope) => [scope, 0]));
const onceKeys = new Set();
for (const trigger of triggers) {
  assert(onceScopes.has(trigger.once_scope), `RuleTrigger.${trigger.id}: invalid once scope`);
  onceCounts[trigger.once_scope] += 1;
  const key = `${trigger.rule_id}\0${trigger.stable_key}`;
  assert(!onceKeys.has(key), `RuleTrigger.${trigger.id}: duplicate rule-local once key`);
  onceKeys.add(key);
  if (trigger.once_scope !== "None") assert(trigger.event && trigger.phase, `RuleTrigger.${trigger.id}: scoped trigger has no event boundary`);
}

const modifiers = decodedRows("ModifierDefinition");
const groups = decodedRows("ModifierStackingGroup");
const groupById = new Map(groups.map((row) => [row.id, row]));
const groupUse = new Map(groups.map((row) => [row.id, 0]));
const conflictClusters = new Map();
for (const modifier of modifiers) {
  const group = groupById.get(modifier.stacking_group_id);
  assert(group, `ModifierDefinition.${modifier.id}: missing stacking group`);
  assert(["Sum", "ReplaceGroup"].includes(group.aggregation), `ModifierDefinition.${modifier.id}: unresolved stacking policy ${group.aggregation}`);
  groupUse.set(group.id, groupUse.get(group.id) + 1);
  const key = stable([
    modifier.stacking_group_id,
    modifier.owner_selector_id,
    modifier.subject_selector_id,
    modifier.stat,
    modifier.formula_stage,
    modifier.formula_purpose,
    modifier.priority,
  ]);
  const cluster = conflictClusters.get(key) ?? { aggregation: group.aggregation, count: 0 };
  assert(cluster.aggregation === group.aggregation, `ModifierDefinition.${modifier.id}: conflict cluster mixes policies`);
  cluster.count += 1;
  conflictClusters.set(key, cluster);
}
for (const [groupId, count] of groupUse) assert(count > 0, `ModifierStackingGroup.${groupId}: unused conflict policy`);
const reachableModifiers = collectNamedIdentityReferences(/modifier(?:_definition|_identity)?_ids?$/);
for (const modifier of modifiers) {
  const sourceAttached = Number.isInteger(modifier.source_rule_id) || Number.isInteger(modifier.source_effect_id);
  assert(sourceAttached || reachableModifiers.has(modifier.id), `ModifierDefinition.${modifier.id}: unreachable from a source or build patch`);
}

const sources = new Map(decodedRows("SourceRecord").map((row) => [row.id, row]));
const evidence = new Map(decodedRows("EvidenceRecord").map((row) => [row.id, row]));
const bindings = decodedRows("ContentEvidenceBinding");
const bindingCounts = new Map();
for (const binding of bindings) {
  const identity = identityById.get(binding.content_id);
  assert(identity, `ContentEvidenceBinding.${binding.content_id}: missing identity`);
  assert(sources.has(binding.source_record_id), `ContentEvidenceBinding.${binding.content_id}: missing source record`);
  assert(evidence.has(binding.evidence_record_id), `ContentEvidenceBinding.${binding.content_id}: missing evidence record`);
  if (binding.quality === "ReleasedTextBoundApproximation") assert(typeof binding.approximation_note === "string" && binding.approximation_note.length > 0, `ContentEvidenceBinding.${binding.content_id}: approximation lacks a note`);
  bindingCounts.set(binding.content_id, (bindingCounts.get(binding.content_id) ?? 0) + 1);
}
for (const identity of enabled) {
  assert(identity.source_record_ids.length > 0, `ContentIdentity.${identity.id}: enabled row has no source provenance`);
  assert((bindingCounts.get(identity.id) ?? 0) > 0, `ContentIdentity.${identity.id}: enabled row has no evidence binding`);
  assert(bindings.some((binding) => binding.content_id === identity.id && identity.source_record_ids.includes(binding.source_record_id)), `ContentIdentity.${identity.id}: identity sources do not support any evidence binding`);
}
for (const source of sources.values()) {
  const linked = [...evidence.values()].some((entry) => entry.source_record_id === source.id && entry.sha256 === source.evidence_sha256);
  assert(linked, `SourceRecord.${source.id}: no exact digest-bound evidence record`);
}

const conflictSummary = { singletons: 0, sum: 0, replace_group: 0 };
for (const cluster of conflictClusters.values()) {
  if (cluster.count === 1) conflictSummary.singletons += 1;
  else if (cluster.aggregation === "Sum") conflictSummary.sum += 1;
  else conflictSummary.replace_group += 1;
}
const report = {
  schema_revision: "starclock.goal01.content-audit.v1",
  config: {
    data_revision: decodedRows("ConfigManifest")[0].data_revision,
    schema_fingerprint: schemaLock.fingerprint,
    bundle_sha256: productionGolden.files["config.sora"],
    output_digest: productionGolden.output_digest,
  },
  manifest_reference_audit: {
    tables: tables.size,
    rows: rowCount,
    references: referenceCount,
    unique_indexes: uniqueIndexCount,
    identities: identities.length,
    enabled_data_ready_or_golden_verified: enabled.length,
  },
  rule_reachability_audit: {
    definitions: rules.length,
    ability_roots: decodedRows("Ability").filter((row) => Number.isInteger(row.entry_rule_identity_id)).length,
    light_cone_roots: decodedRows("LightCone").length,
    unreachable: 0,
    source_digests_bound_to_reference_pack: rules.length,
  },
  native_handler_audit: { definitions: nativeRows.length, rule_bindings: 0, operation_invocations: 0 },
  once_scope_audit: { triggers: triggers.length, counts: onceCounts, duplicate_rule_local_keys: 0 },
  modifier_conflict_audit: {
    definitions: modifiers.length,
    stacking_groups: groups.length,
    reachable_definitions: modifiers.length,
    conflict_clusters: conflictSummary,
    unresolved_policies: 0,
  },
  source_provenance_audit: {
    source_records: sources.size,
    evidence_records: evidence.size,
    bindings: bindings.length,
    enabled_identities_with_sources_and_evidence: enabled.length,
    missing: 0,
  },
};

const output = `${JSON.stringify(report, null, 2)}\n`;
const relativeOutput = "evidence/core-combat-v1/hardening/content-audit.json";
const outputPath = path.join(root, relativeOutput);
if (bless) {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, output);
} else {
  assert(fs.existsSync(outputPath), `${relativeOutput}: missing; run with --bless`);
  assert(fs.readFileSync(outputPath, "utf8") === output, `${relativeOutput}: audit evidence is stale; run with --bless`);
}
console.log(`Goal 01 content audits verified (${sha(output)}; ${rowCount} rows, ${referenceCount} references, ${rules.length} reachable rules, ${modifiers.length} resolved modifiers).`);

function validateType(type, raw, location) {
  if (typeof type === "string") return;
  if (type.Optional !== undefined) {
    if (raw === "Null" || raw === undefined) return;
    validateType(type.Optional, raw, location);
    return;
  }
  if (type.List !== undefined) {
    assert(raw && Array.isArray(raw.List), `${location}: expected encoded list`);
    for (const [index, value] of raw.List.entries()) validateType(type.List, value, `${location}[${index}]`);
    return;
  }
  if (type.Ref !== undefined) {
    const value = primitive(raw);
    const target = keySets.get(`${type.Ref.table}.${type.Ref.field}`);
    assert(target, `${location}: reference target has no audited key set`);
    assert(target.has(stable(value)), `${location}: dangling reference to ${type.Ref.table}.${type.Ref.field}=${value}`);
    referenceCount += 1;
    return;
  }
  if (type.Struct !== undefined) {
    const schema = structs.get(type.Struct);
    assert(schema && raw?.Object, `${location}: invalid ${type.Struct} value`);
    for (const field of schema.fields) validateType(field.ty, raw.Object[field.name], `${location}.${field.name}`);
    return;
  }
  if (type.Union !== undefined) {
    const schema = unions.get(type.Union);
    assert(schema && raw?.Object, `${location}: invalid ${type.Union} value`);
    const tag = primitive(raw.Object[schema.tag]);
    const variant = schema.variants.find((entry) => entry.name === tag);
    assert(variant, `${location}: unknown ${type.Union} variant ${tag}`);
    for (const field of variant.fields) validateType(field.ty, raw.Object[field.name], `${location}.${field.name}`);
  }
}

function collectNamedIdentityReferences(pattern) {
  const values = new Set();
  for (const [tableName, table] of tables) {
    if (tableName === "ModifierDefinition") continue;
    for (const row of table.rows) collect(row.values, "", values);
  }
  return values;
  function collect(value, fieldName, output) {
    if (pattern.test(fieldName)) {
      const decoded = primitive(value);
      for (const item of Array.isArray(decoded) ? decoded : [decoded]) if (Number.isInteger(item)) output.add(item);
    }
    if (!value || typeof value !== "object") return;
    if (Array.isArray(value)) for (const item of value) collect(item, fieldName, output);
    else for (const [name, item] of Object.entries(value)) collect(item, name, output);
  }
}

function collectSourceDigests(directory) {
  const digests = new Set();
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const target = path.join(directory, entry.name);
    if (entry.isDirectory()) for (const digest of collectSourceDigests(target)) digests.add(digest);
    else if (entry.name.endsWith(".json")) walk(readJsonAbsolute(target));
  }
  return digests;
  function walk(value) {
    if (typeof value === "string" && /^[0-9a-f]{64}$/.test(value)) digests.add(value);
    else if (Array.isArray(value)) value.forEach(walk);
    else if (value && typeof value === "object") Object.values(value).forEach(walk);
  }
}

function decodedRows(name) { return tables.get(name).rows.map((row) => Object.fromEntries(Object.entries(row.values).map(([key, value]) => [key, primitive(value)]))); }
function primitive(value) {
  if (value === "Null") return null;
  if (!value || typeof value !== "object") return value;
  if ("Integer" in value) return value.Integer;
  if ("String" in value) return value.String;
  if ("Bool" in value) return value.Bool;
  if ("List" in value) return value.List.map(primitive);
  if ("Object" in value) return Object.fromEntries(Object.entries(value.Object).map(([key, item]) => [key, primitive(item)]));
  throw new Error(`unsupported diagnostic value: ${JSON.stringify(value)}`);
}
function stable(value) { return JSON.stringify(value); }
function sha(value) { return crypto.createHash("sha256").update(value).digest("hex"); }
function readJson(relative) { return readJsonAbsolute(path.join(root, relative)); }
function readJsonAbsolute(file) { return JSON.parse(fs.readFileSync(file, "utf8")); }
function assert(condition, message) { if (!condition) throw new Error(message); }
