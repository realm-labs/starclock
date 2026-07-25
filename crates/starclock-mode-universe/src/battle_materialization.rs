//! Standard Universe encounter rows lowered into validated combat requests.

mod battle_spec;
pub mod catalog_composition;
#[path = "battle_materialization_digest.rs"]
mod materialization_digest;
mod player;

use std::{collections::BTreeMap, sync::Arc};

use starclock_activity::{
    ActivityBattleResultContract, ActivityOptionId, ActivityParticipantCarryDefinition,
    BattleBinding, BattleResultProjection, EncounterInitiativePolicy, EnergyCarryPolicy,
    HpCarryPolicy, LifeCarryPolicy, ParticipantId, ParticipantLock, PreparedBattleVariant,
    PresenceCarryPolicy, ProjectionField, ProjectionId, TechniqueContributionDigest,
};
use starclock_combat::{
    Battle, BattleSeed, BattleSpec, EncounterId, EncounterWaveId, EnemyDefinitionId,
    FormationIndex, ResolvedCombatantSpec, UnitLevel,
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
    battle_assembly::BattleAssemblyKey,
    battle_contribution::UniverseBattleContributionSet,
    battle_overlay::{UniverseEncounterBattleBinding, UniverseEncounterOverlay},
    battle_snapshot::StandardUniverseBattleSnapshot,
    battle_technique::{CompiledUniverseBattleTechnique, UniverseBattleTechniqueDefinition},
    catalog::UniverseCatalog,
    encounter::{DifficultyEnemyBinding, EncounterMemberDefinition, EnemyRole},
    encounter_content_runtime::EncounterContentRuntimeCatalog,
    id::{DifficultyId, EncounterMemberId},
};
use battle_spec::{difficulty_spec, member_spec};
use catalog_composition::UniverseBattleCatalogComposition;
use materialization_digest::{
    coverage_digest, empty_carry_digest, root_digest, snapshot_root_digest,
    technique_variant_digest,
};
use player::player_participants;

pub const UNIVERSE_BATTLE_MATERIALIZATION_REVISION: &str =
    "standard-universe-battle-materialization-v2";
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
    assembly_key: BattleAssemblyKey,
    combat_catalog: Arc<CombatCatalog>,
    overlay: UniverseEncounterOverlay,
    difficulty_specs: Box<[UniverseDifficultyBattleSpec]>,
    enemies: Box<[UniverseEnemyMaterialization]>,
    coverage: UniverseBattleMaterializationCoverage,
    techniques: Box<[UniverseBattleTechniqueDefinition]>,
    digest: [u8; 32],
}

impl UniverseBattleMaterialization {
    #[must_use]
    pub const fn assembly_key(&self) -> BattleAssemblyKey {
        self.assembly_key
    }

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
    pub fn techniques(&self) -> &[UniverseBattleTechniqueDefinition] {
        &self.techniques
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
        let composition = UniverseBattleCatalogComposition::compile(universe)?;
        self.compile_inner(universe, &composition, roster, contributions, None, None)
    }

    pub fn compile_with_technique(
        self,
        universe: &UniverseCatalog,
        roster: &UniverseBattleRoster,
        contributions: &UniverseBattleContributionSet,
        technique: UniverseBattleTechniqueDefinition,
    ) -> Result<UniverseBattleMaterialization, UniverseBattleMaterializationError> {
        let composition = UniverseBattleCatalogComposition::compile(universe)?;
        self.compile_inner(
            universe,
            &composition,
            roster,
            contributions,
            Some(technique),
            None,
        )
    }

    pub fn compile_from_composition(
        self,
        universe: &UniverseCatalog,
        composition: &UniverseBattleCatalogComposition,
        roster: &UniverseBattleRoster,
        contributions: &UniverseBattleContributionSet,
    ) -> Result<UniverseBattleMaterialization, UniverseBattleMaterializationError> {
        self.compile_inner(universe, composition, roster, contributions, None, None)
    }

