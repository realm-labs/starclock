//! Frozen Ordinary vertical-slice battle assembly, execution and settlement.

use std::sync::Arc;

use crate::digest::CanonicalDigestBuilder;
use starclock_activity::{
    ActivityBattleHandoff, ActivityBattlePreparationRequest, ActivityBattleResultContract,
    ActivityOptionId, ActivityParticipantCarryDefinition, ActivityPreparationBoundary,
    ActivityRosterLock, ActivityScopePath, ActivityStateHash, AttemptId, BattleBinding,
    BattleResult, BattleSequence, EncounterInitiativePolicy, EncounterPreparationDefinition,
    EnergyCarryPolicy, GraphActivity, HpCarryPolicy, LifeCarryPolicy, PreparedBattleVariant,
    PresenceCarryPolicy, ProjectionField, ProjectionId, TechniqueContributionDigest,
};
use starclock_combat::{
    AssemblyDigest, BattleSpec, CombatantSpecDigest, ConcedePolicy, EncounterId, Energy,
    FormationIndex, Hp, KeyedTeamResourceSpec, ParticipantSource, ParticipantSpec, Ratio,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, Rounding, SourceDefinitionId,
    TeamResourceSpec, TeamResourceWavePolicy, TeamSide, UnitLevel,
    catalog::{CombatCatalog, builder::CombatCatalogBuilder, definition::EncounterDefinition},
};
use starclock_data::catalog::SimulationCatalog;

use crate::{
    baseline_runner::{NestedBattleExecutionError, NestedBattleExecutor},
    nested_battle_executor::{NestedBattleExecutionReport, UniverseNestedBattleExecutor},
};

use super::{
    battle_settlement_runtime::{
        DivergentUniverseBattleSettlementError, DivergentUniverseBattleSettlementRuntime,
    },
    entry_flow::{DivergentUniverseFlowInstance, DivergentUniverseRuntimeFactory},
    state::TITAN_BOONS_SLOT,
};

const ENCOUNTER_GROUP_ID: &str = "divergent-universe.encounter-group.300202";
const ENCOUNTER_STAGE_ID: &str = "83002081";
const ENCOUNTER_ID_RAW: u32 = 0x7e22_0001;
const PROJECTION_ID_RAW: u32 = 0x7e22_0002;
const TITAN_STAGE_ABILITY_RESOURCE_RAW: u32 = 0x7e22_0003;
const TITAN_STAGE_ABILITY_KEY: &str = "StageAbility_634020";
const ENGAGEMENT_OPTION_RAW: u64 = 0x7e22_0001;
pub(super) const ENEMY_STAT_DIFFICULTY: &str = "standard-universe-v1";
// The released DU candidate exposes source monster locators but the current core
// catalog has no exact stable-key join. Until P6 closes that binding, preserve
// the exact stage wave/slot/level shape and use one deliberately weak, labeled
// proxy so this architecture slice exercises combat without claiming parity.
pub(super) const ENEMY_PROXY: &str = "enemy.antibaryon.minion.variant.01";
const ENEMY_PROXY_STAT_SCALE: i64 = 100_000;

/// Accuracy of the frozen encounter selection and temporary enemy binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseEncounterSelectionAccuracy {
    VersionedProjectPolicyCandidateWithCalibratedSharedMinionProxy,
}

#[derive(Clone, Debug)]
pub struct DivergentUniverseBattleMaterialization {
    combat_catalog: Arc<CombatCatalog>,
    battle_spec: BattleSpec,
    control_battle_spec: BattleSpec,
    contribution: TechniqueContributionDigest,
    stage_id: Box<str>,
    encounter_group_id: Box<str>,
    accuracy: DivergentUniverseEncounterSelectionAccuracy,
}

