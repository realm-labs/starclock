use std::sync::Arc;

use serde_json::Value;
use starclock_agent_api::observation::{
    AgentBattlePhase, AgentTeamSide, MAX_EVENTS_PER_PAGE, ProjectionError, project_event_page,
    project_player_visible,
};
use starclock_combat::{
    Battle, BattleSeed, BattleSpec, BattleSpecDigest, CombatantSpecDigest, ConcedePolicy,
    FormationIndex, Hp, ParticipantSource, ParticipantSpec, ResolvedCombatantSpec,
    ResolvedDefinitionBindings, Speed, TeamResourceSpec, TeamSide, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionResourcePolicy, TargetInvalidationPolicy,
            TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
            SelectorDefinition, UnitDefinition,
        },
    },
};

fn definition<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).expect("test definition ID is non-zero")
}

fn catalog() -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new("agent-projection-fixture-v1", [0x41; 32]);
    for raw in 1..=2 {
        builder.add_selector(SelectorDefinition::new(definition(raw)).with_unit_targets(
            UnitTargetSelector::new(TargetRelation::SelfUnit, TargetPattern::Single).unwrap(),
        ));
        builder.add_program(ProgramDefinition::new(
            definition(raw),
            vec![],
            vec![definition(raw)],
            vec![],
            vec![],
        ));
        builder.add_ability(
            AbilityDefinition::new(definition(raw), definition(raw), definition(raw), vec![])
                .with_action(
                    AbilityActionDefinition::new(
                        AbilityKind::Basic,
                        1,
                        TargetInvalidationPolicy::CancelRemainingForTarget,
                        ActionResourcePolicy::new(
                            0,
                            0,
                            starclock_combat::Energy::ZERO,
                            starclock_combat::Energy::ZERO,
                        ),
                    )
                    .unwrap(),
                ),
        );
        builder.add_unit(UnitDefinition::new(
            definition(raw),
            vec![definition(raw)],
            vec![],
        ));
    }
    builder.add_enemy(EnemyDefinition::new(
        definition(1),
        definition(2),
        vec![definition(2)],
    ));
    builder.add_encounter(EncounterDefinition::new(
        definition(1),
        vec![definition(1)],
        vec![],
    ));
    builder.build().expect("projection catalog is valid")
}

fn combatant(form: u32, digest: u8) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        definition(form),
        UnitLevel::new(80).unwrap(),
        Hp::new(1_000).unwrap(),
        Speed::from_scaled(100_000_000).unwrap(),
        ResolvedDefinitionBindings::new(vec![definition(form)], vec![], vec![]).unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
}

fn fixture_battle() -> Battle {
    let spec = BattleSpec::new(
        "agent-projection-rules-v1",
        BattleSpecDigest::new([0x51; 32]).unwrap(),
        definition(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(4).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                combatant(2, 0x62),
            ),
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, 0x61),
            ),
        ],
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(catalog(), spec, BattleSeed::new([0x71; 32])).unwrap()
}

#[test]
fn player_projection_is_stable_canonical_and_exact() {
    let mut battle = fixture_battle();
    assert_eq!(
        project_player_visible(battle.view()),
        Err(ProjectionError::UnstableBoundary)
    );
    let start = battle
        .view()
        .decision()
        .unwrap()
        .legal_commands()
        .first()
        .unwrap()
        .clone();
    battle.apply(start).unwrap();

    let projected = project_player_visible(battle.view()).unwrap();
    assert_eq!(projected.phase, AgentBattlePhase::AwaitingCommand);
    assert_eq!(projected.committed_revision.as_str(), "1");
    assert_eq!(projected.wave.number.as_str(), "1");
    assert_eq!(projected.wave.total.as_str(), "1");
    assert_eq!(projected.teams[0].side, AgentTeamSide::Player);
    assert_eq!(projected.teams[1].side, AgentTeamSide::Enemy);
    assert_eq!(projected.units.len(), 2);
    assert_eq!(projected.units[0].unit_id.as_str(), "1");
    assert_eq!(projected.units[1].unit_id.as_str(), "2");
    assert_eq!(projected.timeline[0].actor_id.as_str(), "1");
    assert_eq!(projected.timeline[1].actor_id.as_str(), "2");
    assert_eq!(projected.units[0].current_hp.as_str(), "1000");
    assert_eq!(projected.units[0].current_energy_scaled.as_str(), "0");
}

#[test]
fn default_json_omits_private_combat_state_and_unpublished_intent() {
    let mut battle = fixture_battle();
    let start = battle.view().decision().unwrap().legal_commands()[0].clone();
    battle.apply(start).unwrap();
    let json = serde_json::to_string(&project_player_visible(battle.view()).unwrap()).unwrap();
    for forbidden in [
        "enemy_ai_state",
        "ai_graph",
        "ai_candidate",
        "automatic_ability",
        "rule_instances",
        "modifier_instances",
        "source_definition",
        "snapshot",
        "legal_commands",
        "public_intent",
    ] {
        assert!(!json.contains(forbidden), "leaked private key {forbidden}");
    }
    let value: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["units"][0]["unit_id"], "1");
    assert!(value["units"][0].get("public_intent").is_none());
}

#[test]
fn event_pages_are_payload_free_bounded_and_cursor_marked() {
    let mut battle = fixture_battle();
    let start = battle.view().decision().unwrap().legal_commands()[0].clone();
    let resolution = battle.apply(start).unwrap();
    let page = project_event_page(resolution.events()).unwrap();
    assert!(!page.truncated);
    assert_eq!(page.events.len(), 3);
    assert_eq!(page.events[0].event_id.as_str(), "1");
    assert_eq!(page.events[0].root_command_id.as_str(), "1");
    assert_eq!(page.next_cursor.as_str(), "event_3");

    let repeated = vec![resolution.events()[0].clone(); MAX_EVENTS_PER_PAGE + 1];
    let bounded = project_event_page(&repeated).unwrap();
    assert!(bounded.truncated);
    assert_eq!(bounded.events.len(), MAX_EVENTS_PER_PAGE);
    let value = serde_json::to_value(&bounded).unwrap();
    for event in value["events"].as_array().unwrap() {
        let keys = event
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>();
        assert_eq!(keys, ["event_id", "kind", "root_command_id", "summary"]);
    }
}
