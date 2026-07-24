//! Standard Universe encounter rows lowered into validated combat requests.

use std::{collections::BTreeMap, sync::Arc};

use starclock_activity::{
    ActivityBattleResultContract, ActivityOptionId, ActivityParticipantCarryDefinition,
    BattleBinding, BattleResultProjection, EncounterInitiativePolicy, EnergyCarryPolicy,
    HpCarryPolicy, LifeCarryPolicy, ParticipantId, ParticipantLock, PreparedBattleVariant,
    PresenceCarryPolicy, ProjectionField, ProjectionId, TechniqueContributionDigest,
};
use starclock_combat::{
    Battle, BattleSeed, BattleSpec, BattleSpecDigest, CombatantSpecDigest, ConcedePolicy,
    EncounterId, EncounterWaveId, EnemyDefinitionId, Energy, FormationIndex, Hp, ParticipantSource,
    ParticipantSpec, ResolvedCombatantSpec, ResolvedDefinitionBindings, ResolvedModifierBinding,
    Speed, TeamResourceSpec, TeamSide, UnitLevel,
    catalog::{
        CombatCatalog,
        builder::CombatCatalogBuilder,
        definition::EncounterDefinition,
        encounter::{
            EncounterWaveDefinition as CombatEncounterWave, WaveCarry, WaveSlotDefinition,
            WaveTransitionPolicy,
        },
    },
};

use crate::{
    battle_contribution::UniverseBattleContributionSet,
    battle_overlay::{UniverseEncounterBattleBinding, UniverseEncounterOverlay},
    catalog::UniverseCatalog,
    digest::Encoder,
    encounter::{DifficultyEnemyBinding, EncounterMemberDefinition, EnemyRole},
    encounter_content_runtime::EncounterContentRuntimeCatalog,
    id::{DifficultyId, EncounterMemberId},
};

pub const UNIVERSE_BATTLE_MATERIALIZATION_REVISION: &str =
    "standard-universe-battle-materialization-v1";
pub const UNIVERSE_ENEMY_RUNTIME_STAT_POLICY: &str = "goal01-executable-enemy-proxy-stats-v1";

const MEMBER_ENCOUNTER_ID_BASE: u32 = 0x7500_0000;
const DIFFICULTY_ENCOUNTER_ID_BASE: u32 = 0x7510_0000;
const MEMBER_WAVE_ID_BASE: u32 = 0x7520_0000;
const DIFFICULTY_WAVE_ID_BASE: u32 = 0x7530_0000;
const PROJECTION_ID: u32 = 0x7540_0001;
const NORMAL_ENGAGEMENT_OPTION: u32 = 0x7540_0002;
const MEMBER_COUNT: usize = 173;
const MEMBER_ENEMY_SLOT_COUNT: usize = 538;
const DIFFICULTY_BINDING_COUNT: usize = 182;
const ENEMY_VARIANT_COUNT: usize = 86;
const EXACT_ENEMY_VARIANT_COUNT: usize = 13;

