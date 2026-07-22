//! Strict lowering from private Sora transport rows into Universe definitions.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use crate::definition::{
    ActivityDomainBinding, ActivityDomainDecision, DifficultyDefinition, DifficultyKind,
    DomainDecisionPolicy, DomainDefinition, DomainKind, LocalizedText, RecommendedElement,
    RoomDefinition, ScoreThreshold, TopologyDefinition, TopologyNodeDefinition,
    UniverseActivityBindingDefinition, UniverseDefinitions, UniverseProfileDefinition,
    WorldDefinition,
};
use crate::digest::{Encoder, UniverseDefinitionsDigest};
use crate::error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind};
use crate::generated::{
    SoraConfig, universe_activity_decision::UniverseActivityDecision,
    universe_decision_policy::UniverseDecisionPolicy,
    universe_difficulty_kind::UniverseDifficultyKind, universe_domain_kind::UniverseDomainKind,
    universe_map_edge::UniverseMapEdge, universe_map_node::UniverseMapNode,
};
use crate::id::{
    ActivityBindingId, DifficultyId, DomainId, RoomId, TopologyId, TopologyNodeId,
    UniverseProfileId, WorldId,
};

const EXPECTED_ACTIVITY_KEY: &str = "activity.standard-simulated-universe.v1";
const EXPECTED_BINDING_KEY: &str = "universe.activity-binding.standard-main-world.v1";
const EXPECTED_BATTLE_HANDOFF: &str = "activity.battle-handoff.rule-bundle.v1";
const EXPECTED_EXTERNAL_OUTCOME: &str = "activity.external-outcome.command.v1";
const EXPECTED_WORLD_ENTRY_RULE: &str = "universe.rule.run-entry.standard";
const EXPECTED_WORLD_TERMINAL_RULE: &str = "universe.rule.run-terminal.standard";

pub(crate) fn lower(config: &SoraConfig) -> Result<UniverseDefinitions, UniverseCatalogLoadError> {
    let profile = lower_profile(config)?;
    let domains = lower_domains(config)?;
    let difficulties = lower_difficulties(config)?;
    let worlds = lower_worlds(config, profile.id(), &difficulties)?;
    let topologies = lower_topologies(config)?;
    let rooms = lower_rooms(config, &domains)?;
    let activity = lower_activity(config, profile.id(), &domains)?;
    let mut definitions = UniverseDefinitions {
        digest: UniverseDefinitionsDigest::new([0; 32]),
        profile,
        worlds,
        difficulties,
        domains,
        topologies,
        rooms,
        activity,
    };
    definitions.digest = digest(&definitions);
    Ok(definitions)
}

fn lower_profile(
    config: &SoraConfig,
) -> Result<UniverseProfileDefinition, UniverseCatalogLoadError> {
    let mut rows = config.universe_profile().ordered_rows();
    let row = rows
        .next()
        .ok_or_else(|| invalid("Universe profile is absent"))?;
    if rows.next().is_some() {
        return Err(invalid("Universe profile is not singular"));
    }
    Ok(UniverseProfileDefinition::new(
        id::<UniverseProfileId>(row.id, "profile")?,
        checked_key(&row.stable_key, "profile stable key")?,
        checked_token(&row.game_version, "game version")?,
        checked_token(&row.snapshot_date, "snapshot date")?,
        parse_digest(&row.content_manifest_sha256, "content manifest digest")?,
        parse_digest(&row.pack_sha256, "normalized pack digest")?,
    ))
}

