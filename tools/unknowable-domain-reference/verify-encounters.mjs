#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/unknowable-domain-reference/import-encounters.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);

const choices = json("content-reference/unknowable-domain-v1/boss-choices.json");
const pools = json("content-reference/unknowable-domain-v1/boss-pools.json");
const obligations = json(
  "content-reference/unknowable-domain-v1/encounter-source-obligations.json",
);
const groups = json(
  "content-reference/unknowable-domain-v1/encounter-groups.json",
);
const waves = json(
  "content-reference/unknowable-domain-v1/encounter-waves.json",
);
const slots = json("content-reference/unknowable-domain-v1/enemy-slots.json");
assert(choices.length === 6, "displayed boss denominator drift");
assert(pools.length === 13, "area boss-pool denominator drift");
assert(obligations.length === 1524, "encounter source-parent denominator drift");
assert(groups.length === 0 && waves.length === 0 && slots.length === 0,
  "unproven encounter group, wave or slot was imported");

for (const [kind, rows] of [
  ["UnknowableBossChoice", choices],
  ["UnknowableBossPool", pools],
  ["EncounterSourceObligation", obligations],
]) {
  assert(unique(rows.map(({ id }) => id)), `${kind} duplicate stable ID`);
  assert(rows.every((row) =>
    row.kind === kind
      && row.schema_revision === "starclock.unknowable-domain-row.v1"
      && row.coverage_state === "DataReady"
      && row.name_en
      && row.name_zh_cn
      && row.summary_en
      && row.summary_zh_cn
      && row.source_refs.length >= 1
      && row.source_refs.every((source) =>
        source.game_version === "4.4"
          && /^[0-9a-f]{64}$/u.test(source.sha256)),
  ), `${kind} envelope/provenance drift`);
}

const manifest = json(
  "content-manifests/unknowable-domain-v1/content-manifest.json",
);
assert(exactOnce(
  choices.map(({ source_id: id }) => id),
  manifest.categories.boss_choices.records.map(({ id }) => id),
), "boss-choice manifest closure drift");
assert(exactOnce(
  obligations.map(({ source_id: id }) => id),
  manifest.categories.encounter_source_obligations.records.map(({ id }) => id),
), "encounter source-parent manifest closure drift");

const expectedBossIds = [
  "2004011",
  "2034012",
  "3003042",
  "3024023",
  "8024010",
  "8024012",
];
assert(exactOnce(choices.map(({ source_id: id }) => id), expectedBossIds),
  "displayed boss identity drift");
const variants = json("content-reference/v4.4/enemy-variants.json");
const variantBySource = new Map(variants.map((row) =>
  [row.source_monster_id, row]));
assert(choices.every((choice) => {
  const variant = variantBySource.get(choice.source_id);
  return variant
    && choice.enemy_id === variant.enemy_id
    && choice.enemy_variant_id === variant.id
    && choice.pool_id.length >= 1
    && choice.display_level_bindings.length >= 7
    && choice.stage_binding_state === "UnresolvedNoReleasedSelector"
    && choice.reverse_match_audit.accepted_as_reachability === false
    && choice.runtime_lowered === false;
}), "boss shared-enemy closure or stage boundary drift");
assert(choices.reduce((sum, choice) =>
  sum + choice.reverse_match_audit.matching_stage_count, 0) === 53,
"reverse StageConfig audit count drift");
assert(choices.reduce((sum, choice) =>
  sum + choice.reverse_match_audit.matching_rogue_monster_count, 0) === 26,
"reverse RogueMonster audit count drift");
assert(choices.reduce((sum, choice) =>
  sum + choice.reverse_match_audit.matching_group_count, 0) === 34,
"reverse RogueMonsterGroup audit count drift");

assert(pools.every((pool) =>
  pool.area_id === `unknowable-domain.area.${pool.source_id}`
    && pool.difficulty_id.length >= 1
    && pool.candidate_ids.length === 1
    && pool.ordering === "SourceDisplayOrderThenStableEnemyId"
    && pool.fallback === "FailClosedWithoutStageSelector"
    && pool.stage_binding_state === "UnresolvedNoReleasedSelector"
    && pool.runtime_lowered === false),
"area boss-pool binding drift");
assert(exactOnce(
  choices.flatMap(({ pool_id: ids }) => ids),
  [...new Set(pools.map(({ id }) => id))],
), "boss choice to area-pool closure drift");

