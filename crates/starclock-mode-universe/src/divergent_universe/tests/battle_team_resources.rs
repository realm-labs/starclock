//! Production assembly and real resource commands, not Aha lifecycle parity.
use std::sync::Arc;

use crate::divergent_universe::{
    DivergentUniverseAssembledBattle, DivergentUniverseBaselineFixture,
    battle_team_resources::PUNCHLINE,
    tests::{curio_battle_grants::ready, curio_battle_stats::assemble},
};
use starclock_combat::{
    AbilityId, AssemblyDigest, Battle, BattleEvent, BattleEventKind, BattleSeed, BattleSpec,
    CombatantSpecDigest, Command, ConcedePolicy, Energy, FormationIndex, Hp, LifeState,
    ParticipantSource, ParticipantSpec, PresenceState, Ratio, ResolvedCombatantSpec,
    ResolvedDefinitionBindings, ResourceEventData, Scalar, Speed, TeamResourceSpec,
    TeamResourceWavePolicy, TeamSide, UnitId, UnitLevel,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            HitOperationDefinition, OrdinaryDamageDefinition, OrdinaryDamageMultipliers,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, TeamResourceChange,
            TeamResourceChangeDefinition, TeamResourceCost, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
            SelectorDefinition, UnitDefinition,
        },
    },
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
const PARTY: [u32; 4] = [1502, 1005, 1009, 1105];

fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}
fn assembled(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> DivergentUniverseAssembledBattle {
    let (flow, activity) = ready(fixture, family);
    let before = activity.state_hash();
    let result = assemble(fixture, &flow, &activity);
    assert_eq!(activity.state_hash(), before);
    result
}
fn create(assembled: &DivergentUniverseAssembledBattle) -> Battle {
    Battle::create(
        Arc::clone(assembled.combat_catalog()),
        assembled.battle_spec().clone(),
        BattleSeed::new([0xe4; 32]),
    )
    .unwrap()
}
fn balance(battle: &Battle) -> u16 {
    battle
        .view()
        .team(TeamSide::Player)
        .keyed_resource(PUNCHLINE)
        .unwrap()
        .0
}
fn accept(battle: &mut Battle, command: Command) -> Vec<BattleEvent> {
    let result = battle.apply(command).unwrap();
    assert!(result.fault().is_none(), "{:?}", result.fault());
    result.events().to_vec()
}
fn start(battle: &mut Battle) {
    accept(
        battle,
        Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        },
    );
}
fn advance(battle: &mut Battle) {
    let command = if let Some(decision) = battle.decision() {
        decision
            .legal_commands()
            .iter()
            .find(|c| {
                matches!(
                    c,
                    Command::UseAbility { .. }
                        | Command::CommitPreparedAction { .. }
                        | Command::CommitActionFrame { .. }
                )
            })
            .unwrap()
            .clone()
    } else {
        Command::Advance {
            boundary: battle.view().action_boundary().unwrap().id(),
        }
    };
    accept(battle, command);
}

#[test]
fn battle_team_resources_original_elation_roster_enables_one_meter_without_curio() {
    for party in [PARTY, [1502, 1501, 1009, 1105], [1308, 1005, 1009, 1105]] {
        let fixture = DivergentUniverseBaselineFixture::production_for_source_party(party).unwrap();
        for family in FAMILIES {
            let first = assembled(&fixture, family);
            let fresh = assembled(
                &DivergentUniverseBaselineFixture::production_for_source_party(party).unwrap(),
                family,
            );
            assert_eq!(first.assembly_digest(), fresh.assembly_digest());
            assert_eq!(first.battle_spec(), fresh.battle_spec());
            assert_eq!(
                first
                    .battle_spec()
                    .resources(TeamSide::Player)
                    .skill_points(),
                3
            );
            assert!(
                first
                    .battle_spec()
                    .resources(TeamSide::Enemy)
                    .keyed()
                    .is_empty()
            );
            let keyed = first.battle_spec().resources(TeamSide::Player).keyed();
            if party[0] == 1502 {
                let [resource] = keyed else {
                    panic!("one player meter")
                };
                assert_eq!(resource.id(), PUNCHLINE);
                assert_eq!(resource.stable_key(), Some("shared.punchline"));
                assert_eq!(
                    (resource.initial(), resource.maximum(), resource.wave()),
                    (0, 9999, TeamResourceWavePolicy::Persist)
                );
            } else {
                assert!(keyed.is_empty());
            }
            assert_eq!(create(&first).state_hash(), create(&fresh).state_hash());
        }
    }
}

