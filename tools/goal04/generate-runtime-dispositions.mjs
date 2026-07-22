import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const check = process.argv.includes("--check");
const policy = json("policy/goal04-runtime-dispositions.json");
const referenceRoot = "content-reference/standard-universe-v1";
const coverage = json(`${referenceRoot}/coverage.json`);
const sourceRules = json(`${referenceRoot}/mechanic-rules.json`);
const records = [];
const recordById = new Map();

for (const category of coverage.categories) {
  const values = json(`${referenceRoot}/${category.file}`);
  assert(values.length === category.required, `${category.category} count differs`);
  for (const value of values) {
    assert(!recordById.has(value.id), `duplicate content ID ${value.id}`);
    const source = { category: category.category, file: category.file, value };
    recordById.set(value.id, source);
  }
}

for (const source of [...recordById.values()].sort((a, b) => ascii(a.value.id, b.value.id))) {
  const partition = contentPartition(source);
  const disposition = policy.category_dispositions[source.category];
  assert(disposition, `missing disposition for ${source.category}`);
  records.push({
    id: source.value.id,
    source_file: source.file,
    source_category: source.category,
    partition,
    disposition,
    target: dispositionTarget(source, disposition),
    linked_rule_ids: sorted(source.value.rule_ids ?? []),
    implementation_state: policy.implementation_state
  });
}

const dispositionByContent = new Map(records.map((record) => [record.id, record]));
const rules = sourceRules.map((rule) => {
  const content = dispositionByContent.get(rule.source_record_id);
  assert(content, `rule ${rule.id} has missing source ${rule.source_record_id}`);
  const disposition = rule.native_handler_id ? "StaticNativeHandler" : "GenericActivityIr";
  if (rule.native_handler_id)
    assert(policy.native_handler_ids.includes(rule.native_handler_id), `unknown native handler ${rule.native_handler_id}`);
  return {
    id: rule.id,
    source_record_id: rule.source_record_id,
    partition: content.partition,
    disposition,
    target: rule.native_handler_id || ruleTarget(rule.rule_kind),
    rule_kind: rule.rule_kind,
    implementation_state: policy.implementation_state
  };
}).sort((a, b) => ascii(a.id, b.id));

const sourceFixtures = json(`${referenceRoot}/review-fixtures.json`);
const fixtures = sourceFixtures.map((fixture) => {
  assert(fixture.input_ids.length > 0, `fixture ${fixture.id} has no input`);
  const inputs = fixture.input_ids.map((id) => dispositionByContent.get(id));
  assert(inputs.every(Boolean), `fixture ${fixture.id} has unknown input`);
  const partitions = sorted([...new Set(inputs.map((input) => input.partition))]);
  assert(partitions.length === 1, `fixture ${fixture.id} crosses partitions: ${partitions}`);
  return {
    id: fixture.id,
    mechanic_family: fixture.mechanic_family,
    input_ids: sorted(fixture.input_ids),
    partition: partitions[0],
    harness: fixtureHarness(fixture.mechanic_family),
    implementation_state: policy.implementation_state
  };
}).sort((a, b) => ascii(a.id, b.id));

const fixtureIdsByInput = new Map();
for (const fixture of fixtures) for (const input of fixture.input_ids) {
  const ids = fixtureIdsByInput.get(input) ?? [];
  ids.push(fixture.id);
  fixtureIdsByInput.set(input, ids);
}
for (const record of records) record.linked_fixture_ids = sorted(fixtureIdsByInput.get(record.id) ?? []);

const partitionManifest = {
  schema_revision: "starclock.standard-universe-runtime-partitions.v1",
  goal_id: policy.goal_id,
  snapshot: policy.snapshot,
  partitions: policy.partitions.map(({ batch, family }) => ({
    batch,
    family,
    content_ids: records.filter((record) => record.partition === batch).map((record) => record.id),
    rule_ids: rules.filter((rule) => rule.partition === batch).map((rule) => rule.id),
    fixture_ids: fixtures.filter((fixture) => fixture.partition === batch).map((fixture) => fixture.id)
  }))
};

