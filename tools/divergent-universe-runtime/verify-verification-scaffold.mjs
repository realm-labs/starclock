#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildVerificationScaffold } from "./generate-verification-scaffold.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const scaffold = buildVerificationScaffold();
run("node", ["tools/divergent-universe-runtime/generate-verification-scaffold.mjs", "--check"]);

assert(scaffold.status === "BehavioralAuditIncomplete",
  "scaffold status drift");
assert(scaffold.summary.later_batches === 59
  && scaffold.summary.fixed_batches === 46
  && scaffold.summary.generated_mechanic_partitions === 13,
"later batch denominator drift");
assert(scaffold.summary.assigned_catalog_obligations === 6215
  && scaffold.summary.assigned_execution_obligations === 6215
  && scaffold.summary.assigned_mechanic_programs === 669
  && scaffold.summary.assigned_semantic_fixtures === 25
  && scaffold.summary.assigned_research_gaps === 25
  && scaffold.summary.assigned_policy_sources === 54,
"assigned target closure drift");
assert(scaffold.release_gates.length === 7, "release gate denominator drift");

const knownBatches = new Set(scaffold.batches.map(({ batch }) => batch));
knownBatches.add("G22-P0-B6");
for (const [index, batch] of scaffold.batches.entries()) {
  assert(batch.prerequisites.length > 0
    && batch.prerequisites.every((value) => knownBatches.has(value)),
  `${batch.batch} prerequisite closure drift`);
  assert(index === 0 ? batch.prerequisites[0] === "G22-P0-B6"
    : batch.prerequisites[0] === scaffold.batches[index - 1].batch,
  `${batch.batch} is not linearly sequenced`);
  assert(batch.owned_files.packages.length > 0
    && batch.owned_files.patterns.length >= 3,
  `${batch.batch} owned file boundary is empty`);
  assert(batch.focused_gate.commands.length >= 8
    && batch.focused_gate.required_assertions.length === 4,
  `${batch.batch} focused gate is incomplete`);
  if (batch.phase >= 2)
    assert(batch.focused_gate.commands.includes(
      "node tools/divergent-universe-runtime/verify-capability-inventory.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-activity-capability-closure.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-combat-capability-closure.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-build-capability-closure.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-shared-capability-audit.mjs"),
    `${batch.batch} does not retain the capability-inventory gate`);
  if (batch.phase >= 3)
    assert(batch.focused_gate.commands.includes(
      "node tools/divergent-universe-runtime/verify-entry-flow-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-progression-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-economy-persistence-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-mapping-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-scope-identity-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-ordinary-vertical-slice-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-equation-offer-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-equation-progress-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-equation-battle-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-blessing-runtime-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-blessing-interaction-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-equation-blessing-hardening-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-curio-runtime-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-grand-miracle-gamble-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-titan-runtime-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-permanent-progression-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-workbench-curse-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-contribution-snapshot-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-occurrence-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-service-adventure-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-encounter-reachability-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-battle-assembly-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-battle-settlement-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-battle-transition-hardening-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-activity-state-mechanic-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-activity-decision-mechanic-a02-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-activity-decision-mechanic-a03-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-activity-decision-mechanic-a04-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-activity-decision-mechanic-a05-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-activity-decision-mechanic-a06-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-activity-decision-mechanic-a07-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-activity-decision-mechanic-a08-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-activity-decision-mechanic-a09-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-activity-decision-mechanic-a10-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-activity-decision-mechanic-a11-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-activity-external-outcome-mechanic-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-battle-mechanic-reachability-proof.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-battle-mechanic-execution.mjs"),
    `${batch.batch} does not retain the entry-flow execution gate`);
  if (batch.phase >= 7)
    assert(batch.focused_gate.commands.includes(
      "node tools/divergent-universe-runtime/verify-baseline-run-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-cli-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-agent-api-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-mcp-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-replay-divergence-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-matrix-execution.mjs"),
    `${batch.batch} does not retain the adapter execution gates`);
  if (batch.phase >= 8)
    assert(batch.focused_gate.commands.includes(
      "node tools/divergent-universe-runtime/verify-hardening-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-performance-execution.mjs")
      && batch.focused_gate.commands.includes(
        "node tools/divergent-universe-runtime/verify-repository-audit-execution.mjs"),
    `${batch.batch} does not retain the phase-8 execution gates`);
  assert(batch.terminal_evidence.required_artifacts.length >= 3
    && batch.terminal_evidence.terminal_states.length === 6
    && batch.terminal_evidence.forbidden_states.includes("NoOpHandler"),
  `${batch.batch} terminal evidence is incomplete`);
  if (batch.batch === "G22-P8-B4" || batch.batch === "G22-P8-B5")
    assert(batch.focused_gate.commands.includes(
      "cargo test -p starclock-mode-universe --lib divergent_universe::tests::release_room_completion_changes_gameplay_beyond_a_lifecycle_marker -- --ignored --exact"),
    `${batch.batch} must reject marker-only room completion`);
}
for (const gate of scaffold.release_gates)
  assert(knownBatches.has(gate.owner_batch) && gate.acceptance.length > 0,
    `release gate ${gate.id} owner drift`);

const ledger = json("content-manifests/divergent-universe-runtime-v1/batch-ledger.json");
  assert(ledger.completed_through === "G22-P2-B5" && ledger.next_batch === "G22-P3-B1",
  "ledger/scaffold progress drift");

console.log(
  `Divergent Universe verification scaffold verified (${scaffold.summary.later_batches} later batches; `
    + "6,215 obligations; 669 programs; prerequisites/gates/files/evidence complete).",
);

function json(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8"));
}

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
