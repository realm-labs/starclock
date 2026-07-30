use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{
    definition::RecommendedElement,
    gold_gears_catalog::GoldAndGearsBundleSummary,
    gold_gears_generated::{
        SoraConfig, gold_gears_area::GoldGearsArea, gold_gears_beacon::GoldGearsBeacon,
        gold_gears_boss_choice::GoldGearsBossChoice, gold_gears_chessboard::GoldGearsChessboard,
        gold_gears_difficulty_segment::GoldGearsDifficultySegment,
        gold_gears_domain::GoldGearsDomain, gold_gears_map_column::GoldGearsMapColumn,
        gold_gears_map_edge::GoldGearsMapEdge, gold_gears_map_node::GoldGearsMapNode,
        gold_gears_plane::GoldGearsPlane, gold_gears_profile::GoldGearsProfile,
        gold_gears_room::GoldGearsRoom,
    },
};

use super::{
    EXPECTED_STRUCTURAL_ROWS, GoldAndGearsStructuralCatalog, GoldAndGearsStructuralError,
    GoldAndGearsStructuralErrorKind,
    types::{
        AreaDefinition, AreaGroup, AreaId, BeaconDefinition, BeaconId, BossChoiceDefinition,
        BossChoiceId, ChessboardDefinition, ChessboardId, DifficultySegmentDefinition,
        DifficultySegmentId, DomainDefinition, DomainId, DomainResolution, MapColumnDefinition,
        MapColumnId, MapEdgeDefinition, MapEdgeId, MapNodeDefinition, MapNodeId, PlaneDefinition,
        PlaneId, ProfileDefinition, ProfileId, ProfileRowKind, RoomDefinition, RoomId,
    },
};

const ROW_REVISION: &str = "starclock.gold-and-gears-row.v1";
const SUB_MODE: &str = "ChessRogueNous";
const EDGE_POLICY: &str = "forward-nearest-column-within-one-row-v1";

pub(super) fn lower(
    bundle: GoldAndGearsBundleSummary,
    source: &SoraConfig,
) -> Result<GoldAndGearsStructuralCatalog, GoldAndGearsStructuralError> {
    let catalog = GoldAndGearsStructuralCatalog {
        bundle,
        profiles: source
            .gold_gears_profile()
            .ordered_rows()
            .map(lower_profile)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        areas: source
            .gold_gears_area()
            .ordered_rows()
            .map(lower_area)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        difficulty_segments: source
            .gold_gears_difficulty_segment()
            .ordered_rows()
            .map(lower_difficulty_segment)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        planes: source
            .gold_gears_plane()
            .ordered_rows()
            .map(lower_plane)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        chessboards: source
            .gold_gears_chessboard()
            .ordered_rows()
            .map(lower_chessboard)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        columns: source
            .gold_gears_map_column()
            .ordered_rows()
            .map(lower_column)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        nodes: source
            .gold_gears_map_node()
            .ordered_rows()
            .map(lower_node)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        edges: source
            .gold_gears_map_edge()
            .ordered_rows()
            .map(lower_edge)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        rooms: source
            .gold_gears_room()
            .ordered_rows()
            .map(lower_room)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        domains: source
            .gold_gears_domain()
            .ordered_rows()
            .map(lower_domain)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        beacons: source
            .gold_gears_beacon()
            .ordered_rows()
            .map(lower_beacon)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        boss_choices: source
            .gold_gears_boss_choice()
            .ordered_rows()
            .map(lower_boss_choice)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
    };
    validate(&catalog)?;
    Ok(catalog)
}

