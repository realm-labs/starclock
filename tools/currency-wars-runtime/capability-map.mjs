const presentationPattern = /(?:Anim|Camera|CutIn|Effect|FootIK|Guide|Hint|LookAt|Material|Model|RadialBlur|Render|Sound|Timeline|Tutorial|UI|VFX|Visible|Voice|Wait)/i;
const combatPresentationTypes = new Set([
  "AlignTargetToTeamCenter",
  "BattleAudioSwitch",
  "ClearEntityDamageText",
  "ClearEntityFollowAttachPoint",
  "DamagePerformFinish",
  "FireMultiProjectiles",
  "FireProjectile",
  "FrameCaptureIfNeed",
  "HideLevelStage",
  "LevelAudioState",
  "MoveStageOnTargetForward",
  "MoveTeam",
  "MoveToTargetList",
  "MoveToTargetPosition",
  "SetAttachmentScale",
  "SetAttachmentVisibility",
  "SetEnergyBarState",
  "SetEntityFollowAttachPoint",
  "SetEntityPosition",
  "SetHeadButtonEff",
  "SetMonsterEnergyBarState",
  "SetNoShadowCaster",
  "SetShenJunActionBar",
  "SetSkillButtonAdditionalStatus",
  "SetSummonerEnergyBarState",
  "SetTargetCustomAssetPreloadState",
  "SetTeamFormation",
  "SetTeamRootOffset",
  "SetUltraSkillAssetPreload",
  "ShowAttackTime",
  "ShowBattleSkillEnhanced",
  "ShowBossInfoBar",
  "ShowEntityFloatMessage",
  "ShowSkillTextDialog",
  "SkillPerformFinish",
  "StartAim",
  "StopAim",
  "TriggerAutoLayoutTransferPerform",
  "TryStartConnectUltraSkillFrameCapture",
]);

const existingCombatTypes = new Map([
  ["AddModifier", ["RuleOperationTemplate::ApplyEffect"]],
  ["RemoveModifier", ["RuleOperationTemplate::RemoveEffect"]],
  ["RemoveSelfModifier", ["RuleOperationTemplate::RemoveEffect"]],
  ["SetDynamicValue", ["RuleOperationTemplate::SetSlot"]],
  ["DefineDynamicValue", ["StateSlotDef"]],
  ["SetModifierDynamicValue", ["RuleOperationTemplate::ModifyStateSlot"]],
  ["DamageByAttackProperty", ["RuleOperationTemplate::Damage"]],
  ["HealByAttackProperty", ["RuleOperationTemplate::Heal"]],
  ["ModifySPNew", ["RuleOperationTemplate::ModifyResource"]],
  ["TurnInsertAbility", ["RuleOperationTemplate::QueueAction"]],
  ["TriggerAbility", ["RuleOperationTemplate::QueueAction"]],
  ["AdvanceAction", ["RuleOperationTemplate::AdvanceAction"]],
  ["DelayAction", ["RuleOperationTemplate::DelayAction"]],
  ["AddWeakness", ["RuleOperationTemplate::AddWeakness"]],
  ["RemoveWeakness", ["RuleOperationTemplate::RemoveWeakness"]],
  ["LockHP", ["EffectRuntimeDefinition::with_hp_floor"]],
  ["LockTargetHP", ["EffectRuntimeDefinition::with_hp_floor", "RuleOperationTemplate::ApplyEffect"]],
  ["UnlockTargetHP", ["RuleOperationTemplate::RemoveEffect"]],
  ["ModifyTeamBoostPointMax", ["RuleOperationTemplate::ModifySkillPointMaximum"]],
  ["SetHP", ["ConditionExpr::Compare", "RuleOperationTemplate::ConsumeHp", "RuleOperationTemplate::Heal"]],
  ["ForceKill", ["RuleOperationTemplate::TrueDamage", "RuleOperationTemplate::Despawn"]],
  ["AddGridFightDropData", ["RuleOperationTemplate::EmitRuleEvent", "battle-result metric projection"]],
  ["PredicateTaskList", ["ProgramStep::If"]],
  ["ByAnd", ["ConditionExpr::All"]],
  ["ByAny", ["ConditionExpr::Any"]],
  ["ByCompareDynamicValue", ["ConditionExpr::Compare", "ValueExpr::Slot"]],
  ["ByTargetAliveState", ["ConditionExpr::LifePresence"]],
  ["ByIsContainModifier", ["ConditionExpr::EffectExists"]],
]);

