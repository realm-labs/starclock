#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildVerificationScaffold } from "./generate-verification-scaffold.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/currency-wars-runtime-v1";

run("node", ["tools/currency-wars-runtime/generate-verification-scaffold.mjs", "--check"]);
const scaffold = buildVerificationScaffold();
const ledger = json(`${runtimeRoot}/batch-ledger.json`);
const foundation = json(`${runtimeRoot}/foundation.json`);
const partitions = json(`${runtimeRoot}/mechanic-partitions.json`).partitions;
const packages = new Set(JSON.parse(exec("cargo", [
  "metadata", "--no-deps", "--format-version", "1",
])).packages.map(({ name }) => name));

assert(scaffold.status === "RuntimeCoverageCompletePendingNativeRelease",
  "P8-B4 scaffold state drift");
assert(scaffold.summary.later_batches === 88
  && scaffold.summary.fixed_batches === 45
  && scaffold.summary.generated_partitions === 43,
"remaining batch denominator drift");
assert(scaffold.summary.assigned_source_catalog_obligations
  === foundation.denominators.source_obligations,
"catalog obligation assignment is not exact-once");
assert(scaffold.summary.assigned_source_execution_obligations
  === foundation.denominators.source_obligations,
"execution obligation assignment is not exact-once");
assert(scaffold.summary.assigned_mechanic_programs
  === foundation.denominators.mechanic_programs,
"mechanic assignment is not exact-once");
assert(scaffold.summary.assigned_fixture_families
  === foundation.denominators.semantic_fixture_families,
"fixture assignment is not exact-once");
assert(scaffold.summary.assigned_policies === foundation.denominators.policy_gaps,
  "policy assignment is not exact-once");

const p0b6 = ledger.batches.find(({ batch }) => batch === "G21-P0-B6");
const later = ledger.batches.filter(({ ordinal }) => ordinal > p0b6.ordinal);
assert(JSON.stringify(scaffold.batches.map(({ batch }) => batch))
  === JSON.stringify(later.map(({ batch }) => batch)),
"scaffold does not preserve generated ledger order");
assertUnique(scaffold.batches.map(({ batch }) => batch), "batch");

for (let index = 0; index < scaffold.batches.length; index += 1) {
  const row = scaffold.batches[index];
  const expectedPrerequisite = index === 0 ? "G21-P0-B6"
    : scaffold.batches[index - 1].batch;
  assert(row.owners.length > 0 && row.owners.every(nonEmpty),
    `${row.batch} has no exact owner`);
  assert(row.prerequisites.length === 1
    && row.prerequisites[0] === expectedPrerequisite,
  `${row.batch} prerequisite/order drift`);
  assert(nonEmpty(row.deliverable), `${row.batch} has no deliverable`);
  assert(row.focused_gate.packages.length > 0,
    `${row.batch} has no affected package`);
  for (const packageName of row.focused_gate.packages)
    assert(packages.has(packageName), `${row.batch} names unknown package ${packageName}`);
  assert(row.focused_gate.commands.length >= 6
    && row.focused_gate.commands.includes("cargo fmt --all -- --check"),
  `${row.batch} has no focused executable gate`);
  for (const command of row.focused_gate.commands)
    assert(nonEmpty(command) && !/[<>]|TBD|TODO|Axx|Mxx/u.test(command),
      `${row.batch} contains a placeholder command: ${command}`);
  for (const packageName of row.focused_gate.packages) {
    assert(row.focused_gate.commands.includes(
      `cargo clippy -p ${packageName} --all-targets -- -D warnings`),
    `${row.batch} omits ${packageName} Clippy`);
    assert(row.focused_gate.commands.includes(`cargo test -p ${packageName}`),
      `${row.batch} omits ${packageName} tests`);
  }
  assert(row.focused_gate.required_assertions.length === 4,
    `${row.batch} focused assertions drift`);
  assert(row.terminal_evidence.required_artifacts.length >= 3
    && nonEmpty(row.terminal_evidence.completion_assertion),
  `${row.batch} has no terminal evidence target`);
  assert(row.terminal_evidence.forbidden_states.includes("Pending")
    && row.terminal_evidence.forbidden_states.includes("IdentityOnly")
    && row.terminal_evidence.forbidden_states.includes("NoOpHandler"),
  `${row.batch} permits a non-terminal completion substitute`);
  assert(JSON.stringify(row.assigned_targets)
    === JSON.stringify(row.terminal_evidence.assigned_counts),
  `${row.batch} terminal count binding drift`);
}

assert(JSON.stringify(scaffold.batches.map(({ status }) => status))
  === JSON.stringify(later.map(({ status }) => status)),
"scaffold status does not match the ledger");
const readyIndex = scaffold.batches.findIndex(({ status }) => status === "Ready");
assert(readyIndex >= 0
  && scaffold.batches.filter(({ status }) => status === "Ready").length === 1
  && scaffold.batches.slice(0, readyIndex).every(({ status }) => status === "Complete")
  && scaffold.batches.slice(readyIndex + 1).every(({ status }) => status === "Pending"),
"exactly one ordered next batch must be selected");
assert(scaffold.batches.filter(({ focused_gate: gate }) =>
  gate.commands.includes("cargo test --workspace")).map(({ batch }) => batch).join(",")
  === "G21-P8-B5", "workspace test is not isolated to the final gate");

const generated = scaffold.batches.filter(({ kind }) =>
  kind === "GeneratedMechanicPartition");
assert(generated.length === partitions.length, "generated partition scaffold drift");
for (const row of generated) {
  const partition = partitions.find(({ batch }) => batch === row.batch);
  assert(partition !== undefined
    && row.assigned_targets.mechanic_programs === partition.program_count
    && row.assigned_targets.source_execution_obligations === partition.program_count,
  `${row.batch} program terminal target drift`);
  assert(row.terminal_evidence.required_artifacts.some((value) =>
    value.includes(partition.fixture_family)),
  `${row.batch} omits its production fixture family`);
}

assert(scaffold.release_gates.length === 7
  && scaffold.release_gates.every(({ owner_batch: owner }) =>
    scaffold.batches.some(({ batch }) => batch === owner)),
"release gate ownership drift");
assert(scaffold.release_gates.at(-1).owner_batch === "G21-P8-B5",
  "clean-checkout/native evidence must remain the final release gate");

console.log(
  `Currency Wars verification scaffold verified (${scaffold.batches.length} later batches; `
    + `${generated.length} generated partitions; ${scaffold.release_gates.length} release gates).`,
);

function exec(command, args) {
  return execFileSync(command, args, { cwd: root, encoding: "utf8" });
}

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}

function json(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8"));
}

function nonEmpty(value) {
  return typeof value === "string" && value.length > 0;
}

function assertUnique(values, label) {
  assert(new Set(values).size === values.length, `${label} identity is not unique`);
}

function assert(condition, message) {
  if (!condition)
    throw new Error(message);
}