const roomParents = obligations.filter(({ parent_kind: kind }) =>
  kind === "Room");
const bossParents = obligations.filter(({ parent_kind: kind }) =>
  kind === "DisplayedBossIdentity");
assert(roomParents.length === 1518 && bossParents.length === 6,
  "room/boss parent split drift");
assert(exactOnce(
  bossParents.flatMap(({ enemy_variant_ids: ids }) => ids),
  choices.map(({ enemy_variant_id: id }) => id),
), "boss parent enemy-variant closure drift");
assert(bossParents.every((row) =>
  row.expansion_state === "EnemyIdentityResolvedStageUnresolved"
    && row.encounter_group_ids.length === 0
    && row.stage_ids.length === 0
    && row.blocking === false
    && row.runtime_lowered === false),
"boss source-parent boundary drift");

const roomStateCounts = countBy(roomParents, ({ expansion_state: state }) =>
  state);
assert(JSON.stringify(roomStateCounts) === JSON.stringify({
  NoCombatWaveExpansion: 686,
  UnresolvedNoReleasedSelector: 832,
}), "room expansion-state split drift");
const roomTypeCounts = countBy(roomParents, ({ room_type: type }) => type);
assert(JSON.stringify(roomTypeCounts) === JSON.stringify({
  Adventure: 9,
  Battle: 488,
  Boss: 54,
  Elite: 47,
  Encounter: 243,
  Event: 245,
  Reforge: 36,
  Reward: 243,
  Shop: 31,
  Wealth: 122,
}), "room type split drift");
assert(roomParents.every((row) =>
  row.encounter_group_ids.length === 0
    && row.stage_ids.length === 0
    && row.blocking === false
    && row.replacement_condition.includes("released structured data")
    && row.runtime_lowered === false),
"room parent fail-closed boundary drift");

const sourceRoot = path.join(
  root,
  ".cache/content-reference/turnbasedgamedata",
);
const sourceRooms = sourceJson("ExcelOutput/RogueMagicRoom.json");
assert(sourceRooms.length === 1518
  && sourceRooms.every((row) =>
    exactOnce(Object.keys(row), ["RogueRoomID", "RogueRoomType"])),
"RogueMagicRoom published field boundary drift");
const stageRows = sourceJson("ExcelOutput/StageConfig.json");
assert(stageRows.filter((row) =>
  /RogueMagic|Rogue260|MagicRogue/iu.test(JSON.stringify(row))).length === 0,
"a direct Unknowable Domain StageConfig selector appeared; replace boundary");
const groupProgram = sourceJson(
  "Config/Level/Maze/MazeRogue/Rogue260/RogueMagic_Group_Monster.json",
);
assert(!/StageID|MonsterID|RogueMonsterGroupID/iu.test(
  JSON.stringify(groupProgram),
), "mode group program gained an explicit encounter selector");

const boundary = fs.readFileSync(path.join(
  root,
  "evidence/unknowable-domain-reference-v1/encounter-boundary.md",
), "utf8");
for (const phrase of [
  "1,524 source parents",
  "six shared enemy identities",
  "13 area display pools",
  "53 StageConfig rows",
  "26 RogueMonster rows",
  "34 RogueMonsterGroup rows",
  "zero accepted encounter groups",
  "`UnresolvedNoReleasedSelector`",
  "ID range",
])
  assert(boundary.includes(phrase), `encounter boundary omits ${phrase}`);

console.log(
  "Unknowable Domain encounters verified (1,524 source parents; 6 exact " +
  "shared enemy identities; 13 area display pools; 0 unproven groups/waves/" +
  "slots; room-to-StageConfig selection fails closed).",
);

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}

function sourceJson(relative) {
  return JSON.parse(fs.readFileSync(path.join(sourceRoot, relative), "utf8"));
}

function countBy(rows, key) {
  return Object.fromEntries([...Map.groupBy(rows, key).entries()]
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([value, entries]) => [value, entries.length]));
}

function unique(values) {
  return new Set(values).size === values.length;
}

function exactOnce(left, right) {
  const ordered = (values) => [...values].sort();
  return JSON.stringify(ordered(left)) === JSON.stringify(ordered(right));
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