fn lower_profile(row: &GoldGearsProfile) -> Result<ProfileDefinition, GoldAndGearsStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    let kind = match (row.kind.as_str(), row.entry_kind.as_deref()) {
        ("EntryPoint", Some("ResidentActivity")) => ProfileRowKind::ResidentActivity,
        ("EntryPoint", Some("DlcEntrance")) => ProfileRowKind::DlcEntrance,
        ("EntryPoint", Some("ModeTitle")) => ProfileRowKind::ModeTitle,
        ("Profile", None) => ProfileRowKind::RuntimeProfile,
        _ => return invalid_value(&row.stable_key),
    };
    Ok(ProfileDefinition {
        id: ProfileId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        kind,
        source_id: optional_text(row.source_id.as_deref(), &row.stable_key)?,
        unlock_id: optional_text(row.unlock_id.as_deref(), &row.stable_key)?,
        sub_mode: nonempty(&row.sub_mode, &row.stable_key)?,
        game_version: optional_text(row.game_version.as_deref(), &row.stable_key)?,
        reference_runtime_enabled: row.runtime_enabled,
    })
}

fn lower_area(row: &GoldGearsArea) -> Result<AreaDefinition, GoldAndGearsStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    let group = match row.area_group.as_str() {
        "Formal" => AreaGroup::Formal,
        "Guide" => AreaGroup::Guide,
        _ => return invalid_value(&row.stable_key),
    };
    Ok(AreaDefinition {
        id: AreaId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        source_id: nonempty(&row.source_id, &row.stable_key)?,
        group,
        difficulty: difficulty(&row.difficulty, &row.stable_key)?,
        difficulty_segment_sources: text_list(&row.difficulty_segment_ids, &row.stable_key)?,
        plane_sources: text_list(&row.plane_ids, &row.stable_key)?,
        unlock_id: nonempty(&row.unlock_id, &row.stable_key)?,
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
    row: &GoldGearsDifficultySegment,
) -> Result<DifficultySegmentDefinition, GoldAndGearsStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(DifficultySegmentDefinition {
        id: DifficultySegmentId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        source_id: nonempty(&row.source_id, &row.stable_key)?,
        cut_positions: numeric_list(&row.cut_positions, &row.stable_key)?,
        levels: numeric_list(&row.levels, &row.stable_key)?,
    })
}

fn lower_plane(row: &GoldGearsPlane) -> Result<PlaneDefinition, GoldAndGearsStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(PlaneDefinition {
        id: PlaneId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        source_id: nonempty(&row.source_id, &row.stable_key)?,
    })
}

fn lower_chessboard(
    row: &GoldGearsChessboard,
) -> Result<ChessboardDefinition, GoldAndGearsStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    nonempty(&row.config_path, &row.stable_key)?;
    Ok(ChessboardDefinition {
        id: ChessboardId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        source_id: nonempty(&row.source_id, &row.stable_key)?,
        width: positive_u16(row.width, &row.stable_key)?,
        height: positive_u16(row.height, &row.stable_key)?,
        start: MapNodeId(positive(row.start_node_id, &row.stable_key)?),
        end: MapNodeId(positive(row.end_node_id, &row.stable_key)?),
        block_create_group: nonempty(&row.block_create_group_id, &row.stable_key)?,
        event_sources: optional_text_list(row.event_ids.as_deref(), &row.stable_key)?,
    })
}

fn lower_column(
    row: &GoldGearsMapColumn,
) -> Result<MapColumnDefinition, GoldAndGearsStructuralError> {
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

fn lower_node(row: &GoldGearsMapNode) -> Result<MapNodeDefinition, GoldAndGearsStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    let domain_resolution = match row.domain_resolution.as_str() {
        "AuthoredCandidates" => DomainResolution::AuthoredCandidates,
        "Unspecified" => DomainResolution::Unspecified,
        _ => return invalid_value(&row.stable_key),
    };
    Ok(MapNodeDefinition {
        id: MapNodeId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        source_id: nonempty(&row.source_id, &row.stable_key)?,
        chessboard: ChessboardId(positive(row.chessboard_id, &row.stable_key)?),
        column: MapColumnId(positive(row.column_id, &row.stable_key)?),
        position_x: row.position_x,
        position_y: row.position_y,
        domains: optional_text_list(row.domain_ids.as_deref(), &row.stable_key)?,
        domain_resolution,
        is_start: row.is_start,
        is_end: row.is_end,
    })
}