#[test]
fn battle_team_resources_production_skill_exposes_pending_non_cost_delta_lowering() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = assembled(&fixture, family);
        let mut battle = create(&source);
        let mut replay = create(&source);
        assert_eq!(balance(&battle), 0);
        start(&mut battle);
        start(&mut replay);
        let mut found = false;
        for _ in 0..128 {
            let command = battle.decision().and_then(|decision| decision.legal_commands().iter().find(|command| {
                matches!(command, Command::UseAbility {actor, ability, ..} if actor.get() == 1
                    && source.combat_catalog().ability(*ability).and_then(|a| a.action()).is_some_and(|a| a.kind() == AbilityKind::Skill))
            }).cloned());
            if let Some(command) = command {
                let before = balance(&battle);
                let events = accept(&mut battle, command.clone());
                assert_eq!(events, accept(&mut replay, command));
                // Current generic data lowering omits non-cost keyed deltas.
                // This negative boundary grants no Skill execution credit.
                assert_eq!(balance(&battle), before);
                assert!(!events.iter().any(|e| matches!(e.kind(), BattleEventKind::Resource(ResourceEventData::TeamResource { resource, .. }) if *resource == PUNCHLINE)));
                assert_eq!(battle.state_hash(), replay.state_hash());
                assert_eq!(
                    battle.view().rng_draw_count(),
                    replay.view().rng_draw_count()
                );
                found = true;
                break;
            }
            let command = if let Some(decision) = battle.decision() {
                decision
                    .legal_commands()
                    .iter()
                    .find(|c| {
                        matches!(
                            c,
                            Command::UseAbility { .. }
                                | Command::CommitPreparedAction { .. }
                                | Command::CommitActionFrame { .. }
                        )
                    })
                    .unwrap()
                    .clone()
            } else {
                Command::Advance {
                    boundary: battle.view().action_boundary().unwrap().id(),
                }
            };
            assert_eq!(
                accept(&mut battle, command.clone()),
                accept(&mut replay, command)
            );
        }
        assert!(found, "production mapped Skill must be offered");
        assert_eq!(balance(&create(&source)), 0);
    }
}

const GAIN: u32 = 0x7dca_0001;
const SPEND: u32 = 0x7dca_0002;
const SET: u32 = 0x7dca_0003;
const KILL: u32 = 0x7dca_0004;
const IDLE: u32 = 0x7dca_0005;

// Controlled commands retain the exact production-assembled resource spec.
// They isolate lifecycle and overflow, not original character damage semantics.
fn probe(source: &DivergentUniverseAssembledBattle) -> Battle {
    let mut builder = CombatCatalogBuilder::from_catalog(source.combat_catalog(), [0xe5; 32]);
    let form = id(0x7dcb_0001);
    for raw in [GAIN, SPEND, SET, KILL, IDLE] {
        let program = id(raw + 0x10000);
        let selector = id(raw + 0x20000);
        builder.add_selector(SelectorDefinition::new(selector).with_unit_targets(
            UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
        ));
        builder.add_program(ProgramDefinition::new(
            program,
            vec![],
            vec![],
            vec![],
            vec![],
        ));
        let operations = match raw {
            GAIN => vec![HitOperationDefinition::ModifyTeamResource(
                TeamResourceChangeDefinition::new(PUNCHLINE, TeamResourceChange::Gain(10000)),
            )],
            SET => vec![HitOperationDefinition::ModifyTeamResource(
                TeamResourceChangeDefinition::new(PUNCHLINE, TeamResourceChange::Set(1234)),
            )],
            KILL => vec![HitOperationDefinition::Damage(
                OrdinaryDamageDefinition::new(
                    Scalar::checked_from_integer(100).unwrap(),
                    OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
                )
                .unwrap(),
            )],
            _ => vec![],
        };
        let mut resource = ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO);
        if raw == SPEND {
            resource = resource
                .with_team_resource_costs(vec![
                    TeamResourceCost::new("shared.punchline", 2).unwrap(),
                ])
                .unwrap();
        }
        let action = AbilityActionDefinition::new(
            AbilityKind::Basic,
            1,
            TargetInvalidationPolicy::KeepIfPresent,
            resource,
        )
        .unwrap()
        .with_hits(vec![ActionHitDefinition::new(operations)])
        .unwrap();
        builder.add_ability(
            AbilityDefinition::new(id(raw), program, selector, vec![]).with_action(action),
        );
    }
    let abilities = vec![id(GAIN), id(SPEND), id(SET), id(KILL), id(IDLE)];
    builder.add_unit(UnitDefinition::new(form, abilities.clone(), vec![]));
    let plain = |speed, hp, abilities| {
        ResolvedCombatantSpec::new(
            form,
            UnitLevel::new(80).unwrap(),
            Hp::new(hp).unwrap(),
            Speed::from_scaled(speed).unwrap(),
            ResolvedDefinitionBindings::new(abilities, vec![], vec![]).unwrap(),
            CombatantSpecDigest::new([0xe5; 32]).unwrap(),
        )
        .unwrap()
    };
    let mut participants = vec![ParticipantSpec::new(
        TeamSide::Player,
        FormationIndex::new(0).unwrap(),
        ParticipantSource::Player,
        plain(1_000_000_000, 10000, abilities),
    )];
    let mut waves = Vec::new();
    for wave in 1..=2 {
        let enemy = id(0x7dcc_0001 + u32::from(wave));
        builder.add_enemy(EnemyDefinition::new(enemy, form, vec![id(IDLE)]));
        participants.push(
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::EncounterEnemy(enemy),
                plain(1_000_000, 1, vec![id(IDLE)]),
            )
            .with_wave(wave)
            .unwrap(),
        );
        waves.push(vec![enemy]);
    }
    let encounter = id(0x7dcd_0001);
    builder.add_encounter(
        EncounterDefinition::new(encounter, vec![], vec![])
            .with_waves(waves)
            .unwrap(),
    );
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xe5; 32]).unwrap(),
        encounter,
        participants,
        source.battle_spec().resources(TeamSide::Player).clone(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(builder.build().unwrap(), spec, BattleSeed::new([0xe5; 32])).unwrap()
}
fn use_probe(battle: &mut Battle, raw: u32) -> Vec<BattleEvent> {
    for _ in 0..64 {
        if let Some(command) = battle.decision().and_then(|d| d.legal_commands().iter().find(|c| matches!(c, Command::UseAbility {actor, ability, ..} if actor.get() == 1 && ability.get() == raw)).cloned()) {
            return accept(battle, command);
        }
        // Do not select GAIN accidentally while waiting for a stable boundary.
        let command = if let Some(d) = battle.decision() {
            d.legal_commands()
                .iter()
                .find(|c| matches!(c, Command::UseAbility {ability, ..} if ability.get() == IDLE))
                .unwrap()
                .clone()
        } else {
            Command::Advance {
                boundary: battle.view().action_boundary().unwrap().id(),
            }
        };
        accept(battle, command);
    }
    panic!("probe ability not offered");
}

