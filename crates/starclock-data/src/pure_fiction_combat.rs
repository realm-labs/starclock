//! Pure Fiction encounter composition over the shared combat catalog.

use std::{collections::BTreeMap, sync::Arc};

use sha2::{Digest, Sha256};
use starclock_combat::{
    AssemblyDigest, BattleSpec, CombatantSpecDigest, ConcedePolicy, EncounterId, EncounterWaveId,
    EnemyDefinitionId, FormationIndex, KeyedTeamResourceSpec, ParticipantSource, ParticipantSpec,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, ResolvedModifierBinding, RuleBundleId,
    SourceDefinitionId, TeamResourceSpec, TeamResourceWavePolicy, TeamSide, ToughnessLayerSpec,
    catalog::{
        CombatCatalog,
        builder::CombatCatalogBuilder,
        definition::EncounterDefinition,
        encounter::{
            EncounterWaveDefinition, SpawnEndPolicy, SpawnOrdering, SpawnProgramDefinition,
            SpawnRefillTiming, WaveCarry, WaveSlotDefinition, WaveTransitionPolicy,
        },
    },
};
use starclock_mode_challenge::{
    CACOPHONY_SOURCE, MIRTHFUL_CADENCE_BUNDLE, PURE_FICTION_SPAWN_BUNDLE,
    PureFictionCacophonyDefinitions, PureFictionCombatDefinitions, PureFictionEncounter,
    PureFictionEnemyBindingId, PureFictionEnemySlot, PureFictionMechanicsDefinitions,
    PureFictionSpawnEnd, TOCCATA_BUNDLE, TOCCATA_FOLLOW_UP_BOOST, TOCCATA_ULTIMATE_BOOST,
    VARIATION_BUNDLE,
};

use crate::{
    catalog::SimulationCatalog,
    challenge::{ChallengeDataError, message},
    challenge_combat::clone_enemy,
};
use starclock_combat::rule::model::RuleSource;

pub struct PureFictionBattleAssembly {
    pub assembly_digest: AssemblyDigest,
    pub player_resources: TeamResourceSpec,
    pub enemy_resources: TeamResourceSpec,
    pub concede: ConcedePolicy,
    pub cacophony: RuleBundleId,
}

const PUNCHLINE_RESOURCE_ID: SourceDefinitionId =
    SourceDefinitionId::new(0x7f20_0030).expect("reserved Pure Fiction ID is nonzero");
const PUNCHLINE_RESOURCE_KEY: &str = "shared.punchline";

#[derive(Clone, Debug)]
pub struct PureFictionCombatCatalog {
    combat: Arc<CombatCatalog>,
    enemies: Box<[(PureFictionEnemyBindingId, EnemyDefinitionId)]>,
    approximate_enemy_count: usize,
    cacophony_source: RuleSource,
}

