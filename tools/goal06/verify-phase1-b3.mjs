#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const root = path.resolve(process.argv[2] ?? ".");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const has = (text, needle, label) => {
  if (!text.includes(needle)) throw new Error(`${label}: missing ${JSON.stringify(needle)}`);
};

const combat = read("crates/starclock-combat/src/lib.rs");
const combatState = read("crates/starclock-combat/src/codec/state.rs");
const replayV2 = read("crates/starclock-replay/src/format_v2.rs");
const replayV3 = read("crates/starclock-replay/src/format_v3.rs");
const nestedV3 = read("crates/starclock-replay/src/activity_v3.rs");
const universeV3 = read("crates/starclock-mode-universe/src/universe_replay_v3.rs");
const componentTest = read("crates/starclock-replay/tests/component_identity.rs");
const universeTest = read("crates/starclock-mode-universe/tests/battle_materialization.rs");
const configManifest = read("config/generated/debug-json/ConfigManifest.json");
const status = read("docs/goals/06-combat-identity-and-dynamic-assembly-status.md");

for (const [text, needle, label] of [
  [combat, 'STATE_HASH_REVISION: &str = "sha256-v4"', "combat state revision"],
  [combatState, "const STATE_CODEC_VERSION: u16 = 3", "combat state codec"],
  [combatState, "state.identity.combat_input_digest.bytes()", "state combat input identity"],
  [combatState, "state.identity.assembly_digest.bytes()", "state assembly identity"],
  [replayV2, "pub fn decode_replay_v2", "historical v2 decoder"],
  [replayV2, "pub fn encode_replay_v2", "historical v2 encoder"],
  [replayV3, "REPLAY_FORMAT_VERSION_V3: u32 = 3", "v3 envelope version"],
  [replayV3, "pub fn decode_replay_v3", "v3 decoder"],
  [replayV3, "pub fn encode_replay_v3", "v3 encoder"],
  [nestedV3, "component_root: ComponentRootDigest", "nested component root"],
  [nestedV3, "combat_input_codec_revision: Box<str>", "nested codec revision"],
  [nestedV3, "handoff_identity: BattleResultIdentity", "nested handoff identity"],
  [nestedV3, "result_identity: BattleResultIdentity", "nested result identity"],
  [universeV3, "pub fn verify_standard_universe_replay_v3", "v3 universe verifier"],
  [universeV3, "ReplayV3DivergenceKind::Assembly", "assembly divergence"],
  [universeV3, "ReplayV3DivergenceKind::CombatInput", "combat divergence"],
  [componentTest, "frozen_digest", "v2 frozen bytes"],
  [universeTest, "component_and_assembly_corrupt", "ordered identity corruption"],
  [universeTest, "event_and_state_corrupt", "ordered event/state corruption"],
  [configManifest, '"String": "sha256-v4"', "generated config revision"],
  [status, "| `G06-P1-B3` | `Complete` |", "completed ledger row"],
]) {
  has(text, needle, label);
}

const order = [
  "Component",
  "Assembly",
  "CombatInput",
  "Command",
  "Event",
  "State",
  "Result",
  "Activity",
];
let cursor = universeV3.indexOf("pub enum ReplayV3DivergenceKind");
for (const variant of order) {
  const next = universeV3.indexOf(`    ${variant},`, cursor);
  if (next < cursor) throw new Error(`first-divergence order: missing or out-of-order ${variant}`);
  cursor = next + variant.length;
}

console.log("Goal 06 P1-B3 verified (SCBS v3/sha256-v4, replay v3, historical v2).");
