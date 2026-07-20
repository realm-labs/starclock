use std::sync::Arc;

use starclock_combat::{
    AbilityId, DispelCategory, DurationClock, EffectApplicationDefinition, EffectCategory,
    EffectChancePolicy, EffectDefinitionId, EffectRuntimeDefinition, EffectStackPolicy,
    EffectTickPhase, EncounterId, EnemyDefinitionId, Energy, Hp, ProgramId, Ratio, Scalar,
    SelectorId, UnitDefinitionId,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            HealingDefinition, HitOperationDefinition, HpConsumptionDefinition,
            OrdinaryDamageDefinition, OrdinaryDamageMultipliers, TargetInvalidationPolicy,
            TargetPattern, TargetRelation, TeamResourceChange, TeamResourceChangeDefinition,
            UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EffectDefinition, EncounterDefinition, EnemyDefinition,
            ProgramDefinition, SelectorDefinition, UnitDefinition,
        },
    },
};

use super::{BENCHMARK_CATALOG_REVISION, BENCHMARK_CONFIG_DIGEST};

pub(super) fn catalog() -> Arc<CombatCatalog> {
    let mut builder =
        CombatCatalogBuilder::new(BENCHMARK_CATALOG_REVISION, BENCHMARK_CONFIG_DIGEST);
    let selector = SelectorId::new(1).expect("static ID");
    let program = ProgramId::new(1).expect("static ID");
    builder.add_selector(
        SelectorDefinition::new(selector).with_unit_targets(
            UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single)
                .expect("opposing single selector is valid"),
        ),
    );
    builder.add_program(ProgramDefinition::new(
        program,
        vec![],
        vec![selector],
        vec![],
        vec![],
    ));
    let full_program = ProgramId::new(2).expect("static ID");
    let full_effect = EffectDefinitionId::new(1).expect("static ID");
    builder.add_program(ProgramDefinition::new(
        full_program,
        vec![],
        vec![selector],
        vec![full_effect],
        vec![],
    ));
    let effect_runtime = EffectRuntimeDefinition::new(
        EffectCategory::Debuff,
        DispelCategory::DispellableDebuff,
        1,
        Some(2),
        DurationClock::TargetTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .expect("static effect runtime");
    builder.add_effect(
        EffectDefinition::new(full_effect, vec![], vec![]).with_runtime(effect_runtime),
    );

    let ordinary = AbilityId::new(1).expect("static ID");
    let heavy = AbilityId::new(2).expect("static ID");
    let enemy_ability = AbilityId::new(3).expect("static ID");
    let full = AbilityId::new(4).expect("static ID");
    builder.add_ability(ability(ordinary, program, selector, Vec::new()));
    let damage = OrdinaryDamageDefinition::new(
        Scalar::checked_from_integer(1).expect("static scalar"),
        OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).expect("identity multipliers"),
    )
    .expect("static damage definition");
    let hits = (0..8)
        .map(|_| ActionHitDefinition::new(vec![HitOperationDefinition::Damage(damage)]))
        .collect();
    builder.add_ability(ability(heavy, program, selector, hits));
    builder.add_ability(ability(enemy_ability, program, selector, Vec::new()));
    let effect = EffectApplicationDefinition::new(full_effect, EffectChancePolicy::Guaranteed, 1)
        .expect("static effect application");
    let healing = HealingDefinition::new(
        Scalar::checked_from_integer(2).expect("static scalar"),
        Ratio::ZERO,
        Ratio::ZERO,
        Ratio::ZERO,
    )
    .expect("static healing definition");
    let full_hits = (0..4)
        .map(|_| {
            ActionHitDefinition::new(vec![
                HitOperationDefinition::Damage(damage),
                HitOperationDefinition::Heal(healing),
                HitOperationDefinition::ConsumeHp(HpConsumptionDefinition::new(
                    Hp::new(1).expect("static HP"),
                    Hp::new(1).expect("static HP"),
                )),
                HitOperationDefinition::ApplyEffect(effect),
                HitOperationDefinition::ModifyTeamResource(TeamResourceChangeDefinition::new(
                    starclock_combat::SourceDefinitionId::new(1).expect("static ID"),
                    TeamResourceChange::Gain(1),
                )),
            ])
        })
        .collect();
    builder.add_ability(
        AbilityDefinition::new(full, full_program, selector, vec![full_effect]).with_action(
            AbilityActionDefinition::new(
                AbilityKind::Basic,
                4,
                TargetInvalidationPolicy::CancelRemainingForTarget,
                ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
            )
            .expect("static action policy")
            .with_hits(full_hits)
            .expect("static full-kernel hits"),
        ),
    );

    let ordinary_form = UnitDefinitionId::new(1).expect("static ID");
    let heavy_form = UnitDefinitionId::new(2).expect("static ID");
    let enemy_form = UnitDefinitionId::new(3).expect("static ID");
    let full_form = UnitDefinitionId::new(4).expect("static ID");
    builder.add_unit(UnitDefinition::new(ordinary_form, vec![ordinary], vec![]));
    builder.add_unit(UnitDefinition::new(heavy_form, vec![heavy], vec![]));
    builder.add_unit(UnitDefinition::new(enemy_form, vec![enemy_ability], vec![]));
    builder.add_unit(UnitDefinition::new(full_form, vec![full], vec![]));
    let enemy = EnemyDefinitionId::new(1).expect("static ID");
    builder.add_enemy(EnemyDefinition::new(enemy, enemy_form, vec![enemy_ability]));
    for (encounter, count) in [(1, 1), (2, 2), (3, 4)] {
        let definition = EncounterDefinition::new(
            EncounterId::new(encounter).expect("static ID"),
            vec![enemy],
            vec![],
        )
        .with_waves(vec![vec![enemy; count]])
        .expect("one non-empty static wave");
        builder.add_encounter(definition);
    }
    builder.build().expect("benchmark catalog must validate")
}

fn ability(
    id: AbilityId,
    program: ProgramId,
    selector: SelectorId,
    hits: Vec<ActionHitDefinition>,
) -> AbilityDefinition {
    AbilityDefinition::new(id, program, selector, vec![]).with_action(
        AbilityActionDefinition::new(
            AbilityKind::Basic,
            1,
            TargetInvalidationPolicy::CancelRemainingForTarget,
            ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
        )
        .expect("static action policy")
        .with_hits(if hits.is_empty() {
            vec![ActionHitDefinition::new(Vec::new())]
        } else {
            hits
        })
        .expect("static action has at least one hit"),
    )
}
