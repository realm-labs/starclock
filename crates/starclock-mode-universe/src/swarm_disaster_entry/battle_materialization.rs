//! One real immutable Swarm Disaster `BattleSpec` from current Activity state.

use std::sync::Arc;

use starclock_activity::{ActivityRngStreams, ActivityTransactionState};
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
    error::UniverseCatalogLoadError,
};

use super::{
    SwarmDisasterRuntimeInstance,
    battle_snapshot::CompiledSwarmBattleSnapshot,
    encounter_runtime::{EncounterSelection, selection_digest},
    validate::{error as invalid, reference},
};

const ENCOUNTER_ID: EncounterId =
    EncounterId::new(0x7f90_0001).expect("reserved encounter ID is non-zero");
const WAVE_ID_BASE: u32 = 0x7f91_0000;
const DISARRAY_MODIFIER_BASE: u32 = 0x7f92_0000;
const DISARRAY_GROUP_BASE: u32 = 0x7f93_0000;
const DISARRAY_SOURCE_ID: u32 = 0x7f94_0001;
const ENEMY_STAT_DIFFICULTY_KEY: &str = "standard-universe-v1";

#[derive(Clone)]
struct ModifierAttachment {
    group: ModifierStackingGroup,
    definition: ModifierDefinition,
    source: RuleSource,
}

pub(super) struct SwarmBattleMaterialization {
    pub(super) battle_spec: BattleSpec,
    pub(super) combat_catalog: Arc<CombatCatalog>,
    pub(super) selection: EncounterSelection,
    pub(super) snapshot_digest: [u8; 32],
}

impl SwarmDisasterRuntimeInstance {
    /// Selects the current encounter from the labeled Encounter stream and
    /// materializes a construction-validated immutable battle request.
    /// Activity state and roster are read-only; any later assembly failure
    /// restores the encounter RNG transaction.
    pub fn materialize_current_battle(
        &self,
        state: &ActivityTransactionState,
        rng: &mut ActivityRngStreams,
        roster: &UniverseBattleRoster,
    ) -> Result<BattleSpec, UniverseCatalogLoadError> {
        self.resolve_current_battle(state, rng, roster)
            .map(|materialization| materialization.battle_spec)
    }

    pub(super) fn resolve_current_battle(
        &self,
        state: &ActivityTransactionState,
        rng: &mut ActivityRngStreams,
        roster: &UniverseBattleRoster,
    ) -> Result<SwarmBattleMaterialization, UniverseCatalogLoadError> {
        if state.current_battle_attempt_is_settled()
            || roster.participant_lock() != self.participants().digest()
        {
            return Err(reference(
                "invalid Swarm battle participant or attempt lock",
            ));
        }
        rng.transact(|working| {
            let selection = self.select_current_encounter(state, working)?;
            let snapshot = self.compile_battle_snapshot(state, &selection)?;
            let disarray = disarray_modifiers(snapshot.disarray)?;
            let digest = materialization_digest(self, roster, &selection, &snapshot);
            let mut builder =
                CombatCatalogBuilder::from_catalog(self.battle_catalog.combat(), digest);
            add_shared_contributions(&mut builder, &snapshot);
            for attachment in &disarray {
                builder.add_modifier_group(attachment.group.clone());
                builder.add_modifier(attachment.definition.clone());
            }
            builder.add_encounter(encounter_definition(self, &selection)?);
            let combat_catalog = builder
                .build()
                .map_err(|_| invalid("invalid Swarm materialized combat catalog"))?;

            let mut participants = player_participants(
                &self.content_runtime.standard,
                roster,
                &snapshot.shared,
                None,
                &[],
            )
            .map_err(|_| reference("invalid Swarm player battle participants"))?;
            participants.extend(enemy_participants(
                self,
                &selection,
                &combat_catalog,
                &disarray,
                digest,
            )?);
            let battle_spec = BattleSpec::new(
                AssemblyDigest::new(digest).expect("SHA-256 digest is non-zero"),
                ENCOUNTER_ID,
                participants,
                player_resources(&snapshot)?,
                TeamResourceSpec::new(0, 0).expect("empty enemy resources are valid"),
                ConcedePolicy::Allowed,
            )
            .map_err(|_| invalid("invalid Swarm BattleSpec"))?;
            Battle::create(
                Arc::clone(&combat_catalog),
                battle_spec.clone(),
                BattleSeed::new([0x53; 32]),
            )
            .map_err(|_| invalid("Swarm BattleSpec failed construction validation"))?;
            Ok(SwarmBattleMaterialization {
                battle_spec,
                combat_catalog,
                selection,
                snapshot_digest: snapshot.digest,
            })
        })
    }
}

