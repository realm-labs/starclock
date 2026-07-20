//! Minimum one-Battle Activity and ordinary Standard row lowering.

use std::collections::{BTreeMap, BTreeSet};

use starclock_activity::ActivityDefinitionId;
use starclock_mode_standard::{
    StandardBindingId, StandardExpectedOutcome, StandardProfile, StandardProfileId,
    StandardScenarioId,
};

use crate::{
    catalog::{
        CatalogLoadError, IdentityDefinition, IdentityKind, LoadMode, StandardScenarioDefinition,
        domain_fail, positive, require_identity,
    },
    encounter_lower::EncounterDefinitions,
    generated::{self, SoraConfig},
};

#[derive(Debug)]
pub(super) struct StandardDefinitions {
    pub(super) profiles: Box<[StandardProfile]>,
    pub(super) scenarios: Box<[StandardScenarioDefinition]>,
}

impl StandardDefinitions {
    pub(super) fn profile(&self, id: StandardProfileId) -> Option<StandardProfile> {
        self.profiles
            .binary_search_by_key(&id, |profile| profile.id())
            .ok()
            .map(|index| self.profiles[index])
    }

    pub(super) fn scenario(&self, id: StandardScenarioId) -> Option<&StandardScenarioDefinition> {
        self.scenarios
            .binary_search_by_key(&id, |scenario| scenario.id)
            .ok()
            .map(|index| &self.scenarios[index])
    }
}

pub(super) fn convert(
    config: &SoraConfig,
    mode: LoadMode,
    identities: &BTreeMap<u32, &IdentityDefinition>,
    encounters: &EncounterDefinitions,
) -> Result<StandardDefinitions, CatalogLoadError> {
    validate_activities(config, mode, identities, encounters)?;
    let profiles = lower_profiles(config, mode, identities)?;
    let scenarios = lower_scenarios(config, mode, identities, &profiles)?;
    Ok(StandardDefinitions {
        profiles: profiles.into_boxed_slice(),
        scenarios: scenarios.into_boxed_slice(),
    })
}

fn validate_activities(
    config: &SoraConfig,
    mode: LoadMode,
    identities: &BTreeMap<u32, &IdentityDefinition>,
    encounters: &EncounterDefinitions,
) -> Result<(), CatalogLoadError> {
    for activity in config.activity_definition().ordered_rows() {
        let raw_id = positive(activity.id, "ActivityDefinition.id")?;
        require_identity(identities, raw_id, IdentityKind::Other, mode)?;
        let sections = config
            .activity_section()
            .ordered_rows()
            .filter(|section| section.activity_id == activity.id)
            .collect::<Vec<_>>();
        if sections.len() != 1 || sections[0].sequence != 1 {
            return Err(domain_fail(
                "Goal 01 Standard Activity requires exactly one first section",
            ));
        }
        let section = sections[0];
        if section.entry_node_id != activity.entry_node_id {
            return Err(domain_fail("Activity and section entry nodes differ"));
        }
        validate_one_battle_nodes(config, activity.id, section.id)?;
        validate_activity_slots(config, activity.id, section.id)?;
        validate_participant_policy(config, activity.id)?;
        validate_bindings(config, activity.id, encounters)?;
    }
    Ok(())
}

