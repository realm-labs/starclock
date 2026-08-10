//! Composition of mode-owned challenge encounters over the shared combat catalog.

use std::{collections::BTreeMap, sync::Arc};

use sha2::{Digest, Sha256};
use starclock_combat::{
    AssemblyDigest, BattleSpec, CombatantSpecDigest, ConcedePolicy, EncounterId, EncounterWaveId,
    EnemyDefinitionId, KeyedTeamResourceSpec, ParticipantSource, ParticipantSpec,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, ResolvedModifierBinding, RuleBundleId,
    TeamResourceSpec, TeamResourceWavePolicy, TeamSide, ToughnessLayerSpec,
    catalog::{
        CombatCatalog,
        builder::CombatCatalogBuilder,
        definition::{EncounterDefinition, EnemyDefinition},
        encounter::{EncounterWaveDefinition, WaveCarry, WaveSlotDefinition, WaveTransitionPolicy},
    },
    rule::model::RuleSource,
};
use starclock_mode_challenge::{
    APOCALYPTIC_PUNCHLINE_KEY, APOCALYPTIC_PUNCHLINE_RESOURCE, ApocalypticCombatDefinitions,
    ApocalypticEncounter, ApocalypticEnemyBindingId, ApocalypticEnemySlot,
    ApocalypticMechanicsDefinitions, FOLLOW_UP_BOOST, MemoryCombatDefinitions, MemoryEncounter,
    MemoryEnemyBindingId, MemoryEnemySlot, MemoryTurbulenceDefinitions, MemoryWave,
    OPPOSE_TENDERNESS_BUNDLE, RUINOUS_EMBERS_BUNDLE, TURBULENCE_BUNDLE, ULTIMATE_BOOST,
    released_axioms,
};

use crate::{
    catalog::SimulationCatalog,
    challenge::{ChallengeDataError, message},
};

/// Executable Memory encounter catalog composed over one production catalog.
#[derive(Clone, Debug)]
pub struct MemoryCombatCatalog {
    combat: Arc<CombatCatalog>,
    enemies: Box<[(MemoryEnemyBindingId, EnemyDefinitionId)]>,
    approximate_enemy_count: usize,
    turbulence_source: RuleSource,
}

/// Non-topology inputs required to assemble one Memory node battle.
pub struct MemoryBattleAssembly {
    pub assembly_digest: AssemblyDigest,
    pub player_resources: TeamResourceSpec,
    pub enemy_resources: TeamResourceSpec,
    pub concede: ConcedePolicy,
}

/// Non-topology inputs required to assemble one Apocalyptic boss node.
pub struct ApocalypticBattleAssembly {
    pub assembly_digest: AssemblyDigest,
    pub player_resources: TeamResourceSpec,
    pub enemy_resources: TeamResourceSpec,
    pub concede: ConcedePolicy,
    pub axiom: RuleBundleId,
}

/// Executable Apocalyptic boss catalog composed over production behavior donors.
#[derive(Clone, Debug)]
pub struct ApocalypticCombatCatalog {
    combat: Arc<CombatCatalog>,
    enemies: Box<[(ApocalypticEnemyBindingId, EnemyDefinitionId)]>,
    approximate_enemy_count: usize,
    mechanics_source: RuleSource,
}

impl ApocalypticCombatCatalog {
    pub fn compile(
        definitions: &ApocalypticCombatDefinitions,
        production: &SimulationCatalog,
    ) -> Result<Self, ChallengeDataError> {
        let mut resolved = BTreeMap::new();
        let mut aliases = Vec::new();
        for binding in definitions.enemies() {
            let source = production
                .enemy_by_stable_key(binding.behavior_source_key())
                .ok_or_else(|| message("Apocalyptic enemy behavior source is missing"))?;
            let enemy = if binding.behavior_exact() {
                let exact = production
                    .enemy_by_stable_key(binding.stable_key())
                    .ok_or_else(|| message("exact Apocalyptic enemy is missing"))?;
                if exact.id() != source.id() {
                    return Err(message("exact Apocalyptic behavior identity differs"));
                }
                exact.id()
            } else {
                let id = EnemyDefinitionId::new(binding.upstream_monster())
                    .ok_or_else(|| message("invalid mode-owned Apocalyptic enemy id"))?;
                if production.combat_catalog().enemy(id).is_some() {
                    return Err(message(
                        "mode-owned Apocalyptic enemy id collides with production",
                    ));
                }
                aliases.push(clone_enemy(source, id)?);
                id
            };
            if resolved.insert(binding.id(), enemy).is_some() {
                return Err(message("duplicate Apocalyptic enemy binding"));
            }
        }
        let digest = apocalyptic_composition_digest(definitions, production.combat_catalog());
        let mut builder = CombatCatalogBuilder::from_catalog(production.combat_catalog(), digest);
        let mechanics = ApocalypticMechanicsDefinitions::active();
        for group in mechanics.modifier_groups {
            builder.add_modifier_group(group);
        }
        for modifier in mechanics.modifiers {
            builder.add_modifier(modifier);
        }
        for effect in mechanics.effects {
            builder.add_effect(effect);
        }
        for selector in mechanics.selectors {
            builder.add_selector(selector);
        }
        for program in mechanics.programs {
            builder.add_program(program);
        }
        for rule in mechanics.rules {
            builder.add_rule(rule);
        }
        for bundle in mechanics.bundles {
            builder.add_rule_bundle(bundle);
        }
        for alias in aliases {
            builder.add_enemy(alias);
        }
        for encounter in definitions.encounters() {
            let wave = compile_apocalyptic_wave(encounter, &resolved)?;
            builder.add_encounter(
                EncounterDefinition::new(encounter.id(), Vec::new(), Vec::new())
                    .with_authored_waves(WaveTransitionPolicy::AfterAction, vec![wave])
                    .ok_or_else(|| message("invalid authored Apocalyptic encounter"))?,
            );
        }
        let combat = builder
            .build()
            .map_err(|error| message(&format!("invalid Apocalyptic catalog: {error}")))?;
        Ok(Self {
            combat,
            enemies: resolved.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            approximate_enemy_count: definitions
                .enemies()
                .iter()
                .filter(|enemy| !enemy.behavior_exact())
                .count(),
            mechanics_source: mechanics.source,
        })
    }

