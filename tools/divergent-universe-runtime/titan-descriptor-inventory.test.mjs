import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { buildTitanRuntimeExecution } from "./generate-titan-runtime-execution.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const read = (file) => fs.readFileSync(path.join(root, file), "utf8");
const dispositionFile = "content-manifests/divergent-universe-runtime-v1/runtime-dispositions.json";
const ledgerFile = "content-manifests/divergent-universe-runtime-v1/batch-ledger.json";
function changedJson(file, mutate) {
  const data = JSON.parse(read(file));
  mutate(data);
  return { read: (candidate) => candidate === file ? JSON.stringify(data) : read(candidate) };
}

test("current descriptors preserve all pending obligations without pass receipts", () => {
  const artifact = buildTitanRuntimeExecution();
  const assigned = JSON.parse(read(dispositionFile)).obligations
    .filter((row) => row.execution_partition === "G22-P5-B3")
    .map((row) => row.obligation_id).sort();
  assert.deepEqual(artifact.pending_assignments.obligation_ids, assigned);
  assert.equal(assigned.length, 132);
  assert.equal(artifact.summary.terminal_coverage_credit, 0);
  assert.equal(artifact.reference_shape.activity_contributions, 10);
  assert.equal(artifact.reference_shape.battle_contributions, 110);
  assert.equal(artifact.reference_shape.total_talent_cost, "2700");
  assert.equal(artifact.native_test_targets.length, 13);
  assert.equal(artifact.current_boundary.run_entry_fragment_gain, "30");
  assert.equal(artifact.current_boundary.all_activity_effect_consumers_implemented, false);
  assert.equal(artifact.profile_boundary.titan_current_profile_reachability, "Unproven");
  assert.equal(artifact.profile_boundary.current_tourn_mode, "Tourn3");
  assert.equal(artifact.profile_boundary.admitted_current_profile_selector_proofs, 0);
  assert.equal(artifact.profile_boundary.current_profile_gameplay_parity_claimed, false);
  assert.equal(artifact.profile_boundary.non_runtime_exclusion_proven, false);
  for (const forbidden of ["execution_receipt", "capability_probes"])
    assert.equal(Object.hasOwn(artifact, forbidden), false);
  assert.ok(artifact.native_test_targets.every((target) => !Object.hasOwn(target, "result")));
  assert.equal(JSON.stringify(artifact).includes('"Passed"'), false);
});

test("selection descriptors cannot silently promote source obligations", () => {
  const injected = changedJson(dispositionFile, (data) => {
    for (const row of data.obligations)
      if (row.execution_partition === "G22-P5-B3") row.runtime_status = "Terminal";
  });
  assert.throws(() => buildTitanRuntimeExecution(injected), /cannot receive terminal runtime coverage/u);
});

test("fixture names and unlock descriptors cannot close semantic acceptance", () => {
  const injected = changedJson(ledgerFile, (data) => {
    for (const row of data.fixture_assignments)
      if (row.owner_batch === "G22-P5-B3") row.status = "ProductionExecutionPassed";
  });
  assert.throws(() => buildTitanRuntimeExecution(injected), /must remain pending execution/u);
});

test("preserving counts cannot substitute another normalized definition", () => {
  const injected = changedJson(dispositionFile, (data) => {
    data.obligations.find((row) => row.execution_partition === "G22-P5-B3")
      .normalized_record_ids = ["divergent-universe.titan-boon.foreign"];
  });
  assert.throws(() => buildTitanRuntimeExecution(injected), /source\/definition closure drift/u);
});

test("removing the assembly rejection cannot retain a current guard claim", () => {
  assert.throws(() => buildTitanRuntimeExecution({ read: (file) =>
    file.endsWith("battle_assembly_runtime.rs") && !file.includes("/tests/")
      ? read(file).replace("if !contribution.titan().contributions().is_empty()", "if false")
      : read(file) }), /must reject unsupported Titan effect descriptors/u);
});

test("removing the actual entry currency operation cannot retain its consumer inventory", () => {
  assert.throws(() => buildTitanRuntimeExecution({ read: (file) =>
    file.endsWith("/titan_entry.rs") && !file.includes("/tests/")
      ? read(file).replace("credit_operations(self.fragments)", "credit_operations(0)")
      : read(file) }), /missing entry fragment/u);
});

test("current module changes require renewed reachability review", () => {
  for (const mutate of [
    (rows) => { rows[0].tourn_mode = "Tourn2"; },
    (rows) => { rows[0].source_id = "6002101"; },
    (rows) => { rows.push({ ...rows[0] }); },
  ]) {
    assert.throws(() => buildTitanRuntimeExecution(changedJson(
      "content-reference/divergent-universe-v1/modules.json", mutate,
    )), /module selection drift requires a Titan reachability review/u);
  }
});

test("a retained row with a matching selector value is not an admitted proof", () => {
  for (const file of ["titan-types.json", "titan-boons.json", "titan-talents.json"]) {
    const injected = changedJson(`content-reference/divergent-universe-v1/${file}`, (rows) => {
      rows[0].tourn_mode = "Tourn3";
    });
    assert.throws(() => buildTitanRuntimeExecution(injected),
      /new Titan selector evidence requires a reachability review/u);
  }
});
