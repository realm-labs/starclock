#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/swarm-disaster-reference/import-encounters.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);

const groups = json("content-reference/swarm-disaster-v1/encounter-groups.json");
const waves = json("content-reference/swarm-disaster-v1/encounter-waves.json");
const slots = json("content-reference/swarm-disaster-v1/enemy-slots.json");
const rooms = json("content-reference/swarm-disaster-v1/rooms.json");
const areas = json("content-reference/swarm-disaster-v1/areas.json");
const segments = json(
  "content-reference/swarm-disaster-v1/difficulty-segments.json",
);
const choices = json("content-reference/swarm-disaster-v1/boss-choices.json");
const variants = json("content-reference/v4.4/enemy-variants.json");

assert(groups.length === 179, "encounter-group count drift");
assert(waves.length === 347, "encounter-wave count drift");
assert(slots.length === 1070, "enemy-slot count drift");
assert(unique(groups.map(({ id }) => id)), "duplicate encounter-group ID");
assert(unique(waves.map(({ id }) => id)), "duplicate encounter-wave ID");
assert(unique(slots.map(({ id }) => id)), "duplicate enemy-slot ID");

for (const row of [...groups, ...waves, ...slots]) {
  assert(row.schema_revision === "starclock.swarm-disaster-row.v1"
    && row.coverage_state === "DataReady"
    && ["SwarmDisaster", "Shared"].includes(row.ownership),
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
const formalAreas = areas.filter(({ area_kind: kind }) => kind === "Formal");
const formalAreaIds = formalAreas.map(({ id }) => id);
const formalSegmentIds = [...new Set(
  formalAreas.flatMap(({ difficulty_segment_ids: ids }) => ids),
)].sort();
const segmentIds = new Set(segments.map(({ id }) => id));
assert(formalAreaIds.length === 5, "formal area count drift");
assert(formalSegmentIds.length === 15
  && formalSegmentIds.every((id) => segmentIds.has(id)),
"formal difficulty-segment closure drift");

const roleCounts = Object.fromEntries(Object.entries(
  Object.groupBy(groups, ({ encounter_role: role }) => role),
).map(([role, rows]) => [role, rows.length]));
for (const [role, count] of Object.entries({
  CombatPool: 103,
  ElitePool: 40,
  FinalBoss: 1,
  FirstPlaneBossAlternative: 30,
  SecondPlaneBossAlternative: 5,
}))
  assert(roleCounts[role] === count, `${role} encounter-role count drift`);

for (const group of groups) {
  assert(group.room_id === ""
    && group.room_scope.kind === "ResolvedCombatDomain"
    && group.room_scope.source_room_count === rooms.length
    && /^[0-9a-f]{64}$/u.test(group.room_scope.source_room_set_sha256)
    && group.room_scope.static_room_group_join === "Unpublished"
    && group.room_scope.unresolved_behavior === "FailClosed",
  `${group.id} room-selection boundary drift`);
  assert(group.source_namespace === "SwarmDisaster81Series"
    && JSON.stringify(group.eligible_area_ids)
      === JSON.stringify(formalAreaIds)
    && JSON.stringify(group.difficulty_binding.formal_area_ids)
      === JSON.stringify(formalAreaIds)
    && JSON.stringify(group.difficulty_binding.formal_difficulty_segment_ids)
      === JSON.stringify(formalSegmentIds)
    && group.difficulty_binding.unresolved_behavior === "FailClosed"
    && group.weight_policy.candidate_order === "source-group-member-order"
    && group.weight_policy.randomness === "seeded-activity-stream"
    && group.weight_policy.unresolved_behavior === "FailClosed",
  `${group.id} selection/difficulty policy drift`);
  const expectedWaveIds = group.weighted_members
    .flatMap(({ wave_ids: ids }) => ids);
  assert(JSON.stringify(group.wave_ids) === JSON.stringify(expectedWaveIds)
    && group.wave_ids.length > 0,
  `${group.id} wave closure drift`);
  for (const [memberIndex, member] of group.weighted_members.entries()) {
    assert(member.order === memberIndex
      && /^(0|[1-9][0-9]*)$/u.test(member.weight)
      && member.wave_ids.length > 0,
    `${group.id} member order/weight drift`);
    for (const waveId of member.wave_ids) {
      const wave = waveById.get(waveId);
      assert(wave?.encounter_group_id === group.id
        && wave.source_rogue_monster_id === member.source_rogue_monster_id
        && wave.source_stage_id === member.source_stage_id,
      `${group.id} member wave reference drift`);
    }
  }
}

for (const group of groups) {
  const groupWaves = waves.filter(({ encounter_group_id: id }) =>
    id === group.id);
  assert(JSON.stringify(groupWaves.map(({ ordinal }) => ordinal))
    === JSON.stringify(
      Array.from({ length: groupWaves.length }, (_, index) => index + 1),
    ),
  `${group.id} wave ordinals are not contiguous`);
}
for (const wave of waves) {
  assert(groupById.has(wave.encounter_group_id)
    && wave.ordinal >= 1
    && wave.source_member_ordinal >= 1
    && wave.source_wave_index >= 1
    && wave.enemy_slot_ids.length > 0
    && wave.level_binding.unresolved_area_or_plane_behavior === "FailClosed",
  `${wave.id} group/level binding drift`);
  const waveSlots = slots.filter(({ wave_id: id }) => id === wave.id);
  assert(JSON.stringify(wave.enemy_slot_ids)
    === JSON.stringify(waveSlots.map(({ id }) => id))
    && JSON.stringify(waveSlots.map(({ formation_index: index }) => index))
      === JSON.stringify(
        Array.from({ length: waveSlots.length }, (_, index) => index + 1),
      ),
  `${wave.id} enemy-slot order/reference drift`);
  for (const slotId of wave.enemy_slot_ids)
    assert(slotById.get(slotId)?.wave_id === wave.id,
      `${wave.id} enemy-slot reference drift`);
}
for (const slot of slots)
  assert(waveById.has(slot.wave_id)
    && slot.encounter_wave_id === slot.wave_id
    && variantIds.has(slot.enemy_variant_id)
    && slot.formation_index >= 1
    && typeof slot.source_monster_id === "string",
  `${slot.id} enemy identity drift`);

assert(groups.every(({ source_group_id: id }) =>
  Number(id) >= 100001 && Number(id) <= 123001),
"Swarm encounter namespace range drift");
assert(JSON.stringify(groups
  .filter(({ encounter_role: role }) => role === "FinalBoss")
  .map(({ source_group_id: id }) => id))
  === JSON.stringify(["123001"]),
"final-boss group drift");
assert(JSON.stringify(groups
  .filter(({ ownership }) => ownership === "Shared")
  .map(({ source_group_id: id }) => id))
  === JSON.stringify(["111011", "123001"]),
"shared cross-mode encounter classification drift");

const boundChoiceIds = new Set(slots.flatMap(({ boss_choice_ids: ids }) => ids));
assert(JSON.stringify([...boundChoiceIds].sort())
  === JSON.stringify(choices.map(({ id }) => id).sort()),
"boss-choice slot closure drift");
assert(new Set(slots.map(({ enemy_variant_id: id }) => id)).size === 71,
"reachable enemy-variant count drift");
assert(new Set(waves.map(({ source_stage_id: id }) => id)).size === 310,
"reachable StageConfig count drift");

console.log(
  "Swarm Disaster encounters verified (179 groups; 347 exact waves; " +
  "1,070 enemy slots; 71 released variants; both displayed boss choices).",
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
