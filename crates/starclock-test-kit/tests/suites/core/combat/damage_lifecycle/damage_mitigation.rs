use super::*;

pub(super) fn toughness_battle_with_mitigation(mitigated: bool) -> Battle {
    let player = combatant(1, vec![5], 1_000, 1_000_000_000, 0x71);
    let ordinary =
        ToughnessLayerSpec::ordinary(1, starclock_combat::RawToughness::new(50).unwrap())
            .unwrap()
            .with_break_credit(starclock_combat::BreakCreditPolicy::LayerProvider(
                definition(99),
            ));
    let exo = ToughnessLayerSpec::ordinary(2, starclock_combat::RawToughness::new(40).unwrap())
        .unwrap()
        .with_kind(ToughnessLayerKind::ExoToughness)
        .with_break_behavior(true, true, true, false);
    let mut enemy = combatant_with_modifiers(
        2,
        vec![3],
        if mitigated { vec![6, 7] } else { vec![] },
        10_000,
        1_000_000,
        0x72,
    )
    .with_toughness(EnemyRank::Normal, vec![], vec![ordinary, exo])
    .unwrap();
    if mitigated {
        let source = definition(91);
        enemy = enemy
            .with_sources(vec![RuleSource::new(
                source,
                SourceClass::Progression,
                vec![],
                [0x78; 32],
            )])
            .unwrap()
            .with_modifier_bindings(vec![
                ResolvedModifierBinding::new(definition(6), source),
                ResolvedModifierBinding::new(definition(7), source),
            ])
            .unwrap();
    }
    let spec = BattleSpec::new(
        AssemblyDigest::new([0x73; 32]).unwrap(),
        definition(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                player,
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(4).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                enemy,
            ),
        ],
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(catalog(1), spec, BattleSeed::new([0x74; 32])).unwrap()
}

#[test]
fn break_and_super_break_consume_dynamic_target_mitigation() {
    let mut ordinary = toughness_battle_with_mitigation(false);
    start_and_pass(&mut ordinary);
    let ordinary = use_ability(&mut ordinary, 5);
    let mut mitigated = toughness_battle_with_mitigation(true);
    start_and_pass(&mut mitigated);
    let mitigated = use_ability(&mut mitigated, 5);

    for kind in [
        starclock_combat::BreakDamageKind::Initial,
        starclock_combat::BreakDamageKind::SuperBreak,
    ] {
        let ordinary = ordinary
            .events()
            .iter()
            .find_map(|event| match event.kind() {
                BattleEventKind::BreakDamage(data) if data.kind == kind => {
                    Some(data.calculated.get())
                }
                _ => None,
            })
            .expect("ordinary Break event");
        let mitigated = mitigated
            .events()
            .iter()
            .find_map(|event| match event.kind() {
                BattleEventKind::BreakDamage(data) if data.kind == kind => {
                    Some(data.calculated.get())
                }
                _ => None,
            })
            .expect("mitigated Break event");
        assert_eq!(mitigated, ordinary / 2);
    }
}
