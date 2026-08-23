#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildDispositionArtifacts } from "./generate-dispositions.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const outputRoot = path.join(root, "content-manifests/currency-wars-runtime-v1");
const sourceRepository = path.join(root, ".cache/content-reference/turnbasedgamedata");
const sourceRevision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
const artifacts = buildDispositionArtifacts();

run("node", ["tools/currency-wars-runtime/generate-foundation.mjs", "--check"]);
run("node", ["tools/currency-wars-runtime/generate-dispositions.mjs", "--check"]);

const source = artifacts["source-dispositions.json"];
const mechanics = artifacts["mechanic-dispositions.json"];
const partitions = artifacts["mechanic-partitions.json"];
const ledger = artifacts["batch-ledger.json"];
const index = artifacts["runtime-dispositions.json"];
const encounter = artifacts["encounter-execution-audit.json"];
const enemyAffix = artifacts["enemy-affix-execution-audit.json"];
const battleAssembly = artifacts["battle-assembly-execution-audit.json"];
const battleSettlement = artifacts["battle-settlement-execution-audit.json"];
const transitionReplay = artifacts["transition-replay-execution-audit.json"];
const battleBehaviorPolicy = artifacts["battle-behavior-policy-execution-audit.json"];
const avatarBattleBehaviorPolicy =
  artifacts["avatar-battle-behavior-policy-execution-audit.json"];
const avatarBattleBehaviorM03 =
  artifacts["avatar-battle-behavior-m03-execution-audit.json"];
const battleConfigurationM04 =
  artifacts["battle-configuration-m04-execution-audit.json"];
const bondBattleBehaviorM05 =
  artifacts["bond-battle-behavior-m05-execution-audit.json"];
const battleProgramBindingM06 =
  artifacts["battle-program-binding-m06-execution-audit.json"];
const battleAvatarProgramM07 =
  artifacts["battle-avatar-program-m07-execution-audit.json"];
const battleAvatarProgramM08 =
  artifacts["battle-avatar-program-m08-execution-audit.json"];
const battleAvatarProgramM09 =
  artifacts["battle-avatar-program-m09-execution-audit.json"];
const enemyCharacterProgramM10 =
  artifacts["enemy-character-program-m10-execution-audit.json"];
const complexAiGlobalFactorM11 =
  artifacts["complex-ai-global-factor-m11-execution-audit.json"];
const battleAiProgramM12 = artifacts["battle-ai-program-m12-execution-audit.json"];
const globalTaskTemplateM13 =
  artifacts["global-task-template-m13-execution-audit.json"];
const metadataMechanicPartition =
  artifacts["metadata-mechanic-partition-execution-audit.json"];
const baselineController = artifacts["baseline-controller-execution-audit.json"];
const cliReplay = artifacts["cli-replay-execution-audit.json"];
const agentApi = artifacts["agent-api-execution-audit.json"];
const mcp = artifacts["mcp-execution-audit.json"];
const replay = artifacts["replay-execution-audit.json"];
const matrix = artifacts["legal-matrix-execution-audit.json"];
const hardening = artifacts["hardening-execution-audit.json"];
const performance = artifacts["performance-execution-audit.json"];
const repositoryAudit = artifacts["repository-release-audit.json"];
const exactCoverage = artifacts["exact-runtime-coverage-audit.json"];

assert(source.obligations.length === 19_250, "source exact-once denominator drift");
assert(unique(source.obligations.map(({ obligation_id: id }) => id)) === 19_250,
  "source obligations are not exact-once");
assert((source.summary.target_dispositions.Integrated ?? 0)
  + (source.summary.target_dispositions.MetadataOnly ?? 0)
  + (source.summary.target_dispositions.Excluded ?? 0) === 19_250,
"source disposition total drift");
assert(source.summary.target_dispositions.Excluded === 726,
  "evidence-only exclusion denominator drift");
assert(source.summary.target_dispositions.Blocked === undefined,
  "blocked source disposition is forbidden");
const p1b2 = source.obligations.filter(({ catalog_batch: batch }) =>
  batch === "G21-P1-B2");
assert(p1b2.length === 1_438
  && p1b2.filter(({ catalog_status: status }) => status === "ExactCatalogLowered").length
    === 1_417
  && p1b2.filter(({ catalog_status: status }) => status === "ExcludedWithProof").length
    === 21,
"P1-B2 catalog-lowering evidence is incomplete");
const p1b3 = source.obligations.filter(({ catalog_batch: batch }) =>
  batch === "G21-P1-B3");
assert(p1b3.length === 3_867
  && p1b3.filter(({ catalog_status: status }) => status === "ExactCatalogLowered").length
    === 3_669
  && p1b3.filter(({ catalog_status: status }) => status === "ExcludedWithProof").length
    === 198,
"P1-B3 catalog-lowering evidence is incomplete");
const p1b4 = source.obligations.filter(({ catalog_batch: batch }) =>
  batch === "G21-P1-B4");
assert(p1b4.length === 8_481
  && p1b4.filter(({ catalog_status: status }) => status === "ExactCatalogLowered").length
    === 7_997
  && p1b4.filter(({ catalog_status: status }) => status === "ExcludedWithProof").length
    === 484,
"P1-B4 catalog-lowering evidence is incomplete");
const p1b5 = source.obligations.filter(({ catalog_batch: batch }) =>
  batch === "G21-P1-B5");
assert(p1b5.length === 2_113
  && p1b5.filter(({ catalog_status: status }) => status === "ExactCatalogLowered").length
    === 2_107
  && p1b5.filter(({ catalog_status: status }) => status === "ExcludedWithProof").length
    === 6,
"P1-B5 catalog-lowering evidence is incomplete");
const p1b6 = source.obligations.filter(({ catalog_batch: batch }) =>
  batch === "G21-P1-B6");
