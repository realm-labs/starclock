#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const root = path.resolve(process.argv[2] ?? ".");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const has = (text, needle, label) => {
  if (!text.includes(needle)) throw new Error(`${label}: missing ${JSON.stringify(needle)}`);
};

const spec = read("crates/starclock-combat/src/battle/spec.rs");
const cause = read("crates/starclock-combat/src/event/cause.rs");
const event = read("crates/starclock-replay/src/battle_event.rs");
const eventCause = read("crates/starclock-replay/src/battle_event_cause.rs");
const nested = read("crates/starclock-replay/src/nested_battle.rs");
const replayV2 = read("crates/starclock-mode-universe/src/universe_replay_v2.rs");
const replayV3 = read("crates/starclock-mode-universe/src/universe_replay_v3.rs");
const executor = read("crates/starclock-mode-universe/src/nested_battle_executor.rs");
const materializer = read("crates/starclock-mode-universe/src/battle_materialization.rs");
const materializerSpec = read(
  "crates/starclock-mode-universe/src/battle_materialization/battle_spec.rs",
);
const status = read("docs/goals/06-combat-identity-and-dynamic-assembly-status.md");

for (const [text, needle, label] of [
  [spec, "assembly_digest: AssemblyDigest,", "constructor assembly input"],
  [spec, "combat_input_digest = super::spec_codec::combat_input_digest(", "computed combat input"],
  [event, "BATTLE_EVENT_PAYLOAD_VERSION_V1: u16 = 1", "historical event codec"],
  [event, "BATTLE_EVENT_PAYLOAD_VERSION_V2: u16 = 2", "Goal 06 event codec"],
  [eventCause, "version == BATTLE_EVENT_PAYLOAD_VERSION_V1", "v1 reserved field"],
  [nested, "encode_nested_battle_state_payload_v1", "historical nested state encoder"],
  [replayV2, "encode_nested_battle_state_payload_v1", "replay v2 event codec"],
  [replayV3, "encode_nested_battle_state_payload(step.state_hash()", "replay v3 event codec"],
  [executor, "deterministic-battle-input-event-shape-v1", "event commitment compatibility"],
  [executor, "handoff.identity().assembly_digest().bytes()", "commitment assembly identity"],
  [executor, "optional_u32(&mut self.0, None)", "commitment reserved byte"],
  [materializer, "use battle_spec::{difficulty_spec, member_spec};", "materializer split"],
  [materializerSpec, "pub(super) fn member_spec", "member request construction"],
  [materializerSpec, "pub(super) fn difficulty_spec", "difficulty request construction"],
  [status, "| `G06-P1-B4` | `Complete` |", "completed ledger row"],
]) has(text, needle, label);

const currentEventVersion = Number(
  event.match(/BATTLE_EVENT_PAYLOAD_VERSION: u16 = (\d+)/)?.[1],
);
if (!Number.isInteger(currentEventVersion) || currentEventVersion < 2)
  throw new Error("current event codec no longer preserves the Goal 06 v2 boundary");

if (spec.includes("digest: BattleSpecDigest,"))
  throw new Error("BattleSpec constructor still accepts BattleSpecDigest");
if (spec.includes("pub fn new_with_assembly"))
  throw new Error("temporary new_with_assembly constructor remains");
if (cause.includes("activity_source"))
  throw new Error("Activity vocabulary remains in combat Cause");

for (const relative of [
  "crates/starclock-replay/src/battle_event.rs",
  "crates/starclock-mode-universe/src/battle_materialization.rs",
]) {
  const lines = read(relative).split(/\r?\n/).length;
  if (lines > 1_200) throw new Error(`${relative} exceeds the current 1200-line policy at ${lines}`);
}

const rustFiles = [];
const visit = (directory) => {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const absolute = path.join(directory, entry.name);
    if (entry.isDirectory()) visit(absolute);
    else if (entry.name.endsWith(".rs")) rustFiles.push(absolute);
  }
};
visit(path.join(root, "crates"));
for (const file of rustFiles) {
  const source = fs.readFileSync(file, "utf8");
  for (let cursor = source.indexOf("BattleSpec::new(");
    cursor >= 0;
    cursor = source.indexOf("BattleSpec::new(", cursor + 1)) {
    const callPrefix = source.slice(cursor, cursor + 260);
    if (callPrefix.includes("BattleSpecDigest::new"))
      throw new Error(`caller-supplied legacy digest remains in ${path.relative(root, file)}`);
  }
}

console.log(
  `Goal 06 P1-B4 verified (AssemblyDigest construction, event payload v2 preserved; ` +
  `current v${currentEventVersion}, split cores).`,
);
