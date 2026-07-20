//! Enemy AI, mechanically distinct enemy, and encounter row lowering.

use std::collections::{BTreeMap, BTreeSet};

use starclock_combat::{
    AbilityId, AiCandidateId, AiGraphId, AiStateId, AiTransitionId, EncounterId, EncounterWaveId,
    EnemyDefinitionId, EnemyPhaseId, FormationIndex, ProgramId, RuleBundleId, SelectorId,
    UnitDefinitionId,
    catalog::{
        definition::{EncounterDefinition, EnemyDefinition},
        encounter::{
            AiCandidateDefinition, AiCandidateSelection, AiGraphDefinition, AiNoTargetFallback,
            AiStateDefinition, AiTransitionDefinition, AiTransitionTiming, EncounterWaveDefinition,
            EnemyLinkDefinition, EnemyLinkKind, EnemyPhaseCarry, EnemyPhaseDefinition,
            EnemyPhaseTransitionModel, LinkOverflowPolicy, LinkedFormationPolicy, PhaseCarryPolicy,
            WaveCarry, WaveCarryPolicy, WaveSlotDefinition, WaveTransitionPolicy,
        },
    },
};

use crate::{
    catalog::{
        CatalogLoadError, CombatDefinitions, IdentityDefinition, IdentityKind, LoadMode,
        contiguous, domain_fail, parse_decimal, positive, positive_u16, require_identity,
    },
    generated::{self, SoraConfig},
};

#[derive(Debug)]
pub(super) struct EncounterDefinitions {
    pub(super) ai_graphs: Box<[AiGraphDefinition]>,
    pub(super) enemies: Box<[EnemyDefinition]>,
    pub(super) encounters: Box<[EncounterDefinition]>,
}

impl EncounterDefinitions {
    pub(super) fn ai_graph(&self, id: AiGraphId) -> Option<&AiGraphDefinition> {
        lookup(&self.ai_graphs, id, AiGraphDefinition::id)
    }

    pub(super) fn enemy(&self, id: EnemyDefinitionId) -> Option<&EnemyDefinition> {
        lookup(&self.enemies, id, EnemyDefinition::id)
    }

    pub(super) fn encounter(&self, id: EncounterId) -> Option<&EncounterDefinition> {
        lookup(&self.encounters, id, EncounterDefinition::id)
    }
}

fn lookup<T, I: Ord + Copy>(values: &[T], id: I, key: impl Fn(&T) -> I) -> Option<&T> {
    values
        .binary_search_by_key(&id, key)
        .ok()
        .map(|index| &values[index])
}

pub(super) fn convert(
    config: &SoraConfig,
    mode: LoadMode,
    identities: &BTreeMap<u32, &IdentityDefinition>,
    combat: &CombatDefinitions,
) -> Result<EncounterDefinitions, CatalogLoadError> {
    validate_enemy_rows(config, mode, identities, combat)?;
    let ai_graphs = lower_ai_graphs(config, mode, identities)?;
    let enemies = lower_enemies(config, mode, identities)?;
    let encounters = lower_encounters(config, mode, identities, &enemies)?;
    Ok(EncounterDefinitions {
        ai_graphs: ai_graphs.into_boxed_slice(),
        enemies: enemies.into_boxed_slice(),
        encounters: encounters.into_boxed_slice(),
    })
}

fn lower_ai_graphs(
    config: &SoraConfig,
    mode: LoadMode,
    identities: &BTreeMap<u32, &IdentityDefinition>,
) -> Result<Vec<AiGraphDefinition>, CatalogLoadError> {
    let mut graphs = Vec::new();
    for row in config.ai_graph().ordered_rows() {
        let raw_id = positive(row.id, "AiGraph.id")?;
        require_identity(identities, raw_id, IdentityKind::Other, mode)?;
        let id = AiGraphId::new(raw_id).expect("positive AI graph ID");
        let mut states = config
            .ai_state()
            .ordered_rows()
            .filter(|state| state.graph_id == row.id)
            .map(|state| lower_ai_state(config, state))
            .collect::<Result<Vec<_>, _>>()?;
        states.sort_unstable_by_key(AiStateDefinition::id);
        let initial = ai_state_id(row.initial_state_id, "AiGraph.initial_state_id")?;
        let budget = positive_u16(
            row.automatic_transition_budget,
            "AiGraph.automatic_transition_budget",
        )?;
        let graph = AiGraphDefinition::new(id, initial, budget, states).ok_or_else(|| {
            domain_fail(format!("AI graph {} has invalid state topology", row.id))
        })?;
        validate_ai_graph_semantics(&graph)?;
        graphs.push(graph);
    }
    graphs.sort_unstable_by_key(AiGraphDefinition::id);
    Ok(graphs)
}

