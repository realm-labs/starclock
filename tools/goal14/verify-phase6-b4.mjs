#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const matrixPath = "evidence/gold-and-gears-runtime-v1/foundation/coverage-matrix.json";
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/topology/seeded-run-matrix.json",
);
const matrix = json(matrixPath);
assert(evidence.schema_revision === "starclock.gold-and-gears-seeded-run-matrix.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P6-B4"
  && evidence.result === "Pass",
"Goal 14 P6-B4 evidence drift");

const frozen = evidence.frozen_matrix;
assert(frozen.path === matrixPath
  && frozen.sha256 === sha256File(matrixPath)
  && frozen.sha256 === "9c79eaf28d53d6ac39bc8c358a38c78dd5f0d62d36ae0e53314f9d50d6ec747b"
  && matrix.matrix_revision === "gold-and-gears-seeded-matrix-v1"
  && matrix.runs.length === 25
  && new Set(matrix.runs.map((row) => row.difficulty)).size === 5
  && new Set(matrix.runs.map((row) => row.path_id)).size === 9
  && new Set(matrix.runs.map((row) => row.custom_dice_id)).size === 12
  && Math.min(...matrix.runs.map((row) => row.stats_conundrum)) === 0
  && Math.max(...matrix.runs.map((row) => row.stats_conundrum)) === 6
  && Math.min(...matrix.runs.map((row) => row.auxiliary_conundrum)) === 0
  && Math.max(...matrix.runs.map((row) => row.auxiliary_conundrum)) === 6
  && matrix.runs.filter((row) => row.stats_conundrum === 6
    && row.auxiliary_conundrum === 6).length === 1
  && matrix.runs.every((row) => row.expected_planes === 3
    && row.expected_terminal === "Complete"
    && row.replay_verification === "FreshFactoryRequired"),
"P6-B4 frozen matrix coverage drift");
const probes = matrix.runs.flatMap((row) => row.policy_probes)
  .map((probe) => probe.register_id).sort();
assert(probes.length === 16
  && probes.every((probe, index) => probe === `G14-R${String(index + 1).padStart(2, "0")}`),
"P6-B4 policy probes must cover G14-R01..R16 exactly once");

const execution = evidence.execution;
assert(execution.revision === "gold-and-gears-seeded-run-v1"
  && execution.completed_runs === 25
  && execution.fresh_factory_verified_runs === 25
  && execution.total_nested_battles === 404
  && execution.minimum_battles_per_run === 15
  && execution.maximum_battles_per_run === 18
  && execution.terminal === "Completed"
  && execution.matrix_roster_accuracy
    === "SyntheticBalanceIndependentNotObservedNumericParity"
  && execution.runtime_json_reads === 0,
"P6-B4 execution summary drift");
assert(evidence.runs.length === 25
  && evidence.runs.every((run, index) => run.id === matrix.runs[index].id
    && Number.isInteger(run.battle_count)
    && run.battle_count >= 15 && run.battle_count <= 18
    && /^[0-9a-f]{64}$/u.test(run.final_state_hash)
    && /^[0-9a-f]{64}$/u.test(run.transcript_digest))
  && evidence.runs.reduce((sum, run) => sum + run.battle_count, 0) === 404,
"P6-B4 per-run golden evidence drift");
assert(Object.values(evidence.validation).every(Boolean),
  "P6-B4 validation evidence is incomplete");

const seeded = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/seeded_run.rs",
);
const tests = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/seeded_run_tests.rs",
);
const map = text("crates/starclock-mode-universe/src/gold_gears_entry/map_overlay.rs");
const api = text("crates/starclock-mode-universe/src/gold_gears_entry/api.rs");
const executionSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/battle_execution.rs",
);
const activity = text("crates/starclock-activity/src/transaction.rs");
const lifecycle = text("crates/starclock-combat/src/resolver/lifecycle.rs");
const conundrum = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/conundrum_stats_modifier.rs",
);
for (const literal of [
  "GOLD_AND_GEARS_SEEDED_RUN_REVISION",
  "execute_seeded_run",
  "verify_seeded_run",
  "UniverseBattleRoster",
  "execute_started_battle",
  "ReplayDivergence",
  "transcript_digest",
]) assert(seeded.includes(literal), `missing seeded-run contract ${literal}`);
for (const literal of [
  "frozen_matrix_completes_real_battles_and_verifies_from_a_fresh_factory",
  "GoldAndGearsRuntimeFactory::load_candidate",
  "seeded_matrix_roster",
  "row.transcript_digest",
]) assert(tests.includes(literal), `missing seeded matrix regression ${literal}`);
assert(map.includes("required_route") && map.includes("terminal_domain")
  && api.includes("required_plane_route")
  && executionSource.includes("plane_ends()")
  && activity.includes("is_settled_at(self.current_node)")
  && lifecycle.includes("declared maximum-HP minimum")
  && conundrum.includes("source_stack_effect"),
"P6-B4 integration hardening boundary drift");
for (const source of [seeded, map, executionSource, activity, lifecycle, conundrum])
  for (const forbidden of [
    "serde_json", "std::fs", "read_to_string", "SystemTime", "thread_rng", "f32", "f64",
  ]) assert(!source.includes(forbidden),
    `seeded runtime gained forbidden dependency ${forbidden}`);
assert(seeded.split(/\r?\n/u).length <= 800
  && executionSource.split(/\r?\n/u).length <= 800,
"P6-B4 responsibility files should be split before 800 lines");

for (const protectedRoot of [
  "evidence/gold-and-gears-reference-v1",
  "content-manifests/gold-and-gears-v1",
  "content-reference/gold-and-gears-v1",
  "config/gold-and-gears/data",
  "config/gold-and-gears-generated",
]) assert(captureGit([
  "status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot,
]).trim() === "", `protected root has worktree changes: ${protectedRoot}`);

const status = text("docs/goals/14-gold-and-gears-runtime-status.md");
assert(status.includes("| Active phase | Phase 7 — Replay, controllers and external surfaces |")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G14-P7-B1` |")
  && status.includes("| `G14-P6-B4` | `Complete` |"),
"G14-P6-B4 ledger is incomplete");
const testEvidence = evidence.tests;
assert(testEvidence.focused_matrix_tests_passed === 1
  && testEvidence.gold_entry_suite_passed === 130
  && testEvidence.combat_lifecycle_tests_passed === 8
  && testEvidence.activity_suite_passed === 63
  && testEvidence.clippy_passed === true
  && testEvidence.dependency_policy_passed === true
  && testEvidence.goal_verifier_passed === true
  && testEvidence.quick_gate_passed === true
  && Number(testEvidence.quick_gate_seconds) > 0
  && Number.isInteger(testEvidence.quick_selected_harnesses)
  && Number.isInteger(testEvidence.quick_deferred_inputs)
  && testEvidence.full_gate_required === true
  && testEvidence.full_gate_passed === true
  && Number(testEvidence.full_gate_seconds) > 0
  && testEvidence.full_workspace_harnesses === 33,
"P6-B4 test evidence drift");

console.log(
  "Goal 14 P6-B4 verified (25/25 fresh-verified runs; 404 real nested battles; "
  + "5 difficulties, 9 Paths, 12 dice and both 0-6 Conundrum tracks).",
);

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function sha256File(relative) {
  return crypto.createHash("sha256").update(
    fs.readFileSync(path.join(root, relative)),
  ).digest("hex");
}
function captureGit(args) {
  return execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