const existingCombatTriggers = new Map([
  ["OnEnterBattle", "RuleEventPoint::BattleStarted"],
  ["OnListenTurnBegin", "RuleEventPoint::TurnStarted"],
  ["OnListenTurnEnd", "RuleEventPoint::TurnEnded"],
  ["OnBeforeAction", "RuleEventPoint::ActionStarted + TriggerPhase::Before"],
  ["OnAfterAction", "RuleEventPoint::ActionResolved + TriggerPhase::AfterAction"],
  ["OnListenAfterAction", "RuleEventPoint::ActionResolved + TriggerPhase::AfterAction"],
  ["OnBeforeHit", "RuleEventPoint::HitStarted + TriggerPhase::Before"],
  ["OnAfterHit", "RuleEventPoint::HitEnded + TriggerPhase::AfterEvent"],
  ["OnHPChange", "RuleEventPoint::HpChanged"],
  ["OnListenHPChange", "RuleEventPoint::HpChanged"],
  ["OnShieldChange", "RuleEventPoint::ShieldChanged"],
  ["OnListenShieldChange", "RuleEventPoint::ShieldChanged"],
  ["OnBeingBreak", "RuleEventPoint::WeaknessBroken"],
  ["OnListenBreak", "RuleEventPoint::WeaknessBroken"],
  ["OnModifierAdd", "RuleEventPoint::EffectApplied"],
  ["OnListenModifierAdd", "RuleEventPoint::EffectApplied"],
  ["OnModifierRemove", "RuleEventPoint::EffectRemoved"],
  ["OnListenModifierRemove", "RuleEventPoint::EffectRemoved"],
  ["OnListenCharacterDie", "RuleEventPoint::UnitDefeated"],
  ["OnListenRevive", "RuleEventPoint::UnitRevived"],
]);

const existingActivityTypes = new Map([
  ["AddModifier", ["ActivityOperation::AddModifier"]],
  ["DefineDynamicValue", ["ActivityStateDefinition"]],
  ["SetDynamicValue", ["ActivityOperation::SetSlot"]],
  ["PredicateTaskList", ["ActivityOperation::Conditional"]],
  ["ByAny", ["ActivityCondition::Any"]],
  ["ByAnd", ["ActivityCondition::All"]],
  ["ByCompareDynamicValue", ["ActivityCondition::Compare", "ActivityExpression::Slot"]],
  ["ByCompareWaveCount", ["ActivityCondition::Compare", "ActivityExpression::Slot"]],
  ["ByGridFightHasTrait", ["ActivityCondition::OrderedIdSetContains"]],
  ["ByIsContainModifier", ["ActivityExpression::ModifierStacks", "ActivityCondition::Compare"]],
  ["ByTargetAliveState", ["ActivityCondition::ParticipantDefeated", "ActivityCondition::Not"]],
  ["LoopExecuteTaskList", ["ActivityProgramDefinition bounded authoring expansion"]],
  ["GenericSwitchCase", ["ActivityOperation::Conditional"]],
]);

export function sourceDomain(program) {
  if (program.target_execution === "MetadataOnly")
    return "Metadata";
  if (program.capability === "role-build-and-roster")
    return "Build";
  return program.scope === "CrossBattleActivity" ? "Activity" : "Combat";
}

export function classifyConfigurationType(qualifiedName) {
  const name = shortName(qualifiedName);
  if (presentationPattern.test(name) || combatPresentationTypes.has(name))
    return "Presentation";
  if (name.startsWith("Target") || name.includes("TargetSelector"))
    return "Selector";
  if (name.startsWith("By") || name.includes("Predicate"))
    return "Condition";
  if (/(?:DynamicValue|Property|ModifierValue|AttackData|Config|Info)$/.test(name))
    return "State";
  if (/(?:Sequence|Loop|Switch|Random|TaskList)/.test(name))
    return "ControlFlow";
  return "Operation";
}

export function mapConfigurationType(qualifiedName, domain) {
  const name = shortName(qualifiedName);
  const kind = classifyConfigurationType(qualifiedName);
  if (kind === "Presentation")
    return nonAuthoritative("presentation.configuration-shape-filter");
  if (domain === "Combat" && existingCombatTypes.has(name))
    return existing(existingCombatTypes.get(name));
  if (domain === "Combat" && kind === "Selector")
    return existing(["RuleUnitSelector", "RuleUnitSelector::with_candidate_union"]);
  if (domain === "Combat" && kind === "Condition")
    return existing(["ConditionExpr", "EventFilter", "ValueExpr"]);
  if (domain === "Combat" && kind === "State")
    return existing(["StateSlotDef", "ValueExpr", "EffectRuntimeDefinition"]);
  if (domain === "Combat" && kind === "ControlFlow")
    return existing(["ProgramStep::If", "ProgramStep::ForEach", "bounded programs"]);
  if (domain === "Combat")
    return existing([
      "RuleOperationTemplate", "TriggerPhase::Replace",
      "EffectRuntimeDefinition", "battle-result metric projection",
    ]);
  if (domain === "Build")
    return existing([
      "CombatantBuildSpec", "BuildContributionDefinition", "BuildPatch", "BattleRuleDefinition",
    ]);
  if (domain === "Activity" && existingActivityTypes.has(name))
    return existing(existingActivityTypes.get(name));
  if (domain === "Activity" && kind === "Selector")
    return existing(["ActivityExpression collection reads", "ParticipantPool"]);
  if (domain === "Activity" && kind === "Condition")
    return existing(["ActivityCondition", "ActivityComparison"]);
  if (domain === "Activity" && kind === "State")
    return existing(["ActivityStateDefinition", "ActivityOperation replacement mutations"]);
  if (domain === "Activity" && kind === "ControlFlow")
    return existing(["ActivityOperation::Conditional", "ActivityRandomPolicies"]);
  if (domain === "Activity")
    return existing(["ActivityOperation", "ActivityInteractionBinding"]);
  return nonAuthoritative("metadata.no-executable-shape");
}