fn lower_ai_state(
    config: &SoraConfig,
    row: &generated::ai_state::AiState,
) -> Result<AiStateDefinition, CatalogLoadError> {
    let id = ai_state_id(row.id, "AiState.id")?;
    let mut candidate_rows = config
        .ai_candidate()
        .ordered_rows()
        .filter(|candidate| candidate.state_id == row.id)
        .collect::<Vec<_>>();
    candidate_rows.sort_unstable_by_key(|candidate| candidate.sequence);
    contiguous_i32(
        candidate_rows.iter().map(|candidate| candidate.sequence),
        "AI candidates",
    )?;
    let candidates = candidate_rows
        .into_iter()
        .map(|candidate| lower_ai_candidate(config, candidate, id))
        .collect::<Result<Vec<_>, _>>()?;

    let mut transition_rows = config
        .ai_transition()
        .ordered_rows()
        .filter(|transition| transition.state_id == row.id)
        .collect::<Vec<_>>();
    transition_rows.sort_unstable_by_key(|transition| transition.sequence);
    contiguous_i32(
        transition_rows.iter().map(|transition| transition.sequence),
        "AI transitions",
    )?;
    let transitions = transition_rows
        .into_iter()
        .map(|transition| lower_ai_transition(config, transition))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(AiStateDefinition::new(
        id,
        optional_program(row.entry_program_id)?,
        ability_id(
            row.mandatory_fallback_ability_id,
            "AiState.mandatory_fallback_ability_id",
        )?,
        row.turn_counter_reset,
        candidates,
        transitions,
    ))
}

fn lower_ai_candidate(
    config: &SoraConfig,
    row: &generated::ai_candidate::AiCandidate,
    current_state: AiStateId,
) -> Result<AiCandidateDefinition, CatalogLoadError> {
    use generated::{
        ai_candidate_selection::AiCandidateSelection as Selection,
        ai_no_target_fallback::AiNoTargetFallback as Fallback,
    };
    let selection = match row.selection {
        Selection::FirstLegal => {
            if row.weight_expression_id.is_some() || row.rng_purpose_key.is_some() {
                return Err(domain_fail(
                    "first-legal AI candidate has weighted-draw fields",
                ));
            }
            AiCandidateSelection::FirstLegal
        }
        Selection::WeightedDraw => {
            let expression = row
                .weight_expression_id
                .ok_or_else(|| domain_fail("weighted AI candidate lacks a weight expression"))?;
            let purpose = row
                .rng_purpose_key
                .as_deref()
                .ok_or_else(|| domain_fail("weighted AI candidate lacks an RNG purpose"))?;
            if purpose != "enemy-behavior" {
                return Err(domain_fail("unknown enemy-behavior RNG purpose key"));
            }
            AiCandidateSelection::WeightedDraw {
                weight: constant_weight(config, expression)?,
                purpose: starclock_combat::rng::types::DrawPurpose::BEHAVIOR_CHOICE,
            }
        }
    };
    let no_target = match row.no_target_fallback {
        Fallback::UseFallbackAbility => AiNoTargetFallback::UseFallbackAbility(ability_id(
            row.fallback_ability_id
                .ok_or_else(|| domain_fail("AI fallback policy lacks an ability"))?,
            "AiCandidate.fallback_ability_id",
        )?),
        Fallback::StayInState => {
            reject_fallback_ability(row)?;
            AiNoTargetFallback::StayInState
        }
        Fallback::Transition => {
            reject_fallback_ability(row)?;
            AiNoTargetFallback::Transition(current_state)
        }
        Fallback::SkipAction => {
            reject_fallback_ability(row)?;
            AiNoTargetFallback::SkipAction
        }
        Fallback::Fault => {
            reject_fallback_ability(row)?;
            AiNoTargetFallback::Fault
        }
    };
    Ok(AiCandidateDefinition::new(
        AiCandidateId::new(positive(row.id, "AiCandidate.id")?).expect("positive candidate ID"),
        ability_id(row.ability_id, "AiCandidate.ability_id")?,
        crate::rule_lower::lower_condition(config, row.condition_id, &mut BTreeSet::new())?,
        SelectorId::new(positive(
            row.target_selector_id,
            "AiCandidate.target_selector_id",
        )?)
        .expect("positive selector ID"),
        row.priority,
        selection,
        no_target,
    ))
}

