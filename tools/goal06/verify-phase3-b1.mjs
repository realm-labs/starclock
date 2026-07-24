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

const runner = read("crates/starclock-mode-universe/src/baseline_runner.rs");
const executor = read(
  "crates/starclock-mode-universe/src/nested_battle_executor.rs",
);
const replay = read(
  "crates/starclock-mode-universe/src/universe_replay_v3.rs",
);
const cli = read("crates/starclock-cli/src/universe_v1.rs");
const cliTest = read("crates/starclock-cli/tests/universe_cli.rs");
const integration = read(
  "crates/starclock-mode-universe/tests/dynamic_battle_assembly.rs",
);
const evidence = read("docs/goal-06-dynamic-baseline-cli-replay-v3.md");
const status = read("docs/goals/06-combat-identity-and-dynamic-assembly-status.md");

for (const [text, needle, label] of [
  [runner, "pub trait DynamicNestedBattleExecutor", "dynamic executor contract"],
  [runner, "pub fn run_to_terminal_dynamic", "dynamic baseline runner"],
  [runner, ".start_pending_battle(activity)", "shared assembler start"],
  [executor, "pub fn dynamic()", "dynamic-only production executor"],
  [executor, "start.combat_catalog()", "paired exact catalog"],
  [replay, "pub fn record_baseline_run_v3(", "dynamic v3 recorder"],
  [
    replay,
    "verify_standard_universe_replay_v3_dynamic",
    "dynamic v3 verifier",
  ],
  [cli, "record_baseline_run_v3(", "CLI v3 recording"],
  [cli, "UniverseNestedBattleExecutor::dynamic()", "CLI dynamic execution"],
  [cli, "starclock-cli-universe-v3", "CLI schema revision"],
  [cliTest, "decode_replay_v3", "CLI v3 golden"],
  [
    integration,
    "production_baseline_records_and_verifies_dynamic_replay_v3",
    "production integration fixture",
  ],
  [evidence, "released replay-v2", "historical compatibility boundary"],
  [status, "| `G06-P3-B1` | `Complete` |", "completed ledger row"],
  [status, "| Next unblocked batch | `G06-P3-B2` |", "next batch"],
]) {
  has(text, needle, label);
}

for (const relative of [
  "crates/starclock-mode-universe/src/universe_replay_v2.rs",
  "crates/starclock-mode-universe/src/nested_battle_executor.rs",
]) {
  const lines = read(relative).split(/\r?\n/).length;
  if (lines >= 1_050) {
    throw new Error(`${relative} remains near-limit at ${lines} lines`);
  }
}

console.log(
  "Goal 06 P3-B1 verified (dynamic baseline, CLI and replay-v3 round trip).",
);
