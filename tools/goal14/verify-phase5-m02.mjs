#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/mechanics/conundrum-stats-rules.json",
);

assert(evidence.schema_revision
  === "starclock.gold-and-gears-conundrum-stats-rule-execution.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P5-M02"
  && evidence.result === "Pass",
"Goal 14 P5-M02 evidence drift");

const partition = evidence.frozen_partition;
assert(partition.path
  === "content-manifests/gold-and-gears-runtime-v1/rule-partitions.json"
  && partition.sha256
    === "ddcf8eef2ace4984c34a819846d0f25d28ee475a399b6d08de0dc26de85f62c4"
  && partition.family === "conundrum-stats"
  && partition.expected_rules === 6
  && partition.gold_executor === "CombatModifier"
  && partition.exact_public_rules === 0
  && partition.project_policy_rules === 6
  && partition.shared_rules === 0
  && partition.native_handlers === 0
  && partition.fixture_id === "gold-gears.fixture.conundrum-stats",
"P5-M02 frozen partition drift");

const runtime = evidence.runtime;
assert(runtime.revision
  === "gold-and-gears-stats-conundrum-combat-modifier-v1"
  && runtime.policy_revision === "gold-and-gears-conundrum-numeric-policy-v1"
  && runtime.policy_accuracy === "DeterministicProjectPolicyNotObservedParity"
  && runtime.combat_boundary === "ModifierDefinition"
  && runtime.registry === "ModifierRegistry"
  && runtime.executor === "StatResolver"
  && runtime.source_class === "Mode"
  && runtime.composition
    === "LatestContributionPerSourceTagAtOrBelowSelectedLevel"
  && runtime.generic_stats_appended.join(",")
    === "MaximumToughness,ReceivedAttackActionAdvance"
  && runtime.runtime_json_file_reads === 0,
"P5-M02 runtime contract drift");

assert(evidence.rules.length === 6
  && evidence.rules.every((rule) =>
    rule.terminal_disposition === "ProductionExecuted"
      && rule.accuracy === "VersionedProjectPolicy"),
"P5-M02 terminal rule count drift");
assert(evidence.rules.map((rule) => rule.rule_id).join(",") === [
  "gold-gears.rule.conundrum.stats.1",
  "gold-gears.rule.conundrum.stats.2",
  "gold-gears.rule.conundrum.stats.3",
  "gold-gears.rule.conundrum.stats.4",
  "gold-gears.rule.conundrum.stats.5",
  "gold-gears.rule.conundrum.stats.6",
].join(","),
"P5-M02 stable rule IDs drift");
assert(evidence.rules.map((rule) => rule.owner_id).join(",") === [
  "gold-gears.conundrum-level.stats.1",
  "gold-gears.conundrum-level.stats.2",
  "gold-gears.conundrum-level.stats.3",
  "gold-gears.conundrum-level.stats.4",
  "gold-gears.conundrum-level.stats.5",
  "gold-gears.conundrum-level.stats.6",
].join(","),
"P5-M02 owner IDs drift");
assert(evidence.rules[0].ratios_scaled.join(",") === "100000,100000,25000"
  && evidence.rules[1].ratios_scaled.join(",") === "200000,200000,50000"
  && evidence.rules[2].ratios_scaled.join(",") === "150000,75000"
  && evidence.rules[2].trigger_cycle === 6
  && evidence.rules[2].stack_interval_cycles === 1
  && evidence.rules[2].stack_cap === 5
  && evidence.rules[3].ratios_scaled.join(",") === "300000,300000,75000"
  && evidence.rules[4].ratios_scaled.join(",") === "100000,100000"
  && evidence.rules[5].ratios_scaled.join(",") === "400000,400000,100000",
"P5-M02 versioned policy values drift");

const fixture = evidence.production_fixture_probe;
assert(fixture.fixture_id === "gold-gears.fixture.conundrum-stats"
  && fixture.selected_stats_level === 6
  && fixture.active_rule_ids.join(",") === [
    "gold-gears.rule.conundrum.stats.3",
    "gold-gears.rule.conundrum.stats.5",
    "gold-gears.rule.conundrum.stats.6",
  ].join(",")
  && fixture.enemy_rank === "EliteOrBoss"
  && fixture.berserk_stacks === 2
  && fixture.received_attack === true
  && fixture.modifier_count === 7
  && Object.values(fixture.resolved_scaled).join(",")
    === "14000000,3400000,1250000,3300000,100000"
  && fixture.modifier_set_digest
    === "2cc3f09860b86ab4706ae0361299566e0ec741673138ba2e957d5d4daf5e5ded"
  && fixture.aggregate_fixture_execution_owner === "G14-P5-B1",
"P5-M02 production fixture drift");

