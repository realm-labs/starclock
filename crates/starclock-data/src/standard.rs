//! Current production Standard scenario catalog and battle instantiation.

use crate::{
    CharacterDataDefinition,
    catalog::{CatalogManifest, CatalogSummary, load as catalog_load},
};
use std::{collections::BTreeSet, sync::Arc};

use crate::catalog::SimulationCatalog;
use starclock_combat::{
    AbilityId, AssemblyDigest, Battle, BattleSeed, BattleSpec, CombatantSpecDigest, ConcedePolicy,
    EncounterId, EnemyDefinitionId, Energy, FormationIndex, Hp, ParticipantSource, ParticipantSpec,
    ProgramId, Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings, Scalar, SelectorId, Speed,
    TeamResourceSpec, TeamSide, UnitDefinitionId, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            HitOperationDefinition, OrdinaryDamageDefinition, OrdinaryDamageMultipliers,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EnemyDefinition, ProgramDefinition, SelectorDefinition,
            UnitDefinition,
        },
    },
    rng::derive::StreamPath,
};
use starclock_mode_standard::StandardScenarioId;

const PRODUCTION_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const PLAYER_FORM: u32 = 20_001;
const PLAYER_ABILITY: u32 = 20_001;
const PLAYER_PROGRAM: u32 = 20_001;
const PLAYER_SELECTOR: u32 = 20_001;
const ENEMY_PROGRAM: u32 = 20_002;
const ENEMY_SELECTOR: u32 = 14_001;
pub const CONFIG_DIGEST: [u8; 32] = [
    0x91, 0x95, 0x0c, 0xc8, 0x7f, 0x1c, 0xa5, 0x92, 0x14, 0x16, 0x88, 0xc6, 0x66, 0xe9, 0x3d, 0x27,
    0x4e, 0x18, 0xe5, 0x45, 0x88, 0x9a, 0x9e, 0x71, 0xa3, 0xdf, 0xed, 0x8b, 0x15, 0x06, 0xd0, 0x3b,
];

pub const SCENARIOS: [(&str, u32, u32); 6] = [
    ("scenario.standard.basic-single-wave", 278, 89),
    ("scenario.standard.cocolia-phase-change", 279, 91),
    ("scenario.standard.elite-control-counter", 280, 90),
    ("scenario.standard.layered-toughness", 281, 93),
    ("scenario.standard.multi-wave-dot-revival", 282, 92),
    ("scenario.standard.target-invalidation-and-return", 283, 94),
];

pub struct StandardBattle {
    battle: Battle,
    encounter: EncounterId,
    assembly_digest: AssemblyDigest,
    master_seed: u64,
}

/// Immutable production data and combat catalogs shared by isolated sessions.
#[derive(Clone)]
pub struct StandardCatalog {
    data: Arc<SimulationCatalog>,
    combat: Arc<CombatCatalog>,
}

impl StandardCatalog {
    /// Loads and validates the embedded production bundle once.
    pub fn load() -> Result<Self, &'static str> {
        let data = catalog_load(PRODUCTION_BUNDLE)
            .map_err(|_| "production Standard catalog failed to load")?;
        let combat = combat_catalog(&data)?;
        Ok(Self { data, combat })
    }

    /// Returns generated-row-free compatibility metadata for bounded adapters.
    #[must_use]
    pub fn manifest(&self) -> &CatalogManifest {
        self.data.manifest()
    }

    /// Returns only aggregate counts from the validated production catalog.
    #[must_use]
    pub fn summary(&self) -> CatalogSummary {
        self.data.summary()
    }

    /// Looks up one generated-row-free character definition by exact form ID.
    #[must_use]
    pub fn character(&self, id: UnitDefinitionId) -> Option<&CharacterDataDefinition> {
        self.data.character(id)
    }

    /// Constructs one isolated battle from a frozen scenario key and seed policy.
    pub fn instantiate(
        &self,
        scenario_key: &str,
        seed_override: Option<u64>,
    ) -> Result<StandardBattle, &'static str> {
        let (_, scenario_id, encounter_id) = SCENARIOS
            .iter()
            .copied()
            .find(|(key, _, _)| *key == scenario_key)
            .ok_or("unknown frozen Standard scenario")?;
        let descriptor = self
            .data
            .standard_scenario(StandardScenarioId::new(scenario_id).expect("static scenario ID"))
            .ok_or("production Standard descriptor is missing")?;
        let spec = battle_spec(&self.data, encounter_id, scenario_id)?;
        let master_seed = seed_override.unwrap_or(descriptor.master_seed());
        let path = StreamPath::new(
            "standard-v1",
            u64::from(scenario_id),
            1,
            1,
            1,
            1,
            "standard-v1-battle",
        )
        .map_err(|_| "frozen Standard RNG path is invalid")?;
        let assembly_digest = spec.assembly_digest();
        let battle = Battle::create(
            Arc::clone(&self.combat),
            spec,
            BattleSeed::new(path.derive_seed(master_seed).bytes()),
        )
        .map_err(|_| "frozen Standard battle construction failed")?;
        Ok(StandardBattle {
            battle,
            encounter: EncounterId::new(encounter_id).expect("static encounter ID"),
            assembly_digest,
            master_seed,
        })
    }

    /// Looks up one immutable authored enemy graph retained by this catalog.
    #[must_use]
    pub fn ai_graph(
        &self,
        id: starclock_combat::AiGraphId,
    ) -> Option<&starclock_combat::catalog::encounter::AiGraphDefinition> {
        self.data.ai_graph(id)
    }
}