    pub fn compile_from_composition_with_technique(
        self,
        universe: &UniverseCatalog,
        composition: &UniverseBattleCatalogComposition,
        roster: &UniverseBattleRoster,
        contributions: &UniverseBattleContributionSet,
        technique: UniverseBattleTechniqueDefinition,
    ) -> Result<UniverseBattleMaterialization, UniverseBattleMaterializationError> {
        self.compile_inner(
            universe,
            composition,
            roster,
            contributions,
            Some(technique),
            None,
        )
    }

    pub fn compile_snapshot_from_composition(
        self,
        universe: &UniverseCatalog,
        composition: &UniverseBattleCatalogComposition,
        roster: &UniverseBattleRoster,
        snapshot: &StandardUniverseBattleSnapshot,
    ) -> Result<UniverseBattleMaterialization, UniverseBattleMaterializationError> {
        self.compile_inner(
            universe,
            composition,
            roster,
            snapshot.contributions(),
            None,
            Some(snapshot),
        )
    }

    pub fn compile_snapshot_from_composition_with_technique(
        self,
        universe: &UniverseCatalog,
        composition: &UniverseBattleCatalogComposition,
        roster: &UniverseBattleRoster,
        snapshot: &StandardUniverseBattleSnapshot,
        technique: UniverseBattleTechniqueDefinition,
    ) -> Result<UniverseBattleMaterialization, UniverseBattleMaterializationError> {
        self.compile_inner(
            universe,
            composition,
            roster,
            snapshot.contributions(),
            Some(technique),
            Some(snapshot),
        )
    }

    pub fn snapshot_assembly_key(
        self,
        composition: &UniverseBattleCatalogComposition,
        roster: &UniverseBattleRoster,
        snapshot: &StandardUniverseBattleSnapshot,
        technique: Option<UniverseBattleTechniqueDefinition>,
    ) -> Result<BattleAssemblyKey, UniverseBattleMaterializationError> {
        if snapshot.participant_lock() != roster.participant_lock() {
            return Err(UniverseBattleMaterializationError::RosterMismatch);
        }
        let technique = technique
            .map(|definition| compile_technique(composition, roster, definition))
            .transpose()?;
        Ok(BattleAssemblyKey::new(
            composition.digest(),
            roster.participant_lock(),
            composition.content().digest(),
            snapshot.digest(),
            snapshot.carry_digest(),
            technique
                .as_ref()
                .map(CompiledUniverseBattleTechnique::digest),
        ))
    }