fn validate_one_battle_nodes(
    config: &SoraConfig,
    activity_id: i32,
    section_id: i32,
) -> Result<(), CatalogLoadError> {
    use generated::{
        activity_edge_condition::ActivityEdgeCondition as Edge,
        activity_node_kind::ActivityNodeKind as Kind,
        activity_terminal_outcome::ActivityTerminalOutcome as Terminal,
    };
    let nodes = config
        .activity_node()
        .ordered_rows()
        .filter(|node| node.section_id == section_id)
        .collect::<Vec<_>>();
    let battle = nodes
        .iter()
        .filter(|node| matches!(node.kind, Kind::Battle))
        .copied()
        .collect::<Vec<_>>();
    let terminal = nodes
        .iter()
        .filter(|node| matches!(node.kind, Kind::Terminal))
        .copied()
        .collect::<Vec<_>>();
    if battle.len() != 1 || terminal.len() != 3 || nodes.len() != 4 {
        return Err(domain_fail(
            "Goal 01 Standard Activity requires one Battle and three terminal nodes",
        ));
    }
    if battle[0].terminal_outcome.is_some() || nodes.iter().any(|node| node.maximum_visits != 1) {
        return Err(domain_fail("invalid one-Battle node visit/terminal shape"));
    }
    let outcomes = terminal
        .iter()
        .map(|node| node.terminal_outcome)
        .collect::<Vec<_>>();
    if !outcomes.contains(&Some(Terminal::Complete))
        || !outcomes.contains(&Some(Terminal::Failed))
        || !outcomes.contains(&Some(Terminal::Faulted))
    {
        return Err(domain_fail("Activity terminal outcomes are incomplete"));
    }
    let mut edges = config
        .activity_edge()
        .ordered_rows()
        .filter(|edge| edge.activity_id == activity_id)
        .collect::<Vec<_>>();
    edges.sort_unstable_by_key(|edge| edge.priority);
    if edges.len() != 3
        || edges
            .iter()
            .enumerate()
            .any(|(index, edge)| edge.priority != i32::try_from(index + 1).unwrap())
        || edges.iter().any(|edge| {
            edge.source_node_id != battle[0].id
                || edge.maximum_traversals != 1
                || !terminal.iter().any(|node| node.id == edge.target_node_id)
        })
        || !edges
            .iter()
            .any(|edge| matches!(edge.condition, Edge::BattleWon))
        || !edges
            .iter()
            .any(|edge| matches!(edge.condition, Edge::BattleLost))
        || !edges
            .iter()
            .any(|edge| matches!(edge.condition, Edge::BattleFaulted))
    {
        return Err(domain_fail("invalid one-Battle Activity edge set"));
    }
    Ok(())
}

fn validate_activity_slots(
    config: &SoraConfig,
    activity_id: i32,
    section_id: i32,
) -> Result<(), CatalogLoadError> {
    use generated::rule_scope::RuleScope as Scope;
    for slot in config
        .activity_slot()
        .ordered_rows()
        .filter(|slot| slot.activity_id == activity_id)
    {
        if matches!(slot.owner_scope, Scope::Battle) {
            return Err(domain_fail("battle-scoped state belongs to combat"));
        }
        if slot.section_id.is_some_and(|id| id != section_id)
            || slot
                .node_id
                .is_some_and(|id| config.activity_node().get(&id).is_none())
            || config
                .value_expression()
                .get(&slot.initial_expression_id)
                .is_none()
            || slot
                .minimum_expression_id
                .is_some_and(|id| config.value_expression().get(&id).is_none())
            || slot
                .maximum_expression_id
                .is_some_and(|id| config.value_expression().get(&id).is_none())
        {
            return Err(domain_fail(
                "Activity slot has an invalid scope/expression link",
            ));
        }
        let mut resets = config
            .activity_slot_reset()
            .iter()
            .filter(|reset| reset.slot_id == slot.id)
            .collect::<Vec<_>>();
        resets.sort_unstable_by_key(|reset| reset.sequence);
        if resets
            .iter()
            .enumerate()
            .any(|(index, reset)| reset.sequence != i32::try_from(index + 1).unwrap())
        {
            return Err(domain_fail("Activity slot reset order is not contiguous"));
        }
    }
    Ok(())
}

fn validate_participant_policy(
    config: &SoraConfig,
    activity_id: i32,
) -> Result<(), CatalogLoadError> {
    let policies = config
        .participant_policy()
        .ordered_rows()
        .filter(|policy| policy.activity_id == activity_id)
        .collect::<Vec<_>>();
    if policies.len() != 1 {
        return Err(domain_fail(
            "Standard Activity requires exactly one participant policy",
        ));
    }
    let policy = policies[0];
    if policy.team_count != 1
        || !(1..=policy.maximum_team_size).contains(&policy.minimum_team_size)
        || policy.maximum_team_size > 4
        || policy.allow_substitution
    {
        return Err(domain_fail("invalid Standard participant policy"));
    }
    Ok(())
}