fn add_shared_contributions(
    builder: &mut CombatCatalogBuilder,
    snapshot: &CompiledSwarmBattleSnapshot,
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
    instance: &SwarmDisasterRuntimeInstance,
    selection: &EncounterSelection,
) -> Result<EncounterDefinition, UniverseCatalogLoadError> {
    let waves = selection
        .waves
        .iter()
        .enumerate()
        .map(|(wave_index, wave)| {
            let slots = wave
                .slots
                .iter()
                .enumerate()
                .map(|(slot_index, slot)| {
                    let binding = instance
                        .battle_catalog
                        .enemy(&slot.enemy_variant)
                        .ok_or_else(|| reference("unknown Swarm encounter enemy"))?;
                    let initial_phase = instance
                        .battle_catalog
                        .combat()
                        .enemy(binding.combat_enemy())
                        .and_then(|definition| definition.phases().first())
                        .map(starclock_combat::catalog::encounter::EnemyPhaseDefinition::id);
                    WaveSlotDefinition::new(
                        u16::try_from(slot_index + 1)
                            .map_err(|_| invalid("Swarm wave slot overflow"))?,
                        FormationIndex::new(
                            slot.formation_index
                                .checked_sub(1)
                                .ok_or_else(|| invalid("invalid Swarm formation index"))?,
                        )
                        .ok_or_else(|| invalid("invalid Swarm formation index"))?,
                        binding.combat_enemy(),
                        Some(
                            u8::try_from(selection.effective_level)
                                .map_err(|_| invalid("Swarm encounter level overflow"))?,
                        ),
                        initial_phase,
                        true,
                    )
                    .ok_or_else(|| invalid("invalid Swarm encounter wave slot"))
                })
                .collect::<Result<Vec<_>, UniverseCatalogLoadError>>()?;
            EncounterWaveDefinition::new(
                EncounterWaveId::new(
                    WAVE_ID_BASE
                        + u32::try_from(wave_index + 1)
                            .map_err(|_| invalid("Swarm wave identity overflow"))?,
                )
                .ok_or_else(|| invalid("invalid Swarm wave identity"))?,
                u16::try_from(wave_index + 1)
                    .map_err(|_| invalid("Swarm wave ordinal overflow"))?,
                None,
                None,
                WaveCarry::CARRY_ALL,
                slots,
            )
            .ok_or_else(|| invalid("invalid Swarm encounter wave"))
        })
        .collect::<Result<Vec<_>, UniverseCatalogLoadError>>()?;
    EncounterDefinition::new(ENCOUNTER_ID, Vec::new(), Vec::new())
        .with_authored_waves(WaveTransitionPolicy::AfterAction, waves)
        .ok_or_else(|| invalid("invalid Swarm encounter definition"))
}

