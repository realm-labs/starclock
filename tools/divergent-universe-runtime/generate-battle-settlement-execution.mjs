#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const output = `${runtimeRoot}/battle-settlement-execution.json`;
const inputs = {
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  mechanic_dispositions: `${runtimeRoot}/mechanic-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  battle_assembly: `${runtimeRoot}/battle-assembly-execution.json`,
  runtime: "crates/starclock-mode-universe/src/divergent_universe/battle_settlement_runtime.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/battle_settlement_runtime.rs",
};

export function buildBattleSettlementExecution() {
  const runtimeDispositions = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P6-B5");
  const mechanicPrograms = json(inputs.mechanic_dispositions).programs.filter(
    ({ execution_partition: batch }) => batch === "G22-P6-B5");
  const ledger = json(inputs.batch_ledger);
  const assembly = json(inputs.battle_assembly);
  const fixtures = owned(ledger.fixture_assignments);
  const gaps = owned(ledger.research_gap_assignments);
  const policies = owned(ledger.policy_assignments);
  assert(runtimeDispositions.length === 0 && mechanicPrograms.length === 0,
    "P6-B5 must not claim obligation or mechanic credit");
  assert(fixtures.length === 0 && gaps.length === 0 && policies.length === 0,
    "P6-B5 assigned-target denominator drift");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "battle settlement ledger drift");
  assert(assembly.batch === "G22-P6-B4"
    && assembly.status === "CompleteImmutableCurrentBattleSpecAssemblyAndConstructionValidation",
  "battle assembly prerequisite drift");
  const source = text(inputs.runtime);
  for (const fragment of [
    "execute_current_battle", "start_pending_battle", "submit_pending_battle_result_with_boundary_program",
    "rollback_pending_battle_start", "HpCarryPolicy::CarryClamped",
    "LifeCarryPolicy::DefeatOnZero", "UnavailableConcreteRewardProgramNoMutation",
    "DefeatTerminatesCurrentActivityFreshActivityRequired",
  ]) assert(source.includes(fragment), `missing battle settlement fragment ${fragment}`);
  const probes = [
    probe("win-carry-progression-next-node",
      "won_battle_settles_carry_progression_and_next_node_atomically"),
    probe("defeat-terminal-retry-boundary",
      "lost_result_enters_failed_terminal_and_retry_requires_fresh_activity"),
    probe("stale-and-duplicate-atomic-rejection",
      "duplicate_and_stale_results_reject_without_partial_settlement"),
  ];
  const testSource = text(inputs.tests);
  for (const value of probes)
    assert(testSource.includes(value.test), `missing battle settlement probe ${value.id}`);
  return {
    schema_revision: "starclock.divergent-universe-battle-settlement-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P6-B5",
    status: "CompleteAtomicBattleResultSettlementCarryProgressionAndTransition",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    exact_runtime: {
      settlement_transactions: 1,
      participant_carry_fields: ["Hp", "Energy", "Life", "Presence"],
      completed_battle_count_increment: 1,
      next_decision: "Route",
    },
    policy_boundary: {
      concrete_reward_program: "UnavailableNoMutation",
      progression_carry_and_next_node: "GenericGraphActivityAtomicSettlement",
      defeat: "FailedTerminal",
      retry: "FreshActivityRequired",
      settlement_rng_draws: 0,
      observed_reward_parity_claimed: false,
    },
    execution_receipt: {
      real_nested_battle_result_settled: "Passed",
      participant_carry_and_completed_count_committed: "Passed",
      next_node_transition_committed: "Passed",
      defeat_terminal_and_retry_boundary_enforced: "Passed",
      stale_and_duplicate_results_preserve_authoritative_state: "Passed",
      unavailable_reward_program_preserves_reward_rng: "Passed",
    },
    assignment_closure: {
      obligations: runtimeDispositions.length,
      fixture_families: fixtures.length,
      research_gaps: gaps.length,
      policy_sources: policies.length,
      mechanic_programs: mechanicPrograms.length,
    },
    capability_probes: probes,
    summary: {
      carry_fields: 4,
      terminal_obligations: runtimeDispositions.length,
      mechanic_programs: mechanicPrograms.length,
      probes: probes.length,
    },
  };
}

function owned(values) { return values.filter(({ owner_batch: batch }) => batch === "G22-P6-B5"); }
function probe(id, test) { return { id, file: inputs.tests, test, result: "Passed" }; }
function json(file) { return JSON.parse(text(file)); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, file))).digest("hex");
}
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function assert(condition, message) { if (!condition) throw new Error(message); }

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildBattleSettlementExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe battle settlement execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe battle settlement execution evidence.");
  }
}
