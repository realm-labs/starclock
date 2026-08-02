#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/topology/encounter-selection.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-encounter-selection.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P6-B1" && evidence.result === "Pass",
"Goal 20 P6-B1 evidence drift");

const groups = json("content-reference/swarm-disaster-v1/encounter-groups.json");
const waves = json("content-reference/swarm-disaster-v1/encounter-waves.json");
const slots = json("content-reference/swarm-disaster-v1/enemy-slots.json");
const pools = json("content-reference/swarm-disaster-v1/boss-pools.json");
const areas = json("content-reference/swarm-disaster-v1/areas.json");
const segments = json("content-reference/swarm-disaster-v1/difficulty-segments.json");
const dispositions = json("content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json");
const input = evidence.catalog_input;
const members = groups.flatMap((group) => group.weighted_members);
assert(sha256File("config/swarm-disaster-generated/config.sora") === input.candidate_bundle_sha256
  && input.candidate_bundle_sha256 === "385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362"
  && groups.length === input.encounter_groups && groups.length === 179
  && members.length === input.weighted_members && members.length === 347
  && waves.length === input.encounter_waves && waves.length === 347
  && slots.length === input.enemy_slots && slots.length === 1070
  && pools.length === input.boss_pools && pools.length === 15
  && new Set(slots.map((slot) => slot.enemy_variant_id)).size === input.distinct_enemy_identities
  && slots.filter((slot) => slot.boss_choice_ids.length > 0).length === input.boss_choice_slots
  && equal(countBy(groups, "encounter_role"), input.role_counts),
"P6-B1 encounter denominator drift");
assert(new Set(members.flatMap((member) => member.wave_ids)).size === waves.length
  && new Set(waves.flatMap((wave) => wave.enemy_slot_ids)).size === slots.length
  && groups.every((group) => group.room_id === ""),
"P6-B1 encounter exact-once or room boundary drift");

const assigned = dispositions.source_obligations.filter((row) => row.execution_batch === "G20-P6-B1");
assert(assigned.length === 20
  && assigned.every((row) => row.category === "difficulty_segments")
  && evidence.source_execution.assigned_obligations === 20
  && evidence.source_execution.executed_obligations === 20
  && evidence.source_execution.exact_once === true,
"P6-B1 assigned source execution drift");
const formalAreas = areas.filter((area) => area.area_kind === "Formal");
const formalSegments = new Set(formalAreas.flatMap((area) => area.difficulty_segment_ids));
assert(segments.length === 20 && formalAreas.length === 5 && formalSegments.size === 15
  && segments.every((segment) => segment.level_list.length === segment.cut_list.length + 1
    && segment.cut_list.every((cut, index) => index === 0 || segment.cut_list[index - 1] < cut)),
"P6-B1 difficulty segment closure drift");

const rooms = evidence.room_domain_group_policy;
assert(rooms.policy_source_id === "source.goal09.project-policy.rooms"
  && rooms.terminal_state === "VersionedExecutablePolicy"
  && rooms.released_room_rows === 861 && rooms.published_static_room_group_joins === 0
  && rooms.numeric_room_id_inference === false && rooms.resolved_domain_is_required === true
  && Object.keys(rooms.domain_mapping).length === 5
  && rooms.group_order === "numeric-source-group-id"
  && rooms.singleton_behavior === "no-draw"
  && rooms.unresolved_behavior === "fail-closed-before-draw"
  && nonEmpty(rooms.replacement_condition),
"rooms policy is not a truthful terminal executable boundary");
for (const policy of [evidence.encounter_selection_policy, evidence.difficulty_policy])
  assert(policy.current_state === "InheritedPolicy" && policy.remaining_owner === "G20-P6-B3"
    && nonEmpty(policy.replacement_condition), "multi-owner encounter policy closed too early");
assert(evidence.difficulty_policy.catalog_segments === 20
  && evidence.difficulty_policy.ordered_formal_plane_segments === 15
  && evidence.difficulty_policy.level_bucket
    === "count-cut-positions-less-than-or-equal-to-column-index"
  && evidence.difficulty_policy.authored_stage_level_fallback === false,
"P6-B1 executable difficulty policy drift");

const runtime = text("crates/starclock-mode-universe/src/swarm_disaster_entry/encounter_runtime.rs");
const access = text("crates/starclock-mode-universe/src/swarm_disaster_content/encounter_access.rs");
for (const literal of ["ActivityRngLabel::Encounter", "rng.transact", "partition_point",
  "select_current_encounter_digest", "FirstPlaneBossAlternative",
  "SecondPlaneBossAlternative", "FinalBoss", "selection_digest"])
  assert(runtime.includes(literal), `missing encounter runtime contract ${literal}`);
