#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/runtime/entry-profile.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-entry-profile-evidence.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P2-B1"
  && evidence.result === "Pass", "P2-B1 evidence identity drift");
assert(JSON.stringify(evidence.entry_obligations) === JSON.stringify({
  count: 12,
  entry_points: 3,
  formal_difficulties: 5,
  guide_areas_rejected: 3,
  runtime_profiles: 1,
}), "entry obligation denominator drift");
assert(evidence.compiled_matrix.formal_difficulties === 5
  && evidence.compiled_matrix.path_audience_die_pairs === 8
  && evidence.compiled_matrix.valid_combinations === 40
  && evidence.compiled_matrix.rng_draws === 0,
"entry matrix closure drift");
assert(JSON.stringify(evidence.participant_lock) === JSON.stringify({
  minimum_teams: 1,
  maximum_teams: 1,
  maximum_participants: 4,
  uniqueness_scope: "Activity",
  loadout_lock_scope: "Activity",
}), "participant lock contract drift");
assert(evidence.activity_profile.slot_families === 16
  && evidence.activity_profile.activity_scope_slots === 9
  && evidence.activity_profile.section_scope_slots === 4
  && evidence.activity_profile.node_scope_slots === 1
  && evidence.activity_profile.attempt_scope_slots === 2
  && evidence.activity_profile.initial_countdown === 20
  && evidence.activity_profile.initial_cosmic_fragments === 50
  && evidence.activity_profile.authoritative_float_fields === 0,
"Activity profile state contract drift");
assert(evidence.progression_input.communing_dimensions === 7
  && evidence.progression_input.maximum_enforced_per_dimension
  && evidence.progression_input.trail_cabinet_interplay_keys_validated
  && evidence.progression_input.trailblaze_bonus_keys_validated
  && evidence.progression_input.duplicate_keys_rejected,
"entry progression contract drift");
assert(JSON.stringify(evidence.boundary.public_types_implemented) === JSON.stringify([
  "SwarmDisasterEntry", "SwarmDisasterRuntimeFactory", "SwarmDisasterRuntimeInstance",
]) && evidence.boundary.controller_type_deferred_to_controller_batch
  === "SwarmDisasterControllerIdentity"
  && evidence.boundary.generated_public_types === 0
  && evidence.boundary.public_reexports_added === 0
  && evidence.boundary.profile_entry_rule_terminalized === false
  && evidence.boundary.profile_entry_fixture_claimed_executed === false,
"entry public/execution boundary drift");
assert(evidence.tests.entry_unit_passed === 3
  && evidence.tests.swarm_unit_passed === 14
  && evidence.tests.identity_integration_passed === 5,
"entry test evidence drift");

const entry = text("crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs");
const state = text("crates/starclock-mode-universe/src/swarm_disaster_entry/state.rs");
const tests = text("crates/starclock-mode-universe/src/swarm_disaster_entry/tests.rs");
const validation = text("crates/starclock-mode-universe/src/swarm_disaster_entry/validate.rs");
const entryRuntime = `${entry}\n${validation}`;
for (const publicType of evidence.boundary.public_types_implemented)
  assert(entry.includes(`pub struct ${publicType}`), `missing public type ${publicType}`);
assert(entryRuntime.includes("ParticipantUniquenessScope::Activity")
  && entryRuntime.includes("LoadoutLockScope::Activity")
  && entryRuntime.includes("ParticipantPolicy::new("), "participant lock is not compiled");
assert(entryRuntime.includes("entry_selection(&entry.path, &entry.audience_die)")
  && entryRuntime.includes("canonical_communing")
  && entryRuntime.includes("canonical_progression"), "entry validation path is incomplete");
assert(tests.includes("for difficulty in 1_u8..=5")
  && tests.includes("let pairs = [")
  && tests.includes("assert_eq!(instance.state_definition().slots().len(), 16)"),
"five-difficulty/eight-Path executable matrix missing");
assert((state.match(/const [A-Z_]+: u32 = 0x5344_/gu) ?? []).length === 16,
  "typed slot-family implementation count drift");
assert(!/\bf32\b|\bf64\b/u.test(`${entry}\n${state}`),
  "entry compilation introduced authoritative floats");
assert(!/ActivityRng|thread_rng|shuffle|rand::/u.test(`${entry}\n${state}`),
  "entry compilation consumes randomness");
for (const source of [entry, state, tests, validation])
  assert(source.split(/\r?\n/u).length <= 800, "entry source exceeds split threshold");

const dispositions = json("content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json");
const entryObligations = dispositions.source_obligations.filter((row) =>
  row.execution_batch === "G20-P2-B1");
assert(entryObligations.length === 12
  && entryObligations.every((row) => row.target_runtime_disposition === "Integrated"),
"frozen P2-B1 obligation assignment drift");
const profileRule = dispositions.mechanic_rules.find((row) =>
  row.id === "swarm-disaster.mechanic-rule.profile-entry");
assert(profileRule?.implementation_batch === "G20-P5-M01"
  && profileRule.current_state === "Pending",
"P2-B1 prematurely claimed the profile-entry rule");
for (const protectedRoot of [
  "evidence/swarm-disaster-reference-v1",
  "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1",
  "config/swarm-disaster/data",
  "config/swarm-disaster-generated",
]) assert(captureGit(["status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot]).trim() === "",
  `protected root has worktree changes: ${protectedRoot}`);
const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P2-B1` | `Complete` |"), "G20-P2-B1 is incomplete");
assert(status.includes("| Next unblocked batch | `G20-P2-B2` |"),
  "Goal 20 did not advance to P2-B2");

console.log("Goal 20 P2-B1 verified (12 obligations; 5 difficulties; 8 Path/Die pairs; 40 entries; 16 slots; zero RNG draws).");

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function captureGit(args) {
  return execFileSync("git", args, { cwd: root, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 });
}
function assert(condition, message) { if (!condition) throw new Error(message); }