fn reject_fallback_ability(
    row: &generated::ai_candidate::AiCandidate,
) -> Result<(), CatalogLoadError> {
    if row.fallback_ability_id.is_some() {
        Err(domain_fail(
            "AI no-target policy has an extraneous fallback ability",
        ))
    } else {
        Ok(())
    }
}

fn constant_weight(config: &SoraConfig, id: i32) -> Result<u32, CatalogLoadError> {
    use generated::value_expression_node::ValueExpressionNode as Value;
    let expression = config
        .value_expression()
        .get(&id)
        .ok_or_else(|| domain_fail(format!("missing AI weight expression {id}")))?;
    let Value::IntegerLiteral { value } = &expression.node else {
        return Err(domain_fail("AI weight must be a positive integer literal"));
    };
    if *value <= 0 {
        return Err(domain_fail("AI weight must be a positive integer literal"));
    }
    u32::try_from(*value).map_err(|_| domain_fail("AI weight exceeds u32"))
}

fn lower_ai_transition(
    config: &SoraConfig,
    row: &generated::ai_transition::AiTransition,
) -> Result<AiTransitionDefinition, CatalogLoadError> {
    use generated::ai_transition_timing::AiTransitionTiming as Timing;
    let timing = match row.timing {
        Timing::AutomaticBeforeDecision => AiTransitionTiming::AutomaticBeforeDecision,
        Timing::AfterAction => AiTransitionTiming::AfterAction,
        Timing::AfterPhase => AiTransitionTiming::AfterPhase,
        Timing::Explicit => AiTransitionTiming::Explicit,
    };
    Ok(AiTransitionDefinition::new(
        AiTransitionId::new(positive(row.id, "AiTransition.id")?).expect("positive transition ID"),
        ai_state_id(row.target_state_id, "AiTransition.target_state_id")?,
        crate::rule_lower::lower_condition(config, row.condition_id, &mut BTreeSet::new())?,
        row.priority,
        timing,
    ))
}

fn validate_ai_graph_semantics(graph: &AiGraphDefinition) -> Result<(), CatalogLoadError> {
    for state in graph.states() {
        if state.candidates().is_empty() {
            return Err(domain_fail(format!(
                "AI state {} has no action candidate",
                state.id().get()
            )));
        }
        for transition in state.transitions() {
            if graph.state(transition.target()).is_none() {
                return Err(domain_fail(format!(
                    "AI transition {} targets a state outside graph {}",
                    transition.id().get(),
                    graph.id().get()
                )));
            }
        }
    }
    let mut reachable = BTreeSet::from([graph.initial_state()]);
    loop {
        let previous = reachable.len();
        let targets = graph
            .states()
            .iter()
            .filter(|state| reachable.contains(&state.id()))
            .flat_map(|state| {
                state
                    .transitions()
                    .iter()
                    .map(AiTransitionDefinition::target)
            })
            .collect::<Vec<_>>();
        reachable.extend(targets);
        if reachable.len() == previous {
            break;
        }
    }
    if reachable.len() != graph.states().len() {
        return Err(domain_fail(format!(
            "AI graph {} has unreachable states",
            graph.id().get()
        )));
    }
    Ok(())
}

