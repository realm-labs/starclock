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

use super::{SYNTHETIC_STANDARD_CATALOG_REVISION, SYNTHETIC_STANDARD_CONFIG_DIGEST};

pub(super) fn catalog() -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new(
        SYNTHETIC_STANDARD_CATALOG_REVISION,
        SYNTHETIC_STANDARD_CONFIG_DIGEST,
    );
    let player_selector = SelectorId::new(1).expect("static ID is non-zero");
    let enemy_selector = SelectorId::new(2).expect("static ID is non-zero");
    builder.add_selector(
        SelectorDefinition::new(player_selector).with_unit_targets(
            UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single)
                .expect("opposing single selector is valid"),
        ),
    );
    builder.add_selector(
        SelectorDefinition::new(enemy_selector).with_unit_targets(
            UnitTargetSelector::new(TargetRelation::SelfUnit, TargetPattern::Single)
                .expect("self single selector is valid"),
        ),
    );
    for (program, selector) in [
        (ProgramId::new(1).expect("static ID"), player_selector),
        (ProgramId::new(2).expect("static ID"), enemy_selector),
    ] {
        builder.add_program(ProgramDefinition::new(
            program,
            vec![],
            vec![selector],
            vec![],
            vec![],
        ));
    }

    let player_ability = AbilityId::new(1).expect("static ID");
    let enemy_ability = AbilityId::new(2).expect("static ID");
    let damage = OrdinaryDamageDefinition::new(
        Scalar::checked_from_integer(1_000).expect("static scalar is in range"),
        OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).expect("identity multipliers are valid"),
    )
    .expect("static damage definition is valid");
    builder.add_ability(
        AbilityDefinition::new(
            player_ability,
            ProgramId::new(1).expect("static ID"),
            player_selector,
            vec![],
        )
        .with_action(
            action(vec![HitOperationDefinition::Damage(damage)])
                .expect("static player action is valid"),
        ),
    );
    builder.add_ability(
        AbilityDefinition::new(
            enemy_ability,
            ProgramId::new(2).expect("static ID"),
            enemy_selector,
            vec![],
        )
        .with_action(action(Vec::new()).expect("static enemy action is valid")),
    );

    let player_form = UnitDefinitionId::new(1).expect("static ID");
    let enemy_form = UnitDefinitionId::new(2).expect("static ID");
    builder.add_unit(UnitDefinition::new(
        player_form,
        vec![player_ability],
        vec![],
    ));
    builder.add_unit(UnitDefinition::new(enemy_form, vec![enemy_ability], vec![]));
    let enemy = EnemyDefinitionId::new(1).expect("static ID");
    builder.add_enemy(EnemyDefinition::new(enemy, enemy_form, vec![enemy_ability]));
    builder.add_encounter(EncounterDefinition::new(
        EncounterId::new(1).expect("static ID"),
        vec![enemy],
        vec![],
    ));
    builder
        .build()
        .expect("committed synthetic Standard catalog must validate")
}

fn action(operations: Vec<HitOperationDefinition>) -> Option<AbilityActionDefinition> {
    AbilityActionDefinition::new(
        AbilityKind::Basic,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
    )?
    .with_hits(vec![ActionHitDefinition::new(operations)])
}
