import { readFile } from "node:fs/promises";
import { createHash } from "node:crypto";

const policy = JSON.parse(await readFile("policy/agent-api-v1.json", "utf8"));
const fail = (message) => { throw new Error(`agent-api-v1: ${message}`); };
const load = async (path) => ({ path, text: await readFile(path, "utf8") });
const schemaFiles = await Promise.all(policy.schemas.map(load));
const goldenFiles = await Promise.all(policy.goldens.map(load));
const schemas = schemaFiles.map(({ path, text }) => ({ path, value: JSON.parse(text) }));
const goldens = goldenFiles.map(({ path, text }) => ({ path, text, value: JSON.parse(text) }));
const schemaByName = new Map(schemas.map(({ path, value }) => [path.split("/").at(-1), value]));

if (policy.schema_revision !== "agent-api-v1") fail("revision drift");
const limits = policy.limits;
for (const [name, value] of Object.entries(limits)) {
  if (!Number.isSafeInteger(value) || value <= 0) fail(`invalid limit ${name}`);
}

const walk = (value, visit, path = "$") => {
  visit(value, path);
  if (Array.isArray(value)) value.forEach((item, index) => walk(item, visit, `${path}[${index}]`));
  else if (value && typeof value === "object") for (const [key, item] of Object.entries(value)) walk(item, visit, `${path}.${key}`);
};
for (const schema of schemas) {
  walk(schema.value, (value, path) => {
    if (/\.type(?:\[[0-9]+\])?$/.test(path) && (value === "number" || value === "integer")) fail(`${schema.path} permits authoritative JSON numeric type at ${path}`);
  });
}
for (const golden of goldens) {
  walk(golden.value, (value, path) => {
    if (typeof value === "number") fail(`${golden.path} contains JSON number at ${path}`);
    if (path.endsWith(".schema_revision") && value !== "agent-api-v1") fail(`${golden.path} revision drift`);
  });
}

const resolvePointer = (root, pointer) => pointer.slice(2).split("/").reduce((value, segment) => value[segment.replaceAll("~1", "/").replaceAll("~0", "~")], root);
const validate = (schema, value, root, path = "$") => {
  if (schema.$ref) {
    if (schema.$ref.startsWith("#")) return validate(resolvePointer(root, schema.$ref), value, root, path);
    const external = schemaByName.get(schema.$ref);
    if (!external) fail(`unresolved schema reference ${schema.$ref}`);
    return validate(external, value, external, path);
  }
  if (schema.const !== undefined && value !== schema.const) throw new Error(`${path} differs from const`);
  if (schema.enum && !schema.enum.includes(value)) throw new Error(`${path} is outside enum`);
  if (schema.oneOf) {
    const matches = schema.oneOf.filter((candidate) => {
      try { validate(candidate, value, root, path); return true; } catch { return false; }
    });
    if (matches.length !== 1) throw new Error(`${path} matched ${matches.length} oneOf branches`);
    return;
  }
  const types = schema.type === undefined ? [] : Array.isArray(schema.type) ? schema.type : [schema.type];
  if (types.length) {
    const actual = value === null ? "null" : Array.isArray(value) ? "array" : typeof value;
    if (!types.includes(actual)) throw new Error(`${path} expected ${types.join("|")} but was ${actual}`);
  }
  if (typeof value === "string") {
    if (schema.pattern && !new RegExp(schema.pattern).test(value)) throw new Error(`${path} failed pattern`);
    if (schema.minLength !== undefined && value.length < schema.minLength) throw new Error(`${path} is too short`);
    if (schema.maxLength !== undefined && value.length > schema.maxLength) throw new Error(`${path} is too long`);
  }
  if (Array.isArray(value)) {
    if (schema.minItems !== undefined && value.length < schema.minItems) throw new Error(`${path} has too few items`);
    if (schema.maxItems !== undefined && value.length > schema.maxItems) throw new Error(`${path} has too many items`);
    if (schema.items) value.forEach((item, index) => validate(schema.items, item, root, `${path}[${index}]`));
  } else if (value && typeof value === "object") {
    for (const required of schema.required ?? []) if (!(required in value)) throw new Error(`${path}.${required} is required`);
    if (schema.maxProperties !== undefined && Object.keys(value).length > schema.maxProperties) throw new Error(`${path} has too many properties`);
    for (const [key, item] of Object.entries(value)) {
      if (schema.properties?.[key]) validate(schema.properties[key], item, root, `${path}.${key}`);
      else if (schema.additionalProperties === false) throw new Error(`${path}.${key} is not allowed`);
      else if (schema.additionalProperties && typeof schema.additionalProperties === "object") validate(schema.additionalProperties, item, root, `${path}.${key}`);
    }
  }
};

