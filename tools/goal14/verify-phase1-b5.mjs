#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/catalog/content-catalog.json",
);

assert(evidence.schema_revision === "starclock.gold-and-gears-content-catalog.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P1-B5"
  && evidence.result === "Pass",
"Goal 14 content catalog evidence drift");
const expected = {
  GoldGearsBlessing: 162,
  GoldGearsBlessingLevel: 324,
  GoldGearsCurio: 80,
  GoldGearsCurioState: 80,
  GoldGearsOccurrence: 62,
  GoldGearsOccurrenceVariant: 65,
  GoldGearsOccurrenceChoice: 257,
  GoldGearsService: 15,
  GoldGearsAdventureOutcome: 8,
  GoldGearsEncounterGroup: 181,
  GoldGearsEncounterWave: 478,
  GoldGearsEnemySlot: 1513,
  GoldGearsMapEvent: 332,
  GoldGearsBlockCreateRule: 1091,
  GoldGearsMechanicRule: 1224,
  GoldGearsSourceRecord: 9082,
  GoldGearsCoverage: 42,
  GoldGearsResearchGap: 16,
  GoldGearsResearchGapAffectedRecord: 5025,
  GoldGearsReviewFixture: 18,
  GoldGearsPackIndex: 1,
};
assert(JSON.stringify(evidence.lowered_tables) === JSON.stringify(expected)
  && evidence.lowered_table_count === 21
  && evidence.lowered_row_count === 20056
  && Object.values(expected).reduce((sum, count) => sum + count, 0) === 20056,
"content table denominator drift");
const closure = evidence.catalog_closure;
assert(closure.shared_blessings_and_levels === "162/324"
  && closure.gold_and_shared_curios === "19/61"
  && closure.gold_curio_states === 80
  && closure.occurrences_variants_choices === "62/65/257"
  && closure.services_and_adventure_outcomes === "15/8"
  && closure.encounter_groups_waves_slots === "181/478/1513"
  && closure.referenced_enemy_variants === 90
  && closure.existing_core_or_standard_enemy_definitions === 67
  && closure.gold_enemy_definitions_owned_by_p6 === 23
  && closure.map_events_and_block_rules === "332/1091"
  && closure.mechanic_owner_and_fixture_links === 1224
  && closure.validated_private_json_payloads === 12806
  && closure.generated_public_types === 0,
"content cross-catalog closure drift");
assert(evidence.coverage.categories === 42
  && evidence.coverage.required === 7913
  && evidence.coverage.accounted === 7913
  && evidence.coverage.data_ready === 7913
  && evidence.coverage.blocking_categories === 0
  && evidence.coverage.research_gaps === 16
  && evidence.coverage.research_gap_affected_records === 5025,
"published catalog coverage drift");
assert(evidence.policy.mechanic_rules_catalog_disposition === "ReferenceOnly"
  && evidence.policy.policy_bound_rule_rows === 257
  && evidence.policy.inherited_policy_boundaries_terminalized === 0
  && evidence.policy.execution_claimed === false
  && evidence.policy.enemy_materialization_claimed === false,
"catalog lowering was mislabeled as runtime execution");

const lower = text("crates/starclock-mode-universe/src/gold_gears_content/lower.rs");
for (const table of [
  "blessing", "blessing_level", "curio", "curio_state", "occurrence",
  "occurrence_variant", "occurrence_choice", "service", "adventure_outcome",
  "encounter_group", "encounter_wave", "enemy_slot", "map_event",
  "block_create_rule", "mechanic_rule", "source_record", "coverage",
  "research_gap", "research_gap_affected_record", "review_fixture", "pack_index",
])
  assert(lower.includes(`.gold_gears_${table}()`),
    `content lowering path missing: GoldGears${table}`);
const validation = text(
  "crates/starclock-mode-universe/src/gold_gears_content/validate.rs",
);
assert(validation.includes("UniverseCatalog::load(UNIVERSE_BUNDLE, core)")
  && validation.includes("enemy_by_stable_key(row.enemy.as_str())")
  && validation.includes("GOLD_ENEMY_IDENTITIES_PENDING_P6")
  && (validation.match(/"enemy\.[^"\r\n]+"/gu) ?? []).length === 23,
"shared catalog or pending enemy identity validation drift");
for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_content/lower.rs",
  "crates/starclock-mode-universe/src/gold_gears_content/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_content/types.rs",
  "crates/starclock-mode-universe/src/gold_gears_content/validate.rs",
])
  assert(text(relative).split(/\r?\n/u).length <= 800,
    `content catalog source should be split before 800 lines: ${relative}`);
assert(text("crates/starclock-mode-universe/src/gold_gears_identity.rs")
  .includes("GoldAndGearsContentCatalog::load(bytes)"),
"public catalog identity does not validate the content catalog");

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
assert(status.includes("| `G14-P1-B5` | `Complete` |"),
  "G14-P1-B5 is incomplete");
assert(!status.includes("| Active batch | `G14-P1-B5` |")
  && !status.includes("| Next unblocked batch | `G14-P1-B5` |"),
"Goal 14 regressed to G14-P1-B5");
assert(status.includes("| Phase 1 — Bundle and catalogs | `Complete` |"),
  "Phase 1 did not exit");

console.log(
  "Goal 14 P1-B5 verified (21 content tables; 20,056 rows; 7,913/7,913 " +
  "catalog coverage; 90 enemy identities; 23 P6 materialization owners).",
);

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function captureGit(args) {
  return execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