fn lower_domains(config: &SoraConfig) -> Result<Box<[DomainDefinition]>, UniverseCatalogLoadError> {
    let mut source_types = BTreeSet::new();
    let mut definitions = Vec::with_capacity(config.universe_domain().len());
    for row in config.universe_domain().ordered_rows() {
        let source_type = positive_u32(row.source_type, "domain source type")?;
        if !source_types.insert(source_type) {
            return Err(invalid("domain source type is duplicated"));
        }
        definitions.push(DomainDefinition::new(
            id::<DomainId>(row.id, "domain")?,
            checked_key(&row.stable_key, "domain stable key")?,
            source_type,
            match row.kind {
                UniverseDomainKind::CombatPrimary => DomainKind::CombatPrimary,
                UniverseDomainKind::CombatSecondary => DomainKind::CombatSecondary,
                UniverseDomainKind::Occurrence => DomainKind::Occurrence,
                UniverseDomainKind::Encounter => DomainKind::Encounter,
                UniverseDomainKind::Respite => DomainKind::Respite,
                UniverseDomainKind::Elite => DomainKind::Elite,
                UniverseDomainKind::Boss => DomainKind::Boss,
                UniverseDomainKind::Transaction => DomainKind::Transaction,
                UniverseDomainKind::Adventure => DomainKind::Adventure,
            },
            match row.decision_policy {
                UniverseDecisionPolicy::BattleHandoff => DomainDecisionPolicy::BattleHandoff,
                UniverseDecisionPolicy::ExternalCommand => DomainDecisionPolicy::ExternalCommand,
            },
            row.terminal,
            localized(
                &row.name_en,
                &row.name_zh_cn,
                &row.summary_en,
                &row.summary_zh_cn,
                "domain",
            )?,
        ));
    }
    definitions.sort_by_key(DomainDefinition::id);
    if definitions.len() != 9 {
        return Err(invalid(
            "Standard Universe must define exactly nine domain kinds",
        ));
    }
    Ok(definitions.into_boxed_slice())
}

fn lower_difficulties(
    config: &SoraConfig,
) -> Result<Box<[DifficultyDefinition]>, UniverseCatalogLoadError> {
    let mut definitions = Vec::with_capacity(config.universe_difficulty().len());
    let mut semantic_keys = BTreeSet::new();
    for row in config.universe_difficulty().ordered_rows() {
        let world = id::<WorldId>(row.world_id, "difficulty world")?;
        if config.universe_world().get(&row.world_id).is_none() {
            return Err(reference("difficulty references an unknown World"));
        }
        let kind = match row.kind {
            UniverseDifficultyKind::Tutorial => DifficultyKind::Tutorial,
            UniverseDifficultyKind::Standard => DifficultyKind::Standard,
        };
        let ordinal = positive_u8(row.difficulty, "difficulty ordinal")?;
        if !semantic_keys.insert((world, kind, ordinal)) {
            return Err(invalid("World difficulty semantic key is duplicated"));
        }
        let mut elements = Vec::with_capacity(row.recommended_elements.len());
        let mut seen_elements = BTreeSet::new();
        for value in &row.recommended_elements {
            let element = parse_element(value)?;
            if !seen_elements.insert(element) {
                return Err(invalid("recommended element is duplicated"));
            }
            elements.push(element);
        }
        definitions.push(DifficultyDefinition::new(
            id::<DifficultyId>(row.id, "difficulty")?,
            checked_key(&row.stable_key, "difficulty stable key")?,
            world,
            checked_source(&row.source_area_id, "source area ID")?,
            ordinal,
            kind,
            positive_u8(row.recommended_level, "recommended level")?,
            elements.into_boxed_slice(),
            parse_score_curve(&row.score_curve_json)?,
            row.unlock_source_id
                .as_deref()
                .map(|value| checked_source(value, "unlock source ID").map(Into::into))
                .transpose()?,
        ));
    }
    definitions.sort_by_key(DifficultyDefinition::id);
    if definitions.len() != 33 {
        return Err(invalid(
            "Standard Universe must define exactly 33 difficulties",
        ));
    }
    Ok(definitions.into_boxed_slice())
}