    fn compile_inner(
        self,
        universe: &UniverseCatalog,
        composition: &UniverseBattleCatalogComposition,
        roster: &UniverseBattleRoster,
        contributions: &UniverseBattleContributionSet,
        technique: Option<UniverseBattleTechniqueDefinition>,
        snapshot: Option<&StandardUniverseBattleSnapshot>,
    ) -> Result<UniverseBattleMaterialization, UniverseBattleMaterializationError> {
        if snapshot.is_some_and(|snapshot| snapshot.participant_lock() != roster.participant_lock())
        {
            return Err(UniverseBattleMaterializationError::RosterMismatch);
        }
        if composition.digest()
            != materialization_digest::catalog_composition_digest(
                universe,
                composition.content().digest(),
                composition.enemies(),
            )
        {
            return Err(UniverseBattleMaterializationError::CatalogCompositionMismatch);
        }
        let enemy_map = composition
            .enemies()
            .iter()
            .map(|enemy| (enemy.stable_key(), enemy.combat_enemy()))
            .collect::<BTreeMap<_, _>>();
        let technique = technique
            .map(|definition| compile_technique(composition, roster, definition))
            .transpose()?;
        let static_digest = root_digest(
            universe,
            roster,
            contributions,
            composition.enemies(),
            technique.as_ref(),
        );
        let digest = snapshot.map_or(static_digest, |snapshot| {
            snapshot_root_digest(static_digest, snapshot.digest())
        });
        let assembly_key = BattleAssemblyKey::new(
            composition.digest(),
            roster.participant_lock(),
            composition.content().digest(),
            snapshot.map_or_else(
                || contributions.digest(),
                StandardUniverseBattleSnapshot::digest,
            ),
            snapshot.map_or_else(
                empty_carry_digest,
                StandardUniverseBattleSnapshot::carry_digest,
            ),
            technique
                .as_ref()
                .map(CompiledUniverseBattleTechnique::digest),
        );
        let revision = composition.revision();
        let mut builder =
            CombatCatalogBuilder::from_catalog(composition.combat_catalog(), revision, digest);
        for modifier in contributions.modifiers() {
            builder.add_modifier_group(modifier.group().clone());
            builder.add_modifier(modifier.definition().clone());
        }
        for executable in contributions.executable_rules() {
            for group in executable.modifier_groups() {
                builder.add_modifier_group(group.clone());
            }
            for modifier in executable.modifiers() {
                builder.add_modifier(modifier.clone());
            }
            for selector in executable.selectors() {
                builder.add_selector(selector.clone());
            }
            for program in executable.programs() {
                builder.add_program(program.clone());
            }
            for effect in executable.effects() {
                builder.add_effect(effect.clone());
            }
            builder.add_rule(executable.definition().clone());
            builder.add_rule_bundle(executable.bundle().clone());
        }
        if let Some(resonance) = contributions.resonance() {
            for group in resonance.modifier_groups() {
                builder.add_modifier_group(group.clone());
            }
            for modifier in resonance.modifiers() {
                builder.add_modifier(modifier.clone());
            }
            for selector in resonance.selectors() {
                builder.add_selector(selector.clone());
            }
            for effect in resonance.effects() {
                builder.add_effect(effect.clone());
            }
            for program in resonance.programs() {
                builder.add_program(program.clone());
            }
            builder.add_ability(resonance.ability().clone());
            for ability in resonance.auxiliary_abilities() {
                builder.add_ability(ability.clone());
            }
            for countdown in resonance.countdowns() {
                builder.add_countdown(*countdown);
            }
        }
        if let Some(technique) = &technique {
            builder.add_selector(technique.actor_selector().clone());
            builder.add_selector(technique.target_selector().clone());
            builder.add_program(technique.program().clone());
            builder.add_rule(technique.rule().clone());
            builder.add_rule_bundle(technique.bundle().clone());
        }
        let combat_catalog = builder
            .build()
            .map_err(|_| UniverseBattleMaterializationError::InvalidCompositeCatalog)?;
        let carry = snapshot.map_or(&[][..], StandardUniverseBattleSnapshot::participant_carry);
        let players = player_participants(roster, contributions, None, carry)?;
        let technique_players = technique
            .as_ref()
            .map(|technique| player_participants(roster, contributions, Some(technique), carry))
            .transpose()?;
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
                revision,
                digest,
                contributions,
            )?;
            validate_executable(&combat_catalog, &spec)?;
            let mut variants = vec![PreparedBattleVariant::new(
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
            )];
            if let (Some(technique), Some(technique_players)) =
                (technique.as_ref(), technique_players.as_ref())
            {
                let technique_spec = member_spec(
                    member,
                    technique_players,
                    &enemy_map,
                    &combat_catalog,
                    revision,
                    digest,
                    contributions,
                )?;
                validate_executable(&combat_catalog, &technique_spec)?;
                variants.push(PreparedBattleVariant::new(
                    vec![technique.definition().option()],
                    TechniqueContributionDigest::new(technique_variant_digest(
                        contributions.digest(),
                        technique.digest(),
                    ))
                    .expect("combined technique digest is non-zero"),
                    BattleBinding::new(
                        technique_spec,
                        "standard-universe-battle-technique",
                        UNIVERSE_BATTLE_MATERIALIZATION_REVISION,
                        roster.participant_lock(),
                    )
                    .map_err(|_| UniverseBattleMaterializationError::InvalidBattleBinding)?,
                ));
            }
            let preparation = starclock_activity::EncounterPreparationDefinition::new(
                ActivityOptionId::new(u64::from(NORMAL_ENGAGEMENT_OPTION))
                    .expect("reserved engagement option is non-zero"),
                EncounterInitiativePolicy::PlayerControlled,
                roster.participant_lock(),
                0,
                technique
                    .as_ref()
                    .map(|technique| vec![technique.activity_definition()])
                    .unwrap_or_default(),
                variants,
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
        composition
            .content()
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
                revision,
                digest,
                contributions,
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
        let exact = composition
            .enemies()
            .iter()
            .filter(|enemy| enemy.definition_match == EnemyDefinitionMatch::Exact)
            .count();
        if composition.enemies().len() != ENEMY_VARIANT_COUNT || exact != EXACT_ENEMY_VARIANT_COUNT
        {
            return Err(UniverseBattleMaterializationError::InvalidDenominator);
        }
        let coverage_digest = coverage_digest(
            member_wave_count,
            member_enemy_slot_count,
            exact,
            contributions.rules().len(),
            contributions.materialized_rule_binding_count(),
            composition.enemies(),
        );
        let coverage = UniverseBattleMaterializationCoverage {
            member_count: MEMBER_COUNT as u16,
            member_wave_count: member_wave_count as u16,
            member_enemy_slot_count: member_enemy_slot_count as u16,
            difficulty_binding_count: DIFFICULTY_BINDING_COUNT as u16,
            enemy_variant_count: ENEMY_VARIANT_COUNT as u16,
            exact_enemy_variant_count: exact as u16,
            approximate_enemy_variant_count: (composition.enemies().len() - exact) as u16,
            declared_rule_binding_count: u16::try_from(contributions.rules().len())
                .map_err(|_| UniverseBattleMaterializationError::InvalidDenominator)?,
            materialized_rule_binding_count: u16::try_from(
                contributions.materialized_rule_binding_count(),
            )
            .map_err(|_| UniverseBattleMaterializationError::InvalidDenominator)?,
            runtime_stat_policy: UNIVERSE_ENEMY_RUNTIME_STAT_POLICY.into(),
            digest: coverage_digest,
        };
        Ok(UniverseBattleMaterialization {
            assembly_key,
            combat_catalog,
            overlay,
            difficulty_specs: difficulty_specs.into_boxed_slice(),
            enemies: composition.enemies().to_vec().into_boxed_slice(),
            coverage,
            techniques: technique
                .as_ref()
                .map(|technique| vec![technique.definition()])
                .unwrap_or_default()
                .into_boxed_slice(),
            digest,
        })
    }
}

