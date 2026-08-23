//! Back-row battle events materialized as ordinary linked combat entities.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use starclock_combat::{
    AbilityId, ActionGauge, CombatantSpecDigest, FormationIndex, Hp, LinkedEntityKind,
    LinkedUnitCatalogDefinition, LinkedUnitDefinition, OwnerLinkPolicy, PresenceState, ProgramId,
    ResolvedBuildBonuses, ResolvedCombatantSpec, ResolvedDefinitionBindings, Rounding,
    RuleBundleId, RuleId, Scalar, SelectorId, SourceDefinitionId, Speed, StatValue, TriggerId,
    UnitDefinitionId, WaveLinkPolicy,
    catalog::{
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition,
            UnitDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    rule::model::{
        BattleRuleDefinition, ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority,
        RuleEventKind, RuleEventPoint, RuleOperationTemplate, RuleSource, SourceClass, TriggerDef,
        TriggerPhase,
    },
};

use crate::{
    CurrencyWarsBackBattleEvent, CurrencyWarsBattleEventPropertyKind,
    CurrencyWarsContributionSnapshot, CurrencyWarsRoleId,
    battle_assembly::{
        CurrencyWarsBattleAssemblyError, combatant_overlay::attach_rule_bundle, debug_error, error,
    },
};

const DEFINITION_BASE: u32 = 0x7d60_0000;
const DEFINITIONS_PER_EVENT: u32 = 16;

pub(super) fn install(
    builder: &mut CombatCatalogBuilder,
    snapshot: &CurrencyWarsContributionSnapshot,
    combatants: &mut BTreeMap<CurrencyWarsRoleId, ResolvedCombatantSpec>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    if snapshot.battle_overrides.back_battle_events.is_empty() {
        return Ok(());
    }
    let host = combatants
        .keys()
        .next()
        .copied()
        .ok_or_else(|| error("Currency Wars back battle event has no front-row host"))?;
    for (index, event) in snapshot
        .battle_overrides
        .back_battle_events
        .iter()
        .enumerate()
    {
        let host_combatant = combatants
            .get(&host)
            .ok_or_else(|| error("Currency Wars back battle-event host is missing"))?;
        let compiled = compile(event, host_combatant, index)?;
        builder.add_selector(compiled.identity_selector);
        builder.add_selector(compiled.owner_selector);
        builder.add_program(compiled.identity_program);
        builder.add_program(compiled.spawn_program);
        builder.add_ability(compiled.identity_ability);
        builder.add_unit(compiled.unit);
        builder.add_linked_unit(compiled.linked_unit);
        builder.add_rule(compiled.spawn_rule);
        builder.add_rule_bundle(compiled.spawn_bundle);
        let replacement = attach_rule_bundle(
            host_combatant,
            compiled.bundle_id,
            b"starclock.currency-wars.back-battle-event-host.v1",
            compiled.digest,
        )?;
        combatants.insert(host, replacement);
    }
    Ok(())
}

struct CompiledBackBattleEvent {
    identity_selector: SelectorDefinition,
    owner_selector: SelectorDefinition,
    identity_program: ProgramDefinition,
    spawn_program: ProgramDefinition,
    identity_ability: AbilityDefinition,
    unit: UnitDefinition,
    linked_unit: LinkedUnitCatalogDefinition,
    spawn_rule: RuleDefinition,
    spawn_bundle: RuleBundle,
    bundle_id: RuleBundleId,
    digest: [u8; 32],
}

