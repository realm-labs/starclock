use std::sync::Arc;

use starclock_combat::{
    AbilityId, EncounterId, EnemyDefinitionId, Energy, ProgramId, Ratio, Scalar, SelectorId,
    UnitDefinitionId,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            HitOperationDefinition, OrdinaryDamageDefinition, OrdinaryDamageMultipliers,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
            SelectorDefinition, UnitDefinition,
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

    let ordinary = AbilityId::new(1).expect("static ID");
    let heavy = AbilityId::new(2).expect("static ID");
    let enemy_ability = AbilityId::new(3).expect("static ID");
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

    let ordinary_form = UnitDefinitionId::new(1).expect("static ID");
    let heavy_form = UnitDefinitionId::new(2).expect("static ID");
    let enemy_form = UnitDefinitionId::new(3).expect("static ID");
    builder.add_unit(UnitDefinition::new(ordinary_form, vec![ordinary], vec![]));
    builder.add_unit(UnitDefinition::new(heavy_form, vec![heavy], vec![]));
    builder.add_unit(UnitDefinition::new(enemy_form, vec![enemy_ability], vec![]));
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