const dispositions = {
  schema_revision: "starclock.standard-universe-runtime-dispositions.v1",
  goal_id: policy.goal_id,
  snapshot: policy.snapshot,
  source: {
    coverage_sha256: sha256(`${referenceRoot}/coverage.json`),
    rules_sha256: sha256(`${referenceRoot}/mechanic-rules.json`),
    fixtures_sha256: sha256(`${referenceRoot}/review-fixtures.json`)
  },
  records,
  rules,
  fixtures
};

writeOrCheck("content-manifests/standard-universe-runtime-v1/runtime-dispositions.json", dispositions);
writeOrCheck("content-manifests/standard-universe-runtime-v1/partition-manifest.json", partitionManifest);
console.log(`Goal 04 runtime dispositions ${check ? "current" : "generated"}: ${records.length} content, ${rules.length} rules, ${fixtures.length} fixtures, ${partitionManifest.partitions.length} partitions.`);

function contentPartition({ category, value }) {
  if (category === "ability-tree") return "G04-P4-M01";
  if (["paths", "blessings", "resonances", "blessing-levels"].includes(category)) {
    let pathId = value.path_id;
    if (category === "paths") pathId = value.id;
    if (category === "blessing-levels") {
      const blessing = recordById.get(value.blessing_id)?.value;
      assert(blessing, `missing blessing ${value.blessing_id}`);
      pathId = blessing.path_id;
    }
    const partition = policy.path_partitions[pathId];
    assert(partition, `missing path partition for ${pathId}`);
    return partition;
  }
  if (category === "curios") return isExceptionalCurio(value) ? "G04-P4-M12" : "G04-P4-M11";
  if (category === "curio-states") {
    const curio = recordById.get(value.curio_id)?.value;
    assert(curio, `missing curio ${value.curio_id}`);
    return isExceptionalCurio(curio) || policy.curio_exception_state_kinds.includes(value.state_kind)
      ? "G04-P4-M12" : "G04-P4-M11";
  }
  if (["occurrences", "occurrence-variants", "occurrence-choices"].includes(category)) return "G04-P4-M13";
  if (category === "services") return "G04-P4-M14";
  if (["domains", "encounter-groups", "encounter-pools", "maps", "rooms", "world-difficulties", "worlds"].includes(category)) return "G04-P4-M15";
  throw new Error(`unassigned category ${category}`);
}

function isExceptionalCurio(curio) {
  return curio.tags.some((tag) => policy.curio_exception_tags.includes(tag));
}
function dispositionTarget({ category, value }, disposition) {
  if (disposition === "StaticNativeHandler") {
    const handlers = sorted((value.rule_ids ?? []).map((id) => sourceRules.find((rule) => rule.id === id)?.native_handler_id).filter(Boolean));
    assert(handlers.length > 0, `${value.id} lacks native handler`);
    return handlers.join(",");
  }
  if (disposition === "ExplicitPolicy") return `activity.policy.${value.selection_policy}`;
  if (disposition === "DataOnlyMetadata") return `catalog.${category}`;
  const targets = {
    "ability-tree": "activity.program.ability-tree",
    blessings: "activity.inventory.blessing",
    curios: "activity.inventory.curio",
    domains: "activity.graph.domain",
    "occurrence-choices": "activity.program.occurrence-choice",
    paths: "activity.inventory.path",
    services: "activity.program.service"
  };
  return targets[category] ?? "activity.program.shared";
}
function ruleTarget(kind) {
  return ({
    AbilityTreeContribution: "activity.program.ability-tree",
    BlessingDefinition: "activity.inventory.blessing",
    CurioDefinition: "activity.inventory.curio",
    RunService: "activity.program.service"
  })[kind] ?? "activity.program.shared";
}
function fixtureHarness(family) {
  if (family.startsWith("blessing-tag:") || family.startsWith("path:")) return "activity-battle-contribution";
  if (family.startsWith("encounter-")) return "activity-battle-handoff";
  return "activity-semantic-program";
}
function writeOrCheck(relative, value) {
  const output = `${JSON.stringify(value, null, 2)}\n`;
  const target = path.join(root, relative);
  if (check) {
    assert(fs.existsSync(target), `${relative} is missing`);
    assert(fs.readFileSync(target, "utf8").replaceAll("\r\n", "\n") === output, `${relative} is stale`);
  } else {
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, output);
  }
}
function sorted(values) { return [...values].sort(ascii); }
function ascii(a, b) { return a < b ? -1 : a > b ? 1 : 0; }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