fn lower_worlds(
    config: &SoraConfig,
    profile: UniverseProfileId,
    difficulties: &[DifficultyDefinition],
) -> Result<Box<[WorldDefinition]>, UniverseCatalogLoadError> {
    let mut numbers = BTreeSet::new();
    let mut definitions = Vec::with_capacity(config.universe_world().len());
    for row in config.universe_world().ordered_rows() {
        if id::<UniverseProfileId>(row.profile_id, "World profile")? != profile {
            return Err(reference("World references the wrong Universe profile"));
        }
        let number = positive_u8(row.world_number, "World number")?;
        if !numbers.insert(number) {
            return Err(invalid("World number is duplicated"));
        }
        for key in [&row.entry_rule_stable_key, &row.terminal_rule_stable_key] {
            checked_key(key, "World lifecycle rule key")?;
        }
        if row.entry_rule_stable_key != EXPECTED_WORLD_ENTRY_RULE
            || row.terminal_rule_stable_key != EXPECTED_WORLD_TERMINAL_RULE
        {
            return Err(reference(
                "World lifecycle policy differs from the Standard profile contract",
            ));
        }
        let world_id = id::<WorldId>(row.id, "World")?;
        let difficulty_ids = difficulties
            .iter()
            .filter(|difficulty| difficulty.world() == world_id)
            .map(DifficultyDefinition::id)
            .collect::<Vec<_>>();
        if difficulty_ids.is_empty() {
            return Err(reference("World has no difficulty definition"));
        }
        definitions.push(WorldDefinition::new(
            world_id,
            profile,
            checked_key(&row.stable_key, "World stable key")?,
            number,
            localized(
                &row.name_en,
                &row.name_zh_cn,
                &row.summary_en,
                &row.summary_zh_cn,
                "World",
            )?,
            &row.entry_rule_stable_key,
            &row.terminal_rule_stable_key,
            difficulty_ids.into_boxed_slice(),
        ));
    }
    definitions.sort_by_key(WorldDefinition::id);
    if definitions.len() != 9 || numbers.iter().copied().ne(1..=9) {
        return Err(invalid(
            "World numbers must be the complete range 1 through 9",
        ));
    }
    Ok(definitions.into_boxed_slice())
}

fn lower_topologies(
    config: &SoraConfig,
) -> Result<Box<[TopologyDefinition]>, UniverseCatalogLoadError> {
    lower_topology_rows(
        config.universe_map_node().ordered_rows(),
        config.universe_map_edge().iter(),
    )
}

