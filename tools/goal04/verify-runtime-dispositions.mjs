import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-runtime-dispositions.json");
const dispositions = json("content-manifests/standard-universe-runtime-v1/runtime-dispositions.json");
const partitions = json("content-manifests/standard-universe-runtime-v1/partition-manifest.json");
const referenceRoot = "content-reference/standard-universe-v1";

assert(dispositions.schema_revision === "starclock.standard-universe-runtime-dispositions.v1", "disposition revision differs");
assert(partitions.schema_revision === "starclock.standard-universe-runtime-partitions.v1", "partition revision differs");
assert(dispositions.goal_id === policy.goal_id && partitions.goal_id === policy.goal_id, "goal identity differs");
assert(dispositions.records.length === policy.expected.content_records, "content disposition denominator differs");
assert(dispositions.rules.length === policy.expected.rule_bindings, "rule disposition denominator differs");
assert(dispositions.fixtures.length === policy.expected.semantic_fixtures, "fixture disposition denominator differs");
assert(partitions.partitions.length === policy.expected.partitions, "partition denominator differs");

const coverage = json(`${referenceRoot}/coverage.json`);
const sourceContent = new Map();
for (const category of coverage.categories) for (const row of json(`${referenceRoot}/${category.file}`)) {
  assert(!sourceContent.has(row.id), `duplicate source content ${row.id}`);
  sourceContent.set(row.id, { category: category.category, row });
}
const sourceRules = new Map(json(`${referenceRoot}/mechanic-rules.json`).map((rule) => [rule.id, rule]));
const sourceFixtures = new Map(json(`${referenceRoot}/review-fixtures.json`).map((fixture) => [fixture.id, fixture]));
assert(sourceContent.size === dispositions.records.length, "source content set differs");
assert(sourceRules.size === dispositions.rules.length, "source rule set differs");
assert(sourceFixtures.size === dispositions.fixtures.length, "source fixture set differs");

const records = uniqueMap(dispositions.records, "content disposition");
const rules = uniqueMap(dispositions.rules, "rule disposition");
const fixtures = uniqueMap(dispositions.fixtures, "fixture disposition");
assert(equal(sorted(records.keys()), sorted(sourceContent.keys())), "content ID set differs");
assert(equal(sorted(rules.keys()), sorted(sourceRules.keys())), "rule ID set differs");
assert(equal(sorted(fixtures.keys()), sorted(sourceFixtures.keys())), "fixture ID set differs");

for (const record of records.values()) {
  const source = sourceContent.get(record.id);
  assert(record.source_category === source.category, `${record.id} category differs`);
  assert(policy.allowed_dispositions.includes(record.disposition), `${record.id} has unknown disposition`);
  assert(record.implementation_state === implementationState(record.partition), `${record.id} implementation state differs`);
  assert(equal(record.linked_rule_ids, sorted(source.row.rule_ids ?? [])), `${record.id} rule links differ`);
}
for (const rule of rules.values()) {
  const source = sourceRules.get(rule.id);
  assert(records.has(rule.source_record_id), `${rule.id} source is missing`);
  assert(records.get(rule.source_record_id).partition === rule.partition, `${rule.id} partition differs from source`);
  assert(policy.allowed_dispositions.includes(rule.disposition), `${rule.id} has unknown disposition`);
  assert(rule.implementation_state === implementationState(rule.partition), `${rule.id} implementation state differs`);
  assert((source.native_handler_id ? "StaticNativeHandler" : "GenericActivityIr") === rule.disposition, `${rule.id} native/IR disposition differs`);
  if (source.native_handler_id) assert(rule.target === source.native_handler_id, `${rule.id} handler differs`);
}
for (const fixture of fixtures.values()) {
  const source = sourceFixtures.get(fixture.id);
  assert(equal(fixture.input_ids, sorted(source.input_ids)), `${fixture.id} inputs differ`);
  assert(fixture.implementation_state === implementationState(fixture.partition), `${fixture.id} implementation state differs`);
  assert(fixture.input_ids.every((id) => records.get(id)?.partition === fixture.partition), `${fixture.id} crosses partitions`);
}