    #[must_use]
    pub const fn combat(&self) -> &Arc<CombatCatalog> {
        &self.combat
    }

    #[must_use]
    pub const fn approximate_enemy_count(&self) -> usize {
        self.approximate_enemy_count
    }

    pub fn assemble_battle(
        &self,
        definitions: &ApocalypticCombatDefinitions,
        encounter: EncounterId,
        players: Vec<ParticipantSpec>,
        assembly: ApocalypticBattleAssembly,
    ) -> Result<BattleSpec, ChallengeDataError> {
        if players.is_empty()
            || players
                .iter()
                .any(|participant| participant.side() != TeamSide::Player)
        {
            return Err(message("Apocalyptic battle requires only player inputs"));
        }
        if !released_axioms().contains(&assembly.axiom) {
            return Err(message(
                "Apocalyptic Axiom is not supported by this profile",
            ));
        }
        let first = players
            .iter()
            .map(ParticipantSpec::formation)
            .min()
            .ok_or_else(|| message("Apocalyptic battle requires a player"))?;
        let mut players = players
            .iter()
            .map(|participant| {
                self.apply_mechanics(
                    participant,
                    assembly.axiom,
                    participant.formation() == first,
                )
            })
            .collect::<Result<Vec<_>, ChallengeDataError>>()?;
        let definition = definitions
            .encounters()
            .binary_search_by_key(&encounter, ApocalypticEncounter::id)
            .ok()
            .map(|index| &definitions.encounters()[index])
            .ok_or_else(|| message("Apocalyptic encounter definition is missing"))?;
        for slot in definition.slots() {
            players.push(self.enemy_participant(definition, slot)?);
        }
        let player_resources =
            apocalyptic_player_resources(assembly.player_resources, assembly.axiom)?;
        BattleSpec::new(
            assembly.assembly_digest,
            encounter,
            players,
            player_resources,
            assembly.enemy_resources,
            assembly.concede,
        )
        .map_err(|error| message(&format!("invalid Apocalyptic battle spec: {error}")))
    }

    fn apply_mechanics(
        &self,
        participant: &ParticipantSpec,
        axiom: RuleBundleId,
        owns_rules: bool,
    ) -> Result<ParticipantSpec, ChallengeDataError> {
        let base = participant.combatant();
        let mut bundles = base.rule_bundles().to_vec();
        let mut sources = base.sources().to_vec();
        if owns_rules {
            bundles.extend([RUINOUS_EMBERS_BUNDLE, axiom]);
            bundles.sort_unstable();
            if bundles.windows(2).any(|pair| pair[0] == pair[1]) {
                return Err(message(
                    "Apocalyptic rule contribution collides with player build",
                ));
            }
            sources.push(self.mechanics_source.clone());
            sources.sort_unstable_by_key(RuleSource::definition);
            if sources
                .windows(2)
                .any(|pair| pair[0].definition() == pair[1].definition())
            {
                return Err(message("Apocalyptic source collides with player build"));
            }
        }
        let combatant = ResolvedCombatantSpec::new(
            base.form(),
            base.level(),
            base.maximum_hp(),
            base.speed(),
            ResolvedDefinitionBindings::new(
                base.abilities().to_vec(),
                bundles,
                base.modifiers().to_vec(),
            )
            .map_err(|_| message("invalid Apocalyptic player contribution"))?,
            apocalyptic_player_digest(base.digest(), axiom, owns_rules),
        )
        .map_err(|_| message("invalid Apocalyptic player combatant"))?
        .with_base_attack_defense(base.base_attack(), base.base_defense())
        .with_base_effect_stats(base.base_effect_hit_rate(), base.base_effect_resistance())
        .with_energy(base.current_energy(), base.maximum_energy())
        .map_err(|_| message("invalid Apocalyptic player Energy"))?
        .with_toughness(
            base.rank(),
            base.weaknesses().to_vec(),
            base.toughness_layers().to_vec(),
        )
        .map_err(|_| message("invalid Apocalyptic player Toughness"))?
        .with_sources(sources)
        .and_then(|value| value.with_modifier_bindings(base.modifier_bindings().to_vec()))
        .map_err(|_| message("invalid Apocalyptic player source contribution"))?;
        let mut projected = ParticipantSpec::new(
            participant.side(),
            participant.formation(),
            participant.source(),
            combatant,
        )
        .with_locked_combatant_digest(participant.locked_combatant_digest());
        if let Some(initial) = participant.initial_state() {
            projected = projected
                .with_initial_state(initial)
                .ok_or_else(|| message("invalid Apocalyptic player carried state"))?;
        }
        Ok(projected)
    }

