#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/hardening/phase8-b1.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-phase8-b1-evidence.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P8-B1"
  && evidence.result === "LocalMacosArm64PassCrossPlatformContractFrozen",
"P8-B1 evidence identity drift");
const generated = json("evidence/swarm-disaster-runtime-v1/hardening/determinism-hardening.json");
assert(generated.schema_revision === "starclock.goal20-determinism-hardening-evidence.v1"
  && generated.result === "cross-platform-native-contract-and-local-macos-arm64-vectors-frozen",
"P8-B1 generated hardening evidence drift");

for (const [key, value] of Object.entries(evidence.goldens))
  assert(generated.goldens[key] === value, `generated golden drift: ${key}`);
const goldens = evidence.goldens;
assert(goldens.records === 268 && goldens.activity_actions === 48
  && goldens.battle_commands === 74 && goldens.nested_battles === 12
  && goldens.replay_bytes === 88813, "P8-B1 golden denominators drift");
const corpora = evidence.corpora;
assert(corpora.rng_domains === 8 && corpora.draws_per_perturbation === 257
  && corpora.seed_property_cases === 64 && corpora.corrupted_candidate_cases === 3
  && corpora.deterministic_fault_cases === 1 && corpora.invalid_actions === 4096
  && corpora.malformed_swarm_replays === 256
  && corpora.maximum_arbitrary_replay_bytes === 4096
  && corpora.native_replay_filter_tests === 10 && corpora.all_exhaustive_tests === 27,
"P8-B1 corpus denominators drift");
for (const [key, value] of Object.entries(evidence.invariants)) {
  if (key === "runtime_behavior_changed" || key === "new_registry_packages") continue;
  assert(value === true, `missing P8-B1 invariant ${key}`);
}
assert(evidence.invariants.runtime_behavior_changed === false
  && evidence.invariants.new_registry_packages === 0, "P8-B1 implementation boundary drift");

const platform = evidence.platform_contract;
assert(platform.native_gate === "node tools/goal20/run-native-ci.mjs --hardening"
  && platform.required_native_profiles.length === 3
  && platform.locally_executed_profile === "macos-arm64-native"
  && platform.locally_executed === true
  && platform.hosted_windows_and_linux_proof_retained_only_after_successful_ci === true
  && platform.hosted_windows_and_linux_currently_claimed === false
  && platform.compile_only_profiles === 3 && platform.compile_only_runtime_claims === 0,
"P8-B1 platform evidence overclaims execution");

const hardening = text("crates/starclock-mode-universe/src/swarm_disaster_entry/hardening_tests.rs");
for (const literal of [
  "swarm_rng_domains_are_golden_and_do_not_shift_battle_or_unrelated_streams",
  "initial_offers_and_state_are_property_stable_across_swarm_seed_corpus",
  "corrupted_swarm_candidate_failures_are_repeatable_and_bounded",
  "swarm_state_fault_is_deterministic_and_discards_partial_mutation",
  goldens.rng_domain_sha256,
]) assert(hardening.includes(literal), `missing Swarm hardening vector ${literal}`);
assert(!hardening.includes("use sha2"), "Swarm hardening bypasses the private digest owner");
const replay = text("crates/starclock-mode-universe/src/swarm_disaster_entry/replay_tests.rs");
for (const key of [
  "replay_sha256", "activity_command_sha256", "battle_command_sha256",
  "battle_event_state_sha256", "activity_state_sha256",
]) assert(replay.includes(goldens[key]), `missing replay golden ${key}`);
const agent = text("crates/starclock-test-kit/tests/suites/exhaustive/agent_api/swarm_disaster_hardening.rs");
for (const literal of [
  "four_thousand_ninety_six_forged_swarm_actions_preserve_exact_observation",
  "two_hundred_fifty_six_malformed_swarm_replays_fail_repeatably_without_live_mutation",
  "0..4_096_u32", "cases: 256", "prop_assert_eq!(&first, &second)",
]) assert(agent.includes(literal), `missing Swarm agent hardening vector ${literal}`);

const workflow = text(".github/workflows/ci.yml");
const ci = json("policy/ci-matrix.json");
assert(workflow.includes(`run: ${ci.repository_gate}`)
  && !workflow.includes(`run: ${platform.native_gate}`),
"current CI must run one full repository pass without replaying P8-B1");
const policy = json("policy/goal20-determinism-hardening.json");
assert(policy.evidence_boundary.compile_only_runtime_claims === 0
  && policy.evidence_boundary.windows_and_linux_claim_requires_successful_hosted_run === true,
"P8-B1 cross-platform policy drift");
for (const [relative, limit] of [
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/hardening_tests.rs", 400],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/replay_tests.rs", 400],
  ["crates/starclock-test-kit/tests/suites/exhaustive/agent_api/swarm_disaster_hardening.rs", 300],
]) assert(text(relative).split(/\r?\n/u).length - 1 <= limit,
  `${relative} exceeds ${limit} lines`);

const tests = evidence.tests;
assert(tests.focused_swarm_hardening_tests === 4
  && tests.focused_swarm_replay_tests === 1
  && tests.focused_swarm_agent_corpus_tests === 2
  && tests.native_gate_replay_filter_tests === 10 && tests.native_gate_seconds === "9.23"
  && tests.full_swarm_entry_tests === 146 && tests.full_exhaustive_tests === 27
  && tests.all_agent_api_tests === 35 && tests.clippy_passed === true
  && tests.dependency_policy_passed === true && tests.workflow_policy_passed === true
  && tests.source_policy_passed === true && tests.handwritten_rust_files === 966
  && tests.public_reexport_declarations === 72 && tests.generated_drift_checks === 34
  && tests.generated_source_cache_checks_skipped === 4
  && tests.goal_verifiers_passed === true, "P8-B1 test receipt drift");

const ledger = text("docs/goals/20-swarm-disaster-runtime-status.md");
const complete = ledger.includes("| `G20-P8-B1` | `Complete` |");
if (complete) {
  assert(ledger.includes("| Active batch | None |")
    && ledger.includes("| Next unblocked batch | `G20-P8-B2` |"), "P8-B1 ledger state drift");
  assert(tests.quick_gate_passed === true && Number(tests.quick_gate_seconds) > 0
    && Number.isInteger(tests.quick_selected_harnesses)
    && Number.isInteger(tests.quick_direct_packages)
    && Number.isInteger(tests.quick_downstream_packages)
    && Number.isInteger(tests.quick_deferred_inputs)
    && tests.final_quick_cache_hit === true
    && Number.isInteger(tests.final_quick_deferred_inputs)
    && Number(tests.final_quick_gate_seconds) > 0
    && tests.full_gate_passed === true && Number(tests.full_gate_seconds) > 0
    && tests.full_workspace_harnesses === 34 && tests.full_generated_checks === 34
    && tests.full_source_cache_checks_skipped === 4, "P8-B1 terminal gate receipt drift");
} else {
  assert(ledger.includes("| Active batch | `G20-P8-B1` |"), "P8-B1 in-progress ledger drift");
}

console.log("Goal 20 P8-B1 verified (268 records, 8 RNG domains, 4096 rejections and 256 malformed replays).");

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function assert(condition, message) { if (!condition) throw new Error(message); }