fn validate_bindings(
    config: &SoraConfig,
    activity_id: i32,
    encounters: &EncounterDefinitions,
) -> Result<(), CatalogLoadError> {
    let section_ids = config
        .activity_section()
        .ordered_rows()
        .filter(|section| section.activity_id == activity_id)
        .map(|section| section.id)
        .collect::<BTreeSet<_>>();
    let node_ids = config
        .activity_node()
        .ordered_rows()
        .filter(|node| section_ids.contains(&node.section_id))
        .map(|node| node.id)
        .collect::<BTreeSet<_>>();
    let bindings = config
        .battle_binding()
        .ordered_rows()
        .filter(|binding| node_ids.contains(&binding.node_id))
        .collect::<Vec<_>>();
    if bindings.is_empty() {
        return Err(domain_fail(
            "Standard Activity requires at least one Battle binding",
        ));
    }
    for binding in bindings {
        validate_binding(config, binding, encounters)?;
    }
    Ok(())
}

fn validate_binding(
    config: &SoraConfig,
    binding: &generated::battle_binding::BattleBinding,
    encounters: &EncounterDefinitions,
) -> Result<(), CatalogLoadError> {
    let encounter_id = starclock_combat::EncounterId::new(positive(
        binding.encounter_id,
        "BattleBinding.encounter_id",
    )?)
    .expect("positive encounter ID");
    if encounters.encounter(encounter_id).is_none()
        || config
            .participant_policy()
            .get(&binding.participant_policy_id)
            .is_none()
        || config
            .battle_result_projection()
            .get(&binding.projection_id)
            .is_none()
        || !valid_sha256(&binding.participant_lock_sha256)
        || binding.seed_stream_label.trim().is_empty()
        || binding.battle_spec_policy_revision.trim().is_empty()
    {
        return Err(domain_fail(
            "Battle binding has an invalid reference or identity",
        ));
    }
    validate_projection(config, binding.projection_id)?;
    validate_participant_slots(config, binding.id)?;
    Ok(())
}

fn validate_projection(config: &SoraConfig, projection: i32) -> Result<(), CatalogLoadError> {
    use generated::battle_result_projection_field_node::BattleResultProjectionFieldNode as Field;
    let mut rows = config
        .battle_result_projection_field()
        .iter()
        .filter(|row| row.projection_id == projection)
        .collect::<Vec<_>>();
    rows.sort_unstable_by_key(|row| row.sequence);
    let exact = rows.len() == 4
        && matches!(rows[0].field, Field::Outcome { .. })
        && matches!(rows[1].field, Field::FinalStateHash { .. })
        && matches!(rows[2].field, Field::EventDigest { .. })
        && matches!(rows[3].field, Field::TerminalFault { .. });
    if !exact
        || rows
            .iter()
            .enumerate()
            .any(|(index, row)| row.sequence != i32::try_from(index + 1).unwrap())
    {
        return Err(domain_fail(
            "Standard result projection must be the exact four core fields",
        ));
    }
    Ok(())
}

fn validate_participant_slots(config: &SoraConfig, binding: i32) -> Result<(), CatalogLoadError> {
    let slots = config
        .battle_participant_slot()
        .iter()
        .filter(|slot| slot.battle_binding_id == binding)
        .collect::<Vec<_>>();
    if slots.is_empty() || slots.len() > 4 || slots.iter().any(|slot| slot.team_index != 0) {
        return Err(domain_fail(
            "Standard binding requires one to four player slots",
        ));
    }
    let mut formations = BTreeSet::new();
    let mut characters = BTreeSet::new();
    for slot in slots {
        if !(0..=3).contains(&slot.formation_index)
            || !formations.insert(slot.formation_index)
            || !characters.insert(slot.character_id)
            || !valid_sha256(&slot.resolved_spec_sha256)
            || !valid_sha256(&slot.build_digest_sha256)
            || slot.build_catalog_revision.trim().is_empty()
        {
            return Err(domain_fail(
                "invalid or duplicate Standard participant slot",
            ));
        }
    }
    Ok(())
}