    fn enemy_participant(
        &self,
        encounter: &ApocalypticEncounter,
        slot: &ApocalypticEnemySlot,
    ) -> Result<ParticipantSpec, ChallengeDataError> {
        let enemy_id = self
            .enemies
            .binary_search_by_key(&slot.binding(), |(id, _)| *id)
            .ok()
            .map(|index| self.enemies[index].1)
            .ok_or_else(|| message("Apocalyptic enemy binding was not composed"))?;
        let enemy = self
            .combat
            .enemy(enemy_id)
            .ok_or_else(|| message("composed Apocalyptic enemy is missing"))?;
        let stats = slot.stats();
        let layers = if stats.toughness().get() == 0 {
            Vec::new()
        } else {
            vec![
                ToughnessLayerSpec::ordinary(1, stats.toughness())
                    .map_err(|_| message("invalid Apocalyptic Toughness layer"))?,
            ]
        };
        let combatant = ResolvedCombatantSpec::new(
            enemy.unit(),
            encounter.level(),
            stats.maximum_hp(),
            stats.speed(),
            ResolvedDefinitionBindings::new(enemy.abilities().to_vec(), Vec::new(), Vec::new())
                .map_err(|_| message("invalid Apocalyptic enemy bindings"))?,
            apocalyptic_enemy_digest(self.combat.digest().bytes(), encounter.id(), slot),
        )
        .map_err(|_| message("invalid Apocalyptic enemy combatant"))?
        .with_base_attack_defense(stats.attack(), stats.defense())
        .with_base_effect_stats(stats.effect_hit_rate(), stats.effect_resistance())
        .with_toughness(stats.rank(), stats.weaknesses().to_vec(), layers)
        .map_err(|_| message("invalid Apocalyptic weakness profile"))?;
        Ok(ParticipantSpec::new(
            TeamSide::Enemy,
            slot.formation(),
            ParticipantSource::EncounterEnemy(enemy_id),
            combatant,
        ))
    }
}

impl MemoryCombatCatalog {
    /// Resolves reviewed definitions and explicit behavior donors, then validates
    /// every ordinary and Starward encounter as one immutable combat catalog.
    pub fn compile(
        definitions: &MemoryCombatDefinitions,
        production: &SimulationCatalog,
    ) -> Result<Self, ChallengeDataError> {
        let mut resolved = BTreeMap::new();
        let mut aliases = Vec::new();
        for binding in definitions.enemies() {
            let source = production
                .enemy_by_stable_key(binding.behavior_source_key())
                .ok_or_else(|| message("Memory enemy behavior source is missing"))?;
            let enemy = if binding.behavior_exact() {
                let exact = production
                    .enemy_by_stable_key(binding.stable_key())
                    .ok_or_else(|| message("exact Memory enemy is missing"))?;
                if exact.id() != source.id() {
                    return Err(message("exact Memory behavior source identity differs"));
                }
                exact.id()
            } else {
                let id = EnemyDefinitionId::new(binding.upstream_variant())
                    .ok_or_else(|| message("invalid mode-owned Memory enemy id"))?;
                if production.combat_catalog().enemy(id).is_some() {
                    return Err(message(
                        "mode-owned Memory enemy id collides with production",
                    ));
                }
                aliases.push(clone_enemy(source, id)?);
                id
            };
            if resolved.insert(binding.id(), enemy).is_some() {
                return Err(message("duplicate Memory enemy binding id"));
            }
        }

        let digest = composition_digest(definitions, production.combat_catalog());
        let mut builder = CombatCatalogBuilder::from_catalog(production.combat_catalog(), digest);
        let turbulence = MemoryTurbulenceDefinitions::active();
        builder.add_modifier_group(turbulence.modifier_group);
        for modifier in turbulence.modifiers {
            builder.add_modifier(modifier);
        }
        for selector in turbulence.selectors {
            builder.add_selector(selector);
        }
        for program in turbulence.programs {
            builder.add_program(program);
        }
        builder.add_rule(turbulence.rule);
        builder.add_rule_bundle(turbulence.bundle);
        for alias in aliases {
            builder.add_enemy(alias);
        }
        for encounter in definitions.encounters() {
            let waves = encounter
                .waves()
                .iter()
                .map(|wave| {
                    compile_wave(
                        encounter.id().get(),
                        encounter.level().get(),
                        wave,
                        &resolved,
                    )
                })
                .collect::<Result<Vec<_>, ChallengeDataError>>()?;
            builder.add_encounter(
                EncounterDefinition::new(encounter.id(), Vec::new(), Vec::new())
                    .with_authored_waves(WaveTransitionPolicy::AfterAction, waves)
                    .ok_or_else(|| message("invalid authored Memory encounter"))?,
            );
        }
        let combat = builder
            .build()
            .map_err(|error| message(&format!("invalid Memory combat catalog: {error}")))?;
        let enemies = resolved.into_iter().collect::<Vec<_>>().into_boxed_slice();
        Ok(Self {
            combat,
            enemies,
            approximate_enemy_count: definitions
                .enemies()
                .iter()
                .filter(|binding| !binding.behavior_exact())
                .count(),
            turbulence_source: turbulence.source,
        })
    }

    #[must_use]
    pub const fn combat(&self) -> &Arc<CombatCatalog> {
        &self.combat
    }