fn validate_enemy_rows(
    config: &SoraConfig,
    mode: LoadMode,
    identities: &BTreeMap<u32, &IdentityDefinition>,
    combat: &CombatDefinitions,
) -> Result<(), CatalogLoadError> {
    for row in config.enemy_template().ordered_rows() {
        let id = positive(row.id, "EnemyTemplate.id")?;
        require_identity(identities, id, IdentityKind::Other, mode)?;
        non_negative_decimal(&row.base_aggro_decimal, "enemy base aggro")?;
        if config.ai_graph().get(&row.default_ai_graph_id).is_none() {
            return Err(domain_fail(
                "enemy template refers to a missing default AI graph",
            ));
        }
    }
    for row in config.enemy_stat().iter() {
        for (value, name, positive_required) in [
            (&row.hp_decimal, "enemy HP", true),
            (&row.atk_decimal, "enemy ATK", true),
            (&row.def_decimal, "enemy DEF", true),
            (&row.spd_decimal, "enemy SPD", true),
            (
                &row.effect_resistance_decimal,
                "enemy effect resistance",
                false,
            ),
            (&row.crit_damage_decimal, "enemy critical damage", false),
        ] {
            let scaled = non_negative_decimal(value, name)?;
            if positive_required && scaled == 0 {
                return Err(domain_fail(format!("{name} must be positive")));
            }
        }
    }
    for row in config.enemy_ability().ordered_rows() {
        let id = ability_id(row.id, "EnemyAbility.id")?;
        if combat.ability_level_cap(id).is_none() {
            return Err(domain_fail(
                "enemy ability wrapper refers to a missing ability",
            ));
        }
        if row
            .fallback_ability_id
            .is_some_and(|fallback| config.enemy_ability().get(&fallback).is_none())
        {
            return Err(domain_fail(
                "enemy ability refers to a missing fallback ability",
            ));
        }
    }
    Ok(())
}

fn lower_enemies(
    config: &SoraConfig,
    mode: LoadMode,
    identities: &BTreeMap<u32, &IdentityDefinition>,
) -> Result<Vec<EnemyDefinition>, CatalogLoadError> {
    let mut enemies = Vec::new();
    for row in config.enemy_variant().ordered_rows() {
        let raw_id = positive(row.id, "EnemyVariant.id")?;
        require_identity(identities, raw_id, IdentityKind::Other, mode)?;
        if config.enemy_template().get(&row.template_id).is_none()
            || config.ai_graph().get(&row.ai_graph_id).is_none()
        {
            return Err(domain_fail(
                "enemy variant has a missing template or AI graph",
            ));
        }
        validate_variant_observations(config, row.id)?;
        let abilities = ordered_variant_abilities(config, row.id)?;
        let phases = ordered_phases(config, row.id)?;
        let links = ordered_links(config, row.id)?;
        let id = EnemyDefinitionId::new(raw_id).expect("positive enemy ID");
        let enemy = EnemyDefinition::new(
            id,
            UnitDefinitionId::new(raw_id).expect("positive unit ID"),
            abilities,
        )
        .with_orchestration(
            AiGraphId::new(positive(row.ai_graph_id, "EnemyVariant.ai_graph_id")?)
                .expect("positive graph ID"),
            phases,
        )
        .ok_or_else(|| domain_fail(format!("enemy variant {} has invalid phases", row.id)))?
        .with_links(links)
        .ok_or_else(|| domain_fail(format!("enemy variant {} has invalid links", row.id)))?;
        enemies.push(enemy);
    }
    enemies.sort_unstable_by_key(EnemyDefinition::id);
    Ok(enemies)
}

fn validate_variant_observations(
    config: &SoraConfig,
    variant: i32,
) -> Result<(), CatalogLoadError> {
    if !config
        .enemy_stat()
        .iter()
        .any(|row| row.variant_id == variant)
    {
        return Err(domain_fail(format!(
            "enemy variant {variant} has no stat row"
        )));
    }
    let mut weaknesses = config
        .enemy_weakness()
        .iter()
        .filter(|row| row.variant_id == variant)
        .collect::<Vec<_>>();
    weaknesses.sort_unstable_by_key(|row| row.sequence);
    contiguous_i32(
        weaknesses.iter().map(|row| row.sequence),
        "enemy weaknesses",
    )?;
    if weaknesses.is_empty() {
        return Err(domain_fail(format!(
            "enemy variant {variant} has no weakness"
        )));
    }
    let mut layers = config
        .enemy_toughness_layer()
        .iter()
        .filter(|row| row.variant_id == variant)
        .collect::<Vec<_>>();
    layers.sort_unstable_by_key(|row| row.sequence);
    contiguous_i32(
        layers.iter().map(|row| row.sequence),
        "enemy Toughness layers",
    )?;
    if layers.is_empty() || !layers.iter().any(|row| row.active_at_start) {
        return Err(domain_fail(format!(
            "enemy variant {variant} has no active Toughness layer"
        )));
    }
    for row in layers {
        if non_negative_decimal(&row.maximum_decimal, "Toughness maximum")? == 0 {
            return Err(domain_fail("Toughness maximum must be positive"));
        }
        let recovery = non_negative_decimal(&row.recovery_ratio_decimal, "Toughness recovery")?;
        if recovery > 1_000_000 {
            return Err(domain_fail("Toughness recovery ratio exceeds one"));
        }
    }
    for row in config
        .enemy_resistance()
        .iter()
        .filter(|row| row.variant_id == variant)
    {
        if !(0..=1_000_000).contains(&parse_decimal(&row.value_decimal)?) {
            return Err(domain_fail(
                "enemy elemental resistance is outside zero through one",
            ));
        }
    }
    for row in config
        .enemy_debuff_resistance()
        .iter()
        .filter(|row| row.variant_id == variant)
    {
        if row.category_key.trim().is_empty()
            || !(0..=1_000_000).contains(&parse_decimal(&row.value_decimal)?)
        {
            return Err(domain_fail("enemy debuff resistance is invalid"));
        }
    }
    Ok(())
}