assert(evidence.tier_fixture_probe.levels
  .map((entry) => `${entry.level}:${entry.resolved_scaled.join("/")}`)
  .join(",") === [
    "1:11000000/2200000/1025000",
    "2:12000000/2400000/1050000",
    "4:13000000/2600000/1075000",
    "6:14000000/2800000/1100000",
  ].join(","),
"P5-M02 tier fixture drift");
assert(Object.values(evidence.validation).every(Boolean),
"P5-M02 validation evidence drift");

const tests = evidence.tests;
assert(tests.focused_stats_modifier_tests_passed === 4
  && tests.entry_suite_passed === 84
  && tests.clippy_passed === true
  && tests.dependency_policy_passed === true
  && tests.workspace_check_passed === true
  && tests.quick_gate_passed === true
  && tests.quick_gate_seconds !== null
  && tests.quick_selected_harnesses !== null
  && tests.quick_direct_packages === 2
  && tests.quick_downstream_packages_checked !== null
  && tests.quick_rust_receipt !== null
  && tests.final_quick_gate_seconds !== null
  && tests.final_quick_rust_receipt === "CacheHit"
  && tests.final_quick_deferred_inputs !== null
  && tests.full_gate_required === true
  && tests.full_gate_passed === true
  && tests.full_gate_seconds !== null
  && tests.full_workspace_harnesses !== null
  && tests.full_cache_dependent_checks_skipped !== null,
"P5-M02 test evidence drift");

const manifest = json(partition.path);
const frozen = manifest.partitions.find((candidate) => candidate.id === "G14-P5-M02");
assert(frozen !== undefined
  && frozen.family_id === "conundrum-stats"
  && frozen.expected_rules === 6
  && frozen.rule_ids.join(",")
    === evidence.rules.map((rule) => rule.rule_id).join(",")
  && frozen.gold_executor === "CombatModifier"
  && frozen.fixture_ids.join(",") === "gold-gears.fixture.conundrum-stats",
"P5-M02 no longer matches its frozen assignment");

const runtimeSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/conundrum_stats_modifier.rs",
);
for (const literal of [
  "gold-and-gears-stats-conundrum-combat-modifier-v1",
  "compile_stats_conundrum_modifiers",
  "ModifierDefinition",
  "ModifierStackingGroup",
  "RuleSource",
  "BERSERK_STACK_SLOT",
  "MaximumToughness",
  "ReceivedAttackActionAdvance",
  "EliteOrBossAfterReceivedAttackWhileBerserk",
])
  assert(runtimeSource.includes(literal),
    `missing P5-M02 runtime contract ${literal}`);
for (const forbidden of [
  "std::fs",
  "read_to_string",
  "SystemTime",
  "HashMap",
  "f32",
  "f64",
])
  assert(!runtimeSource.includes(forbidden),
    `P5-M02 runtime gained forbidden dependency ${forbidden}`);

const combatModel = text("crates/starclock-combat/src/modifier/model.rs");
assert(combatModel.indexOf("MaximumToughness")
  > combatModel.indexOf("ToughnessRecovery")
  && combatModel.indexOf("ReceivedAttackActionAdvance")
    > combatModel.indexOf("MaximumToughness"),
"P5-M02 generic stat append-only contract drift");

const regression = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/conundrum_stats_modifier_tests.rs",
);
for (const literal of [
  "stats_partition_binds_exactly_six_versioned_policy_rules",
  "stats_fixture_executes_all_active_modifiers_through_combat_resolver",
  "every_enemy_stat_tier_executes_its_exact_percent_of_base_values",
  "rank_berserk_and_received_attack_activation_is_fail_closed",
])
  assert(regression.includes(literal), `missing P5-M02 regression ${literal}`);

for (const relative of [
  "crates/starclock-combat/src/modifier/model.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/error.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/conundrum_stats_modifier.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/conundrum_stats_modifier_tests.rs",
])
  assert(physicalLineCount(text(relative)) <= 1200,
    `P5-M02 source exceeds handwritten limit: ${relative}`);

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
  && status.includes("| Next unblocked batch | `G14-P5-M03` |")
  && status.includes("| `G14-P5-M02` | `Complete` |"),
"G14-P5-M02 ledger is incomplete");

console.log(
  "Goal 14 P5-M02 verified (6/6 versioned-policy Stats Conundrum rules " +
  "production-executed through generic combat modifiers).",
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
function physicalLineCount(contents) {
  const lines = contents.split(/\r?\n/u);
  return lines.at(-1) === "" ? lines.length - 1 : lines.length;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
