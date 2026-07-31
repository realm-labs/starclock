#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

const rootArgument = process.argv.slice(2).find((value) => !value.startsWith("--"));
const root = path.resolve(rootArgument ?? ".");
const bless = process.argv.includes("--bless");
const policyPath = "policy/goal07-hardening.json";
const evidencePath =
  "evidence/standard-universe-mechanics-complete-v1/hardening/runtime-hardening.json";
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const json = (relative) => JSON.parse(read(relative));
const sha256 = (value) =>
  crypto.createHash("sha256").update(value).digest("hex");
const assert = (condition, message) => {
  if (!condition) {
    throw new Error(message);
  }
};
const includes = (source, marker, label) =>
  assert(source.includes(marker), `${label} omits ${JSON.stringify(marker)}`);

const policy = json(policyPath);
assert(
  policy.schema_revision === "starclock.goal07-hardening.v1",
  "unexpected Goal 07 hardening policy revision",
);
assert(policy.batch === "G07-P6-B3", "unexpected Goal 07 hardening batch");
assert(
  policy.wall_budget_seconds >= 60 && policy.wall_budget_seconds <= 180,
  "focused hardening budget must remain within one to three minutes",
);
assert(
  Object.values(policy.contracts).every((value) => value === true),
  "every Goal 07 hardening contract must be enabled",
);

const sources = {
  enemyAi: "crates/starclock-ai/src/tests.rs",
  baselineAi: "crates/starclock-ai/src/baseline.rs",
  activityRng: "crates/starclock-activity/src/activity_rng.rs",
  activityHardening: "crates/starclock-test-kit/tests/suites/activity/activity/activity_hardening.rs",
  activityTransaction: "crates/starclock-test-kit/tests/suites/activity/activity/activity_transaction.rs",
  combatRng: "crates/starclock-test-kit/tests/suites/core/combat/rng_golden.rs",
  combatResources: "crates/starclock-test-kit/tests/suites/core/combat/action_resources.rs",
  curioRuntime: "crates/starclock-test-kit/tests/suites/universe/curio_runtime.rs",
  runRuntime: "crates/starclock-test-kit/tests/suites/universe/run_runtime.rs",
  runLongRun:
    "crates/starclock-test-kit/tests/suites/universe/run_runtime/hardening.rs",
  activitySession: "crates/starclock-test-kit/tests/suites/adapter/agent_api/activity_session_loop.rs",
  agentHardening: "crates/starclock-test-kit/tests/suites/exhaustive/agent_api/hardening_corpus.rs",
  standardSessions: "crates/starclock-test-kit/tests/suites/adapter/agent_api/standard_session_loop.rs",
};
const sourceText = Object.fromEntries(
  Object.entries(sources).map(([key, relative]) => [key, read(relative)]),
);
for (const [key, marker] of Object.entries({
  enemyAi: "no_target_fallback_returns_only_an_exact_offered_command",
  baselineAi: "missing_authored_hint_rejects_instead_of_inventing_a_score",
  activityRng: `pub const ALL: [Self; ${policy.corpora.activity_rng_streams}]`,
  activityHardening:
    "four_thousand_ninety_six_invalid_commands_preserve_bytes_hash_and_rng",
  activityTransaction:
    "internal_fault_discards_partial_work_and_commits_only_faulted_settlement",
  combatRng: "stream_path_components_isolate_future_activity_substreams",
  combatResources:
    "ultimate_and_skill_resources_gate_offers_and_multi_hit_target_locks",
  curioRuntime:
    "hundred_curio_charge_cycles_stay_bounded_and_reject_stale_consumption",
  runLongRun:
    "hundred_fragment_cycles_preserve_bounds_and_rejected_spends_roll_back",
  activitySession:
    "concurrent_real_sessions_share_catalog_but_not_mutable_state",
  agentHardening: "seeded_race_corpus_allows_exactly_one_commit_per_round",
  standardSessions:
    "every_frozen_standard_scenario_finishes_through_agent_values_only",
})) {
  includes(sourceText[key], marker, `${key} hardening fixture`);
}

