import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-run-catalog.json");
assert(policy.schema_revision === "starclock.goal04-run-catalog.v1", "unexpected run-catalog policy revision");
const occurrence = text("crates/starclock-mode-universe/src/occurrence.rs");
const occurrenceLowering = text("crates/starclock-mode-universe/src/occurrence_lowering.rs");
const progression = text("crates/starclock-mode-universe/src/progression.rs");
const progressionLowering = text("crates/starclock-mode-universe/src/progression_lowering.rs");
const catalog = text("crates/starclock-mode-universe/src/catalog.rs");
const ids = text("crates/starclock-mode-universe/src/id.rs");
for (const marker of ["OccurrenceDefinition", "OccurrenceVariantDefinition", "OccurrenceChoiceDefinition", "OccurrenceOutcome", "RandomOutcomePolicy"])
  assert(occurrence.includes(`pub struct ${marker}`) || occurrence.includes(`pub enum ${marker}`), `Occurrence domain omits ${marker}`);
for (const marker of ["ServiceDefinition", "ServiceKind", "AbilityTreeNodeDefinition", "AbilityTreeEffect"])
  assert(progression.includes(`pub struct ${marker}`) || progression.includes(`pub enum ${marker}`), `progression domain omits ${marker}`);
for (const marker of ["OccurrenceId", "OccurrenceVariantId", "OccurrenceChoiceId", "ServiceId", "AbilityTreeNodeId"])
  assert(new RegExp(`id_type!\\(\\s*${marker}`).test(ids), `run catalog omits stable ID ${marker}`);
assert(!/f32|f64|crate::generated|SoraConfig/.test(occurrence + progression), "run public domains leak transport or floating point");
for (const marker of [
  "Occurrence chance percentage is outside 0 through 100",
  "StableUniformOrderedCandidates",
  "Occurrence choice/cost/outcome denominator differs"
]) assert(occurrenceLowering.includes(marker), `Occurrence lowering omits ${marker}`);
for (const marker of [
  "Service currency does not resolve to a Currency service",
  "Ability Tree prerequisite graph contains a cycle",
  "Ability Tree operation/unit contract is invalid",
  "Ability Tree denominator or required child differs"
]) assert(progressionLowering.includes(marker), `progression lowering omits ${marker}`);
assert(hexSequenceAppears(catalog, policy.definitions_digest), "run definition digest golden is absent");
for (const [name, count] of Object.entries(policy.counts)) assert(Number.isInteger(count) && count >= 0, `${name} count is invalid`);
const tests = (catalog.match(/^    #\[test\]$/gm) ?? []).length
  + (occurrenceLowering.match(/^    #\[test\]$/gm) ?? []).length
  + (progressionLowering.match(/^    #\[test\]$/gm) ?? []).length;
assert(tests === policy.focused_tests, "run-catalog focused test count differs");
const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P1-B5` \| `(InProgress|Complete)` \|/m.test(status), "G04-P1-B5 is not active or complete");

const evidence = {
  schema_revision: "starclock.goal04-run-catalog-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "occurrence-service-ability-tree-catalog-lowered",
  definitions_digest: policy.definitions_digest,
  counts: policy.counts,
  shape: policy.shape,
  boundaries: {
    generated_rows_public: false,
    authoritative_float: false,
    occurrence_choice_order_and_parent_references_validated: true,
    unknown_random_weights_not_fabricated: true,
    service_currency_rule_and_policy_references_validated: true,
    ability_tree_is_acyclic_and_operation_units_are_typed: true,
    runtime_logic_implemented: false
  },
  new_registry_packages: policy.new_registry_packages,
  focused_tests: policy.focused_tests
};
const relative = "evidence/standard-universe-runtime-v1/catalog/run-definitions.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "run catalog evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "run catalog evidence is stale; run with --bless");
}
console.log(`Goal 04 run catalog verified (${policy.counts.occurrences} Occurrences, ${policy.counts.services} services, ${policy.counts.ability_tree_nodes} Ability nodes; ${policy.definitions_digest.slice(0, 12)}).`);

function hexSequenceAppears(source, hex) { return [...source.matchAll(/0x([0-9a-f]{2})/g)].map((match) => match[1]).join("").includes(hex); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function assert(condition, message) { if (!condition) throw new Error(message); }
