//! Frozen production Standard-v1 scenario catalog and battle instantiation.

use std::{collections::BTreeSet, sync::Arc};

use crate::catalog::SimulationCatalog;
use starclock_combat::{
    AbilityId, Battle, BattleSeed, BattleSpec, BattleSpecDigest, CombatantSpecDigest,
    ConcedePolicy, EncounterId, EnemyDefinitionId, Energy, FormationIndex, Hp, ParticipantSource,
    ParticipantSpec, ProgramId, Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings, Scalar,
    SelectorId, Speed, TeamResourceSpec, TeamSide, UnitDefinitionId, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            HitOperationDefinition, OrdinaryDamageDefinition, OrdinaryDamageMultipliers,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{AbilityDefinition, ProgramDefinition, SelectorDefinition, UnitDefinition},
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
    0xf5, 0xcb, 0x9e, 0xba, 0x2e, 0x5c, 0x52, 0x29, 0xbb, 0xf4, 0x71, 0xdf, 0xa6, 0x99, 0x1b, 0x99,
    0x79, 0x32, 0xd8, 0x2a, 0xdb, 0x3d, 0xe3, 0x9a, 0xfd, 0xa8, 0x2e, 0xa2, 0x6d, 0x0c, 0xb1, 0xc8,
];
pub const CATALOG_REVISION: &str = "core-combat-v1-standard-v1";
pub const RULES_REVISION: &str = "core-combat-rules-v1";

pub const SCENARIOS: [(&str, u32, u32); 6] = [
    ("scenario.standard-v1.basic-single-wave", 278, 89),
    ("scenario.standard-v1.cocolia-phase-change", 279, 91),
    ("scenario.standard-v1.elite-control-counter", 280, 90),
    ("scenario.standard-v1.layered-toughness", 281, 93),
    ("scenario.standard-v1.multi-wave-dot-revival", 282, 92),
    (
        "scenario.standard-v1.target-invalidation-and-return",
        283,
        94,
    ),
];

pub struct StandardV1Battle {
    battle: Battle,
    encounter: EncounterId,
    spec_digest: BattleSpecDigest,
    master_seed: u64,
}

/// Immutable production data and combat catalogs shared by isolated sessions.
#[derive(Clone)]
pub struct StandardV1Catalog {
    data: Arc<SimulationCatalog>,
    combat: Arc<CombatCatalog>,
}

impl StandardV1Catalog {
    /// Loads and validates the embedded production bundle once.
    pub fn load() -> Result<Self, &'static str> {
        let data = crate::catalog::load(PRODUCTION_BUNDLE)
            .map_err(|_| "production Standard-v1 catalog failed to load")?;
        let combat = combat_catalog(&data)?;
        Ok(Self { data, combat })
    }

    /// Constructs one isolated battle from a frozen scenario key and seed policy.
    pub fn instantiate(
        &self,
        scenario_key: &str,
        seed_override: Option<u64>,
    ) -> Result<StandardV1Battle, &'static str> {
        let (_, scenario_id, encounter_id) = SCENARIOS
            .iter()
            .copied()
            .find(|(key, _, _)| *key == scenario_key)
            .ok_or("unknown frozen Standard-v1 scenario")?;
        let descriptor = self
            .data
            .standard_scenario(StandardScenarioId::new(scenario_id).expect("static scenario ID"))
            .ok_or("production Standard-v1 descriptor is missing")?;
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
        .map_err(|_| "frozen Standard-v1 RNG path is invalid")?;
        let spec_digest = spec.digest();
        let battle = Battle::create(
            Arc::clone(&self.combat),
            spec,
            BattleSeed::new(path.derive_seed(master_seed).bytes()),
        )
        .map_err(|_| "frozen Standard-v1 battle construction failed")?;
        Ok(StandardV1Battle {
            battle,
            encounter: EncounterId::new(encounter_id).expect("static encounter ID"),
            spec_digest,
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

impl StandardV1Battle {
    pub fn battle_mut(&mut self) -> &mut Battle {
        &mut self.battle
    }

    pub fn into_battle(self) -> Battle {
        self.battle
    }

    pub const fn encounter(&self) -> EncounterId {
        self.encounter
    }

    pub const fn spec_digest(&self) -> BattleSpecDigest {
        self.spec_digest
    }

    pub const fn master_seed(&self) -> u64 {
        self.master_seed
    }
}

pub fn instantiate(
    scenario_key: &str,
    seed_override: Option<u64>,
) -> Result<StandardV1Battle, &'static str> {
    StandardV1Catalog::load()?.instantiate(scenario_key, seed_override)
}

fn combat_catalog(data: &SimulationCatalog) -> Result<Arc<CombatCatalog>, &'static str> {
    let mut builder = CombatCatalogBuilder::new(CATALOG_REVISION, CONFIG_DIGEST);
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
            .ok_or("frozen Standard-v1 enemy is missing")?;
        abilities.extend(enemy.abilities().iter().copied());
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
    for raw in 13_001..=13_017 {
        builder.add_ai_graph(
            data.ai_graph(starclock_combat::AiGraphId::new(raw).expect("frozen graph ID"))
                .ok_or("frozen Standard-v1 AI graph is missing")?
                .clone(),
        );
    }
    for raw in 95..=111 {
        let enemy = data
            .enemy(EnemyDefinitionId::new(raw).expect("frozen enemy ID"))
            .ok_or("frozen Standard-v1 enemy is missing")?;
        builder.add_unit(UnitDefinition::new(
            UnitDefinitionId::new(raw).expect("frozen unit ID"),
            enemy.abilities().to_vec(),
            vec![],
        ));
        builder.add_enemy(enemy.clone());
    }
    for raw in 89..=94 {
        builder.add_encounter(
            data.encounter(EncounterId::new(raw).expect("frozen encounter ID"))
                .ok_or("frozen Standard-v1 encounter is missing")?
                .clone(),
        );
    }
    builder
        .build()
        .map_err(|_| "frozen Standard-v1 combat catalog is invalid")
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
        .ok_or("frozen Standard-v1 encounter is missing")?;
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
    .map_err(|_| "frozen Standard-v1 player golden is invalid")?;
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
            .map_err(|_| "frozen Standard-v1 enemy golden is invalid")?;
            participants.push(
                ParticipantSpec::new(
                    TeamSide::Enemy,
                    slot.formation()
                        .ok_or("frozen Standard-v1 slot lacks a formation")?,
                    ParticipantSource::EncounterEnemy(slot.enemy()),
                    combatant,
                )
                .with_wave(u16::try_from(wave_index + 1).expect("frozen wave count fits u16"))
                .expect("enemy wave assignment"),
            );
        }
    }
    BattleSpec::new(
        RULES_REVISION,
        BattleSpecDigest::new([u8::try_from(scenario_raw - 277).expect("scenario ordinal"); 32])
            .expect("nonzero spec digest"),
        encounter_id,
        participants,
        TeamResourceSpec::new(3, 5).expect("standard skill points"),
        TeamResourceSpec::new(0, 0).expect("empty enemy resource"),
        ConcedePolicy::Allowed,
    )
    .map_err(|_| "frozen Standard-v1 battle spec is invalid")
}