    #[must_use]
    pub fn enemy(&self, binding: MemoryEnemyBindingId) -> Option<EnemyDefinitionId> {
        self.enemies
            .binary_search_by_key(&binding, |(id, _)| *id)
            .ok()
            .map(|index| self.enemies[index].1)
    }

    #[must_use]
    pub const fn approximate_enemy_count(&self) -> usize {
        self.approximate_enemy_count
    }

    /// Builds one complete node battle from already resolved player participants
    /// and the typed enemy occurrence stats retained by the challenge bundle.
    pub fn assemble_battle(
        &self,
        definitions: &MemoryCombatDefinitions,
        encounter: EncounterId,
        players: Vec<ParticipantSpec>,
        assembly: MemoryBattleAssembly,
    ) -> Result<BattleSpec, ChallengeDataError> {
        if players
            .iter()
            .any(|participant| participant.side() != TeamSide::Player)
        {
            return Err(message("Memory battle player input contains a non-player"));
        }
        let first_formation = players
            .iter()
            .map(ParticipantSpec::formation)
            .min()
            .ok_or_else(|| message("Memory battle requires at least one player"))?;
        let mut players = players
            .iter()
            .map(|participant| {
                self.apply_turbulence(participant, participant.formation() == first_formation)
            })
            .collect::<Result<Vec<_>, ChallengeDataError>>()?;
        let definition = definitions
            .encounters()
            .binary_search_by_key(&encounter, MemoryEncounter::id)
            .ok()
            .map(|index| &definitions.encounters()[index])
            .ok_or_else(|| message("Memory encounter definition is missing"))?;
        for wave in definition.waves() {
            for slot in wave.slots() {
                players.push(self.enemy_participant(definition, wave.sequence(), slot)?);
            }
        }
        BattleSpec::new(
            assembly.assembly_digest,
            encounter,
            players,
            assembly.player_resources,
            assembly.enemy_resources,
            assembly.concede,
        )
        .map_err(|error| message(&format!("invalid Memory battle specification: {error}")))
    }

    fn apply_turbulence(
        &self,
        participant: &ParticipantSpec,
        owns_accumulator: bool,
    ) -> Result<ParticipantSpec, ChallengeDataError> {
        let base = participant.combatant();
        let mut modifiers = base.modifiers().to_vec();
        modifiers.extend([ULTIMATE_BOOST, FOLLOW_UP_BOOST]);
        modifiers.sort_unstable();
        if modifiers.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(message(
                "Memory Turbulence modifier identity collides with player build",
            ));
        }
        let mut bundles = base.rule_bundles().to_vec();
        if owns_accumulator {
            bundles.push(TURBULENCE_BUNDLE);
            bundles.sort_unstable();
            if bundles.windows(2).any(|pair| pair[0] == pair[1]) {
                return Err(message(
                    "Memory Turbulence rule identity collides with player build",
                ));
            }
        }
        let mut sources = base.sources().to_vec();
        sources.push(self.turbulence_source.clone());
        sources.sort_unstable_by_key(RuleSource::definition);
        if sources
            .windows(2)
            .any(|pair| pair[0].definition() == pair[1].definition())
        {
            return Err(message(
                "Memory Turbulence source identity collides with player build",
            ));
        }
        let mut bindings = base.modifier_bindings().to_vec();
        bindings.extend([
            ResolvedModifierBinding::new(ULTIMATE_BOOST, self.turbulence_source.definition()),
            ResolvedModifierBinding::new(FOLLOW_UP_BOOST, self.turbulence_source.definition()),
        ]);
        bindings.sort_unstable_by_key(|binding| binding.definition());
        let digest = turbulence_player_digest(base.digest(), owns_accumulator);
        let mut combatant = ResolvedCombatantSpec::new(
            base.form(),
            base.level(),
            base.maximum_hp(),
            base.speed(),
            ResolvedDefinitionBindings::new(base.abilities().to_vec(), bundles, modifiers)
                .map_err(|_| message("invalid Memory player definition contribution"))?,
            digest,
        )
        .map_err(|_| message("invalid Memory player combatant"))?
        .with_base_attack_defense(base.base_attack(), base.base_defense())
        .with_base_effect_stats(base.base_effect_hit_rate(), base.base_effect_resistance())
        .with_energy(base.current_energy(), base.maximum_energy())
        .map_err(|_| message("invalid Memory player Energy"))?
        .with_toughness(
            base.rank(),
            base.weaknesses().to_vec(),
            base.toughness_layers().to_vec(),
        )
        .map_err(|_| message("invalid Memory player Toughness"))?;
        combatant = combatant
            .with_sources(sources)
            .and_then(|value| value.with_modifier_bindings(bindings))
            .map_err(|_| message("invalid Memory player source contribution"))?;
        let mut projected = ParticipantSpec::new(
            participant.side(),
            participant.formation(),
            participant.source(),
            combatant,
        )
        .with_locked_combatant_digest(participant.locked_combatant_digest());
        if let Some(initial) = participant.initial_state() {
            projected = projected
                .with_initial_state(initial)
                .ok_or_else(|| message("invalid Memory player carried state"))?;
        }
        Ok(projected)
    }

    fn enemy_participant(
        &self,
        encounter: &MemoryEncounter,
        wave: u16,
        slot: &MemoryEnemySlot,
    ) -> Result<ParticipantSpec, ChallengeDataError> {
        let enemy_id = self
            .enemy(slot.binding())
            .ok_or_else(|| message("Memory enemy binding was not composed"))?;
        let enemy = self
            .combat
            .enemy(enemy_id)
            .ok_or_else(|| message("composed Memory enemy is missing"))?;
        let stats = slot.stats();
        let mut layers = Vec::new();
        if stats.toughness().get() > 0 {
            layers.push(
                ToughnessLayerSpec::ordinary(1, stats.toughness())
                    .map_err(|_| message("invalid Memory enemy Toughness layer"))?,
            );
        }
        let combatant = ResolvedCombatantSpec::new(
            enemy.unit(),
            encounter.level(),
            stats.maximum_hp(),
            stats.speed(),
            ResolvedDefinitionBindings::new(enemy.abilities().to_vec(), Vec::new(), Vec::new())
                .map_err(|_| message("invalid Memory enemy definition bindings"))?,
            enemy_digest(self.combat.digest().bytes(), encounter.id(), wave, slot),
        )
        .map_err(|_| message("invalid Memory enemy combatant"))?
        .with_base_attack_defense(stats.attack(), stats.defense())
        .with_base_effect_stats(stats.effect_hit_rate(), stats.effect_resistance())
        .with_toughness(stats.rank(), stats.weaknesses().to_vec(), layers)
        .map_err(|_| message("invalid Memory enemy weakness or Toughness profile"))?;
        ParticipantSpec::new(
            TeamSide::Enemy,
            slot.formation(),
            ParticipantSource::EncounterEnemy(enemy_id),
            combatant,
        )
        .with_wave(wave)
        .ok_or_else(|| message("invalid Memory enemy entry wave"))
    }
}

