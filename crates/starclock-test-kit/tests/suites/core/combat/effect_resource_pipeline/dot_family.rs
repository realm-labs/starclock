//! Actual accepted-command coverage of semantic candidate selection.

use super::{action, catalog, combatant, definition, dot_damage, execute_probe};
use starclock_combat::{
    AssemblyDigest, Battle, BattleEventKind, BattleSeed, BattleSpec, ConcedePolicy, DamageKind,
    DispelCategory, DotDefinition, DotDetonationDefinition, DotDetonationFilter,
    DotDetonationSelection, DotFamily, DurationClock, EffectApplicationDefinition, EffectCategory,
    EffectChancePolicy, EffectRuntimeDefinition, EffectStackPolicy, EffectTickPhase,
    FormationIndex, ParticipantSource, ParticipantSpec, Ratio, Scalar, TeamResourceSpec, TeamSide,
    catalog::{
        CombatCatalog,
        action::{AbilityProgramBinding, AbilityProgramTiming, HitOperationDefinition},
        builder::{CatalogBuildErrorKind, CombatCatalogBuilder},
        definition::{AbilityDefinition, EffectDefinition, ProgramDefinition, SelectorDefinition},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    formula::model::CombatElement,
    rng::types::DrawPurpose,
    rule::model::{
        ProgramStep, RuleDotSelection, RuleEffectChancePolicy, RuleOperationTemplate, RuleValue,
        ValueExpr,
    },
};
use std::sync::Arc;

fn runtime(category: EffectCategory, damaging: bool) -> EffectRuntimeDefinition {
    let runtime = EffectRuntimeDefinition::new(
        category,
        DispelCategory::DispellableDebuff,
        2,
        Some(3),
        DurationClock::TargetTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::IndependentInstances,
    )
    .unwrap();
    if damaging {
        runtime
            .with_dot(DotDefinition::new(
                dot_damage(10),
                CombatElement::Fire,
                Some(definition(77)),
            ))
            .unwrap()
    } else {
        runtime
    }
}

fn builder(detonation: DotDetonationDefinition, ir: bool) -> CombatCatalogBuilder {
    let mut builder = CombatCatalogBuilder::from_catalog(&catalog(), [0xb4; 32]);
    // All share Fire damage and a tag except 11. Element never implies family.
    for (raw, family, tag) in [
        (10, Some(DotFamily::Burn), 77),
        (11, Some(DotFamily::Burn), 88),
        (12, Some(DotFamily::Burn), 77),
        (13, None, 77),
        (14, Some(DotFamily::Shock), 77),
        (15, Some(DotFamily::WindShear), 77),
    ] {
        let runtime = runtime(EffectCategory::Dot, true)
            .with_dot(DotDefinition::new(
                dot_damage(i64::from(raw)),
                CombatElement::Fire,
                Some(definition(tag)),
            ))
            .unwrap();
        let mut effect =
            EffectDefinition::new(definition(raw), vec![], vec![]).with_runtime(runtime);
        if let Some(family) = family {
            effect = effect.with_dot_family(family);
        }
        builder.add_effect(effect);
    }
    let effects = (10..=15).map(definition).collect::<Vec<_>>();
    let mut ability =
        AbilityDefinition::new(definition(1), definition(1), definition(1), effects.clone());
    if ir {
        builder.add_selector(
            SelectorDefinition::new(definition(3)).with_rule_units(
                RuleUnitSelector::new(
                    RuleSelectorOrigin::PrimaryTarget,
                    RuleSelectorSide::Opposing,
                    RuleLifePredicate::Alive,
                    RulePresencePredicate::Present,
                    RuleSelectorReference::CurrentState,
                    RuleSelectorOrdering::StableId,
                    1,
                    1,
                    RuleEmptyPoolPolicy::Fault,
                    RuleSelectorChoice::First,
                    None,
                    false,
                )
                .unwrap(),
            ),
        );
        let mut steps = effects
            .iter()
            .map(|effect| {
                ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                    selector: definition(3),
                    effect: *effect,
                    stacks: ValueExpr::Literal(RuleValue::Integer(if effect.get() == 10 {
                        2
                    } else {
                        1
                    })),
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    rng_purpose: None,
                })
            })
            .collect::<Vec<_>>();
        steps.push(ProgramStep::Operation(RuleOperationTemplate::DetonateDot {
            selector: definition(3),
            fraction: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(
                detonation.fraction().scaled(),
            ))),
            required_tag: detonation.required_tag(),
            selection: RuleDotSelection::Filtered {
                filter: detonation.filter(),
                selection: detonation.selection(),
            },
        }));
        builder.add_program(
            ProgramDefinition::new(definition(3), vec![], vec![definition(3)], effects, vec![])
                .with_steps(steps),
        );
        ability = ability.with_action(action(vec![])).with_programs(vec![
            AbilityProgramBinding::new(1, AbilityProgramTiming::Hits, definition(3)).unwrap(),
        ]);
    } else {
        let mut operations = effects
            .iter()
            .map(|effect| {
                HitOperationDefinition::ApplyEffect(
                    EffectApplicationDefinition::new(
                        *effect,
                        EffectChancePolicy::Guaranteed,
                        if effect.get() == 10 { 2 } else { 1 },
                    )
                    .unwrap(),
                )
            })
            .collect::<Vec<_>>();
        operations.push(HitOperationDefinition::DetonateDots(detonation));
        ability = ability.with_action(action(operations));
    }
    assert!(builder.replace_ability(ability));
    builder
}