fn enemy_participants(
    instance: &SwarmDisasterRuntimeInstance,
    selection: &EncounterSelection,
    catalog: &CombatCatalog,
    disarray: &[ModifierAttachment],
    root_digest: [u8; 32],
) -> Result<Vec<ParticipantSpec>, UniverseCatalogLoadError> {
    let level = u8::try_from(selection.effective_level)
        .ok()
        .and_then(UnitLevel::new)
        .ok_or_else(|| invalid("invalid Swarm enemy level"))?;
    let mut output = Vec::new();
    for (wave_index, wave) in selection.waves.iter().enumerate() {
        for (slot_index, slot) in wave.slots.iter().enumerate() {
            let binding = instance
                .battle_catalog
                .enemy(&slot.enemy_variant)
                .ok_or_else(|| reference("unknown Swarm enemy identity"))?;
            let definition = catalog
                .enemy(binding.combat_enemy())
                .ok_or_else(|| reference("missing Swarm enemy definition"))?;
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
                        .ok_or_else(|| invalid("invalid reviewed Swarm enemy HP"))?;
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
            let digest = combatant_digest(root_digest, &slot.enemy_variant, wave_index, slot_index);
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
                .map_err(|_| invalid("invalid Swarm enemy bindings"))?,
                CombatantSpecDigest::new(digest).expect("SHA-256 digest is non-zero"),
            )
            .map_err(|_| invalid("invalid Swarm enemy combatant"))?
            .with_base_attack_defense(attack, defense)
            .with_base_effect_stats(effect_hit_rate, effect_resistance)
            .with_energy(Energy::ZERO, Energy::ZERO)
            .map_err(|_| invalid("invalid Swarm enemy energy"))?;
            let combatant = if reviewed_stats.is_some() {
                let profile = instance
                    .content_runtime
                    .standard
                    .simulation_catalog()
                    .enemy_runtime_profile(binding.stat_source())
                    .ok_or_else(|| reference("missing reviewed Swarm enemy profile"))?;
                combatant
                    .with_toughness(
                        profile.rank(),
                        profile.weaknesses().to_vec(),
                        profile.toughness_layers().to_vec(),
                    )
                    .map_err(|_| invalid("invalid Swarm enemy toughness"))?
            } else {
                combatant
            };
            let base = ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(
                    slot.formation_index
                        .checked_sub(1)
                        .ok_or_else(|| invalid("invalid Swarm participant formation"))?,
                )
                .ok_or_else(|| invalid("invalid Swarm participant formation"))?,
                ParticipantSource::EncounterEnemy(binding.combat_enemy()),
                combatant,
            )
            .with_wave(
                u16::try_from(wave_index + 1)
                    .map_err(|_| invalid("Swarm participant wave overflow"))?,
            )
            .ok_or_else(|| invalid("invalid Swarm enemy participant"))?;
            output.push(attach_modifiers(&base, disarray, root_digest)?);
        }
    }
    Ok(output)
}

fn disarray_modifiers(
    values: (i64, i64, i64),
) -> Result<Vec<ModifierAttachment>, UniverseCatalogLoadError> {
    let source_digest = {
        let mut encoder = Encoder::new(b"starclock.swarm-disaster.disarray-source");
        encoder.i64(values.0);
        encoder.i64(values.1);
        encoder.i64(values.2);
        encoder.finish()
    };
    let source = RuleSource::new(
        SourceDefinitionId::new(DISARRAY_SOURCE_ID)
            .ok_or_else(|| invalid("invalid Disarray source identity"))?,
        SourceClass::Mode,
        vec![],
        source_digest,
    );
    let mut specs = Vec::new();
    specs.extend(damage_specs(FormulaStage::DamageBoost, values.0));
    specs.extend(damage_specs(FormulaStage::Mitigation, values.1));
    if values.2 != 0 {
        specs.push((
            StatKind::Spd,
            FormulaStage::PercentOfBase,
            FormulaPurpose::Stat,
            values.2,
        ));
    }
    specs
        .into_iter()
        .enumerate()
        .map(|(index, (stat, stage, purpose, percent))| {
            let raw = u32::try_from(index + 1)
                .map_err(|_| invalid("Disarray modifier identity overflow"))?;
            let group = ModifierStackingGroup {
                id: ModifierStackingGroupId::new(DISARRAY_GROUP_BASE + raw)
                    .ok_or_else(|| invalid("invalid Disarray group identity"))?,
                aggregation: ModifierAggregation::UniquePerSource,
                comparator: None,
            };
            let scaled = percent
                .checked_mul(10_000)
                .ok_or_else(|| invalid("Disarray percentage overflow"))?;
            let definition = ModifierDefinition {
                id: ModifierDefinitionId::new(DISARRAY_MODIFIER_BASE + raw)
                    .ok_or_else(|| invalid("invalid Disarray modifier identity"))?,
                stat,
                stage,
                purpose,
                value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(scaled))),
                stacking_group: group.id,
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: stage,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: None,
                filters: Vec::<ModifierFilter>::new().into_boxed_slice(),
            };
            Ok(ModifierAttachment {
                group,
                definition,
                source: source.clone(),
            })
        })
        .collect()
}

fn damage_specs(
    stage: FormulaStage,
    percent: i64,
) -> Vec<(StatKind, FormulaStage, FormulaPurpose, i64)> {
    if percent == 0 {
        return Vec::new();
    }
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
    .map(|purpose| (StatKind::Atk, stage, purpose, percent))
    .collect()
}

