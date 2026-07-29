#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/gold-and-gears-reference/import-encounters.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);

const groups = json("content-reference/gold-and-gears-v1/encounter-groups.json");
const waves = json("content-reference/gold-and-gears-v1/encounter-waves.json");
const slots = json("content-reference/gold-and-gears-v1/enemy-slots.json");
const rooms = json("content-reference/gold-and-gears-v1/rooms.json");
const areas = json("content-reference/gold-and-gears-v1/areas.json");
const choices = json("content-reference/gold-and-gears-v1/boss-choices.json");
const variants = json("content-reference/v4.4/enemy-variants.json");

assert(groups.length === 181, "encounter-group count drift");
assert(waves.length === 478, "encounter-wave count drift");
assert(slots.length === 1513, "enemy-slot count drift");
assert(unique(groups.map(({ id }) => id)), "duplicate encounter-group ID");
assert(unique(waves.map(({ id }) => id)), "duplicate encounter-wave ID");
assert(unique(slots.map(({ id }) => id)), "duplicate enemy-slot ID");

for (const row of [...groups, ...waves, ...slots]) {
  assert(row.schema_revision === "starclock.gold-and-gears-row.v1"
    && row.coverage_state === "DataReady"
    && ["GoldAndGears", "Shared"].includes(row.ownership),
  `${row.id} common envelope drift`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof row[field] === "string" && row[field].trim() !== "",
      `${row.id} has empty ${field}`);
  assert(JSON.stringify(row.tags) === JSON.stringify([...row.tags].sort()),
    `${row.id} tags are not canonical`);
  assert(Array.isArray(row.source_refs) && row.source_refs.length > 0,
    `${row.id} has no provenance`);
}

const groupById = new Map(groups.map((row) => [row.id, row]));
const waveById = new Map(waves.map((row) => [row.id, row]));
const slotById = new Map(slots.map((row) => [row.id, row]));
const variantIds = new Set(variants.map(({ id }) => id));
const formalAreaIds = areas.filter(({ area_group: group }) => group === "Formal")
  .map(({ id }) => id).sort();
const roleCounts = Object.fromEntries(Object.entries(
  Object.groupBy(groups, ({ encounter_role: role }) => role),
).map(([role, rows]) => [role, rows.length]));
for (const [role, count] of Object.entries({
  CombatPool: 123,
  ElitePool: 6,
  FinalBoss: 3,
  FirstPlaneBossAlternative: 35,
  GuideBoss: 2,
  SecondPlaneBossAlternative: 12,
}))
  assert(roleCounts[role] === count, `${role} encounter-role count drift`);

for (const group of groups) {
  assert(group.parent_room_id === ""
    && group.parent_room_scope.kind === "ResolvedCombatDomain"
    && group.parent_room_scope.source_room_count === rooms.length
    && /^[0-9a-f]{64}$/u.test(
      group.parent_room_scope.source_room_set_sha256,
    )
    && group.parent_room_scope.static_room_group_join === "Unpublished"
    && group.selection_policy.policy_id === "encounter-selection-v1"
    && group.selection_policy.unresolved_behavior === "FailClosed",
  `${group.id} room-selection boundary drift`);
  if (group.encounter_role === "GuideBoss")
    assert(group.ownership === "Shared"
      && group.difficulty_binding.formal_area_ids.length === 0,
    `${group.id} guide binding drift`);
  else
    assert(group.ownership === "GoldAndGears"
      && JSON.stringify(group.difficulty_binding.formal_area_ids)
        === JSON.stringify(formalAreaIds),
    `${group.id} formal difficulty binding drift`);
  for (const member of group.weighted_members) {
    assert(/^(0|[1-9][0-9]*)$/u.test(member.weight)
      && member.wave_ids.length > 0,
    `${group.id} member weight/wave drift`);
    for (const waveId of member.wave_ids) {
      const wave = waveById.get(waveId);
      assert(wave?.encounter_group_id === group.id
        && wave.source_rogue_monster_id === member.source_rogue_monster_id
        && wave.source_stage_id === member.source_stage_id,
      `${group.id} member wave reference drift`);
    }
  }
}

for (const wave of waves) {
  assert(groupById.has(wave.encounter_group_id)
    && wave.wave_index >= 1
    && wave.enemy_slot_ids.length > 0
    && wave.level_binding.unresolved_area_or_plane_behavior === "FailClosed",
  `${wave.id} group/level binding drift`);
  for (const slotId of wave.enemy_slot_ids)
    assert(slotById.get(slotId)?.encounter_wave_id === wave.id,
      `${wave.id} enemy-slot reference drift`);
}
for (const slot of slots)
  assert(waveById.has(slot.encounter_wave_id)
    && variantIds.has(slot.enemy_variant_id)
    && slot.slot_index >= 1
    && typeof slot.source_monster_id === "string",
  `${slot.id} enemy identity drift`);

const coreGroups = groups.filter(({ source_namespace: source }) =>
  source === "GoldAndGears82Series");
assert(coreGroups.length === 179
  && coreGroups.every(({ source_group_id: id }) =>
    Number(id) >= 200001 && Number(id) <= 223003),
"Gold encounter namespace drift");
assert(JSON.stringify(groups
  .filter(({ encounter_role: role }) => role === "FinalBoss")
  .map(({ source_group_id: id }) => id))
  === JSON.stringify(["223001", "223002", "223003"]),
"final-boss group drift");
assert(JSON.stringify(groups
  .filter(({ encounter_role: role }) => role === "GuideBoss")
  .map(({ source_group_id: id }) => id))
  === JSON.stringify(["111011", "123001"]),
"guide-boss group drift");

const boundChoiceIds = new Set(slots.flatMap(({ boss_choice_ids: ids }) => ids));
assert(JSON.stringify([...boundChoiceIds].sort())
  === JSON.stringify(choices.map(({ id }) => id).sort()),
"boss-choice slot closure drift");
assert(new Set(slots.map(({ enemy_variant_id: id }) => id)).size === 90,
"reachable enemy-variant count drift");
assert(new Set(waves.map(({ source_stage_id: id }) => id)).size === 375,
"reachable StageConfig count drift");

console.log(
  "Gold and Gears encounters verified (181 groups; 478 exact waves; " +
  "1,513 enemy slots; 90 released variants; all six displayed boss choices).",
);

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function unique(values) {
  return new Set(values).size === values.length;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
