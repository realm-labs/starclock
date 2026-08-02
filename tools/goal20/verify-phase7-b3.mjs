#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/interfaces/universe-cli.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-cli-evidence.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P7-B3"
  && evidence.result === "Pass",
"Goal 20 P7-B3 evidence drift");

const contract = evidence.contract;
assert(contract.cli_revision === "starclock-cli-swarm-disaster-v1"
  && contract.mode === "swarm-disaster"
  && contract.profile === "swarm-disaster.profile.v1"
  && contract.run_command === "starclock universe run --mode swarm-disaster"
  && contract.config_command
    === "starclock universe config validate --mode swarm-disaster"
  && contract.coverage_command
    === "starclock universe coverage --mode swarm-disaster"
  && contract.replay_command === "starclock replay verify FILE"
  && contract.human_and_json_modes === true
  && contract.component_addressed_replay === true
  && contract.component_count === 10
  && contract.replay_envelope === "ReplayV2"
  && contract.fixture_revision === "swarm-disaster-synthetic-baseline-fixture-v1"
  && contract.fixture_accuracy === "SyntheticBalanceIndependentNotObservedNumericParity"
  && contract.real_nested_battles === true
  && contract.new_public_domain_types === 0
  && contract.new_public_reexports === 0,
"P7-B3 CLI contract drift");

const configuration = evidence.configuration;
assert(configuration.bundle_sha256
  === "385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362"
  && configuration.tables === 65
  && configuration.rows === 33380
  && configuration.source_obligations === 6963
  && configuration.mechanic_rules === 23
  && configuration.fixtures === 23
  && configuration.policy_boundaries === 31,
"P7-B3 configuration receipt drift");
const coverage = evidence.coverage;
assert(coverage.source_categories === 42
  && coverage.runtime_slices === 42
  && coverage.source_obligations === 6963
  && coverage.integrated === 6282
  && coverage.shared_integrated === 652
  && coverage.external_outcomes === 6
  && coverage.metadata === 23
  && coverage.mechanic_rules === 23
  && coverage.fixtures === 23
  && coverage.native_handlers === 0
  && coverage.digest
    === "8aeb60d2c1b322f9dcf8f84bc45dc1901276633398cdb60a984ccc4846f0bff4",
"P7-B3 coverage receipt drift");
const run = evidence.representative_run;
assert(run.seed === 20001
  && run.area === "swarm-disaster.area.201"
  && run.path === "universe.path.preservation"
  && run.audience_die === "swarm-disaster.audience-die.1"
  && run.controller === "baseline"
  && run.battle_executor === "swarm-disaster-nested-battle-execution-v1"
  && run.component_root
    === "a87894170e22188cb00078c339e806a6e3387f5e49baf7fd7782f6f0732c823c"
  && run.actions === 48
  && run.nested_battles === 12
  && run.battle_commands === 68
  && run.terminal === "Completed"
  && run.state_hash
    === "eb870454531b7d109bd43cef38f5d320df85dbbb76ce9732c4eca022a4881075"
  && run.replay_bytes === 81107
  && run.replay_sha256
    === "d052a392d91dd93e9e8baf44b80940fb9a57111384b052332f6c21ad869a73a4"
  && run.fresh_verification_passed === true
  && run.corruption_exit_code === 4,
"P7-B3 representative CLI golden drift");
assert(Object.values(evidence.compatibility).every(Boolean),
  "P7-B3 compatibility evidence is incomplete");

const cli = text("crates/starclock-cli/src/swarm_disaster_v1.rs");
const main = text("crates/starclock-cli/src/main.rs");
const fixture = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/baseline_fixture.rs",
);
const replay = text("crates/starclock-mode-universe/src/swarm_disaster_entry/replay.rs");
const tests = text("crates/starclock-cli/tests/universe_cli.rs");
for (const literal of [
  "starclock-cli-swarm-disaster-v1",
  "universe-config-validation",
  "universe-coverage",
  "encode_complete_swarm_replay_v2",
  "verify_complete_swarm_replay_v2",
  "decode_replay_v2",
]) assert(cli.includes(literal), `missing Swarm CLI boundary ${literal}`);
for (const literal of [
  "swarm_disaster_v1::requested",
  "swarm_disaster_v1::run",
  "swarm_disaster_v1::coverage",
  "swarm_disaster_v1::config_validate",
  "swarm_disaster_v1::is_replay",
  "swarm_disaster_v1::verify_replay",
]) assert(main.includes(literal), `missing CLI dispatch ${literal}`);
for (const literal of [
  "SWARM_DISASTER_BASELINE_FIXTURE_REVISION",
  "SWARM_DISASTER_BASELINE_FIXTURE_ACCURACY",
  "compile_synthetic_baseline_fixture",
  "swarm_disaster_component_set",
  "build_catalog_digest",
  "SyntheticBalanceIndependentNotObservedNumericParity",
]) assert(fixture.includes(literal), `missing baseline fixture boundary ${literal}`);
assert(replay.includes("pub const fn battle_command_count(self) -> u32"),
  "Swarm replay lacks CLI command-count diagnostic");