fn lower_topology_rows<'a>(
    node_rows: impl IntoIterator<Item = &'a UniverseMapNode>,
    edge_rows: impl IntoIterator<Item = &'a UniverseMapEdge>,
) -> Result<Box<[TopologyDefinition]>, UniverseCatalogLoadError> {
    let mut nodes_by_id = BTreeMap::new();
    let mut nodes_by_map: BTreeMap<u32, Vec<TopologyNodeId>> = BTreeMap::new();
    for row in node_rows {
        let node_id = id::<TopologyNodeId>(row.id, "topology node")?;
        let source_map = positive_u32(row.source_map_id, "source map ID")?;
        positive_u32(row.source_node_id, "source node ID")?;
        checked_key(&row.stable_key, "topology node stable key")?;
        nodes_by_map.entry(source_map).or_default().push(node_id);
        nodes_by_id.insert(node_id, row);
    }
    let mut outgoing: BTreeMap<TopologyNodeId, Vec<(u32, TopologyNodeId)>> = BTreeMap::new();
    for edge in edge_rows {
        let source = id::<TopologyNodeId>(edge.source_node_id, "edge source")?;
        let target = id::<TopologyNodeId>(edge.target_node_id, "edge target")?;
        let sequence = positive_u32(edge.sequence, "edge sequence")?;
        let source_row = nodes_by_id
            .get(&source)
            .ok_or_else(|| reference("topology edge source is unresolved"))?;
        let target_row = nodes_by_id
            .get(&target)
            .ok_or_else(|| reference("topology edge target is unresolved"))?;
        if source_row.source_map_id != target_row.source_map_id {
            return Err(graph("topology edge crosses source-map boundaries"));
        }
        outgoing.entry(source).or_default().push((sequence, target));
    }
    for edges in outgoing.values_mut() {
        edges.sort_by_key(|edge| edge.0);
        if edges
            .iter()
            .map(|edge| edge.0)
            .ne(1..=u32::try_from(edges.len()).unwrap_or(0))
        {
            return Err(graph("topology edge sequences are not contiguous"));
        }
        if edges
            .iter()
            .map(|edge| edge.1)
            .collect::<BTreeSet<_>>()
            .len()
            != edges.len()
        {
            return Err(graph("topology edge target is duplicated"));
        }
    }

    let mut definitions = Vec::with_capacity(nodes_by_map.len());
    for (source_map, mut node_ids) in nodes_by_map {
        node_ids.sort_unstable();
        let source_node_ids = node_ids
            .iter()
            .map(|id| positive_u32(nodes_by_id[id].source_node_id, "source node ID"))
            .collect::<Result<BTreeSet<_>, _>>()?;
        if source_node_ids.len() != node_ids.len() {
            return Err(graph("topology repeats a source node ID"));
        }
        let starts = node_ids
            .iter()
            .copied()
            .filter(|id| nodes_by_id[id].is_start)
            .collect::<Vec<_>>();
        if starts.len() != 1 {
            return Err(graph("each topology must have exactly one start node"));
        }
        let start = starts[0];
        let mut reachable = BTreeSet::new();
        let mut pending = vec![start];
        while let Some(node) = pending.pop() {
            if reachable.insert(node) {
                pending.extend(outgoing.get(&node).into_iter().flatten().map(|edge| edge.1));
            }
        }
        if reachable.len() != node_ids.len() {
            return Err(graph("topology contains a node unreachable from its start"));
        }
        let mut indegree = node_ids
            .iter()
            .copied()
            .map(|id| (id, 0u32))
            .collect::<BTreeMap<_, _>>();
        for node in &node_ids {
            for (_, target) in outgoing.get(node).into_iter().flatten() {
                let value = indegree.get_mut(target).expect("cross-map edges rejected");
                *value = value
                    .checked_add(1)
                    .ok_or_else(|| graph("topology indegree overflow"))?;
            }
        }
        let mut ready = indegree
            .iter()
            .filter_map(|(id, value)| (*value == 0).then_some(*id))
            .collect::<Vec<_>>();
        let mut settled = 0usize;
        while let Some(node) = ready.pop() {
            settled += 1;
            for (_, target) in outgoing.get(&node).into_iter().flatten() {
                let value = indegree.get_mut(target).expect("cross-map edges rejected");
                *value -= 1;
                if *value == 0 {
                    ready.push(*target);
                }
            }
        }
        if settled != node_ids.len() {
            return Err(graph("topology contains an unbounded cycle"));
        }
        let terminals = node_ids
            .iter()
            .copied()
            .filter(|node| outgoing.get(node).is_none_or(Vec::is_empty))
            .collect::<Vec<_>>();
        if terminals.is_empty() {
            return Err(graph("topology has no reachable terminal node"));
        }
        let mut nodes = Vec::with_capacity(node_ids.len());
        for node_id in node_ids {
            let row = nodes_by_id[&node_id];
            let targets = outgoing
                .get(&node_id)
                .into_iter()
                .flatten()
                .map(|edge| edge.1)
                .collect::<Vec<_>>();
            nodes.push(TopologyNodeDefinition::new(
                node_id,
                &row.stable_key,
                positive_u32(row.source_node_id, "source node ID")?,
                targets.into_boxed_slice(),
            ));
        }
        definitions.push(TopologyDefinition::new(
            TopologyId::new(start.get()).expect("start ID is non-zero"),
            source_map,
            start,
            terminals.into_boxed_slice(),
            nodes.into_boxed_slice(),
        ));
    }
    definitions.sort_by_key(TopologyDefinition::id);
    if definitions.is_empty() {
        return Err(graph("Universe topology catalog is empty"));
    }
    Ok(definitions.into_boxed_slice())
}