const allowedBatches = policy.partitions.map((partition) => partition.batch);
assert(equal(partitions.partitions.map((partition) => partition.batch), allowedBatches), "partition order differs");
for (const partition of partitions.partitions) {
  assert(equal(partition.content_ids, sorted([...records.values()].filter((row) => row.partition === partition.batch).map((row) => row.id))), `${partition.batch} content membership differs`);
  assert(equal(partition.rule_ids, sorted([...rules.values()].filter((row) => row.partition === partition.batch).map((row) => row.id))), `${partition.batch} rule membership differs`);
  assert(equal(partition.fixture_ids, sorted([...fixtures.values()].filter((row) => row.partition === partition.batch).map((row) => row.id))), `${partition.batch} fixture membership differs`);
  const expected = policy.expected.partition_membership[partition.batch];
  assert(equal([partition.content_ids.length, partition.rule_ids.length, partition.fixture_ids.length], expected), `${partition.batch} frozen counts differ`);
}

const contentDispositions = counts(dispositions.records, "disposition", policy.allowed_dispositions);
const ruleDispositions = counts(dispositions.rules, "disposition", policy.allowed_dispositions);
assert(equal(contentDispositions, policy.expected.content_dispositions), "content disposition distribution differs");
assert(equal(ruleDispositions, policy.expected.rule_dispositions), "rule disposition distribution differs");

const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P0-B3` \| `(InProgress|Complete)` \|/m.test(status), "G04-P0-B3 is not active or complete");
const document = text("docs/standard-universe-runtime-disposition-register.md");
for (const marker of ["2,201", "786", "78", "G04-P4-M15", "does not mean implemented"])
  assert(document.includes(marker), `disposition document omits ${marker}`);

const executable = {
  content_records: dispositions.records.filter((row) => row.implementation_state === "Executable").length,
  rule_bindings: dispositions.rules.filter((row) => row.implementation_state === "Executable").length,
  semantic_fixtures: dispositions.fixtures.filter((row) => row.implementation_state === "Executable").length
};
const evidence = {
  schema_revision: "starclock.goal04-runtime-disposition-evidence.v1",
  goal_id: policy.goal_id,
  result: executable.content_records === dispositions.records.length ? "implemented" : "partially-implemented",
  totals: {
    content_records: dispositions.records.length,
    rule_bindings: dispositions.rules.length,
    semantic_fixtures: dispositions.fixtures.length,
    partitions: partitions.partitions.length
  },
  disposition_counts: { content: contentDispositions, rules: ruleDispositions },
  partition_counts: Object.fromEntries(partitions.partitions.map((partition) => [partition.batch, {
    content: partition.content_ids.length,
    rules: partition.rule_ids.length,
    fixtures: partition.fixture_ids.length
  }])),
  digests: {
    policy_sha256: sha256("policy/goal04-runtime-dispositions.json"),
    runtime_dispositions_sha256: sha256("content-manifests/standard-universe-runtime-v1/runtime-dispositions.json"),
    partition_manifest_sha256: sha256("content-manifests/standard-universe-runtime-v1/partition-manifest.json"),
    source_coverage_sha256: sha256(`${referenceRoot}/coverage.json`),
    source_rules_sha256: sha256(`${referenceRoot}/mechanic-rules.json`),
    source_fixtures_sha256: sha256(`${referenceRoot}/review-fixtures.json`)
  },
  implementation_states: {
    planned: {
      content_records: dispositions.records.length - executable.content_records,
      rule_bindings: dispositions.rules.length - executable.rule_bindings,
      semantic_fixtures: dispositions.fixtures.length - executable.semantic_fixtures
    },
    executable
  },
  executable_count: executable.content_records + executable.rule_bindings + executable.semantic_fixtures
};
const relative = "evidence/standard-universe-runtime-v1/foundation/runtime-disposition-summary.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "runtime disposition evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "runtime disposition evidence is stale; run with --bless");
}
console.log(`Goal 04 runtime dispositions verified (${records.size} content, ${rules.size} rules, ${fixtures.size} fixtures, ${partitions.partitions.length} exact partitions; ${executable.content_records}/${executable.rule_bindings}/${executable.semantic_fixtures} executable).`);

function uniqueMap(values, label) { const map = new Map(); for (const value of values) { assert(!map.has(value.id), `duplicate ${label} ${value.id}`); map.set(value.id, value); } return map; }
function counts(values, field, keys) { return Object.fromEntries(keys.map((key) => [key, values.filter((value) => value[field] === key).length])); }
function implementationState(partition) { return policy.implementation_states?.[partition] ?? policy.default_implementation_state; }
function sorted(values) { return [...values].sort((a, b) => a < b ? -1 : a > b ? 1 : 0); }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