fn ordered_variant_abilities(
    config: &SoraConfig,
    variant: i32,
) -> Result<Vec<AbilityId>, CatalogLoadError> {
    let mut rows = config
        .enemy_variant_ability()
        .iter()
        .filter(|row| row.variant_id == variant)
        .collect::<Vec<_>>();
    rows.sort_unstable_by_key(|row| row.sequence);
    contiguous_i32(rows.iter().map(|row| row.sequence), "enemy abilities")?;
    if rows.is_empty() {
        return Err(domain_fail(format!(
            "enemy variant {variant} has no ability"
        )));
    }
    let mut abilities = rows
        .into_iter()
        .map(|row| ability_id(row.ability_id, "EnemyVariantAbility.ability_id"))
        .collect::<Result<Vec<_>, _>>()?;
    abilities.sort_unstable();
    abilities.dedup();
    Ok(abilities)
}

fn ordered_phases(
    config: &SoraConfig,
    variant: i32,
) -> Result<Vec<EnemyPhaseDefinition>, CatalogLoadError> {
    let mut rows = config
        .enemy_phase()
        .ordered_rows()
        .filter(|row| row.variant_id == variant)
        .collect::<Vec<_>>();
    rows.sort_unstable_by_key(|row| row.sequence);
    contiguous_i32(rows.iter().map(|row| row.sequence), "enemy phases")?;
    rows.into_iter()
        .map(|row| lower_phase(config, row))
        .collect()
}

fn lower_phase(
    config: &SoraConfig,
    row: &generated::enemy_phase::EnemyPhase,
) -> Result<EnemyPhaseDefinition, CatalogLoadError> {
    use generated::enemy_phase_transition_model::EnemyPhaseTransitionModel as Transition;
    let transition = match row.transition_model {
        Transition::TransformSameUnit => EnemyPhaseTransitionModel::TransformSameUnit,
        Transition::ExplicitWave => EnemyPhaseTransitionModel::ExplicitWave,
        Transition::ReplaceLinkedVariant => {
            return Err(domain_fail(
                "ReplaceLinkedVariant requires an explicit target absent from schema v1",
            ));
        }
    };
    let explicit = optional_program(row.entry_program_id)?;
    let carry = EnemyPhaseCarry {
        hp: phase_carry(row.hp_carry, explicit)?,
        action_gauge: phase_carry(row.action_gauge_carry, explicit)?,
        effects: phase_carry(row.effect_carry, explicit)?,
        toughness: phase_carry(row.toughness_carry, explicit)?,
        summons: phase_carry(row.summon_carry, explicit)?,
    };
    Ok(EnemyPhaseDefinition::new(
        EnemyPhaseId::new(positive(row.id, "EnemyPhase.id")?).expect("positive phase ID"),
        positive_u16(row.sequence, "EnemyPhase.sequence")?,
        crate::rule_lower::lower_condition(config, row.entry_condition_id, &mut BTreeSet::new())?,
        crate::rule_lower::lower_condition(config, row.exit_condition_id, &mut BTreeSet::new())?,
        row.replacement_priority,
        AiGraphId::new(positive(row.ai_graph_id, "EnemyPhase.ai_graph_id")?)
            .expect("positive AI graph ID"),
        row.targetable,
        transition,
        explicit,
        carry,
    ))
}