fn compile_technique(
    composition: &UniverseBattleCatalogComposition,
    roster: &UniverseBattleRoster,
    definition: UniverseBattleTechniqueDefinition,
) -> Result<CompiledUniverseBattleTechnique, UniverseBattleMaterializationError> {
    let entry = roster
        .entries()
        .iter()
        .find(|entry| entry.participant() == definition.participant())
        .ok_or(UniverseBattleMaterializationError::TechniqueMismatch)?;
    if entry
        .combatant()
        .abilities()
        .binary_search(&definition.ability())
        .is_err()
    {
        return Err(UniverseBattleMaterializationError::TechniqueMismatch);
    }
    CompiledUniverseBattleTechnique::compile(composition.combat_catalog(), definition)
        .map_err(|_| UniverseBattleMaterializationError::InvalidTechnique)
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UniverseBattleMaterializationError {
    InvalidEncounterContent,
    CatalogCompositionMismatch,
    RosterMismatch,
    MissingProxyEnemy,
    MissingEnemyMapping,
    ContributionCollision,
    InvalidCompositeCatalog,
    InvalidEncounter,
    InvalidLevel,
    InvalidCombatant,
    InvalidCarry,
    InvalidBattleSpec,
    NonExecutableBattleSpec,
    InvalidBattleBinding,
    InvalidBattleOverlay,
    InvalidDenominator,
    IdentityOverflow,
    InvalidTechnique,
    TechniqueMismatch,
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