assert(p1b6.length === 3_323
  && p1b6.filter(({ catalog_status: status }) => status === "ExactCatalogLowered").length
    === 3_306
  && p1b6.filter(({ catalog_status: status }) => status === "ExcludedWithProof").length
    === 17,
"P1-B6 catalog-lowering evidence is incomplete");
assert(source.obligations.every(({ catalog_batch: batch, catalog_status: status }) =>
  batch === "G21-P1-B1" || batch === "G21-P1-B2" || batch === "G21-P1-B3"
    || batch === "G21-P1-B4" || batch === "G21-P1-B5"
    || batch === "G21-P1-B6" || status === "ExcludedWithProof"),
"a later catalog batch claims early lowering");
const p6b1 = source.obligations.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-B1");
assert(p6b1.length === 939
  && p6b1.every(({ runtime_status: status }) => status === "Terminal")
  && encounter.status === "Complete"
  && encounter.immutable_runtime_inputs.gridfight_monsters === 160
  && encounter.immutable_runtime_inputs.elite_scaling_groups === 146
  && encounter.immutable_runtime_inputs.enemy_difficulty_rows === 603
  && encounter.immutable_runtime_inputs.formation_waves === 5
  && encounter.exact_reachability.current_monster_unreachable_elite_scaling_groups === 138,
"P6-B1 encounter execution closure is incomplete");
const p6b2 = source.obligations.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-B2");
assert(p6b2.length === 414
  && p6b2.every(({ runtime_status: status }) => status === "Terminal")
  && enemyAffix.result === "Pass"
  && enemyAffix.production_denominators.enemy_affix_definitions === 51
  && enemyAffix.production_denominators.enemy_affix_maze_buffs === 67
  && enemyAffix.production_denominators.enemy_difficulty_rows === 603
  && enemyAffix.exact_behavior.resolver_content_id_branches === 0,
"P6-B2 enemy Affix execution closure is incomplete");
const p6b3 = source.obligations.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-B3");
assert(p6b3.length === 1_428
  && p6b3.every(({ runtime_status: status }) => status === "Terminal")
  && battleAssembly.result === "Pass"
  && battleAssembly.production_denominators.integrated_source_rows === 1_340
  && battleAssembly.production_denominators.excluded_source_rows === 88
  && battleAssembly.exact_behavior.immutable_snapshot_only
  && battleAssembly.exact_behavior.live_activity_lookup_after_assembly === false
  && battleAssembly.exact_behavior.resolver_content_id_branches === 0,
"P6-B3 BattleSpec assembly execution closure is incomplete");
const p6b4 = source.obligations.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-B4");
assert(p6b4.length === 1_122
  && p6b4.every(({ runtime_status: status }) => status === "Terminal")
  && battleSettlement.result === "Pass"
  && battleSettlement.production_denominators.integrated_source_rows === 1_122
  && battleSettlement.exact_behavior.returned_state_hash_is_final
  && battleSettlement.exact_behavior.rejected_mutation_preserves_state_and_rng,
"P6-B4 battle settlement execution closure is incomplete");
assert(transitionReplay.result === "Pass"
  && transitionReplay.assigned_source_rows === 0
  && transitionReplay.exact_behavior.mode_id_resolver_branches === 0
  && transitionReplay.replay_boundary.deferred_owner === "G21-P7-B5",
"P6-B5 transition reconstruction closure is incomplete");
const p6m01Sources = source.obligations.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M01");
assert(p6m01Sources.length === 37
  && p6m01Sources.every(({ runtime_status: status }) => status === "Terminal")
  && p6m01Sources.filter(({ accuracy_class: accuracy }) =>
    accuracy === "VersionedProjectPolicy").length === 9
  && battleBehaviorPolicy.status === "VersionedProjectPolicyExecutable"
  && battleBehaviorPolicy.executable_policy_count === 9
  && battleBehaviorPolicy.terminal_metadata_count === 28
  && battleBehaviorPolicy.selected_behavior.same_released_family_bindings === 4
  && battleBehaviorPolicy.selected_behavior.deterministic_rank_fallback_bindings === 5
  && battleBehaviorPolicy.selected_behavior.raw_postfix_interpreter === false
  && battleBehaviorPolicy.selected_behavior.observed_parity_claimed === false,
"P6-M01 battle behavior policy closure is incomplete");
const p6m02Sources = source.obligations.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M02");
assert(p6m02Sources.length === 64
  && p6m02Sources.every(({ runtime_status: status }) => status === "Terminal")
  && p6m02Sources.filter(({ accuracy_class: accuracy }) =>
    accuracy === "VersionedProjectPolicy").length === 29
  && avatarBattleBehaviorPolicy.status === "VersionedProjectPolicyExecutable"
  && avatarBattleBehaviorPolicy.executable_policy_count === 29
  && avatarBattleBehaviorPolicy.terminal_metadata_count === 35
  && avatarBattleBehaviorPolicy.archetypes.role_battle_event === 28
  && avatarBattleBehaviorPolicy.archetypes.augment_battle_event === 1
  && avatarBattleBehaviorPolicy.binding_policies.exact_battle_event === 28
  && avatarBattleBehaviorPolicy.binding_policies.typed_augment_controller === 1
  && avatarBattleBehaviorPolicy.released_binding_totals.battle_event_ids === 28
  && avatarBattleBehaviorPolicy.selected_behavior.augment_controller_policy_id
    === "currency-wars.augment-controller-contribution-policy.v1"
  && avatarBattleBehaviorPolicy.selected_behavior.raw_postfix_interpreter === false
  && avatarBattleBehaviorPolicy.selected_behavior.observed_parity_claimed === false,
"P6-M02 avatar battle behavior policy closure is incomplete");
const p6m03Sources = source.obligations.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M03");
assert(p6m03Sources.length === 64
  && p6m03Sources.every(({ runtime_status: status }) => status === "Terminal")
  && p6m03Sources.filter(({ accuracy_class: accuracy }) =>
    accuracy === "VersionedProjectPolicy").length === 32
  && avatarBattleBehaviorM03.status === "VersionedProjectPolicyExecutable"
  && avatarBattleBehaviorM03.executable_policy_count === 32
  && avatarBattleBehaviorM03.terminal_metadata_count === 32
  && avatarBattleBehaviorM03.archetypes.role_battle_event === 32
  && avatarBattleBehaviorM03.binding_policies.exact_battle_event === 28
  && avatarBattleBehaviorM03.binding_policies.same_family_battle_event_fallback === 4
  && avatarBattleBehaviorM03.released_binding_totals.role_ids === 25
  && avatarBattleBehaviorM03.released_binding_totals.avatar_ids === 36
  && avatarBattleBehaviorM03.released_binding_totals.battle_event_ids === 39
  && avatarBattleBehaviorM03.selected_behavior.raw_postfix_interpreter === false
  && avatarBattleBehaviorM03.selected_behavior.observed_parity_claimed === false,
"P6-M03 avatar battle behavior policy closure is incomplete");
const p6m04Sources = source.obligations.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M04");
assert(p6m04Sources.length === 64
  && p6m04Sources.every(({ runtime_status: status }) => status === "Terminal")
  && p6m04Sources.filter(({ accuracy_class: accuracy }) =>
    accuracy === "VersionedProjectPolicy").length === 29
  && battleConfigurationM04.status === "VersionedProjectPolicyExecutable"
  && battleConfigurationM04.executable_policy_count === 29
  && battleConfigurationM04.terminal_metadata_count === 35
  && battleConfigurationM04.policy_families.exact_avatar_battle_events === 21
  && battleConfigurationM04.policy_families.typed_configuration_controllers === 8
  && Object.keys(battleConfigurationM04.configuration_archetypes).length === 8
  && battleConfigurationM04.selected_behavior.raw_postfix_interpreter === false
  && battleConfigurationM04.selected_behavior.observed_parity_claimed === false,
"P6-M04 battle configuration policy closure is incomplete");
const p6m05Sources = source.obligations.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M05");
assert(p6m05Sources.length === 64
  && p6m05Sources.every(({ runtime_status: status }) => status === "Terminal")
  && p6m05Sources.filter(({ accuracy_class: accuracy }) =>
    accuracy === "VersionedProjectPolicy").length === 31
  && bondBattleBehaviorM05.status === "VersionedProjectPolicyExecutable"
  && bondBattleBehaviorM05.executable_policy_count === 31
  && bondBattleBehaviorM05.terminal_metadata_count === 33
  && bondBattleBehaviorM05.archetypes.BondStageAbilityController === 25
  && bondBattleBehaviorM05.archetypes.MultiBondStageAbilityController === 1
  && bondBattleBehaviorM05.archetypes.WolfHuntSummonController === 5
  && bondBattleBehaviorM05.released_binding_totals.bond_ids === 36
  && bondBattleBehaviorM05.selected_behavior.raw_postfix_interpreter === false
  && bondBattleBehaviorM05.selected_behavior.observed_parity_claimed === false,
"P6-M05 Bond battle behavior policy closure is incomplete");
const p6m06Sources = source.obligations.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M06");
assert(p6m06Sources.length === 64
  && p6m06Sources.every(({ runtime_status: status }) => status === "Terminal")
  && p6m06Sources.filter(({ accuracy_class: accuracy }) =>
    accuracy === "VersionedProjectPolicy").length === 26
  && battleProgramBindingM06.status === "VersionedProjectPolicyExecutable"
  && battleProgramBindingM06.executable_policy_count === 26
  && battleProgramBindingM06.terminal_metadata_count === 38
  && battleProgramBindingM06.archetypes.CoreAvatarAbility === 7
  && battleProgramBindingM06.archetypes.ServantAbility === 1
  && battleProgramBindingM06.archetypes.RoleBattleEvent === 10
  && battleProgramBindingM06.archetypes.BondStageAbility === 4
  && battleProgramBindingM06.archetypes.AugmentStageAbility === 1
  && battleProgramBindingM06.archetypes.MonsterTagController === 2
  && battleProgramBindingM06.archetypes.EquipmentController === 1
  && battleProgramBindingM06.released_binding_totals.role_ids === 19
  && battleProgramBindingM06.released_binding_totals.battle_event_ids === 10
  && battleProgramBindingM06.released_binding_totals.bond_ids === 4
  && battleProgramBindingM06.released_binding_totals.augment_maze_buff_ids === 7
  && battleProgramBindingM06.released_binding_totals.enemy_affix_maze_buff_ids === 15
  && battleProgramBindingM06.released_binding_totals.equipment_ids === 8
  && battleProgramBindingM06.selected_behavior.raw_postfix_interpreter === false
  && battleProgramBindingM06.selected_behavior.observed_parity_claimed === false,
"P6-M06 battle-program binding policy closure is incomplete");
const p6m07Sources = source.obligations.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M07");
assert(p6m07Sources.length === 64
  && p6m07Sources.every(({ runtime_status: status }) => status === "Terminal")
  && battleAvatarProgramM07.status === "VersionedProjectPolicyExecutable"
  && battleAvatarProgramM07.executable_policy_count === 30
  && battleAvatarProgramM07.terminal_metadata_count === 34
  && battleAvatarProgramM07.binding_policy_count === 29
  && battleAvatarProgramM07.common_controller_count === 1
  && battleAvatarProgramM07.presentation_camera_count === 2
  && battleAvatarProgramM07.layout_descriptor_count === 32
  && battleAvatarProgramM07.archetypes.CoreAvatarAbility === 26
  && battleAvatarProgramM07.archetypes.ServantAbility === 2
  && battleAvatarProgramM07.archetypes.RoleBattleEvent === 1
  && battleAvatarProgramM07.released_binding_totals.role_ids === 31
  && battleAvatarProgramM07.released_binding_totals.avatar_ids === 28
  && battleAvatarProgramM07.released_binding_totals.servant_ids === 2
  && battleAvatarProgramM07.released_binding_totals.battle_event_ids === 1
  && battleAvatarProgramM07.selected_behavior.raw_postfix_interpreter === false
  && battleAvatarProgramM07.selected_behavior.observed_parity_claimed === false,
"P6-M07 Avatar-program policy closure is incomplete");
const p6m08Sources = source.obligations.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M08");
assert(p6m08Sources.length === 63
  && p6m08Sources.every(({ runtime_status: status }) => status === "Terminal")
  && battleAvatarProgramM08.status === "VersionedProjectPolicyExecutable"
  && battleAvatarProgramM08.executable_policy_count === 35
  && battleAvatarProgramM08.terminal_metadata_count === 28
  && battleAvatarProgramM08.binding_policy_count === 35
  && battleAvatarProgramM08.layout_descriptor_count === 28
  && battleAvatarProgramM08.archetypes.CoreAvatarAbility === 15
  && battleAvatarProgramM08.archetypes.RoleBattleEvent === 19
  && battleAvatarProgramM08.archetypes.BondStageAbility === 1
  && battleAvatarProgramM08.released_binding_totals.role_ids === 31
  && battleAvatarProgramM08.released_binding_totals.avatar_ids === 29
  && battleAvatarProgramM08.released_binding_totals.battle_event_ids === 19
  && battleAvatarProgramM08.released_binding_totals.bond_ids === 1
  && battleAvatarProgramM08.selected_behavior.raw_postfix_interpreter === false
  && battleAvatarProgramM08.selected_behavior.observed_parity_claimed === false,
"P6-M08 Avatar/BattleEvent-program policy closure is incomplete");
const p6m09Sources = source.obligations.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M09");
assert(p6m09Sources.length === 64
  && p6m09Sources.every(({ runtime_status: status }) => status === "Terminal")
  && battleAvatarProgramM09.status === "VersionedProjectPolicyExecutable"
  && battleAvatarProgramM09.executable_policy_count === 42
  && battleAvatarProgramM09.terminal_metadata_count === 22
  && battleAvatarProgramM09.binding_policy_count === 42
  && battleAvatarProgramM09.layout_descriptor_count === 22
  && battleAvatarProgramM09.archetypes.CoreAvatarAbility === 1
  && battleAvatarProgramM09.archetypes.RoleBattleEvent === 41
  && battleAvatarProgramM09.released_binding_totals.role_ids === 62
  && battleAvatarProgramM09.released_binding_totals.avatar_ids === 57
  && battleAvatarProgramM09.released_binding_totals.battle_event_ids === 88
  && battleAvatarProgramM09.selected_behavior.raw_postfix_interpreter === false
  && battleAvatarProgramM09.selected_behavior.observed_parity_claimed === false,
"P6-M09 BattleEvent-configuration policy closure is incomplete");
const p6m10Sources = source.obligations.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M10");
assert(p6m10Sources.length === 12
  && p6m10Sources.every(({ runtime_status: status }) => status === "Terminal")
  && enemyCharacterProgramM10.status === "ExactExecutable"
  && enemyCharacterProgramM10.accuracy === "ExactEvidence"
  && enemyCharacterProgramM10.exact_program_count === 11
  && enemyCharacterProgramM10.terminal_metadata_count === 1
  && enemyCharacterProgramM10.released_binding_totals.shared_enemy_keys === 11
  && enemyCharacterProgramM10.released_binding_totals.source_template_ids === 11
  && enemyCharacterProgramM10.released_shape_totals.ability_names === 60
  && enemyCharacterProgramM10.released_shape_totals.skills === 129
  && enemyCharacterProgramM10.released_shape_totals.skill_ability_bindings === 95
  && enemyCharacterProgramM10.released_shape_totals.dynamic_sources === 290,
"P6-M10 enemy character-configuration closure is incomplete");
const p6m11Sources = source.obligations.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M11");
assert(p6m11Sources.length === 64
  && p6m11Sources.every(({ runtime_status: status }) => status === "Terminal")
  && complexAiGlobalFactorM11.status === "VersionedProjectPolicyExecutable"
  && complexAiGlobalFactorM11.accuracy
    === "ExactAuthoredShapeWithPolicyBoundMapperSemantics"
  && complexAiGlobalFactorM11.exact_program_count === 1
  && complexAiGlobalFactorM11.terminal_metadata_count === 63
  && complexAiGlobalFactorM11.released_shape_totals.groups === 2
  && complexAiGlobalFactorM11.released_shape_totals.factors === 5
  && complexAiGlobalFactorM11.released_shape_totals.ranges === 13
  && complexAiGlobalFactorM11.released_shape_totals.property_ratio_sources === 2
  && complexAiGlobalFactorM11.released_shape_totals.ai_tag_sources === 1
  && complexAiGlobalFactorM11.released_shape_totals.contains_modifier_sources === 2
  && complexAiGlobalFactorM11.policy_boundary.confidence === "PolicyOnlyNotObservedParity"
  && complexAiGlobalFactorM11.policy_boundary.observed_parity_claimed === false,
"P6-M11 global Complex AI factor closure is incomplete");
const p6m12Sources = source.obligations.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M12");
assert(p6m12Sources.length === 24
  && p6m12Sources.every(({ runtime_status: status }) => status === "Terminal")
  && battleAiProgramM12.status === "ExecutableWithExplicitPolicyBoundary"
  && battleAiProgramM12.exact_program_count === 4
  && battleAiProgramM12.terminal_metadata_count === 20
  && battleAiProgramM12.complex_ai_shape.groups === 9
  && battleAiProgramM12.complex_ai_shape.factors === 20
  && battleAiProgramM12.complex_ai_shape.ranges === 42
  && battleAiProgramM12.complex_ai_shape.confidence === "PolicyOnlyNotObservedParity"
  && battleAiProgramM12.complex_ai_shape.observed_parity_claimed === false
  && battleAiProgramM12.enemy_ai_shape.programs === 3
  && battleAiProgramM12.enemy_ai_shape.bindings === 4
  && battleAiProgramM12.enemy_ai_shape.variables === 2
  && battleAiProgramM12.enemy_ai_shape.decisions === 41
  && battleAiProgramM12.enemy_ai_shape.skill_uses === 55
  && battleAiProgramM12.enemy_ai_shape.typed_nodes === 459,
"P6-M12 battle AI closure is incomplete");
const p6m13Sources = source.obligations.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M13");
assert(p6m13Sources.length === 64
  && p6m13Sources.every(({ runtime_status: status }) => status === "Terminal")
  && globalTaskTemplateM13.status === "ExecutableExactTemplateLibrary"
  && globalTaskTemplateM13.exact_program_count === 1
  && globalTaskTemplateM13.terminal_metadata_count === 63
  && globalTaskTemplateM13.released_shape.templates === 13
  && globalTaskTemplateM13.released_shape.executable_modifier_templates === 6
  && globalTaskTemplateM13.released_shape.presentation_only_templates === 7
  && globalTaskTemplateM13.released_shape.typed_nodes === 235
  && globalTaskTemplateM13.released_shape.add_modifier_nodes === 11,
"P6-M13 global task-template closure is incomplete");
const metadataPartitionPattern = /^G21-P6-M(?:1[4-9]|2[0-9]|3[0-2])$/u;
const completedMetadataPartitions = partitions.partitions.filter(({ batch }) =>
  metadataPartitionPattern.test(batch)
    && ledger.batches.find(({ batch: ledgerBatch }) => ledgerBatch === batch)?.status
      === "Complete");