impl PureFictionCombatCatalog {
    pub fn compile(
        definitions: &PureFictionCombatDefinitions,
        production: &SimulationCatalog,
    ) -> Result<Self, ChallengeDataError> {
        let mut resolved = BTreeMap::new();
        let mut aliases = Vec::new();
        for binding in definitions.enemies() {
            let source = production
                .enemy_by_stable_key(binding.behavior_source_key())
                .ok_or_else(|| message("Pure Fiction enemy behavior source is missing"))?;
            let enemy = if binding.behavior_exact() {
                let exact = production
                    .enemy_by_stable_key(binding.stable_key())
                    .ok_or_else(|| message("exact Pure Fiction enemy is missing"))?;
                if exact.id() != source.id() {
                    return Err(message("exact Pure Fiction behavior identity differs"));
                }
                exact.id()
            } else {
                let id = EnemyDefinitionId::new(binding.upstream_monster())
                    .ok_or_else(|| message("invalid mode-owned Pure Fiction enemy id"))?;
                if production.combat_catalog().enemy(id).is_some() {
                    return Err(message(
                        "mode-owned Pure Fiction enemy id collides with production",
                    ));
                }
                aliases.push(clone_enemy(source, id)?);
                id
            };
            if resolved.insert(binding.id(), enemy).is_some() {
                return Err(message("duplicate Pure Fiction enemy binding"));
            }
        }
        let digest = composition_digest(definitions, production.combat_catalog());
        let mut builder = CombatCatalogBuilder::from_catalog(production.combat_catalog(), digest);
        let mechanics = PureFictionMechanicsDefinitions::active();
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
        builder.add_ability(mechanics.ability);
        builder.add_countdown(mechanics.countdown);
        builder.add_rule(mechanics.rule);
        builder.add_rule_bundle(mechanics.bundle);
        let cacophony = PureFictionCacophonyDefinitions::active();
        for group in cacophony.modifier_groups {
            builder.add_modifier_group(group);
        }
        for modifier in cacophony.modifiers {
            builder.add_modifier(modifier);
        }
        for effect in cacophony.effects {
            builder.add_effect(effect);
        }
        for selector in cacophony.selectors {
            builder.add_selector(selector);
        }
        for program in cacophony.programs {
            builder.add_program(program);
        }
        for rule in cacophony.rules {
            builder.add_rule(rule);
        }
        for bundle in cacophony.bundles {
            builder.add_rule_bundle(bundle);
        }
        for alias in aliases {
            builder.add_enemy(alias);
        }
        for encounter in definitions.encounters() {
            let waves = expanded_waves(encounter)?;
            let authored = waves
                .iter()
                .map(|wave| compile_wave(encounter, wave, &resolved))
                .collect::<Result<Vec<_>, ChallengeDataError>>()?;
            builder.add_encounter(
                EncounterDefinition::new(encounter.id(), Vec::new(), Vec::new())
                    .with_authored_waves(WaveTransitionPolicy::AfterAction, authored)
                    .ok_or_else(|| message("invalid authored Pure Fiction encounter"))?,
            );
        }
        let combat = builder
            .build()
            .map_err(|error| message(&format!("invalid Pure Fiction catalog: {error}")))?;
        Ok(Self {
            combat,
            enemies: resolved.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            approximate_enemy_count: definitions
                .enemies()
                .iter()
                .filter(|enemy| !enemy.behavior_exact())
                .count(),
            cacophony_source: cacophony.source,
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
        definitions: &PureFictionCombatDefinitions,
        encounter_id: EncounterId,
        players: Vec<ParticipantSpec>,
        assembly: PureFictionBattleAssembly,
    ) -> Result<BattleSpec, ChallengeDataError> {
        if players.is_empty()
            || players
                .iter()
                .any(|participant| participant.side() != TeamSide::Player)
        {
            return Err(message("Pure Fiction battle requires only player inputs"));
        }
        if ![TOCCATA_BUNDLE, VARIATION_BUNDLE, MIRTHFUL_CADENCE_BUNDLE]
            .contains(&assembly.cacophony)
        {
            return Err(message(
                "Pure Fiction Cacophony is not supported by this profile",
            ));
        }
        let first = players
            .iter()
            .map(ParticipantSpec::formation)
            .min()
            .ok_or_else(|| message("Pure Fiction battle requires a player"))?;
        let mut players = players
            .iter()
            .map(|participant| {
                self.apply_cacophony(
                    participant,
                    assembly.cacophony,
                    participant.formation() == first,
                )
            })
            .collect::<Result<Vec<_>, ChallengeDataError>>()?;
        let encounter = definitions
            .encounters()
            .binary_search_by_key(&encounter_id, PureFictionEncounter::id)
            .ok()
            .map(|index| &definitions.encounters()[index])
            .ok_or_else(|| message("Pure Fiction encounter definition is missing"))?;
        for wave in expanded_waves(encounter)? {
            for slot in &wave.slots {
                players.push(self.enemy_participant(encounter, wave.sequence, slot)?);
            }
        }
        let player_resources =
            pure_fiction_player_resources(assembly.player_resources, assembly.cacophony)?;
        BattleSpec::new(
            assembly.assembly_digest,
            encounter_id,
            players,
            player_resources,
            assembly.enemy_resources,
            assembly.concede,
        )
        .map_err(|error| message(&format!("invalid Pure Fiction battle spec: {error}")))
    }

    fn apply_cacophony(
        &self,
        participant: &ParticipantSpec,
        selected: RuleBundleId,
        owns_rule: bool,
    ) -> Result<ParticipantSpec, ChallengeDataError> {
        let base = participant.combatant();
        let mut bundles = base.rule_bundles().to_vec();
        if owns_rule {
            bundles.extend([PURE_FICTION_SPAWN_BUNDLE, selected]);
            bundles.sort_unstable();
            if bundles.windows(2).any(|pair| pair[0] == pair[1]) {
                return Err(message(
                    "Pure Fiction Cacophony rule collides with player build",
                ));
            }
        }
        let mut modifiers = base.modifiers().to_vec();
        let mut bindings = base.modifier_bindings().to_vec();
        if selected == TOCCATA_BUNDLE {
            modifiers.extend([TOCCATA_ULTIMATE_BOOST, TOCCATA_FOLLOW_UP_BOOST]);
            bindings.extend([
                ResolvedModifierBinding::new(TOCCATA_ULTIMATE_BOOST, CACOPHONY_SOURCE),
                ResolvedModifierBinding::new(TOCCATA_FOLLOW_UP_BOOST, CACOPHONY_SOURCE),
            ]);
        }
        modifiers.sort_unstable();
        bindings.sort_unstable_by_key(|binding| binding.definition());
        if modifiers.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(message(
                "Pure Fiction Cacophony modifier collides with player build",
            ));
        }
        let mut sources = base.sources().to_vec();
        sources.push(self.cacophony_source.clone());
        sources.sort_unstable_by_key(RuleSource::definition);
        if sources
            .windows(2)
            .any(|pair| pair[0].definition() == pair[1].definition())
        {
            return Err(message(
                "Pure Fiction Cacophony source collides with player build",
            ));
        }
        let combatant = ResolvedCombatantSpec::new(
            base.form(),
            base.level(),
            base.maximum_hp(),
            base.speed(),
            ResolvedDefinitionBindings::new(base.abilities().to_vec(), bundles, modifiers)
                .map_err(|_| message("invalid Pure Fiction player contribution"))?,
            cacophony_digest(base.digest(), selected, owns_rule),
        )
        .map_err(|_| message("invalid Pure Fiction player combatant"))?
        .with_base_attack_defense(base.base_attack(), base.base_defense())
        .with_base_effect_stats(base.base_effect_hit_rate(), base.base_effect_resistance())
        .with_energy(base.current_energy(), base.maximum_energy())
        .map_err(|_| message("invalid Pure Fiction player Energy"))?
        .with_toughness(
            base.rank(),
            base.weaknesses().to_vec(),
            base.toughness_layers().to_vec(),
        )
        .map_err(|_| message("invalid Pure Fiction player Toughness"))?
        .with_sources(sources)
        .and_then(|value| value.with_modifier_bindings(bindings))
        .map_err(|_| message("invalid Pure Fiction player source contribution"))?;
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
                .ok_or_else(|| message("invalid Pure Fiction carried state"))?;
        }
        Ok(projected)
    }

