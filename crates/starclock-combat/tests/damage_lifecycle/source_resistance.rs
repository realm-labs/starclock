use super::*;

#[test]
fn source_resistance_stage_is_applied_as_ordinary_damage_penetration() {
    let plain = damage(false);
    let penetrated = damage(true);
    assert_eq!(penetrated, plain * 125 / 100);
}

fn damage(with_penetration: bool) -> i64 {
    let spec = BattleSpec::new(
        "source-resistance-penetration-v1",
        AssemblyDigest::new([0x5b; 32]).unwrap(),
        definition(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                scaling_combatant(with_penetration),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(4).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                combatant(2, vec![3], 10_000, 1_000_000, 0x4b),
            ),
        ],
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    let mut battle = Battle::create(catalog(1), spec, BattleSeed::new([0x6b; 32])).unwrap();
    start_and_pass(&mut battle);
    use_ability(&mut battle, 6)
        .events()
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some(data.raw.scaled()),
            _ => None,
        })
        .expect("scaling damage")
}

fn scaling_combatant(with_penetration: bool) -> ResolvedCombatantSpec {
    let source = definition(93);
    let modifiers = with_penetration
        .then(|| definition(8))
        .into_iter()
        .collect();
    let mut combatant = ResolvedCombatantSpec::new(
        definition(1),
        UnitLevel::new(80).unwrap(),
        Hp::new(1_000).unwrap(),
        Speed::from_scaled(1_000_000_000).unwrap(),
        ResolvedDefinitionBindings::new(vec![definition(6)], vec![], modifiers).unwrap(),
        CombatantSpecDigest::new([0x7b; 32]).unwrap(),
    )
    .unwrap()
    .with_base_attack_defense(
        StatValue::from_scaled(2_000_000_000).unwrap(),
        StatValue::from_scaled(0).unwrap(),
    );
    if with_penetration {
        combatant = combatant
            .with_sources(vec![RuleSource::new(
                source,
                SourceClass::Progression,
                vec![],
                [0x78; 32],
            )])
            .unwrap()
            .with_modifier_bindings(vec![ResolvedModifierBinding::new(definition(8), source)])
            .unwrap();
    }
    combatant
}
