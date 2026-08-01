use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::Deserialize;

use crate::{
    definition::RecommendedElement,
    swarm_disaster_catalog::SwarmDisasterBundleSummary,
    swarm_disaster_generated::{
        SoraConfig, swarm_disaster_area::SwarmDisasterArea,
        swarm_disaster_beacon::SwarmDisasterBeacon,
        swarm_disaster_boss_choice::SwarmDisasterBossChoice,
        swarm_disaster_chessboard::SwarmDisasterChessboard,
        swarm_disaster_difficulty_segment::SwarmDisasterDifficultySegment,
        swarm_disaster_domain::SwarmDisasterDomain,
        swarm_disaster_map_column::SwarmDisasterMapColumn,
        swarm_disaster_map_edge::SwarmDisasterMapEdge,
        swarm_disaster_map_node::SwarmDisasterMapNode, swarm_disaster_plane::SwarmDisasterPlane,
        swarm_disaster_profile::SwarmDisasterProfile, swarm_disaster_room::SwarmDisasterRoom,
    },
};

use super::{
    EXPECTED_STRUCTURAL_ROWS, SwarmDisasterStructuralCatalog, SwarmDisasterStructuralError,
    SwarmDisasterStructuralErrorKind,
    types::{
        AreaDefinition, AreaId, AreaKind, BeaconDefinition, BeaconId, BossChoiceDefinition,
        BossChoiceId, ChessboardDefinition, ChessboardId, DifficultySegmentDefinition,
        DifficultySegmentId, DomainDefinition, DomainId, DomainResolution, MapColumnDefinition,
        MapColumnId, MapEdgeDefinition, MapEdgeId, MapNodeDefinition, MapNodeId, PlaneDefinition,
        PlaneId, ProfileDefinition, ProfileId, ProfileRowKind, RoomDefinition, RoomId,
    },
};

const ROW_REVISION: &str = "starclock.swarm-disaster-row.v1";
const SUB_MODE: &str = "ChessRogue";
const EDGE_POLICY: &str = "forward-nearest-column-within-one-row-v1";
const TERMINAL_POLICY: &str = "AuthoredEndGridItem";
const DOMAIN_SELECTION_POLICY: &str = r#"{"candidate_order":"StableNodeId","candidate_source":"AuthoredNodeCandidatesOrBlockCreationWeights","no_legal_target":"NoOp","weighted_sampling":"IntegerWeight"}"#;
const DOMAIN_REPLACEMENT_POLICY: &str = r#"{"mutation_order":"StableNodeId","no_legal_target":"NoOp","preserve_terminal_nodes":true,"preserve_unmentioned_metadata":true,"trigger_source":"TypedTopologyConsequence"}"#;
const BOSS_DECAY_POLICY: &str = r#"{"kind":"TypedBossDecayReference","resolution_state":"DeferredToG09-P1-B3","unresolved_reference_policy":"RejectAtPackCompilation"}"#;

pub(super) fn lower(
    bundle: SwarmDisasterBundleSummary,
    source: &SoraConfig,
) -> Result<SwarmDisasterStructuralCatalog, SwarmDisasterStructuralError> {
    let catalog = SwarmDisasterStructuralCatalog {
        bundle,
        profiles: source
            .swarm_disaster_profile()
            .ordered_rows()
            .map(lower_profile)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        areas: source
            .swarm_disaster_area()
            .ordered_rows()
            .map(lower_area)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        difficulty_segments: source
            .swarm_disaster_difficulty_segment()
            .ordered_rows()
            .map(lower_difficulty_segment)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        planes: source
            .swarm_disaster_plane()
            .ordered_rows()
            .map(lower_plane)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        chessboards: source
            .swarm_disaster_chessboard()
            .ordered_rows()
            .map(lower_chessboard)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        columns: source
            .swarm_disaster_map_column()
            .ordered_rows()
            .map(lower_column)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        nodes: source
            .swarm_disaster_map_node()
            .ordered_rows()
            .map(lower_node)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        edges: source
            .swarm_disaster_map_edge()
            .ordered_rows()
            .map(lower_edge)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        rooms: source
            .swarm_disaster_room()
            .ordered_rows()
            .map(lower_room)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        domains: source
            .swarm_disaster_domain()
            .ordered_rows()
            .map(lower_domain)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        beacons: source
            .swarm_disaster_beacon()
            .ordered_rows()
            .map(lower_beacon)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        boss_choices: source
            .swarm_disaster_boss_choice()
            .ordered_rows()
            .map(lower_boss_choice)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
    };
    validate(&catalog)?;
    Ok(catalog)
}

