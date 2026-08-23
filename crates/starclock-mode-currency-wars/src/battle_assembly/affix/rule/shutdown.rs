//! Attack-count damage overrides for shutdown and elemental Curbed Affixes.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use starclock_combat::{
    DispelCategory, DurationClock, EffectCategory, EffectDefinitionId, EffectRuntimeTemplate,
    EffectStackPolicy, EffectTickPhase, ModifierDefinitionId, ModifierStackingGroupId,
    ResolvedCombatantSpec, Scalar, SelectorId, SourceDefinitionId,
    catalog::{
        action::AbilityTag,
        builder::CombatCatalogBuilder,
        definition::{EffectDefinition, ProgramDefinition, SelectorDefinition},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
    formula::model::CombatElement,
    modifier::model::{
        FormulaPurpose, FormulaStage, FormulaSubject, ModifierAggregation, ModifierDefinition,
        ModifierFilter, ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{
        ConditionExpr, EventFilter, OnceScope, RuleEffectChancePolicy, RuleEventPoint,
        RuleOperationTemplate, RuleSource, RuleValue, SourceClass, TriggerDef, ValueExpr,
    },
};

use crate::{
    CurrencyWarsContributionSnapshot, CurrencyWarsEnemyAffixBehavior,
    CurrencyWarsEnemyAffixSemantic, CurrencyWarsRoleId,
    battle_assembly::{
        CurrencyWarsBattleAssemblyError, CurrencyWarsBattleResources,
        combatant_overlay::attach_source_tag, debug_error, error,
    },
};

use super::{
    actor, definition_raw, integer_parameter, integer_value, operation, players,
    program_definition, program_id_for, trigger,
};

const ELEMENT_SELECTOR_BASE: u32 = 0x7d80_0060;
const ELEMENT_TAG_BASE: u32 = 0x7d80_0070;
const BACKEND_SELECTOR: u32 = 0x7d80_0078;
const DAMAGE_PURPOSES: [FormulaPurpose; 7] = [
    FormulaPurpose::OrdinaryDamage,
    FormulaPurpose::Dot,
    FormulaPurpose::Break,
    FormulaPurpose::SuperBreak,
    FormulaPurpose::AdditionalDamage,
    FormulaPurpose::JointDamage,
    FormulaPurpose::ElationDamage,
];
const ELEMENTS: [CombatElement; 7] = [
    CombatElement::Physical,
    CombatElement::Fire,
    CombatElement::Ice,
    CombatElement::Lightning,
    CombatElement::Wind,
    CombatElement::Quantum,
    CombatElement::Imaginary,
];

pub(super) fn tag_combatants(
    resources: &CurrencyWarsBattleResources,
    snapshot: &CurrencyWarsContributionSnapshot,
    combatants: &mut BTreeMap<CurrencyWarsRoleId, ResolvedCombatantSpec>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    if !snapshot
        .enemy_affix_behaviors
        .iter()
        .any(|behavior| element_for(behavior.semantic).is_some())
    {
        return Ok(());
    }
    for role in &snapshot.roles {
        let Some(combatant) = combatants.get(&role.role.id) else {
            continue;
        };
        let element = resources.role_element(role.role.id).ok_or_else(|| {
            error(&format!(
                "Currency Wars Curbed Affix released element is missing for role {}",
                role.role.id.get()
            ))
        })?;
        let source = RuleSource::new(
            element_tag(element)?,
            SourceClass::Mode,
            Vec::new(),
            tag_digest(snapshot.digest.bytes(), element),
        );
        let replacement = attach_source_tag(
            combatant,
            source,
            b"starclock.currency-wars.enemy-affix-element-tag.v1",
            tag_digest(snapshot.digest.bytes(), element),
        )?;
        combatants.insert(role.role.id, replacement);
    }
    Ok(())
}

pub(super) fn compile(
    builder: &mut CombatCatalogBuilder,
    behavior: &CurrencyWarsEnemyAffixBehavior,
    programs: &mut Vec<ProgramDefinition>,
    triggers: &mut Vec<TriggerDef>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    let eligible = if behavior.semantic == CurrencyWarsEnemyAffixSemantic::BackendShutdown {
        backend()
    } else {
        element_for(behavior.semantic)
            .map(element_selector)
            .transpose()?
            .unwrap_or_else(players)
    };
    let attacks = integer_parameter(behavior, 0)?;
    let effect = effect_id(behavior, 20)?;
    let group = group_id(behavior, 21)?;
    builder.add_modifier_group(ModifierStackingGroup {
        id: group,
        aggregation: ModifierAggregation::Maximum,
        comparator: None,
    });
    let mut modifiers = Vec::new();
    for (index, purpose) in DAMAGE_PURPOSES.into_iter().enumerate() {
        let id = modifier_id(
            behavior,
            22_u32
                .checked_add(u32::try_from(index).map_err(debug_error)?)
                .ok_or_else(|| error("Currency Wars shutdown modifier ID overflow"))?,
        )?;
        builder.add_modifier(ModifierDefinition {
            id,
            stat: StatKind::Atk,
            stage: FormulaStage::DamageOverride,
            purpose,
            value: ValueExpr::Literal(RuleValue::Scalar(
                Scalar::checked_from_integer(1).expect("one damage is a valid scalar"),
            )),
            stacking_group: group,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::DamageOverride,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: Box::new([ModifierFilter::FormulaSubject(FormulaSubject::Source)]),
        });
        modifiers.push(id);
    }
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Debuff,
        DispelCategory::NonDispellable,
        u16::try_from(attacks).map_err(debug_error)?,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .ok_or_else(|| error("Currency Wars shutdown effect is invalid"))?;
    builder.add_effect(
        EffectDefinition::new(effect, Vec::new(), modifiers).with_runtime_template(runtime),
    );
    let apply = program_id_for(behavior, 1)?;
    let spend = program_id_for(behavior, 2)?;
    programs.push(program_definition(
        apply,
        Vec::new(),
        vec![effect],
        vec![operation(RuleOperationTemplate::ApplyEffect {
            selector: eligible,
            effect,
            stacks: integer_value(attacks),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        })],
    ));
    programs.push(program_definition(
        spend,
        Vec::new(),
        vec![effect],
        vec![operation(RuleOperationTemplate::AdjustEffectStacks {
            selector: actor(),
            effect,
            delta: ValueExpr::Literal(RuleValue::Integer(-1)),
        })],
    ));
    triggers.push(trigger(
        behavior,
        10,
        RuleEventPoint::BattleStarted,
        EventFilter::default(),
        ConditionExpr::Literal(true),
        OnceScope::Battle,
        apply,
    )?);
    triggers.push(trigger(
        behavior,
        11,
        RuleEventPoint::ActionResolved,
        EventFilter {
            actor_selector: Some(eligible),
            ability_tag: Some(AbilityTag::Attack),
            ..EventFilter::default()
        },
        ConditionExpr::EffectExists {
            selector: actor(),
            effect,
        },
        OnceScope::Action,
        spend,
    )?);
    Ok(())
}

pub(super) fn selectors() -> Result<Vec<SelectorDefinition>, CurrencyWarsBattleAssemblyError> {
    let mut selectors = ELEMENTS
        .into_iter()
        .map(|element| {
            let selector = RuleUnitSelector::new(
                RuleSelectorOrigin::Owner,
                RuleSelectorSide::Same,
                RuleLifePredicate::Alive,
                RulePresencePredicate::Present,
                RuleSelectorReference::CurrentState,
                RuleSelectorOrdering::Formation,
                0,
                32,
                RuleEmptyPoolPolicy::NoOp,
                RuleSelectorChoice::All,
                None,
                false,
            )
            .ok_or_else(|| error("Currency Wars elemental selector is invalid"))?
            .with_predicates(vec![RuleSelectorPredicate::HasTag(element_tag(element)?)]);
            Ok(SelectorDefinition::new(element_selector(element)?).with_rule_units(selector))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let backend_units = RuleUnitSelector::new(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Untargetable,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        0,
        32,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::All,
        None,
        false,
    )
    .ok_or_else(|| error("Currency Wars backend selector is invalid"))?
    .with_predicates(vec![RuleSelectorPredicate::FormationRange {
        minimum: 16,
        maximum: u8::MAX,
    }]);
    selectors.push(SelectorDefinition::new(backend()).with_rule_units(backend_units));
    Ok(selectors)
}

pub(super) fn selector_ids() -> Vec<SelectorId> {
    let mut selectors = ELEMENTS
        .into_iter()
        .map(|element| element_selector(element).expect("reserved element selector is non-zero"))
        .collect::<Vec<_>>();
    selectors.push(backend());
    selectors
}

fn backend() -> SelectorId {
    SelectorId::new(BACKEND_SELECTOR).expect("reserved backend selector is non-zero")
}

const fn element_for(semantic: CurrencyWarsEnemyAffixSemantic) -> Option<CombatElement> {
    match semantic {
        CurrencyWarsEnemyAffixSemantic::CurbedPhysical => Some(CombatElement::Physical),
        CurrencyWarsEnemyAffixSemantic::CurbedFire => Some(CombatElement::Fire),
        CurrencyWarsEnemyAffixSemantic::CurbedIce => Some(CombatElement::Ice),
        CurrencyWarsEnemyAffixSemantic::CurbedLightning => Some(CombatElement::Lightning),
        CurrencyWarsEnemyAffixSemantic::CurbedWind => Some(CombatElement::Wind),
        CurrencyWarsEnemyAffixSemantic::CurbedQuantum => Some(CombatElement::Quantum),
        CurrencyWarsEnemyAffixSemantic::CurbedImaginary => Some(CombatElement::Imaginary),
        _ => None,
    }
}

fn element_index(element: CombatElement) -> u32 {
    match element {
        CombatElement::Physical => 0,
        CombatElement::Fire => 1,
        CombatElement::Ice => 2,
        CombatElement::Lightning => 3,
        CombatElement::Wind => 4,
        CombatElement::Quantum => 5,
        CombatElement::Imaginary => 6,
    }
}

fn element_selector(element: CombatElement) -> Result<SelectorId, CurrencyWarsBattleAssemblyError> {
    ELEMENT_SELECTOR_BASE
        .checked_add(element_index(element))
        .and_then(SelectorId::new)
        .ok_or_else(|| error("Currency Wars element selector ID is invalid"))
}

fn element_tag(
    element: CombatElement,
) -> Result<SourceDefinitionId, CurrencyWarsBattleAssemblyError> {
    ELEMENT_TAG_BASE
        .checked_add(element_index(element))
        .and_then(SourceDefinitionId::new)
        .ok_or_else(|| error("Currency Wars element source tag ID is invalid"))
}

fn effect_id(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    offset: u32,
) -> Result<EffectDefinitionId, CurrencyWarsBattleAssemblyError> {
    EffectDefinitionId::new(definition_raw(behavior, offset)?)
        .ok_or_else(|| error("Currency Wars shutdown effect ID is invalid"))
}

fn group_id(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    offset: u32,
) -> Result<ModifierStackingGroupId, CurrencyWarsBattleAssemblyError> {
    ModifierStackingGroupId::new(definition_raw(behavior, offset)?)
        .ok_or_else(|| error("Currency Wars shutdown group ID is invalid"))
}

fn modifier_id(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    offset: u32,
) -> Result<ModifierDefinitionId, CurrencyWarsBattleAssemblyError> {
    ModifierDefinitionId::new(definition_raw(behavior, offset)?)
        .ok_or_else(|| error("Currency Wars shutdown modifier ID is invalid"))
}

fn tag_digest(root: [u8; 32], element: CombatElement) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"starclock.currency-wars.enemy-affix-element-tag.v1");
    hash.update(root);
    hash.update([element as u8]);
    hash.finalize().into()
}
