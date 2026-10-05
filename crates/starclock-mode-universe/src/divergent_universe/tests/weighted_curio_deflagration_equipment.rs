//! Accepted equipment, untouched production commands and immutable unequip handoffs.
use crate::divergent_universe::tests::{
    curio_battle_grants::ready,
    curio_battle_stats::assemble,
    weighted_curio_burn_fixture::{Probe, attack, scenario},
    weighted_curio_deflagration_native::{EFFECT, FAMILIES, applications, fixture, payloads},
};
use crate::divergent_universe::weighted_curio::WeightedCurioSlotLimit;
use starclock_combat::{
    Battle, BattleEvent, BattleEventKind, BattleSeed, Command, DamageKind, UnitId,
    catalog::action::AbilityTag,
};
use std::sync::Arc;

fn execute(battle: &mut Battle, command: Command) -> Vec<BattleEvent> {
    let outcome = battle.apply(command).unwrap();
    assert!(outcome.fault().is_none(), "{:?}", outcome.fault());
    outcome.events().to_vec()
}
fn progress(battle: &mut Battle) -> Vec<BattleEvent> {
    let command = if let Some(decision) = battle.decision() {
        decision
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::UseAbility { .. }))
            .unwrap()
            .clone()
    } else {
        Command::Advance {
            boundary: battle.view().action_boundary().unwrap().id(),
        }
    };
    execute(battle, command)
}

#[test]
fn deflagration_equipment_untouched_production_attack_and_tick_execute_in_both_families() {
    let fixture = fixture(1);
    let definition = &fixture
        .factory()
        .decision_catalog()
        .weighted_curio_deflagrations()[0];
    let runtime = fixture.factory().weighted_curio_runtime().unwrap();
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        let hash = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                hash,
                WeightedCurioSlotLimit::new(1).unwrap(),
                std::slice::from_ref(&definition.weighted_curio),
            )
            .unwrap();
        let before = activity.canonical_state_bytes();
        let source = assemble(&fixture, &flow, &activity);
        assert_eq!(activity.canonical_state_bytes(), before);
        let protocol = fixture
            .factory()
            .contribution_snapshot_runtime()
            .unwrap()
            .snapshot(&flow, &activity)
            .unwrap();
        let run = || {
            // No replacement form, action, target, effect or constructor here.
            let mut battle = Battle::create(
                Arc::clone(source.combat_catalog()),
                source.battle_spec().clone(),
                BattleSeed::new([0xd7; 32]),
            )
            .unwrap();
            let owner = UnitId::new(1).unwrap();
            let start = Command::StartBattle {
                decision: battle.decision().unwrap().id(),
            };
            let mut events = execute(&mut battle, start);
            let mut fired = false;
            for _ in 0..128 {
                let offered = battle.decision().and_then(|decision| decision.legal_commands().iter().find(|command|
                    matches!(command, Command::UseAbility { actor, ability, .. } if *actor == owner
                        && source.combat_catalog().ability(*ability).unwrap().action().unwrap().tags().contains(AbilityTag::Attack)))).cloned();
                if let Some(command) = offered {
                    let hit = execute(&mut battle, command);
                    assert!(applications(&hit) > 0);
                    events.extend(hit);
                    fired = true;
                    break;
                }
                events.extend(progress(&mut battle));
            }
            assert!(fired, "production Fire owner received no Attack decision");
            let effects = battle
                .view()
                .effects_by_id()
                .filter(|effect| effect.definition().get() == EFFECT)
                .collect::<Vec<_>>();
            assert!(!effects.is_empty());
            for effect in effects {
                let level = battle
                    .view()
                    .units_by_id()
                    .find(|unit| unit.id() == effect.target())
                    .unwrap()
                    .level()
                    .get();
                let ratio = definition.hp_ratios_millionths[usize::from(level - 1)];
                let factor = 1_000_000
                    + protocol
                        .difficulty_protocol()
                        .maximum_hp_increase_millionths();
                let expected = i128::from(ratio) * 100 * i128::from(factor) / 1_000_000 * 2;
                assert_eq!(i128::from(effect.magnitude().scaled()), expected);
                assert_eq!(effect.applier(), owner);
                assert_eq!(effect.remaining(), Some(2));
            }
            let mut ticked = false;
            for _ in 0..128 {
                let step = progress(&mut battle);
                if step.iter().any(|event| {
                    matches!(event.kind(), BattleEventKind::Damage(data)
                    if data.kind == DamageKind::DotTick)
                        && event.cause().applier() == Some(owner)
                        && event
                            .cause()
                            .source_definition()
                            .is_some_and(|source| source.get() == 0x7f74_0020)
                }) {
                    ticked = true;
                    events.extend(step);
                    break;
                }
                events.extend(step);
            }
            assert!(
                ticked,
                "production target never reached its natural Deflagration tick"
            );
            (
                payloads(&events),
                battle.state_hash(),
                battle.view().rng_draw_count(),
            )
        };
        assert_eq!(run(), run(), "{family:?}");
        assert_eq!(activity.canonical_state_bytes(), before);
    }
}

#[test]
fn deflagration_equipment_unequip_is_atomic_composes_and_preserves_old_assemblies() {
    let fixture = fixture(1);
    let runtime = fixture.factory().weighted_curio_runtime().unwrap();
    let catalog = fixture.factory().decision_catalog();
    let selected = [
        catalog.weighted_curio_deflagrations()[0]
            .weighted_curio
            .clone(),
        catalog.weighted_curio_footsteps()[0].weighted_curio.clone(),
        catalog.weighted_curio_overflows()[0].weighted_curio.clone(),
    ];
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        let unequipped = assemble(&fixture, &flow, &activity);
        let stale = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                stale,
                WeightedCurioSlotLimit::new(3).unwrap(),
                &selected,
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
                    WeightedCurioSlotLimit::new(3).unwrap(),
                    &[]
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(activity.debug_view(), debug);
        let equipped = assemble(&fixture, &flow, &activity);
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_ne!(unequipped.assembly_digest(), equipped.assembly_digest());
        let probe = || {
            let mut test = scenario(
                &equipped,
                Probe {
                    hits: 1,
                    full_party: true,
                    second_owner: true,
                    ..Probe::default()
                },
            );
            let events = attack(&mut test);
            assert_eq!(applications(&events), 1);
            (payloads(&events), test.battle.state_hash())
        };
        let old = probe();
        let hash = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                hash,
                WeightedCurioSlotLimit::new(3).unwrap(),
                &[],
            )
            .unwrap();
        let before = activity.canonical_state_bytes();
        let removed = assemble(&fixture, &flow, &activity);
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_ne!(removed.assembly_digest(), equipped.assembly_digest());
        assert!(
            removed
                .combat_catalog()
                .effect(EFFECT.try_into().unwrap())
                .is_none()
        );
        let mut test = scenario(
            &removed,
            Probe {
                hits: 1,
                full_party: true,
                second_owner: true,
                ..Probe::default()
            },
        );
        assert_eq!(applications(&attack(&mut test)), 0);
        assert_eq!(probe(), old, "old immutable handoff changed after unequip");
        let rebuilt = assemble(&fixture, &flow, &activity);
        assert_eq!(rebuilt.battle_spec(), removed.battle_spec());
        assert_eq!(rebuilt.assembly_digest(), removed.assembly_digest());
    }
}