fn lower_edge(row: &GoldGearsMapEdge) -> Result<MapEdgeDefinition, GoldAndGearsStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(MapEdgeDefinition {
        id: MapEdgeId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        chessboard: ChessboardId(positive(row.chessboard_id, &row.stable_key)?),
        source: MapNodeId(positive(row.source_node_id, &row.stable_key)?),
        target: MapNodeId(positive(row.target_node_id, &row.stable_key)?),
        policy: nonempty(&row.policy, &row.stable_key)?,
    })
}

fn lower_room(row: &GoldGearsRoom) -> Result<RoomDefinition, GoldAndGearsStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(RoomDefinition {
        id: RoomId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        source_id: nonempty(&row.source_id, &row.stable_key)?,
        sub_mode: nonempty(&row.sub_mode, &row.stable_key)?,
        sections: nonnegative_numeric_list(&row.section_ids, &row.stable_key)?,
    })
}

fn lower_domain(row: &GoldGearsDomain) -> Result<DomainDefinition, GoldAndGearsStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(DomainDefinition {
        id: DomainId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        source_id: nonempty(&row.source_id, &row.stable_key)?,
    })
}

fn lower_beacon(row: &GoldGearsBeacon) -> Result<BeaconDefinition, GoldAndGearsStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(BeaconDefinition {
        id: BeaconId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        source_id: nonempty(&row.source_id, &row.stable_key)?,
    })
}

fn lower_boss_choice(
    row: &GoldGearsBossChoice,
) -> Result<BossChoiceDefinition, GoldAndGearsStructuralError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(BossChoiceDefinition {
        id: BossChoiceId(positive(row.id, &row.stable_key)?),
        stable_key: stable(&row.stable_key)?,
        source_id: nonempty(&row.source_id, &row.stable_key)?,
        display_level: positive_u16(row.display_level, &row.stable_key)?,
        weakness_elements: row
            .weakness_elements
            .iter()
            .map(|value| element(value, &row.stable_key))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        monster_template_id: nonempty(&row.monster_template_id, &row.stable_key)?,
    })
}

