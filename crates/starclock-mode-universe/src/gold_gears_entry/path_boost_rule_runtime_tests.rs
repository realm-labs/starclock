use std::collections::BTreeMap;

use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityRngContext,
    ActivityRngStreams, ActivitySlotId, ActivityTransactionOutcome, ActivityTransactionState,
    ActivityValue,
};
use starclock_combat::{
    ModifierInstanceId, Scalar, UnitId,
    modifier::{
        model::{
            ActiveModifier, FormulaModifierQuery, FormulaPurpose, FormulaStage, FormulaSubject,
            ModifierQueryContext, StatKind, StatQuery,
        },
        registry::ModifierRegistry,
        resolve::StatResolver,
    },
    rule::model::SourceClass,
};

use crate::digest::Encoder;

use super::{
    CONUNDRUM_AREA_KEY, GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance,
    path_boost_rule_runtime::{
        GOLD_AND_GEARS_PATH_BOOST_EXECUTION_REVISION, GoldAndGearsPathBoostCombatSet,
        GoldAndGearsPathBoostRuleKind, GoldAndGearsPathBoostRuleOwnership,
    },
    progression_runtime::GoldAndGearsPathBoostStat,
    state_layout::{PROGRESSION_DICE_PATH_BOOST_STACKS_KEY, PROGRESSION_SLOT},
    tests::{compiled_fixture, entry},
};

#[test]
fn path_boost_partition_binds_exactly_495_terminal_rules() {
    let factory = factory();
    let bindings = factory.path_boost_rule_bindings();
    assert_eq!(bindings.len(), 495);
    assert!(
        bindings
            .windows(2)
            .all(|pair| pair[0].rule_id() < pair[1].rule_id())
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| binding.kind() == GoldAndGearsPathBoostRuleKind::PathBoost)
            .count(),
        9
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| {
                binding.kind() == GoldAndGearsPathBoostRuleKind::BlessingDefinition
            })
            .count(),
        162
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| binding.kind() == GoldAndGearsPathBoostRuleKind::BlessingLevel)
            .count(),
        324
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| { binding.ownership() == GoldAndGearsPathBoostRuleOwnership::Shared })
            .count(),
        486
    );
    assert!(bindings.iter().all(|binding| {
        binding.accuracy() == "ExactPublic"
            && binding.executor()
                == if binding.ownership() == GoldAndGearsPathBoostRuleOwnership::Shared {
                    "ReleasedSharedExecutor"
                } else {
                    "CombatRuleIr"
                }
    }));
    assert_eq!(
        GOLD_AND_GEARS_PATH_BOOST_EXECUTION_REVISION,
        "gold-and-gears-path-boost-execution-v1"
    );
    assert_eq!(
        hex(factory.path_boost_execution_digest()),
        "7d51e9f2f62e5a264d1c63480f78aa97e71b4ce6073f2a4e12ad5c16843761ee"
    );
}

#[test]
fn all_486_shared_blessing_rules_execute_through_the_released_runtime() {
    let instance = compiled_fixture(factory());
    let blessing_ids = instance
        .content_runtime
        .blessings
        .definitions()
        .iter()
        .map(|definition| definition.blessing())
        .collect::<Vec<_>>();
    assert_eq!(blessing_ids.len(), 162);

    let mut state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    );
    for blessing in &blessing_ids {
        commit(
            &instance,
            &mut state,
            instance.compile_blessing_acquisition(*blessing).unwrap(),
        );
    }
    let level_one = instance
        .blessing_contributions(
            &blessing_ids
                .iter()
                .map(|blessing| (*blessing, 1))
                .collect::<Vec<_>>(),
        )
        .unwrap();
    assert_eq!(level_one.entries().len(), 162);
    assert!(
        level_one
            .entries()
            .iter()
            .all(|entry| entry.level().level() == 1)
    );

    for blessing in &blessing_ids {
        commit(
            &instance,
            &mut state,
            instance.compile_blessing_enhancement(*blessing).unwrap(),
        );
    }
    let level_two = instance
        .blessing_contributions(
            &blessing_ids
                .iter()
                .map(|blessing| (*blessing, 2))
                .collect::<Vec<_>>(),
        )
        .unwrap();
    assert_eq!(level_two.entries().len(), 162);
    assert!(
        level_two
            .entries()
            .iter()
            .all(|entry| entry.level().level() == 2)
    );
    assert_eq!(state.command_sequence(), 324);
    assert_ne!(level_one.digest(), level_two.digest());
    let rng = activity_rng(&instance, 14_508);
    assert_eq!(
        rng.snapshots()
            .iter()
            .map(|entry| entry.draw_count())
            .sum::<u64>(),
        0
    );
    assert_eq!(
        hex(level_one.digest()),
        "566a62d4d53f184a8bf2cbc92676baaa647c969ab33b52bd4dc71d8aef73f06e"
    );
    assert_eq!(
        hex(level_two.digest()),
        "a5c9a0504320061814481792e9d5119e1d245fb8932ca37fc99f695850e60663"
    );
    assert_eq!(
        state_hash(&instance, &state, &rng),
        "f68cc10352f98866a48a26092390b40dc4f40aa89dc08948ce495aa9d124af88"
    );
}

