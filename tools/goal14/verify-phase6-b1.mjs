#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/topology/encounter-selection.json",
);
assert(evidence.schema_revision
  === "starclock.gold-and-gears-encounter-selection.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P6-B1"
  && evidence.result === "Pass",
"Goal 14 P6-B1 evidence drift");

const groups = json("content-reference/gold-and-gears-v1/encounter-groups.json");
const waves = json("content-reference/gold-and-gears-v1/encounter-waves.json");
const slots = json("content-reference/gold-and-gears-v1/enemy-slots.json");
const areas = json("content-reference/gold-and-gears-v1/areas.json");
const segments = json("content-reference/gold-and-gears-v1/difficulty-segments.json");
const input = evidence.catalog_input;
const members = groups.flatMap((group) => group.weighted_members);
const roleCounts = countBy(groups, "encounter_role");
assert(input.candidate_bundle_sha256
  === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b"
  && sha256File("config/gold-and-gears-generated/config.sora")
    === input.candidate_bundle_sha256
  && groups.length === input.encounter_groups && groups.length === 181
  && members.length === input.weighted_members && members.length === 478
  && waves.length === input.encounter_waves && waves.length === 478
  && slots.length === input.enemy_slots && slots.length === 1513
  && new Set(slots.map((slot) => slot.enemy_variant_id)).size
    === input.distinct_enemy_identities
  && slots.filter((slot) => slot.boss_choice_ids.length > 0).length
    === input.boss_choice_slots
  && JSON.stringify(roleCounts) === JSON.stringify(input.role_counts),
"P6-B1 encounter denominator drift");
assert(new Set(members.flatMap((member) => member.wave_ids)).size === waves.length
  && new Set(waves.flatMap((wave) => wave.enemy_slot_ids)).size === slots.length,
"P6-B1 encounter exact-once reference closure drift");

const room = evidence.room_domain_group_policy;
assert(room.register_id === "G14-R15"
  && room.terminal_state === "VersionedExecutablePolicy"
  && room.revision === "gold-and-gears-encounter-selection-policy-v1"
  && room.accuracy === "DeterministicProjectPolicyNotObservedParity"
  && room.room_join_evidence === "Unpublished"
  && Object.keys(room.domain_mapping).length === 4
  && room.group_order === "numeric-source-group-id"
  && room.singleton_behavior === "no-draw"
  && room.unresolved_behavior === "fail-closed-before-draw"
  && nonEmpty(room.replacement_condition),
"G14-R15 is not a truthful terminal executable policy");

const difficulty = evidence.difficulty_policy;
const formalAreas = areas.filter((area) => area.area_group === "Formal");
const formalSegments = new Set(formalAreas.flatMap(
  (area) => area.difficulty_segment_ids,
));
assert(difficulty.register_id === "G14-R16"
  && difficulty.terminal_state === "VersionedExecutablePolicy"
  && difficulty.revision === "gold-and-gears-encounter-difficulty-policy-v1"
  && difficulty.accuracy === "DeterministicProjectPolicyNotObservedParity"
  && formalAreas.length === difficulty.formal_areas
  && formalSegments.size === difficulty.ordered_plane_segments
  && [...formalSegments].every((source) => segments.some(
    (segment) => segment.source_id === source
      && segment.levels.length === segment.cut_positions.length + 1,
  ))
  && difficulty.level_bucket
    === "count-cut-positions-less-than-or-equal-to-column-index"
  && difficulty.authored_stage_level_fallback === false
  && difficulty.unresolved_behavior === "fail-closed-before-draw"
  && nonEmpty(difficulty.replacement_condition),
"G14-R16 is not a truthful terminal executable policy");

const runtime = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/encounter_runtime.rs",
);
const lower = text(
  "crates/starclock-mode-universe/src/gold_gears_content/lower.rs",
);
for (const literal of [
  "ActivityRngLabel::Encounter",
  "rng.transact",
  "partition_point",
  "GoldAndGearsEncounterSelection",
  "select_current_encounter",
  "FirstPlaneBossAlternative",
  "SecondPlaneBossAlternative",
  "FinalBoss",
])
  assert(runtime.includes(literal), `missing encounter runtime contract ${literal}`);
for (const forbidden of [
  "serde_json",
  "std::fs",
  "read_to_string",
  "HashMap",
  "SystemTime",
  "thread_rng",
  "f32",
  "f64",
])
  assert(!runtime.includes(forbidden),
    `encounter runtime gained forbidden dependency ${forbidden}`);
for (const literal of [
  "parse_encounter_members",
  "validate_encounter_group_policy",
  "validate_encounter_level_policy",
  "positive_weight",
])
  assert(lower.includes(literal), `typed encounter lowering missing ${literal}`);
assert(runtime.split(/\r?\n/u).length <= 800,
  "encounter runtime should be split before 800 lines");

assert(Object.values(evidence.validation).every(Boolean)
  && evidence.runtime_boundary.mutation === "none"
  && evidence.runtime_boundary.rng_label === "Encounter"
  && evidence.runtime_boundary.rng_transactional === true
  && evidence.runtime_boundary.runtime_json_reads === 0,
"P6-B1 runtime validation drift");
const tests = evidence.tests;
assert(tests.focused_encounter_tests_passed === 6
  && tests.gold_entry_suite_passed === 122
  && tests.clippy_passed === true
  && tests.dependency_policy_passed === true
  && tests.goal_verifier_passed === true
  && tests.quick_gate_passed === true
  && Number(tests.quick_gate_seconds) > 0
  && Number.isInteger(tests.quick_selected_harnesses)
  && Number.isInteger(tests.quick_deferred_inputs)
  && tests.full_gate_required === true
  && tests.full_gate_passed === true
  && Number(tests.full_gate_seconds) > 0
  && Number.isInteger(tests.full_workspace_harnesses),
"P6-B1 test evidence drift");

for (const protectedRoot of [
  "evidence/gold-and-gears-reference-v1",
  "content-manifests/gold-and-gears-v1",
  "content-reference/gold-and-gears-v1",
  "config/gold-and-gears/data",
  "config/gold-and-gears-generated",
])
  assert(captureGit([
    "status",
    "--porcelain=v1",
    "--untracked-files=all",
    "--",
    protectedRoot,
  ]).trim() === "", `protected root has worktree changes: ${protectedRoot}`);

const status = text("docs/goals/14-gold-and-gears-runtime-status.md");
assert(status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G14-P6-B2` |")
  && status.includes("| `G14-P6-B1` | `Complete` |")
  && status.includes("| `G14-R15` | `VersionedExecutablePolicy` |")
  && status.includes("| `G14-R16` | `VersionedExecutablePolicy` |"),
"G14-P6-B1 ledger is incomplete");

console.log(
  "Goal 14 P6-B1 verified (181 groups; 478 weighted members/waves; " +
  "1,513 slots; 90 enemies; G14-R15/R16 terminal).",
);

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function sha256File(relative) {
  return crypto.createHash("sha256").update(
    fs.readFileSync(path.join(root, relative)),
  ).digest("hex");
}
function countBy(rows, field) {
  return Object.fromEntries(
    [...rows.reduce((counts, row) => {
      counts.set(row[field], (counts.get(row[field]) ?? 0) + 1);
      return counts;
    }, new Map())].toSorted(([left], [right]) => left.localeCompare(right)),
  );
}
function captureGit(args) {
  return execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
}
function nonEmpty(value) {
  return typeof value === "string" && value.length > 0;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