fn lower_rooms(
    config: &SoraConfig,
    domains: &[DomainDefinition],
) -> Result<Box<[RoomDefinition]>, UniverseCatalogLoadError> {
    let known_domains = domains
        .iter()
        .map(DomainDefinition::id)
        .collect::<BTreeSet<_>>();
    let mut covered_domains = BTreeSet::new();
    let mut definitions = Vec::with_capacity(config.universe_room().len());
    for row in config.universe_room().ordered_rows() {
        let domain = id::<DomainId>(row.domain_id, "room domain")?;
        if !known_domains.contains(&domain) {
            return Err(reference("room references an unknown domain"));
        }
        covered_domains.insert(domain);
        let mut section_ids = Vec::with_capacity(row.section_ids.len());
        let mut seen_sections = BTreeSet::new();
        for section in &row.section_ids {
            let section = u32::try_from(*section)
                .map_err(|_| invalid("room section ID cannot be negative"))?;
            if !seen_sections.insert(section) {
                return Err(invalid("room section ID is duplicated"));
            }
            section_ids.push(section);
        }
        definitions.push(RoomDefinition::new(
            id::<RoomId>(row.id, "room")?,
            checked_key(&row.stable_key, "room stable key")?,
            domain,
            checked_source(&row.source_room_id, "source room ID")?,
            checked_source(&row.map_entrance, "map entrance")?,
            checked_source(&row.source_group_id, "source group ID")?,
            section_ids.into_boxed_slice(),
        ));
    }
    definitions.sort_by_key(RoomDefinition::id);
    if definitions.len() != 163 || covered_domains != known_domains {
        return Err(reference(
            "rooms must cover every domain exactly through valid references",
        ));
    }
    Ok(definitions.into_boxed_slice())
}