#[test]
fn all_nine_path_boost_rules_execute_through_combat_modifiers() {
    let factory = factory();
    let paths = factory
        .unique
        .paths
        .iter()
        .map(|path| path.identity.stable_key.to_string())
        .collect::<Vec<_>>();
    let mut observed = Vec::new();
    let mut modifier_definitions = 0;
    let mut combat_digests = Vec::new();
    for path in paths {
        let instance = compile_path(factory, &path);
        let state = state_with_boost_stacks(&instance, 3);
        let set = instance.compile_path_boost_combat_set(&state).unwrap();
        assert!(set.digest().iter().any(|byte| *byte != 0));
        assert_eq!(
            set.binding().ratio_scaled(),
            instance.dice_path_boost_value_scaled() * 3
        );
        assert_eq!(set.binding().source().class(), SourceClass::Mode);
        execute_set(&set);
        modifier_definitions += set.binding().definitions().len();
        combat_digests.push((set.binding().source_rule_id().to_owned(), set.digest()));
        observed.push((
            set.binding().source_rule_id().to_owned(),
            set.binding().stat(),
        ));
    }
    observed.sort_unstable();
    assert_eq!(observed.len(), 9);
    assert_eq!(modifier_definitions, 16);
    combat_digests.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(
        hex(combat_digest(&combat_digests)),
        "5664b8314469e7d88551ded855becddd7b457b50771452bfdf28ddb739b56df7"
    );
    assert_eq!(
        observed
            .iter()
            .map(|(rule, _)| rule.as_str())
            .collect::<Vec<_>>(),
        (650_100..=650_108)
            .map(|source| format!("gold-gears.rule.path-boost.{source}"))
            .collect::<Vec<_>>()
    );
}

#[test]
fn path_boost_rejections_and_filters_fail_closed() {
    let factory = factory();
    let follow_up_path = factory
        .unique
        .paths
        .iter()
        .find(|path| {
            let instance = compile_path(factory, &path.identity.stable_key);
            let state = state_with_boost_stacks(&instance, 1);
            instance
                .compile_path_boost_combat_set(&state)
                .is_ok_and(|set| {
                    set.binding().stat() == GoldAndGearsPathBoostStat::FollowUpAttackDamage
                })
        })
        .unwrap();
    let instance = compile_path(factory, &follow_up_path.identity.stable_key);
    assert!(try_state_with_boost_stacks(&instance, -1).is_none());

    let set = instance
        .compile_path_boost_combat_set(&state_with_boost_stacks(&instance, 2))
        .unwrap();
    let (registry, instances) = registry_and_instances(&set);
    let bases = BTreeMap::new();
    let resolver = StatResolver::new(&registry, &bases, &instances);
    let query = FormulaModifierQuery {
        subject: UnitId::new(1).unwrap(),
        stage: FormulaStage::DamageBoost,
        purpose: FormulaPurpose::OrdinaryDamage,
    };
    assert_eq!(
        resolver
            .query_formula(query, &ModifierQueryContext::default())
            .unwrap(),
        Scalar::ZERO
    );
    let context = ModifierQueryContext {
        ability_tags: vec!["follow_up".into()].into_boxed_slice(),
        formula_subject: Some(FormulaSubject::Source),
        ..ModifierQueryContext::default()
    };
    assert_eq!(
        resolver.query_formula(query, &context).unwrap().scaled(),
        set.binding().ratio_scaled()
    );
}