const corpus = policy.corpora;
assert(
  (sourceText.enemyAi.match(/#\[test\]/g) ?? []).length ===
    corpus.enemy_ai_legality_tests,
  "enemy AI legality-test denominator drift",
);
assert(
  (sourceText.baselineAi.match(/#\[test\]/g) ?? []).length ===
    corpus.baseline_player_legality_tests,
  "baseline player legality-test denominator drift",
);
for (const [source, marker, label] of [
  [sourceText.activitySession, `const SESSIONS: usize = ${corpus.concurrent_shared_factory_sessions}`, "concurrent session denominator"],
  [sourceText.activitySession, `const CORPUS_CASES: usize = ${corpus.agent_replay_mutations}`, "Agent replay mutation denominator"],
  [sourceText.activityHardening, `0..${corpus.invalid_activity_commands.toLocaleString("en-US").replace(",", "_")}_u32`, "invalid Activity command denominator"],
  [sourceText.activityHardening, `1..=${corpus.rng_perturbation_draws_per_stream}_u16`, "RNG perturbation denominator"],
  [sourceText.runLongRun, `const CYCLES: u64 = ${corpus.long_run_resource_cycles}`, "resource cycle denominator"],
  [sourceText.curioRuntime, `const CYCLES: u32 = ${corpus.long_run_curio_charge_cycles}`, "Curio charge cycle denominator"],
]) {
  includes(source, marker, label);
}
const hardeningCorpus = json(
  "evidence/agent-control-mcp-v1/security/hardening-corpus.json",
);
assert(
  hardeningCorpus.races.rounds === corpus.idempotency_race_rounds,
  "idempotency race-round denominator drift",
);
assert(
  hardeningCorpus.settlement.scenario_ids.length ===
    corpus.standard_battle_scenarios,
  "standard Battle scenario denominator drift",
);

const matrixEvidence = json(policy.matrix_evidence.path);
const matrix = matrixEvidence.matrix.coverage;
for (const field of [
  "complete_runs",
  "nested_battles",
  "battle_commands",
  "battle_state_records",
]) {
  assert(
    matrix[field] === policy.matrix_evidence[field],
    `integrated matrix ${field} drift`,
  );
}
assert(
  matrix.battle_commands === matrix.battle_state_records,
  "integrated matrix battle command/state parity drift",
);

const started = process.hrtime.bigint();
for (const command of policy.focused_commands) {
  execFileSync(command[0], command.slice(1), {
    cwd: root,
    stdio: "inherit",
    windowsHide: true,
  });
}
const elapsedMs = Number((process.hrtime.bigint() - started) / 1_000_000n);
assert(
  elapsedMs <= policy.wall_budget_seconds * 1_000,
  `Goal 07 hardening exceeded ${policy.wall_budget_seconds}s: ${elapsedMs}ms`,
);

const evidence = {
  schema_revision: "starclock.goal07-hardening-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "pass",
  coverage: {
    ai: {
      enemy_legality_tests: corpus.enemy_ai_legality_tests,
      baseline_player_legality_tests: corpus.baseline_player_legality_tests,
      standard_battle_scenarios: corpus.standard_battle_scenarios,
      complete_universe_runs: matrix.complete_runs,
      production_nested_battles: matrix.nested_battles,
      production_battle_commands: matrix.battle_commands,
    },
    concurrency: {
      shared_factory_sessions: corpus.concurrent_shared_factory_sessions,
      idempotency_race_rounds: corpus.idempotency_race_rounds,
    },
    rollback: {
      invalid_activity_commands: corpus.invalid_activity_commands,
      agent_replay_mutations: corpus.agent_replay_mutations,
      resource_rejections: corpus.long_run_resource_cycles,
      stale_charge_rejections: corpus.long_run_curio_charge_cycles,
    },
    rng_isolation: {
      streams: corpus.activity_rng_streams,
      perturbation_draws_per_stream: corpus.rng_perturbation_draws_per_stream,
    },
    long_run: {
      resource_cycles: corpus.long_run_resource_cycles,
      resource_commands:
        corpus.long_run_resource_cycles * corpus.resource_commands_per_cycle,
      curio_charge_cycles: corpus.long_run_curio_charge_cycles,
      charge_consumptions:
        corpus.long_run_curio_charge_cycles *
        corpus.curio_charge_consumptions_per_cycle,
    },
  },
  contracts: policy.contracts,
  inputs: {
    policy_sha256: sha256(read(policyPath)),
    matrix_evidence_sha256: sha256(read(policy.matrix_evidence.path)),
    hardening_corpus_sha256: sha256(
      read("evidence/agent-control-mcp-v1/security/hardening-corpus.json"),
    ),
    sources: Object.fromEntries(
      Object.entries(sources).map(([key, relative]) => [
        key,
        { path: relative, sha256: sha256(read(relative)) },
      ]),
    ),
  },
};
const output = `${JSON.stringify(evidence, null, 2)}\n`;
const absoluteEvidence = path.join(root, evidencePath);
if (bless) {
  fs.mkdirSync(path.dirname(absoluteEvidence), { recursive: true });
  fs.writeFileSync(absoluteEvidence, output);
} else {
  assert(
    fs.existsSync(absoluteEvidence),
    "Goal 07 hardening evidence is missing; run with --bless",
  );
  assert(
    read(evidencePath).replaceAll("\r\n", "\n") === output,
    "Goal 07 hardening evidence is stale; run with --bless",
  );
}
console.log(
  `Goal 07 hardening verified (${matrix.complete_runs} runs, ` +
    `${corpus.concurrent_shared_factory_sessions} concurrent sessions, ` +
    `${corpus.long_run_resource_cycles} long-run cycles).`,
);
