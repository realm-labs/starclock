//! One real immutable Gold and Gears `BattleSpec` from a current Activity snapshot.

use std::sync::Arc;

use starclock_activity::{ActivityTransactionState, ParticipantLockDigest};
use starclock_combat::{
    AssemblyDigest, Battle, BattleSeed, BattleSpec, CombatantSpecDigest, ConcedePolicy,
    EncounterId, EncounterWaveId, Energy, FormationIndex, Hp, KeyedTeamResourceSpec,
    ModifierDefinitionId, ModifierStackingGroupId, ParticipantSource, ParticipantSpec,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, ResolvedModifierBinding, Rounding, Scalar,
    SourceDefinitionId, Speed, StatValue, TeamResourceSpec, TeamResourceWavePolicy, TeamSide,
    UnitLevel,
    catalog::{
        CombatCatalog,
        builder::CombatCatalogBuilder,
        definition::EncounterDefinition,
        encounter::{EncounterWaveDefinition, WaveCarry, WaveSlotDefinition, WaveTransitionPolicy},
    },
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition, ModifierFilter,
        ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{RuleSource, RuleValue, SourceClass, ValueExpr},
};

use crate::{
    battle_materialization::{UniverseBattleRoster, player_participants},
    battle_rule_lowering::{RESONANCE_RESOURCE_ID, RESONANCE_RESOURCE_KEY},
    digest::Encoder,
};

use super::{
    GoldAndGearsBattleAssemblyContext, GoldAndGearsBattleContributionSnapshot,
    GoldAndGearsEncounterSelection, GoldAndGearsEnemyDefinitionBinding, GoldAndGearsEntryError,
    GoldAndGearsNeuralBattleStat, GoldAndGearsRuntimeInstance,
    battle_snapshot::CompiledGoldBattleSnapshot,
    conundrum_stats_modifier::GoldAndGearsStatsConundrumActivation,
};

const ENCOUNTER_ID: EncounterId =
    EncounterId::new(0x7f50_0001).expect("reserved encounter ID is non-zero");
const WAVE_ID_BASE: u32 = 0x7f51_0000;
const NEURAL_MODIFIER_BASE: u32 = 0x7f60_0000;
const NEURAL_GROUP_BASE: u32 = 0x7f61_0000;
const NEURAL_SOURCE_BASE: u32 = 0x7f62_0000;
const ENEMY_STAT_DIFFICULTY_KEY: &str = "standard-universe-v1";

#[derive(Clone, Debug)]
pub struct GoldAndGearsBattleMaterialization {
    combat_catalog: Arc<CombatCatalog>,
    battle_spec: BattleSpec,
    contributions: GoldAndGearsBattleContributionSnapshot,
    participant_lock: ParticipantLockDigest,
    enemy_definitions: Box<[GoldAndGearsEnemyDefinitionBinding]>,
    enemy_definition_digest: [u8; 32],
    enemy_definition_count: u16,
    mode_owned_enemy_definition_count: u16,
    reviewed_stat_source_count: u16,
    fallback_stat_source_count: u16,
    digest: [u8; 32],
}