fn compile(
    event: &CurrencyWarsBackBattleEvent,
    host: &ResolvedCombatantSpec,
    index: usize,
) -> Result<CompiledBackBattleEvent, CurrencyWarsBattleAssemblyError> {
    let base = DEFINITION_BASE
        .checked_add(
            u32::try_from(index)
                .map_err(debug_error)?
                .checked_mul(DEFINITIONS_PER_EVENT)
                .ok_or_else(|| error("Currency Wars back battle-event ID overflow"))?,
        )
        .ok_or_else(|| error("Currency Wars back battle-event ID overflow"))?;
    let form = unit_id(base, 1)?;
    let ability = ability_id(base, 2)?;
    let identity_program_id = program_id(base, 3)?;
    let identity_selector_id = selector_id(base, 4)?;
    let source = source_id(base, 5)?;
    let owner_selector_id = selector_id(base, 6)?;
    let spawn_program_id = program_id(base, 7)?;
    let spawn_rule_id = rule_id(base, 8)?;
    let bundle_id = rule_bundle_id(base, 9)?;
    let trigger_id = trigger_id(base, 10)?;
    let digest = event_digest(event, index);
    let combatant = event_combatant(event, host, form, ability, digest)?;
    let formation = FormationIndex::new(
        16_u8
            .checked_add(u8::try_from(index).map_err(debug_error)?)
            .ok_or_else(|| error("Currency Wars back battle-event formation overflow"))?,
    )
    .ok_or_else(|| error("Currency Wars back battle-event formation is invalid"))?;
    let linked = LinkedUnitDefinition::new(
        combatant,
        source,
        formation,
        LinkedEntityKind::SharedActor,
        PresenceState::Untargetable,
        None,
        ActionGauge::from_scaled(0).map_err(debug_error)?,
        OwnerLinkPolicy::Persist,
        OwnerLinkPolicy::Persist,
        WaveLinkPolicy::Persist,
    )
    .ok_or_else(|| error("Currency Wars back battle-event linked unit is invalid"))?;
    let linked_unit = LinkedUnitCatalogDefinition::new(form, linked)
        .ok_or_else(|| error("Currency Wars back battle-event linked catalog entry is invalid"))?;
    let identity_selector = SelectorDefinition::new(identity_selector_id);
    let owner_selector = SelectorDefinition::new(owner_selector_id).with_rule_units(
        RuleUnitSelector::new(
            RuleSelectorOrigin::Team,
            RuleSelectorSide::Same,
            // The attached host is only a deterministic rule anchor. Select any active
            // front-row participant as the linked entity's runtime owner; if Activity carry
            // enters with the whole team defeated, there is nothing to materialize.
            RuleLifePredicate::Alive,
            RulePresencePredicate::Present,
            RuleSelectorReference::CurrentState,
            RuleSelectorOrdering::StableId,
            0,
            1,
            RuleEmptyPoolPolicy::NoOp,
            RuleSelectorChoice::First,
            None,
            false,
        )
        .ok_or_else(|| error("Currency Wars back battle-event owner selector is invalid"))?,
    );
    let identity_program = ProgramDefinition::new(
        identity_program_id,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let identity_ability = AbilityDefinition::new(
        ability,
        identity_program_id,
        identity_selector_id,
        Vec::new(),
    );
    let unit = UnitDefinition::new(form, vec![ability], Vec::new());
    let spawn_program = ProgramDefinition::new(
        spawn_program_id,
        Vec::new(),
        vec![owner_selector_id],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::Summon {
            owner_selector: owner_selector_id,
            unit_definition: form,
        },
    )]);
    let priority = ReactionPriority::new(
        -150_i16
            .checked_add(i16::try_from(index).map_err(debug_error)?)
            .ok_or_else(|| error("Currency Wars back battle-event priority overflow"))?,
    );
    let runtime = BattleRuleDefinition::new(
        RuleSource::new(source, SourceClass::Mode, Vec::new(), digest),
        Vec::new(),
        vec![TriggerDef {
            id: trigger_id,
            event: RuleEventKind::Battle,
            event_point: RuleEventPoint::BattleStarted,
            phase: TriggerPhase::AfterEvent,
            filter: EventFilter::default(),
            condition: ConditionExpr::Literal(true),
            once_scope: OnceScope::Battle,
            priority,
            program: spawn_program_id,
        }],
        None,
    );
    let spawn_rule = RuleDefinition::new(
        spawn_rule_id,
        vec![spawn_program_id],
        vec![owner_selector_id],
    )
    .with_runtime(runtime);
    let spawn_bundle = RuleBundle::new(bundle_id, vec![spawn_rule_id]);
    Ok(CompiledBackBattleEvent {
        identity_selector,
        owner_selector,
        identity_program,
        spawn_program,
        identity_ability,
        unit,
        linked_unit,
        spawn_rule,
        spawn_bundle,
        bundle_id,
        digest,
    })
}