fn lower_activity(
    config: &SoraConfig,
    profile: UniverseProfileId,
    domains: &[DomainDefinition],
) -> Result<UniverseActivityBindingDefinition, UniverseCatalogLoadError> {
    let mut rows = config.universe_activity_binding().ordered_rows();
    let row = rows
        .next()
        .ok_or_else(|| invalid("Universe Activity binding is absent"))?;
    if rows.next().is_some()
        || row.stable_key != EXPECTED_BINDING_KEY
        || row.activity_stable_key != EXPECTED_ACTIVITY_KEY
        || row.battle_handoff_contract != EXPECTED_BATTLE_HANDOFF
        || row.external_outcome_contract != EXPECTED_EXTERNAL_OUTCOME
        || !row.participant_digest_locked
        || !row.scoped_slots_supported
        || !row.fork_join_reserved
    {
        return Err(invalid(
            "Universe Activity binding contract differs from Goal 04",
        ));
    }
    let binding_id = id::<ActivityBindingId>(row.id, "Activity binding")?;
    if id::<UniverseProfileId>(row.profile_id, "Activity profile")? != profile {
        return Err(reference("Activity binding references the wrong profile"));
    }
    let by_id = domains
        .iter()
        .map(|domain| (domain.id(), domain))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    let mut bindings = Vec::with_capacity(config.universe_activity_domain_binding().len());
    for (index, item) in config.universe_activity_domain_binding().iter().enumerate() {
        if id::<ActivityBindingId>(item.activity_binding_id, "domain Activity binding")?
            != binding_id
            || positive_u32(item.sequence, "domain binding sequence")?
                != u32::try_from(index + 1).unwrap_or(0)
        {
            return Err(reference(
                "Activity domain-binding parent or sequence is invalid",
            ));
        }
        let domain_id = id::<DomainId>(item.domain_id, "Activity domain")?;
        let domain = by_id
            .get(&domain_id)
            .ok_or_else(|| reference("Activity binding references an unknown domain"))?;
        if !seen.insert(domain_id) {
            return Err(invalid("Activity binding repeats a domain"));
        }
        let decision = match item.decision_kind {
            UniverseActivityDecision::BattleCommand => ActivityDomainDecision::BattleCommand,
            UniverseActivityDecision::RunCommand => ActivityDomainDecision::RunCommand,
            UniverseActivityDecision::ExternalOutcome => ActivityDomainDecision::ExternalOutcome,
        };
        let expected = match (domain.decision_policy(), domain.kind()) {
            (DomainDecisionPolicy::BattleHandoff, _) => ActivityDomainDecision::BattleCommand,
            (DomainDecisionPolicy::ExternalCommand, DomainKind::Adventure) => {
                ActivityDomainDecision::ExternalOutcome
            }
            (DomainDecisionPolicy::ExternalCommand, _) => ActivityDomainDecision::RunCommand,
        };
        if decision != expected {
            return Err(invalid(
                "Activity decision kind conflicts with its domain policy",
            ));
        }
        bindings.push(ActivityDomainBinding::new(domain_id, decision));
    }
    if seen.len() != domains.len() {
        return Err(reference("Activity binding does not cover every domain"));
    }
    Ok(UniverseActivityBindingDefinition::new(
        binding_id,
        &row.stable_key,
        profile,
        &row.activity_stable_key,
        &row.battle_handoff_contract,
        &row.external_outcome_contract,
        bindings.into_boxed_slice(),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScoreThresholdTransport {
    tier: u8,
    score: String,
}

fn parse_score_curve(value: &str) -> Result<Box<[ScoreThreshold]>, UniverseCatalogLoadError> {
    let rows: Vec<ScoreThresholdTransport> = serde_json::from_str(value)
        .map_err(|_| embedded("difficulty score curve is not canonical typed JSON"))?;
    if rows.is_empty() || rows.len() > 32 {
        return Err(embedded(
            "difficulty score curve length is outside 1 through 32",
        ));
    }
    let mut tiers = BTreeSet::new();
    let mut previous_score = 0;
    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        let score = row
            .score
            .parse::<u32>()
            .map_err(|_| embedded("difficulty score is not a canonical u32"))?;
        if row.tier == 0 || score == 0 || !tiers.insert(row.tier) || score < previous_score {
            return Err(embedded(
                "difficulty score curve is unordered, duplicated or non-positive",
            ));
        }
        previous_score = score;
        result.push(ScoreThreshold::new(row.tier, score));
    }
    Ok(result.into_boxed_slice())
}

fn parse_element(value: &str) -> Result<RecommendedElement, UniverseCatalogLoadError> {
    match value {
        "Physical" => Ok(RecommendedElement::Physical),
        "Fire" => Ok(RecommendedElement::Fire),
        "Ice" => Ok(RecommendedElement::Ice),
        "Lightning" | "Thunder" => Ok(RecommendedElement::Lightning),
        "Wind" => Ok(RecommendedElement::Wind),
        "Quantum" => Ok(RecommendedElement::Quantum),
        "Imaginary" => Ok(RecommendedElement::Imaginary),
        _ => Err(invalid(
            "difficulty contains an unknown recommended element",
        )),
    }
}

fn digest(definitions: &UniverseDefinitions) -> UniverseDefinitionsDigest {
    let mut encoder = Encoder::new(b"starclock-standard-universe-definitions-v1");
    encoder.u32(definitions.profile.id().get());
    encoder.text(definitions.profile.stable_key());
    encoder.text(definitions.profile.game_version());
    encoder.text(definitions.profile.snapshot_date());
    encoder.digest(definitions.profile.content_manifest_digest());
    encoder.digest(definitions.profile.normalized_pack_digest());
    encoder.u32(definitions.worlds.len() as u32);
    for value in &definitions.worlds {
        encoder.u32(value.id().get());
        encoder.u32(value.profile().get());
        encoder.text(value.stable_key());
        encoder.u8(value.number());
        encode_text(&mut encoder, value.text());
        encoder.text(value.entry_rule_key());
        encoder.text(value.terminal_rule_key());
        encoder.u32(value.difficulties().len() as u32);
        for id in value.difficulties() {
            encoder.u32(id.get());
        }
    }
    encoder.u32(definitions.difficulties.len() as u32);
    for value in &definitions.difficulties {
        encoder.u32(value.id().get());
        encoder.text(value.stable_key());
        encoder.u32(value.world().get());
        encoder.text(value.source_area_id());
        encoder.u8(value.ordinal());
        encoder.u8(value.kind() as u8);
        encoder.u8(value.recommended_level());
        encoder.u32(value.recommended_elements().len() as u32);
        for element in value.recommended_elements() {
            encoder.u8(*element as u8);
        }
        encoder.u32(value.score_curve().len() as u32);
        for point in value.score_curve() {
            encoder.u8(point.tier());
            encoder.u32(point.score());
        }
        encoder.optional_text(value.unlock_source_id());
    }
    encoder.u32(definitions.domains.len() as u32);
    for value in &definitions.domains {
        encoder.u32(value.id().get());
        encoder.text(value.stable_key());
        encoder.u32(value.source_type());
        encoder.u8(value.kind() as u8);
        encoder.u8(value.decision_policy() as u8);
        encoder.bool(value.is_terminal());
        encode_text(&mut encoder, value.text());
    }
    encoder.u32(definitions.topologies.len() as u32);
    for value in &definitions.topologies {
        encoder.u32(value.id().get());
        encoder.u32(value.source_map_id());
        encoder.u32(value.start().get());
        encoder.u32(value.terminals().len() as u32);
        for id in value.terminals() {
            encoder.u32(id.get());
        }
        encoder.u32(value.nodes().len() as u32);
        for node in value.nodes() {
            encoder.u32(node.id().get());
            encoder.text(node.stable_key());
            encoder.u32(node.source_node_id());
            encoder.u32(node.outgoing().len() as u32);
            for id in node.outgoing() {
                encoder.u32(id.get());
            }
        }
    }
    encoder.u32(definitions.rooms.len() as u32);
    for value in &definitions.rooms {
        encoder.u32(value.id().get());
        encoder.text(value.stable_key());
        encoder.u32(value.domain().get());
        encoder.text(value.source_room_id());
        encoder.text(value.map_entrance());
        encoder.text(value.source_group_id());
        encoder.u32(value.section_ids().len() as u32);
        for section in value.section_ids() {
            encoder.u32(*section);
        }
    }
    encoder.u32(definitions.activity.id().get());
    encoder.text(definitions.activity.stable_key());
    encoder.u32(definitions.activity.profile().get());
    encoder.text(definitions.activity.activity_key());
    encoder.text(definitions.activity.battle_handoff_contract());
    encoder.text(definitions.activity.external_outcome_contract());
    encoder.u32(definitions.activity.domains().len() as u32);
    for binding in definitions.activity.domains() {
        encoder.u32(binding.domain().get());
        encoder.u8(binding.decision() as u8);
    }
    UniverseDefinitionsDigest::new(encoder.finish())
}

fn encode_text(encoder: &mut Encoder, value: &LocalizedText) {
    encoder.text(value.name_en());
    encoder.text(value.name_zh_cn());
    encoder.text(value.summary_en());
    encoder.text(value.summary_zh_cn());
}

fn localized(
    name_en: &str,
    name_zh_cn: &str,
    summary_en: &str,
    summary_zh_cn: &str,
    label: &str,
) -> Result<LocalizedText, UniverseCatalogLoadError> {
    for value in [name_en, name_zh_cn, summary_en, summary_zh_cn] {
        if value.trim().is_empty() || value.len() > 2_048 {
            return Err(invalid(format!(
                "{label} localized text is empty or oversized"
            )));
        }
    }
    Ok(LocalizedText::new(
        name_en,
        name_zh_cn,
        summary_en,
        summary_zh_cn,
    ))
}

fn checked_key<'a>(value: &'a str, label: &str) -> Result<&'a str, UniverseCatalogLoadError> {
    if !value.is_empty()
        && value.len() <= 200
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        Ok(value)
    } else {
        Err(invalid(format!("{label} is not a bounded stable key")))
    }
}

