const activityPresentationTypes = new Set([
  "ActiveVirtualCamera",
  "AdvNpcFaceToPlayer",
  "AnimSetParameter",
  "BlockInputController",
  "EnableBillboard",
  "EnablePlayerPlayIdleShow",
  "GlobalTimeSlow",
  "LockPlayerControl",
  "PlayAndWaitRogueSimpleTalk",
  "PlayRogueSimpleTalk",
  "PlayScreenTransfer",
  "RadialBlurEffect",
  "SetEntityVisible",
  "SetGameplayBGMEmotionState",
  "SetLocalPlayerDitherAlpha",
  "ShowRogueTalkBg",
  "ShowRogueTalkUI",
  "ShowTransitionLoadingUI",
  "ShowUI",
  "SwitchCharacterAnchor",
  "SwitchCharacterAnchorV2",
  "TriggerAnimState",
  "TriggerSound",
  "UnLockPlayerControl",
  "VCameraConfigChange",
  "WaitPerformanceEnd",
  "WaitRogueSimpleTalkFinish",
]);

const combatPresentationTypes = new Set([
  "AddBuffPerform",
  "DebugLog",
  "ModifierAttachEffect",
  "TriggerAnimState",
  "WaitSecond",
]);

const existingCombatOperations = new Map([
  ["AddModifier", ["RuleOperationTemplate::ApplyEffect"]],
  ["RemoveModifier", ["RuleOperationTemplate::RemoveEffect"]],
  ["RemoveSelfModifier", ["RuleOperationTemplate::RemoveEffect"]],
  ["RemoveEffect", ["RuleOperationTemplate::RemoveEffect"]],
  ["SetDynamicValue", ["RuleOperationTemplate::SetSlot"]],
  ["DefineDynamicValue", ["StateSlotDef"]],
  ["SetModifierDynamicValue", ["RuleOperationTemplate::ModifyStateSlot"]],
  ["SetDynamicValueByProperty", ["RuleOperationTemplate::SetSlot", "ValueExpr::QueryStat"]],
  ["SetDynamicValueByCopying", ["RuleOperationTemplate::SetSlot", "ValueExpr::Slot"]],
  ["SetDynamicValueByVariateType", ["RuleOperationTemplate::SetSlot", "ValueExpr::Slot"]],
  ["SetDynamicValueByDamageDataProperty", [
    "RuleOperationTemplate::SetSlot", "ValueExpr::ReadEventProperty",
  ]],
  ["SetDynamicValueByHealDataProperty", [
    "RuleOperationTemplate::SetSlot", "ValueExpr::ReadEventProperty",
  ]],
  ["SetDynamicValueByShield", ["RuleOperationTemplate::SetSlot", "ValueExpr::QueryShield"]],
  ["SetDynamicValueByModifierValue", [
    "RuleOperationTemplate::SetSlot", "ValueExpr::QueryEffectStacks",
  ]],
  ["SetDynamicValueByCurrentBP", ["RuleOperationTemplate::SetSlot", "ValueExpr::ReadResource"]],
  ["SetDynamicValueByMaxBP", ["RuleOperationTemplate::SetSlot", "ValueExpr::ReadResource"]],
  ["SetDynamicValueByBreakBaseDamage", [
    "RuleOperationTemplate::SetSlot", "ValueExpr::QueryFormulaStage",
  ]],
  ["SetDynamicValueByHardLevelProperty", [
    "RuleOperationTemplate::SetSlot", "ValueExpr::AbilityParameter",
  ]],
  ["DamageByAttackProperty", ["RuleOperationTemplate::Damage"]],
  ["HealHP", ["RuleOperationTemplate::Heal"]],
  ["LoseHPByRatio", ["RuleOperationTemplate::ConsumeHp"]],
  ["ModifyActionDelay", [
    "RuleOperationTemplate::AdvanceAction",
    "RuleOperationTemplate::DelayAction",
  ]],
  ["ModifyDamageData", [
    "RuleOperationTemplate::ProposeReplacement", "ValueExpr::ReadEventProperty",
    "TriggerPhase::Replace",
  ]],
  ["ModifyTeamBoostPoint", ["RuleOperationTemplate::ModifyResource"]],
  ["RemoveShield", ["RuleOperationTemplate::RemoveShield"]],
  ["InitShield", ["RuleOperationTemplate::Shield"]],
  ["StackProperty", ["RuleOperationTemplate::ApplyEffect", "BattleRuleDefinition"]],
  ["SetResilience", ["RuleOperationTemplate::ApplyEffect", "BattleRuleDefinition"]],
  ["TriggerEffect", ["RuleOperationTemplate::ApplyEffect"]],
  ["TriggerEffectList", ["RuleOperationTemplate::ApplyEffect"]],
  ["Remodifier", ["RuleOperationTemplate::RemoveEffect", "RuleOperationTemplate::ApplyEffect"]],
  ["ReduceStanceRatio", ["RuleOperationTemplate::ReduceToughness"]],
  ["StackWeakness", ["RuleOperationTemplate::AddWeakness"]],
  ["AttachSkillTypeDisable", ["RuleOperationTemplate::ApplyEffect"]],
  ["HitDamageSplit", ["RuleOperationTemplate::ProposeReplacement", "TriggerPhase::Replace"]],
  ["TriggerModifierCustomEvent", ["RuleOperationTemplate::EmitRuleEvent"]],
  ["Retarget", ["RuleUnitSelector::with_candidate_union"]],
  ["PredicateTaskList", ["ProgramStep::If"]],
  ["ByAnd", ["ConditionExpr::All"]],
  ["ByAny", ["ConditionExpr::Any"]],
  ["ByCompareDynamicValue", ["ConditionExpr::Compare", "ValueExpr::Slot"]],
  ["ByTargetTeam", ["RuleUnitSelector"]],
  ["ByIsContainModifier", ["ConditionExpr::EffectExists"]],
  ["TargetAlias", ["RuleUnitSelector"]],
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

export function sourceDomain(program) {
  if (["MetadataOnly", "ExcludedWithProof"].includes(program.execution_disposition))
    return "Metadata";
  return program.runtime_domain === "Battle" ? "Combat" : "Activity";
}

export function classifySourceType(qualifiedName) {
  const name = shortName(qualifiedName);
  if (qualifiedName.startsWith("Structure:")) return "Structure";
  if (name.startsWith("Target") || name.includes("Selector")) return "Selector";
  if (name.startsWith("By") || name.includes("Predicate")) return "Condition";
  if (/(?:DynamicValue|SharedFloat|SharedInt|SharedString|AttackData)$/.test(name))
    return "State";
  if (/(?:Sequence|Loop|Switch|RandomCase|TaskList)/.test(name)) return "ControlFlow";
  return "Operation";
}

export function mapOperation(qualifiedName, domain) {
  const name = shortName(qualifiedName);
  const kind = classifySourceType(qualifiedName);
  if (domain === "Metadata") return nonAuthoritative("metadata.source-layout-only");
  if (qualifiedName === "None") return nonAuthoritative("source.null-type-token");
  if (qualifiedName === "Structure:BakeInfoLayouts"
      || qualifiedName === "Structure:DialogueList"
      || qualifiedName === "Structure:DialogueType")
    return nonAuthoritative("presentation.source-structure");
  if (domain === "Activity" && activityPresentationTypes.has(name))
    return nonAuthoritative("presentation.activity-source-operation");
  if (domain === "Combat" && combatPresentationTypes.has(name))
    return nonAuthoritative("presentation.combat-perform-operation");
  if (domain === "Combat" && existingCombatOperations.has(name))
    return existing(existingCombatOperations.get(name));
  if (domain === "Combat" && kind === "Selector")
    return existing(["RuleUnitSelector", "RuleUnitSelector::with_candidate_union"]);
  if (domain === "Combat" && kind === "Condition")
    return existing(["ConditionExpr", "EventFilter", "ValueExpr"]);
  if (domain === "Combat" && kind === "State")
    return existing(["StateSlotDef", "RuleEventFacts", "EventFilter", "ValueExpr"]);
  if (domain === "Combat" && kind === "ControlFlow")
    return existing(["ProgramStep", "bounded programs"]);
  if (domain === "Combat")
    return missing(`mode.divergent-universe.combat.operation.${slug(name)}`,
      ["RuleOperationTemplate"]);

  if (["SetDynamicValue", "SetDynamicValueByCustomName"].includes(name))
    return existing(["ActivityOperation::SetSlot", "ActivityExpression"]);
  if (qualifiedName === "Structure:OptionList" || name === "PlayRogueOptionTalk")
    return existing([
      "ActivityOperation::Offer", "ActivityOptionDefinition",
    ]);
  if (["WaitCustomString", "TriggerCustomString", "WaitEntityEventV2",
    "TriggerEntityEventV2", "WaitDialogueEvent"].includes(name))
    return nonAuthoritative("presentation.source-graph-signal-sequencing");
  if (["FinishLevelGraph", "SetRogueRoomFinish", "SetAllRogueDoorState",
    "RogueDoorSetGotoInfo", "RogueTournEnterNextRoom", "RogueTournFinish"].includes(name))
    return existing([
      "ActivityOperation::Traverse", "ActivityOperation::Relocate",
      "ActivityOperation::Terminal",
    ]);
  if (kind === "Selector")
    return nonAuthoritative("presentation.source-scene-entity-selector");
  if (kind === "Condition")
    return name === "WaitPredicateSucc"
      ? nonAuthoritative("presentation.empty-source-wait-boundary")
      : existing(["ActivityCondition", "ActivityExpression"]);
  if (kind === "State")
    return existing([
      "ActivityStateDefinition", "ActivityExpression",
    ]);
  if (kind === "ControlFlow")
    return existing([
      "ActivityOperation::Conditional", "ActivityProgramDefinition",
    ]);
  return missing(`mode.divergent-universe.activity.operation.${slug(name)}`,
    ["ActivityOperation"]);
}

export function mapExpression(domain, dynamic, authoritative = true) {
  if (!authoritative) return nonAuthoritative("presentation.expression-shape");
  if (!dynamic)
    return existing(domain === "Combat"
      ? ["ValueExpr::Literal"] : ["ActivityExpression::Literal"]);
  return missing("shared.version-4.4-postfix-opcode-semantics",
    domain === "Combat" ? ["ValueExpr"] : ["ActivityExpression"]);
}

export function mapTypedCondition(domain, qualifiedName, authoritative = true) {
  if (!authoritative) return nonAuthoritative("presentation.condition-shape");
  const name = shortName(qualifiedName);
  if (domain === "Combat" && existingCombatOperations.has(name))
    return existing(existingCombatOperations.get(name));
  if (domain === "Combat")
    return existing(["ConditionExpr", "EventFilter", "ValueExpr"]);
  return name === "WaitPredicateSucc"
    ? nonAuthoritative("presentation.empty-source-wait-boundary")
    : existing(["ActivityCondition", "ActivityExpression"]);
}

export function mapSelector(domain, qualifiedName, authoritative = true) {
  if (!authoritative) return nonAuthoritative("presentation.selector-shape");
  if (domain === "Combat")
    return existing(["RuleUnitSelector", "RuleUnitSelector::with_candidate_union"]);
  return nonAuthoritative("presentation.source-scene-entity-selector");
}

export function mapTrigger(domain, trigger, authoritative = true) {
  if (!authoritative) return nonAuthoritative("presentation.trigger-shape");
  if (domain === "Combat" && existingCombatTriggers.has(trigger))
    return existing([existingCombatTriggers.get(trigger)]);
  if (domain === "Combat")
    return existing([
      "RuleEventPoint", "TriggerPhase", "EventFilter", "ConditionExpr",
    ]);
  if (["AcceptedModeDecision", "ModeOrRoomLifecycle",
    "AcceptedExternalAdventureResult"].includes(trigger))
    return existing(["ActivityProgramDefinition", "ActivityInteractionBinding"]);
  return missing(`activity.trigger.${slug(trigger)}`, [
    "ActivityProgramDefinition", "ActivityCommand",
  ]);
}

export function mapState(domain, stateKind, authoritative = true) {
  if (!authoritative) return nonAuthoritative("presentation.state-shape");
  if (domain === "Combat")
    return existing(["StateSlotDef", "RuleValue", "ValueExpr"]);
  return existing(["ActivityStateDefinition", "ActivityValue", "ActivityExpression"]);
}

export function mapLifecycle(domain, hook, authoritative = true) {
  if (!authoritative) return nonAuthoritative("presentation.lifecycle-shape");
  if (domain === "Combat" && existingCombatTriggers.has(hook))
    return existing([existingCombatTriggers.get(hook)]);
  if (domain === "Combat" && hook === "BattleScopedSourceProgram")
    return existing(["BattleRuleScope", "SlotResetPoint", "BattleRuleDefinition"]);
  if (domain === "Combat")
    return existing([
      "TriggerDef", "RuleEventPoint", "TriggerPhase", "SlotResetPoint",
    ]);
  if (["CrossBattleDecisionLifecycle", "CrossBattleStateLifecycle",
    "ExternalOutcomeSettlement"].includes(hook))
    return existing(["ActivityScope", "ActivitySnapshotBoundary", "ActivityProgramDefinition"]);
  return existing([
    "GraphActivityNodeProgram", "ActivityInteractionBinding",
    "ActivitySnapshotBoundary", "ActivityProgramDefinition",
  ]);
}

export function mapRecordShape(program) {
  if (["MetadataOnly", "ExcludedWithProof"].includes(program.execution_disposition))
    return nonAuthoritative("metadata.source-layout-only");
  return program.runtime_domain === "Battle"
    ? existing(["BattleRuleDefinition", "RuleUnitSelector"])
    : existing(["ActivityProgramDefinition", "ActivityStateDefinition"]);
}

export function isPresentationOperation(qualifiedName, domain) {
  const name = shortName(qualifiedName);
  return domain === "Metadata"
    || domain === "Activity" && activityPresentationTypes.has(name)
    || domain === "Combat" && combatPresentationTypes.has(name)
    || qualifiedName.startsWith("Structure:Dialogue")
    || qualifiedName === "Structure:BakeInfoLayouts";
}

function shortName(qualifiedName) {
  return qualifiedName.slice(Math.max(
    qualifiedName.lastIndexOf("."), qualifiedName.lastIndexOf(":"),
  ) + 1);
}

function slug(value) {
  return value.replace(/([a-z0-9])([A-Z])/g, "$1-$2")
    .replace(/[^A-Za-z0-9]+/g, "-").replace(/^-|-$/g, "").toLowerCase();
}

function existing(support) {
  return { disposition: "ExistingPrimitive", existing_support: support, missing_capability: null };
}

function missing(capability, support) {
  return { disposition: "MissingCapability", existing_support: support, missing_capability: capability };
}

function nonAuthoritative(capability) {
  return { disposition: "NonAuthoritative", existing_support: [], missing_capability: capability };
}