fn event_combatant(
    event: &CurrencyWarsBackBattleEvent,
    host: &ResolvedCombatantSpec,
    form: UnitDefinitionId,
    ability: AbilityId,
    digest: [u8; 32],
) -> Result<ResolvedCombatantSpec, CurrencyWarsBattleAssemblyError> {
    let mut hp = Scalar::ONE;
    let mut attack = Scalar::ZERO;
    let mut defense = Scalar::ZERO;
    let mut attack_delta = Scalar::ZERO;
    let mut attack_ratio = Scalar::ZERO;
    let mut critical_rate = Scalar::ZERO;
    let mut critical_damage = Scalar::ZERO;
    let mut effect_hit_rate = Scalar::ZERO;
    let mut damage_boosts = [Scalar::ZERO; 7];
    for property in &event.properties {
        let value = property.value.scalar().map_err(debug_error)?;
        match property.kind {
            CurrencyWarsBattleEventPropertyKind::BaseHp => hp = value,
            CurrencyWarsBattleEventPropertyKind::BaseAttack => attack = value,
            CurrencyWarsBattleEventPropertyKind::BaseDefence => defense = value,
            CurrencyWarsBattleEventPropertyKind::AttackDelta => attack_delta = value,
            CurrencyWarsBattleEventPropertyKind::AttackAddedRatio => attack_ratio = value,
            CurrencyWarsBattleEventPropertyKind::CriticalChance => critical_rate = value,
            CurrencyWarsBattleEventPropertyKind::CriticalDamage => critical_damage = value,
            CurrencyWarsBattleEventPropertyKind::StatusProbability => effect_hit_rate = value,
            CurrencyWarsBattleEventPropertyKind::AllDamageTypeAddedRatio => {
                damage_boosts.fill(value);
            }
            CurrencyWarsBattleEventPropertyKind::FireAddedRatio => damage_boosts[0] = value,
            CurrencyWarsBattleEventPropertyKind::FirePenetration
            | CurrencyWarsBattleEventPropertyKind::MaximumEnergy => {}
        }
    }
    attack = attack
        .checked_add(attack_delta)
        .and_then(|value| {
            value.checked_mul(
                Scalar::ONE.checked_add(attack_ratio)?,
                Rounding::NearestTiesEven,
            )
        })
        .map_err(debug_error)?;
    let maximum_hp = Hp::from_scalar(hp, Rounding::NearestTiesEven).map_err(debug_error)?;
    let maximum_hp = if maximum_hp.get() == 0 {
        Hp::new(1).expect("one HP is the linked-unit minimum")
    } else {
        maximum_hp
    };
    let speed = event
        .speed
        .map(|value| value.scalar().map_err(debug_error))
        .transpose()?
        .unwrap_or(Scalar::ONE);
    let combatant_digest = CombatantSpecDigest::new(digest)
        .ok_or_else(|| error("Currency Wars back battle-event combatant digest is zero"))?;
    let attack = StatValue::from_scaled(attack.scaled()).map_err(debug_error)?;
    let defense = StatValue::from_scaled(defense.scaled()).map_err(debug_error)?;
    ResolvedCombatantSpec::new(
        form,
        host.level(),
        maximum_hp,
        Speed::from_scaled(speed.scaled()).map_err(debug_error)?,
        ResolvedDefinitionBindings::new(vec![ability], Vec::new(), Vec::new())
            .map_err(debug_error)?,
        combatant_digest,
    )
    .map_err(debug_error)
    .map(|value| {
        value
            .with_base_attack_defense(attack, defense)
            .with_base_effect_stats(effect_hit_rate, Scalar::ZERO)
            .with_build_bonuses(ResolvedBuildBonuses::new(
                critical_rate,
                critical_damage,
                Scalar::ZERO,
                Scalar::ZERO,
                Scalar::ZERO,
                damage_boosts,
            ))
    })
}

fn event_digest(event: &CurrencyWarsBackBattleEvent, index: usize) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"starclock.currency-wars.back-battle-event.v1");
    hash.update(event.event_id.to_le_bytes());
    hash.update(
        u64::try_from(index)
            .expect("battle-event index fits u64")
            .to_le_bytes(),
    );
    hash.update([
        event.kind.canonical_tag(),
        event.team.canonical_tag(),
        u8::from(event.hard_level),
    ]);
    for ability in &event.abilities {
        hash.update(ability.as_bytes());
        hash.update([0]);
    }
    match event.speed {
        None => hash.update([0]),
        Some(speed) => {
            hash.update([1]);
            hash.update(speed.significand().to_le_bytes());
            hash.update([speed.decimal_places()]);
        }
    }
    for value in &event.values {
        hash.update(value.significand().to_le_bytes());
        hash.update([value.decimal_places()]);
    }
    for property in &event.properties {
        hash.update([property.kind.canonical_tag()]);
        hash.update(property.value.significand().to_le_bytes());
        hash.update([property.value.decimal_places()]);
    }
    hash.finalize().into()
}

macro_rules! definition_id {
    ($function:ident, $type:ty, $message:literal) => {
        fn $function(base: u32, offset: u32) -> Result<$type, CurrencyWarsBattleAssemblyError> {
            <$type>::new(
                base.checked_add(offset)
                    .ok_or_else(|| error("Currency Wars back battle-event ID overflow"))?,
            )
            .ok_or_else(|| error($message))
        }
    };
}

definition_id!(unit_id, UnitDefinitionId, "invalid linked-unit ID");
definition_id!(ability_id, AbilityId, "invalid linked ability ID");
definition_id!(program_id, ProgramId, "invalid linked program ID");
definition_id!(selector_id, SelectorId, "invalid linked selector ID");
definition_id!(source_id, SourceDefinitionId, "invalid linked source ID");
definition_id!(rule_id, RuleId, "invalid linked rule ID");
definition_id!(rule_bundle_id, RuleBundleId, "invalid linked bundle ID");
definition_id!(trigger_id, TriggerId, "invalid linked trigger ID");