fn lower_profile(
    row: &SwarmDisasterProfile,
) -> Result<ProfileDefinition, SwarmDisasterStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    let kind = match (row.kind.as_str(), row.entry_kind.as_deref()) {
        ("EntryPoint", Some("ResidentActivity")) => ProfileRowKind::ResidentActivity,
        ("EntryPoint", Some("DlcEntrance")) => ProfileRowKind::DlcEntrance,
        ("EntryPoint", Some("ModeTitle")) => ProfileRowKind::ModeTitle,
        ("SwarmProfile", None) => ProfileRowKind::RuntimeProfile,
        _ => return invalid_value(&row.stable_key),
    };
    Ok(ProfileDefinition {
        id: ProfileId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        kind,
        sub_mode: nonempty(
            row.sub_mode.as_deref().ok_or_else(|| {
                error(SwarmDisasterStructuralErrorKind::Metadata, &row.stable_key)
            })?,
            &row.stable_key,
        )?,
        runtime_enabled: row.runtime_enabled,
    })
}

fn lower_area(row: &SwarmDisasterArea) -> Result<AreaDefinition, SwarmDisasterStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    let kind = match row.area_kind.as_str() {
        "Formal" => AreaKind::Formal,
        "Guide" => AreaKind::Guide,
        _ => return invalid_value(&row.stable_key),
    };
    nonempty(&row.displayed_monsters_json, &row.stable_key)?;
    nonempty(&row.score_thresholds_json, &row.stable_key)?;
    Ok(AreaDefinition {
        id: AreaId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        source_id: nonempty(&row.source_id, &row.stable_key)?,
        kind,
        difficulty: difficulty(&row.difficulty, &row.stable_key)?,
        difficulty_segment_keys: text_list(&row.difficulty_segment_ids, &row.stable_key)?,
        plane_keys: text_list(&row.plane_ids, &row.stable_key)?,
        recommended_level: positive_u16(row.recommended_level, &row.stable_key)?,
        recommended_elements: row
            .recommended_elements
            .iter()
            .map(|value| element(value, &row.stable_key))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
    })
}

fn lower_difficulty_segment(
    row: &SwarmDisasterDifficultySegment,
) -> Result<DifficultySegmentDefinition, SwarmDisasterStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(DifficultySegmentDefinition {
        id: DifficultySegmentId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        source_id: nonempty(&row.source_id, &row.stable_key)?,
        cut_positions: numeric_list(&row.cut_list, &row.stable_key)?,
        levels: numeric_list(&row.level_list, &row.stable_key)?,
    })
}

fn lower_plane(row: &SwarmDisasterPlane) -> Result<PlaneDefinition, SwarmDisasterStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    if row.terminal_policy != TERMINAL_POLICY {
        return invalid_value(&row.stable_key);
    }
    Ok(PlaneDefinition {
        id: PlaneId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        source_id: nonempty(&row.source_id, &row.stable_key)?,
        plane_number: positive_u8(row.plane_number, &row.stable_key)?,
        chessboard_keys: text_list(&row.chessboard_ids, &row.stable_key)?,
    })
}

fn lower_chessboard(
    row: &SwarmDisasterChessboard,
) -> Result<ChessboardDefinition, SwarmDisasterStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    nonempty(&row.source_config_path, &row.stable_key)?;
    if row
        .event_ids
        .as_deref()
        .is_some_and(|values| values.iter().any(|value| value.trim().is_empty()))
    {
        return invalid_value(&row.stable_key);
    }
    Ok(ChessboardDefinition {
        id: ChessboardId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        source_id: nonempty(&row.source_id, &row.stable_key)?,
        width: positive_u16(row.width, &row.stable_key)?,
        height: positive_u16(row.height, &row.stable_key)?,
        start: MapNodeId(positive(row.start_node_id, &row.stable_key)?),
        end: MapNodeId(positive(row.end_node_id, &row.stable_key)?),
        block_create_group: nonempty(&row.block_create_group_id, &row.stable_key)?,
    })
}