fn use_probe_pair(battle: &mut Battle, replay: &mut Battle, raw: u32) -> Vec<BattleEvent> {
    let events = use_probe(battle, raw);
    assert_eq!(events, use_probe(replay, raw));
    assert_eq!(battle.state_hash(), replay.state_hash());
    assert_eq!(
        battle.view().rng_draw_count(),
        replay.view().rng_draw_count()
    );
    events
}

#[test]
fn battle_team_resources_commands_clamp_overflow_spend_reject_and_persist_across_waves() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = assembled(&fixture, family);
        let mut battle = probe(&source);
        let mut replay = probe(&source);
        start(&mut battle);
        start(&mut replay);
        while battle.decision().is_none() {
            advance(&mut battle);
            advance(&mut replay);
        }
        let rejected = Command::UseAbility {
            decision: battle.decision().unwrap().id(),
            actor: UnitId::new(1).unwrap(),
            ability: AbilityId::new(SPEND).unwrap(),
            primary_target: Some(UnitId::new(2).unwrap()),
        };
        let before = (battle.state_hash(), battle.view().rng_draw_count());
        assert!(battle.apply(rejected).is_err());
        assert_eq!(
            (battle.state_hash(), battle.view().rng_draw_count()),
            before
        );
        let first = use_probe_pair(&mut battle, &mut replay, GAIN);
        assert_eq!(balance(&battle), 9999);
        assert!(first.iter().any(|e| matches!(e.kind(), BattleEventKind::Resource(ResourceEventData::TeamResource { resource, attempted: 10000, effective: 9999, overflow: 1, before: 0, after: 9999, .. }) if *resource == PUNCHLINE)));
        let capped = use_probe_pair(&mut battle, &mut replay, GAIN);
        assert!(capped.iter().any(|e| matches!(e.kind(), BattleEventKind::Resource(ResourceEventData::TeamResource { resource, attempted: 10000, effective: 0, overflow: 10000, before: 9999, after: 9999, .. }) if *resource == PUNCHLINE)));
        let spent = use_probe_pair(&mut battle, &mut replay, SPEND);
        assert_eq!(balance(&battle), 9997);
        assert!(spent.iter().any(|e| matches!(e.kind(), BattleEventKind::Resource(ResourceEventData::TeamResource { resource, attempted: 2, effective: 2, before: 9999, after: 9997, .. }) if *resource == PUNCHLINE)));
        use_probe_pair(&mut battle, &mut replay, SET);
        assert_eq!(balance(&battle), 1234);
        use_probe_pair(&mut battle, &mut replay, KILL);
        for _ in 0..64 {
            if battle.view().units_by_id().any(|u| {
                u.side() == TeamSide::Enemy
                    && u.entry_wave() == 2
                    && u.life() == LifeState::Alive
                    && u.presence() == PresenceState::Present
            }) {
                break;
            }
            advance(&mut battle);
            advance(&mut replay);
        }
        assert!(
            battle
                .view()
                .units_by_id()
                .any(|u| u.side() == TeamSide::Enemy
                    && u.entry_wave() == 2
                    && u.presence() == PresenceState::Present)
        );
        assert_eq!(balance(&battle), 1234);
        assert_eq!(battle.state_hash(), replay.state_hash());
        assert_eq!(balance(&probe(&source)), 0);
    }
}