fn execute_set(set: &GoldAndGearsPathBoostCombatSet) {
    let unit = UnitId::new(1).unwrap();
    let (registry, instances) = registry_and_instances(set);
    let bases = BTreeMap::from([
        ((unit, StatKind::EffectHitRate), Scalar::ZERO),
        ((unit, StatKind::OutgoingHealing), Scalar::ZERO),
        ((unit, StatKind::CritDamage), Scalar::ZERO),
    ]);
    let resolver = StatResolver::new(&registry, &bases, &instances);
    let ratio = set.binding().ratio_scaled();
    match set.binding().stat() {
        GoldAndGearsPathBoostStat::EffectHitRate
        | GoldAndGearsPathBoostStat::OutgoingHealing
        | GoldAndGearsPathBoostStat::CriticalDamage => {
            let stat = match set.binding().stat() {
                GoldAndGearsPathBoostStat::EffectHitRate => StatKind::EffectHitRate,
                GoldAndGearsPathBoostStat::OutgoingHealing => StatKind::OutgoingHealing,
                GoldAndGearsPathBoostStat::CriticalDamage => StatKind::CritDamage,
                _ => unreachable!("closed stat boost set"),
            };
            assert_eq!(
                resolver
                    .query(
                        StatQuery {
                            subject: unit,
                            stat,
                            purpose: FormulaPurpose::Stat,
                        },
                        &ModifierQueryContext::default(),
                    )
                    .unwrap()
                    .scaled(),
                ratio
            );
        }
        GoldAndGearsPathBoostStat::ShieldGain => {
            assert_formula(
                &resolver,
                FormulaStage::Shield,
                FormulaPurpose::Shield,
                None,
                ratio,
            );
        }
        GoldAndGearsPathBoostStat::DamageOverTime => assert_formula(
            &resolver,
            FormulaStage::DamageBoost,
            FormulaPurpose::Dot,
            None,
            ratio,
        ),
        GoldAndGearsPathBoostStat::DamageDealt => {
            for purpose in [
                FormulaPurpose::OrdinaryDamage,
                FormulaPurpose::Dot,
                FormulaPurpose::Break,
                FormulaPurpose::SuperBreak,
                FormulaPurpose::AdditionalDamage,
                FormulaPurpose::JointDamage,
                FormulaPurpose::ElationDamage,
                FormulaPurpose::TrueDamage,
            ] {
                assert_formula(&resolver, FormulaStage::DamageBoost, purpose, None, ratio);
            }
        }
        GoldAndGearsPathBoostStat::FollowUpAttackDamage => assert_formula(
            &resolver,
            FormulaStage::DamageBoost,
            FormulaPurpose::OrdinaryDamage,
            Some("follow_up"),
            ratio,
        ),
        GoldAndGearsPathBoostStat::BasicAttackDamage => assert_formula(
            &resolver,
            FormulaStage::DamageBoost,
            FormulaPurpose::OrdinaryDamage,
            Some("basic"),
            ratio,
        ),
        GoldAndGearsPathBoostStat::UltimateDamage => assert_formula(
            &resolver,
            FormulaStage::DamageBoost,
            FormulaPurpose::OrdinaryDamage,
            Some("ultimate"),
            ratio,
        ),
    }
}

fn assert_formula(
    resolver: &StatResolver<'_>,
    stage: FormulaStage,
    purpose: FormulaPurpose,
    tag: Option<&str>,
    expected: i64,
) {
    let context = ModifierQueryContext {
        ability_tags: tag
            .map(|tag| vec![Box::<str>::from(tag)].into_boxed_slice())
            .unwrap_or_default(),
        formula_subject: Some(FormulaSubject::Source),
        ..ModifierQueryContext::default()
    };
    assert_eq!(
        resolver
            .query_formula(
                FormulaModifierQuery {
                    subject: UnitId::new(1).unwrap(),
                    stage,
                    purpose,
                },
                &context,
            )
            .unwrap()
            .scaled(),
        expected
    );
}