fn lower_column(
    row: &SwarmDisasterMapColumn,
) -> Result<MapColumnDefinition, SwarmDisasterStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(MapColumnDefinition {
        id: MapColumnId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        chessboard: ChessboardId(positive(row.chessboard_id, &row.stable_key)?),
        index: nonnegative_u16(row.column_index, &row.stable_key)?,
        position_x: row.position_x,
        node_keys: text_list(&row.node_ids, &row.stable_key)?,
    })
}

fn lower_node(
    row: &SwarmDisasterMapNode,
) -> Result<MapNodeDefinition, SwarmDisasterStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    nonempty(&row.source_id, &row.stable_key)?;
    let domain_resolution = match row.domain_resolution.as_str() {
        "AuthoredCandidates" => DomainResolution::AuthoredCandidates,
        "Unspecified" => DomainResolution::Unspecified,
        _ => return invalid_value(&row.stable_key),
    };
    Ok(MapNodeDefinition {
        id: MapNodeId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        chessboard: ChessboardId(positive(row.chessboard_id, &row.stable_key)?),
        column: MapColumnId(positive(row.column_id, &row.stable_key)?),
        position_x: row.position_x,
        domain_keys: optional_text_list(row.domain_candidates.as_deref(), &row.stable_key)?,
        domain_resolution,
        is_start: row.is_start,
        is_end: row.is_end,
    })
}

fn lower_edge(
    row: &SwarmDisasterMapEdge,
) -> Result<MapEdgeDefinition, SwarmDisasterStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    if row.policy_id != EDGE_POLICY {
        return invalid_value(&row.stable_key);
    }
    Ok(MapEdgeDefinition {
        id: MapEdgeId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        chessboard: ChessboardId(positive(row.chessboard_id, &row.stable_key)?),
        source: MapNodeId(positive(row.from_node_id, &row.stable_key)?),
        target: MapNodeId(positive(row.to_node_id, &row.stable_key)?),
    })
}

fn lower_room(row: &SwarmDisasterRoom) -> Result<RoomDefinition, SwarmDisasterStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    if row.domain_id.is_some()
        || row.encounter_pool_ids.is_some()
        || row.domain_binding_state != "NotPublishedInRoomRow"
        || row.encounter_binding_state != "DeferredToG09-P2-B5"
    {
        return invalid_value(&row.stable_key);
    }
    Ok(RoomDefinition {
        id: RoomId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        source_id: nonempty(&row.source_id, &row.stable_key)?,
        sub_mode: nonempty(&row.sub_mode, &row.stable_key)?,
        sections: nonnegative_numeric_list(&row.section_ids, &row.stable_key)?,
    })
}

fn lower_domain(
    row: &SwarmDisasterDomain,
) -> Result<DomainDefinition, SwarmDisasterStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    if row.selection_policy_json != DOMAIN_SELECTION_POLICY
        || row.replacement_policy_json != DOMAIN_REPLACEMENT_POLICY
    {
        return invalid_value(&row.stable_key);
    }
    Ok(DomainDefinition {
        id: DomainId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        source_id: nonempty(&row.source_id, &row.stable_key)?,
    })
}

fn lower_beacon(
    row: &SwarmDisasterBeacon,
) -> Result<BeaconDefinition, SwarmDisasterStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    if row.application_stage != "TopologyMutationResolution"
        || row.copy_policy_json != r#""OnlyWhenExplicitlyRequestedByTypedEffect""#
        || row.blanking_policy_json != r#""PreserveUnlessExplicitlyRequestedByTypedEffect""#
    {
        return invalid_value(&row.stable_key);
    }
    Ok(BeaconDefinition {
        id: BeaconId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        source_id: nonempty(&row.source_id, &row.stable_key)?,
        block_intro_id: nonempty(&row.block_intro_id, &row.stable_key)?,
    })
}

