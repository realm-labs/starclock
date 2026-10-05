//! Real command/ordinary-formula effects of an explicit native Skill policy.
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture,
    tests::weighted_curio_footstep_fixture::{
        Probe, begin, cast, cast_as, concede, remainder, scenario,
    },
    weighted_curio_footstep::skill_damage::{SkillDamagePolicy, SkillDamagePolicyError},
};
use starclock_build::light_cone::CombatPath;
use starclock_combat::{
    Battle, BattleEvent, BattleEventKind, Command, EffectEventData, Scalar, TeamSide,
};
use starclock_replay::battle_event::encode_battle_event_payload;

fn policy() -> SkillDamagePolicy {
    SkillDamagePolicy::new(Scalar::from_scaled(80_000), 10, [0xa1; 32]).unwrap()
}
fn probe() -> Probe {
    Probe {
        skill: Some(policy()),
        ..Probe::default()
    }
}
fn stacks(battle: &Battle, owner: u64) -> u16 {
    battle
        .view()
        .effects_by_id()
        .filter(|effect| {
            effect.target().get() == owner
                && (0x7f2c_0000..0x7f2d_0000).contains(&effect.definition().get())
        })
        .map(|effect| effect.stacks())
        .sum()
}
fn damage(events: &[BattleEvent]) -> Vec<i64> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) if data.target.get() == 3 => Some(data.applied.get()),
            _ => None,
        })
        .collect()
}

#[test]
fn weighted_curio_footstep_skill_damage_grants_after_complete_action_not_per_hit_and_caps() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let run = || {
        let mut battle = scenario(&fixture, &probe());
        begin(&mut battle);
        assert_eq!(stacks(&battle, 1), 0);
        let mut events = cast(&mut battle, 11, 3);
        assert_eq!(damage(&events), [100, 100, 100]);
        assert_eq!(stacks(&battle, 1), 1);
        assert_eq!(damage(&cast(&mut battle, 12, 3)), [108]);
        assert_eq!(stacks(&battle, 1), 1); // A Basic receives but cannot acquire stacks.
        for before in 1..=11 {
            let current = cast(&mut battle, 11, 3);
            assert_eq!(damage(&current), [100 + 8 * before.min(10); 3]);
            assert_eq!(
                stacks(&battle, 1),
                u16::try_from((before + 1).min(10)).unwrap()
            );
            events.extend(current);
        }
        assert_eq!(damage(&cast(&mut battle, 12, 3)), [180]);
        assert_eq!(battle.view().rng_draw_count(), 0);
        let payloads = events
            .iter()
            .map(|event| encode_battle_event_payload(event).unwrap())
            .collect::<Vec<_>>();
        (events, payloads, battle.state_hash())
    };
    assert_eq!(run(), run());
}

#[test]
fn weighted_curio_footstep_skill_damage_support_skills_and_hp_return_compose_without_double_counting()
 {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let mut battle = scenario(&fixture, &probe());
    begin(&mut battle);
    cast(&mut battle, 7, 1); // Consume/heal/consume: two HP facts, one Skill envelope.
    assert_eq!(stacks(&battle, 1), 1);
    assert_eq!(
        remainder(&battle, 1),
        Scalar::checked_from_integer(10).unwrap()
    );
    assert_eq!(battle.view().team(TeamSide::Player).skill_points(), 1);
    cast(&mut battle, 2, 1); // Healing-only Skill still counts.
    assert_eq!(stacks(&battle, 1), 2);
    assert_eq!(damage(&cast(&mut battle, 12, 3)), [116]);
    assert_eq!(stacks(&battle, 1), 2);
}

#[test]
fn weighted_curio_footstep_skill_damage_qualifies_original_paths_and_blocks_inherited_other_owners()
{
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for path in [
        CombatPath::Destruction,
        CombatPath::Remembrance,
        CombatPath::Hunt,
        CombatPath::Erudition,
        CombatPath::Harmony,
        CombatPath::Nihility,
        CombatPath::Preservation,
        CombatPath::Abundance,
        CombatPath::Elation,
    ] {
        let eligible = matches!(path, CombatPath::Destruction | CombatPath::Remembrance);
        let mut battle = scenario(
            &fixture,
            &Probe {
                path,
                inherit_buddy: true,
                ..probe()
            },
        );
        begin(&mut battle);
        cast_as(&mut battle, 2, 11, 3);
        assert_eq!(stacks(&battle, 2), 0);
        assert_eq!(stacks(&battle, 1), 0);
        cast(&mut battle, 11, 3);
        assert_eq!(stacks(&battle, 1), u16::from(eligible));
        assert_eq!(
            damage(&cast(&mut battle, 12, 3)),
            [if eligible { 108 } else { 100 }]
        );
        assert_eq!(damage(&cast_as(&mut battle, 2, 12, 3)), [100]);
    }
}

#[test]
fn weighted_curio_footstep_skill_damage_independent_formation_one_does_not_borrow_first_owner_stacks()
 {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let mut battle = scenario(
        &fixture,
        &Probe {
            bind_buddy: true,
            ..probe()
        },
    );
    begin(&mut battle);
    cast(&mut battle, 11, 3);
    assert_eq!(stacks(&battle, 1), 1);
    assert_eq!(stacks(&battle, 2), 0);
    assert_eq!(damage(&cast_as(&mut battle, 2, 12, 3)), [100]);
    cast_as(&mut battle, 2, 11, 3);
    cast_as(&mut battle, 2, 11, 3);
    assert_eq!(stacks(&battle, 1), 1);
    assert_eq!(stacks(&battle, 2), 2);
    assert_eq!(damage(&cast_as(&mut battle, 2, 12, 3)), [116]);
    assert_eq!(damage(&cast(&mut battle, 12, 3)), [108]);
}