fn phase_carry(
    value: generated::phase_carry_policy::PhaseCarryPolicy,
    explicit: Option<ProgramId>,
) -> Result<PhaseCarryPolicy, CatalogLoadError> {
    use generated::phase_carry_policy::PhaseCarryPolicy as Carry;
    Ok(match value {
        Carry::CarryExact => PhaseCarryPolicy::CarryExact,
        Carry::CarryRatio => PhaseCarryPolicy::CarryRatio,
        Carry::Reset => PhaseCarryPolicy::Reset,
        Carry::Clear => PhaseCarryPolicy::Clear,
        Carry::ExplicitProgram => PhaseCarryPolicy::ExplicitProgram(
            explicit
                .ok_or_else(|| domain_fail("explicit phase carry requires entry_program_id"))?,
        ),
    })
}

fn ordered_links(
    config: &SoraConfig,
    variant: i32,
) -> Result<Vec<EnemyLinkDefinition>, CatalogLoadError> {
    let mut rows = config
        .enemy_link()
        .iter()
        .filter(|row| row.owner_variant_id == variant)
        .collect::<Vec<_>>();
    rows.sort_unstable_by_key(|row| row.sequence);
    contiguous_i32(rows.iter().map(|row| row.sequence), "enemy links")?;
    rows.into_iter().map(lower_link).collect()
}

fn lower_link(
    row: &generated::enemy_link::EnemyLink,
) -> Result<EnemyLinkDefinition, CatalogLoadError> {
    use generated::{
        enemy_link_kind::EnemyLinkKind as Kind,
        link_overflow_policy::LinkOverflowPolicy as Overflow,
        link_wave_persistence::LinkWavePersistence as Wave,
        linked_formation_policy::LinkedFormationPolicy as Formation,
        owner_defeat_policy::OwnerDefeatPolicy as Owner,
    };
    let kind = match row.kind {
        Kind::Summon => EnemyLinkKind::Summon,
        Kind::SharedHp => EnemyLinkKind::SharedHp,
        Kind::Part => EnemyLinkKind::Part,
        Kind::Countdown => EnemyLinkKind::Countdown,
        Kind::TimelineActor => EnemyLinkKind::TimelineActor,
    };
    let overflow = match row.overflow_policy {
        Overflow::Reject => LinkOverflowPolicy::Reject,
        Overflow::ReplaceOldest => LinkOverflowPolicy::ReplaceOldest,
        Overflow::ReplaceNewest => LinkOverflowPolicy::ReplaceNewest,
        Overflow::Skip => LinkOverflowPolicy::Skip,
    };
    let owner = match row.owner_defeat_policy {
        Owner::Despawn | Owner::Depart => starclock_combat::OwnerLinkPolicy::Depart,
        Owner::Defeat => starclock_combat::OwnerLinkPolicy::Defeat,
        Owner::Persist => starclock_combat::OwnerLinkPolicy::Persist,
        Owner::Transfer => {
            return Err(domain_fail(
                "enemy link transfer is not representable in Goal 01",
            ));
        }
    };
    let wave = match row.wave_persistence {
        Wave::WaveOwned => starclock_combat::WaveLinkPolicy::Depart,
        Wave::EncounterOwned => starclock_combat::WaveLinkPolicy::Persist,
        Wave::Explicit => {
            return Err(domain_fail(
                "explicit link wave policy lacks a program reference",
            ));
        }
    };
    if parse_decimal(&row.initial_action_gauge_decimal)? != 0 {
        return Err(domain_fail(
            "nonzero linked initial Action Gauge is not yet representable",
        ));
    }
    let formation = match row.formation_policy {
        Formation::NextAvailable => {
            reject_fixed_formation(row)?;
            LinkedFormationPolicy::NextAvailable
        }
        Formation::NoFormationSlot => {
            reject_fixed_formation(row)?;
            LinkedFormationPolicy::NoFormationSlot
        }
        Formation::FixedSlot => LinkedFormationPolicy::Fixed(formation_index(
            row.fixed_formation_index
                .ok_or_else(|| domain_fail("fixed link lacks formation index"))?,
            "EnemyLink.fixed_formation_index",
        )?),
    };
    EnemyLinkDefinition::new(
        positive_u16(row.sequence, "EnemyLink.sequence")?,
        EnemyDefinitionId::new(positive(
            row.linked_variant_id,
            "EnemyLink.linked_variant_id",
        )?)
        .expect("positive linked enemy ID"),
        kind,
        positive_u16(row.maximum_simultaneous, "EnemyLink.maximum_simultaneous")?,
        overflow,
        owner,
        wave,
        row.contributes_to_victory,
        formation,
    )
    .ok_or_else(|| domain_fail("invalid enemy link bounds"))
}