fn apocalyptic_player_resources(
    resources: TeamResourceSpec,
    axiom: RuleBundleId,
) -> Result<TeamResourceSpec, ChallengeDataError> {
    if axiom != OPPOSE_TENDERNESS_BUNDLE
        || resources
            .keyed()
            .iter()
            .any(|resource| resource.stable_key() == Some(APOCALYPTIC_PUNCHLINE_KEY))
    {
        return Ok(resources);
    }
    let mut keyed = resources.keyed().to_vec();
    keyed.push(
        KeyedTeamResourceSpec::new(
            APOCALYPTIC_PUNCHLINE_RESOURCE,
            0,
            999,
            TeamResourceWavePolicy::Persist,
        )
        .and_then(|resource| resource.with_stable_key(APOCALYPTIC_PUNCHLINE_KEY))
        .ok_or_else(|| message("invalid Apocalyptic Punchline resource"))?,
    );
    TeamResourceSpec::new(resources.skill_points(), resources.maximum_skill_points())
        .and_then(|value| value.with_keyed(keyed))
        .ok_or_else(|| message("invalid Apocalyptic player resources"))
}

fn compile_wave(
    encounter: u32,
    level: u8,
    wave: &MemoryWave,
    enemies: &BTreeMap<MemoryEnemyBindingId, EnemyDefinitionId>,
) -> Result<EncounterWaveDefinition, ChallengeDataError> {
    let wave_id = encounter
        .checked_mul(10)
        .and_then(|value| value.checked_add(u32::from(wave.sequence())))
        .and_then(EncounterWaveId::new)
        .ok_or_else(|| message("Memory wave identity overflow"))?;
    let slots = wave
        .slots()
        .iter()
        .map(|slot| {
            WaveSlotDefinition::new(
                slot.spawn_sequence(),
                slot.formation(),
                *enemies
                    .get(&slot.binding())
                    .ok_or_else(|| message("Memory slot behavior binding is missing"))?,
                Some(level),
                None,
                true,
            )
            .ok_or_else(|| message("invalid Memory wave slot"))
        })
        .collect::<Result<Vec<_>, ChallengeDataError>>()?;
    EncounterWaveDefinition::new(
        wave_id,
        wave.sequence(),
        None,
        None,
        WaveCarry::CARRY_ALL,
        slots,
    )
    .ok_or_else(|| message("invalid Memory encounter wave"))
}

fn compile_apocalyptic_wave(
    encounter: &ApocalypticEncounter,
    enemies: &BTreeMap<ApocalypticEnemyBindingId, EnemyDefinitionId>,
) -> Result<EncounterWaveDefinition, ChallengeDataError> {
    let wave_id = encounter
        .id()
        .get()
        .checked_mul(10)
        .and_then(|value| value.checked_add(1))
        .and_then(EncounterWaveId::new)
        .ok_or_else(|| message("Apocalyptic wave identity overflow"))?;
    let slots = encounter
        .slots()
        .iter()
        .enumerate()
        .map(|(index, slot)| {
            WaveSlotDefinition::new(
                u16::try_from(index + 1)
                    .map_err(|_| message("Apocalyptic spawn sequence exceeds u16"))?,
                slot.formation(),
                *enemies
                    .get(&slot.binding())
                    .ok_or_else(|| message("Apocalyptic behavior binding is missing"))?,
                Some(encounter.level().get()),
                None,
                true,
            )
            .ok_or_else(|| message("invalid Apocalyptic wave slot"))
        })
        .collect::<Result<Vec<_>, ChallengeDataError>>()?;
    EncounterWaveDefinition::new(wave_id, 1, None, None, WaveCarry::CARRY_ALL, slots)
        .ok_or_else(|| message("invalid Apocalyptic encounter wave"))
}