impl StandardBattle {
    pub fn battle_mut(&mut self) -> &mut Battle {
        &mut self.battle
    }

    pub fn into_battle(self) -> Battle {
        self.battle
    }

    pub const fn encounter(&self) -> EncounterId {
        self.encounter
    }

    pub const fn assembly_digest(&self) -> AssemblyDigest {
        self.assembly_digest
    }

    pub const fn master_seed(&self) -> u64 {
        self.master_seed
    }
}

pub fn instantiate(
    scenario_key: &str,
    seed_override: Option<u64>,
) -> Result<StandardBattle, &'static str> {
    StandardCatalog::load()?.instantiate(scenario_key, seed_override)
}

fn combat_catalog(data: &SimulationCatalog) -> Result<Arc<CombatCatalog>, &'static str> {
    let mut builder = CombatCatalogBuilder::new(CONFIG_DIGEST);
    let player_selector = SelectorId::new(PLAYER_SELECTOR).expect("static selector ID");
    let enemy_selector = SelectorId::new(ENEMY_SELECTOR).expect("static selector ID");
    builder.add_selector(
        SelectorDefinition::new(player_selector).with_unit_targets(
            UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single)
                .expect("opposing single selector"),
        ),
    );
    builder.add_selector(
        SelectorDefinition::new(enemy_selector).with_unit_targets(
            UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single)
                .expect("opposing single selector"),
        ),
    );
    builder.add_program(ProgramDefinition::new(
        ProgramId::new(PLAYER_PROGRAM).expect("static program ID"),
        vec![],
        vec![player_selector],
        vec![],
        vec![],
    ));
    builder.add_program(ProgramDefinition::new(
        ProgramId::new(ENEMY_PROGRAM).expect("static program ID"),
        vec![],
        vec![enemy_selector],
        vec![],
        vec![],
    ));
    let player_ability = AbilityId::new(PLAYER_ABILITY).expect("static ability ID");
    builder.add_ability(
        AbilityDefinition::new(
            player_ability,
            ProgramId::new(PLAYER_PROGRAM).expect("static program ID"),
            player_selector,
            vec![],
        )
        .with_action(action(1_000)),
    );
    builder.add_unit(UnitDefinition::new(
        UnitDefinitionId::new(PLAYER_FORM).expect("static form ID"),
        vec![player_ability],
        vec![],
    ));

    let mut abilities = BTreeSet::new();
    for raw in 95..=111 {
        let enemy = data
            .enemy(EnemyDefinitionId::new(raw).expect("frozen enemy ID"))
            .ok_or("frozen Standard enemy is missing")?;
        abilities.extend(enemy.abilities().iter().copied());
    }
    let fixture_graphs = (13_001..=13_017)
        .map(|raw| {
            data.ai_graph(starclock_combat::AiGraphId::new(raw).expect("frozen graph ID"))
                .ok_or("frozen Standard AI graph is missing")
                .cloned()
        })
        .collect::<Result<Vec<_>, _>>()?;
    for graph in &fixture_graphs {
        for state in graph.states() {
            abilities.insert(state.mandatory_fallback());
            abilities.extend(
                state
                    .candidates()
                    .iter()
                    .map(|candidate| candidate.ability()),
            );
        }
    }
    for ability in abilities {
        builder.add_ability(
            AbilityDefinition::new(
                ability,
                ProgramId::new(ENEMY_PROGRAM).expect("static program ID"),
                enemy_selector,
                vec![],
            )
            .with_action(action(1)),
        );
    }
    for graph in fixture_graphs {
        builder.add_ai_graph(graph);
    }
    for raw in 95..=111 {
        let enemy = data
            .enemy(EnemyDefinitionId::new(raw).expect("frozen enemy ID"))
            .ok_or("frozen Standard enemy is missing")?;
        let fixture_graph =
            starclock_combat::AiGraphId::new(13_001 + raw - 95).expect("frozen graph ID");
        let graph = data
            .ai_graph(fixture_graph)
            .ok_or("frozen Standard AI graph is missing")?;
        let mut fixture_abilities = enemy.abilities().to_vec();
        for state in graph.states() {
            fixture_abilities.push(state.mandatory_fallback());
            fixture_abilities.extend(
                state
                    .candidates()
                    .iter()
                    .map(|candidate| candidate.ability()),
            );
        }
        fixture_abilities.sort_unstable();
        fixture_abilities.dedup();
        builder.add_unit(UnitDefinition::new(
            UnitDefinitionId::new(raw).expect("frozen unit ID"),
            fixture_abilities.clone(),
            vec![],
        ));
        builder.add_enemy(
            EnemyDefinition::new(enemy.id(), enemy.unit(), fixture_abilities)
                .with_orchestration(
                    fixture_graph,
                    enemy
                        .phases()
                        .iter()
                        .map(|phase| {
                            starclock_combat::catalog::encounter::EnemyPhaseDefinition::new(
                                phase.id(),
                                phase.sequence(),
                                phase.entry_condition().clone(),
                                phase.exit_condition().clone(),
                                phase.replacement_priority(),
                                fixture_graph,
                                phase.targetable(),
                                phase.transition(),
                                None,
                                phase.carry(),
                            )
                        })
                        .collect(),
                )
                .expect("frozen fixture orchestration"),
        );
    }
    for raw in 89..=94 {
        builder.add_encounter(
            data.encounter(EncounterId::new(raw).expect("frozen encounter ID"))
                .ok_or("frozen Standard encounter is missing")?
                .clone(),
        );
    }
    builder
        .build()
        .map_err(|_| "frozen Standard combat catalog is invalid")
}