impl GoldAndGearsBattleMaterialization {
    pub const fn combat_catalog(&self) -> &Arc<CombatCatalog> {
        &self.combat_catalog
    }
    pub const fn battle_spec(&self) -> &BattleSpec {
        &self.battle_spec
    }
    pub const fn contributions(&self) -> &GoldAndGearsBattleContributionSnapshot {
        &self.contributions
    }
    pub const fn participant_lock(&self) -> ParticipantLockDigest {
        self.participant_lock
    }
    pub fn enemy_definitions(&self) -> &[GoldAndGearsEnemyDefinitionBinding] {
        &self.enemy_definitions
    }
    pub const fn enemy_definition_digest(&self) -> [u8; 32] {
        self.enemy_definition_digest
    }
    pub const fn enemy_definition_count(&self) -> u16 {
        self.enemy_definition_count
    }
    pub const fn mode_owned_enemy_definition_count(&self) -> u16 {
        self.mode_owned_enemy_definition_count
    }
    pub const fn reviewed_stat_source_count(&self) -> u16 {
        self.reviewed_stat_source_count
    }
    pub const fn fallback_stat_source_count(&self) -> u16 {
        self.fallback_stat_source_count
    }
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

#[derive(Clone)]
struct ModifierAttachment {
    group: ModifierStackingGroup,
    definition: ModifierDefinition,
    source: RuleSource,
}

impl GoldAndGearsRuntimeInstance {
    /// Consumes one immutable view of current inventories and mode state and
    /// emits a construction-validated battle request. It performs no RNG draw
    /// and never mutates the supplied Activity state or roster.
    pub fn materialize_current_battle(
        &self,
        state: &ActivityTransactionState,
        selection: &GoldAndGearsEncounterSelection,
        roster: &UniverseBattleRoster,
        context: &GoldAndGearsBattleAssemblyContext,
    ) -> Result<GoldAndGearsBattleMaterialization, GoldAndGearsEntryError> {
        if state.current_battle_attempt_is_settled()
            || roster.participant_lock() != self.participants().digest()
            || self.encounter_role_for_node(state, state.current_node()) != Some(selection.role())
            || selection.waves().is_empty()
        {
            return Err(GoldAndGearsEntryError::InvalidBattleMaterialization);
        }
        let snapshot = self.compile_battle_snapshot(state, context)?;
        let neural = neural_modifiers(self.neural_battle_stat_contributions())?;
        let path = snapshot
            .path_boost
            .binding()
            .definitions()
            .iter()
            .zip(snapshot.path_boost.binding().groups())
            .map(|(definition, group)| ModifierAttachment {
                group: group.clone(),
                definition: definition.clone(),
                source: snapshot.path_boost.binding().source().clone(),
            })
            .collect::<Vec<_>>();
        let conundrum = snapshot
            .conundrum
            .bindings()
            .iter()
            .map(|binding| ModifierAttachment {
                group: binding.group().clone(),
                definition: binding.definition().clone(),
                source: binding.source().clone(),
            })
            .collect::<Vec<_>>();
        let enemy_conundrum = snapshot
            .conundrum
            .bindings()
            .iter()
            .filter(|binding| {
                binding.activation() == GoldAndGearsStatsConundrumActivation::EveryEnemy
            })
            .map(|binding| ModifierAttachment {
                group: binding.group().clone(),
                definition: binding.definition().clone(),
                source: binding.source().clone(),
            })
            .collect::<Vec<_>>();
        let digest = materialization_digest(self, selection, roster, &snapshot, &neural);
        let mut builder = CombatCatalogBuilder::from_catalog(self.battle_catalog.combat(), digest);
        add_shared_contributions(&mut builder, &snapshot);
        for attachment in path.iter().chain(&neural).chain(&conundrum) {
            builder.add_modifier_group(attachment.group.clone());
            builder.add_modifier(attachment.definition.clone());
        }
        if let Some(effect) = snapshot.conundrum.source_stack_effect() {
            builder.add_effect(effect);
        }
        builder.add_encounter(encounter_definition(self, selection)?);
        let combat_catalog = builder.build().map_err(|error| {
            GoldAndGearsEntryError::InvalidBattleCatalog(error.to_string().into())
        })?;

        let mut players = player_participants(
            &self.content_runtime.standard,
            roster,
            &snapshot.shared,
            None,
            &[],
        )
        .map_err(|_| GoldAndGearsEntryError::InvalidPlayerBattleParticipants)?;
        let player_modifiers = path.iter().chain(&neural).cloned().collect::<Vec<_>>();
        players = players
            .iter()
            .map(|participant| attach_modifiers(participant, &player_modifiers, digest))
            .collect::<Result<Vec<_>, _>>()?;
        let mut participants = players;
        participants.extend(
            enemy_participants(self, selection, &combat_catalog, &enemy_conundrum, digest)
                .map_err(|_| GoldAndGearsEntryError::InvalidEnemyBattleParticipants)?,
        );
        let battle_spec = BattleSpec::new(
            AssemblyDigest::new(digest).expect("SHA-256 digest is non-zero"),
            ENCOUNTER_ID,
            participants,
            player_resources(&snapshot)?,
            TeamResourceSpec::new(0, 0).expect("empty enemy resources are valid"),
            ConcedePolicy::Allowed,
        )
        .map_err(|_| GoldAndGearsEntryError::InvalidBattleSpec)?;
        Battle::create(
            Arc::clone(&combat_catalog),
            battle_spec.clone(),
            BattleSeed::new([0x47; 32]),
        )
        .map_err(|_| GoldAndGearsEntryError::InvalidBattleConstruction)?;
        let level = u8::try_from(selection.effective_level())
            .ok()
            .and_then(UnitLevel::new)
            .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?;
        let reviewed_stat_source_count = self
            .battle_catalog
            .enemies()
            .iter()
            .filter(|binding| {
                self.content_runtime
                    .standard
                    .simulation_catalog()
                    .enemy_runtime_stat(binding.stat_source(), level, ENEMY_STAT_DIFFICULTY_KEY)
                    .is_some()
            })
            .count();
        Ok(GoldAndGearsBattleMaterialization {
            combat_catalog,
            battle_spec,
            contributions: snapshot.summary,
            participant_lock: roster.participant_lock(),
            enemy_definitions: self.battle_catalog.enemies().into(),
            enemy_definition_digest: self.battle_catalog.digest(),
            enemy_definition_count: u16::try_from(self.battle_catalog.enemies().len())
                .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?,
            mode_owned_enemy_definition_count: u16::try_from(
                self.battle_catalog
                    .enemies()
                    .iter()
                    .filter(|binding| binding.mode_owned())
                    .count(),
            )
            .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?,
            reviewed_stat_source_count: u16::try_from(reviewed_stat_source_count)
                .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?,
            fallback_stat_source_count: u16::try_from(
                self.battle_catalog.enemies().len() - reviewed_stat_source_count,
            )
            .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?,
            digest,
        })
    }
}

fn add_shared_contributions(
    builder: &mut CombatCatalogBuilder,
    snapshot: &CompiledGoldBattleSnapshot,
) {
    for modifier in snapshot.shared.modifiers() {
        builder.add_modifier_group(modifier.group().clone());
        builder.add_modifier(modifier.definition().clone());
    }
    for executable in snapshot.shared.executable_rules() {
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
    if let Some(resonance) = snapshot.shared.resonance() {
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
}

fn encounter_definition(
    instance: &GoldAndGearsRuntimeInstance,
    selection: &GoldAndGearsEncounterSelection,
) -> Result<EncounterDefinition, GoldAndGearsEntryError> {
    let waves = selection
        .waves()
        .iter()
        .enumerate()
        .map(|(wave_index, wave)| {
            let slots =
                wave.slots()
                    .iter()
                    .enumerate()
                    .map(|(slot_index, slot)| {
                        let binding = instance
                            .battle_catalog
                            .enemy(slot.enemy())
                            .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?;
                        let initial_phase = instance
                            .battle_catalog
                            .combat()
                            .enemy(binding.combat_enemy())
                            .and_then(|definition| definition.phases().first())
                            .map(starclock_combat::catalog::encounter::EnemyPhaseDefinition::id);
                        WaveSlotDefinition::new(
                            u16::try_from(slot_index + 1).map_err(|_| {
                                GoldAndGearsEntryError::InvalidBattleMaterialization
                            })?,
                            FormationIndex::new(u8::try_from(slot_index).map_err(|_| {
                                GoldAndGearsEntryError::InvalidBattleMaterialization
                            })?)
                            .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?,
                            binding.combat_enemy(),
                            Some(u8::try_from(selection.effective_level()).map_err(|_| {
                                GoldAndGearsEntryError::InvalidBattleMaterialization
                            })?),
                            initial_phase,
                            true,
                        )
                        .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
            EncounterWaveDefinition::new(
                EncounterWaveId::new(
                    WAVE_ID_BASE
                        + u32::try_from(wave_index + 1)
                            .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?,
                )
                .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?,
                u16::try_from(wave_index + 1)
                    .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?,
                None,
                None,
                WaveCarry::CARRY_ALL,
                slots,
            )
            .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)
        })
        .collect::<Result<Vec<_>, _>>()?;
    EncounterDefinition::new(ENCOUNTER_ID, Vec::new(), Vec::new())
        .with_authored_waves(WaveTransitionPolicy::AfterAction, waves)
        .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)
}

fn enemy_participants(
    instance: &GoldAndGearsRuntimeInstance,
    selection: &GoldAndGearsEncounterSelection,
    catalog: &CombatCatalog,
    conundrum: &[ModifierAttachment],
    root_digest: [u8; 32],
) -> Result<Vec<ParticipantSpec>, GoldAndGearsEntryError> {
    let level = u8::try_from(selection.effective_level())
        .ok()
        .and_then(UnitLevel::new)
        .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?;
    let mut output = Vec::new();
    for (wave_index, wave) in selection.waves().iter().enumerate() {
        for (slot_index, slot) in wave.slots().iter().enumerate() {
            let binding = instance
                .battle_catalog
                .enemy(slot.enemy())
                .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?;
            let definition = catalog
                .enemy(binding.combat_enemy())
                .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?;
            let reviewed_stats = instance
                .content_runtime
                .standard
                .simulation_catalog()
                .enemy_runtime_stat(binding.stat_source(), level, ENEMY_STAT_DIFFICULTY_KEY);
            let (hp, speed, attack, defense, effect_hit_rate, effect_resistance) = reviewed_stats
                .map_or_else(
                || {
                    Ok((
                        Hp::new(1).expect("fallback HP is positive"),
                        Speed::from_scaled(50_000_000).expect("fallback Speed is valid"),
                        StatValue::from_scaled(0).expect("fallback ATK is valid"),
                        StatValue::from_scaled(0).expect("fallback DEF is valid"),
                        Scalar::ZERO,
                        Scalar::ZERO,
                    ))
                },
                |stats| {
                    let hp = stats
                        .hp()
                        .rounded_integer(Rounding::NearestTiesAway)
                        .ok()
                        .and_then(|value| Hp::new(value).ok())
                        .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?;
                    Ok((
                        hp,
                        stats.speed(),
                        stats.attack(),
                        stats.defense(),
                        stats.effect_hit_rate(),
                        stats.effect_resistance(),
                    ))
                },
            )?;
            let digest = combatant_digest(root_digest, slot.enemy(), wave_index, slot_index);
            let combatant = ResolvedCombatantSpec::new(
                definition.unit(),
                level,
                hp,
                speed,
                ResolvedDefinitionBindings::new(
                    definition.abilities().to_vec(),
                    Vec::new(),
                    Vec::new(),
                )
                .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?,
                CombatantSpecDigest::new(digest).expect("SHA-256 digest is non-zero"),
            )
            .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?
            .with_base_attack_defense(attack, defense)
            .with_base_effect_stats(effect_hit_rate, effect_resistance)
            .with_energy(Energy::ZERO, Energy::ZERO)
            .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?;
            let combatant = if reviewed_stats.is_some() {
                let profile = instance
                    .content_runtime
                    .standard
                    .simulation_catalog()
                    .enemy_runtime_profile(binding.stat_source())
                    .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?;
                combatant
                    .with_toughness(
                        profile.rank(),
                        profile.weaknesses().to_vec(),
                        profile.toughness_layers().to_vec(),
                    )
                    .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?
            } else {
                combatant
            };
            let base = ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(
                    u8::try_from(slot_index)
                        .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?,
                )
                .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?,
                ParticipantSource::EncounterEnemy(binding.combat_enemy()),
                combatant,
            )
            .with_wave(
                u16::try_from(wave_index + 1)
                    .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?,
            )
            .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?;
            output.push(attach_modifiers(&base, conundrum, root_digest)?);
        }
    }
    Ok(output)
}

fn attach_modifiers(
    participant: &ParticipantSpec,
    attachments: &[ModifierAttachment],
    root_digest: [u8; 32],
) -> Result<ParticipantSpec, GoldAndGearsEntryError> {
    if attachments.is_empty() {
        return Ok(participant.clone());
    }
    let base = participant.combatant();
    let mut modifiers = base.modifiers().to_vec();
    modifiers.extend(
        attachments
            .iter()
            .map(|attachment| attachment.definition.id),
    );
    modifiers.sort_unstable();
    modifiers.dedup();
    let mut sources = base.sources().to_vec();
    sources.extend(
        attachments
            .iter()
            .map(|attachment| attachment.source.clone()),
    );
    sources.sort_unstable_by_key(|source| source.definition());
    sources.dedup_by_key(|source| source.definition());
    let mut bindings = base.modifier_bindings().to_vec();
    bindings.extend(attachments.iter().map(|attachment| {
        ResolvedModifierBinding::new(attachment.definition.id, attachment.source.definition())
    }));
    bindings.sort_unstable_by_key(|binding| binding.definition());
    bindings.dedup_by_key(|binding| binding.definition());
    let mut encoder = Encoder::new(b"starclock.gold-and-gears.attached-combatant.v1");
    encoder.digest(root_digest);
    encoder.digest(base.digest().bytes());
    for binding in &bindings {
        encoder.u32(binding.definition().get());
    }
    let mut combatant = ResolvedCombatantSpec::new(
        base.form(),
        base.level(),
        base.maximum_hp(),
        base.speed(),
        ResolvedDefinitionBindings::new(
            base.abilities().to_vec(),
            base.rule_bundles().to_vec(),
            modifiers,
        )
        .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?,
        CombatantSpecDigest::new(encoder.finish()).expect("SHA-256 digest is non-zero"),
    )
    .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?
    .with_base_attack_defense(base.base_attack(), base.base_defense())
    .with_base_effect_stats(base.base_effect_hit_rate(), base.base_effect_resistance())
    .with_energy(base.current_energy(), base.maximum_energy())
    .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?
    .with_toughness(
        base.rank(),
        base.weaknesses().to_vec(),
        base.toughness_layers().to_vec(),
    )
    .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?;
    combatant = combatant
        .with_sources(sources)
        .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?;
    combatant = combatant
        .with_modifier_bindings(bindings)
        .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?;
    let mut output = ParticipantSpec::new(
        participant.side(),
        participant.formation(),
        participant.source(),
        combatant,
    )
    .with_wave(participant.wave())
    .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?
    .with_locked_combatant_digest(participant.locked_combatant_digest());
    if let Some(initial) = participant.initial_state() {
        output = output
            .with_initial_state(initial)
            .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?;
    }
    Ok(output)
}

fn neural_modifiers(
    contributions: &[super::GoldAndGearsNeuralStatContribution],
) -> Result<Vec<ModifierAttachment>, GoldAndGearsEntryError> {
    let mut output = Vec::new();
    for (source_index, contribution) in contributions.iter().enumerate() {
        let specs = neural_specs(contribution.stat());
        let source_raw = u32::try_from(source_index + 1)
            .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?;
        let source = RuleSource::new(
            SourceDefinitionId::new(NEURAL_SOURCE_BASE + source_raw)
                .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?,
            SourceClass::Mode,
            vec![],
            neural_source_digest(contribution),
        );
        for (spec_index, (stat, stage, purpose, filters)) in specs.into_iter().enumerate() {
            let ordinal = source_raw
                .checked_mul(16)
                .and_then(|value| value.checked_add(u32::try_from(spec_index + 1).ok()?))
                .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?;
            let group = ModifierStackingGroup {
                id: ModifierStackingGroupId::new(NEURAL_GROUP_BASE + ordinal)
                    .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?,
                aggregation: ModifierAggregation::UniquePerSource,
                comparator: None,
            };
            let definition = ModifierDefinition {
                id: ModifierDefinitionId::new(NEURAL_MODIFIER_BASE + ordinal)
                    .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?,
                stat,
                stage,
                purpose,
                value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(
                    contribution.ratio_scaled(),
                ))),
                stacking_group: group.id,
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: stage,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: None,
                filters,
            };
            output.push(ModifierAttachment {
                group,
                definition,
                source: source.clone(),
            });
        }
    }
    Ok(output)
}