#[test]
fn weighted_curio_footstep_skill_damage_uses_named_common_channels_not_true_damage_and_tears_down()
{
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let mut battle = scenario(&fixture, &probe());
    begin(&mut battle);
    cast(&mut battle, 11, 3);
    for ability in [12, 14, 16] {
        assert_eq!(damage(&cast(&mut battle, ability, 3)), [108]);
    }
    assert_eq!(damage(&cast(&mut battle, 13, 3)), [100]);
    let removed = cast(&mut battle, 17, 1);
    assert!(removed.iter().any(|event| matches!(event.kind(), BattleEventKind::Effect(EffectEventData::Removed {definition, ..}) if definition.get()==0x7f2c_0010)));
    assert_eq!(stacks(&battle, 1), 0);
    assert_eq!(damage(&cast(&mut battle, 12, 3)), [100]);
    cast(&mut battle, 11, 3);
    assert_eq!(stacks(&battle, 1), 1);
    assert_eq!(damage(&cast(&mut battle, 12, 3)), [108]);
}

#[test]
fn weighted_curio_footstep_skill_damage_rejection_terminal_cleanup_and_fresh_reset_are_real() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let mut battle = scenario(&fixture, &probe());
    let stale = Command::StartBattle {
        decision: battle.decision().unwrap().id(),
    };
    begin(&mut battle);
    cast(&mut battle, 11, 3);
    let before = battle.state_hash();
    assert!(battle.apply(stale).is_err());
    assert_eq!(battle.state_hash(), before);
    assert_eq!(stacks(&battle, 1), 1);
    concede(&mut battle);
    assert_eq!(stacks(&battle, 1), 0);
    let mut won = scenario(&fixture, &probe());
    begin(&mut won);
    cast(&mut won, 11, 3);
    cast(&mut won, 18, 3);
    assert_eq!(stacks(&won, 1), 0);
    let fresh = scenario(&fixture, &probe());
    assert_eq!(stacks(&fresh, 1), 0);
}

#[test]
fn weighted_curio_footstep_skill_damage_validates_operands_and_hashes_each_policy_input() {
    assert_eq!(
        SkillDamagePolicy::new(Scalar::ZERO, 10, [0; 32]).unwrap_err(),
        SkillDamagePolicyError::InvalidDamagePerStack
    );
    assert_eq!(
        SkillDamagePolicy::new(Scalar::from_scaled(-1), 10, [0; 32]).unwrap_err(),
        SkillDamagePolicyError::InvalidDamagePerStack
    );
    assert_eq!(
        SkillDamagePolicy::new(Scalar::ONE, 0, [0; 32]).unwrap_err(),
        SkillDamagePolicyError::InvalidMaximumStacks
    );
    assert_eq!(
        SkillDamagePolicy::new(Scalar::from_scaled(i64::MAX), 2, [0; 32]).unwrap_err(),
        SkillDamagePolicyError::UnrepresentableMaximumBonus
    );
    SkillDamagePolicy::new(Scalar::from_scaled(1), u16::MAX, [0; 32]).unwrap();
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let initial = scenario(&fixture, &probe()).state_hash();
    for (amount, cap, identity) in [
        (80_001, 10, [0xa1; 32]),
        (80_000, 9, [0xa1; 32]),
        (80_000, 10, [0xa2; 32]),
    ] {
        let changed = scenario(
            &fixture,
            &Probe {
                skill: Some(
                    SkillDamagePolicy::new(Scalar::from_scaled(amount), cap, identity).unwrap(),
                ),
                ..Probe::default()
            },
        );
        assert_ne!(initial, changed.state_hash());
    }
}

#[test]
fn weighted_curio_footstep_skill_damage_checked_formula_fault_retains_prior_stacks_and_hp() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let mut battle = scenario(
        &fixture,
        &Probe {
            skill: Some(
                SkillDamagePolicy::new(Scalar::from_scaled(i64::MAX), 1, [0xa3; 32]).unwrap(),
            ),
            ..Probe::default()
        },
    );
    begin(&mut battle);
    cast(&mut battle, 2, 1); // A support Skill can attach the valid one-stack ratio.
    assert_eq!(stacks(&battle, 1), 1);
    let mut attack = None;
    for _ in 0..32 {
        let offered = battle.decision().map(|decision| decision.legal_commands());
        attack = offered.and_then(|commands| commands.iter().find(|command| matches!(command,
            Command::UseAbility {actor,ability,primary_target,..}
                if actor.get()==1 && ability.get()==12 && primary_target.is_some_and(|target|target.get()==3)
        ))).cloned();
        if attack.is_some() {
            break;
        }
        let command = offered
            .and_then(|commands| {
                commands.iter().find(|command| {
                    matches!(command,
                        Command::UseAbility {ability,..} if ability.get()==10
                    )
                })
            })
            .cloned()
            .or_else(|| battle.advance_command())
            .unwrap();
        let result = battle.apply(command).unwrap();
        assert!(result.fault().is_none());
    }
    let before_hp = battle
        .view()
        .units_by_id()
        .map(|unit| (unit.id(), unit.current_hp()))
        .collect::<Vec<_>>();
    let result = battle.apply(attack.unwrap()).unwrap();
    assert!(result.fault().is_some());
    assert!(!result.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Damage(_) | BattleEventKind::Effect(_) | BattleEventKind::RuleState(_)
    )));
    assert_eq!(stacks(&battle, 1), 1);
    assert_eq!(
        before_hp,
        battle
            .view()
            .units_by_id()
            .map(|unit| (unit.id(), unit.current_hp()))
            .collect::<Vec<_>>()
    );
    assert_eq!(battle.view().rng_draw_count(), 0);
}