fn action(damage: i64) -> AbilityActionDefinition {
    let damage = OrdinaryDamageDefinition::new(
        Scalar::checked_from_integer(damage).expect("static damage is in range"),
        OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).expect("identity multipliers"),
    )
    .expect("positive static damage");
    AbilityActionDefinition::new(
        AbilityKind::Basic,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
    )
    .expect("one-hit action")
    .with_hits(vec![ActionHitDefinition::new(vec![
        HitOperationDefinition::Damage(damage),
    ])])
    .expect("one concrete hit")
}

fn battle_spec(
    data: &SimulationCatalog,
    encounter_raw: u32,
    scenario_raw: u32,
) -> Result<BattleSpec, &'static str> {
    let encounter_id = EncounterId::new(encounter_raw).expect("frozen encounter ID");
    let encounter = data
        .encounter(encounter_id)
        .ok_or("frozen Standard encounter is missing")?;
    let player = ResolvedCombatantSpec::new(
        UnitDefinitionId::new(PLAYER_FORM).expect("static player form"),
        UnitLevel::new(80).expect("static player level"),
        Hp::new(100_000).expect("static player HP"),
        Speed::from_scaled(200_000_000).expect("static player Speed"),
        ResolvedDefinitionBindings::new(
            vec![AbilityId::new(PLAYER_ABILITY).expect("static player ability")],
            vec![],
            vec![],
        )
        .expect("canonical player bindings"),
        CombatantSpecDigest::new([scenario_raw as u8; 32]).expect("nonzero player digest"),
    )
    .map_err(|_| "frozen Standard player golden is invalid")?;
    let mut participants = vec![ParticipantSpec::new(
        TeamSide::Player,
        FormationIndex::new(0).expect("static player formation"),
        ParticipantSource::Player,
        player,
    )];
    for (wave_index, wave) in encounter.waves().iter().enumerate() {
        for (slot_index, slot) in wave.slots().iter().enumerate() {
            let enemy = data
                .enemy(slot.enemy())
                .ok_or("encounter slot enemy is missing")?;
            let mut digest = [0_u8; 32];
            digest[..4].copy_from_slice(&slot.enemy().get().to_be_bytes());
            digest[4] = u8::try_from(wave_index + 1).expect("frozen wave count fits u8");
            digest[5] = u8::try_from(slot_index + 1).expect("frozen slot count fits u8");
            let combatant = ResolvedCombatantSpec::new(
                UnitDefinitionId::new(slot.enemy().get()).expect("frozen enemy unit"),
                UnitLevel::new(24).expect("frozen enemy level"),
                Hp::new(1).expect("positive golden enemy HP"),
                Speed::from_scaled(50_000_000).expect("static enemy Speed"),
                ResolvedDefinitionBindings::new(enemy.abilities().to_vec(), vec![], vec![])
                    .expect("canonical enemy bindings"),
                CombatantSpecDigest::new(digest).expect("nonzero enemy digest"),
            )
            .map_err(|_| "frozen Standard enemy golden is invalid")?;
            participants.push(
                ParticipantSpec::new(
                    TeamSide::Enemy,
                    slot.formation()
                        .ok_or("frozen Standard slot lacks a formation")?,
                    ParticipantSource::EncounterEnemy(slot.enemy()),
                    combatant,
                )
                .with_wave(u16::try_from(wave_index + 1).expect("frozen wave count fits u16"))
                .expect("enemy wave assignment"),
            );
        }
    }
    BattleSpec::new(
        AssemblyDigest::new([u8::try_from(scenario_raw - 277).expect("scenario ordinal"); 32])
            .expect("nonzero spec digest"),
        encounter_id,
        participants,
        TeamResourceSpec::new(3, 5).expect("standard skill points"),
        TeamResourceSpec::new(0, 0).expect("empty enemy resource"),
        ConcedePolicy::Allowed,
    )
    .map_err(|_| "frozen Standard battle spec is invalid")
}