type NeuralSpec = (
    StatKind,
    FormulaStage,
    FormulaPurpose,
    Box<[ModifierFilter]>,
);

fn neural_specs(stat: GoldAndGearsNeuralBattleStat) -> Vec<NeuralSpec> {
    let one = |stat, stage| {
        vec![(
            stat,
            stage,
            FormulaPurpose::Stat,
            Vec::<ModifierFilter>::new().into_boxed_slice(),
        )]
    };
    match stat {
        GoldAndGearsNeuralBattleStat::PartyAttackRatio => {
            one(StatKind::Atk, FormulaStage::PercentOfBase)
        }
        GoldAndGearsNeuralBattleStat::PartyMaximumHpRatio => {
            one(StatKind::Hp, FormulaStage::PercentOfBase)
        }
        GoldAndGearsNeuralBattleStat::PartyDefenseRatio => {
            one(StatKind::Def, FormulaStage::PercentOfBase)
        }
        GoldAndGearsNeuralBattleStat::PartySpeedRatio => {
            one(StatKind::Spd, FormulaStage::PercentOfBase)
        }
        GoldAndGearsNeuralBattleStat::PartyEffectHitRateRatio => {
            one(StatKind::EffectHitRate, FormulaStage::Flat)
        }
        GoldAndGearsNeuralBattleStat::PartyEffectResistanceRatio => {
            one(StatKind::EffectResistance, FormulaStage::Flat)
        }
        GoldAndGearsNeuralBattleStat::PartyCriticalRateRatio => {
            one(StatKind::CritRate, FormulaStage::Flat)
        }
        GoldAndGearsNeuralBattleStat::PartyCriticalDamageRatio => {
            one(StatKind::CritDamage, FormulaStage::Flat)
        }
        GoldAndGearsNeuralBattleStat::PartyDamageTakenReductionRatio => {
            damage_specs(FormulaStage::Mitigation)
        }
        GoldAndGearsNeuralBattleStat::PartyDamageDealtRatio
        | GoldAndGearsNeuralBattleStat::PathResonanceDamageRatio => {
            damage_specs(FormulaStage::DamageBoost)
        }
    }
}

