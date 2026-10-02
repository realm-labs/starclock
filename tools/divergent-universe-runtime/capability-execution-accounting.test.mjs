import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";
import { summarizeProgramExecution } from "./capability-execution-accounting.mjs";

const dispositionPath = new URL(
  "../../content-manifests/divergent-universe-runtime-v1/mechanic-dispositions.json",
  import.meta.url,
);
function row(disposition, runtimeStatus = disposition === "Pending" ? "Pending" : "Terminal") {
  return { mechanic_id: `probe.${disposition}`, execution_disposition: disposition,
    runtime_status: runtimeStatus };
}

test("current 669 source programs retain 663 pending and six non-runtime terminals", () => {
  const input = JSON.parse(fs.readFileSync(dispositionPath, "utf8"));
  assert.equal(input.programs.length, 669);
  const summary = summarizeProgramExecution(input.programs);
  assert.deepEqual(summary, {
    executable_programs: 0, metadata_only_programs: 3, excluded_programs: 3,
    pending_programs: 663, terminal_programs: 6,
    execution_dispositions: { ExcludedWithProof: 3, MetadataOnly: 3, Pending: 663 },
    runtime_status: { Pending: 663, Terminal: 6 },
  });
  assert.deepEqual(summary.execution_dispositions, input.summary.execution_dispositions);
  assert.deepEqual(summary.runtime_status, input.summary.runtime_status);
});

test("closed current IR, Activity and static-handler dispositions are counted separately", () => {
  const summary = summarizeProgramExecution([
    "ExactRuleIr", "ExactActivityProgram", "PolicyRuleIr", "PolicyActivityProgram",
    "StaticHandler", "MetadataOnly", "ExcludedWithProof", "Pending",
  ].map((disposition) => row(disposition)));
  assert.equal(summary.executable_programs, 5);
  assert.equal(summary.terminal_programs, 7);
  assert.equal(summary.pending_programs, 1);
  assert.equal(summary.metadata_only_programs, 1);
  assert.equal(summary.excluded_programs, 1);
});

test("source-layout and catalog support cannot become generic executable credit", () => {
  for (const disposition of ["ExactExecutable", "ExistingPrimitive", "CatalogOnly", "IdentityOnly", "Blocked", undefined])
    assert.throws(() => summarizeProgramExecution([row(disposition)]), /unsupported current execution disposition/u);
});

test("a terminal claim cannot hide a pending or missing runtime status", () => {
  for (const probe of [row("Pending", "Terminal"), row("MetadataOnly", "Pending"),
    row("PolicyRuleIr", "Pending"), row("ExactRuleIr", "Unknown"),
    { mechanic_id: "probe.missing", execution_disposition: "Pending" }])
    assert.throws(() => summarizeProgramExecution([probe]), /disposition\/status mismatch/u);
});

test("duplicate or missing source identities fail exact-once accounting", () => {
  assert.throws(() => summarizeProgramExecution([row("Pending"), row("Pending")]), /identity is missing or duplicated/u);
  assert.throws(() => summarizeProgramExecution([{ ...row("Pending"), mechanic_id: "" }]), /identity is missing or duplicated/u);
});

test("count order is stable and empty input grants no credit", () => {
  const probes = [row("Pending"), row("MetadataOnly"), row("ExactRuleIr")];
  assert.deepEqual(summarizeProgramExecution(probes), summarizeProgramExecution(probes.toReversed()));
  assert.deepEqual(summarizeProgramExecution([]), {
    executable_programs: 0, metadata_only_programs: 0, excluded_programs: 0,
    pending_programs: 0, terminal_programs: 0, execution_dispositions: {}, runtime_status: {},
  });
});

test("generated capability rows and summary retain the current source dispositions", () => {
  const input = JSON.parse(fs.readFileSync(dispositionPath, "utf8"));
  const inventory = JSON.parse(fs.readFileSync(new URL(
    "../../content-manifests/divergent-universe-runtime-v1/capability-inventory.json",
    import.meta.url,
  ), "utf8"));
  const expected = summarizeProgramExecution(input.programs);
  assert.equal(inventory.programs.length, 669);
  assert.deepEqual(summarizeProgramExecution(inventory.programs), expected);
  for (const [field, value] of Object.entries(expected))
    assert.deepEqual(inventory.summary[field], value, field);
  const byId = new Map(input.programs.map((program) => [program.mechanic_id, program]));
  for (const program of inventory.programs) {
    const source = byId.get(program.mechanic_id);
    assert.ok(source, program.mechanic_id);
    assert.equal(program.execution_disposition, source.execution_disposition);
    assert.equal(program.runtime_status, source.runtime_status);
  }
});