#[cfg(test)]
mod tests {
    use starclock_combat::{BattlePhase, Command, DecisionKind};

    use super::*;

    #[test]
    fn cloned_factory_shares_immutable_catalogs_only() {
        let first = StandardCatalog::load().unwrap();
        let second = first.clone();
        assert!(Arc::ptr_eq(&first.data, &second.data));
        assert!(Arc::ptr_eq(&first.combat, &second.combat));
    }

    #[test]
    fn every_scenario_reaches_its_expected_seeded_terminal_state() {
        const EXPECTED: [(&str, usize, &str); 6] = [
            (
                SCENARIOS[0].0,
                161,
                "ba4a03c81869a030cc313fd95ae1e2431baa988a1a961006a84638fc162bd9a3",
            ),
            (
                SCENARIOS[1].0,
                32,
                "1d58df2ea29bc8c14a96c08d2d7d6954dca792d8e5763ebf4cbbfec3eec3e058",
            ),
            (
                SCENARIOS[2].0,
                113,
                "3d5b4dc4384223c5ba42a7a54ae3d1dd9ca31680f9d42fe9ecc80e96c520b45e",
            ),
            (
                SCENARIOS[3].0,
                48,
                "372ae30284fe7ce4c6d9d4753bb493ce518704c84dff8e905cbc41bc40bb7280",
            ),
            (
                SCENARIOS[4].0,
                331,
                "99010b69effab03b5b9337efa23d944c495e8d4bf7b7d4a88a6e01909a490a22",
            ),
            (
                SCENARIOS[5].0,
                441,
                "80309d65d116e5ca41291c4af08bf8fa2e6fb8593526b65c36f918822a1b851f",
            ),
        ];
        for (scenario, expected_events, expected_hash) in EXPECTED {
            let mut instantiated =
                instantiate(scenario, None).expect("current scenario instantiates");
            let battle = instantiated.battle_mut();
            let mut events = 0;
            let mut commands = 0;
            while !battle.view().phase().is_terminal() {
                assert!(commands < 512, "current scenario exceeded command budget");
                let command = if battle.view().phase() == BattlePhase::ReadyToAdvance {
                    battle
                        .advance_command()
                        .expect("ready battle has an action boundary")
                } else {
                    let decision = battle.decision().expect("nonterminal decision");
                    match decision.kind() {
                        DecisionKind::BattleStart => decision.legal_commands().first(),
                        DecisionKind::NormalAction => decision
                            .legal_commands()
                            .iter()
                            .find(|command| matches!(command, Command::UseAbility { .. })),
                        DecisionKind::PreparedAction => {
                            decision.legal_commands().iter().find(|command| {
                                matches!(command, Command::CommitPreparedAction { .. })
                            })
                        }
                        DecisionKind::ActionFrame => decision
                            .legal_commands()
                            .iter()
                            .find(|command| matches!(command, Command::CommitActionFrame { .. })),
                        DecisionKind::BattleChoice => None,
                    }
                    .cloned()
                    .expect("golden decision has a supported command")
                };
                let resolution = battle.apply(command).expect("offered command applies");
                events += resolution.events().len();
                commands += 1;
            }
            assert_eq!(battle.view().phase(), BattlePhase::Won);
            assert_eq!(events, expected_events, "event-count drift for {scenario}");
            assert_eq!(
                hex(battle.state_hash().bytes()),
                expected_hash,
                "state-hash drift for {scenario}"
            );
        }
    }

    fn hex(bytes: [u8; 32]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}