pub(super) fn clone_enemy(
    source: &EnemyDefinition,
    id: EnemyDefinitionId,
) -> Result<EnemyDefinition, ChallengeDataError> {
    let mut definition = EnemyDefinition::new(id, source.unit(), source.abilities().to_vec());
    if !source.links().is_empty() {
        definition = definition
            .with_links(source.links().to_vec())
            .ok_or_else(|| message("invalid Memory enemy donor links"))?;
    }
    if let Some(graph) = source.ai_graph() {
        definition = definition
            .with_orchestration(graph, source.phases().to_vec())
            .ok_or_else(|| message("invalid Memory enemy donor orchestration"))?;
    }
    Ok(definition)
}

fn composition_digest(
    definitions: &MemoryCombatDefinitions,
    production: &CombatCatalog,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"starclock.challenge.memory-combat.v1");
    hash.update(production.digest().bytes());
    for enemy in definitions.enemies() {
        hash.update(enemy.id().get().to_be_bytes());
        hash.update(enemy.upstream_variant().to_be_bytes());
        hash.update(enemy.stable_key().as_bytes());
        hash.update([0]);
        hash.update(enemy.behavior_source_key().as_bytes());
        hash.update([u8::from(enemy.behavior_exact())]);
    }
    for encounter in definitions.encounters() {
        hash.update(encounter.id().get().to_be_bytes());
        hash.update([encounter.level().get()]);
        hash.update(encounter.hard_level_group().to_be_bytes());
        for wave in encounter.waves() {
            hash.update(wave.sequence().to_be_bytes());
            for slot in wave.slots() {
                hash.update(slot.binding().get().to_be_bytes());
                hash.update(slot.spawn_sequence().to_be_bytes());
                hash.update([slot.formation().get()]);
            }
        }
    }
    hash.finalize().into()
}

fn apocalyptic_composition_digest(
    definitions: &ApocalypticCombatDefinitions,
    production: &CombatCatalog,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"starclock.challenge.apocalyptic-combat.v1");
    hash.update(production.digest().bytes());
    for enemy in definitions.enemies() {
        hash.update(enemy.id().get().to_be_bytes());
        hash.update(enemy.upstream_monster().to_be_bytes());
        hash.update(enemy.stable_key().as_bytes());
        hash.update([0]);
        hash.update(enemy.behavior_source_key().as_bytes());
        hash.update([u8::from(enemy.behavior_exact())]);
    }
    for encounter in definitions.encounters() {
        hash.update(encounter.id().get().to_be_bytes());
        hash.update([encounter.level().get()]);
        for slot in encounter.slots() {
            hash.update(slot.binding().get().to_be_bytes());
            hash.update([slot.formation().get(), u8::from(slot.score_included())]);
            hash.update(slot.stats().maximum_hp().get().to_be_bytes());
        }
    }
    hash.finalize().into()
}

fn enemy_digest(
    catalog: [u8; 32],
    encounter: EncounterId,
    wave: u16,
    slot: &MemoryEnemySlot,
) -> CombatantSpecDigest {
    let mut hash = Sha256::new();
    hash.update(b"starclock.challenge.memory-enemy.v1");
    hash.update(catalog);
    hash.update(encounter.get().to_be_bytes());
    hash.update(wave.to_be_bytes());
    hash.update(slot.binding().get().to_be_bytes());
    hash.update(slot.spawn_sequence().to_be_bytes());
    hash.update([slot.formation().get()]);
    hash.update(slot.stats().maximum_hp().get().to_be_bytes());
    hash.update(slot.stats().speed().scaled().to_be_bytes());
    CombatantSpecDigest::new(hash.finalize().into()).expect("domain-separated SHA-256 is non-zero")
}

fn apocalyptic_enemy_digest(
    catalog: [u8; 32],
    encounter: EncounterId,
    slot: &ApocalypticEnemySlot,
) -> CombatantSpecDigest {
    let mut hash = Sha256::new();
    hash.update(b"starclock.challenge.apocalyptic-enemy.v1");
    hash.update(catalog);
    hash.update(encounter.get().to_be_bytes());
    hash.update(slot.binding().get().to_be_bytes());
    hash.update([slot.formation().get()]);
    hash.update(slot.stats().maximum_hp().get().to_be_bytes());
    CombatantSpecDigest::new(hash.finalize().into()).expect("domain-separated SHA-256 is non-zero")
}

fn apocalyptic_player_digest(
    base: CombatantSpecDigest,
    axiom: RuleBundleId,
    owns_rules: bool,
) -> CombatantSpecDigest {
    let mut hash = Sha256::new();
    hash.update(b"starclock.challenge.apocalyptic-player.v1");
    hash.update(base.bytes());
    hash.update(axiom.get().to_be_bytes());
    hash.update([u8::from(owns_rules)]);
    CombatantSpecDigest::new(hash.finalize().into()).expect("domain-separated SHA-256 is non-zero")
}

