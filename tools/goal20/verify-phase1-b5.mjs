#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/catalog/content-catalog.json");

assert(evidence.schema_revision === "starclock.swarm-disaster-content-catalog.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P1-B5"
  && evidence.result === "Pass",
"Goal 20 content catalog evidence drift");
const expected = {
  SwarmDisasterMapEvent: 349,
  SwarmDisasterBlockCreateRule: 1212,
  SwarmDisasterTopologyConsequence: 13,
  SwarmDisasterBlessing: 144,
  SwarmDisasterBlessingLevel: 288,
  SwarmDisasterPoolMembership: 184,
  SwarmDisasterCurio: 66,
  SwarmDisasterCurioState: 66,
  SwarmDisasterCurioRule: 66,
  SwarmDisasterOccurrence: 75,
  SwarmDisasterOccurrenceVariant: 57,
  SwarmDisasterOccurrenceChoice: 308,
  SwarmDisasterService: 15,
  SwarmDisasterAdventureOutcome: 6,
  SwarmDisasterCurrency: 1,
  SwarmDisasterServiceRule: 19,
  SwarmDisasterEncounterGroup: 179,
  SwarmDisasterEncounterWave: 347,
  SwarmDisasterEnemySlot: 1070,
  SwarmDisasterBossPool: 15,
  SwarmDisasterMechanicRule: 23,
  SwarmDisasterSourceRecord: 8139,
  SwarmDisasterCoverage: 6963,
  SwarmDisasterResearchGap: 31,
  SwarmDisasterResearchGapAffected: 5560,
  SwarmDisasterReviewFixture: 23,
  SwarmDisasterReconcileReceipt: 609,
  SwarmDisasterManifest: 1,
  SwarmDisasterPackIndex: 63,
};
assert(JSON.stringify(evidence.lowered_tables) === JSON.stringify(expected),
  "content table denominator drift");
assert(Object.values(expected).reduce((sum, count) => sum + count, 0) === 25892
  && evidence.lowered_row_count === 25892
  && evidence.lowered_table_count === 29,
"content row closure drift");
assert(JSON.stringify(evidence.bundle_catalog_closure) === JSON.stringify({
  structural: "12/6716",
  unique: "24/772",
  content: "29/25892",
  total: "65/33380",
}), "65-table bundle catalog closure drift");
assert(evidence.validation.topology_event_rule_consequence_closure === "349/1212/13"
  && evidence.validation.blessing_level_pool_closure === "144/288/184"
  && evidence.validation.curio_state_rule_closure === "66/66/66"
  && evidence.validation.occurrence_variant_choice_closure === "75/57/308"
  && evidence.validation.service_adventure_currency_rule_closure === "15/6/1/19"
  && evidence.validation.encounter_group_wave_slot_boss_pool_closure === "179/347/1070/15"
  && evidence.validation.rule_source_coverage_gap_affected_fixture_receipt_manifest_pack_closure
    === "23/8139/6963/31/5560/23/609/1/63"
  && evidence.validation.generated_public_types === 0
  && evidence.validation.public_reexports_added === 0,
"content reference validation drift");
assert(evidence.policy.manifest_runtime_loading === "ForbiddenReferenceOnly"
  && evidence.policy.mechanic_rule_disposition === "ReferenceOnly"
  && evidence.policy.embedded_programs
    === "private-validated-json-text-pending-typed-execution-lowering"
  && evidence.policy.inherited_policy_boundaries_terminalized === 0
  && evidence.policy.execution_claimed === false,
"catalog lowering was mislabeled as executable parity");
assert(evidence.tests.content_unit_passed === 2
  && evidence.tests.swarm_unit_passed === 11
  && evidence.tests.identity_integration_passed === 4,
"content catalog test evidence drift");

const lowerRoot = "crates/starclock-mode-universe/src/swarm_disaster_content/lower";
const sources = [
  `${lowerRoot}/mod.rs`,
  `${lowerRoot}/topology.rs`,
  `${lowerRoot}/inventory.rs`,
  `${lowerRoot}/encounter.rs`,
  `${lowerRoot}/audit.rs`,
  "crates/starclock-mode-universe/src/swarm_disaster_content/mod.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_content/types.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_content/validate.rs",
].map((file) => [file, text(file)]);
const lowered = sources.map(([, source]) => source).join("\n");
for (const table of [
  "map_event", "block_create_rule", "topology_consequence", "blessing",
  "blessing_level", "pool_membership", "curio", "curio_state", "curio_rule",
  "occurrence", "occurrence_variant", "occurrence_choice", "service",
  "adventure_outcome", "currency", "service_rule", "encounter_group",
  "encounter_wave", "enemy_slot", "boss_pool", "mechanic_rule", "source_record",
  "coverage", "research_gap", "research_gap_affected", "review_fixture",
  "reconcile_receipt", "manifest", "pack_index",
]) assert(lowered.includes(`swarm_disaster_${table}`),
  `content lowering path missing: SwarmDisaster${table}`);
assert(!/\bf32\b|\bf64\b/u.test(lowered),
  "content catalog introduced floating authoritative arithmetic");
for (const [file, source] of sources)
  assert(source.split(/\r?\n/u).length <= 800,
    `${file} should be split before 800 lines`);
assert(text("tools/dependency-policy/verify.mjs").includes(
  '"crates/starclock-mode-universe/src/swarm_disaster_content/lower/mod.rs",'),
"private Swarm content embedded-field owner is not dependency-audited");
assert(text("crates/starclock-mode-universe/src/swarm_disaster_identity.rs")
  .includes("SwarmDisasterContentCatalog::load(bytes, &structural, &unique)"),
"catalog identity does not validate the content catalog");

const dispositions = json("content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json");
assert(dispositions.policy_boundaries.length === 31
  && dispositions.policy_boundaries.every((row) => row.current_state === "InheritedPolicy"),
"a P1 catalog batch prematurely terminalized an inherited policy boundary");
for (const protectedRoot of [
  "evidence/swarm-disaster-reference-v1",
  "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1",
  "config/swarm-disaster/data",
  "config/swarm-disaster-generated",
]) assert(captureGit(["status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot]).trim() === "",
  `protected root has worktree changes: ${protectedRoot}`);
const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P1-B5` | `Complete` |"), "G20-P1-B5 is incomplete");
assert(status.includes("| Active phase | Phase 2 — Entry, topology, Countdown and Disarray |")
  && status.includes("| Next unblocked batch | `G20-P2-B1` |"),
"Goal 20 did not advance to P2-B1");

console.log("Goal 20 P1-B5 verified (29 tables; 25,892 rows; complete 65/33,380 catalog closure; no execution claim).");

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function captureGit(args) {
  return execFileSync("git", args, { cwd: root, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 });
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