fn battle(catalog: Arc<CombatCatalog>) -> Battle {
    let spec = BattleSpec::new(
        AssemblyDigest::new([0x41; 32]).unwrap(),
        definition(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, 1, 200_000_000, 1),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                combatant(2, 2, 101_000_000, 2),
            ),
        ],
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(catalog, spec, BattleSeed::new([0x51; 32])).unwrap()
}

fn detonation(
    filter: DotDetonationFilter,
    tag: Option<u32>,
    random: bool,
) -> DotDetonationDefinition {
    DotDetonationDefinition::new(Ratio::from_scaled(2_000_000), tag.map(definition))
        .unwrap()
        .with_filter(filter)
        .with_selection(if random {
            DotDetonationSelection::RandomOne(DrawPurpose::new(300).unwrap())
        } else {
            DotDetonationSelection::All
        })
}

#[test]
fn dot_family_filters_intersect_before_random_selection_and_preserve_instances() {
    let filter = DotDetonationFilter::family(DotFamily::Burn).excluding(definition(12));
    for ir in [false, true] {
        let (battle, events, _) = execute_probe(battle(
            builder(detonation(filter, Some(77), true), ir)
                .build()
                .unwrap(),
        ));
        let retained = battle.view().effects_by_id().collect::<Vec<_>>();
        assert_eq!(retained.len(), 6);
        assert!(retained.iter().all(|effect| effect.remaining() == Some(3)));
        let burns = events
            .iter()
            .filter_map(|event| match event.kind() {
                BattleEventKind::Damage(data) if data.kind == DamageKind::DotDetonation => {
                    Some((event.cause(), data))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(burns.len(), 1);
        let effect = retained
            .iter()
            .find(|effect| effect.definition().get() == 10)
            .unwrap();
        assert_eq!(effect.stacks(), 2);
        assert_eq!(burns[0].1.source_effect, Some(effect.id()));
        assert_eq!(burns[0].1.calculated.get(), 40);
        assert_eq!(burns[0].0.applier(), Some(effect.applier()));
        assert_eq!(
            burns[0].0.source_definition(),
            Some(effect.source_definition())
        );
        assert_eq!(battle.view().rng_draw_count(), 0);
    }
}

#[test]
fn dot_family_empty_random_pool_draws_nothing_and_unfiltered_selection_is_unchanged() {
    for (filter, expected) in [
        (DotDetonationFilter::family(DotFamily::Bleed), 0),
        (DotDetonationFilter::default(), 6),
    ] {
        let (battle, events, _) = execute_probe(battle(
            builder(detonation(filter, None, expected == 0), false)
                .build()
                .unwrap(),
        ));
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event.kind(),
            BattleEventKind::Damage(data) if data.kind == DamageKind::DotDetonation))
                .count(),
            expected
        );
        assert_eq!(battle.view().rng_draw_count(), 0);
    }
}

#[test]
fn dot_family_all_matching_candidates_keep_canonical_order_and_damage() {
    let filter = DotDetonationFilter::family(DotFamily::Burn).excluding(definition(12));
    for ir in [false, true] {
        let (battle, events, _) = execute_probe(battle(
            builder(detonation(filter, None, false), ir)
                .build()
                .unwrap(),
        ));
        let damage = events
            .iter()
            .filter_map(|event| match event.kind() {
                BattleEventKind::Damage(data) if data.kind == DamageKind::DotDetonation => {
                    let effect = battle
                        .view()
                        .effects_by_id()
                        .find(|effect| Some(effect.id()) == data.source_effect)
                        .unwrap();
                    Some((effect.definition().get(), data.calculated.get()))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(damage, [(10, 40), (11, 22)]);
        assert_eq!(battle.view().rng_draw_count(), 0);
    }
}

#[test]
fn dot_family_random_pool_replays_with_only_eligible_definitions() {
    let definition = detonation(
        DotDetonationFilter::family(DotFamily::Burn).excluding(definition(12)),
        None,
        true,
    );
    for ir in [false, true] {
        let catalog = builder(definition, ir).build().unwrap();
        let (first, events, hash) = execute_probe(battle(Arc::clone(&catalog)));
        let (_, repeated, repeated_hash) = execute_probe(battle(catalog));
        assert_eq!(events, repeated);
        assert_eq!(hash, repeated_hash);
        assert_eq!(first.view().rng_draw_count(), 1);
        let damages = events
            .iter()
            .filter_map(|event| match event.kind() {
                BattleEventKind::Damage(data) if data.kind == DamageKind::DotDetonation => {
                    Some(data)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(damages.len(), 1);
        let source = first
            .view()
            .effects_by_id()
            .find(|effect| Some(effect.id()) == damages[0].source_effect)
            .unwrap();
        assert!([10, 11].contains(&source.definition().get()));
    }
}

#[test]
fn dot_family_catalog_rejects_non_damage_families_and_missing_exclusions() {
    for (category, damaging) in [
        (EffectCategory::Buff, false),
        (EffectCategory::Dot, false),
        (EffectCategory::Control, true),
    ] {
        let mut builder = CombatCatalogBuilder::from_catalog(&catalog(), [0xb4; 32]);
        builder.add_effect(
            EffectDefinition::new(definition(10), vec![], vec![])
                .with_runtime(runtime(category, damaging))
                .with_dot_family(DotFamily::Burn),
        );
        assert_eq!(
            builder.build().unwrap_err().kind(),
            CatalogBuildErrorKind::InvalidDefinition
        );
    }
    for ir in [false, true] {
        let invalid = detonation(
            DotDetonationFilter::family(DotFamily::Burn).excluding(definition(999)),
            None,
            false,
        );
        assert!(builder(invalid, ir).build().is_err());
    }
}