assert(metadataMechanicPartition.status === "SequentialMetadataOnlyClosure"
  && metadataMechanicPartition.authoritative_operation_count === 0
  && metadataMechanicPartition.reviewed_batch_count === 19
  && metadataMechanicPartition.reviewed_program_count === 1_135
  && metadataMechanicPartition.receipts[0]?.batch === "G21-P6-M14"
  && metadataMechanicPartition.receipts.at(-1)?.batch === "G21-P6-M32"
  && metadataMechanicPartition.reviewed_batch_count === completedMetadataPartitions.length
  && metadataMechanicPartition.receipts.length === completedMetadataPartitions.length
  && metadataMechanicPartition.reviewed_program_count
    === completedMetadataPartitions.reduce((sum, partition) =>
      sum + partition.program_count, 0),
"P6 metadata-only partition receipt accounting drift");
for (const [index, receipt] of metadataMechanicPartition.receipts.entries()) {
  const partition = completedMetadataPartitions[index];
  const programs = mechanics.programs.filter(({ execution_batch: batch }) =>
    batch === receipt.batch);
  assert(receipt.batch === partition.batch
    && receipt.status === "TerminalMetadataOnly"
    && receipt.program_count === partition.program_count
    && receipt.terminal_program_count === partition.program_count
    && receipt.partition_freeze_sha256 === partition.freeze_sha256
    && receipt.operation_kinds.PreserveExactSourceContribution
      === partition.program_count
    && programs.length === partition.program_count
    && programs.every(({ target_execution: target, runtime_status: status }) =>
      target === "MetadataOnly" && status === "Terminal")
    && /^[0-9a-f]{64}$/u.test(receipt.exact_source_receipt_sha256),
  `${receipt.batch} metadata-only exact-once receipt drift`);
}
assert(baselineController.result === "Pass"
  && baselineController.assigned_source_rows === 0
  && baselineController.controller.owner === "starclock-ai"
  && baselineController.controller.activity_step_budget === 1_024
  && baselineController.controller.battle_command_budget === 10_000
  && /^[0-9a-f]{64}$/u.test(baselineController.controller.identity_sha256)
  && baselineController.command_contract.direct_authoritative_mutation === false
  && baselineController.complete_run_evidence.battle_nodes_per_run === 7
  && baselineController.complete_run_evidence.standard_terminal === "Completed"
  && baselineController.complete_run_evidence.overclock_terminal === "Completed"
  && baselineController.complete_run_evidence.same_seed_report_equality
  && baselineController.complete_run_evidence.real_nested_battles,
"P7-B1 deterministic baseline-controller closure is incomplete");
assert(cliReplay.result === "Pass"
  && cliReplay.assigned_source_rows === 0
  && cliReplay.commands.validate.startsWith("currency-wars config validate")
  && cliReplay.commands.inspect.startsWith("currency-wars inspect")
  && cliReplay.commands.coverage.startsWith("currency-wars coverage")
  && cliReplay.commands.run.startsWith("currency-wars run")
  && cliReplay.commands.verify.startsWith("replay verify")
  && cliReplay.current_coverage.source_obligations === 19_250
  && cliReplay.current_coverage.source_terminal === 19_250
  && cliReplay.current_coverage.source_pending === 0
  && cliReplay.current_coverage.mechanic_programs === 2_367
  && cliReplay.current_coverage.mechanic_terminal === 2_367
  && cliReplay.current_coverage.native_handlers === 0
  && cliReplay.replay_contract.component_count === 9
  && cliReplay.replay_contract.first_divergence_categories.length === 5
  && cliReplay.replay_contract.accepted_activity_commands
  && cliReplay.replay_contract.accepted_battle_commands
  && cliReplay.replay_contract.expected_activity_states
  && cliReplay.replay_contract.expected_battle_states_and_events
  && cliReplay.replay_contract.fresh_immutable_reexecution
  && cliReplay.replay_contract.exact_byte_verification
  && cliReplay.production_run_evidence.standard_terminal === "Completed"
  && cliReplay.production_run_evidence.overclock_terminal === "Completed"
  && cliReplay.production_run_evidence.nested_battles_per_run === 7,
"P7-B2 CLI coverage/run/replay closure is incomplete");
assert(agentApi.result === "Pass"
  && agentApi.assigned_source_rows === 0
  && agentApi.manifest_contract.route_summaries === 26
  && agentApi.manifest_contract.difficulty_summaries === 97
  && agentApi.manifest_contract.gambits.length === 2
  && agentApi.manifest_contract.baseline_fixture_roles.length === 4
  && agentApi.manifest_contract.exact_configuration_digest
  && agentApi.manifest_contract.exact_content_digest
  && agentApi.manifest_contract.generated_rows_exposed === false
  && agentApi.session_contract.shared_registry_ownership
  && agentApi.session_contract.shared_registry_quotas
  && agentApi.session_contract.opaque_action_tokens
  && agentApi.session_contract.expected_state_hash_required
  && agentApi.session_contract.exact_boundary_required
  && agentApi.session_contract.bounded_idempotency_cache
  && agentApi.session_contract.direct_authoritative_mutation === false
  && agentApi.observation_contract.source === "ActivityPlayerView"
  && agentApi.observation_contract.debug_view_exposed === false
  && agentApi.observation_contract.combat_catalog_exposed === false
  && agentApi.observation_contract.generated_configuration_rows_exposed === false
  && agentApi.incremental_execution.encounter_and_preparation_are_distinct_actions
  && agentApi.incremental_execution.preparation_executes_one_real_nested_battle
  && agentApi.incremental_execution.stale_action_preserves_state
  && agentApi.replay_boundary.shared_component_reconstruction
  && agentApi.replay_boundary.terminal_session_export
  && agentApi.replay_boundary.fresh_agent_verification,
"P7-B3 Agent API closure is incomplete");
assert(mcp.result === "Pass"
  && mcp.assigned_source_rows === 0
  && mcp.surface.create_tool.includes("currency-wars")
  && mcp.surface.authoritative_session_owner.includes("ActivityAgentSessionRegistry")
  && mcp.surface.mcp_owned_runtime_state === false
  && mcp.authorization.create_scope === "starclock:activity:create"
  && mcp.authorization.observe_scope === "starclock:activity:read"
  && mcp.authorization.action_scope === "starclock:activity:act"
  && mcp.authorization.cancel_scope === "starclock:activity:close"
  && mcp.authorization.tenant_and_principal_ownership_checked_before_disclosure
  && mcp.idempotency_and_cancellation.exact_idempotency_key_binding
  && mcp.idempotency_and_cancellation.response_loss_retry_is_byte_equal
  && mcp.idempotency_and_cancellation.mcp_cancel_notification_does_not_rollback_committed_action
  && mcp.idempotency_and_cancellation.retry_after_cancel_does_not_commit_twice
  && mcp.idempotency_and_cancellation.close_cancels_session_and_releases_quota
  && mcp.event_pagination.maximum_events_per_page === 256
  && mcp.event_pagination.maximum_retained_events === 8_192
  && mcp.event_pagination.future_cursor_rejected
  && mcp.event_pagination.expired_cursor_rejected
  && mcp.event_pagination.idempotent_retry_duplicates_event === false
  && mcp.event_pagination.generated_rows_or_private_state_exposed === false
  && mcp.execution_evidence.encounter_and_preparation_are_distinct_mcp_calls
  && mcp.execution_evidence.preparation_settles_one_real_nested_battle
  && mcp.execution_evidence.nested_battle_count_observed === 1,
"P7-B4 MCP closure is incomplete");