const MINION_PROXY: &str = "enemy.flamespawn.minion.variant.01";
const MINION_LV2_PROXY: &str = "enemy.voidranger-reaver.minionlv2.variant.01";
const ELITE_PROXY: &str = "enemy.voidranger-trampler.elite.variant.01";
const BOSS_PROXY: &str = "enemy.cocolia-mother-of-deception.bigboss.variant.01";

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EnemyDefinitionMatch {
    Exact,
    ApproximateProxy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UniverseEnemyMaterialization {
    stable_key: Box<str>,
    source_enemy: Option<EnemyDefinitionId>,
    combat_enemy: EnemyDefinitionId,
    proxy_stable_key: Option<Box<str>>,
    definition_match: EnemyDefinitionMatch,
}

impl UniverseEnemyMaterialization {
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn source_enemy(&self) -> Option<EnemyDefinitionId> {
        self.source_enemy
    }
    #[must_use]
    pub const fn combat_enemy(&self) -> EnemyDefinitionId {
        self.combat_enemy
    }
    #[must_use]
    pub fn proxy_stable_key(&self) -> Option<&str> {
        self.proxy_stable_key.as_deref()
    }
    #[must_use]
    pub const fn definition_match(&self) -> EnemyDefinitionMatch {
        self.definition_match
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UniverseBattleRosterEntry {
    participant: ParticipantId,
    formation: FormationIndex,
    combatant: ResolvedCombatantSpec,
}

impl UniverseBattleRosterEntry {
    #[must_use]
    pub const fn participant(&self) -> ParticipantId {
        self.participant
    }
    #[must_use]
    pub const fn formation(&self) -> FormationIndex {
        self.formation
    }
    #[must_use]
    pub const fn combatant(&self) -> &ResolvedCombatantSpec {
        &self.combatant
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UniverseBattleRoster {
    participant_lock: starclock_activity::ParticipantLockDigest,
    entries: Box<[UniverseBattleRosterEntry]>,
}

impl UniverseBattleRoster {
    pub fn new(
        lock: &ParticipantLock,
        combatants: Vec<(ParticipantId, ResolvedCombatantSpec)>,
    ) -> Result<Self, UniverseBattleMaterializationError> {
        if combatants.len() != lock.entries().len() {
            return Err(UniverseBattleMaterializationError::RosterMismatch);
        }
        let mut entries = Vec::with_capacity(combatants.len());
        for locked in lock.entries() {
            let (_, combatant) = combatants
                .iter()
                .find(|(participant, _)| *participant == locked.participant())
                .ok_or(UniverseBattleMaterializationError::RosterMismatch)?;
            if locked.team_index() != 0
                || locked.character() != combatant.form()
                || locked.build().resolved_spec_digest() != combatant.digest()
            {
                return Err(UniverseBattleMaterializationError::RosterMismatch);
            }
            entries.push(UniverseBattleRosterEntry {
                participant: locked.participant(),
                formation: FormationIndex::new(locked.formation_index())
                    .ok_or(UniverseBattleMaterializationError::RosterMismatch)?,
                combatant: combatant.clone(),
            });
        }
        entries.sort_by_key(|entry| entry.formation);
        Ok(Self {
            participant_lock: lock.digest(),
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn participant_lock(&self) -> starclock_activity::ParticipantLockDigest {
        self.participant_lock
    }
    #[must_use]
    pub fn entries(&self) -> &[UniverseBattleRosterEntry] {
        &self.entries
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UniverseDifficultyBattleSpec {
    ordinal: u16,
    difficulty: DifficultyId,
    role: EnemyRole,
    source_monster_id: Box<str>,
    enemy_variant_key: Box<str>,
    level: UnitLevel,
    battle_spec: BattleSpec,
}

impl UniverseDifficultyBattleSpec {
    #[must_use]
    pub const fn ordinal(&self) -> u16 {
        self.ordinal
    }
    #[must_use]
    pub const fn difficulty(&self) -> DifficultyId {
        self.difficulty
    }
    #[must_use]
    pub const fn role(&self) -> EnemyRole {
        self.role
    }
    #[must_use]
    pub fn source_monster_id(&self) -> &str {
        &self.source_monster_id
    }
    #[must_use]
    pub fn enemy_variant_key(&self) -> &str {
        &self.enemy_variant_key
    }
    #[must_use]
    pub const fn level(&self) -> UnitLevel {
        self.level
    }
    #[must_use]
    pub const fn battle_spec(&self) -> &BattleSpec {
        &self.battle_spec
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UniverseBattleMaterializationCoverage {
    member_count: u16,
    member_wave_count: u16,
    member_enemy_slot_count: u16,
    difficulty_binding_count: u16,
    enemy_variant_count: u16,
    exact_enemy_variant_count: u16,
    approximate_enemy_variant_count: u16,
    declared_rule_binding_count: u16,
    materialized_rule_binding_count: u16,
    runtime_stat_policy: Box<str>,
    digest: [u8; 32],
}

impl UniverseBattleMaterializationCoverage {
    #[must_use]
    pub const fn member_count(&self) -> u16 {
        self.member_count
    }
    #[must_use]
    pub const fn member_wave_count(&self) -> u16 {
        self.member_wave_count
    }
    #[must_use]
    pub const fn member_enemy_slot_count(&self) -> u16 {
        self.member_enemy_slot_count
    }
    #[must_use]
    pub const fn difficulty_binding_count(&self) -> u16 {
        self.difficulty_binding_count
    }
    #[must_use]
    pub const fn enemy_variant_count(&self) -> u16 {
        self.enemy_variant_count
    }
    #[must_use]
    pub const fn exact_enemy_variant_count(&self) -> u16 {
        self.exact_enemy_variant_count
    }
    #[must_use]
    pub const fn approximate_enemy_variant_count(&self) -> u16 {
        self.approximate_enemy_variant_count
    }
    #[must_use]
    pub const fn declared_rule_binding_count(&self) -> u16 {
        self.declared_rule_binding_count
    }
    #[must_use]
    pub const fn materialized_rule_binding_count(&self) -> u16 {
        self.materialized_rule_binding_count
    }
    #[must_use]
    pub fn runtime_stat_policy(&self) -> &str {
        &self.runtime_stat_policy
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

#[derive(Clone, Debug)]
pub struct UniverseBattleMaterialization {
    combat_catalog: Arc<CombatCatalog>,
    overlay: UniverseEncounterOverlay,
    difficulty_specs: Box<[UniverseDifficultyBattleSpec]>,
    enemies: Box<[UniverseEnemyMaterialization]>,
    coverage: UniverseBattleMaterializationCoverage,
    digest: [u8; 32],
}

impl UniverseBattleMaterialization {
    #[must_use]
    pub const fn combat_catalog(&self) -> &Arc<CombatCatalog> {
        &self.combat_catalog
    }
    #[must_use]
    pub const fn overlay(&self) -> &UniverseEncounterOverlay {
        &self.overlay
    }
    #[must_use]
    pub fn difficulty_specs(&self) -> &[UniverseDifficultyBattleSpec] {
        &self.difficulty_specs
    }
    #[must_use]
    pub fn enemies(&self) -> &[UniverseEnemyMaterialization] {
        &self.enemies
    }
    #[must_use]
    pub const fn coverage(&self) -> &UniverseBattleMaterializationCoverage {
        &self.coverage
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UniverseBattleMaterializer;

impl UniverseBattleMaterializer {
    pub fn compile(
        self,
        universe: &UniverseCatalog,
        roster: &UniverseBattleRoster,
        contributions: &UniverseBattleContributionSet,
    ) -> Result<UniverseBattleMaterialization, UniverseBattleMaterializationError> {
        let content = EncounterContentRuntimeCatalog::compile(universe)
            .map_err(|_| UniverseBattleMaterializationError::InvalidEncounterContent)?;
        let enemies = materialize_enemies(universe, &content)?;
        let enemy_map = enemies
            .iter()
            .map(|enemy| (enemy.stable_key(), enemy.combat_enemy()))
            .collect::<BTreeMap<_, _>>();
        let digest = root_digest(universe, roster, contributions, &enemies);
        let revision = format!(
            "{}+{}",
            universe
                .simulation_catalog()
                .combat_catalog()
                .revision()
                .as_str(),
            UNIVERSE_BATTLE_MATERIALIZATION_REVISION
        );
        let mut builder = CombatCatalogBuilder::from_catalog(
            universe.simulation_catalog().combat_catalog(),
            revision.clone(),
            digest,
        );
        for modifier in contributions.modifiers() {
            builder.add_modifier_group(modifier.group().clone());
            builder.add_modifier(modifier.definition().clone());
        }
        for member in members(universe) {
            builder.add_encounter(member_encounter(member, &enemy_map)?);
        }
        for (index, binding) in universe.difficulty_enemy_bindings().iter().enumerate() {
            builder.add_encounter(difficulty_encounter(index, binding, &enemy_map)?);
        }
        let combat_catalog = builder
            .build()
            .map_err(|_| UniverseBattleMaterializationError::InvalidCompositeCatalog)?;
        let players = player_participants(roster, contributions)?;
        let contract = settlement_contract(roster)?;
        let mut overlay_bindings = Vec::with_capacity(MEMBER_COUNT);
        let mut member_wave_count = 0_usize;
        let mut member_enemy_slot_count = 0_usize;
        for member in members(universe) {
            member_wave_count += member.waves().len();
            member_enemy_slot_count += member
                .waves()
                .iter()
                .map(|wave| wave.enemies().len())
                .sum::<usize>();
            let spec = member_spec(
                member,
                &players,
                &enemy_map,
                &combat_catalog,
                &revision,
                digest,
            )?;
            validate_executable(&combat_catalog, &spec)?;
            let preparation = starclock_activity::EncounterPreparationDefinition::new(
                ActivityOptionId::new(u64::from(NORMAL_ENGAGEMENT_OPTION))
                    .expect("reserved engagement option is non-zero"),
                EncounterInitiativePolicy::PlayerControlled,
                roster.participant_lock(),
                0,
                Vec::new(),
                vec![PreparedBattleVariant::new(
                    Vec::new(),
                    TechniqueContributionDigest::new(contributions.digest())
                        .expect("contribution digest is non-zero"),
                    BattleBinding::new(
                        spec,
                        "standard-universe-battle",
                        UNIVERSE_BATTLE_MATERIALIZATION_REVISION,
                        roster.participant_lock(),
                    )
                    .map_err(|_| UniverseBattleMaterializationError::InvalidBattleBinding)?,
                )],
            )
            .map_err(|_| UniverseBattleMaterializationError::InvalidBattleBinding)?;
            overlay_bindings.push(UniverseEncounterBattleBinding::new(
                member.id(),
                Arc::new(preparation),
                Arc::clone(&contract),
            ));
        }
        let overlay = UniverseEncounterOverlay::new(overlay_bindings)
            .map_err(|_| UniverseBattleMaterializationError::InvalidBattleOverlay)?;
        content
            .validate_overlay(&overlay)
            .map_err(|_| UniverseBattleMaterializationError::InvalidBattleOverlay)?;

        let mut difficulty_specs = Vec::with_capacity(DIFFICULTY_BINDING_COUNT);
        for (index, binding) in universe.difficulty_enemy_bindings().iter().enumerate() {
            let spec = difficulty_spec(
                index,
                binding,
                &players,
                &enemy_map,
                &combat_catalog,
                &revision,
                digest,
            )?;
            validate_executable(&combat_catalog, &spec)?;
            difficulty_specs.push(UniverseDifficultyBattleSpec {
                ordinal: u16::try_from(index + 1)
                    .map_err(|_| UniverseBattleMaterializationError::IdentityOverflow)?,
                difficulty: binding.difficulty(),
                role: binding.role(),
                source_monster_id: binding.source_monster_id().into(),
                enemy_variant_key: binding.enemy_variant_key().into(),
                level: checked_level(binding.level())?,
                battle_spec: spec,
            });
        }
        if overlay.bindings().len() != MEMBER_COUNT
            || member_wave_count != MEMBER_COUNT
            || member_enemy_slot_count != MEMBER_ENEMY_SLOT_COUNT
            || difficulty_specs.len() != DIFFICULTY_BINDING_COUNT
        {
            return Err(UniverseBattleMaterializationError::InvalidDenominator);
        }
        let exact = enemies
            .iter()
            .filter(|enemy| enemy.definition_match == EnemyDefinitionMatch::Exact)
            .count();
        if enemies.len() != ENEMY_VARIANT_COUNT || exact != EXACT_ENEMY_VARIANT_COUNT {
            return Err(UniverseBattleMaterializationError::InvalidDenominator);
        }
        let coverage_digest = coverage_digest(
            member_wave_count,
            member_enemy_slot_count,
            exact,
            contributions.rules().len(),
            &enemies,
        );
        let coverage = UniverseBattleMaterializationCoverage {
            member_count: MEMBER_COUNT as u16,
            member_wave_count: member_wave_count as u16,
            member_enemy_slot_count: member_enemy_slot_count as u16,
            difficulty_binding_count: DIFFICULTY_BINDING_COUNT as u16,
            enemy_variant_count: ENEMY_VARIANT_COUNT as u16,
            exact_enemy_variant_count: exact as u16,
            approximate_enemy_variant_count: (enemies.len() - exact) as u16,
            declared_rule_binding_count: u16::try_from(contributions.rules().len())
                .map_err(|_| UniverseBattleMaterializationError::InvalidDenominator)?,
            materialized_rule_binding_count: 0,
            runtime_stat_policy: UNIVERSE_ENEMY_RUNTIME_STAT_POLICY.into(),
            digest: coverage_digest,
        };
        Ok(UniverseBattleMaterialization {
            combat_catalog,
            overlay,
            difficulty_specs: difficulty_specs.into_boxed_slice(),
            enemies: enemies.into_boxed_slice(),
            coverage,
            digest,
        })
    }
}

fn members(catalog: &UniverseCatalog) -> impl Iterator<Item = &EncounterMemberDefinition> {
    catalog
        .encounter_groups()
        .iter()
        .flat_map(|group| group.members())
}

fn materialize_enemies(
    universe: &UniverseCatalog,
    content: &EncounterContentRuntimeCatalog,
) -> Result<Vec<UniverseEnemyMaterialization>, UniverseBattleMaterializationError> {
    let data = universe.simulation_catalog();
    content
        .enemy_variant_keys()
        .iter()
        .map(|stable_key| {
            if let Some(enemy) = data.enemy_by_stable_key(stable_key) {
                return Ok(UniverseEnemyMaterialization {
                    stable_key: stable_key.clone(),
                    source_enemy: Some(enemy.id()),
                    combat_enemy: enemy.id(),
                    proxy_stable_key: None,
                    definition_match: EnemyDefinitionMatch::Exact,
                });
            }
            let proxy_key = proxy_key(stable_key);
            let proxy = data
                .enemy_by_stable_key(proxy_key)
                .ok_or(UniverseBattleMaterializationError::MissingProxyEnemy)?;
            Ok(UniverseEnemyMaterialization {
                stable_key: stable_key.clone(),
                source_enemy: None,
                combat_enemy: proxy.id(),
                proxy_stable_key: Some(proxy_key.into()),
                definition_match: EnemyDefinitionMatch::ApproximateProxy,
            })
        })
        .collect()
}

fn proxy_key(stable_key: &str) -> &'static str {
    if stable_key.contains(".bigboss.") {
        BOSS_PROXY
    } else if stable_key.contains(".elite.") {
        ELITE_PROXY
    } else if stable_key.contains(".minionlv2.") {
        MINION_LV2_PROXY
    } else {
        MINION_PROXY
    }
}

fn member_encounter(
    member: &EncounterMemberDefinition,
    enemies: &BTreeMap<&str, EnemyDefinitionId>,
) -> Result<EncounterDefinition, UniverseBattleMaterializationError> {
    let encounter = member_encounter_id(member.id())?;
    let waves = member
        .waves()
        .iter()
        .enumerate()
        .map(|(wave_index, wave)| {
            let slots = wave
                .enemies()
                .iter()
                .enumerate()
                .map(|(slot_index, slot)| {
                    let enemy = *enemies
                        .get(slot.enemy_variant_key())
                        .ok_or(UniverseBattleMaterializationError::MissingEnemyMapping)?;
                    WaveSlotDefinition::new(
                        checked_sequence(slot_index)?,
                        checked_formation(slot_index)?,
                        enemy,
                        Some(checked_level(member.stage_level())?.get()),
                        None,
                        true,
                    )
                    .ok_or(UniverseBattleMaterializationError::InvalidEncounter)
                })
                .collect::<Result<Vec<_>, _>>()?;
            CombatEncounterWave::new(
                member_wave_id(wave.id().get())?,
                checked_sequence(wave_index)?,
                None,
                None,
                WaveCarry::CARRY_ALL,
                slots,
            )
            .ok_or(UniverseBattleMaterializationError::InvalidEncounter)
        })
        .collect::<Result<Vec<_>, _>>()?;
    EncounterDefinition::new(encounter, Vec::new(), Vec::new())
        .with_authored_waves(WaveTransitionPolicy::AfterAction, waves)
        .ok_or(UniverseBattleMaterializationError::InvalidEncounter)
}

fn difficulty_encounter(
    index: usize,
    binding: &DifficultyEnemyBinding,
    enemies: &BTreeMap<&str, EnemyDefinitionId>,
) -> Result<EncounterDefinition, UniverseBattleMaterializationError> {
    let enemy = *enemies
        .get(binding.enemy_variant_key())
        .ok_or(UniverseBattleMaterializationError::MissingEnemyMapping)?;
    let encounter = difficulty_encounter_id(index)?;
    let wave = CombatEncounterWave::new(
        difficulty_wave_id(index)?,
        1,
        None,
        None,
        WaveCarry::CARRY_ALL,
        vec![
            WaveSlotDefinition::new(
                1,
                FormationIndex::new(0).expect("zero formation is valid"),
                enemy,
                Some(checked_level(binding.level())?.get()),
                None,
                true,
            )
            .expect("checked difficulty slot is valid"),
        ],
    )
    .expect("checked difficulty wave is valid");
    EncounterDefinition::new(encounter, Vec::new(), Vec::new())
        .with_authored_waves(WaveTransitionPolicy::AfterAction, vec![wave])
        .ok_or(UniverseBattleMaterializationError::InvalidEncounter)
}

fn player_participants(
    roster: &UniverseBattleRoster,
    contributions: &UniverseBattleContributionSet,
) -> Result<Vec<ParticipantSpec>, UniverseBattleMaterializationError> {
    roster
        .entries()
        .iter()
        .map(|entry| {
            Ok(ParticipantSpec::new(
                TeamSide::Player,
                entry.formation(),
                ParticipantSource::Player,
                apply_party_modifiers(entry.combatant(), contributions)?,
            ))
        })
        .collect()
}

fn apply_party_modifiers(
    base: &ResolvedCombatantSpec,
    contributions: &UniverseBattleContributionSet,
) -> Result<ResolvedCombatantSpec, UniverseBattleMaterializationError> {
    let mut modifier_ids = base.modifiers().to_vec();
    modifier_ids.extend(
        contributions
            .modifiers()
            .iter()
            .map(|binding| binding.definition().id),
    );
    modifier_ids.sort_unstable();
    if modifier_ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(UniverseBattleMaterializationError::ContributionCollision);
    }
    let mut sources = base.sources().to_vec();
    sources.extend(
        contributions
            .modifiers()
            .iter()
            .map(|binding| binding.source().clone()),
    );
    sources.sort_unstable_by_key(|source| source.definition());
    if sources
        .windows(2)
        .any(|pair| pair[0].definition() == pair[1].definition())
    {
        return Err(UniverseBattleMaterializationError::ContributionCollision);
    }
    let mut modifier_bindings = base.modifier_bindings().to_vec();
    modifier_bindings.extend(contributions.modifiers().iter().map(|binding| {
        ResolvedModifierBinding::new(binding.definition().id, binding.source().definition())
    }));
    modifier_bindings.sort_unstable_by_key(|binding| binding.definition());
    let digest = combatant_digest(base, contributions);
    let mut resolved = ResolvedCombatantSpec::new(
        base.form(),
        base.level(),
        base.maximum_hp(),
        base.speed(),
        ResolvedDefinitionBindings::new(
            base.abilities().to_vec(),
            base.rule_bundles().to_vec(),
            modifier_ids,
        )
        .map_err(|_| UniverseBattleMaterializationError::InvalidCombatant)?,
        CombatantSpecDigest::new(digest).expect("SHA-256 digest is non-zero"),
    )
    .map_err(|_| UniverseBattleMaterializationError::InvalidCombatant)?
    .with_base_attack_defense(base.base_attack(), base.base_defense())
    .with_energy(base.current_energy(), base.maximum_energy())
    .map_err(|_| UniverseBattleMaterializationError::InvalidCombatant)?
    .with_toughness(
        base.rank(),
        base.weaknesses().to_vec(),
        base.toughness_layers().to_vec(),
    )
    .map_err(|_| UniverseBattleMaterializationError::InvalidCombatant)?;
    resolved = resolved
        .with_sources(sources)
        .map_err(|_| UniverseBattleMaterializationError::InvalidCombatant)?;
    resolved
        .with_modifier_bindings(modifier_bindings)
        .map_err(|_| UniverseBattleMaterializationError::InvalidCombatant)
}

fn member_spec(
    member: &EncounterMemberDefinition,
    players: &[ParticipantSpec],
    enemy_map: &BTreeMap<&str, EnemyDefinitionId>,
    catalog: &CombatCatalog,
    revision: &str,
    root_digest: [u8; 32],
) -> Result<BattleSpec, UniverseBattleMaterializationError> {
    let mut participants = players.to_vec();
    for (wave_index, wave) in member.waves().iter().enumerate() {
        for (slot_index, slot) in wave.enemies().iter().enumerate() {
            let enemy = *enemy_map
                .get(slot.enemy_variant_key())
                .ok_or(UniverseBattleMaterializationError::MissingEnemyMapping)?;
            participants.push(enemy_participant(
                catalog,
                enemy,
                checked_level(member.stage_level())?,
                wave_index,
                slot_index,
                slot.enemy_variant_key(),
            )?);
        }
    }
    BattleSpec::new(
        revision,
        BattleSpecDigest::new(spec_digest(
            root_digest,
            0,
            member.id().get(),
            &participants,
        ))
        .expect("SHA-256 digest is non-zero"),
        member_encounter_id(member.id())?,
        participants,
        TeamResourceSpec::new(3, 5).expect("standard player resources are valid"),
        TeamResourceSpec::new(0, 0).expect("empty enemy resources are valid"),
        ConcedePolicy::Allowed,
    )
    .map_err(|_| UniverseBattleMaterializationError::InvalidBattleSpec)
}

#[allow(clippy::too_many_arguments)]
fn difficulty_spec(
    index: usize,
    binding: &DifficultyEnemyBinding,
    players: &[ParticipantSpec],
    enemy_map: &BTreeMap<&str, EnemyDefinitionId>,
    catalog: &CombatCatalog,
    revision: &str,
    root_digest: [u8; 32],
) -> Result<BattleSpec, UniverseBattleMaterializationError> {
    let enemy = *enemy_map
        .get(binding.enemy_variant_key())
        .ok_or(UniverseBattleMaterializationError::MissingEnemyMapping)?;
    let mut participants = players.to_vec();
    participants.push(enemy_participant(
        catalog,
        enemy,
        checked_level(binding.level())?,
        0,
        0,
        binding.enemy_variant_key(),
    )?);
    BattleSpec::new(
        revision,
        BattleSpecDigest::new(spec_digest(
            root_digest,
            1,
            u32::try_from(index + 1)
                .map_err(|_| UniverseBattleMaterializationError::IdentityOverflow)?,
            &participants,
        ))
        .expect("SHA-256 digest is non-zero"),
        difficulty_encounter_id(index)?,
        participants,
        TeamResourceSpec::new(3, 5).expect("standard player resources are valid"),
        TeamResourceSpec::new(0, 0).expect("empty enemy resources are valid"),
        ConcedePolicy::Allowed,
    )
    .map_err(|_| UniverseBattleMaterializationError::InvalidBattleSpec)
}

fn enemy_participant(
    catalog: &CombatCatalog,
    enemy_id: EnemyDefinitionId,
    level: UnitLevel,
    wave_index: usize,
    slot_index: usize,
    source_key: &str,
) -> Result<ParticipantSpec, UniverseBattleMaterializationError> {
    let enemy = catalog
        .enemy(enemy_id)
        .ok_or(UniverseBattleMaterializationError::MissingProxyEnemy)?;
    let combatant = ResolvedCombatantSpec::new(
        enemy.unit(),
        level,
        Hp::new(1).expect("Goal 01 executable proxy HP is positive"),
        Speed::from_scaled(50_000_000).expect("static proxy Speed is valid"),
        ResolvedDefinitionBindings::new(enemy.abilities().to_vec(), Vec::new(), Vec::new())
            .map_err(|_| UniverseBattleMaterializationError::InvalidCombatant)?,
        CombatantSpecDigest::new(enemy_digest(
            enemy_id, level, wave_index, slot_index, source_key,
        ))
        .expect("SHA-256 digest is non-zero"),
    )
    .map_err(|_| UniverseBattleMaterializationError::InvalidCombatant)?
    .with_energy(Energy::ZERO, Energy::ZERO)
    .map_err(|_| UniverseBattleMaterializationError::InvalidCombatant)?;
    ParticipantSpec::new(
        TeamSide::Enemy,
        checked_formation(slot_index)?,
        ParticipantSource::EncounterEnemy(enemy_id),
        combatant,
    )
    .with_wave(checked_sequence(wave_index)?)
    .ok_or(UniverseBattleMaterializationError::InvalidBattleSpec)
}

fn settlement_contract(
    roster: &UniverseBattleRoster,
) -> Result<Arc<ActivityBattleResultContract>, UniverseBattleMaterializationError> {
    let mut fields = vec![
        ProjectionField::Outcome,
        ProjectionField::FinalStateHash,
        ProjectionField::EventDigest,
        ProjectionField::TerminalFault,
    ];
    fields.extend(
        roster
            .entries()
            .iter()
            .map(|entry| ProjectionField::ParticipantState(entry.participant())),
    );
    let projection = BattleResultProjection::new(
        ProjectionId::new(PROJECTION_ID).expect("reserved projection ID is non-zero"),
        fields,
    )
    .map_err(|_| UniverseBattleMaterializationError::InvalidBattleBinding)?;
    let carry = roster
        .entries()
        .iter()
        .map(|entry| {
            ActivityParticipantCarryDefinition::new(
                entry.participant(),
                HpCarryPolicy::CarryExact,
                EnergyCarryPolicy::CarryExact,
                LifeCarryPolicy::CarryExact,
                PresenceCarryPolicy::CarryExact,
            )
        })
        .collect();
    ActivityBattleResultContract::new(Arc::new(projection), carry, Vec::new())
        .map(Arc::new)
        .map_err(|_| UniverseBattleMaterializationError::InvalidBattleBinding)
}

fn validate_executable(
    catalog: &Arc<CombatCatalog>,
    spec: &BattleSpec,
) -> Result<(), UniverseBattleMaterializationError> {
    Battle::create(
        Arc::clone(catalog),
        spec.clone(),
        BattleSeed::new([0x5a; 32]),
    )
    .map(|_| ())
    .map_err(|_| UniverseBattleMaterializationError::NonExecutableBattleSpec)
}

fn checked_level(raw: u32) -> Result<UnitLevel, UniverseBattleMaterializationError> {
    u8::try_from(raw)
        .ok()
        .and_then(UnitLevel::new)
        .ok_or(UniverseBattleMaterializationError::InvalidLevel)
}

fn checked_sequence(index: usize) -> Result<u16, UniverseBattleMaterializationError> {
    u16::try_from(index + 1).map_err(|_| UniverseBattleMaterializationError::IdentityOverflow)
}

fn checked_formation(index: usize) -> Result<FormationIndex, UniverseBattleMaterializationError> {
    u8::try_from(index)
        .ok()
        .and_then(FormationIndex::new)
        .ok_or(UniverseBattleMaterializationError::InvalidEncounter)
}

fn member_encounter_id(
    member: EncounterMemberId,
) -> Result<EncounterId, UniverseBattleMaterializationError> {
    EncounterId::new(
        MEMBER_ENCOUNTER_ID_BASE
            .checked_add(member.get())
            .ok_or(UniverseBattleMaterializationError::IdentityOverflow)?,
    )
    .ok_or(UniverseBattleMaterializationError::IdentityOverflow)
}

fn difficulty_encounter_id(
    index: usize,
) -> Result<EncounterId, UniverseBattleMaterializationError> {
    EncounterId::new(
        DIFFICULTY_ENCOUNTER_ID_BASE
            .checked_add(
                u32::try_from(index + 1)
                    .map_err(|_| UniverseBattleMaterializationError::IdentityOverflow)?,
            )
            .ok_or(UniverseBattleMaterializationError::IdentityOverflow)?,
    )
    .ok_or(UniverseBattleMaterializationError::IdentityOverflow)
}

fn member_wave_id(raw: u32) -> Result<EncounterWaveId, UniverseBattleMaterializationError> {
    EncounterWaveId::new(
        MEMBER_WAVE_ID_BASE
            .checked_add(raw)
            .ok_or(UniverseBattleMaterializationError::IdentityOverflow)?,
    )
    .ok_or(UniverseBattleMaterializationError::IdentityOverflow)
}

fn difficulty_wave_id(index: usize) -> Result<EncounterWaveId, UniverseBattleMaterializationError> {
    EncounterWaveId::new(
        DIFFICULTY_WAVE_ID_BASE
            .checked_add(
                u32::try_from(index + 1)
                    .map_err(|_| UniverseBattleMaterializationError::IdentityOverflow)?,
            )
            .ok_or(UniverseBattleMaterializationError::IdentityOverflow)?,
    )
    .ok_or(UniverseBattleMaterializationError::IdentityOverflow)
}

fn root_digest(
    universe: &UniverseCatalog,
    roster: &UniverseBattleRoster,
    contributions: &UniverseBattleContributionSet,
    enemies: &[UniverseEnemyMaterialization],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.standard-universe.battle-materialization.root.v1");
    encoder.text(UNIVERSE_BATTLE_MATERIALIZATION_REVISION);
    encoder.digest(universe.identity().universe_bundle_digest().bytes());
    encoder.digest(roster.participant_lock().bytes());
    encoder.digest(contributions.digest());
    encoder.u32(enemies.len() as u32);
    for enemy in enemies {
        encoder.text(enemy.stable_key());
        encoder.u8(enemy.definition_match as u8);
        encoder.u32(enemy.combat_enemy().get());
        encoder.optional_text(enemy.proxy_stable_key());
    }
    encoder.finish()
}

fn combatant_digest(
    base: &ResolvedCombatantSpec,
    contributions: &UniverseBattleContributionSet,
) -> [u8; 32] {
    let mut encoder =
        Encoder::new(b"starclock.standard-universe.player-combatant-materialization.v1");
    encoder.digest(base.digest().bytes());
    encoder.digest(contributions.digest());
    encoder.finish()
}

fn enemy_digest(
    enemy: EnemyDefinitionId,
    level: UnitLevel,
    wave_index: usize,
    slot_index: usize,
    source_key: &str,
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.standard-universe.enemy-proxy-combatant.v1");
    encoder.text(UNIVERSE_ENEMY_RUNTIME_STAT_POLICY);
    encoder.text(source_key);
    encoder.u32(enemy.get());
    encoder.u8(level.get());
    encoder.u32(wave_index as u32);
    encoder.u32(slot_index as u32);
    encoder.finish()
}

fn spec_digest(
    root: [u8; 32],
    kind: u8,
    identity: u32,
    participants: &[ParticipantSpec],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.standard-universe.battle-spec.v1");
    encoder.digest(root);
    encoder.u8(kind);
    encoder.u32(identity);
    encoder.u32(participants.len() as u32);
    for participant in participants {
        encoder.u8(participant.side() as u8);
        encoder.u8(participant.formation().get());
        encoder.u32(u32::from(participant.wave()));
        encoder.digest(participant.combatant().digest().bytes());
    }
    encoder.finish()
}

fn coverage_digest(
    wave_count: usize,
    enemy_slot_count: usize,
    exact: usize,
    declared_rules: usize,
    enemies: &[UniverseEnemyMaterialization],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.standard-universe.battle-coverage.v1");
    encoder.u32(MEMBER_COUNT as u32);
    encoder.u32(wave_count as u32);
    encoder.u32(enemy_slot_count as u32);
    encoder.u32(DIFFICULTY_BINDING_COUNT as u32);
    encoder.u32(enemies.len() as u32);
    encoder.u32(exact as u32);
    encoder.u32((enemies.len() - exact) as u32);
    encoder.u32(declared_rules as u32);
    encoder.u32(0);
    encoder.text(UNIVERSE_ENEMY_RUNTIME_STAT_POLICY);
    for enemy in enemies {
        encoder.text(enemy.stable_key());
        encoder.u8(enemy.definition_match as u8);
        encoder.u32(enemy.combat_enemy().get());
        encoder.optional_text(enemy.proxy_stable_key());
    }
    encoder.finish()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UniverseBattleMaterializationError {
    InvalidEncounterContent,
    RosterMismatch,
    MissingProxyEnemy,
    MissingEnemyMapping,
    ContributionCollision,
    InvalidCompositeCatalog,
    InvalidEncounter,
    InvalidLevel,
    InvalidCombatant,
    InvalidBattleSpec,
    NonExecutableBattleSpec,
    InvalidBattleBinding,
    InvalidBattleOverlay,
    InvalidDenominator,
    IdentityOverflow,
}

impl core::fmt::Display for UniverseBattleMaterializationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Standard Universe battle materialization failed: {self:?}"
        )
    }
}

impl std::error::Error for UniverseBattleMaterializationError {}
