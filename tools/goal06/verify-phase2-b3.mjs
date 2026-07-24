#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const root = path.resolve(process.argv[2] ?? ".");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const has = (text, needle, label) => {
  if (!text.includes(needle)) throw new Error(`${label}: missing ${JSON.stringify(needle)}`);
};

const activity = read("crates/starclock-activity/src/graph_activity.rs");
const preparation = read("crates/starclock-activity/src/battle_preparation.rs");
const assembler = read(
  "crates/starclock-mode-universe/src/dynamic_battle_assembler.rs",
);
const materializer = read(
  "crates/starclock-mode-universe/src/battle_materialization.rs",
);
const player = read(
  "crates/starclock-mode-universe/src/battle_materialization/player.rs",
);
const production = read("crates/starclock-mode-universe/src/production_runtime.rs");
const tests = read(
  "crates/starclock-mode-universe/tests/dynamic_battle_assembly.rs",
);
const status = read("docs/goals/06-combat-identity-and-dynamic-assembly-status.md");

for (const [text, needle, label] of [
  [activity, "pub fn start_assembled_pending_battle(", "atomic generic start"],
  [activity, "let mut working = self.state.transaction_copy();", "working Activity state"],
  [activity, "self.state = working;", "publish after success"],
  [preparation, "replace_pending_binding(", "placeholder replacement"],
  [preparation, "binding.battle_spec().encounter() != pending.battle_spec().encounter()", "encounter validation"],
  [assembler, "pub struct StandardUniverseBattleAssembler", "shared assembler"],
  [assembler, "snapshot.source_state_hash() != expected_state_hash", "stale snapshot check"],
  [assembler, ".start_assembled_pending_battle(", "atomic mode handoff"],
  [assembler, "combat_catalog: Arc<CombatCatalog>", "paired catalog"],
  [materializer, "compile_snapshot_from_composition", "snapshot materialization"],
  [materializer, "snapshot_root_digest(static_digest, snapshot.digest())", "assembly provenance"],
  [player, "ParticipantInitialState::new(", "carry application"],
  [production, "battle_assembler: Arc<StandardUniverseBattleAssembler>", "production ownership"],
  [tests, "pending_encounter_is_assembled_and_sealed_from_one_current_snapshot", "integration fixture"],
  [tests, "Battle::create(", "real executable request"],
  [status, "| `G06-P2-B3` | `Complete` |", "completed ledger row"],
]) has(text, needle, label);

for (const relative of [
  "crates/starclock-mode-universe/src/battle_materialization.rs",
  "crates/starclock-activity/src/battle_preparation.rs",
]) {
  const lines = read(relative).split(/\r?\n/).length;
  if (lines >= 1_050) throw new Error(`${relative} remains near-limit at ${lines} lines`);
}

console.log("Goal 06 P2-B3 verified (snapshot assembly and atomic pending-battle seal).");