export function mapExpression(domain, dynamic, authoritative = true) {
  if (!authoritative)
    return nonAuthoritative("presentation.configuration-shape-filter");
  if (!dynamic)
    return existing(domain === "Activity"
      ? ["ActivityExpression::Literal"]
      : domain === "Build" ? ["BuildSpec parameter"] : ["ValueExpr::Literal"]);
  const support = domain === "Activity"
    ? ["ActivityExpression"]
    : domain === "Build" ? ["BuildSpec", "BuildPatch"] : ["ValueExpr"];
  return missing("shared.version-4.4-postfix-opcode-semantics", support);
}

export function mapSelector(domain, authoritative = true) {
  if (!authoritative)
    return nonAuthoritative("presentation.configuration-shape-filter");
  if (domain === "Combat")
    return existing(["RuleUnitSelector", "RuleUnitSelector::with_candidate_union"]);
  if (domain === "Build")
    return existing([
      "BuildContributionApplicability", "CombatantBuildSpec::with_contributions",
    ]);
  return existing(["ActivityExpression collection reads", "ParticipantPool"]);
}

export function mapTrigger(domain, trigger, authoritative = true) {
  if (!authoritative)
    return nonAuthoritative("presentation.configuration-shape-filter");
  if (domain === "Combat" && existingCombatTriggers.has(trigger))
    return existing([existingCombatTriggers.get(trigger)]);
  if (domain === "Combat")
    return existing(["RuleEventPoint", "TriggerPhase", "EventFilter", "ConditionExpr"]);
  if (domain === "Build")
    return existing(["BuildContributionDefinition", "BuildPatch", "BattleRuleDefinition"]);
  return existing(["ActivityProgramDefinition", "GraphActivityNodeProgram"]);
}

export function mapState(domain, stateKind, authoritative = true) {
  if (!authoritative)
    return nonAuthoritative("presentation.configuration-shape-filter");
  if (domain === "Combat") {
    if (stateKind === "ModifierDefinition")
      return existing(["EffectDefinition", "BattleRuleDefinition"]);
    return existing(["StateSlotDef", "ValueExpr", "RuleEventFacts"]);
  }
  if (domain === "Build")
    return existing(["CombatantBuildSpec", "BuildContributionDefinition", "BuildPatch"]);
  if (["ModifierDefinition", "DynamicValueDefinition"].includes(stateKind))
    return existing(["ActivityModifierDefinition", "ActivityStateDefinition"]);
  return existing(["ActivityStateDefinition", "ActivityOperation replacement mutations"]);
}

export function mapLifecycle(domain, hook, authoritative = true) {
  if (!authoritative)
    return nonAuthoritative("presentation.configuration-shape-filter");
  if (domain === "Combat" && existingCombatTriggers.has(hook))
    return existing([existingCombatTriggers.get(hook)]);
  if (domain === "Combat")
    return existing(["TriggerDef", "RuleEventPoint", "TriggerPhase", "SlotResetPoint"]);
  if (domain === "Build")
    return existing(["BuildContributionDefinition", "BuildPatch", "BattleRuleDefinition"]);
  if (["OnInitSequece", "OnStartSequece"].includes(hook))
    return existing(["GraphActivityNodeProgram", "ActivityProgramDefinition"]);
  return existing([
    "GraphActivityNodeProgram", "ActivitySnapshotBoundary", "SlotResetPoint",
  ]);
}

export function mapRecordShape(program) {
  if (program.target_execution === "MetadataOnly")
    return nonAuthoritative("metadata.no-executable-shape");
  if (program.capability === "role-build-and-roster")
    return existing(["CurrencyWarsBuildCatalog", "ResolvedCombatantSpec"]);
  if (program.scope === "CrossBattleActivity")
    return existing(["ActivityProgramDefinition", "ActivityStateDefinition"]);
  return existing(["BattleRuleDefinition", "RuleUnitSelector"]);
}

function shortName(qualifiedName) {
  return qualifiedName.slice(qualifiedName.lastIndexOf(".") + 1);
}

function existing(support) {
  return {
    disposition: "ExistingPrimitive",
    existing_support: support,
    missing_capability: null,
  };
}

function missing(capability, support) {
  return {
    disposition: "MissingCapability",
    existing_support: support,
    missing_capability: capability,
  };
}

function nonAuthoritative(capability) {
  return {
    disposition: "NonAuthoritative",
    existing_support: [],
    missing_capability: capability,
  };
}