assert(mechanics.programs.length === 2_367, "mechanic exact-once denominator drift");
assert(unique(mechanics.programs.map(({ mechanic_id: id }) => id)) === 2_367,
  "mechanic programs are not exact-once");
assert(mechanics.programs.every(({ target_execution: target }) =>
  ["ExactRuleIr", "ExactActivityProgram", "PolicyRuleIr", "PolicyActivityProgram",
    "MetadataOnly"].includes(target)),
"mechanic target execution is not terminally typed");
const p6m01Programs = mechanics.programs.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M01");
assert(p6m01Programs.length === 37
  && p6m01Programs.filter(({ target_execution: target }) => target === "PolicyRuleIr").length
    === 9
  && p6m01Programs.filter(({ target_execution: target }) => target === "MetadataOnly").length
    === 28
  && p6m01Programs.every(({ runtime_status: status }) => status === "Terminal"),
"P6-M01 mechanic partition is not terminally typed");
const p6m02Programs = mechanics.programs.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M02");
assert(p6m02Programs.length === 64
  && p6m02Programs.filter(({ target_execution: target }) => target === "PolicyRuleIr").length
    === 29
  && p6m02Programs.filter(({ target_execution: target }) => target === "MetadataOnly").length
    === 35
  && p6m02Programs.every(({ runtime_status: status }) => status === "Terminal"),
"P6-M02 mechanic partition is not terminally typed");
const p6m03Programs = mechanics.programs.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M03");
assert(p6m03Programs.length === 64
  && p6m03Programs.filter(({ target_execution: target }) => target === "PolicyRuleIr").length
    === 32
  && p6m03Programs.filter(({ target_execution: target }) => target === "MetadataOnly").length
    === 32
  && p6m03Programs.every(({ runtime_status: status }) => status === "Terminal"),
"P6-M03 mechanic partition is not terminally typed");
const p6m04Programs = mechanics.programs.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M04");
assert(p6m04Programs.length === 64
  && p6m04Programs.filter(({ target_execution: target }) => target === "PolicyRuleIr").length
    === 29
  && p6m04Programs.filter(({ target_execution: target }) => target === "MetadataOnly").length
    === 35
  && p6m04Programs.every(({ runtime_status: status }) => status === "Terminal"),
"P6-M04 mechanic partition is not terminally typed");
const p6m05Programs = mechanics.programs.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M05");
assert(p6m05Programs.length === 64
  && p6m05Programs.filter(({ target_execution: target }) => target === "PolicyRuleIr").length
    === 31
  && p6m05Programs.filter(({ target_execution: target }) => target === "MetadataOnly").length
    === 33
  && p6m05Programs.every(({ runtime_status: status }) => status === "Terminal"),
"P6-M05 mechanic partition is not terminally typed");
const p6m06Programs = mechanics.programs.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M06");
assert(p6m06Programs.length === 64
  && p6m06Programs.filter(({ target_execution: target }) => target === "PolicyRuleIr").length
    === 26
  && p6m06Programs.filter(({ target_execution: target }) => target === "MetadataOnly").length
    === 38
  && p6m06Programs.every(({ runtime_status: status }) => status === "Terminal"),
"P6-M06 mechanic partition is not terminally typed");
const p6m07Programs = mechanics.programs.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M07");
assert(p6m07Programs.length === 64
  && p6m07Programs.filter(({ target_execution: target }) => target === "PolicyRuleIr").length
    === 30
  && p6m07Programs.filter(({ target_execution: target }) => target === "MetadataOnly").length
    === 34
  && p6m07Programs.every(({ runtime_status: status }) => status === "Terminal"),
"P6-M07 mechanic partition is not terminally typed");
const p6m08Programs = mechanics.programs.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M08");
assert(p6m08Programs.length === 63
  && p6m08Programs.filter(({ target_execution: target }) => target === "PolicyRuleIr").length
    === 35
  && p6m08Programs.filter(({ target_execution: target }) => target === "MetadataOnly").length
    === 28
  && p6m08Programs.every(({ runtime_status: status }) => status === "Terminal"),
"P6-M08 mechanic partition is not terminally typed");
const p6m09Programs = mechanics.programs.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M09");
assert(p6m09Programs.length === 64
  && p6m09Programs.filter(({ target_execution: target }) => target === "PolicyRuleIr").length
    === 42
  && p6m09Programs.filter(({ target_execution: target }) => target === "MetadataOnly").length
    === 22
  && p6m09Programs.every(({ runtime_status: status }) => status === "Terminal"),
"P6-M09 mechanic partition is not terminally typed");
const p6m10Programs = mechanics.programs.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M10");
assert(p6m10Programs.length === 12
  && p6m10Programs.filter(({ target_execution: target }) => target === "ExactRuleIr").length
    === 11
  && p6m10Programs.filter(({ target_execution: target }) => target === "MetadataOnly").length
    === 1
  && p6m10Programs.every(({ runtime_status: status }) => status === "Terminal"),
"P6-M10 mechanic partition is not terminally typed");
const p6m11Programs = mechanics.programs.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M11");
assert(p6m11Programs.length === 64
  && p6m11Programs.filter(({ target_execution: target }) => target === "ExactRuleIr").length
    === 1
  && p6m11Programs.filter(({ target_execution: target }) => target === "MetadataOnly").length
    === 63
  && p6m11Programs.every(({ runtime_status: status }) => status === "Terminal"),
"P6-M11 mechanic partition is not terminally typed");
const p6m12Programs = mechanics.programs.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M12");
assert(p6m12Programs.length === 24
  && p6m12Programs.filter(({ target_execution: target }) => target === "ExactRuleIr").length
    === 4
  && p6m12Programs.filter(({ target_execution: target }) => target === "MetadataOnly").length
    === 20
  && p6m12Programs.every(({ runtime_status: status }) => status === "Terminal"),
"P6-M12 mechanic partition is not terminally typed");
const p6m13Programs = mechanics.programs.filter(({ execution_batch: batch }) =>
  batch === "G21-P6-M13");