fn validate(catalog: &GoldAndGearsStructuralCatalog) -> Result<(), GoldAndGearsStructuralError> {
    let counts = [
        (catalog.profiles.len(), 4, "profiles"),
        (catalog.areas.len(), 8, "areas"),
        (catalog.difficulty_segments.len(), 16, "difficulty-segments"),
        (catalog.planes.len(), 8, "planes"),
        (catalog.chessboards.len(), 115, "chessboards"),
        (catalog.columns.len(), 1_313, "columns"),
        (catalog.nodes.len(), 2_502, "nodes"),
        (catalog.edges.len(), 3_407, "edges"),
        (catalog.rooms.len(), 1_224, "rooms"),
        (catalog.domains.len(), 12, "domains"),
        (catalog.beacons.len(), 6, "beacons"),
        (catalog.boss_choices.len(), 6, "boss-choices"),
    ];
    if counts
        .iter()
        .any(|(actual, expected, _)| actual != expected)
        || catalog.row_count() != EXPECTED_STRUCTURAL_ROWS
    {
        let key = counts
            .iter()
            .find(|(actual, expected, _)| actual != expected)
            .map_or("structural-total", |(_, _, key)| *key);
        return fail(GoldAndGearsStructuralErrorKind::Denominator, key);
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
    validate_area_references(catalog)?;
    validate_leaf_rows(catalog)?;
    validate_graphs(catalog)
}

fn validate_profiles(
    catalog: &GoldAndGearsStructuralCatalog,
) -> Result<(), GoldAndGearsStructuralError> {
    unique(
        catalog.profiles.iter().map(|row| row.stable_key.as_ref()),
        "profiles",
    )?;
    let kinds = [
        ProfileRowKind::ResidentActivity,
        ProfileRowKind::DlcEntrance,
        ProfileRowKind::ModeTitle,
        ProfileRowKind::RuntimeProfile,
    ];
    for kind in kinds {
        if catalog
            .profiles
            .iter()
            .filter(|row| row.kind == kind)
            .count()
            != 1
        {
            return fail(GoldAndGearsStructuralErrorKind::Metadata, "profile-kind");
        }
    }
    for row in &catalog.profiles {
        if row.sub_mode.as_ref() != SUB_MODE {
            return fail(GoldAndGearsStructuralErrorKind::Metadata, &row.stable_key);
        }
        match row.kind {
            ProfileRowKind::RuntimeProfile
                if row.source_id.is_none()
                    && row.unlock_id.is_none()
                    && row.game_version.as_deref() == Some("4.4")
                    && row.reference_runtime_enabled == Some(false) => {}
            ProfileRowKind::ResidentActivity
                if row.source_id.as_deref() == Some("102")
                    && row.unlock_id.as_deref() == Some("50019")
                    && row.game_version.is_none()
                    && row.reference_runtime_enabled.is_none() => {}
            ProfileRowKind::DlcEntrance | ProfileRowKind::ModeTitle
                if row.source_id.is_some()
                    && row.unlock_id.is_none()
                    && row.game_version.is_none()
                    && row.reference_runtime_enabled.is_none() => {}
            _ => return fail(GoldAndGearsStructuralErrorKind::Metadata, &row.stable_key),
        }
    }
    Ok(())
}

fn validate_area_references(
    catalog: &GoldAndGearsStructuralCatalog,
) -> Result<(), GoldAndGearsStructuralError> {
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
    let segments = catalog
        .difficulty_segments
        .iter()
        .map(|row| row.source_id.as_ref())
        .collect::<BTreeSet<_>>();
    let planes = catalog
        .planes
        .iter()
        .map(|row| row.source_id.as_ref())
        .collect::<BTreeSet<_>>();
    for segment in &catalog.difficulty_segments {
        if segment.levels.is_empty()
            || segment.cut_positions.len() + 1 != segment.levels.len()
            || !strictly_increasing(&segment.cut_positions)
            || !strictly_increasing(&segment.levels)
        {
            return fail(
                GoldAndGearsStructuralErrorKind::Metadata,
                &segment.stable_key,
            );
        }
    }
    for area in &catalog.areas {
        if area.difficulty_segment_sources.is_empty()
            || area.plane_sources.is_empty()
            || area.unlock_id.is_empty()
            || area.recommended_level == 0
            || area.recommended_elements.is_empty()
            || !unique_values(&area.recommended_elements)
            || area
                .difficulty_segment_sources
                .iter()
                .any(|id| !segments.contains(id.as_ref()))
            || area
                .plane_sources
                .iter()
                .any(|id| !planes.contains(id.as_ref()))
            || !unique_values(&area.difficulty_segment_sources)
            || !unique_values(&area.plane_sources)
        {
            return fail(GoldAndGearsStructuralErrorKind::Reference, &area.stable_key);
        }
        match area.group {
            AreaGroup::Formal if area.difficulty <= 5 => {}
            AreaGroup::Guide if area.difficulty == 1 => {}
            _ => return fail(GoldAndGearsStructuralErrorKind::Metadata, &area.stable_key),
        }
    }
    Ok(())
}

fn validate_leaf_rows(
    catalog: &GoldAndGearsStructuralCatalog,
) -> Result<(), GoldAndGearsStructuralError> {
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
            return fail(GoldAndGearsStructuralErrorKind::Metadata, &room.stable_key);
        }
    }
    for domain in &catalog.domains {
        if domain.source_id.is_empty() {
            return fail(
                GoldAndGearsStructuralErrorKind::Metadata,
                &domain.stable_key,
            );
        }
    }
    for beacon in &catalog.beacons {
        if beacon.source_id.is_empty() {
            return fail(
                GoldAndGearsStructuralErrorKind::Metadata,
                &beacon.stable_key,
            );
        }
    }
    for boss in &catalog.boss_choices {
        if boss.source_id.as_ref() != boss.monster_template_id.as_ref()
            || boss.display_level == 0
            || boss.weakness_elements.is_empty()
            || !unique_values(&boss.weakness_elements)
        {
            return fail(GoldAndGearsStructuralErrorKind::Metadata, &boss.stable_key);
        }
    }
    Ok(())
}