#[cfg(test)]
mod tests {
    use starclock_combat::{BattlePhase, Command, DecisionKind};

    use super::*;

    #[test]
    fn cloned_factory_shares_immutable_catalogs_only() {
        let first = StandardV1Catalog::load().unwrap();
        let second = first.clone();
        assert!(Arc::ptr_eq(&first.data, &second.data));
        assert!(Arc::ptr_eq(&first.combat, &second.combat));
    }

    #[test]
    fn every_frozen_scenario_reaches_its_seeded_terminal_golden() {
        const EXPECTED: [(&str, usize, &str); 6] = [
            (
                SCENARIOS[0].0,
                66,
                "5021cdd6019e0a100ad35e36ffb69fdb4860600db472c77fb8b33a9571b507ec",
            ),
            (
                SCENARIOS[1].0,
                18,
                "87d2523332871b19cf4773373d031c6473bac29a48d17e796e0584cda296b344",
            ),
            (
                SCENARIOS[2].0,
                50,
                "c6c1a62d408e6c31f45624440802e64d79cbc359faf9ffb58b62b25be3879603",
            ),
            (
                SCENARIOS[3].0,
                18,
                "d3459759678910e92a719341a837a2ceca24a05bc1f5abbfa2190556e21e9c06",
            ),
            (
                SCENARIOS[4].0,
                182,
                "c89ee783c91ce046d6b3b07ee0b29376417dc34ccc2f6935510bab180254a588",
            ),
            (
                SCENARIOS[5].0,
                182,
                "413356b9d452876c51b269e62703072eef916ef866fd1420cfd7164e7383356b",
            ),
        ];
        for (scenario, expected_events, expected_hash) in EXPECTED {
            let mut instantiated =
                instantiate(scenario, None).expect("frozen scenario instantiates");
            let battle = instantiated.battle_mut();
            let mut events = 0;
            let mut commands = 0;
            while !battle.view().phase().is_terminal() {
                assert!(commands < 512, "frozen scenario exceeded command budget");
                let decision = battle.decision().expect("nonterminal decision");
                let command = match decision.kind() {
                    DecisionKind::BattleStart => decision.legal_commands().first(),
                    DecisionKind::InterruptWindow => decision
                        .legal_commands()
                        .iter()
                        .find(|command| matches!(command, Command::PassInterruptWindow { .. })),
                    DecisionKind::NormalAction => decision
                        .legal_commands()
                        .iter()
                        .find(|command| matches!(command, Command::UseAbility { .. })),
                    DecisionKind::BattleChoice => None,
                }
                .cloned()
                .expect("golden decision has a supported command");
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
