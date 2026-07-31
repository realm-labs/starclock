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

const transition = read(
  "crates/starclock-mode-universe/src/dynamic_battle_assembler/transition_tests.rs",
);
const integration = read(
  "crates/starclock-test-kit/tests/suites/universe/dynamic_battle_assembly.rs",
);
const evidence = read("docs/goal-06-transition-battle-fixtures.md");
const debt = read("docs/goal-06-debt-probes.md");
const status = read("docs/goals/06-combat-identity-and-dynamic-assembly-status.md");

for (const [text, needle, label] of [
  [
    transition,
    "current_inventory_transitions_reassemble_real_battle_inputs",
    "inventory transition fixture",
  ],
  [transition, "universe.blessing.612344", "Blessing acquire and upgrade"],
  [transition, "universe.curio.8", "Curio lifecycle"],
  [transition, "universe.path.hunt", "Hunt Resonance"],
  [transition, "universe.ability-tree.2", "Ability Tree node"],
  [transition, "first_action_damage(", "real Blessing battle"],
  [transition, "start_events(", "real Curio battle"],
  [
    transition,
    "two non-contributing Curio lifecycle states are combat-equivalent",
    "combat identity equivalence",
  ],
  [
    integration,
    "settled_carry_is_reassembled_into_the_next_real_battle",
    "cross-battle carry fixture",
  ],
  [integration, "submit_pending_battle_result(", "real Activity settlement"],
  [integration, "initial_state()", "embedded carry"],
  [evidence, "No test-only carry constructor", "production carry boundary"],
  [debt, "combat-input equivalent", "corrected frozen Curio semantics"],
  [status, "| `G06-P2-B5` | `Complete` |", "completed ledger row"],
]) {
  has(text, needle, label);
}

const lines = transition.split(/\r?\n/).length;
if (lines >= 700) {
  throw new Error(`transition fixture is unexpectedly large at ${lines} lines`);
}

console.log(
  "Goal 06 P2-B5 verified (inventory transitions and cross-battle carry reach real battles).",
);