fn checked_token<'a>(value: &'a str, label: &str) -> Result<&'a str, UniverseCatalogLoadError> {
    if !value.is_empty() && value.len() <= 80 && value.bytes().all(|byte| byte.is_ascii_graphic()) {
        Ok(value)
    } else {
        Err(invalid(format!("{label} is not a bounded token")))
    }
}

fn checked_source<'a>(value: &'a str, label: &str) -> Result<&'a str, UniverseCatalogLoadError> {
    if !value.is_empty() && value.len() <= 120 && value.bytes().all(|byte| byte.is_ascii_graphic())
    {
        Ok(value)
    } else {
        Err(invalid(format!("{label} is empty or malformed")))
    }
}

fn parse_digest(value: &str, label: &str) -> Result<[u8; 32], UniverseCatalogLoadError> {
    if value.len() != 64 {
        return Err(invalid(format!("{label} is not SHA-256")));
    }
    let mut bytes = [0; 32];
    for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
        let text =
            std::str::from_utf8(chunk).map_err(|_| invalid(format!("{label} is not ASCII")))?;
        bytes[index] = u8::from_str_radix(text, 16)
            .map_err(|_| invalid(format!("{label} is not lowercase hexadecimal")))?;
        if text.bytes().any(|byte| byte.is_ascii_uppercase()) {
            return Err(invalid(format!("{label} is not lowercase hexadecimal")));
        }
    }
    Ok(bytes)
}