fn validate_graphs(
    catalog: &GoldAndGearsStructuralCatalog,
) -> Result<(), GoldAndGearsStructuralError> {
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
            || board.block_create_group.is_empty()
            || board.event_sources.iter().any(|value| value.is_empty())
        {
            return fail(GoldAndGearsStructuralErrorKind::Metadata, &board.stable_key);
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
            return fail(GoldAndGearsStructuralErrorKind::Graph, &board.stable_key);
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
                    GoldAndGearsStructuralErrorKind::Reference,
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
            return fail(GoldAndGearsStructuralErrorKind::Graph, &board.stable_key);
        }
        for node in &board_nodes {
            let Some(column) = columns.get(&node.column) else {
                return fail(GoldAndGearsStructuralErrorKind::Reference, &node.stable_key);
            };
            if column.chessboard != board.id
                || node
                    .domains
                    .iter()
                    .any(|id| !domain_keys.contains(id.as_ref()))
                || (node.domain_resolution == DomainResolution::AuthoredCandidates
                    && node.domains.is_empty())
                || (node.domain_resolution == DomainResolution::Unspecified
                    && !node.domains.is_empty())
            {
                return fail(GoldAndGearsStructuralErrorKind::Reference, &node.stable_key);
            }
        }
        if listed_nodes.len() != board_nodes.len() {
            return fail(
                GoldAndGearsStructuralErrorKind::Reference,
                &board.stable_key,
            );
        }
        validate_board_edges(board, catalog, &nodes, &columns)?;
    }
    Ok(())
}