impl DivergentUniverseBattleMaterialization {
    #[must_use]
    pub const fn combat_catalog(&self) -> &Arc<CombatCatalog> {
        &self.combat_catalog
    }
    #[must_use]
    pub const fn battle_spec(&self) -> &BattleSpec {
        &self.battle_spec
    }
    #[must_use]
    pub const fn control_battle_spec(&self) -> &BattleSpec {
        &self.control_battle_spec
    }
    #[must_use]
    pub const fn contribution_digest(&self) -> TechniqueContributionDigest {
        self.contribution
    }
    #[must_use]
    pub fn stage_id(&self) -> &str {
        &self.stage_id
    }
    #[must_use]
    pub fn encounter_group_id(&self) -> &str {
        &self.encounter_group_id
    }
    #[must_use]
    pub const fn accuracy(&self) -> DivergentUniverseEncounterSelectionAccuracy {
        self.accuracy
    }

    /// Produces the explicit no-Titan control used by deterministic release
    /// verification. It is not a selectable gameplay variant.
    #[must_use]
    pub fn control_verification(&self) -> Self {
        let control_contribution = digest(&[
            b"starclock.divergent-universe.vertical-slice.control-contribution.v1",
            &self.control_battle_spec.combat_input_digest().bytes(),
        ]);
        Self {
            combat_catalog: Arc::clone(&self.combat_catalog),
            battle_spec: self.control_battle_spec.clone(),
            control_battle_spec: self.control_battle_spec.clone(),
            contribution: TechniqueContributionDigest::new(control_contribution)
                .expect("SHA-256 control contribution digest is non-zero"),
            stage_id: self.stage_id.clone(),
            encounter_group_id: self.encounter_group_id.clone(),
            accuracy: self.accuracy,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DivergentUniverseBattleExecution {
    result: BattleResult,
    report: NestedBattleExecutionReport,
    settlement: starclock_activity::ActivityBattleSettlement,
    post_settlement_state_hash: ActivityStateHash,
}

impl DivergentUniverseBattleExecution {
    #[must_use]
    pub const fn result(&self) -> &BattleResult {
        &self.result
    }
    #[must_use]
    pub const fn report(&self) -> &NestedBattleExecutionReport {
        &self.report
    }
    #[must_use]
    pub const fn settlement(&self) -> starclock_activity::ActivityBattleSettlement {
        self.settlement
    }
    #[must_use]
    pub const fn post_settlement_state_hash(&self) -> ActivityStateHash {
        self.post_settlement_state_hash
    }
}

impl DivergentUniverseRuntimeFactory {
    pub fn materialize_first_ordinary_vertical_slice_battle(
        &self,
        flow: &DivergentUniverseFlowInstance,
        core: &SimulationCatalog,
    ) -> Result<DivergentUniverseBattleMaterialization, DivergentUniverseBattleError> {
        if !flow.is_first_ordinary_vertical_slice()
            || flow.component_digest != self.bundle.identity().component_digest().bytes()
        {
            return Err(DivergentUniverseBattleError::DefinitionMismatch);
        }
        let content = flow
            .first_ordinary_vertical_slice_content()
            .ok_or(DivergentUniverseBattleError::DefinitionMismatch)?;
        let mapping = flow
            .mapping_snapshot()
            .ok_or(DivergentUniverseBattleError::MappingRequired)?;
        mapping
            .validate(flow.component_digest, flow.definition.participants())
            .map_err(|_| DivergentUniverseBattleError::MappingRequired)?;
        validate_frozen_party(core, mapping)?;

        let encounter_catalog = self.bundle.encounter_catalog();
        let group = encounter_catalog
            .groups()
            .iter()
            .find(|value| value.id.as_str() == ENCOUNTER_GROUP_ID)
            .filter(|value| {
                value
                    .candidate_stage_ids
                    .iter()
                    .any(|stage| stage.as_ref() == ENCOUNTER_STAGE_ID)
            })
            .ok_or(DivergentUniverseBattleError::EncounterJoin)?;
        let mut waves = encounter_catalog
            .waves()
            .iter()
            .filter(|value| value.stage_id.as_ref() == ENCOUNTER_STAGE_ID)
            .collect::<Vec<_>>();
        waves.sort_unstable_by_key(|value| value.wave_index);
        if waves.is_empty()
            || waves
                .iter()
                .enumerate()
                .any(|(index, wave)| usize::from(wave.wave_index) != index + 1)
        {
            return Err(DivergentUniverseBattleError::EncounterJoin);
        }

        let encounter = EncounterId::new(ENCOUNTER_ID_RAW)
            .expect("reserved Divergent Universe encounter ID is non-zero");
        if core.combat_catalog().encounter(encounter).is_some() {
            return Err(DivergentUniverseBattleError::IdentityCollision);
        }
        let mut enemy_waves = Vec::with_capacity(waves.len());
        let mut enemy_participants = Vec::new();
        for wave in &waves {
            let mut slots = wave
                .enemy_slots
                .iter()
                .map(|id| {
                    encounter_catalog
                        .slots()
                        .iter()
                        .find(|slot| &slot.id == id)
                        .ok_or(DivergentUniverseBattleError::EncounterJoin)
                })
                .collect::<Result<Vec<_>, _>>()?;
            slots.sort_unstable_by_key(|slot| slot.slot_index);
            let mut enemy_ids = Vec::with_capacity(slots.len());
            for slot in slots {
                let enemy = core
                    .enemy_by_stable_key(&slot.monster_id)
                    .or_else(|| core.enemy_by_stable_key(ENEMY_PROXY))
                    .ok_or(DivergentUniverseBattleError::MissingEnemy)?;
                let level = parse_level(&slot.level)?;
                enemy_ids.push(enemy.id());
                enemy_participants.push(enemy_participant(
                    core,
                    enemy,
                    level,
                    wave.wave_index,
                    slot.slot_index,
                    &slot.monster_id,
                )?);
            }
            enemy_waves.push(enemy_ids);
        }

        let catalog_digest = digest(&[
            b"starclock.divergent-universe.vertical-slice.combat-catalog.v1",
            &core.combat_catalog().digest().bytes(),
            &flow.component_digest,
            ENCOUNTER_STAGE_ID.as_bytes(),
        ]);
        let definition = EncounterDefinition::new(encounter, Vec::new(), Vec::new())
            .with_waves(enemy_waves)
            .ok_or(DivergentUniverseBattleError::InvalidEncounter)?;
        let mut builder = CombatCatalogBuilder::from_catalog(core.combat_catalog(), catalog_digest);
        builder.add_encounter(definition);
        let combat_catalog = builder
            .build()
            .map_err(|_| DivergentUniverseBattleError::InvalidEncounter)?;

        let mut participants = player_participants(flow, mapping)?;
        participants.extend(enemy_participants);
        let control_assembly = assembly_digest(flow, mapping.digest().bytes(), [0; 32], false);
        let contribution_bytes = content.contribution_digest();
        let contributed_assembly =
            assembly_digest(flow, mapping.digest().bytes(), contribution_bytes, true);
        let control_battle_spec =
            battle_spec(control_assembly, encounter, participants.clone(), false)?;
        let battle_spec = battle_spec(contributed_assembly, encounter, participants, true)?;
        if battle_spec.combat_input_digest() == control_battle_spec.combat_input_digest() {
            return Err(DivergentUniverseBattleError::ContributionNotBound);
        }
        Ok(DivergentUniverseBattleMaterialization {
            combat_catalog,
            battle_spec,
            control_battle_spec,
            contribution: TechniqueContributionDigest::new(contribution_bytes)
                .expect("SHA-256 contribution digest is non-zero"),
            stage_id: ENCOUNTER_STAGE_ID.into(),
            encounter_group_id: group.id.as_str().into(),
            accuracy: DivergentUniverseEncounterSelectionAccuracy::VersionedProjectPolicyCandidateWithCalibratedSharedMinionProxy,
        })
    }
}

impl DivergentUniverseFlowInstance {
    pub fn start_first_ordinary_vertical_slice_battle(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        attempt: AttemptId,
        sequence: BattleSequence,
        materialization: &DivergentUniverseBattleMaterialization,
    ) -> Result<ActivityBattleHandoff, DivergentUniverseBattleError> {
        self.validate_battle_boundary(activity, expected_state_hash, materialization)?;
        let view = activity.player_view();
        let decision = view
            .decision()
            .filter(|decision| {
                decision.kind() == starclock_activity::ActivityDecisionKind::Encounter
                    && decision.options().len() == 1
            })
            .ok_or(DivergentUniverseBattleError::EncounterNotOffered)?;
        let option = decision.options()[0].id();
        let battle_node_raw = u32::try_from(self.layers().len())
            .ok()
            .and_then(|count| count.checked_add(3))
            .and_then(starclock_activity::NodeId::new)
            .ok_or(DivergentUniverseBattleError::InvalidScope)?;
        let section = starclock_activity::SectionId::new(1)
            .expect("the first logical layer has a non-zero section");
        let path = ActivityScopePath::new(activity.instance())
            .enter_section(section)
            .and_then(|path| path.enter_node(battle_node_raw))
            .and_then(|path| path.enter_attempt(attempt))
            .map_err(|_| DivergentUniverseBattleError::InvalidScope)?;
        let participants = self.definition.participants().as_ref().clone();
        let roster = ActivityRosterLock::new(
            ActivityScopePath::new(activity.instance()),
            participants.clone(),
        )
        .map_err(|_| DivergentUniverseBattleError::InvalidScope)?;
        let binding = BattleBinding::new(
            materialization.battle_spec.clone(),
            "divergent-universe-vertical-slice-battle",
            participants.digest(),
        )
        .map_err(|_| DivergentUniverseBattleError::InvalidBattleBinding)?;
        let normal = ActivityOptionId::new(ENGAGEMENT_OPTION_RAW)
            .expect("reserved engagement option is non-zero");
        let preparation = EncounterPreparationDefinition::new(
            normal,
            EncounterInitiativePolicy::PlayerControlled,
            participants.digest(),
            0,
            Vec::new(),
            vec![PreparedBattleVariant::new(
                Vec::new(),
                materialization.contribution,
                binding,
            )],
        )
        .map(Arc::new)
        .map_err(|_| DivergentUniverseBattleError::InvalidBattleBinding)?;
        let request = ActivityBattlePreparationRequest::new(path, roster, sequence, 0, preparation);
        let resolution = activity
            .engage_encounter(expected_state_hash, decision.id(), option, request)
            .map_err(|_| DivergentUniverseBattleError::PreparationRejected)?;
        if resolution.boundary() != ActivityPreparationBoundary::Decision
            || activity
                .choose_preparation_option(resolution.state_hash(), normal)
                .map_err(|_| DivergentUniverseBattleError::PreparationRejected)?
                != ActivityPreparationBoundary::BattleReady
        {
            return Err(DivergentUniverseBattleError::PreparationRejected);
        }
        activity
            .start_pending_battle(activity.state_hash(), settlement_contract(&participants)?)
            .map_err(|_| DivergentUniverseBattleError::BattleStartRejected)
    }

    pub fn execute_first_ordinary_vertical_slice_battle(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        attempt: AttemptId,
        sequence: BattleSequence,
        materialization: &DivergentUniverseBattleMaterialization,
    ) -> Result<DivergentUniverseBattleExecution, DivergentUniverseBattleError> {
        let handoff = self.start_first_ordinary_vertical_slice_battle(
            activity,
            expected_state_hash,
            attempt,
            sequence,
            materialization,
        )?;
        let mut executor =
            UniverseNestedBattleExecutor::new(Arc::clone(&materialization.combat_catalog));
        let result = match executor.execute(&handoff) {
            Ok(result) => result,
            Err(error) => {
                let restored = activity.rollback_pending_battle_start();
                debug_assert!(restored, "this adapter started the exact pending battle");
                return Err(DivergentUniverseBattleError::Execution(error));
            }
        };
        let report = executor
            .last_report()
            .cloned()
            .ok_or(DivergentUniverseBattleError::MissingExecutionReport)?;
        let resolution = DivergentUniverseBattleSettlementRuntime
            .settle_started_result(self, activity, activity.state_hash(), result.clone(), None)
            .map_err(DivergentUniverseBattleError::Settlement)?;
        if activity.player_view().completed_battle_count() == 0 {
            return Err(DivergentUniverseBattleError::SettlementRejected);
        }
        Ok(DivergentUniverseBattleExecution {
            result,
            report,
            settlement: resolution.settlement(),
            post_settlement_state_hash: activity.state_hash(),
        })
    }

    fn validate_battle_boundary(
        &self,
        activity: &GraphActivity,
        expected_state_hash: ActivityStateHash,
        materialization: &DivergentUniverseBattleMaterialization,
    ) -> Result<(), DivergentUniverseBattleError> {
        if !self.is_first_ordinary_vertical_slice()
            || activity.definition().identity() != self.definition.identity()
            || materialization.stage_id() != ENCOUNTER_STAGE_ID
            || materialization.encounter_group_id() != ENCOUNTER_GROUP_ID
        {
            return Err(DivergentUniverseBattleError::DefinitionMismatch);
        }
        if expected_state_hash != activity.state_hash() {
            return Err(DivergentUniverseBattleError::StaleState);
        }
        let content = self
            .first_ordinary_vertical_slice_content()
            .ok_or(DivergentUniverseBattleError::DefinitionMismatch)?;
        let accepted = activity
            .player_view()
            .slots()
            .iter()
            .find(|slot| slot.id() == TITAN_BOONS_SLOT)
            .and_then(|slot| match slot.value() {
                starclock_activity::ActivityValue::OrderedIdSet(values) => Some(values),
                _ => None,
            })
            .is_some_and(|values| values.binary_search(&content.titan_boon_key()).is_ok());
        if !accepted {
            return Err(DivergentUniverseBattleError::ContributionNotAccepted);
        }
        Ok(())
    }
}

pub(super) fn player_participants(
    flow: &DivergentUniverseFlowInstance,
    mapping: &super::mapping::DivergentUniverseMappingSnapshot,
) -> Result<Vec<ParticipantSpec>, DivergentUniverseBattleError> {
    let lock = flow.definition.participants();
    let mut participants = Vec::with_capacity(mapping.participants().len());
    for mapped in mapping.participants() {
        let locked = lock
            .entries()
            .iter()
            .find(|entry| entry.participant() == mapped.participant())
            .ok_or(DivergentUniverseBattleError::MappingRequired)?;
        let formation = FormationIndex::new(locked.formation_index())
            .ok_or(DivergentUniverseBattleError::InvalidBattleSpec)?;
        participants.push(
            ParticipantSpec::new(
                TeamSide::Player,
                formation,
                ParticipantSource::Player,
                mapped.compiled().combatant().clone(),
            )
            .with_locked_combatant_digest(locked.build().resolved_spec_digest()),
        );
    }
    Ok(participants)
}

pub(super) fn enemy_participant(
    core: &SimulationCatalog,
    enemy: &starclock_combat::catalog::definition::EnemyDefinition,
    level: UnitLevel,
    wave: u16,
    slot: u16,
    stable_key: &str,
) -> Result<ParticipantSpec, DivergentUniverseBattleError> {
    let stats = core
        .enemy_runtime_stat(enemy.id(), level, ENEMY_STAT_DIFFICULTY)
        .or_else(|| core.nearest_enemy_runtime_stat(enemy.id(), level, ENEMY_STAT_DIFFICULTY))
        .ok_or(DivergentUniverseBattleError::MissingEnemyStats)?;
    let proxy_scale = Ratio::from_scaled(ENEMY_PROXY_STAT_SCALE);
    let maximum_hp = proxy_scale
        .checked_apply(stats.hp(), Rounding::NearestTiesEven)
        .map_err(|_| DivergentUniverseBattleError::MissingEnemyStats)?
        .rounded_integer(Rounding::NearestTiesAway)
        .ok()
        .and_then(|value| Hp::new(value).ok())
        .ok_or(DivergentUniverseBattleError::MissingEnemyStats)?;
    let profile = core
        .enemy_runtime_profile(enemy.id())
        .ok_or(DivergentUniverseBattleError::MissingEnemyStats)?;
    let combatant = ResolvedCombatantSpec::new(
        enemy.unit(),
        level,
        maximum_hp,
        stats.speed(),
        ResolvedDefinitionBindings::new(enemy.abilities().to_vec(), Vec::new(), Vec::new())
            .map_err(|_| DivergentUniverseBattleError::InvalidBattleSpec)?,
        CombatantSpecDigest::new(digest(&[
            b"starclock.divergent-universe.vertical-slice.enemy.v1",
            stable_key.as_bytes(),
            &level.get().to_le_bytes(),
            &wave.to_le_bytes(),
            &slot.to_le_bytes(),
            &ENEMY_PROXY_STAT_SCALE.to_le_bytes(),
        ]))
        .expect("SHA-256 enemy digest is non-zero"),
    )
    .map_err(|_| DivergentUniverseBattleError::InvalidBattleSpec)?
    .with_base_attack_defense(
        stats
            .attack()
            .checked_scale(proxy_scale, Rounding::NearestTiesEven)
            .map_err(|_| DivergentUniverseBattleError::MissingEnemyStats)?,
        stats
            .defense()
            .checked_scale(proxy_scale, Rounding::NearestTiesEven)
            .map_err(|_| DivergentUniverseBattleError::MissingEnemyStats)?,
    )
    .with_base_effect_stats(stats.effect_hit_rate(), stats.effect_resistance())
    .with_energy(Energy::ZERO, Energy::ZERO)
    .map_err(|_| DivergentUniverseBattleError::InvalidBattleSpec)?
    .with_toughness(
        profile.rank(),
        profile.weaknesses().to_vec(),
        profile.toughness_layers().to_vec(),
    )
    .map_err(|_| DivergentUniverseBattleError::InvalidBattleSpec)?;
    let formation = slot
        .checked_sub(1)
        .and_then(|value| u8::try_from(value).ok())
        .and_then(FormationIndex::new)
        .ok_or(DivergentUniverseBattleError::InvalidBattleSpec)?;
    ParticipantSpec::new(
        TeamSide::Enemy,
        formation,
        ParticipantSource::EncounterEnemy(enemy.id()),
        combatant,
    )
    .with_wave(wave)
    .ok_or(DivergentUniverseBattleError::InvalidBattleSpec)
}

fn battle_spec(
    assembly: AssemblyDigest,
    encounter: EncounterId,
    participants: Vec<ParticipantSpec>,
    install_titan_stage_ability: bool,
) -> Result<BattleSpec, DivergentUniverseBattleError> {
    let player_resources =
        TeamResourceSpec::new(3, 5).expect("ordinary player resources are valid");
    let player_resources = if install_titan_stage_ability {
        player_resources
            .with_keyed(vec![
                KeyedTeamResourceSpec::new(
                    SourceDefinitionId::new(TITAN_STAGE_ABILITY_RESOURCE_RAW)
                        .expect("reserved stage-ability resource id is non-zero"),
                    1,
                    1,
                    TeamResourceWavePolicy::Persist,
                )
                .expect("stage-ability resource bounds are valid")
                .with_stable_key(TITAN_STAGE_ABILITY_KEY)
                .expect("stage-ability stable key is valid"),
            ])
            .expect("single stage-ability resource is unique and valid")
    } else {
        player_resources
    };
    BattleSpec::new(
        assembly,
        encounter,
        participants,
        player_resources,
        TeamResourceSpec::new(0, 0).expect("empty enemy resources are valid"),
        ConcedePolicy::Allowed,
    )
    .map_err(|_| DivergentUniverseBattleError::InvalidBattleSpec)
}

fn settlement_contract(
    participants: &starclock_activity::ParticipantLock,
) -> Result<Arc<ActivityBattleResultContract>, DivergentUniverseBattleError> {
    let mut fields = vec![
        ProjectionField::Outcome,
        ProjectionField::FinalStateHash,
        ProjectionField::EventDigest,
        ProjectionField::TerminalFault,
    ];
    fields.extend(
        participants
            .entries()
            .iter()
            .map(|entry| ProjectionField::ParticipantState(entry.participant())),
    );
    let projection = starclock_activity::BattleResultProjection::new(
        ProjectionId::new(PROJECTION_ID_RAW).expect("reserved projection ID is non-zero"),
        fields,
    )
    .map_err(|_| DivergentUniverseBattleError::InvalidSettlementContract)?;
    let carry = participants
        .entries()
        .iter()
        .map(|entry| {
            ActivityParticipantCarryDefinition::new(
                entry.participant(),
                HpCarryPolicy::CarryClamped,
                EnergyCarryPolicy::CarryClamped,
                LifeCarryPolicy::DefeatOnZero,
                PresenceCarryPolicy::DepartIfDefeated,
            )
        })
        .collect();
    ActivityBattleResultContract::new(Arc::new(projection), carry, Vec::new())
        .map(Arc::new)
        .map_err(|_| DivergentUniverseBattleError::InvalidSettlementContract)
}

fn validate_frozen_party(
    core: &SimulationCatalog,
    mapping: &super::mapping::DivergentUniverseMappingSnapshot,
) -> Result<(), DivergentUniverseBattleError> {
    const AVATARS: [u32; 4] = [1308, 1005, 1009, 1105];
    let mapped = mapping
        .participants()
        .iter()
        .map(|value| value.compiled().combatant().form())
        .collect::<Vec<_>>();
    if AVATARS.iter().any(|avatar| {
        core.character_form_for_source_avatar(*avatar)
            .is_none_or(|form| !mapped.contains(&form))
    }) {
        return Err(DivergentUniverseBattleError::PartyMismatch);
    }
    Ok(())
}

pub(super) fn parse_level(value: &str) -> Result<UnitLevel, DivergentUniverseBattleError> {
    value
        .parse::<u8>()
        .ok()
        .and_then(UnitLevel::new)
        .ok_or(DivergentUniverseBattleError::EncounterJoin)
}

fn assembly_digest(
    flow: &DivergentUniverseFlowInstance,
    mapping: [u8; 32],
    contribution: [u8; 32],
    contributed: bool,
) -> AssemblyDigest {
    AssemblyDigest::new(digest(&[
        b"starclock.divergent-universe.vertical-slice.battle-assembly.v1",
        &flow.component_digest,
        &flow.definition.identity().definition_digest().bytes(),
        &mapping,
        &contribution,
        &[u8::from(contributed)],
    ]))
    .expect("SHA-256 assembly digest is non-zero")
}

pub(super) fn digest(parts: &[&[u8]]) -> [u8; 32] {
    let mut hash = CanonicalDigestBuilder::new();
    for part in parts {
        hash.update(
            u64::try_from(part.len())
                .expect("slice length fits u64")
                .to_le_bytes(),
        );
        hash.update(part);
    }
    hash.finalize()
}

#[derive(Debug)]
pub enum DivergentUniverseBattleError {
    DefinitionMismatch,
    MappingRequired,
    PartyMismatch,
    EncounterJoin,
    MissingEnemy,
    MissingEnemyStats,
    IdentityCollision,
    InvalidEncounter,
    InvalidBattleSpec,
    InvalidBattleBinding,
    InvalidSettlementContract,
    InvalidScope,
    EncounterNotOffered,
    ContributionNotAccepted,
    ContributionNotBound,
    StaleState,
    PreparationRejected,
    BattleStartRejected,
    Execution(NestedBattleExecutionError),
    MissingExecutionReport,
    Settlement(DivergentUniverseBattleSettlementError),
    SettlementRejected,
}

impl std::fmt::Display for DivergentUniverseBattleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "invalid Divergent Universe vertical-slice battle: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseBattleError {}