fn registry_and_instances(
    set: &GoldAndGearsPathBoostCombatSet,
) -> (ModifierRegistry, Vec<ActiveModifier>) {
    let registry = ModifierRegistry::new(
        set.binding().groups().to_vec(),
        set.binding().definitions().to_vec(),
    )
    .unwrap();
    let unit = UnitId::new(1).unwrap();
    let instances = set
        .binding()
        .definitions()
        .iter()
        .enumerate()
        .map(|(index, definition)| {
            let instance = ModifierInstanceId::new(u64::try_from(index + 1).unwrap()).unwrap();
            ActiveModifier {
                instance,
                definition: definition.id,
                owner: unit,
                subject: unit,
                source: set.binding().source().definition(),
                source_class: set.binding().source().class(),
                insertion_sequence: instance.get(),
                application_action: None,
                source_effect: None,
                slots: Box::new([]),
                captured_value: None,
                captured_stats: Box::new([]),
            }
        })
        .collect();
    (registry, instances)
}

fn factory() -> &'static GoldAndGearsRuntimeFactory {
    super::tests::shared_factory()
}

fn compile_path(factory: &GoldAndGearsRuntimeFactory, path: &str) -> GoldAndGearsRuntimeInstance {
    let dice = &factory.unique.dice[0];
    factory
        .compile_entry(entry(factory, CONUNDRUM_AREA_KEY, path, dice))
        .unwrap()
}

fn state_with_boost_stacks(
    instance: &GoldAndGearsRuntimeInstance,
    stacks: i64,
) -> ActivityTransactionState {
    try_state_with_boost_stacks(instance, stacks).unwrap()
}

fn try_state_with_boost_stacks(
    instance: &GoldAndGearsRuntimeInstance,
    stacks: i64,
) -> Option<ActivityTransactionState> {
    let slot = ActivitySlotId::new(PROGRESSION_SLOT).unwrap();
    let ActivityValue::BoundedCounterMap(initial) = instance
        .state_definition()
        .slots()
        .iter()
        .find(|definition| definition.id() == slot)
        .unwrap()
        .initial()
    else {
        panic!("expected progression counter map");
    };
    let mut values = initial.to_vec();
    let index = values
        .binary_search_by_key(&PROGRESSION_DICE_PATH_BOOST_STACKS_KEY, |(key, _)| *key)
        .unwrap();
    values[index].1 = stacks;
    ActivityTransactionState::new_with_initial_values(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
        vec![(
            slot,
            ActivityValue::BoundedCounterMap(values.into_boxed_slice()),
        )],
    )
    .ok()
}

fn commit(
    instance: &GoldAndGearsRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: starclock_activity::ActivityProgramDefinition,
) {
    program
        .validate_against(instance.state_definition(), instance.graph_definition())
        .unwrap();
    let cause = ActivityCause::new(
        state.command_sequence() + 1,
        program.id(),
        state.current_node(),
    )
    .unwrap();
    assert!(matches!(
        state.apply_program(&program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn identity() -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(14).unwrap(),
        ActivityDefinitionDigest::new([0x14; 32]).unwrap(),
        ActivityConfigDigest::new([0x47; 32]).unwrap(),
    )
}

fn activity_rng(instance: &GoldAndGearsRuntimeInstance, seed: u64) -> ActivityRngStreams {
    let identity = identity();
    ActivityRngStreams::new(ActivityRngContext::new(
        ActivityMasterSeed::from_u64(seed),
        identity.id(),
        identity.definition_digest(),
        identity.config_digest(),
        instance.graph_definition().digest(),
        ActivityInstanceId::new(1).unwrap(),
        None,
        Some(instance.graph_definition().entry()),
        None,
        0,
    ))
}

fn state_hash(
    instance: &GoldAndGearsRuntimeInstance,
    state: &ActivityTransactionState,
    rng: &ActivityRngStreams,
) -> String {
    state
        .state_hash(
            identity(),
            instance.graph_definition(),
            ActivityInstanceId::new(1).unwrap(),
            rng,
        )
        .bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn combat_digest(entries: &[(String, [u8; 32])]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-gold-gears-path-boost-fixture-v1");
    encoder.u32(entries.len() as u32);
    for (rule, digest) in entries {
        encoder.text(rule);
        encoder.digest(*digest);
    }
    encoder.finish()
}

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
