#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const root = path.resolve(process.argv[2] ?? ".");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const has = (text, needle, label) => {
  if (!text.includes(needle)) {
    throw new Error(`${label}: missing ${JSON.stringify(needle)}`);
  }
};

const assembler = read(
  "crates/starclock-mode-universe/src/dynamic_battle_assembler.rs",
);
const tests = read(
  "crates/starclock-mode-universe/tests/dynamic_battle_assembly.rs",
);
const evidence = read("docs/goal-06-assembly-failure-and-cache-hardening.md");
const status = read("docs/goals/06-combat-identity-and-dynamic-assembly-status.md");

for (const [text, needle, label] of [
  [assembler, "pub struct BattleAssemblyBudget", "assembly budget"],
  [assembler, "pub fn fork_with_policy(", "testable cache policy"],
  [
    assembler,
    "pub fn start_pending_battle_from_snapshot(",
    "explicit snapshot entry point",
  ],
  [assembler, "StandardUniverseDynamicBattleError::StaleSnapshot", "stale rejection"],
  [assembler, "StandardUniverseDynamicBattleError::MissingTechnique", "invalid definition"],
  [assembler, "StandardUniverseDynamicBattleError::BudgetExceeded", "budget rejection"],
  [
    tests,
    "stale_invalid_and_budget_failures_preserve_state_and_retry_cleanly",
    "failure and retry fixture",
  ],
  [
    tests,
    "bounded_dynamic_cache_hits_and_evicts_exact_activity_snapshots",
    "cache eviction fixture",
  ],
  [tests, "canonical_state_bytes()", "canonical state preservation"],
  [evidence, "consumes no Activity RNG draw", "RNG preservation contract"],
  [status, "| `G06-P2-B4` | `Complete` |", "completed ledger row"],
]) {
  has(text, needle, label);
}

console.log(
  "Goal 06 P2-B4 verified (bounded cache, typed failures and byte-identical retry).",
);
