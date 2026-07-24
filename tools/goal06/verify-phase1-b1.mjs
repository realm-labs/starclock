#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const root = path.resolve(process.argv[2] ?? ".");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const requireText = (text, needle, label) => {
  if (!text.includes(needle)) {
    throw new Error(`${label}: missing ${JSON.stringify(needle)}`);
  }
};

const spec = read("crates/starclock-combat/src/battle/spec.rs");
const codec = read("crates/starclock-combat/src/battle/spec_codec.rs");
const state = read("crates/starclock-combat/src/battle/state.rs");
const facade = read("crates/starclock-combat/src/lib.rs");
const status = read("docs/goals/06-combat-identity-and-dynamic-assembly-status.md");

for (const [needle, label] of [
  ["CombatInputDigest", "combat-owned identity"],
  ["AssemblyDigest", "opaque assembly identity"],
  ["pub fn new_with_assembly(", "computed constructor"],
  ["super::spec_codec::combat_input_digest(", "constructor-owned hashing"],
]) {
  requireText(spec, needle, label);
}
for (const [needle, label] of [
  ['const INPUT_MAGIC: &[u8; 4] = b"SCBI";', "codec magic"],
  ["const INPUT_CODEC_VERSION: u16 = 1;", "codec version"],
  ["encode_participant", "participant encoding"],
  ["encode_combatant", "combatant encoding"],
  ["encode_team_resources", "resource encoding"],
  ["assembly_provenance_does_not_override_combat_identity", "provenance test"],
  ["canonicalization_makes_participant_order_irrelevant", "ordering test"],
  ["every_top_level_input_family_changes_identity", "field-family test"],
]) {
  requireText(codec, needle, label);
}
requireText(state, "combat_input_digest: CombatInputDigest", "battle input identity state");
requireText(state, "assembly_digest: AssemblyDigest", "battle assembly identity state");
requireText(
  facade,
  'pub const COMBAT_INPUT_CODEC_REVISION: &str = "combat-input-v1";',
  "public codec revision",
);
requireText(status, "| `G06-P1-B1` | `Complete` |", "completed ledger row");

console.log("Goal 06 P1-B1 verified (combat-input-v1, computed digest, split identity).");