fn validate_board_edges(
    board: &ChessboardDefinition,
    catalog: &GoldAndGearsStructuralCatalog,
    nodes: &BTreeMap<MapNodeId, &MapNodeDefinition>,
    columns: &BTreeMap<MapColumnId, &MapColumnDefinition>,
) -> Result<(), GoldAndGearsStructuralError> {
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
            return fail(GoldAndGearsStructuralErrorKind::Reference, &edge.stable_key);
        };
        let (Some(source_column), Some(target_column)) =
            (columns.get(&source.column), columns.get(&target.column))
        else {
            return fail(GoldAndGearsStructuralErrorKind::Reference, &edge.stable_key);
        };
        if source.chessboard != board.id
            || target.chessboard != board.id
            || edge.policy.as_ref() != EDGE_POLICY
            || target_column.index != source_column.index + 1
            || !pairs.insert((edge.source, edge.target))
        {
            return fail(GoldAndGearsStructuralErrorKind::Graph, &edge.stable_key);
        }
        adjacency.entry(edge.source).or_default().push(edge.target);
        reverse.entry(edge.target).or_default().push(edge.source);
    }
    let from_start = reachable(board.start, &adjacency);
    let reaches_end = reachable(board.end, &reverse);
    if !from_start.contains(&board.end) || !reaches_end.contains(&board.start) {
        return fail(GoldAndGearsStructuralErrorKind::Graph, &board.stable_key);
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

pub(super) fn difficulty(value: &str, key: &str) -> Result<u8, GoldAndGearsStructuralError> {
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
) -> Result<RecommendedElement, GoldAndGearsStructuralError> {
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

fn metadata(key: &str, revision: &str, kind: &str) -> Result<(), GoldAndGearsStructuralError> {
    stable(key)?;
    if revision != ROW_REVISION || kind.is_empty() {
        return fail(GoldAndGearsStructuralErrorKind::Metadata, key);
    }
    Ok(())
}

fn stable(value: &str) -> Result<Box<str>, GoldAndGearsStructuralError> {
    if value.is_empty() || value.len() > 256 || !value.bytes().all(|byte| byte.is_ascii_graphic()) {
        return fail(GoldAndGearsStructuralErrorKind::Identifier, value);
    }
    Ok(value.into())
}

fn nonempty(value: &str, key: &str) -> Result<Box<str>, GoldAndGearsStructuralError> {
    if value.trim().is_empty() {
        return invalid_value(key);
    }
    Ok(value.into())
}

fn optional_text(
    value: Option<&str>,
    key: &str,
) -> Result<Option<Box<str>>, GoldAndGearsStructuralError> {
    value.map(|value| nonempty(value, key)).transpose()
}

fn text_list(values: &[String], key: &str) -> Result<Box<[Box<str>]>, GoldAndGearsStructuralError> {
    values
        .iter()
        .map(|value| nonempty(value, key))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn optional_text_list(
    values: Option<&[String]>,
    key: &str,
) -> Result<Box<[Box<str>]>, GoldAndGearsStructuralError> {
    values.map_or_else(
        || Ok(Vec::<Box<str>>::new().into_boxed_slice()),
        |values| text_list(values, key),
    )
}

fn numeric_list(values: &[String], key: &str) -> Result<Box<[u16]>, GoldAndGearsStructuralError> {
    values
        .iter()
        .map(|value| {
            value
                .parse::<u16>()
                .ok()
                .filter(|value| *value > 0)
                .ok_or_else(|| error(GoldAndGearsStructuralErrorKind::Identifier, key))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn nonnegative_numeric_list(
    values: &[String],
    key: &str,
) -> Result<Box<[u16]>, GoldAndGearsStructuralError> {
    values
        .iter()
        .map(|value| {
            value
                .parse::<u16>()
                .map_err(|_| error(GoldAndGearsStructuralErrorKind::Identifier, key))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn positive(value: i32, key: &str) -> Result<u32, GoldAndGearsStructuralError> {
    u32::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| error(GoldAndGearsStructuralErrorKind::Identifier, key))
}

fn positive_u16(value: i32, key: &str) -> Result<u16, GoldAndGearsStructuralError> {
    u16::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| error(GoldAndGearsStructuralErrorKind::Identifier, key))
}

fn nonnegative_u16(value: i32, key: &str) -> Result<u16, GoldAndGearsStructuralError> {
    u16::try_from(value).map_err(|_| error(GoldAndGearsStructuralErrorKind::Identifier, key))
}

fn sequential(
    values: impl Iterator<Item = u32>,
    key: &str,
) -> Result<(), GoldAndGearsStructuralError> {
    if values.enumerate().any(|(index, value)| {
        u32::try_from(index)
            .ok()
            .and_then(|index| index.checked_add(1))
            != Some(value)
    }) {
        return fail(GoldAndGearsStructuralErrorKind::Identifier, key);
    }
    Ok(())
}

fn unique<'a>(
    mut values: impl Iterator<Item = &'a str>,
    key: &str,
) -> Result<(), GoldAndGearsStructuralError> {
    let mut found = BTreeSet::new();
    if values.any(|value| !found.insert(value)) {
        return fail(GoldAndGearsStructuralErrorKind::Duplicate, key);
    }
    Ok(())
}

fn unique_values<T: Ord>(values: &[T]) -> bool {
    values.iter().collect::<BTreeSet<_>>().len() == values.len()
}

fn strictly_increasing(values: &[u16]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn invalid_value<T>(key: &str) -> Result<T, GoldAndGearsStructuralError> {
    fail(GoldAndGearsStructuralErrorKind::Metadata, key)
}

fn fail<T>(
    kind: GoldAndGearsStructuralErrorKind,
    key: &str,
) -> Result<T, GoldAndGearsStructuralError> {
    Err(error(kind, key))
}

fn error(kind: GoldAndGearsStructuralErrorKind, key: &str) -> GoldAndGearsStructuralError {
    GoldAndGearsStructuralError {
        kind,
        key: key.into(),
    }
}
