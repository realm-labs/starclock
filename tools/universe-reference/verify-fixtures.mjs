import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] ?? ".");
const packRoot = path.join(root, "content-reference", "standard-universe-v1");
const readPack = async (file) => JSON.parse(await readFile(path.join(packRoot, file), "utf8"));
const assert = (condition, message) => { if (!condition) throw new Error(message); };

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value && typeof value === "object")
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  return JSON.stringify(value);
}

function project(values, segment) {
  const projected = [];
  for (const value of values) {
    if (segment === "length" && Array.isArray(value)) {
      projected.push(value.length);
      continue;
    }
    if (Array.isArray(value)) {
      projected.push(...project(value, segment));
      continue;
    }
    if (!value || typeof value !== "object") continue;
    if (Object.hasOwn(value, segment)) projected.push(value[segment]);
  }
  return projected;
}

function evaluate(record, fact) {
  let values = [record];
  for (const segment of fact.path.split(".")) values = project(values, segment);
  const leaves = values.flat(Infinity);
  if (fact.operator === "contains")
    return leaves.some((value) => value === fact.value);
  if (fact.operator === "equals") return leaves.length === 1 && leaves[0] === fact.value;
  throw new Error(`unsupported fixture operator ${fact.operator}`);
}

function unwrap(value) {
  if (Object.hasOwn(value, "String")) return value.String;
  if (Object.hasOwn(value, "Integer")) return value.Integer;
  if (Object.hasOwn(value, "Bool")) return value.Bool;
  if (Object.hasOwn(value, "List")) return value.List.map(unwrap);
  throw new Error(`unsupported Sora debug value ${JSON.stringify(value)}`);
}

const coverage = await readPack("coverage.json");
const fixtures = await readPack("review-fixtures.json");
const rules = await readPack("mechanic-rules.json");
const records = new Map();
for (const category of coverage.categories)
  for (const row of await readPack(category.file)) records.set(row.id, row);
for (const row of rules) records.set(row.id, row);

const families = new Set();
let factsExecuted = 0;
for (const fixture of fixtures) {
  assert(!families.has(fixture.mechanic_family), `${fixture.id}: duplicate mechanic family`);
  families.add(fixture.mechanic_family);
  assert(fixture.expected_facts.length > 0, `${fixture.id}: no expected facts`);
  const inputs = fixture.input_ids.map((id) => {
    assert(records.has(id), `${fixture.id}: missing input ${id}`);
    return records.get(id);
  });
  for (const input of inputs)
    assert(input.mechanism_quality === fixture.quality_floor, `${fixture.id}: ${input.id} quality does not meet fixture floor`);
  for (const fact of fixture.expected_facts) {
    assert(["contains", "equals"].includes(fact.operator), `${fixture.id}: unsupported operator`);
    assert(inputs.some((input) => evaluate(input, fact)), `${fixture.id}: expected ${fact.path} ${fact.operator} ${fact.value}`);
    factsExecuted += 1;
  }
  if (fixture.quality_floor === "ProjectPolicy") {
    assert(fixture.note.trim(), `${fixture.id}: ProjectPolicy replacement note missing`);
    for (const input of inputs) {
      assert(input.note.trim(), `${fixture.id}: ${input.id} replacement note missing`);
      const randomOutcomes = (input.outcomes ?? []).filter((outcome) => outcome.unspecified_random_policy);
      assert(randomOutcomes.length > 0, `${fixture.id}: ${input.id} lacks an explicit deterministic replacement policy`);
    }
  }
}

const debug = JSON.parse(
  await readFile(path.join(root, "config", "generated", "debug-json", "UniverseReviewFixture.json"), "utf8"),
);
const debugByKey = new Map(debug.table.rows.map(({ values }) => [unwrap(values.stable_key), values]));
assert(debugByKey.size === fixtures.length, "Sora fixture row count drifted");
for (const fixture of fixtures) {
  const row = debugByKey.get(fixture.id);
  assert(row, `${fixture.id}: missing from Sora debug export`);
  assert(unwrap(row.mechanic_family) === fixture.mechanic_family, `${fixture.id}: mechanic family export drifted`);
  assert(unwrap(row.quality_floor) === fixture.quality_floor, `${fixture.id}: quality export drifted`);
  assert(canonical(unwrap(row.input_stable_keys)) === canonical(fixture.input_ids), `${fixture.id}: input export drifted`);
  assert(unwrap(row.initial_state_json) === canonical(fixture.initial_state), `${fixture.id}: initial-state export drifted`);
  assert(unwrap(row.commands_json) === canonical(fixture.commands), `${fixture.id}: command export drifted`);
  assert(unwrap(row.expected_facts_json) === canonical(fixture.expected_facts), `${fixture.id}: expected-fact export drifted`);
}

console.log(
  `Executed ${factsExecuted} semantic facts across ${fixtures.length} distinct mechanic-family fixtures; ` +
  `${debugByKey.size} Sora fixture rows match the normalized contract.`,
);