assert(p6m13Programs.length === 64
  && p6m13Programs.filter(({ target_execution: target }) => target === "ExactRuleIr").length
    === 1
  && p6m13Programs.filter(({ target_execution: target }) => target === "MetadataOnly").length
    === 63
  && p6m13Programs.every(({ runtime_status: status }) => status === "Terminal"),
"P6-M13 mechanic partition is not terminally typed");
assert(mechanics.programs.every(({ static_handler: handler }) => handler === null),
  "native handler admitted without audit");
assert(mechanics.summary.native_handlers_admitted === 0,
  "native handler denominator must start at zero");
verifyMetadataSources(mechanics.programs
  .filter(({ target_execution: target }) => target === "MetadataOnly"));
const progressionPrograms = mechanics.programs.filter(({ runtime_status: status,
  source_path: sourcePath }) => status === "Terminal"
  && ["ExcelOutput/GridFightExpertRestrict.json",
    "ExcelOutput/GridFightSeasonExpScore.json"].includes(sourcePath));
assert(progressionPrograms.length === 85
  && progressionPrograms.filter(({ execution_batch: batch }) =>
    batch === "G21-P5-A03").length === 64
  && progressionPrograms.filter(({ execution_batch: batch }) =>
    batch === "G21-P5-A04").length === 21,
"A03/A04 progression execution closure drift");

