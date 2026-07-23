import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const artifactOnly = process.env.STARCLOCK_ARTIFACT_CHECK_ONLY === "1";
const policy = json("policy/goal04-mechanic-fixture-audit.json");
assert(policy.schema_revision === "starclock.goal04-mechanic-fixture-audit.v1", "unexpected fixture-audit policy revision");
assert(policy.partitions.length === policy.expected.partitions, "fixture partition denominator differs");

run("node", ["tools/universe-reference/verify-fixtures.mjs", "."]);

const reference = json("content-reference/standard-universe-v1/review-fixtures.json");
const dispositions = json("content-manifests/standard-universe-runtime-v1/runtime-dispositions.json");
assert(reference.length === policy.expected.fixtures, "reference fixture denominator differs");
assert(dispositions.fixtures.length === policy.expected.fixtures, "runtime fixture denominator differs");
assert(reference.reduce((total, fixture) => total + fixture.expected_facts.length, 0) === policy.expected.semantic_facts, "semantic-fact denominator differs");
const referenceById = new Map(reference.map((fixture) => [fixture.id, fixture]));
const seen = new Set();
const testTargets = [];
const sourceSha256 = {};
for (const partition of policy.partitions) {
  const owned = dispositions.fixtures.filter((fixture) => fixture.partition === partition.batch);
  assert(owned.length === partition.fixtures, `${partition.batch}: fixture denominator differs`);
  for (const fixture of owned) {
    assert(fixture.implementation_state === "Executable", `${fixture.id}: fixture is not executable`);
    assert(referenceById.has(fixture.id), `${fixture.id}: reference fixture is missing`);
    assert(!seen.has(fixture.id), `${fixture.id}: fixture is assigned more than once`);
    seen.add(fixture.id);
    const source = referenceById.get(fixture.id);
    assert(source.mechanic_family === fixture.mechanic_family, `${fixture.id}: mechanic family differs`);
    assert(canonical(source.input_ids) === canonical(fixture.input_ids), `${fixture.id}: input binding differs`);
  }
  const relative = `crates/starclock-mode-universe/tests/${partition.test_target}.rs`;
  const testSource = text(relative);
  for (const marker of partition.markers)
    assert(testSource.includes(`fn ${marker}()`), `${partition.batch}: runtime fixture test omits ${marker}`);
  testTargets.push(partition.test_target);
  sourceSha256[relative] = sha256(relative);
}
assert(seen.size === reference.length, "not every fixture is assigned to one tested partition");

const cargoArgs = ["test", "-p", "starclock-mode-universe"];
for (const target of testTargets) cargoArgs.push("--test", target);
if (!artifactOnly) run("cargo", cargoArgs);

const evidence = {
  schema_revision: "starclock.goal04-mechanic-fixture-audit-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "all-sora-reference-semantic-expectations-match-and-all-partition-runtime-fixture-suites-pass",
  counts: {
    fixtures: reference.length,
    semantic_facts: policy.expected.semantic_facts,
    partitions: policy.partitions.length,
    sora_fixture_rows: policy.expected.sora_fixture_rows,
    runtime_test_targets: testTargets.length
  },
  partition_fixtures: Object.fromEntries(policy.partitions.map((partition) => [partition.batch, partition.fixtures])),
  contracts: {
    reference_expected_facts_executed: true,
    sora_rows_byte_semantics_compared: true,
    runtime_dispositions_executable: true,
    runtime_partition_tests_executed: true,
    fixture_assignment_exact_once: true
  },
  digests: {
    reference_fixtures_sha256: sha256("content-reference/standard-universe-v1/review-fixtures.json"),
    runtime_dispositions_sha256: sha256("content-manifests/standard-universe-runtime-v1/runtime-dispositions.json"),
    policy_sha256: sha256("policy/goal04-mechanic-fixture-audit.json")
  },
  source_sha256: sourceSha256,
  new_registry_packages: []
};
const relativeEvidence = "evidence/standard-universe-runtime-v1/profile/mechanic-fixture-audit.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relativeEvidence)), { recursive: true });
  fs.writeFileSync(path.join(root, relativeEvidence), output);
} else {
  assert(fs.existsSync(path.join(root, relativeEvidence)), "fixture audit evidence is missing; run with --bless");
  assert(text(relativeEvidence).replaceAll("\r\n", "\n") === output, "fixture audit evidence is stale; run with --bless");
}
console.log(`Goal 04 mechanic fixtures verified (${reference.length} fixtures, ${policy.expected.semantic_facts} facts, ${testTargets.length} runtime suites).`);

function run(command, args) {
  const result = spawnSync(command, args, { cwd: root, stdio: "inherit" });
  if (result.error) throw result.error;
  assert(result.status === 0, `command failed (${result.status}): ${command} ${args.join(" ")}`);
}
function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value && typeof value === "object")
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  return JSON.stringify(value);
}
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
