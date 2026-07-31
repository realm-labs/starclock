#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/interfaces/universe-cli.json",
);
assert(evidence.schema_revision === "starclock.gold-and-gears-cli-evidence.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P7-B3"
  && evidence.result === "Pass",
"Goal 14 P7-B3 evidence drift");

const contract = evidence.contract;
assert(contract.cli_revision === "starclock-cli-gold-and-gears-v1"
  && contract.mode === "gold-and-gears"
  && contract.profile === "gold-gears.profile.v1"
  && contract.run_command === "starclock universe run --mode gold-and-gears"
  && contract.config_command
    === "starclock universe config validate --mode gold-and-gears"
  && contract.coverage_command
    === "starclock universe coverage --mode gold-and-gears"
  && contract.replay_command === "starclock replay verify FILE"
  && contract.human_and_json_modes === true
  && contract.component_addressed_replay === true
  && contract.component_count === 10
  && contract.replay_envelope === "ReplayV2"
  && contract.fixture_revision === "gold-and-gears-synthetic-baseline-fixture-v1"
  && contract.fixture_accuracy === "SyntheticBalanceIndependentNotObservedNumericParity"
  && contract.real_nested_battles === true,
"P7-B3 CLI contract drift");

const configuration = evidence.configuration;
assert(configuration.bundle_sha256
  === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b"
  && configuration.tables === 52
  && configuration.rows === 29140
  && configuration.source_obligations === 7913
  && configuration.mechanic_rules === 1224
  && configuration.fixtures === 18
  && configuration.policy_boundaries === 16,
"P7-B3 configuration receipt drift");
const coverage = evidence.coverage;
assert(coverage.source_categories === 42
  && coverage.runtime_slices === 44
  && coverage.source_obligations === 7913
  && coverage.integrated === 7181
  && coverage.shared_integrated === 706
  && coverage.external_outcomes === 8
  && coverage.metadata === 18
  && coverage.mechanic_rules === 1224
  && coverage.fixtures === 18
  && coverage.native_handlers === 0
  && coverage.digest
    === "f2d927d197cb77c548522bf39383a68e927f3881412f44dee8a0b4302c38ca9d",
"P7-B3 coverage receipt drift");
const run = evidence.representative_run;
assert(run.seed === 14001
  && run.area === "gold-gears.area.401"
  && run.path === "universe.path.abundance"
  && run.custom_dice === "gold-gears.custom-dice.101"
  && run.controller === "baseline"
  && run.battle_executor === "gold-and-gears-nested-battle-execution-v1"
  && run.component_root
    === "e52ba8dc22197daa70cbdc6e40f9327bc757e12bd17ae11a8fe65c410c780dc3"
  && run.actions === 62
  && run.nested_battles === 17
  && run.battle_commands === 97
  && run.terminal === "Completed"
  && run.state_hash
    === "aa084c9c37e8c3b251fa3e97c6145668997a8160b9db2d7264a5e53c767f8455"
  && run.replay_bytes === 107359
  && run.replay_sha256
    === "71ad733fb0c1a222d70cfd76f755bab65e23f1ca13ea81c3b612e74d0dc277ac"
  && run.fresh_verification_passed === true
  && run.corruption_exit_code === 4,
"P7-B3 representative CLI golden drift");
assert(Object.values(evidence.compatibility).every(Boolean),
  "P7-B3 compatibility evidence is incomplete");

const cli = text("crates/starclock-cli/src/gold_gears_v1.rs");
const main = text("crates/starclock-cli/src/main.rs");
const fixture = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/baseline_fixture.rs",
);
const replay = text("crates/starclock-mode-universe/src/gold_gears_entry/replay.rs");
const tests = text("crates/starclock-cli/tests/universe_cli.rs");
for (const literal of [
  "starclock-cli-gold-and-gears-v1",
  "universe-config-validation",
  "universe-coverage",
  "record_gold_and_gears_run",
  "gold_and_gears_header_v2",
  "encode_gold_and_gears_replay",
  "verify_gold_and_gears_replay",
  "decode_replay_v2",
]) assert(cli.includes(literal), `missing Gold CLI boundary ${literal}`);
for (const literal of [
  "gold_gears_v1::requested",
  "gold_gears_v1::run",
  "gold_gears_v1::coverage",
  "gold_gears_v1::config_validate",
  "gold_gears_v1::is_replay",
  "gold_gears_v1::verify_replay",
]) assert(main.includes(literal), `missing CLI dispatch ${literal}`);
for (const literal of [
  "GOLD_AND_GEARS_BASELINE_FIXTURE_REVISION",
  "GOLD_AND_GEARS_BASELINE_FIXTURE_ACCURACY",
  "compile_synthetic_baseline_fixture",
  "gold_and_gears_component_set",
  "build_catalog_digest",
  "SyntheticBalanceIndependentNotObservedNumericParity",
]) assert(fixture.includes(literal), `missing baseline fixture boundary ${literal}`);
assert(replay.includes("pub fn battle_command_count(&self) -> usize"),
  "recorded Gold replay lacks CLI command-count diagnostic");
for (const literal of [
  "gold_and_gears_configuration_and_coverage_are_machine_readable",
  "gold_and_gears_human_diagnostics_match_the_json_run",
  "gold_and_gears_run_round_trips_component_replay_and_detects_corruption",
  run.component_root,
  run.state_hash,
  run.replay_sha256,
]) assert(tests.includes(literal), `missing Gold CLI regression ${literal}`);
for (const source of [fixture, replay])
  for (const forbidden of [
    "serde_json", "std::fs", "SystemTime", "thread_rng", "f32", "f64",
  ]) assert(!source.includes(forbidden),
    `Gold domain CLI support gained forbidden dependency ${forbidden}`);
assert(cli.split(/\r?\n/u).length <= 500
  && fixture.split(/\r?\n/u).length <= 500,
"P7-B3 responsibility files exceed planned split bounds");

assert(captureGit([
  "status", "--porcelain=v1", "--untracked-files=all", "--",
  "crates/starclock-cli/src/universe_v1.rs",
]).trim() === "", "Standard Universe CLI source changed in P7-B3");
for (const standard of [
  "crates/starclock-mode-universe/src/universe_replay.rs",
  "crates/starclock-mode-universe/src/universe_replay_v2.rs",
  "crates/starclock-mode-universe/src/universe_replay_v3.rs",
]) assert(captureGit([
  "status", "--porcelain=v1", "--untracked-files=all", "--", standard,
]).trim() === "", `Standard replay source changed in P7-B3: ${standard}`);
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
  && status.includes("| Next unblocked batch | `G14-P7-B4` |")
  && status.includes("| `G14-P7-B3` | `Complete` |"),
"G14-P7-B3 ledger is incomplete");
const testEvidence = evidence.tests;
assert(testEvidence.focused_universe_cli_tests_passed === 6
  && testEvidence.all_cli_tests_passed === 16
  && testEvidence.gold_entry_suite_passed === 134
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
"P7-B3 test evidence drift");

console.log(
  "Goal 14 P7-B3 verified (human/JSON config, coverage, 17-battle run and "
  + "107359-byte ReplayV2 round trip).",
);

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
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