fn positive_u32(value: i32, label: &str) -> Result<u32, UniverseCatalogLoadError> {
    u32::try_from(value)
        .ok()
        .filter(|value| *value != 0)
        .ok_or_else(|| invalid(format!("{label} must be positive")))
}

fn positive_u8(value: i32, label: &str) -> Result<u8, UniverseCatalogLoadError> {
    u8::try_from(value)
        .ok()
        .filter(|value| *value != 0)
        .ok_or_else(|| invalid(format!("{label} must be a positive u8")))
}

trait TransportId: Sized {
    fn from_transport(raw: i32) -> Option<Self>;
}
macro_rules! transport_id {
    ($($name:ty),+ $(,)?) => { $(impl TransportId for $name { fn from_transport(raw: i32) -> Option<Self> { u32::try_from(raw).ok().and_then(<$name>::new) } })+ };
}
transport_id!(
    UniverseProfileId,
    WorldId,
    DifficultyId,
    DomainId,
    TopologyNodeId,
    RoomId,
    ActivityBindingId
);

fn id<T: TransportId>(value: i32, label: &str) -> Result<T, UniverseCatalogLoadError> {
    T::from_transport(value).ok_or_else(|| invalid(format!("{label} ID must be a positive u32")))
}

fn invalid(message: impl Into<Box<str>>) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidDefinition, message)
}
fn reference(message: impl Into<Box<str>>) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidReference, message)
}
fn graph(message: impl Into<Box<str>>) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidGraph, message)
}
fn embedded(message: impl Into<Box<str>>) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidEmbeddedData, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_curve_parser_is_typed_bounded_and_monotonic() {
        let curve = parse_score_curve(r#"[{"score":"10","tier":1},{"score":"10","tier":2}]"#)
            .expect("curve");
        assert_eq!(curve.len(), 2);
        assert_eq!(curve[1].tier(), 2);
        assert!(parse_score_curve(r#"[{"score":"9","tier":2},{"score":"8","tier":1}]"#).is_err());
        assert!(parse_score_curve(r#"[{"score":9,"tier":1}]"#).is_err());
        assert!(parse_score_curve(r#"[{"score":"9","tier":1,"extra":0}]"#).is_err());
    }

    #[test]
    fn topology_lowering_rejects_dangling_edges_and_ignores_position_hints() {
        let nodes = vec![
            UniverseMapNode {
                id: 1,
                stable_key: "universe.map.test.node.1".to_owned(),
                source_map_id: 99,
                source_node_id: 1,
                is_start: true,
                position_x: 10,
                position_y: 20,
            },
            UniverseMapNode {
                id: 2,
                stable_key: "universe.map.test.node.2".to_owned(),
                source_map_id: 99,
                source_node_id: 2,
                is_start: false,
                position_x: 30,
                position_y: 40,
            },
        ];
        let edges = [UniverseMapEdge {
            source_node_id: 1,
            sequence: 1,
            target_node_id: 2,
        }];
        let first = lower_topology_rows(nodes.iter(), edges.iter()).expect("topology");
        let mut moved = nodes.clone();
        moved[0].position_x = -500;
        moved[1].position_y = 700;
        let second = lower_topology_rows(moved.iter(), edges.iter()).expect("moved topology");
        assert_eq!(first, second);

        let dangling = [UniverseMapEdge {
            source_node_id: 1,
            sequence: 1,
            target_node_id: 3,
        }];
        let error = lower_topology_rows(nodes.iter(), dangling.iter()).unwrap_err();
        assert_eq!(error.kind(), UniverseCatalogLoadErrorKind::InvalidReference);
    }
}