fn damage_specs(stage: FormulaStage) -> Vec<NeuralSpec> {
    [
        FormulaPurpose::OrdinaryDamage,
        FormulaPurpose::Dot,
        FormulaPurpose::Break,
        FormulaPurpose::SuperBreak,
        FormulaPurpose::AdditionalDamage,
        FormulaPurpose::JointDamage,
        FormulaPurpose::ElationDamage,
        FormulaPurpose::TrueDamage,
    ]
    .into_iter()
    .map(|purpose| {
        (
            StatKind::Atk,
            stage,
            purpose,
            Vec::<ModifierFilter>::new().into_boxed_slice(),
        )
    })
    .collect()
}

fn player_resources(
    snapshot: &CompiledGoldBattleSnapshot,
) -> Result<TeamResourceSpec, GoldAndGearsEntryError> {
    let resources = TeamResourceSpec::new(3, 5).expect("player resources are valid");
    let Some(resonance) = snapshot.shared.resonance() else {
        return Ok(resources);
    };
    let keyed = KeyedTeamResourceSpec::new(
        RESONANCE_RESOURCE_ID,
        resonance.initial_energy(),
        resonance.maximum_energy(),
        TeamResourceWavePolicy::Persist,
    )
    .and_then(|resource| resource.with_stable_key(RESONANCE_RESOURCE_KEY))
    .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?;
    resources
        .with_keyed(vec![keyed])
        .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)
}