    fn enemy_participant(
        &self,
        encounter: &PureFictionEncounter,
        wave: u16,
        slot: &PureFictionEnemySlot,
    ) -> Result<ParticipantSpec, ChallengeDataError> {
        let enemy_id = self
            .enemies
            .binary_search_by_key(&slot.binding(), |(id, _)| *id)
            .ok()
            .map(|index| self.enemies[index].1)
            .ok_or_else(|| message("Pure Fiction enemy binding was not composed"))?;
        let enemy = self
            .combat
            .enemy(enemy_id)
            .ok_or_else(|| message("composed Pure Fiction enemy is missing"))?;
        let stats = slot.stats();
        let layers = if stats.toughness().get() == 0 {
            Vec::new()
        } else {
            vec![
                ToughnessLayerSpec::ordinary(1, stats.toughness())
                    .map_err(|_| message("invalid Pure Fiction Toughness layer"))?,
            ]
        };
        let combatant = ResolvedCombatantSpec::new(
            enemy.unit(),
            encounter.level(),
            stats.maximum_hp(),
            stats.speed(),
            ResolvedDefinitionBindings::new(enemy.abilities().to_vec(), Vec::new(), Vec::new())
                .map_err(|_| message("invalid Pure Fiction enemy bindings"))?,
            enemy_digest(self.combat.digest().bytes(), encounter.id(), wave, slot),
        )
        .map_err(|_| message("invalid Pure Fiction enemy combatant"))?
        .with_base_attack_defense(stats.attack(), stats.defense())
        .with_base_effect_stats(stats.effect_hit_rate(), stats.effect_resistance())
        .with_toughness(stats.rank(), stats.weaknesses().to_vec(), layers)
        .map_err(|_| message("invalid Pure Fiction weakness profile"))?;
        ParticipantSpec::new(
            TeamSide::Enemy,
            slot.formation(),
            ParticipantSource::EncounterEnemy(enemy_id),
            combatant,
        )
        .with_wave(wave)
        .ok_or_else(|| message("invalid Pure Fiction enemy entry wave"))
    }
}

