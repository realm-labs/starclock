#!/usr/bin/env node

import fs from "node:fs";

function text(path) {
  return fs.readFileSync(path, "utf8");
}
function json(path) {
  return JSON.parse(text(path));
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const evidence = json(
  "evidence/gold-and-gears-runtime-v1/hardening/phase8-b1.json",
);
assert(
  evidence.schema_revision === "starclock.gold-and-gears-phase8-b1-evidence.v1"
    && evidence.goal_id === "gold-and-gears-runtime-v1"
    && evidence.batch === "G14-P8-B1"
    && evidence.result === "LocalWindowsPassCrossPlatformContractFrozen",
  "P8-B1 evidence identity drift",
);
const generated = json(
  "evidence/gold-and-gears-runtime-v1/hardening/determinism-hardening.json",
);
assert(
  generated.schema_revision
    === "starclock.goal14-determinism-hardening-evidence.v1"
    && generated.result
      === "cross-platform-native-contract-and-local-windows-vectors-frozen",
  "P8-B1 generated hardening evidence drift",
);

const goldens = evidence.goldens;
for (const [key, value] of Object.entries(goldens))
  assert(generated.goldens[key] === value, `generated golden drift: ${key}`);
assert(
  goldens.records === 356
    && goldens.activity_actions === 62
    && goldens.battle_commands === 99
    && goldens.nested_battles === 17
    && goldens.replay_bytes === 111347,
  "P8-B1 golden denominators drift",
);

const corpora = evidence.corpora;
assert(
  corpora.rng_domains === 7
    && corpora.draws_per_perturbation === 257
    && corpora.seed_property_cases === 64
    && corpora.corrupted_candidate_cases === 3
    && corpora.deterministic_fault_cases === 1
    && corpora.invalid_actions === 4096
    && corpora.malformed_gold_replays === 256
    && corpora.maximum_arbitrary_replay_bytes === 4096
    && corpora.inherited_replay_property_tests === 9
    && corpora.all_exhaustive_tests === 25,
  "P8-B1 corpus denominators drift",
);
for (const [key, value] of Object.entries(evidence.invariants)) {
  if (key === "runtime_behavior_changed" || key === "new_registry_packages") continue;
  assert(value === true, `missing P8-B1 invariant ${key}`);
}
assert(evidence.invariants.runtime_behavior_changed === false
  && evidence.invariants.new_registry_packages === 0,
"P8-B1 implementation boundary drift");

const platform = evidence.platform_contract;
assert(
  platform.native_gate === "node tools/goal14/run-native-ci.mjs --hardening"
    && platform.required_native_profiles.length === 3
    && platform.locally_executed_profile === "windows-x64-native"
    && platform.locally_executed === true
    && platform.hosted_linux_and_macos_proof_retained_only_after_successful_ci === true
    && platform.hosted_linux_and_macos_currently_claimed === false
    && platform.compile_only_profiles === 3
    && platform.compile_only_runtime_claims === 0,
  "P8-B1 platform evidence overclaims execution",
);

const hardening = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/hardening_tests.rs",
);
for (const literal of [
  "gold_rng_domains_are_golden_and_do_not_shift_battle_or_unrelated_streams",
  "initial_offers_and_state_are_property_stable_across_seed_corpus",
  "corrupted_candidate_failures_are_repeatable_and_bounded",
  "gold_state_fault_is_deterministic_and_discards_partial_mutation",
  goldens.rng_domain_sha256,
]) assert(hardening.includes(literal), `missing Gold hardening vector ${literal}`);
assert(!hardening.includes("use sha2"),
  "Gold hardening bypasses the private digest owner");

const replay = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/replay_tests.rs",
);
for (const key of [
  "replay_sha256",
  "activity_command_sha256",
  "battle_command_sha256",
  "battle_event_state_sha256",
  "activity_state_sha256",
]) assert(replay.includes(goldens[key]), `missing replay golden ${key}`);

const agent = text(
  "crates/starclock-test-kit/tests/suites/exhaustive/agent_api/gold_gears_hardening.rs",
);
for (const literal of [
  "four_thousand_ninety_six_forged_gold_actions_preserve_exact_observation",
  "two_hundred_fifty_six_malformed_gold_replays_fail_repeatably_without_live_mutation",
  "0..4_096_u32",
  "cases: 256",
  "prop_assert_eq!(&first, &second)",
]) assert(agent.includes(literal), `missing Gold agent hardening vector ${literal}`);

const workflow = text(".github/workflows/ci.yml");
const ciPolicy = json("policy/ci-matrix.json");
assert(workflow.includes(`run: ${ciPolicy.repository_gate}`) &&
  !workflow.includes(`run: ${platform.native_gate}`),
"current CI must run one full repository pass without replaying P8-B1");
const policy = json("policy/goal14-determinism-hardening.json");
assert(policy.evidence_boundary.compile_only_runtime_claims === 0
  && policy.evidence_boundary.linux_and_macos_claim_requires_successful_hosted_run === true,
"P8-B1 cross-platform policy drift");

for (const [path, limit] of [
  ["crates/starclock-mode-universe/src/gold_gears_entry/hardening_tests.rs", 400],
  ["crates/starclock-mode-universe/src/gold_gears_entry/replay_tests.rs", 400],
  ["crates/starclock-test-kit/tests/suites/exhaustive/agent_api/gold_gears_hardening.rs", 300],
]) assert(text(path).split(/\r?\n/u).length <= limit,
  `${path} exceeds ${limit} lines`);

const tests = evidence.tests;
assert(
  tests.focused_gold_hardening_tests === 4
    && tests.focused_gold_replay_tests === 1
    && tests.focused_gold_agent_corpus_tests === 2
    && tests.native_gate_inherited_replay_tests === 9
    && tests.native_gate_seconds === "15.5"
    && tests.full_gold_entry_tests === 138
    && tests.full_exhaustive_tests === 25
    && tests.all_agent_api_tests === 32
    && tests.clippy_passed === true
    && tests.dependency_policy_passed === true
    && tests.generated_drift_checks === 29
    && tests.generated_source_cache_checks_skipped === 4
    && tests.goal_verifiers_passed === true
    && tests.cold_quick_timeouts === 2
    && tests.quick_gate_passed === true
    && tests.quick_gate_seconds === "125.7"
    && tests.quick_selected_harnesses === 7
    && tests.quick_direct_packages === 2
    && tests.quick_downstream_packages === 3
    && tests.quick_deferred_inputs === 6
    && tests.final_quick_cache_hit === true
    && tests.final_quick_deferred_inputs === 8
    && tests.final_quick_gate_seconds === "5.5"
    && tests.full_gate_passed === true
    && tests.full_gate_seconds === "282.9"
    && tests.full_workspace_harnesses === 33,
  "P8-B1 test receipt drift",
);

const ledger = text("docs/goals/14-gold-and-gears-runtime-status.md");
assert(
  ledger.includes("| `G14-P8-B1` | `Complete` |"),
  "P8-B1 completion row is missing",
);

console.log(
  "Goal 14 P8-B1 verified (356 records, 7 RNG domains, 4096 rejections and 256 malformed replays).",
);