fn lower_boss_choice(
    row: &SwarmDisasterBossChoice,
) -> Result<BossChoiceDefinition, SwarmDisasterStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    if row.later_boss_consequence_json != BOSS_DECAY_POLICY {
        return invalid_value(&row.stable_key);
    }
    let weakness: WeaknessConsequence = serde_json::from_str(&row.weakness_consequence_json)
        .map_err(|_| error(SwarmDisasterStructuralErrorKind::Metadata, &row.stable_key))?;
    if weakness.kind != "IntrinsicWeaknessSet" {
        return invalid_value(&row.stable_key);
    }
    Ok(BossChoiceDefinition {
        id: BossChoiceId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        source_id: nonempty(&row.source_id, &row.stable_key)?,
        display_level: positive_u16(row.display_level, &row.stable_key)?,
        enemy_variant_id: nonempty(&row.enemy_variant_id, &row.stable_key)?,
        weakness_elements: weakness
            .elements
            .iter()
            .map(|value| element(value, &row.stable_key))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WeaknessConsequence {
    kind: String,
    elements: Vec<String>,
}

fn validate(catalog: &SwarmDisasterStructuralCatalog) -> Result<(), SwarmDisasterStructuralError> {
    let counts = [
        (catalog.profiles.len(), 4, "profiles"),
        (catalog.areas.len(), 8, "areas"),
        (catalog.difficulty_segments.len(), 20, "difficulty-segments"),
        (catalog.planes.len(), 11, "planes"),
        (catalog.chessboards.len(), 101, "chessboards"),
        (catalog.columns.len(), 1_109, "columns"),
        (catalog.nodes.len(), 1_991, "nodes"),
        (catalog.edges.len(), 2_593, "edges"),
        (catalog.rooms.len(), 861, "rooms"),
        (catalog.domains.len(), 12, "domains"),
        (catalog.beacons.len(), 4, "beacons"),
        (catalog.boss_choices.len(), 2, "boss-choices"),
    ];
    if catalog.bundle.table_count() != 65
        || counts
            .iter()
            .any(|(actual, expected, _)| actual != expected)
        || catalog.row_count() != EXPECTED_STRUCTURAL_ROWS
    {
        let key = counts
            .iter()
            .find(|(actual, expected, _)| actual != expected)
            .map_or("structural-total", |(_, _, key)| *key);
        return fail(SwarmDisasterStructuralErrorKind::Denominator, key);
    }
    sequential(catalog.profiles.iter().map(|row| row.id.0), "profiles")?;
    sequential(catalog.areas.iter().map(|row| row.id.0), "areas")?;
    sequential(
        catalog.difficulty_segments.iter().map(|row| row.id.0),
        "difficulty-segments",
    )?;
    sequential(catalog.planes.iter().map(|row| row.id.0), "planes")?;
    sequential(
        catalog.chessboards.iter().map(|row| row.id.0),
        "chessboards",
    )?;
    sequential(catalog.columns.iter().map(|row| row.id.0), "columns")?;
    sequential(catalog.nodes.iter().map(|row| row.id.0), "nodes")?;
    sequential(catalog.edges.iter().map(|row| row.id.0), "edges")?;
    sequential(catalog.rooms.iter().map(|row| row.id.0), "rooms")?;
    sequential(catalog.domains.iter().map(|row| row.id.0), "domains")?;
    sequential(catalog.beacons.iter().map(|row| row.id.0), "beacons")?;
    sequential(
        catalog.boss_choices.iter().map(|row| row.id.0),
        "boss-choices",
    )?;
    validate_profiles(catalog)?;
    validate_references(catalog)?;
    validate_leaf_rows(catalog)?;
    validate_graphs(catalog)
}

fn validate_profiles(
    catalog: &SwarmDisasterStructuralCatalog,
) -> Result<(), SwarmDisasterStructuralError> {
    unique(
        catalog.profiles.iter().map(|row| row.stable_key.as_ref()),
        "profiles",
    )?;
    for kind in [
        ProfileRowKind::ResidentActivity,
        ProfileRowKind::DlcEntrance,
        ProfileRowKind::ModeTitle,
        ProfileRowKind::RuntimeProfile,
    ] {
        if catalog
            .profiles
            .iter()
            .filter(|row| row.kind == kind)
            .count()
            != 1
        {
            return fail(SwarmDisasterStructuralErrorKind::Metadata, "profiles");
        }
    }
    if catalog.profiles.iter().any(|row| {
        row.sub_mode.as_ref() != SUB_MODE
            || (row.kind == ProfileRowKind::RuntimeProfile && row.runtime_enabled != Some(false))
            || (row.kind != ProfileRowKind::RuntimeProfile && row.runtime_enabled.is_some())
    }) {
        return fail(SwarmDisasterStructuralErrorKind::Metadata, "profiles");
    }
    Ok(())
}

fn validate_references(
    catalog: &SwarmDisasterStructuralCatalog,
) -> Result<(), SwarmDisasterStructuralError> {
    unique(
        catalog.areas.iter().map(|row| row.stable_key.as_ref()),
        "areas",
    )?;
    unique(
        catalog
            .difficulty_segments
            .iter()
            .map(|row| row.stable_key.as_ref()),
        "difficulty-segments",
    )?;
    unique(
        catalog.planes.iter().map(|row| row.stable_key.as_ref()),
        "planes",
    )?;
    let segment_keys = catalog
        .difficulty_segments
        .iter()
        .map(|row| row.stable_key.as_ref())
        .collect::<BTreeSet<_>>();
    let plane_keys = catalog
        .planes
        .iter()
        .map(|row| row.stable_key.as_ref())
        .collect::<BTreeSet<_>>();
    let board_keys = catalog
        .chessboards
        .iter()
        .map(|row| row.stable_key.as_ref())
        .collect::<BTreeSet<_>>();
    for area in &catalog.areas {
        if area.source_id.is_empty()
            || area.difficulty_segment_keys.is_empty()
            || area.plane_keys.is_empty()
            || area.recommended_level == 0
            || area.recommended_elements.is_empty()
            || !unique_values(&area.recommended_elements)
            || area
                .difficulty_segment_keys
                .iter()
                .any(|key| !segment_keys.contains(key.as_ref()))
            || area
                .plane_keys
                .iter()
                .any(|key| !plane_keys.contains(key.as_ref()))
            || (area.kind == AreaKind::Formal && !(1..=5).contains(&area.difficulty))
            || (area.kind == AreaKind::Guide && area.difficulty != 1)
        {
            return fail(
                SwarmDisasterStructuralErrorKind::Reference,
                &area.stable_key,
            );
        }
    }
    for segment in &catalog.difficulty_segments {
        if segment.source_id.is_empty()
            || segment.cut_positions.is_empty()
            || segment.levels.is_empty()
            || !strictly_increasing(&segment.cut_positions)
            || !strictly_increasing(&segment.levels)
        {
            return fail(
                SwarmDisasterStructuralErrorKind::Metadata,
                &segment.stable_key,
            );
        }
    }
    for plane in &catalog.planes {
        if plane.source_id.is_empty()
            || !(1..=3).contains(&plane.plane_number)
            || plane.chessboard_keys.is_empty()
            || !unique_values(&plane.chessboard_keys)
            || plane
                .chessboard_keys
                .iter()
                .any(|key| !board_keys.contains(key.as_ref()))
        {
            return fail(
                SwarmDisasterStructuralErrorKind::Reference,
                &plane.stable_key,
            );
        }
    }
    Ok(())
}

fn validate_leaf_rows(
    catalog: &SwarmDisasterStructuralCatalog,
) -> Result<(), SwarmDisasterStructuralError> {
    unique(
        catalog.rooms.iter().map(|row| row.stable_key.as_ref()),
        "rooms",
    )?;
    unique(
        catalog.domains.iter().map(|row| row.stable_key.as_ref()),
        "domains",
    )?;
    unique(
        catalog.beacons.iter().map(|row| row.stable_key.as_ref()),
        "beacons",
    )?;
    unique(
        catalog
            .boss_choices
            .iter()
            .map(|row| row.stable_key.as_ref()),
        "boss-choices",
    )?;
    for room in &catalog.rooms {
        if room.source_id.is_empty()
            || room.sub_mode.as_ref() != SUB_MODE
            || room.sections.is_empty()
            || !unique_values(&room.sections)
        {
            return fail(SwarmDisasterStructuralErrorKind::Metadata, &room.stable_key);
        }
    }
    for domain in &catalog.domains {
        if domain.source_id.is_empty() {
            return fail(
                SwarmDisasterStructuralErrorKind::Metadata,
                &domain.stable_key,
            );
        }
    }
    for beacon in &catalog.beacons {
        if beacon.source_id.is_empty() || beacon.block_intro_id.is_empty() {
            return fail(
                SwarmDisasterStructuralErrorKind::Metadata,
                &beacon.stable_key,
            );
        }
    }
    for boss in &catalog.boss_choices {
        if boss.source_id.as_ref() != boss.enemy_variant_id.as_ref()
            || boss.display_level == 0
            || boss.weakness_elements.is_empty()
            || !unique_values(&boss.weakness_elements)
        {
            return fail(SwarmDisasterStructuralErrorKind::Metadata, &boss.stable_key);
        }
    }
    Ok(())
}

fn validate_graphs(
    catalog: &SwarmDisasterStructuralCatalog,
) -> Result<(), SwarmDisasterStructuralError> {
    unique(
        catalog
            .chessboards
            .iter()
            .map(|row| row.stable_key.as_ref()),
        "chessboards",
    )?;
    unique(
        catalog.columns.iter().map(|row| row.stable_key.as_ref()),
        "columns",
    )?;
    unique(
        catalog.nodes.iter().map(|row| row.stable_key.as_ref()),
        "nodes",
    )?;
    unique(
        catalog.edges.iter().map(|row| row.stable_key.as_ref()),
        "edges",
    )?;
    let columns = catalog
        .columns
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    let nodes = catalog
        .nodes
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    let node_keys = catalog
        .nodes
        .iter()
        .map(|row| (row.stable_key.as_ref(), row))
        .collect::<BTreeMap<_, _>>();
    let domain_keys = catalog
        .domains
        .iter()
        .map(|row| row.stable_key.as_ref())
        .collect::<BTreeSet<_>>();
    for board in &catalog.chessboards {
        if board.source_id.is_empty()
            || board.width == 0
            || board.height == 0
            || board.block_create_group.is_empty()
        {
            return fail(
                SwarmDisasterStructuralErrorKind::Metadata,
                &board.stable_key,
            );
        }
        let board_columns = catalog
            .columns
            .iter()
            .filter(|column| column.chessboard == board.id)
            .collect::<Vec<_>>();
        let board_nodes = catalog
            .nodes
            .iter()
            .filter(|node| node.chessboard == board.id)
            .collect::<Vec<_>>();
        if board_columns.is_empty()
            || board_nodes.is_empty()
            || board_columns
                .iter()
                .enumerate()
                .any(|(index, column)| usize::from(column.index) != index)
            || board_columns
                .iter()
                .any(|column| column.node_keys.len() > usize::from(board.height))
        {
            return fail(SwarmDisasterStructuralErrorKind::Graph, &board.stable_key);
        }
        let mut column_positions = BTreeSet::new();
        let mut listed_nodes = BTreeSet::new();
        for column in board_columns {
            if column.node_keys.is_empty()
                || !column_positions.insert(column.position_x)
                || column.node_keys.iter().any(|key| {
                    let Some(node) = node_keys.get(key.as_ref()) else {
                        return true;
                    };
                    node.chessboard != board.id
                        || node.column != column.id
                        || node.position_x != column.position_x
                        || !listed_nodes.insert(node.id)
                })
            {
                return fail(
                    SwarmDisasterStructuralErrorKind::Reference,
                    &column.stable_key,
                );
            }
        }
        let start = nodes.get(&board.start);
        let end = nodes.get(&board.end);
        if start.is_none_or(|node| node.chessboard != board.id || !node.is_start)
            || end.is_none_or(|node| node.chessboard != board.id || !node.is_end)
            || board_nodes.iter().filter(|node| node.is_start).count() != 1
            || board_nodes.iter().filter(|node| node.is_end).count() != 1
        {
            return fail(SwarmDisasterStructuralErrorKind::Graph, &board.stable_key);
        }
        for node in &board_nodes {
            let Some(column) = columns.get(&node.column) else {
                return fail(
                    SwarmDisasterStructuralErrorKind::Reference,
                    &node.stable_key,
                );
            };
            if column.chessboard != board.id
                || node
                    .domain_keys
                    .iter()
                    .any(|id| !domain_keys.contains(id.as_ref()))
                || (node.domain_resolution == DomainResolution::AuthoredCandidates
                    && node.domain_keys.is_empty())
                || (node.domain_resolution == DomainResolution::Unspecified
                    && !node.domain_keys.is_empty())
            {
                return fail(
                    SwarmDisasterStructuralErrorKind::Reference,
                    &node.stable_key,
                );
            }
        }
        if listed_nodes.len() != board_nodes.len() {
            return fail(
                SwarmDisasterStructuralErrorKind::Reference,
                &board.stable_key,
            );
        }
        validate_board_edges(board, catalog, &nodes, &columns)?;
    }
    Ok(())
}

fn validate_board_edges(
    board: &ChessboardDefinition,
    catalog: &SwarmDisasterStructuralCatalog,
    nodes: &BTreeMap<MapNodeId, &MapNodeDefinition>,
    columns: &BTreeMap<MapColumnId, &MapColumnDefinition>,
) -> Result<(), SwarmDisasterStructuralError> {
    let edges = catalog
        .edges
        .iter()
        .filter(|edge| edge.chessboard == board.id)
        .collect::<Vec<_>>();
    let mut adjacency = BTreeMap::<MapNodeId, Vec<MapNodeId>>::new();
    let mut reverse = BTreeMap::<MapNodeId, Vec<MapNodeId>>::new();
    let mut pairs = BTreeSet::new();
    for edge in edges {
        let (Some(source), Some(target)) = (nodes.get(&edge.source), nodes.get(&edge.target))
        else {
            return fail(
                SwarmDisasterStructuralErrorKind::Reference,
                &edge.stable_key,
            );
        };
        let (Some(source_column), Some(target_column)) =
            (columns.get(&source.column), columns.get(&target.column))
        else {
            return fail(
                SwarmDisasterStructuralErrorKind::Reference,
                &edge.stable_key,
            );
        };
        if source.chessboard != board.id
            || target.chessboard != board.id
            || target_column.index != source_column.index + 1
            || !pairs.insert((edge.source, edge.target))
        {
            return fail(SwarmDisasterStructuralErrorKind::Graph, &edge.stable_key);
        }
        adjacency.entry(edge.source).or_default().push(edge.target);
        reverse.entry(edge.target).or_default().push(edge.source);
    }
    let from_start = reachable(board.start, &adjacency);
    let reaches_end = reachable(board.end, &reverse);
    if !from_start.contains(&board.end) || !reaches_end.contains(&board.start) {
        return fail(SwarmDisasterStructuralErrorKind::Graph, &board.stable_key);
    }
    Ok(())
}

fn reachable(start: MapNodeId, edges: &BTreeMap<MapNodeId, Vec<MapNodeId>>) -> BTreeSet<MapNodeId> {
    let mut reached = BTreeSet::from([start]);
    let mut pending = VecDeque::from([start]);
    while let Some(source) = pending.pop_front() {
        for target in edges.get(&source).into_iter().flatten() {
            if reached.insert(*target) {
                pending.push_back(*target);
            }
        }
    }
    reached
}

pub(super) fn difficulty(value: &str, key: &str) -> Result<u8, SwarmDisasterStructuralError> {
    match value {
        "Difficulty_1" => Ok(1),
        "Difficulty_2" => Ok(2),
        "Difficulty_3" => Ok(3),
        "Difficulty_4" => Ok(4),
        "Difficulty_5" => Ok(5),
        _ => invalid_value(key),
    }
}

pub(super) fn element(
    value: &str,
    key: &str,
) -> Result<RecommendedElement, SwarmDisasterStructuralError> {
    match value {
        "Physical" => Ok(RecommendedElement::Physical),
        "Fire" => Ok(RecommendedElement::Fire),
        "Ice" => Ok(RecommendedElement::Ice),
        "Thunder" => Ok(RecommendedElement::Lightning),
        "Wind" => Ok(RecommendedElement::Wind),
        "Quantum" => Ok(RecommendedElement::Quantum),
        "Imaginary" => Ok(RecommendedElement::Imaginary),
        _ => invalid_value(key),
    }
}

fn metadata(key: &str, revision: &str, kind: &str) -> Result<(), SwarmDisasterStructuralError> {
    stable(key)?;
    if revision != ROW_REVISION || kind.is_empty() {
        return fail(SwarmDisasterStructuralErrorKind::Metadata, key);
    }
    Ok(())
}

fn stable(value: &str) -> Result<Box<str>, SwarmDisasterStructuralError> {
    if value.is_empty() || value.len() > 256 || !value.bytes().all(|byte| byte.is_ascii_graphic()) {
        return fail(SwarmDisasterStructuralErrorKind::Identifier, value);
    }
    Ok(value.into())
}

fn nonempty(value: &str, key: &str) -> Result<Box<str>, SwarmDisasterStructuralError> {
    if value.trim().is_empty() {
        return invalid_value(key);
    }
    Ok(value.into())
}

fn text_list(
    values: &[String],
    key: &str,
) -> Result<Box<[Box<str>]>, SwarmDisasterStructuralError> {
    values
        .iter()
        .map(|value| nonempty(value, key))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn optional_text_list(
    values: Option<&[String]>,
    key: &str,
) -> Result<Box<[Box<str>]>, SwarmDisasterStructuralError> {
    values.map_or_else(
        || Ok(Vec::<Box<str>>::new().into_boxed_slice()),
        |values| text_list(values, key),
    )
}

fn numeric_list(values: &[String], key: &str) -> Result<Box<[u16]>, SwarmDisasterStructuralError> {
    values
        .iter()
        .map(|value| {
            value
                .parse::<u16>()
                .ok()
                .filter(|value| *value > 0)
                .ok_or_else(|| error(SwarmDisasterStructuralErrorKind::Identifier, key))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn nonnegative_numeric_list(
    values: &[String],
    key: &str,
) -> Result<Box<[u16]>, SwarmDisasterStructuralError> {
    values
        .iter()
        .map(|value| {
            value
                .parse::<u16>()
                .map_err(|_| error(SwarmDisasterStructuralErrorKind::Identifier, key))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn positive(value: i32, key: &str) -> Result<u32, SwarmDisasterStructuralError> {
    u32::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| error(SwarmDisasterStructuralErrorKind::Identifier, key))
}

fn positive_u16(value: i32, key: &str) -> Result<u16, SwarmDisasterStructuralError> {
    u16::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| error(SwarmDisasterStructuralErrorKind::Identifier, key))
}

fn positive_u8(value: i32, key: &str) -> Result<u8, SwarmDisasterStructuralError> {
    u8::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| error(SwarmDisasterStructuralErrorKind::Identifier, key))
}

fn nonnegative_u16(value: i32, key: &str) -> Result<u16, SwarmDisasterStructuralError> {
    u16::try_from(value).map_err(|_| error(SwarmDisasterStructuralErrorKind::Identifier, key))
}

fn sequential(
    values: impl Iterator<Item = u32>,
    key: &str,
) -> Result<(), SwarmDisasterStructuralError> {
    if values.enumerate().any(|(index, value)| {
        u32::try_from(index)
            .ok()
            .and_then(|index| index.checked_add(1))
            != Some(value)
    }) {
        return fail(SwarmDisasterStructuralErrorKind::Identifier, key);
    }
    Ok(())
}

fn unique<'a>(
    mut values: impl Iterator<Item = &'a str>,
    key: &str,
) -> Result<(), SwarmDisasterStructuralError> {
    let mut found = BTreeSet::new();
    if values.any(|value| !found.insert(value)) {
        return fail(SwarmDisasterStructuralErrorKind::Duplicate, key);
    }
    Ok(())
}

fn unique_values<T: Ord>(values: &[T]) -> bool {
    values.iter().collect::<BTreeSet<_>>().len() == values.len()
}

fn strictly_increasing(values: &[u16]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn invalid_value<T>(key: &str) -> Result<T, SwarmDisasterStructuralError> {
    fail(SwarmDisasterStructuralErrorKind::Metadata, key)
}

fn fail<T>(
    kind: SwarmDisasterStructuralErrorKind,
    key: &str,
) -> Result<T, SwarmDisasterStructuralError> {
    Err(error(kind, key))
}

fn error(kind: SwarmDisasterStructuralErrorKind, key: &str) -> SwarmDisasterStructuralError {
    SwarmDisasterStructuralError {
        kind,
        key: key.into(),
    }
}