fn lower_profiles(
    config: &SoraConfig,
    mode: LoadMode,
    identities: &BTreeMap<u32, &IdentityDefinition>,
) -> Result<Vec<StandardProfile>, CatalogLoadError> {
    let mut profiles = Vec::new();
    for row in config.standard_profile().ordered_rows() {
        let raw_id = positive(row.id, "StandardProfile.id")?;
        require_identity(identities, raw_id, IdentityKind::Other, mode)?;
        if row.player_team_count != 1
            || row.has_global_clock
            || row.has_score
            || row.has_seasonal_rules
        {
            return Err(domain_fail("Standard profile contains challenge semantics"));
        }
        let maximum_party_size = u8::try_from(row.maximum_party_size)
            .map_err(|_| domain_fail("Standard maximum party size exceeds u8"))?;
        let profile = StandardProfile::new(
            StandardProfileId::new(raw_id).expect("positive profile ID"),
            ActivityDefinitionId::new(positive(row.activity_id, "StandardProfile.activity_id")?)
                .expect("positive activity ID"),
            maximum_party_size,
            crate::encounter_lower::wave_transition(row.default_wave_transition),
        )
        .ok_or_else(|| domain_fail("invalid Standard profile"))?;
        profiles.push(profile);
    }
    profiles.sort_unstable_by_key(|profile| profile.id());
    Ok(profiles)
}

fn lower_scenarios(
    config: &SoraConfig,
    mode: LoadMode,
    identities: &BTreeMap<u32, &IdentityDefinition>,
    profiles: &[StandardProfile],
) -> Result<Vec<StandardScenarioDefinition>, CatalogLoadError> {
    use generated::standard_expected_outcome::StandardExpectedOutcome as Outcome;
    let mut scenarios = Vec::new();
    for row in config.standard_scenario().ordered_rows() {
        let raw_id = positive(row.id, "StandardScenario.id")?;
        require_identity(identities, raw_id, IdentityKind::Other, mode)?;
        let profile_id =
            StandardProfileId::new(positive(row.profile_id, "StandardScenario.profile_id")?)
                .expect("positive profile ID");
        let profile = profiles
            .binary_search_by_key(&profile_id, |profile| profile.id())
            .ok()
            .map(|index| profiles[index])
            .ok_or_else(|| domain_fail("Standard scenario refers to a missing profile"))?;
        let activity =
            ActivityDefinitionId::new(positive(row.activity_id, "StandardScenario.activity_id")?)
                .expect("positive activity ID");
        if profile.activity() != activity
            || config
                .battle_binding()
                .get(&row.battle_binding_id)
                .is_none()
        {
            return Err(domain_fail("Standard scenario binding/profile mismatch"));
        }
        let master_seed = parse_seed(&row.master_seed_hex)?;
        let expected_outcome = match row.expected_outcome {
            Outcome::Won => StandardExpectedOutcome::Won,
            Outcome::Lost => StandardExpectedOutcome::Lost,
            Outcome::Faulted => StandardExpectedOutcome::Faulted,
        };
        scenarios.push(StandardScenarioDefinition {
            id: StandardScenarioId::new(raw_id).expect("positive scenario ID"),
            profile: profile_id,
            activity,
            binding: StandardBindingId::new(positive(
                row.battle_binding_id,
                "StandardScenario.battle_binding_id",
            )?)
            .expect("positive binding ID"),
            master_seed,
            expected_outcome,
        });
    }
    scenarios.sort_unstable_by_key(|scenario| scenario.id);
    Ok(scenarios)
}

fn parse_seed(value: &str) -> Result<u64, CatalogLoadError> {
    if value.len() != 16 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(domain_fail(
            "Standard master seed is not exactly 16 hex digits",
        ));
    }
    u64::from_str_radix(value, 16).map_err(|_| domain_fail("invalid Standard master seed"))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
