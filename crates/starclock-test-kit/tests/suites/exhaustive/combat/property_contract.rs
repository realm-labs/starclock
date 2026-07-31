use proptest::{
    collection::{btree_map, vec},
    prelude::*,
    test_runner::{Config as ProptestConfig, FileFailurePersistence, RngAlgorithm, RngSeed},
};
use starclock_combat::{
    DispelCategory, DurationClock, EffectCategory, EffectRuntimeTemplate, EffectStackPolicy,
    EffectTickPhase, NumericError, ProgramId, Ratio, Rounding, Scalar, UnitDefinitionId,
    catalog::{
        action::{
            AbilityProgramBinding, AbilityProgramTiming, TargetPattern, TargetRelation,
            UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::UnitDefinition,
    },
    rng::{
        engine::DeterministicRng,
        types::{DrawPurpose, RngError, RngSeed as CombatRngSeed},
    },
    rule::model::ValueExpr,
};

const ARITHMETIC_SEED: u64 = 0x6e75_6d65_7269_6321;
const RNG_SEED: u64 = 0x7261_6e67_652d_6d61;
const CATALOG_SEED: u64 = 0x6361_7461_6c6f_6721;
const SELECTOR_TIMING_SEED: u64 = 0x7365_6c65_6374_6f72;

fn property_config(seed: u64) -> ProptestConfig {
    ProptestConfig {
        cases: 256,
        max_shrink_iters: 4_096,
        failure_persistence: Some(Box::new(FileFailurePersistence::SourceParallel(
            "proptest-regressions",
        ))),
        rng_algorithm: RngAlgorithm::ChaCha,
        rng_seed: RngSeed::Fixed(seed),
        ..ProptestConfig::default()
    }
}

proptest! {
    #![proptest_config(property_config(SELECTOR_TIMING_SEED))]

    #[test]
    fn selector_and_timing_constructors_reject_every_invalid_generated_boundary(
        relation_raw in 0_u8..3,
        pattern_raw in 0_u8..3,
        timing_raw in 0_u8..5,
        sequence in any::<u16>(),
        stack_limit in any::<u16>(),
        permanent in any::<bool>(),
        has_duration in any::<bool>(),
    ) {
        let relation = [TargetRelation::SelfUnit, TargetRelation::Allied, TargetRelation::Opposing]
            [usize::from(relation_raw)];
        let pattern = [TargetPattern::Single, TargetPattern::Blast, TargetPattern::All]
            [usize::from(pattern_raw)];
        let selector = UnitTargetSelector::new(relation, pattern);
        prop_assert_eq!(selector.is_some(), relation != TargetRelation::SelfUnit || pattern == TargetPattern::Single);
        if let Some(selector) = selector {
            prop_assert_eq!(selector.relation(), relation);
            prop_assert_eq!(selector.pattern(), pattern);
        }

        let timing = [
            AbilityProgramTiming::Entry,
            AbilityProgramTiming::BeforeHits,
            AbilityProgramTiming::Hits,
            AbilityProgramTiming::AfterHits,
            AbilityProgramTiming::Resolved,
        ][usize::from(timing_raw)];
        let binding = AbilityProgramBinding::new(sequence, timing, ProgramId::new(1).unwrap());
        prop_assert_eq!(binding.is_some(), sequence != 0);
        if let Some(binding) = binding {
            prop_assert_eq!(binding.sequence(), sequence);
            prop_assert_eq!(binding.timing(), timing);
        }

        let clock = if permanent { DurationClock::Permanent } else { DurationClock::TargetTurnEnd };
        let duration = has_duration.then_some(ValueExpr::EventId);
        let template = EffectRuntimeTemplate::new(
            EffectCategory::Buff,
            DispelCategory::NonDispellable,
            stack_limit,
            duration,
            clock,
            EffectTickPhase::None,
            EffectStackPolicy::Refresh,
        );
        prop_assert_eq!(template.is_some(), stack_limit != 0 && permanent != has_duration);
    }
}

fn id(raw: u32) -> UnitDefinitionId {
    UnitDefinitionId::try_from(raw).expect("generated IDs are non-zero")
}

fn build_unit_catalog(entries: &[(u32, u16)]) -> Vec<u32> {
    let mut builder = CombatCatalogBuilder::new("property-catalog-v1", [0x93; 32]);
    for (raw, _) in entries {
        builder.add_unit(UnitDefinition::new(id(*raw), vec![], vec![]));
    }
    builder
        .build()
        .expect("unique units form a valid catalog")
        .unit_ids()
        .map(UnitDefinitionId::get)
        .collect()
}

proptest! {
    #![proptest_config(property_config(ARITHMETIC_SEED))]

    #[test]
    fn fixed_point_operations_preserve_ordering_and_rounding_duals(
        left_raw in -1_000_000_000_000_i64..=1_000_000_000_000,
        right_raw in -1_000_000_000_000_i64..=1_000_000_000_000,
        divisor_raw in prop_oneof![1_i64..=1_000_000, -1_000_000_i64..=-1],
    ) {
        let left = Scalar::from_scaled(left_raw);
        let right = Scalar::from_scaled(right_raw);
        let divisor = Scalar::from_scaled(divisor_raw);

        prop_assert_eq!(left.checked_add(right), right.checked_add(left));
        prop_assert_eq!(
            left.checked_add(right).and_then(|sum| sum.checked_sub(right)),
            Ok(left)
        );
        prop_assert_eq!(
            left.checked_mul(Scalar::ONE, Rounding::NearestTiesEven),
            Ok(left)
        );

        for mode in [
            Rounding::Floor,
            Rounding::Ceil,
            Rounding::TowardZero,
            Rounding::AwayFromZero,
            Rounding::NearestTiesAway,
            Rounding::NearestTiesEven,
        ] {
            prop_assert_eq!(
                left.checked_mul(right, mode),
                right.checked_mul(left, mode),
                "multiplication must be commutative for {:?}",
                mode
            );
        }

        let product_floor = left.checked_mul(right, Rounding::Floor).unwrap();
        let product_ceil = left.checked_mul(right, Rounding::Ceil).unwrap();
        prop_assert!(product_floor <= product_ceil);
        let negated_left = left.checked_neg().unwrap();
        prop_assert_eq!(
            negated_left.checked_mul(right, Rounding::Floor),
            left.checked_mul(right, Rounding::Ceil)
                .and_then(Scalar::checked_neg)
        );

        let quotient_floor = left.checked_div(divisor, Rounding::Floor).unwrap();
        let quotient_ceil = left.checked_div(divisor, Rounding::Ceil).unwrap();
        prop_assert!(quotient_floor <= quotient_ceil);
        prop_assert!(quotient_ceil.scaled() - quotient_floor.scaled() <= 1);
        prop_assert_eq!(
            left.checked_div(Scalar::ZERO, Rounding::Floor),
            Err(NumericError::DivisionByZero)
        );

        let ratio = Ratio::from_scaled(right_raw);
        prop_assert_eq!(
            ratio.checked_apply(left, Rounding::NearestTiesEven),
            left.checked_mul(right, Rounding::NearestTiesEven)
        );
    }
}

proptest! {
    #![proptest_config(property_config(RNG_SEED))]

    #[test]
    fn range_and_weight_mapping_are_bounded_reproducible_and_draw_accounted(
        seed in any::<[u8; 32]>(),
        upper in 1_u32..=u32::MAX,
        weights in vec(0_u64..=1_000_000, 0..64),
        overflow_tail in 1_u64..=u64::MAX,
    ) {
        let seed = CombatRngSeed::new(seed);
        let mut first = DeterministicRng::from_seed(seed);
        let mut second = DeterministicRng::from_seed(seed);

        let first_range = first
            .choose_index(DrawPurpose::BOUNCE_TARGET, upper)
            .unwrap()
            .expect("positive candidate count selects");
        let second_range = second
            .choose_index(DrawPurpose::BOUNCE_TARGET, upper)
            .unwrap()
            .expect("positive candidate count selects");
        prop_assert_eq!(first_range, second_range);
        prop_assert!(first_range.value() < u64::from(upper));
        prop_assert_eq!(first_range.upper(), u64::from(upper));
        prop_assert_eq!(first_range.sample().purpose(), DrawPurpose::BOUNCE_TARGET);
        prop_assert_eq!(
            first.draw_count(),
            u64::from(first_range.rejected_draws()) + 1
        );

        let mut weighted_first = DeterministicRng::from_seed(seed);
        let mut weighted_second = DeterministicRng::from_seed(seed);
        let selection = weighted_first
            .choose_weighted(DrawPurpose::AGGRO_TARGET, &weights)
            .unwrap();
        prop_assert_eq!(
            selection,
            weighted_second
                .choose_weighted(DrawPurpose::AGGRO_TARGET, &weights)
                .unwrap()
        );
        let total = weights.iter().sum::<u64>();
        match selection {
            None => {
                prop_assert_eq!(total, 0);
                prop_assert_eq!(weighted_first.draw_count(), 0);
            }
            Some(value) => {
                let index = value.index() as usize;
                prop_assert!(index < weights.len());
                prop_assert!(weights[index] > 0);
                prop_assert_eq!(value.range().upper(), total);
                prop_assert!(value.range().value() < total);
                prop_assert_eq!(
                    weighted_first.draw_count(),
                    u64::from(value.range().rejected_draws()) + 1
                );
            }
        }

        let mut no_draw = DeterministicRng::from_seed(seed);
        prop_assert_eq!(
            no_draw.choose_weighted(
                DrawPurpose::AGGRO_TARGET,
                &[u64::MAX, overflow_tail]
            ),
            Err(RngError::WeightTotalOverflow)
        );
        prop_assert_eq!(no_draw.draw_count(), 0);
        prop_assert_eq!(
            no_draw.choose_index(DrawPurpose::BOUNCE_TARGET, 0),
            Ok(None)
        );
        prop_assert_eq!(no_draw.draw_count(), 0);
    }
}

proptest! {
    #![proptest_config(property_config(CATALOG_SEED))]

    #[test]
    fn catalog_unit_indexes_ignore_every_generated_insertion_order(
        units in btree_map(1_u32..=50_000, any::<u16>(), 1..64),
    ) {
        let mut first = units
            .iter()
            .map(|(raw, order)| (*raw, *order))
            .collect::<Vec<_>>();
        first.sort_by_key(|(raw, order)| (*order, *raw));
        let mut second = first.clone();
        second.reverse();

        let first_ids = build_unit_catalog(&first);
        let second_ids = build_unit_catalog(&second);
        let expected = units.keys().copied().collect::<Vec<_>>();
        prop_assert_eq!(&first_ids, &expected);
        prop_assert_eq!(&second_ids, &expected);
        prop_assert_eq!(first_ids, second_ids);
    }
}