fn reject_fixed_formation(row: &generated::enemy_link::EnemyLink) -> Result<(), CatalogLoadError> {
    if row.fixed_formation_index.is_some() {
        Err(domain_fail(
            "non-fixed link has an extraneous formation index",
        ))
    } else {
        Ok(())
    }
}

fn lower_encounters(
    config: &SoraConfig,
    mode: LoadMode,
    identities: &BTreeMap<u32, &IdentityDefinition>,
    enemies: &[EnemyDefinition],
) -> Result<Vec<EncounterDefinition>, CatalogLoadError> {
    let mut encounters = Vec::new();
    for row in config.encounter().ordered_rows() {
        use generated::{
            encounter_loss_policy::EncounterLossPolicy as Loss,
            encounter_victory_policy::EncounterVictoryPolicy as Victory,
        };
        let raw_id = positive(row.id, "Encounter.id")?;
        require_identity(identities, raw_id, IdentityKind::Other, mode)?;
        if row.difficulty_key.trim().is_empty() || row.environment_key.trim().is_empty() {
            return Err(domain_fail("encounter difficulty/environment key is empty"));
        }
        if !matches!(row.victory_policy, Victory::DefeatRequiredHostiles)
            || !matches!(row.loss_policy, Loss::NoControllableAllies)
        {
            return Err(domain_fail(
                "Goal 01 Standard requires ordinary encounter terminal policies",
            ));
        }
        if row.initial_skill_points > row.maximum_skill_points {
            return Err(domain_fail("encounter initial Skill Points exceed maximum"));
        }
        let mut rule_rows = config
            .encounter_rule_binding()
            .iter()
            .filter(|binding| binding.encounter_id == row.id)
            .collect::<Vec<_>>();
        rule_rows.sort_unstable_by_key(|binding| binding.sequence);
        contiguous_i32(
            rule_rows.iter().map(|binding| binding.sequence),
            "encounter rules",
        )?;
        let rules = rule_rows
            .into_iter()
            .map(|binding| {
                RuleBundleId::new(positive(binding.rule_id, "EncounterRuleBinding.rule_id")?)
                    .ok_or_else(|| domain_fail("zero encounter rule ID"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let encounter = EncounterDefinition::new(
            EncounterId::new(raw_id).expect("positive encounter ID"),
            Vec::new(),
            rules,
        )
        .with_authored_waves(
            wave_transition(row.wave_transition),
            ordered_waves(config, row.id, enemies)?,
        )
        .ok_or_else(|| domain_fail(format!("encounter {} has invalid waves", row.id)))?;
        encounters.push(encounter);
    }
    encounters.sort_unstable_by_key(EncounterDefinition::id);
    Ok(encounters)
}

fn ordered_waves(
    config: &SoraConfig,
    encounter: i32,
    enemies: &[EnemyDefinition],
) -> Result<Vec<EncounterWaveDefinition>, CatalogLoadError> {
    let mut rows = config
        .encounter_wave()
        .ordered_rows()
        .filter(|row| row.encounter_id == encounter)
        .collect::<Vec<_>>();
    rows.sort_unstable_by_key(|row| row.sequence);
    contiguous_i32(rows.iter().map(|row| row.sequence), "encounter waves")?;
    rows.into_iter()
        .map(|row| lower_wave(config, row, enemies))
        .collect()
}

fn lower_wave(
    config: &SoraConfig,
    row: &generated::encounter_wave::EncounterWave,
    enemies: &[EnemyDefinition],
) -> Result<EncounterWaveDefinition, CatalogLoadError> {
    let entry = optional_program(row.entry_program_id)?;
    let exit = optional_program(row.exit_program_id)?;
    let carry = WaveCarry {
        hp: wave_carry(row.hp_carry, exit)?,
        energy: wave_carry(row.energy_carry, exit)?,
        skill_points: wave_carry(row.skill_point_carry, exit)?,
        effects: wave_carry(row.effect_carry, exit)?,
        action_gauge: wave_carry(row.action_gauge_carry, exit)?,
    };
    let mut slot_rows = config
        .wave_slot()
        .iter()
        .filter(|slot| slot.wave_id == row.id)
        .collect::<Vec<_>>();
    slot_rows.sort_unstable_by_key(|slot| slot.spawn_sequence);
    contiguous_i32(
        slot_rows.iter().map(|slot| slot.spawn_sequence),
        "wave slots",
    )?;
    let slots = slot_rows
        .into_iter()
        .map(|slot| {
            let enemy = EnemyDefinitionId::new(positive(
                slot.enemy_variant_id,
                "WaveSlot.enemy_variant_id",
            )?)
            .expect("positive enemy ID");
            if lookup(enemies, enemy, EnemyDefinition::id).is_none() {
                return Err(domain_fail("wave slot refers to a missing lowered enemy"));
            }
            WaveSlotDefinition::new(
                positive_u16(slot.spawn_sequence, "WaveSlot.spawn_sequence")?,
                formation_index(slot.formation_index, "WaveSlot.formation_index")?,
                enemy,
                slot.level_override
                    .map(|value| {
                        u8::try_from(value)
                            .map_err(|_| domain_fail("wave level override exceeds u8"))
                    })
                    .transpose()?,
                slot.initial_phase_id
                    .map(|value| {
                        EnemyPhaseId::new(positive(value, "WaveSlot.initial_phase_id")?)
                            .ok_or_else(|| domain_fail("zero initial phase ID"))
                    })
                    .transpose()?,
                slot.required_for_victory,
            )
            .ok_or_else(|| domain_fail("invalid wave slot bounds"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    EncounterWaveDefinition::new(
        EncounterWaveId::new(positive(row.id, "EncounterWave.id")?).expect("positive wave ID"),
        positive_u16(row.sequence, "EncounterWave.sequence")?,
        entry,
        exit,
        carry,
        slots,
    )
    .ok_or_else(|| domain_fail("invalid encounter wave"))
}

fn wave_carry(
    value: generated::wave_carry_policy::WaveCarryPolicy,
    explicit: Option<ProgramId>,
) -> Result<WaveCarryPolicy, CatalogLoadError> {
    use generated::wave_carry_policy::WaveCarryPolicy as Carry;
    Ok(match value {
        Carry::CarryExact => WaveCarryPolicy::CarryExact,
        Carry::Reset => WaveCarryPolicy::Reset,
        Carry::Clear => WaveCarryPolicy::Clear,
        Carry::ExplicitProgram => WaveCarryPolicy::ExplicitProgram(
            explicit.ok_or_else(|| domain_fail("explicit wave carry requires exit_program_id"))?,
        ),
    })
}

pub(super) fn wave_transition(
    value: generated::wave_transition_policy::WaveTransitionPolicy,
) -> WaveTransitionPolicy {
    use generated::wave_transition_policy::WaveTransitionPolicy as Transition;
    match value {
        Transition::AfterAction => WaveTransitionPolicy::AfterAction,
        Transition::AfterPhase => WaveTransitionPolicy::AfterPhase,
        Transition::AfterHit => WaveTransitionPolicy::AfterHit,
        Transition::Explicit => WaveTransitionPolicy::Explicit,
    }
}

fn ability_id(value: i32, field: &str) -> Result<AbilityId, CatalogLoadError> {
    Ok(AbilityId::new(positive(value, field)?).expect("positive ability ID"))
}

fn ai_state_id(value: i32, field: &str) -> Result<AiStateId, CatalogLoadError> {
    Ok(AiStateId::new(positive(value, field)?).expect("positive AI state ID"))
}

fn optional_program(value: Option<i32>) -> Result<Option<ProgramId>, CatalogLoadError> {
    value
        .map(|value| {
            ProgramId::new(positive(value, "program reference")?)
                .ok_or_else(|| domain_fail("zero program reference"))
        })
        .transpose()
}

fn formation_index(value: i32, field: &str) -> Result<FormationIndex, CatalogLoadError> {
    let value = u8::try_from(value).map_err(|_| domain_fail(format!("{field} exceeds u8")))?;
    FormationIndex::new(value).ok_or_else(|| domain_fail(format!("{field} exceeds 31")))
}

fn non_negative_decimal(value: &str, field: &str) -> Result<i64, CatalogLoadError> {
    let value = parse_decimal(value)?;
    if value < 0 {
        Err(domain_fail(format!("{field} is negative")))
    } else {
        Ok(value)
    }
}

fn contiguous_i32(
    values: impl Iterator<Item = i32>,
    description: &str,
) -> Result<(), CatalogLoadError> {
    let values = values
        .map(|value| positive_u16(value, description))
        .collect::<Result<Vec<_>, _>>()?;
    contiguous(values.into_iter(), description)
}