for (const literal of [
  "swarm_disaster_configuration_and_coverage_are_machine_readable",
  "swarm_disaster_human_diagnostics_match_the_json_run",
  "swarm_disaster_run_round_trips_component_replay_and_detects_corruption",
  run.component_root,
  run.state_hash,
  run.replay_sha256,
]) assert(tests.includes(literal), `missing Swarm CLI regression ${literal}`);
for (const source of [fixture, replay])
  for (const forbidden of [
    "serde_json", "std::fs", "SystemTime", "thread_rng", "f32", "f64",
  ]) assert(forbidden === "f32" || forbidden === "f64"
    ? !new RegExp(`\\b${forbidden}\\b`, "u").test(source)
    : !source.includes(forbidden),
  `Swarm domain CLI support gained forbidden dependency ${forbidden}`);
assert(cli.split(/\r?\n/u).length <= 500
  && fixture.split(/\r?\n/u).length <= 500,
"P7-B3 responsibility files exceed planned split bounds");
assert(!fixture.includes("pub use") && !cli.includes("pub use"),
  "P7-B3 added a public re-export");

for (const protectedSource of [
  "crates/starclock-cli/src/universe_v1.rs",
  "crates/starclock-cli/src/gold_gears_v1.rs",
  "crates/starclock-mode-universe/src/universe_replay.rs",
  "crates/starclock-mode-universe/src/universe_replay_v2.rs",
  "crates/starclock-mode-universe/src/universe_replay_v3.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/replay.rs",
]) assert(captureGit([
  "status", "--porcelain=v1", "--untracked-files=all", "--", protectedSource,
]).trim() === "", `protected CLI/replay source changed in P7-B3: ${protectedSource}`);
for (const protectedRoot of [
  "evidence/swarm-disaster-reference-v1",
  "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1",
  "config/swarm-disaster/data",
  "config/swarm-disaster-generated",
]) assert(captureGit([
  "status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot,
]).trim() === "", `protected root has worktree changes: ${protectedRoot}`);

const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| Active phase | Phase 7 — Replay, controllers and external surfaces |")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P7-B4` |")
  && status.includes("| `G20-P7-B3` | `Complete` |"),
"G20-P7-B3 ledger is incomplete");
const testEvidence = evidence.tests;
assert(testEvidence.focused_universe_cli_tests_passed === 9
  && testEvidence.all_cli_tests_passed === 19
  && testEvidence.swarm_entry_suite_passed === 142
  && testEvidence.clippy_passed === true
  && testEvidence.dependency_policy_passed === true
  && testEvidence.source_policy_passed === true
  && testEvidence.handwritten_rust_files === 958
  && testEvidence.public_reexport_declarations === 72
  && testEvidence.runtime_contract_passed === true
  && testEvidence.goal_verifier_passed === true
  && testEvidence.quick_gate_passed === true
  && Number(testEvidence.quick_gate_seconds) > 0
  && Number.isInteger(testEvidence.quick_selected_harnesses)
  && Number.isInteger(testEvidence.quick_deferred_inputs)
  && Number(testEvidence.final_tree_quick_gate_seconds) > 0
  && testEvidence.final_tree_quick_rust_receipt === "CacheHit"
  && testEvidence.full_gate_required === true
  && testEvidence.full_gate_passed === true
  && Number(testEvidence.full_gate_seconds) > 0
  && testEvidence.full_generated_checks === 33
  && testEvidence.full_source_cache_skips === 4
  && testEvidence.full_workspace_harnesses === 34,
"P7-B3 test evidence drift");

console.log(
  "Goal 20 P7-B3 verified (human/JSON config, coverage, 12-battle run and "
  + "81107-byte ReplayV2 round trip).",
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