fn pure_fiction_player_resources(
    resources: TeamResourceSpec,
    cacophony: RuleBundleId,
) -> Result<TeamResourceSpec, ChallengeDataError> {
    if cacophony != MIRTHFUL_CADENCE_BUNDLE
        || resources
            .keyed()
            .iter()
            .any(|resource| resource.stable_key() == Some(PUNCHLINE_RESOURCE_KEY))
    {
        return Ok(resources);
    }
    let mut keyed = resources.keyed().to_vec();
    keyed.push(
        KeyedTeamResourceSpec::new(
            PUNCHLINE_RESOURCE_ID,
            0,
            999,
            TeamResourceWavePolicy::Persist,
        )
        .and_then(|resource| resource.with_stable_key(PUNCHLINE_RESOURCE_KEY))
        .expect("bounded Punchline contribution is valid"),
    );
    TeamResourceSpec::new(resources.skill_points(), resources.maximum_skill_points())
        .and_then(|value| value.with_keyed(keyed))
        .ok_or_else(|| message("Pure Fiction Punchline resource collides with player resources"))
}

struct ExpandedWave {
    sequence: u16,
    slots: Vec<PureFictionEnemySlot>,
    required_from: usize,
    end: PureFictionSpawnEnd,
    maximum_simultaneous: u8,
}

fn expanded_waves(
    encounter: &PureFictionEncounter,
) -> Result<Vec<ExpandedWave>, ChallengeDataError> {
    let source = encounter
        .waves()
        .first()
        .ok_or_else(|| message("Pure Fiction refill source wave is missing"))?;
    encounter
        .waves()
        .iter()
        .map(|wave| {
            if wave.refill_source_wave().is_none() {
                return Ok(ExpandedWave {
                    sequence: wave.sequence(),
                    slots: wave.slots().to_vec(),
                    required_from: wave.slots().len(),
                    end: wave.spawn_end(),
                    maximum_simultaneous: wave.maximum_simultaneous(),
                });
            }
            let mut slots = source
                .slots()
                .iter()
                .take(4)
                .enumerate()
                .map(|(index, slot)| {
                    slot.relocated(
                        u16::try_from(index + 1).expect("four refill slots fit u16"),
                        FormationIndex::new(u8::try_from(index).expect("four slots fit u8"))
                            .expect("formations zero through three are valid"),
                    )
                })
                .collect::<Vec<_>>();
            if slots.len() != 4 || wave.slots().len() != 1 {
                return Err(message("Pure Fiction derived refill topology drift"));
            }
            slots.push(
                wave.slots()[0]
                    .relocated(5, FormationIndex::new(4).expect("formation four is valid")),
            );
            Ok(ExpandedWave {
                sequence: wave.sequence(),
                slots,
                required_from: 4,
                end: wave.spawn_end(),
                maximum_simultaneous: wave.maximum_simultaneous(),
            })
        })
        .collect()
}