const partitionIds = partitions.partitions
  .flatMap(({ mechanic_ids: ids }) => ids);
assert(partitionIds.length === 2_367 && unique(partitionIds) === 2_367,
  "partition mechanic coverage is not exact-once");
assert(partitions.partitions.every(({ program_count: count, mechanic_ids: ids }) =>
  count === ids.length && count > 0 && count <= 64),
"mechanic partition exceeds the 64-program cap");
assert(partitions.summary.activity_partitions >= 9
  && partitions.summary.battle_partitions >= 29
  && partitions.summary.partitions >= 38,
"scope partition lower bound drift");
assert(partitions.freeze.batch === "G21-P2-B5"
  && partitions.freeze.state === "FrozenPendingExecution",
"mechanic partitions are not frozen at the shared-capability boundary");
for (const partition of partitions.partitions) {
  const { freeze_sha256: expected, ...payload } = partition;
  assert(hashValue(payload) === expected, `${partition.batch} freeze digest drift`);
}
assert(hashValue(partitions.partitions) === partitions.freeze.partition_set_sha256,
  "mechanic partition-set freeze digest drift");

assert(ledger.fixture_assignments.length === 28
  && unique(ledger.fixture_assignments.map(({ fixture_family_id: id }) => id)) === 28,
"fixture assignment is not exact-once");
assert(ledger.policy_assignments.length === 12
  && unique(ledger.policy_assignments.map(({ field }) => field)) === 12,
"policy assignment is not exact-once");
const configurationPolicy = ledger.policy_assignments.find(({ field }) =>
  field === "mechanic.configuration_program");
assert(configurationPolicy?.status === "VersionedProjectPolicyExecutable"
  && configurationPolicy.confidence === "PolicyOnlyNotObservedParity"
  && configurationPolicy.rejected_alternatives.length === 3
  && configurationPolicy.selected_behavior.includes("Never interpret raw PostfixBase64"),
"configuration-program policy is not executable and replaceable");
assert(unique(ledger.batches.map(({ batch }) => batch)) === ledger.batches.length,
  "batch ledger identity is not unique");
assert(ledger.batches.every(({ owner, prerequisites, terminal_evidence: evidence }) =>
  owner.length > 0 && Array.isArray(prerequisites) && evidence.length > 0),
"batch ledger ownership or gate target is incomplete");
assert(replay.result === "Pass" && replay.component_contract.count === 9
  && replay.first_divergence.length === 5,
"P7-B5 replay execution audit drift");
assert(matrix.result === "Pass" && matrix.assigned_source_rows === 194
  && matrix.generated_entries === 97
  && matrix.execution.fresh_replay_per_terminal_report,
"P7-B6 legal matrix execution audit drift");
assert(hardening.result === "Pass"
  && Object.keys(hardening.suites).length === 7
  && hardening.bounded_runtime_changes.retained_link_history_limit === 4_096,
"P8-B1 hardening execution audit drift");
assert(performance.result === "Pass" && performance.workloads.length === 8
  && performance.workloads.every(({ guard_elapsed_ns: guard, baseline_elapsed_ns: baseline }) =>
    guard >= baseline),
"P8-B2 performance execution audit drift");
assert(repositoryAudit.result === "Pass"
  && repositoryAudit.sora.version === "0.6.1"
  && repositoryAudit.sora.generated_tables === 111
  && repositoryAudit.sora.generated_rows === 78_607
  && repositoryAudit.sora.workbook_sheets === 111
  && repositoryAudit.sora.visually_reviewed_sheets === 111
  && repositoryAudit.sora.generated_roots.length === 3
  && repositoryAudit.source_policy.unsafe_rust_allowed === false
  && repositoryAudit.source_policy.inline_backend_float_allowed === false
  && repositoryAudit.dependency_policy.license_policy_checked
  && repositoryAudit.architecture.native_handlers_admitted === 0
  && repositoryAudit.architecture.resolver_content_id_branches === 0
  && repositoryAudit.architecture.prior_release_manifests_verified === 4
  && repositoryAudit.architecture.other_mode_rows_promoted === 0
  && repositoryAudit.provenance.exact_manifest_digests_verified
  && repositoryAudit.provenance.ambient_branch_state_required === false
  && Object.values(repositoryAudit.semantic_fixture_evidence)
    .every((status) => status === "Executed"),
"P8-B3 repository release audit drift");
assert(exactCoverage.result === "Pass"
  && exactCoverage.denominators.source_obligations === 19_250
  && exactCoverage.denominators.mechanic_programs === 2_367
  && exactCoverage.denominators.semantic_fixture_families === 28
  && exactCoverage.denominators.project_policies === 12
  && JSON.stringify(exactCoverage.terminal_counts)
    === JSON.stringify(exactCoverage.denominators)
  && exactCoverage.matrix_source_rows === 194
  && exactCoverage.semantic_source_rows === 28
  && exactCoverage.fixture_evidence_paths > 28
  && exactCoverage.policies_with_replacement_conditions === 12
  && Object.values(exactCoverage.forbidden_states).every((count) => count === 0),
"P8-B4 exact runtime coverage audit drift");
const expectedCompletedThrough = "G21-P8-B4";
const completedIndex = ledger.batches.findIndex(({ batch }) =>
  batch === expectedCompletedThrough);