for (const forbidden of ["serde_json", "std::fs", "read_to_string", "HashMap", "SystemTime",
  "thread_rng", "f32", "f64"])
  assert(!runtime.includes(forbidden), `encounter runtime gained forbidden dependency ${forbidden}`);
for (const literal of ["EncounterRuntimeInput", "GroupDifficultyPolicy", "GroupWeightPolicy",
  "WaveLevelPolicy", "BossSelectionPolicy", "serde_json::from_str"])
  assert(access.includes(literal), `typed encounter access missing ${literal}`);
assert(!runtime.includes("pub struct ") && !runtime.includes("pub use ")
  && physicalLineCount(runtime) <= 800 && physicalLineCount(access) <= 400,
"P6-B1 visibility or source boundary drift");

assert(Object.values(evidence.validation).every(Boolean)
  && evidence.runtime_boundary.mutation === "none"
  && evidence.runtime_boundary.rng_label === "Encounter"
  && evidence.runtime_boundary.rng_transactional === true
  && evidence.runtime_boundary.runtime_json_file_reads === 0
  && evidence.runtime_boundary.battle_spec_materialization === "DeferredToG20P6B2",
"P6-B1 runtime validation drift");
const api = evidence.api_and_policy;
assert(api.new_public_mode_types === 0 && api.public_runtime_methods_added === 1
  && api.public_reexports_added === 0 && api.source_policy_handwritten_files === 938
  && api.source_policy_public_reexports === 72
  && api.second_activity_or_battle_state_machine_added === false,
"P6-B1 API or source-policy drift");
const tests = evidence.tests;
assert(tests.focused_encounter_tests_passed === 6 && tests.entry_suite_passed === 128
  && tests.swarm_suite_passed === 139 && tests.identity_integration_passed === 5
  && tests.clippy_passed === true && tests.dependency_policy_passed === true
  && tests.source_policy_passed === true && tests.quick_gate_passed === true
  && Number(tests.quick_gate_seconds) > 0 && tests.quick_selected_harnesses === 3
  && tests.quick_deferred_inputs === 1 && tests.quick_rust_receipt === "CacheMiss"
  && Number(tests.cache_hit_quick_gate_seconds) > 0 && tests.cache_hit_quick_deferred_inputs === 3
  && tests.cache_hit_quick_rust_receipt === "CacheHit"
  && Number(tests.final_quick_gate_seconds) > 0 && tests.final_quick_deferred_inputs === 3
  && tests.final_quick_rust_receipt === "CacheMiss"
  && tests.full_gate_required === true && tests.full_gate_passed === true
  && Number(tests.full_gate_seconds) > 0 && tests.full_generated_checks === 33
  && tests.full_source_cache_skips === 4 && tests.full_workspace_harnesses === 34
  && Number(tests.final_full_gate_seconds) > 0
  && Number(tests.final_full_workspace_seconds) > 0,
"P6-B1 test evidence drift");

for (const protectedRoot of ["evidence/swarm-disaster-reference-v1",
  "content-manifests/swarm-disaster-v1", "content-reference/swarm-disaster-v1",
  "config/swarm-disaster/data", "config/swarm-disaster-generated"])
  assert(captureGit(["status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot]).trim() === "",
    `protected root changed: ${protectedRoot}`);
const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P6-B2` |")
  && status.includes("| Phase 6 — Encounters and full-run integration | `InProgress` |")
  && status.includes("| `G20-P6-B1` | `Complete` |"),
"G20-P6-B1 ledger is incomplete");

console.log("Goal 20 P6-B1 verified (179 groups; 347 weighted members/waves; 1,070 slots; 15 boss pools; 20 difficulty segments). ");

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256File(relative) { return crypto.createHash("sha256").update(
  fs.readFileSync(path.join(root, relative))).digest("hex"); }
function countBy(rows, field) { return Object.fromEntries([...rows.reduce((counts, row) => {
  counts.set(row[field], (counts.get(row[field]) ?? 0) + 1); return counts;
}, new Map())].toSorted(([left], [right]) => left.localeCompare(right))); }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function captureGit(args) { return execFileSync("git", args, { cwd: root, encoding: "utf8" }); }
function physicalLineCount(contents) { const lines = contents.split(/\r?\n/u);
  return lines.at(-1) === "" ? lines.length - 1 : lines.length; }
function nonEmpty(value) { return typeof value === "string" && value.length > 0; }
function assert(condition, message) { if (!condition) throw new Error(message); }