fn compile_wave(
    encounter: &PureFictionEncounter,
    wave: &ExpandedWave,
    enemies: &BTreeMap<PureFictionEnemyBindingId, EnemyDefinitionId>,
) -> Result<EncounterWaveDefinition, ChallengeDataError> {
    let wave_id = encounter
        .id()
        .get()
        .checked_mul(10)
        .and_then(|value| value.checked_add(u32::from(wave.sequence)))
        .and_then(EncounterWaveId::new)
        .ok_or_else(|| message("Pure Fiction wave identity overflow"))?;
    let slots = wave
        .slots
        .iter()
        .enumerate()
        .map(|(index, slot)| {
            WaveSlotDefinition::new(
                slot.spawn_sequence(),
                slot.formation(),
                *enemies
                    .get(&slot.binding())
                    .ok_or_else(|| message("Pure Fiction behavior binding is missing"))?,
                Some(encounter.level().get()),
                None,
                index >= wave.required_from,
            )
            .ok_or_else(|| message("invalid Pure Fiction wave slot"))
        })
        .collect::<Result<Vec<_>, ChallengeDataError>>()?;
    let refill_slots = wave.slots[..wave.required_from]
        .iter()
        .map(PureFictionEnemySlot::formation)
        .collect();
    let end = match wave.end {
        PureFictionSpawnEnd::DefeatQuota(quota) => SpawnEndPolicy::DefeatQuota(quota),
        PureFictionSpawnEnd::RequiredSlotsDefeated => SpawnEndPolicy::RequiredSlotsDefeated,
    };
    let spawn = SpawnProgramDefinition::new(
        SpawnRefillTiming::AfterDefeatSettlement,
        SpawnOrdering::AuthoredSlot,
        u16::from(wave.maximum_simultaneous),
        refill_slots,
        end,
    )
    .ok_or_else(|| message("invalid Pure Fiction spawn program"))?;
    EncounterWaveDefinition::new(
        wave_id,
        wave.sequence,
        None,
        None,
        WaveCarry::CARRY_ALL,
        slots,
    )
    .and_then(|definition| definition.with_spawn_program(spawn))
    .ok_or_else(|| message("invalid Pure Fiction encounter wave"))
}

fn composition_digest(
    definitions: &PureFictionCombatDefinitions,
    production: &CombatCatalog,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"starclock.challenge.pure-fiction-combat.v1");
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
        for wave in encounter.waves() {
            hash.update(wave.sequence().to_be_bytes());
            hash.update(wave.score_cap().to_be_bytes());
            hash.update(wave.normal_defeat_true_damage_scaled().to_be_bytes());
            for slot in wave.slots() {
                hash.update(slot.binding().get().to_be_bytes());
                hash.update(slot.spawn_sequence().to_be_bytes());
                hash.update([slot.formation().get()]);
            }
        }
    }
    hash.finalize().into()
}

fn enemy_digest(
    catalog: [u8; 32],
    encounter: EncounterId,
    wave: u16,
    slot: &PureFictionEnemySlot,
) -> CombatantSpecDigest {
    let mut hash = Sha256::new();
    hash.update(b"starclock.challenge.pure-fiction-enemy.v1");
    hash.update(catalog);
    hash.update(encounter.get().to_be_bytes());
    hash.update(wave.to_be_bytes());
    hash.update(slot.binding().get().to_be_bytes());
    hash.update(slot.spawn_sequence().to_be_bytes());
    hash.update([slot.formation().get()]);
    hash.update(slot.stats().maximum_hp().get().to_be_bytes());
    CombatantSpecDigest::new(hash.finalize().into()).expect("domain-separated SHA-256 is non-zero")
}

