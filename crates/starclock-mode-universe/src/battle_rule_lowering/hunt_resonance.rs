use super::*;
use starclock_combat::catalog::action::{HitOperationDefinition, ScalingDamageDefinition};

pub(super) fn lower(
    catalog: &UniverseCatalog,
    binding: &UniverseBattleRuleBinding,
    initial_energy: u16,
    damage_ratio: i64,
) -> Result<ExecutableResonance, BattleRuleLoweringError> {
    let resonance = catalog
        .resonances()
        .iter()
        .find(|definition| definition.stable_key() == binding.source_record_key())
        .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
    let ratio = Ratio::from_scaled(parameter(resonance.parameters(), 1)?)
        .checked_mul(
            Ratio::ONE
                .checked_add(Ratio::from_scaled(damage_ratio))
                .map_err(|_| BattleRuleLoweringError::InvalidParameter)?,
            starclock_combat::Rounding::NearestTiesEven,
        )
        .map_err(|_| BattleRuleLoweringError::InvalidParameter)?;
    let action = AbilityActionDefinition::new(
        AbilityKind::Ultimate,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(
            0,
            0,
            starclock_combat::Energy::ZERO,
            starclock_combat::Energy::ZERO,
        )
        .with_team_resource_costs(vec![
            TeamResourceCost::new(RESONANCE_RESOURCE_KEY, 100)
                .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
        ])
        .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?
    .with_tags(&[AbilityTag::Attack, AbilityTag::Ultimate, AbilityTag::Assist])
    .with_hits(vec![ActionHitDefinition::new(vec![
        HitOperationDefinition::ScalingDamage(
            ScalingDamageDefinition::new(
                StatKind::Atk,
                ratio,
                DamageClass::Additional,
                CombatElement::Wind,
            )
            .map_err(|_| BattleRuleLoweringError::InvalidDefinition)?,
        ),
    ])])
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let selector = SelectorDefinition::new(RESONANCE_SELECTOR_ID).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::All)
            .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
    );
    let program = ProgramDefinition::new(
        RESONANCE_PROGRAM_ID,
        Vec::new(),
        vec![RESONANCE_SELECTOR_ID],
        Vec::new(),
        Vec::new(),
    );
    let ability = AbilityDefinition::new(
        RESONANCE_ABILITY_ID,
        RESONANCE_PROGRAM_ID,
        RESONANCE_SELECTOR_ID,
        Vec::new(),
    )
    .with_action(action);
    Ok(ExecutableResonance {
        modifier_groups: Box::new([]),
        modifiers: Box::new([]),
        selectors: vec![selector].into_boxed_slice(),
        effects: Box::new([]),
        programs: vec![program].into_boxed_slice(),
        ability,
        initial_energy: initial_energy.min(100),
        maximum_energy: 100,
    })
}