const ordinary = goldens.find(({ path }) => path.endsWith("ordinary-observation.json")).value;
const heavy = goldens.find(({ path }) => path.endsWith("trigger-heavy-action-response.json")).value;
const stale = goldens.find(({ path }) => path.endsWith("stale-decision-error.json")).value;
for (const [name, value] of [["observation.schema.json", ordinary], ["action.schema.json", heavy], ["error.schema.json", stale]]) {
  const schema = schemaByName.get(name);
  try { validate(schema, value, schema); } catch (error) { fail(`${name} golden validation: ${error.message}`); }
}
const observations = [ordinary, heavy.observation];
const forbidden = new Set(["command", "legal_commands", "enemy_ai_state", "ai_graph", "ai_candidate", "rng_state", "future_draw", "resolver_queue", "internal_store"]);
for (const observation of observations) {
  if (observation.visibility_policy !== "player_visible") fail("golden must exercise default visibility");
  if (observation.legal_actions.length > limits.max_offered_actions) fail("offered-action bound exceeded");
  if (observation.events.length > limits.max_events_per_page) fail("event page bound exceeded");
  if (observation.battle.units.length > limits.max_units || observation.battle.effects.length > limits.max_effects || observation.battle.timeline.length > limits.max_timeline_entries) fail("projection collection bound exceeded");
  const bytes = Buffer.byteLength(JSON.stringify(observation));
  if (bytes > limits.max_observation_bytes) fail(`observation is ${bytes} bytes`);
  walk(observation, (_value, path) => {
    const key = path.split(".").at(-1)?.replace(/\[.*$/, "");
    if (forbidden.has(key)) fail(`hidden field ${key} leaked`);
  });
}
if (!heavy.observation.events_truncated || heavy.observation.events.length <= ordinary.events.length) fail("trigger-heavy golden does not exercise paging pressure");
for (const [field, limit] of [["accepted_commands", limits.max_accepted_commands_per_settlement], ["emitted_events", limits.max_emitted_events_per_settlement], ["resolver_operations", limits.max_resolver_operations_per_settlement]]) {
  if (BigInt(heavy.settlement[field]) > BigInt(limit)) fail(`settlement ${field} exceeds budget`);
}
if (Buffer.byteLength(goldenFiles.find(({ path }) => path.endsWith("trigger-heavy-action-response.json")).text) > limits.max_observation_bytes) fail("trigger-heavy response exceeds observation envelope");
if (stale.code !== "stale_decision" || stale.committed !== false || stale.retryable !== true) fail("stale decision golden semantics drifted");
if (Buffer.byteLength(JSON.stringify(stale)) > limits.max_error_bytes) fail("error envelope exceeds bound");

const digest = createHash("sha256");
for (const { path, text } of [...schemaFiles, ...goldenFiles].sort((a, b) => a.path.localeCompare(b.path))) {
  digest.update(path).update("\0").update(text).update("\0");
}
const actualDigest = digest.digest("hex");
if (actualDigest !== policy.schema_bundle_sha256) fail(`schema bundle digest drifted to ${actualDigest}`);
console.log(`agent-api-v1 verified: ${actualDigest}`);