assert(ledger.completed_through === expectedCompletedThrough
  && ledger.next_batch === ledger.batches[completedIndex + 1]?.batch
  && ledger.completed_through === "G21-P8-B4"
  && ledger.next_batch === "G21-P8-B5",
"P8-B4 exact-coverage ledger transition drift");

for (const [file, expected] of Object.entries(index.artifact_digests)) {
  const actual = crypto.createHash("sha256")
    .update(fs.readFileSync(path.join(outputRoot, file)))
    .digest("hex");
  assert(actual === expected, `runtime disposition artifact digest drift: ${file}`);
}
assert(index.summary.pending_source_obligations === 0
  && index.summary.pending_mechanic_programs === 0
  && index.release_state === "RuntimeCoverageCompletePendingNativeRelease",
"P8-B4 runtime coverage state is not terminal");

console.log(
  `Currency Wars dispositions verified (${source.obligations.length.toLocaleString("en-US")} sources; `
    + `${mechanics.programs.length.toLocaleString("en-US")} programs; `
    + `${partitions.partitions.length} partitions; 28 fixtures; 12 policies).`,
);

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}

function verifyMetadataSources(programs) {
  const byPath = new Map();
  for (const program of programs)
    byPath.set(program.source_path, program);
  const allowedLayoutKeys = new Set([
    "BakeInfoLayouts", "Dependency", "Offset", "TypeId", "UniqueName",
  ]);
  const allowedAssetKeys = new Set([
    "$type", "AdvEffectList", "AdvResidentEffects", "Alias", "AnimatorStateEvents",
    "AnimatorStateGroupEvents", "AnimatorStateName", "Art", "ArtNodeLodQuality",
    "BakeInfoLayouts", "ButtonIcon", "ButtonText", "ConfigID", "ConfigName",
    "ConfigNameList", "DefaultLevelGraphPath", "DisableAnimEventLayers",
    "EffectInstanceList",
    "EffectInstanceMap", "EffectPath", "EffectTypes", "EmitterType", "EntityType", "ID",
    "EntityLodQuality", "EventList", "Evanescia_00_Model", "ForceImmediateFadeOut",
    "ForceSimulateImmediately", "ForceTrigger", "GridFight_Topaz", "IsAttachToTimeline",
    "LodModelConfig", "LodPath", "MinMutexTime", "MinMutexType", "ModelInstanceList",
    "Name", "NormalizedTime",
    "OnEnter", "OnExit", "PassiveSkill02", "Skill01", "Skill02", "Skill03", "Skill04",
    "Skill11", "Skill21", "Skill22", "Skill31", "SkillP01", "SkillP02", "SkillPC01",
    "PropButtonConfigs", "RootModelPath", "RuntimeName", "SkillPC02", "SoundName",
    "States", "StopOnAnimStateExit", "SummonEntityList", "SyncTargetAnimatorParam",
    "TagComponents", "TagContainer", "TargetType", "TickLodBoundCenter",
    "TickLodBoundSize", "UniqueEffectName", "Value", "X", "Y", "Z",
  ]);
  const allowedTableKeys = new Set([
    "Hash", "ID", "ModifySkillDesc", "ModifySkillID", "ModifySkillSimpleDesc",
    "ModifySkillType", "Icon", "NpcDesc", "NpcName", "NpcType", "PositionRegion",
    "RoleID", "RoleRemark", "RoundIcon", "SkillComeFrom", "SkillID", "SubIconType",
    "TagDesc",
  ]);
  const allowedTutorialKeys = new Set([
    "$type", "ActionType", "Block", "Consumable", "ConsumableID",
    "CurProgress", "CustomString", "CustomTextDirection", "CustomTimeType",
    "DisableBlackMask", "EmptyGridRegion", "EnableActionList",
    "EnableBattleOperationList", "EnableClickInHintArea", "EntityEventList",
    "Equip", "EuqipID", "Event", "FailedTaskList", "FindEmpty",
    "ForceSetNavigation", "Gender", "GoNextImmediately", "GridRegion",
    "GridRegionList", "GuideHintShowConfig", "GuideHintType", "GuideID",
    "GuideTalkID", "GuideText", "GuideTextShowConfig", "GuideTextType",
    "GuideUIContextConfig", "Hash", "IsAutoMatchGuideHintType", "IsFinish",
    "IsForbid", "IsShow", "Lock", "Name", "NodeID", "NodeIDList",
    "NodeVisibleParam", "OPType", "OffsetX", "OffsetY", "OnController",
    "OnInitSequece", "OnMobile", "OnMobileOrPC", "OnPC",
    "OnStartSequece", "OnSuccessImmediate", "OnTrigger", "OrbID",
    "OverrideActionName", "OverrideTextPrefabPath", "Param", "Pause",
    "PopupPanelType", "PopupPanelVisibleParam", "PosIndex", "Predicate",
    "ProtectTime", "RemoveEquipTrackParam", "Role", "RoleID", "ScaleX",
    "ScaleY", "SetGoldNumParam", "ShowAnim", "ShowDelay", "ShowKeyMapTip",
    "SubToastHintParam", "SuccessTaskList", "TalkIDList", "TargetEquip",
    "TargetEvent", "TargetGrid", "TargetRole", "TaskEnabled", "TaskList",
    "TextID", "Title", "ToastHintParam", "TopHintParam", "TotalProgress",
    "TutorialKey", "Type", "UIControllerName", "UseCustomConfig", "Value",
    "ValueSource", "Visible", "WaitForExit", "WaitSecond",
  ]);
  const allowedConsoleKeys = new Set([
    "$type", "ButtonConfigs", "ButtonIcon", "ButtonName", "ButtonText",
    "ButtonsByName", "Case", "Cases", "Custom", "EntityEventList",
    "EventName", "FailedTaskList", "FixedValue", "FromAnyState", "FromState",
    "ID", "InteractID", "IsClient", "IsDynamic", "IsEnable", "Key", "Name",
    "OnChange", "OnEvent", "OnInitSequece", "OnPressedCallback",
    "OnStartSequece", "OnSuccess", "Param", "Predicate", "SoundName",
    "SuccessTaskList", "SwitchRef", "TargetType", "TaskList", "ToAnyState",
    "ToState", "TriggerName", "Type", "Value", "ValueSource", "Values",
  ]);
  const allowedBattlePresentationKeys = new Set([
    "$type", "AbilityList", "AdditiveNormalConfig", "AimDamp", "AimOffset",
    "AimRatio", "AimTargetType", "Alias", "AnchorOffset", "AnchorRatio",
    "AnchorTargetType", "AnchorToAimAngle", "AnimStateName", "AttackType", "BaseCycle",
    "BlendConfig", "BlendTime", "BlendType", "CameraConfig", "CameraState",
    "CameraTimelineAssetName", "CloseupShotConfig", "CompareType", "CompareValue",
    "CustomCurveName", "CycleDamping", "DistanceAttenuation", "Dutch", "FailedTaskList",
    "FixedValue", "FOV",
    "FollowDamp", "FollowElevationAngle", "FollowPoleAngle", "FollowRadius",
    "ForbidChangeOffset", "ForbidDynamicOffset", "GlobalModifiers", "GlobalTemplates",
    "IsAliveOnly", "IsDynamic", "IsLocalOffset", "IsTargetIgnoreCameraDither", "IsUseFullPeriod",
    "ModifierName", "Name",
    "NormalConfig", "NormalizedTimeEnd", "OnStart", "Override", "OverrideShakeConfigV2",
    "PerlinNoiseAmplitude", "PerlinNoiseFreq",
    "Predicate", "RangeAttenuation", "RangeAttenuationDelay",
    "RangeAttenuationTarget", "ResetToDefault", "RotationFreqV3",
    "RotationalAmplitude", "ShakeConfigV2", "ShakeDir", "ShakeRange",
    "ShakeDistance", "ShakeScale", "ShakeTemplateName", "ShakeTime", "ShowEntityConfig",
    "ShowTargetType", "SuccessTaskList", "TargetInfo", "TargetType",
    "TemplateName", "TransTypeAim", "TransTypeFollow", "Value", "ValueType", "X", "Y", "Z",
  ]);
  const allowedStageEffectKeys = new Set([
    "$type", "AbilityList", "EffectPath", "EnumIndex", "FixedValue",
    "ForceSimulateImmediately", "IsAttachToTimeline", "IsDynamic", "Name", "OnStart",
    "Tag", "TargetInfo", "TargetType", "Value", "WaitTime",
  ]);
  for (const sourcePath of [...byPath.keys()].sort()) {
    const bytes = execFileSync("git", ["show", `${sourceRevision}:${sourcePath}`], {
      cwd: sourceRepository,
      maxBuffer: 64 * 1024 * 1024,
    });
    const value = JSON.parse(bytes.toString("utf8"));
    const program = byPath.get(sourcePath);
    const digestBytes = sourcePath.startsWith("ExcelOutput/")
      ? Buffer.from(JSON.stringify(value[Number(program.source_locator)]))
      : bytes;
    const digest = crypto.createHash("sha256").update(digestBytes).digest("hex");
    const expectedDigest = program.source_sha256;
    assert(digest === expectedDigest,
      `metadata source digest drift: ${sourcePath}`);
    if (sourcePath.startsWith("Config/ConfigCharacter/GridFight/")
      && !sourcePath.endsWith(".layout.json")) {
      assert(program.metadata_basis
        === "The released Version 4.4 role, servant and summon selection tables contain no binding to this preserved character override.",
      `unreachable character override lost its exact reachability proof: ${sourcePath}`);
      continue;
    }
    if (sourcePath
      === "Config/ConfigAbility/BattleEvent/GridFight/3.5/EquipmentAbility/GridFight_Equipment_01.json") {
      assert(program.metadata_basis
        === "The released Version 4.4 equipment definitions contain no ability binding to this preserved legacy equipment configuration.",
      `unreachable equipment configuration lost its exact reachability proof: ${sourcePath}`);
      continue;
    }
    if (sourcePath
      === "Config/ConfigAbility/BattleEvent/GridFight/3.5/OriginAbility/GridFight_Origin_Common_StageAbility.json") {
      assert(program.metadata_basis
        === "The released Version 4.4 source contains no Ability, modifier, callback or typed configuration node and therefore has no authoritative runtime operation."
        && Array.isArray(value.AbilityList) && value.AbilityList.length === 0
        && Object.keys(value.GlobalModifiers).length === 0
        && Array.isArray(value.GlobalTemplates) && value.GlobalTemplates.length === 0,
      `empty Origin configuration lost its exact metadata proof: ${sourcePath}`);
      continue;
    }
    const keys = recursiveKeys(value);
    const allowed = sourcePath
      === "Config/Level/Props/Common/InitLevelGraph_Prop_Common_GridFightConsole_01.json"
      ? allowedConsoleKeys
      : sourcePath.startsWith("Config/ConfigAbility/BattleEvent/Effect/")
          && sourcePath.endsWith("_Effect_Ability.json")
        ? allowedStageEffectKeys
      : sourcePath.startsWith("Config/ConfigAbility/GridFight/3.5/Camera/")
          && !sourcePath.endsWith(".layout.json")
        || [
          "Config/ConfigAbility/BattleEvent/GridFight/3.5/AvatarAbility/BattleEvent_GridFight_Yanqing_00_Camera.json",
          "Config/ConfigAbility/BattleEvent/GridFight/3.5/Basic/GridFight_05_Camera.json",
          "Config/ConfigAbility/BattleEvent/GridFight/3.5/OriginAbility/GridFight_Origin_1008_StageAbility_Camera.json",
          "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Gepard_00_Camera.json",
          "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_PlayerBoyServant_30_Camera.json",
        ].includes(sourcePath)
        || sourcePath
          === "Config/ConfigAbility/GridFight/4.0/Monster/Monster_GridFight_W5_Vtuber_00_Ability.json"
        ? allowedBattlePresentationKeys
      : sourcePath.includes("/TutorialTask/")
      ? allowedTutorialKeys
      : sourcePath.endsWith(".layout.json")
      ? allowedLayoutKeys
      : sourcePath.includes("/AssetPreload/")
        || sourcePath.startsWith("Config/ConfigAnimEvents/")
        || sourcePath.startsWith("Config/ConfigEntity/Props/Common/Prop_Common_GridFight")
        || sourcePath.startsWith("Config/Props/Common/Prop_Common_GridFight")
        ? allowedAssetKeys : allowedTableKeys;
    for (const key of keys)
      assert(allowed.has(key),
        `metadata-only source acquired an unaudited field ${key}: ${sourcePath}`);
  }
}

function recursiveKeys(value, keys = new Set()) {
  if (Array.isArray(value)) {
    for (const item of value)
      recursiveKeys(item, keys);
  } else if (value !== null && typeof value === "object") {
    for (const [key, item] of Object.entries(value)) {
      keys.add(key);
      recursiveKeys(item, keys);
    }
  }
  return keys;
}

function unique(values) {
  return new Set(values).size;
}

function hashValue(value) {
  return crypto.createHash("sha256")
    .update(`${JSON.stringify(value, null, 2)}\n`)
    .digest("hex");
}

function assert(condition, message) {
  if (!condition)
    throw new Error(message);
}
