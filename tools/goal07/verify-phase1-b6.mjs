import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const gate = json("policy/goal07-shared-capability-gate.json");
assert(gate.schema_revision === "starclock.goal07-shared-capability-gate.v1"
  && gate.batch === "G07-P1-B6", "shared capability gate identity differs");
for (const [field, expected] of Object.entries({
  capability_batches: 5,
  runtime_probe_markers: 15,
  formal_excel_probe_rows: 6,
  generated_content_partitions: 104,
  assigned_content_records: 2201,
  assigned_mechanic_rules: 786,
  assigned_semantic_fixtures: 78,
  assigned_enemy_variants: 86,
  assigned_encounter_members: 173,
})) assert(gate[field] === expected, `${field} denominator differs`);
assert(Object.values(gate.contracts).every((value) => value === true),
  "shared capability contract is incomplete");

const status = text("docs/goals/07-standard-universe-mechanics-completion-status.md");
const probes = gate.shared_capabilities.flatMap(({ runtime_probes: entries }) => entries);
const formal = gate.shared_capabilities.flatMap(
  ({ formal_excel_probes: entries }) => entries,
);
assert(gate.shared_capabilities.length === gate.capability_batches,
  "shared capability batch count differs");
assert(probes.length === gate.runtime_probe_markers, "runtime probe count differs");
assert(formal.length === gate.formal_excel_probe_rows, "formal probe count differs");
for (const capability of gate.shared_capabilities) {
  assert(status.includes(`| \`${capability.batch}\` | \`Complete\` |`),
    `${capability.batch} is not complete`);
  assert(exists(capability.policy) && exists(capability.verifier),
    `${capability.batch}: policy or verifier is missing`);
  const policy = json(capability.policy);
  assert(policy.batch === capability.batch
    && Object.values(policy.contracts).every((value) => value === true),
  `${capability.batch}: capability policy differs`);
  run("node", [capability.verifier]);
}
for (const probe of probes)
  assert(exists(probe.path) && text(probe.path).includes(probe.marker),
    `runtime probe is absent: ${probe.path}#${probe.marker}`);

const checkedAuthors = new Set();
for (const probe of formal) {
  for (const relative of [probe.workbook, probe.author, probe.generated_table])
    assert(exists(relative), `formal probe input is missing: ${relative}`);
  const rows = json(probe.generated_table).table.rows;
  assert(rows.some(({ values }) =>
    values[probe.id_field]?.Integer === probe.id),
  `formal probe row is missing: ${probe.generated_table} ${probe.id}`);
  if (!checkedAuthors.has(probe.author)) {
    run("uv", ["run", "--with", "openpyxl", "python", probe.author, "--check"]);
    checkedAuthors.add(probe.author);
  }
}

const native = json("policy/native-handler-audit.json");
const admission = native.goal07_admission_contract;
assert(native.registry_revision === gate.native_handler_admission.registry_revision,
  "native registry revision differs");
assert(native.admitted_handlers.length === admission.current_admitted_handlers
  && admission.current_admitted_handlers
    === gate.native_handler_admission.current_admitted_handlers,
"native admitted-handler denominator differs");
assert(admission.maximum_new_handlers_per_partition
  === gate.native_handler_admission.maximum_new_handlers_per_partition,
"native admission cap differs");
assert(JSON.stringify(admission.required_fields)
  === JSON.stringify(gate.native_handler_admission.required_fields),
"native admission field contract differs");
assert(Object.values(admission.contracts).every((value) => value === true),
  "native admission contract is incomplete");
run("node", ["tools/repository-check/verify-native-handlers.mjs"]);

const manifest = json(
  "content-manifests/standard-universe-mechanics-complete-v1/content-partitions.json",
);
assert(manifest.partitions.length === gate.generated_content_partitions,
  "content partition denominator differs");
for (const [field, expected] of Object.entries({
  records: gate.assigned_content_records,
  rules: gate.assigned_mechanic_rules,
  fixtures: gate.assigned_semantic_fixtures,
  enemy_variants: gate.assigned_enemy_variants,
  encounter_members: gate.assigned_encounter_members,
})) assert(manifest.summary.assigned[field] === expected,
  `assigned ${field} denominator differs`);
assert(manifest.partitions[0]?.id === gate.partition_receipt.first_partition,
  "first content partition differs");
run("node", ["tools/goal07/generate-content-progress.mjs", "--check"]);
run("node", [
  "tools/goal07/verify-content-partition.mjs",
  "--partition",
  gate.partition_receipt.first_partition,
  "--expect-pending",
]);
assert(status.includes("| `G07-P1-B6` | `Complete` |"), "G07-P1-B6 is not complete");
assert(status.includes("| Active batch | `G07-P2-M01-S01` |"),
  "first content partition is not active");
console.log(
  "Goal 07 P1-B6 verified " +
  "(5 shared capability families, 15 runtime probes, 6 formal Excel rows, " +
  "0 native handlers, 104 receipt-gated partitions).",
);

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}
function absolute(relative) { return path.join(root, relative); }
function exists(relative) {
  return fs.statSync(absolute(relative), { throwIfNoEntry: false })?.isFile();
}
function text(relative) { return fs.readFileSync(absolute(relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function assert(condition, message) { if (!condition) throw new Error(message); }
