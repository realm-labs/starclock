import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal05-integration-coverage.json");
const sourcePath = "content-manifests/standard-universe-runtime-v1/runtime-dispositions.json";
const source = json(sourcePath);
const matrixPath = "evidence/standard-universe-end-to-end-v1/coverage/seeded-matrix.json";
const matrix = json(matrixPath);
assert(policy.schema_revision === "starclock.goal05-integration-coverage.v1", "unexpected policy revision");
assert(source.records.length === policy.denominators.records, "record denominator drift");
assert(source.rules.length === policy.denominators.rules, "rule denominator drift");
assert(source.fixtures.length === policy.denominators.fixtures, "fixture denominator drift");
assert(matrix.matrix.runs.length === policy.denominators.real_seeded_runs, "matrix denominator drift");
assert(matrix.matrix.battle_assembly.encounter_members === policy.denominators.encounter_members, "encounter denominator drift");
assert(matrix.matrix.battle_assembly.enemy_variants === policy.denominators.enemy_variants, "enemy denominator drift");
assert(matrix.matrix.battle_assembly.exact_enemy_definitions === policy.denominators.exact_enemy_definitions, "exact enemy denominator drift");
assert(matrix.matrix.battle_assembly.approximate_enemy_proxies === policy.denominators.approximate_enemy_proxies, "proxy denominator drift");

const allowed = new Set(policy.allowed_states);
const recordsById = new Map(source.records.map((record) => [record.id, record]));
assert(recordsById.size === source.records.length, "duplicate source record ID");
for (const category of new Set(source.records.map((record) => record.source_category)))
  assert(policy.record_category_states[category], `unassigned source category ${category}`);
for (const id of Object.keys(policy.record_overrides))
  assert(recordsById.has(id), `unknown record override ${id}`);

const records = source.records.map((record) => {
  const state = policy.record_overrides[record.id] ?? policy.record_category_states[record.source_category];
  assert(allowed.has(state), `invalid state for ${record.id}`);
  return {
    id: record.id,
    source_category: record.source_category,
    integration_state: state,
    basis: basis(record.source_category, state, record.id)
  };
});
const recordStates = new Map(records.map((record) => [record.id, record.integration_state]));
const rules = source.rules.map((rule) => {
  const state = recordStates.get(rule.source_record_id);
  assert(state, `rule ${rule.id} lost source record ${rule.source_record_id}`);
  return {
    id: rule.id,
    source_record_id: rule.source_record_id,
    rule_kind: rule.rule_kind,
    integration_state: state,
    basis: state === "Integrated"
      ? "production-lowering-or-authoritative-activity-path"
      : state === "Policy"
        ? "selection-policy-not-effect-runtime"
        : "typed-goal04-evaluator-retained-with-explicit-end-to-end-gap"
  };
});
const fixtures = source.fixtures.map((fixture) => ({
  id: fixture.id,
  mechanic_family: fixture.mechanic_family,
  integration_state: policy.fixture_state,
  basis: "verification-fixture-not-runtime-content"
}));
assert(new Set(records.map((record) => record.id)).size === policy.denominators.records, "record assignment is not exact-once");
assert(new Set(rules.map((rule) => rule.id)).size === policy.denominators.rules, "rule assignment is not exact-once");
assert(new Set(fixtures.map((fixture) => fixture.id)).size === policy.denominators.fixtures, "fixture assignment is not exact-once");

const manifest = {
  schema_revision: "starclock.standard-universe-integration-dispositions.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  source: {
    goal_id: source.goal_id,
    runtime_dispositions_sha256: sha256(sourcePath),
    seeded_matrix_sha256: sha256(matrixPath)
  },
  summary: {
    records: summarize(records),
    rules: summarize(rules),
    fixtures: summarize(fixtures),
    encounter_execution: {
      members_executable: policy.denominators.encounter_members,
      enemy_variants: policy.denominators.enemy_variants,
      exact_definition_matches: policy.denominators.exact_enemy_definitions,
      approximate_definition_proxies: policy.denominators.approximate_enemy_proxies,
      runtime_stat_accuracy: "RetainedApproximation"
    },
    real_matrix: {
      runs: matrix.matrix.runs.length,
      nested_battles: matrix.matrix.coverage.nested_battles,
      battle_commands: matrix.matrix.coverage.battle_commands,
      atomic_external_outcomes: matrix.matrix.coverage.external_outcome_actions
    }
  },
  records,
  rules,
  fixtures,
  contracts: policy.contracts
};
const relative = "content-manifests/standard-universe-end-to-end-v1/integration-dispositions.json";
const output = `${JSON.stringify(manifest, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "integration manifest is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "integration manifest drift; run with --bless");
}
console.log(
  `Goal 05 integration coverage verified (${records.length} records, ${rules.length} rules, ` +
  `${fixtures.length} fixtures, ${manifest.summary.real_matrix.nested_battles} real battles).`
);

function basis(category, state, id) {
  if (policy.record_overrides[id]) return "representative-executable-combat-rule-or-resonance";
  if (state === "Policy") return "deterministic-selection-policy";
  if (state === "Integrated") {
    if (["domains", "maps", "rooms", "worlds"].includes(category)) return "production-activity-graph";
    if (["occurrences", "occurrence-variants"].includes(category)) return "atomic-interaction-routing";
    return "production-runtime-path";
  }
  return "typed-goal04-evaluator-or-data-with-explicit-unlowered-or-approximate-boundary";
}
function summarize(values) {
  const summary = Object.fromEntries(policy.allowed_states.map((state) => [state, 0]));
  for (const value of values) summary[value.integration_state] += 1;
  return summary;
}
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex");
}
function assert(condition, message) { if (!condition) throw new Error(message); }
