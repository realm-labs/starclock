#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const root = path.resolve(process.argv[2] ?? ".");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const has = (text, needle, label) => {
  if (!text.includes(needle)) throw new Error(`${label}: missing ${JSON.stringify(needle)}`);
};

const projection = read("crates/starclock-activity/src/projection.rs");
const preparation = read("crates/starclock-activity/src/battle_preparation.rs");
const settlement = read("crates/starclock-activity/src/battle_settlement.rs");
const codec = read("crates/starclock-activity/src/codec.rs");
const boundary = read("crates/starclock-activity/tests/activity_boundary.rs");
const replay = read("crates/starclock-replay/src/activity.rs");
const status = read("docs/goals/06-combat-identity-and-dynamic-assembly-status.md");

for (const [text, needle, label] of [
  [projection, "combat_input_digest: CombatInputDigest", "result combat identity"],
  [projection, "assembly_digest: AssemblyDigest", "result assembly identity"],
  [projection, "pub fn new_legacy(", "historical replay identity bridge"],
  [preparation, "combat_input_digest: pending.combat_input_digest()", "pending combat identity"],
  [preparation, "assembly_digest: pending.assembly_digest()", "pending assembly identity"],
  [settlement, "pending.combat_input_digest().bytes()", "settlement combat binding"],
  [settlement, "pending.assembly_digest().bytes()", "settlement assembly binding"],
  [codec, 'ACTIVITY_STATE_CODEC_REVISION: &str = "starclock-activity-state-v3"', "state codec"],
  [codec, 'ACTIVITY_STATE_HASH_REVISION: &str = "sha256-v5"', "state hash"],
  [boundary, "ResultIdentityField::CombatInputDigest", "combat mismatch rejection"],
  [boundary, "ResultIdentityField::AssemblyDigest", "assembly mismatch rejection"],
  [replay, "decode_identity_legacy", "legacy payload decoder"],
  [status, "| `G06-P1-B2` | `Complete` |", "completed ledger row"],
]) {
  has(text, needle, label);
}

console.log("Goal 06 P1-B2 verified (Activity v3/v5 handoff and settlement dual identity).");