fn materialization_digest(
    instance: &GoldAndGearsRuntimeInstance,
    selection: &GoldAndGearsEncounterSelection,
    roster: &UniverseBattleRoster,
    snapshot: &CompiledGoldBattleSnapshot,
    neural: &[ModifierAttachment],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.gold-and-gears.battle-materialization.v1");
    encoder.digest(instance.battle_catalog.digest());
    encoder.digest(roster.participant_lock().bytes());
    encoder.text(selection.group());
    encoder.u32(selection.source_group_id());
    encoder.text(selection.source_stage_id());
    encoder.u32(u32::from(selection.effective_level()));
    encoder.u32(selection.waves().len() as u32);
    for wave in selection.waves() {
        encoder.text(wave.key());
        encoder.text(wave.source_stage_id());
        encoder.u32(u32::from(wave.wave_index()));
        encoder.text(wave.stage_type());
        encoder.u32(u32::from(wave.authored_stage_level()));
        encoder.u32(u32::from(wave.hard_level_group()));
        encoder.u32(wave.stage_ability_ids().len() as u32);
        for ability in wave.stage_ability_ids() {
            encoder.text(ability);
        }
        encoder.u32(wave.slots().len() as u32);
        for slot in wave.slots() {
            encoder.text(slot.key());
            encoder.text(slot.source_slot());
            encoder.text(slot.source_monster_id());
            encoder.text(slot.enemy());
            encoder.u32(slot.boss_choices().len() as u32);
            for choice in slot.boss_choices() {
                encoder.text(choice);
            }
        }
    }
    encoder.digest(snapshot.summary.digest());
    encoder.u32(neural.len() as u32);
    for attachment in neural {
        encoder.u32(attachment.definition.id.get());
    }
    encoder.finish()
}

fn combatant_digest(root: [u8; 32], key: &str, wave: usize, slot: usize) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.gold-and-gears.enemy-combatant.v1");
    encoder.digest(root);
    encoder.text(key);
    encoder.u32(wave as u32);
    encoder.u32(slot as u32);
    encoder.finish()
}

fn neural_source_digest(contribution: &super::GoldAndGearsNeuralStatContribution) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.gold-and-gears.neural-battle-source.v1");
    encoder.text(contribution.source_node());
    encoder.u8(contribution.stat() as u8);
    encoder.i64(contribution.ratio_scaled());
    encoder.finish()
}