fn attach_modifiers(
    participant: &ParticipantSpec,
    attachments: &[ModifierAttachment],
    root_digest: [u8; 32],
) -> Result<ParticipantSpec, UniverseCatalogLoadError> {
    if attachments.is_empty() {
        return Ok(participant.clone());
    }
    let base = participant.combatant();
    let mut modifiers = base.modifiers().to_vec();
    modifiers.extend(attachments.iter().map(|value| value.definition.id));
    modifiers.sort_unstable();
    modifiers.dedup();
    let mut sources = base.sources().to_vec();
    sources.extend(attachments.iter().map(|value| value.source.clone()));
    sources.sort_unstable_by_key(|source| source.definition());
    sources.dedup_by_key(|source| source.definition());
    let mut bindings = base.modifier_bindings().to_vec();
    bindings.extend(
        attachments.iter().map(|value| {
            ResolvedModifierBinding::new(value.definition.id, value.source.definition())
        }),
    );
    bindings.sort_unstable_by_key(|binding| binding.definition());
    bindings.dedup_by_key(|binding| binding.definition());
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.attached-combatant");
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
        .map_err(|_| invalid("invalid attached Swarm bindings"))?,
        CombatantSpecDigest::new(encoder.finish()).expect("SHA-256 digest is non-zero"),
    )
    .map_err(|_| invalid("invalid attached Swarm combatant"))?
    .with_base_attack_defense(base.base_attack(), base.base_defense())
    .with_base_effect_stats(base.base_effect_hit_rate(), base.base_effect_resistance())
    .with_energy(base.current_energy(), base.maximum_energy())
    .map_err(|_| invalid("invalid attached Swarm energy"))?
    .with_toughness(
        base.rank(),
        base.weaknesses().to_vec(),
        base.toughness_layers().to_vec(),
    )
    .map_err(|_| invalid("invalid attached Swarm toughness"))?;
    combatant = combatant
        .with_sources(sources)
        .map_err(|_| invalid("invalid attached Swarm sources"))?;
    combatant = combatant
        .with_modifier_bindings(bindings)
        .map_err(|_| invalid("invalid attached Swarm modifier bindings"))?;
    let mut output = ParticipantSpec::new(
        participant.side(),
        participant.formation(),
        participant.source(),
        combatant,
    )
    .with_wave(participant.wave())
    .ok_or_else(|| invalid("invalid attached Swarm participant"))?
    .with_locked_combatant_digest(participant.locked_combatant_digest());
    if let Some(initial) = participant.initial_state() {
        output = output
            .with_initial_state(initial)
            .ok_or_else(|| invalid("invalid attached Swarm initial state"))?;
    }
    Ok(output)
}

fn player_resources(
    snapshot: &CompiledSwarmBattleSnapshot,
) -> Result<TeamResourceSpec, UniverseCatalogLoadError> {
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
    .ok_or_else(|| invalid("invalid Swarm Resonance resource"))?;
    resources
        .with_keyed(vec![keyed])
        .ok_or_else(|| invalid("invalid Swarm player resources"))
}

fn materialization_digest(
    instance: &SwarmDisasterRuntimeInstance,
    roster: &UniverseBattleRoster,
    selection: &EncounterSelection,
    snapshot: &CompiledSwarmBattleSnapshot,
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.battle-materialization");
    encoder.digest(instance.battle_catalog.digest());
    encoder.digest(roster.participant_lock().bytes());
    encoder.digest(selection_digest(selection));
    encoder.digest(snapshot.digest);
    encoder.u32(u32::from(snapshot.blessing_count));
    encoder.u32(u32::from(snapshot.curio_count));
    encoder.u8(snapshot.interplay_count);
    encoder.u8(snapshot.trail_effect_count);
    encoder.optional_text(snapshot.next_battle_face.as_deref());
    encoder.finish()
}

fn combatant_digest(root: [u8; 32], enemy: &str, wave: usize, slot: usize) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.enemy-combatant");
    encoder.digest(root);
    encoder.text(enemy);
    encoder.u32(u32::try_from(wave).expect("wave index is bounded"));
    encoder.u32(u32::try_from(slot).expect("slot index is bounded"));
    encoder.finish()
}
