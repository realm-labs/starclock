#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const root = path.resolve(process.argv[2] ?? ".");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const has = (text, needle, label) => {
  if (!text.includes(needle)) throw new Error(`${label}: missing ${JSON.stringify(needle)}`);
};

const snapshot = read("crates/starclock-mode-universe/src/battle_snapshot.rs");
const access = read(
  "crates/starclock-mode-universe/src/runtime/battle_contribution_access.rs",
);
const activityView = read("crates/starclock-activity/src/view.rs");
const tests = read("crates/starclock-test-kit/tests/suites/universe/encounter_runtime.rs");
const status = read("docs/goals/06-combat-identity-and-dynamic-assembly-status.md");

for (const [text, needle, label] of [
  [snapshot, "pub struct StandardUniverseBattleSnapshot", "immutable snapshot"],
  [snapshot, "source_state_hash: ActivityStateHash", "stale-state identity"],
  [snapshot, "path: PathContributionSet", "Path projection"],
  [snapshot, "blessings: BlessingContributionSet", "Blessing levels"],
  [snapshot, "curios: CurioContributionSet", "Curio lifecycle"],
  [snapshot, "ability_tree: AbilityTreeContributionSet", "Ability Tree selection"],
  [snapshot, "ability_projection: AbilityRuntimeProjection", "Ability Tree values"],
  [snapshot, "participant_carry: Box<[ActivityParticipantCarryState]>", "carry projection"],
  [snapshot, "encoder.digest(source_state_hash.bytes())", "snapshot provenance"],
  [snapshot, "encoder.digest(carry_digest)", "carry identity"],
  [access, "pub fn battle_start_snapshot(", "authoritative projection entry"],
  [access, "view.completed_battle_count() > 0", "derived battle history"],
  [access, "Err(StandardUniverseBattleContributionError::ContextMismatch)", "context rejection"],
  [activityView, "pub const fn completed_battle_count", "generic completed count"],
  [tests, "snapshot.source_state_hash(), activity.view().state_hash()", "snapshot provenance fixture"],
  [tests, "assert!(snapshot.digest().iter().any(|byte| *byte != 0))", "snapshot identity fixture"],
  [tests, "before_reroll_snapshot.source_state_hash()", "rejected mutation provenance fixture"],
  [status, "| `G06-P2-B2` | `Complete` |", "completed ledger row"],
]) has(text, needle, label);

if (snapshot.includes("f32") || snapshot.includes("f64"))
  throw new Error("snapshot introduced authoritative floating point");
if (snapshot.includes("serde"))
  throw new Error("snapshot identity must use the canonical encoder");

console.log("Goal 06 P2-B2 verified (current typed Activity battle snapshot and carry).");