fn cacophony_digest(
    base: CombatantSpecDigest,
    selected: RuleBundleId,
    owns_rule: bool,
) -> CombatantSpecDigest {
    let mut hash = Sha256::new();
    hash.update(b"starclock.challenge.pure-fiction-cacophony.v1");
    hash.update(base.bytes());
    hash.update(selected.get().to_be_bytes());
    hash.update([u8::from(owns_rule)]);
    CombatantSpecDigest::new(hash.finalize().into()).expect("domain-separated SHA-256 is non-zero")
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use starclock_combat::{
        AssemblyDigest, Battle, BattleSeed, CombatantSpecDigest, ConcedePolicy, FormationIndex, Hp,
        ParticipantSource, ParticipantSpec, ResolvedCombatantSpec, ResolvedDefinitionBindings,
        Speed, TeamResourceSpec, TeamSide, UnitLevel,
    };
    use starclock_mode_challenge::{
        PURE_FICTION_CONCORDANT_EFFECT, TOCCATA_BUNDLE, TOCCATA_ULTIMATE_BOOST,
    };

    use super::{PureFictionBattleAssembly, PureFictionCombatCatalog};
    use crate::{catalog::load, challenge::pure_fiction_combat_definitions};

    const PRODUCTION: &[u8] = include_bytes!("../../../config/generated/config.sora");

    #[test]
    fn production_pure_fiction_catalog_composes_all_playable_encounters() {
        let production = load(PRODUCTION).expect("production catalog loads");
        let definitions = pure_fiction_combat_definitions().expect("Pure Fiction definitions load");
        let catalog = PureFictionCombatCatalog::compile(&definitions, &production)
            .expect("Pure Fiction catalog composes");
        assert_eq!(definitions.encounters().len(), 9);
        assert_eq!(catalog.approximate_enemy_count(), 30);
        assert!(
            definitions
                .encounters()
                .iter()
                .all(|encounter| catalog.combat().encounter(encounter.id()).is_some())
        );

        let unit = catalog
            .combat()
            .unit_ids()
            .find_map(|id| catalog.combat().unit(id))
            .expect("production catalog has a unit");
        let player = ResolvedCombatantSpec::new(
            unit.id(),
            UnitLevel::new(80).unwrap(),
            Hp::new(100_000).unwrap(),
            Speed::from_scaled(200_000_000).unwrap(),
            ResolvedDefinitionBindings::new(unit.abilities().to_vec(), Vec::new(), Vec::new())
                .unwrap(),
            CombatantSpecDigest::new([0x71; 32]).unwrap(),
        )
        .unwrap();
        let encounter = definitions.encounters()[0].id();
        let spec = catalog
            .assemble_battle(
                &definitions,
                encounter,
                vec![ParticipantSpec::new(
                    TeamSide::Player,
                    FormationIndex::new(0).unwrap(),
                    ParticipantSource::Player,
                    player,
                )],
                PureFictionBattleAssembly {
                    assembly_digest: AssemblyDigest::new([0x72; 32]).unwrap(),
                    player_resources: TeamResourceSpec::new(3, 5).unwrap(),
                    enemy_resources: TeamResourceSpec::new(0, 0).unwrap(),
                    concede: ConcedePolicy::Allowed,
                    cacophony: TOCCATA_BUNDLE,
                },
            )
            .expect("Pure Fiction battle assembles");
        assert!(
            spec.participants()[0]
                .combatant()
                .rule_bundles()
                .contains(&TOCCATA_BUNDLE)
        );
        assert!(
            spec.participants()[0]
                .combatant()
                .modifiers()
                .contains(&TOCCATA_ULTIMATE_BOOST)
        );
        let mut battle = Battle::create(
            Arc::clone(catalog.combat()),
            spec,
            BattleSeed::new([0x73; 32]),
        )
        .expect("assembled Pure Fiction battle starts");
        assert_eq!(battle.view().encounter().number(), 1);
        assert_eq!(
            battle
                .view()
                .units_by_id()
                .filter(|unit| unit.side() == TeamSide::Enemy)
                .count(),
            15
        );
        let start = battle
            .decision()
            .and_then(|decision| decision.legal_commands().first())
            .cloned()
            .expect("new battle offers start");
        battle.apply(start).expect("Pure Fiction battle starts");
        assert_eq!(
            battle
                .view()
                .effects_by_id()
                .filter(|effect| effect.definition() == PURE_FICTION_CONCORDANT_EFFECT)
                .count(),
            5
        );
    }
}