fn turbulence_player_digest(
    base: CombatantSpecDigest,
    owns_accumulator: bool,
) -> CombatantSpecDigest {
    let mut hash = Sha256::new();
    hash.update(b"starclock.challenge.memory-player-turbulence.v1");
    hash.update(base.bytes());
    hash.update([u8::from(owns_accumulator)]);
    CombatantSpecDigest::new(hash.finalize().into()).expect("domain-separated SHA-256 is non-zero")
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{
        ApocalypticBattleAssembly, ApocalypticCombatCatalog, MemoryBattleAssembly,
        MemoryCombatCatalog,
    };
    use crate::{
        catalog::load,
        challenge::{apocalyptic_shadow_combat_definitions, memory_of_chaos_combat_definitions},
    };
    use starclock_combat::{
        AssemblyDigest, Battle, BattleClockEventData, BattleEventKind, BattlePhase, BattleSeed,
        CombatantSpecDigest, Command, ConcedePolicy, DecisionKind, EncounterId, Energy,
        FormationIndex, Hp, ParticipantSource, ParticipantSpec, ResolvedCombatantSpec,
        ResolvedDefinitionBindings, Speed, TeamResourceSpec, TeamSide, UnitLevel,
        catalog::action::AbilityKind, rule::model::RuleValue,
    };
    use starclock_mode_challenge::{
        FOLLOW_UP_BOOST, SHATTERSTRIKE_BUNDLE, TURBULENCE_RULE, TURBULENCE_SOURCE, ULTIMATE_BOOST,
        memory_of_chaos::MemoryProfile,
    };

    const PRODUCTION: &[u8] = include_bytes!("../../../config/generated/config.sora");

    #[test]
    fn apocalyptic_catalog_composes_all_playable_encounters() {
        let production = load(PRODUCTION).expect("production catalog loads");
        let definitions =
            apocalyptic_shadow_combat_definitions().expect("Apocalyptic combat definitions load");
        let catalog = ApocalypticCombatCatalog::compile(&definitions, &production)
            .expect("Apocalyptic catalog composes");
        assert_eq!(catalog.approximate_enemy_count(), 10);
        assert!(definitions.encounters().iter().all(|encounter| {
            catalog
                .combat()
                .encounter(encounter.id())
                .is_some_and(|compiled| compiled.waves().len() == 1)
        }));
        let unit = catalog
            .combat()
            .unit_ids()
            .find_map(|id| catalog.combat().unit(id))
            .expect("production catalog contains a player form");
        let player = ResolvedCombatantSpec::new(
            unit.id(),
            UnitLevel::new(80).unwrap(),
            Hp::new(100_000).unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
            ResolvedDefinitionBindings::new(unit.abilities().to_vec(), Vec::new(), Vec::new())
                .unwrap(),
            CombatantSpecDigest::new([0x41; 32]).unwrap(),
        )
        .unwrap();
        let spec = catalog
            .assemble_battle(
                &definitions,
                EncounterId::new(420_474).unwrap(),
                vec![ParticipantSpec::new(
                    TeamSide::Player,
                    FormationIndex::new(0).unwrap(),
                    ParticipantSource::Player,
                    player,
                )],
                ApocalypticBattleAssembly {
                    assembly_digest: AssemblyDigest::new([0x42; 32]).unwrap(),
                    player_resources: TeamResourceSpec::new(3, 5).unwrap(),
                    enemy_resources: TeamResourceSpec::new(0, 0).unwrap(),
                    concede: ConcedePolicy::Allowed,
                    axiom: SHATTERSTRIKE_BUNDLE,
                },
            )
            .expect("Apocalyptic battle assembles");
        let mut battle = Battle::create(
            Arc::clone(catalog.combat()),
            spec,
            BattleSeed::new([0x43; 32]),
        )
        .expect("assembled Apocalyptic battle starts");
        let start = battle
            .decision()
            .and_then(|decision| decision.legal_commands().first())
            .cloned()
            .expect("Apocalyptic battle exposes its start transition");
        battle
            .apply(start)
            .expect("Apocalyptic battle start applies");
        assert_eq!(
            battle
                .view()
                .units_by_id()
                .filter(|unit| unit.side() == TeamSide::Enemy)
                .count(),
            2
        );
        assert_eq!(
            battle.view().effects_by_id().count(),
            4,
            "Ruinous Embers and Shatterstrike apply to both enemies at battle start",
        );
    }

    #[test]
    fn memory_combat_catalog_composes_all_playable_encounters() {
        let production = load(PRODUCTION).expect("production catalog loads");
        let definitions =
            memory_of_chaos_combat_definitions().expect("Memory combat definitions load");
        let catalog = MemoryCombatCatalog::compile(&definitions, &production)
            .expect("Memory combat catalog composes");
        assert_eq!(catalog.approximate_enemy_count(), 22);
        for encounter in definitions.encounters() {
            let compiled = catalog
                .combat()
                .encounter(encounter.id())
                .expect("playable Memory encounter exists");
            assert_eq!(compiled.waves().len(), 2);
        }
        assert!(
            catalog
                .combat()
                .encounter(EncounterId::new(30_123_011).unwrap())
                .is_some()
        );
        let unit = catalog
            .combat()
            .unit_ids()
            .find_map(|id| {
                let unit = catalog.combat().unit(id)?;
                unit.abilities()
                    .iter()
                    .any(|ability| {
                        catalog
                            .combat()
                            .ability(*ability)
                            .and_then(|definition| definition.action())
                            .is_some_and(|action| action.kind() == AbilityKind::Ultimate)
                    })
                    .then_some(unit)
            })
            .expect("production catalog has an Ultimate-capable form");
        let player = ResolvedCombatantSpec::new(
            unit.id(),
            UnitLevel::new(80).unwrap(),
            Hp::new(100_000).unwrap(),
            Speed::from_scaled(200_000_000).unwrap(),
            ResolvedDefinitionBindings::new(unit.abilities().to_vec(), Vec::new(), Vec::new())
                .unwrap(),
            CombatantSpecDigest::new([0x31; 32]).unwrap(),
        )
        .unwrap()
        .with_energy(
            Energy::from_scaled(1_000_000_000).unwrap(),
            Energy::from_scaled(1_000_000_000).unwrap(),
        )
        .unwrap();
        let spec = catalog
            .assemble_battle(
                &definitions,
                EncounterId::new(30_123_011).unwrap(),
                vec![ParticipantSpec::new(
                    TeamSide::Player,
                    FormationIndex::new(0).unwrap(),
                    ParticipantSource::Player,
                    player,
                )],
                MemoryBattleAssembly {
                    assembly_digest: AssemblyDigest::new([0x32; 32]).unwrap(),
                    player_resources: TeamResourceSpec::new(3, 5).unwrap(),
                    enemy_resources: TeamResourceSpec::new(0, 0).unwrap(),
                    concede: ConcedePolicy::Allowed,
                },
            )
            .expect("Memory battle assembles")
            .with_clock(
                MemoryProfile::version_4_4_clock()
                    .compile(30)
                    .expect("released Memory clock compiles"),
            );
        let mut battle = Battle::create(
            Arc::clone(catalog.combat()),
            spec,
            BattleSeed::new([0x33; 32]),
        )
        .expect("assembled Memory battle starts");
        assert_eq!(battle.view().encounter().total_waves(), 2);
        assert_eq!(
            battle
                .view()
                .rule_instances_by_id()
                .filter(|rule| rule.rule() == TURBULENCE_RULE)
                .count(),
            1
        );
        assert_eq!(
            battle
                .view()
                .modifier_instances_by_id()
                .filter(|modifier| {
                    [ULTIMATE_BOOST, FOLLOW_UP_BOOST].contains(&modifier.definition())
                })
                .count(),
            2
        );

        battle
            .apply(Command::StartBattle {
                decision: battle.decision().unwrap().id(),
            })
            .expect("Memory battle starts");
        let ultimate = battle
            .available_ultimates()
            .into_iter()
            .next()
            .expect("full Energy exposes an Ultimate");
        battle
            .apply(
                battle
                    .request_ultimate_command(ultimate)
                    .expect("Ultimate request is legal"),
            )
            .expect("Ultimate request applies");
        drive_pending_action(&mut battle);
        assert_eq!(turbulence_hits(&battle), 1);

        let mut saw_cycle = false;
        let mut saw_turbulence_damage = false;
        for _ in 0..32 {
            let command = next_progress_command(&battle);
            let resolution = battle
                .apply(command)
                .expect("offered Memory command applies");
            saw_cycle |= resolution.events().iter().any(|event| {
                matches!(
                    event.kind(),
                    BattleEventKind::Clock(BattleClockEventData::CycleTicked { .. })
                )
            });
            saw_turbulence_damage |= resolution.events().iter().any(|event| {
                matches!(event.kind(), BattleEventKind::Damage(_))
                    && event.cause().source_definition() == Some(TURBULENCE_SOURCE)
            });
            if saw_cycle {
                break;
            }
        }
        assert!(saw_cycle, "bounded progression reaches the next cycle");
        assert!(
            saw_turbulence_damage,
            "cycle start discharges stored True DMG"
        );
        assert_eq!(turbulence_hits(&battle), 0);
    }

    fn drive_pending_action(battle: &mut Battle) {
        for _ in 0..8 {
            if battle.view().phase() == BattlePhase::ReadyToAdvance {
                return;
            }
            let decision = battle.decision().expect("pending action has a decision");
            if decision.kind() == DecisionKind::NormalAction {
                return;
            }
            let command = match decision.kind() {
                DecisionKind::PreparedAction => decision
                    .legal_commands()
                    .iter()
                    .find(|command| matches!(command, Command::CommitPreparedAction { .. })),
                DecisionKind::ActionFrame => decision
                    .legal_commands()
                    .iter()
                    .find(|command| matches!(command, Command::CommitActionFrame { .. })),
                _ => None,
            }
            .cloned()
            .expect("Ultimate decision exposes a commit command");
            battle
                .apply(command)
                .expect("Ultimate continuation applies");
        }
        panic!("Ultimate exceeded bounded continuation steps");
    }

    fn next_progress_command(battle: &Battle) -> Command {
        if let Some(command) = battle.advance_command() {
            return command;
        }
        let decision = battle
            .decision()
            .expect("nonterminal Memory battle has a decision");
        match decision.kind() {
            DecisionKind::NormalAction => decision
                .legal_commands()
                .iter()
                .find(|command| matches!(command, Command::UseAbility { .. })),
            DecisionKind::PreparedAction => decision
                .legal_commands()
                .iter()
                .find(|command| matches!(command, Command::CommitPreparedAction { .. })),
            DecisionKind::ActionFrame => decision
                .legal_commands()
                .iter()
                .find(|command| matches!(command, Command::CommitActionFrame { .. })),
            DecisionKind::BattleStart | DecisionKind::BattleChoice => None,
        }
        .cloned()
        .expect("Memory progression exposes a supported command")
    }

    fn turbulence_hits(battle: &Battle) -> i64 {
        battle
            .view()
            .rule_instances_by_id()
            .find(|rule| rule.rule() == TURBULENCE_RULE)
            .and_then(|rule| {
                rule.slots().find_map(|(_, value)| match value {
                    RuleValue::Integer(value) => Some(*value),
                    _ => None,
                })
            })
            .expect("Memory Turbulence exposes its public accumulator")
    }
}
