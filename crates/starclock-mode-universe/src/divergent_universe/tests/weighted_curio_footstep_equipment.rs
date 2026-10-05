//! Both-family equipment handoffs over retained authored bindings.
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture,
    tests::{
        curio_battle_grants::ready,
        curio_battle_stats::assemble,
        weighted_curio_footstep_equipment_fixture::{
            DAMAGE, HEAL, LOSS, cast, damage, probe, stacks,
        },
        weighted_curio_footstep_fixture::{concede, remainder},
        weighted_curio_overflow_fixture::start,
    },
    weighted_curio::WeightedCurioSlotLimit,
};
use starclock_build::light_cone::CombatPath;
use starclock_combat::{Battle, BattleSeed, Command, DecisionId, Scalar, TeamSide};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use starclock_replay::battle_event::encode_battle_event_payload;
use std::sync::Arc;

#[test]
fn weighted_curio_footstep_equipment_handoffs_both_families_execute_authored_original_policies() {
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1107, 1402, 1009, 1002])
            .unwrap();
    let runtime = fixture.factory().weighted_curio_runtime().unwrap();
    let definition = &fixture
        .factory()
        .decision_catalog()
        .weighted_curio_footsteps()[0];
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let (flow, mut activity) = ready(&fixture, family);
        let stale = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                stale,
                WeightedCurioSlotLimit::new(1).unwrap(),
                std::slice::from_ref(&definition.weighted_curio),
            )
            .unwrap();
        let before = activity.canonical_state_bytes();
        let debug = activity.debug_view();
        assert!(
            runtime
                .replace_accepted_loadout(
                    &flow,
                    &mut activity,
                    stale,
                    WeightedCurioSlotLimit::new(1).unwrap(),
                    &[]
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(activity.debug_view(), debug);
        let source = assemble(&fixture, &flow, &activity);
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(activity.debug_view(), debug);
        let paths = source
            .battle_spec()
            .participants()
            .iter()
            .filter(|p| p.side() == TeamSide::Player)
            .map(|p| {
                fixture
                    .core()
                    .build_catalog()
                    .character(p.combatant().form())
                    .unwrap()
                    .path()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            paths,
            [
                CombatPath::Destruction,
                CombatPath::Remembrance,
                CombatPath::Harmony,
                CombatPath::Hunt
            ]
        );
        let mut actual = Battle::create(
            Arc::clone(source.combat_catalog()),
            source.battle_spec().clone(),
            BattleSeed::new([0xb5; 32]),
        )
        .unwrap();
        start(&mut actual);
        concede(&mut actual);
        for owner in 1..=4 {
            let run = || {
                let mut battle = probe(&source);
                assert_eq!(stacks(&battle, owner), 0);
                let mut events = cast(&mut battle, owner, LOSS);
                events.extend(cast(&mut battle, owner, HEAL));
                events.extend(cast(&mut battle, owner, LOSS));
                assert_eq!(
                    remainder(&battle, owner),
                    if owner <= 2 {
                        Scalar::checked_from_integer(10).unwrap()
                    } else {
                        Scalar::ZERO
                    }
                );
                assert_eq!(
                    battle.view().team(TeamSide::Player).skill_points(),
                    u16::from(owner <= 2)
                );
                assert_eq!(stacks(&battle, owner), if owner <= 2 { 3 } else { 0 });
                assert_eq!(
                    damage(&cast(&mut battle, owner, DAMAGE)),
                    [if owner <= 2 { 124 } else { 100 }]
                );
                for other in 1..=4 {
                    if other != owner {
                        assert_eq!(stacks(&battle, other), 0);
                    }
                }
                assert_eq!(battle.view().rng_draw_count(), 0);
                let hash = battle.state_hash();
                assert!(
                    battle
                        .apply(Command::Concede {
                            decision: DecisionId::new(999_999).unwrap()
                        })
                        .is_err()
                );
                assert_eq!(battle.state_hash(), hash);
                let payloads = events
                    .iter()
                    .map(|event| encode_battle_event_payload(event).unwrap())
                    .collect::<Vec<_>>();
                concede(&mut battle);
                assert_eq!(stacks(&battle, owner), 0);
                assert_eq!(remainder(&battle, owner), Scalar::ZERO);
                (events, payloads, battle.state_hash())
            };
            assert_eq!(run(), run());
        }
        let hash = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                hash,
                WeightedCurioSlotLimit::new(1).unwrap(),
                &[],
            )
            .unwrap();
        let empty = assemble(&fixture, &flow, &activity);
        assert_ne!(empty.battle_spec(), source.battle_spec());
        let mut battle = probe(&empty);
        cast(&mut battle, 1, LOSS);
        cast(&mut battle, 1, HEAL);
        cast(&mut battle, 1, LOSS);
        assert_eq!(battle.view().team(TeamSide::Player).skill_points(), 0);
        assert_eq!(stacks(&battle, 1), 0);
        assert_eq!(damage(&cast(&mut battle, 1, DAMAGE)), [100]);
    }
}
