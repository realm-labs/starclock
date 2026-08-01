use std::collections::{BTreeMap, BTreeSet};

use super::{
    EXPECTED_UNIQUE_ROWS, SwarmDisasterUniqueCatalog, SwarmDisasterUniqueError,
    SwarmDisasterUniqueErrorKind, types::TrailNodeId,
};

pub(super) fn catalog(
    catalog: &SwarmDisasterUniqueCatalog,
) -> Result<(), SwarmDisasterUniqueError> {
    let counts = [
        (catalog.countdown.len(), 1, "countdown"),
        (catalog.boss_decay_levels.len(), 42, "boss-decay-levels"),
        (catalog.audience_paths.len(), 8, "audience-paths"),
        (catalog.audience_dice.len(), 8, "audience-dice"),
        (catalog.dice_rarities.len(), 3, "dice-rarities"),
        (catalog.dice_faces.len(), 42, "dice-faces"),
        (catalog.dice_targets.len(), 42, "dice-targets"),
        (catalog.dice_controls.len(), 4, "dice-controls"),
        (catalog.communing_choices.len(), 21, "communing-choices"),
        (
            catalog.communing_dimensions.len(),
            7,
            "communing-dimensions",
        ),
        (catalog.point_adjustments.len(), 55, "point-adjustments"),
        (catalog.trail_nodes.len(), 63, "trail-nodes"),
        (catalog.trail_prerequisites.len(), 56, "trail-prerequisites"),
        (catalog.trail_effects.len(), 63, "trail-effects"),
        (catalog.cabinets.len(), 31, "cabinets"),
        (catalog.objectives.len(), 31, "objectives"),
        (catalog.finish_conditions.len(), 102, "finish-conditions"),
        (catalog.unlocks.len(), 110, "unlocks"),
        (catalog.chapters.len(), 13, "chapters"),
        (catalog.bonuses.len(), 6, "bonuses"),
        (catalog.paths.len(), 8, "paths"),
        (catalog.path_boosts.len(), 8, "path-boosts"),
        (catalog.resonances.len(), 32, "resonances"),
        (catalog.interplays.len(), 16, "interplays"),
    ];
    if catalog.bundle.table_count() != 65
        || counts
            .iter()
            .any(|(actual, expected, _)| actual != expected)
        || catalog.row_count() != EXPECTED_UNIQUE_ROWS
    {
        let key = counts
            .iter()
            .find(|(actual, expected, _)| actual != expected)
            .map_or("unique-total", |(_, _, key)| *key);
        return fail(SwarmDisasterUniqueErrorKind::Denominator, key);
    }
    sequential(catalog.countdown.iter().map(|row| row.id.0), "countdown")?;
    sequential(
        catalog.boss_decay_levels.iter().map(|row| row.id.0),
        "boss-decay-levels",
    )?;
    sequential(
        catalog.audience_paths.iter().map(|row| row.id.0),
        "audience-paths",
    )?;
    sequential(
        catalog.audience_dice.iter().map(|row| row.id.0),
        "audience-dice",
    )?;
    sequential(
        catalog.dice_rarities.iter().map(|row| row.id.0),
        "dice-rarities",
    )?;
    sequential(catalog.dice_faces.iter().map(|row| row.id.0), "dice-faces")?;
    sequential(
        catalog.dice_targets.iter().map(|row| row.id.0),
        "dice-targets",
    )?;
    sequential(
        catalog.dice_controls.iter().map(|row| row.id.0),
        "dice-controls",
    )?;
    sequential(
        catalog.communing_choices.iter().map(|row| row.id.0),
        "communing-choices",
    )?;
    sequential(
        catalog.communing_dimensions.iter().map(|row| row.id.0),
        "communing-dimensions",
    )?;
    sequential(
        catalog.point_adjustments.iter().map(|row| row.id.0),
        "point-adjustments",
    )?;
    sequential(
        catalog.trail_nodes.iter().map(|row| row.id.0),
        "trail-nodes",
    )?;
    sequential(
        catalog.trail_prerequisites.iter().map(|row| row.id.0),
        "trail-prerequisites",
    )?;
    sequential(
        catalog.trail_effects.iter().map(|row| row.id.0),
        "trail-effects",
    )?;
    sequential(catalog.cabinets.iter().map(|row| row.id.0), "cabinets")?;
    sequential(catalog.objectives.iter().map(|row| row.id.0), "objectives")?;
    sequential(
        catalog.finish_conditions.iter().map(|row| row.id.0),
        "finish-conditions",
    )?;
    sequential(catalog.unlocks.iter().map(|row| row.id.0), "unlocks")?;
    sequential(catalog.chapters.iter().map(|row| row.id.0), "chapters")?;
    sequential(catalog.bonuses.iter().map(|row| row.id.0), "bonuses")?;
    sequential(catalog.paths.iter().map(|row| row.id.0), "paths")?;
    sequential(
        catalog.path_boosts.iter().map(|row| row.id.0),
        "path-boosts",
    )?;
    sequential(catalog.resonances.iter().map(|row| row.id.0), "resonances")?;
    sequential(catalog.interplays.iter().map(|row| row.id.0), "interplays")?;
    unique_keys(catalog)?;
    validate_countdown(catalog)?;
    validate_dice(catalog)?;
    validate_communing(catalog)?;
    validate_pathstrider(catalog)?;
    validate_paths(catalog)
}

fn unique_keys(catalog: &SwarmDisasterUniqueCatalog) -> Result<(), SwarmDisasterUniqueError> {
    macro_rules! unique_table {
        ($field:ident) => {
            unique(
                catalog.$field.iter().map(|row| row.key.as_ref()),
                stringify!($field),
            )?;
        };
    }
    unique_table!(countdown);
    unique_table!(boss_decay_levels);
    unique_table!(audience_paths);
    unique_table!(audience_dice);
    unique_table!(dice_rarities);
    unique_table!(dice_faces);
    unique_table!(dice_targets);
    unique_table!(dice_controls);
    unique_table!(communing_choices);
    unique_table!(communing_dimensions);
    unique_table!(point_adjustments);
    unique_table!(trail_nodes);
    unique_table!(trail_prerequisites);
    unique_table!(trail_effects);
    unique_table!(cabinets);
    unique_table!(objectives);
    unique_table!(finish_conditions);
    unique_table!(unlocks);
    unique_table!(chapters);
    unique_table!(bonuses);
    unique_table!(paths);
    unique_table!(path_boosts);
    unique_table!(resonances);
    unique_table!(interplays);
    Ok(())
}

fn validate_countdown(
    catalog: &SwarmDisasterUniqueCatalog,
) -> Result<(), SwarmDisasterUniqueError> {
    let countdown = &catalog.countdown[0];
    if countdown.initial.as_ref() != "20"
        || countdown.warning.as_ref() != "5"
        || countdown.movement_delta.as_ref() != "-1"
        || countdown.tiers.is_empty()
    {
        return fail(SwarmDisasterUniqueErrorKind::Metadata, &countdown.key);
    }
    for row in &catalog.boss_decay_levels {
        if row.threshold.is_empty() || row.tier.is_empty() || row.effect_program.is_empty() {
            return fail(SwarmDisasterUniqueErrorKind::Metadata, &row.key);
        }
    }
    Ok(())
}

fn validate_dice(catalog: &SwarmDisasterUniqueCatalog) -> Result<(), SwarmDisasterUniqueError> {
    let paths = catalog
        .audience_paths
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    let dice = catalog
        .audience_dice
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    let rarities = catalog
        .dice_rarities
        .iter()
        .map(|row| row.id)
        .collect::<BTreeSet<_>>();
    let targets = catalog
        .dice_targets
        .iter()
        .map(|row| row.id)
        .collect::<BTreeSet<_>>();
    let face_keys = catalog
        .dice_faces
        .iter()
        .map(|row| row.key.as_ref())
        .collect::<BTreeSet<_>>();
    let mut listed_faces = BTreeSet::new();
    let mut path_sorts = BTreeSet::new();
    for path in &catalog.audience_paths {
        let Some(die) = dice.get(&path.audience_die) else {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &path.key);
        };
        if die.audience_path != path.id
            || die.shared_path != path.shared_path
            || !path_sorts.insert(path.sort)
            || path.initial_program.is_empty()
            || path.passive_program.is_empty()
        {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &path.key);
        }
    }
    for die in &catalog.audience_dice {
        let Some(path) = paths.get(&die.audience_path) else {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &die.key);
        };
        if path.audience_die != die.id
            || die.shared_path != path.shared_path
            || die.face_keys.is_empty()
            || !unique_values(&die.face_keys)
            || die
                .face_keys
                .iter()
                .any(|key| !face_keys.contains(key.as_ref()) || !listed_faces.insert(key.as_ref()))
        {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &die.key);
        }
    }
    if listed_faces.len() != catalog.dice_faces.len() {
        return fail(SwarmDisasterUniqueErrorKind::Reference, "dice-face-closure");
    }
    for rarity in &catalog.dice_rarities {
        if rarity.rank == 0 {
            return fail(SwarmDisasterUniqueErrorKind::Metadata, &rarity.key);
        }
    }
    for face in &catalog.dice_faces {
        if !dice.contains_key(&face.audience_die)
            || !rarities.contains(&face.rarity)
            || !targets.contains(&face.target)
            || face.sort == 0
            || face.effect_program.is_empty()
            || !catalog
                .audience_dice
                .iter()
                .find(|die| die.id == face.audience_die)
                .is_some_and(|die| die.face_keys.iter().any(|key| key == &face.key))
        {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &face.key);
        }
    }
    let mut target_sources = BTreeSet::new();
    for target in &catalog.dice_targets {
        if target.source_id.is_empty()
            || target.candidate_filter.is_empty()
            || !target_sources.insert(target.source_id.as_ref())
        {
            return fail(SwarmDisasterUniqueErrorKind::Duplicate, &target.key);
        }
    }
    let mut operations = BTreeSet::new();
    for control in &catalog.dice_controls {
        if control.operation.is_empty()
            || control.resource_cost.is_empty()
            || !operations.insert(control.operation.as_ref())
        {
            return fail(SwarmDisasterUniqueErrorKind::Duplicate, &control.key);
        }
    }
    Ok(())
}

fn validate_communing(
    catalog: &SwarmDisasterUniqueCatalog,
) -> Result<(), SwarmDisasterUniqueError> {
    let dimensions = catalog
        .communing_dimensions
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    let shared_paths = catalog
        .audience_paths
        .iter()
        .map(|row| row.shared_path.as_ref())
        .collect::<BTreeSet<_>>();
    for dimension in &catalog.communing_dimensions {
        if !shared_paths.contains(dimension.shared_path.as_ref()) || dimension.maximum == 0 {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &dimension.key);
        }
    }
    for choice in &catalog.communing_choices {
        if !shared_paths.contains(choice.shared_path.as_ref())
            || choice.story_stage == 0
            || choice.operations.is_empty()
        {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &choice.key);
        }
    }
    let mut adjustment_order = BTreeSet::new();
    for adjustment in &catalog.point_adjustments {
        if !dimensions.contains_key(&adjustment.dimension)
            || adjustment.source_id.is_empty()
            || adjustment.source_kind.is_empty()
            || adjustment.delta.is_empty()
            || !adjustment_order.insert((
                adjustment.source_kind.as_ref(),
                adjustment.source_id.as_ref(),
                adjustment.ordinal,
            ))
        {
            return fail(SwarmDisasterUniqueErrorKind::Ordering, &adjustment.key);
        }
    }
    let nodes = catalog
        .trail_nodes
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    let node_keys = catalog
        .trail_nodes
        .iter()
        .map(|row| row.key.as_ref())
        .collect::<BTreeSet<_>>();
    let prerequisite_keys = catalog
        .trail_prerequisites
        .iter()
        .map(|row| row.key.as_ref())
        .collect::<BTreeSet<_>>();
    let effect_keys = catalog
        .trail_effects
        .iter()
        .map(|row| row.key.as_ref())
        .collect::<BTreeSet<_>>();
    let mut listed_prerequisites = BTreeSet::new();
    let mut listed_effects = BTreeSet::new();
    for node in &catalog.trail_nodes {
        if !dimensions.contains_key(&node.dimension)
            || node.effect_keys.is_empty()
            || node.threshold.is_empty()
            || node.effect_keys.iter().any(|key| {
                !effect_keys.contains(key.as_ref()) || !listed_effects.insert(key.as_ref())
            })
            || node.prerequisite_keys.iter().any(|key| {
                !prerequisite_keys.contains(key.as_ref())
                    || !listed_prerequisites.insert(key.as_ref())
            })
        {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &node.key);
        }
    }
    if listed_effects.len() != catalog.trail_effects.len()
        || listed_prerequisites.len() != catalog.trail_prerequisites.len()
    {
        return fail(SwarmDisasterUniqueErrorKind::Reference, "trail-closure");
    }
    validate_trail_prerequisites(catalog, &nodes, &node_keys)?;
    for effect in &catalog.trail_effects {
        if !nodes.contains_key(&effect.node)
            || effect.operations.is_empty()
            || !catalog
                .trail_nodes
                .iter()
                .find(|node| node.id == effect.node)
                .is_some_and(|node| node.effect_keys.iter().any(|key| key == &effect.key))
        {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &effect.key);
        }
    }
    for chapter in &catalog.chapters {
        if chapter.dimension.is_none() != chapter.threshold.is_none()
            || chapter
                .dimension
                .is_some_and(|dimension| !dimensions.contains_key(&dimension))
            || !(1..=3).contains(&chapter.layer)
            || chapter
                .threshold
                .as_ref()
                .is_some_and(|threshold| threshold.is_empty())
        {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &chapter.key);
        }
    }
    Ok(())
}

fn validate_trail_prerequisites(
    catalog: &SwarmDisasterUniqueCatalog,
    nodes: &BTreeMap<TrailNodeId, &super::types::TrailNodeDefinition>,
    node_keys: &BTreeSet<&str>,
) -> Result<(), SwarmDisasterUniqueError> {
    let mut order = BTreeSet::new();
    for prerequisite in &catalog.trail_prerequisites {
        if !nodes.contains_key(&prerequisite.node)
            || !nodes.contains_key(&prerequisite.required_node)
            || prerequisite.node == prerequisite.required_node
            || prerequisite.required_points.is_empty()
            || !order.insert((prerequisite.node, prerequisite.ordinal))
            || !node_keys.contains(
                nodes
                    .get(&prerequisite.required_node)
                    .map_or("", |node| node.key.as_ref()),
            )
        {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &prerequisite.key);
        }
    }
    Ok(())
}

fn validate_pathstrider(
    catalog: &SwarmDisasterUniqueCatalog,
) -> Result<(), SwarmDisasterUniqueError> {
    let cabinets = catalog
        .cabinets
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    let cabinet_keys = catalog
        .cabinets
        .iter()
        .map(|row| row.key.as_ref())
        .collect::<BTreeSet<_>>();
    let finish_keys = catalog
        .finish_conditions
        .iter()
        .map(|row| row.key.as_ref())
        .collect::<BTreeSet<_>>();
    let unlock_keys = catalog
        .unlocks
        .iter()
        .map(|row| row.key.as_ref())
        .collect::<BTreeSet<_>>();
    let finishes = catalog
        .finish_conditions
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    for cabinet in &catalog.cabinets {
        if cabinet.objective_id.is_empty()
            || cabinet.point_deltas.is_empty()
            || !unique_values(&cabinet.prerequisite_keys)
            || !unique_values(&cabinet.unlock_keys)
            || cabinet
                .prerequisite_keys
                .iter()
                .chain(cabinet.unlock_keys.iter())
                .any(|key| !cabinet_keys.contains(key.as_ref()))
        {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &cabinet.key);
        }
    }
    for objective in &catalog.objectives {
        if !cabinets.contains_key(&objective.cabinet)
            || objective.finish_key.is_empty()
            || objective.progress_policy.is_empty()
        {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &objective.key);
        }
    }
    for finish in &catalog.finish_conditions {
        if finish.target.is_empty()
            || finish.unlock_keys.is_empty()
            || finish
                .unlock_keys
                .iter()
                .any(|key| !unlock_keys.contains(key.as_ref()))
            || (finish.enabled && finish.key.contains("external-quest-condition"))
        {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &finish.key);
        }
    }
    for unlock in &catalog.unlocks {
        if !finishes.contains_key(&unlock.finish)
            || unlock.consequence.is_empty()
            || !finish_keys.contains(
                finishes
                    .get(&unlock.finish)
                    .map_or("", |finish| finish.key.as_ref()),
            )
        {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &unlock.key);
        }
    }
    Ok(())
}

fn validate_paths(catalog: &SwarmDisasterUniqueCatalog) -> Result<(), SwarmDisasterUniqueError> {
    let paths = catalog
        .paths
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    let dice = catalog
        .audience_dice
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    let resonances = catalog
        .resonances
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    let mut sorts = BTreeSet::new();
    for path in &catalog.paths {
        let Some(die) = dice.get(&path.audience_die) else {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &path.key);
        };
        let Some(resonance) = resonances.get(&path.resonance) else {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &path.key);
        };
        if die.shared_path != path.shared_path
            || resonance.path != path.id
            || !sorts.insert(path.sort)
        {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &path.key);
        }
    }
    for boost in &catalog.path_boosts {
        if !paths.contains_key(&boost.path) || boost.effect_program.is_empty() {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &boost.key);
        }
    }
    for resonance in &catalog.resonances {
        if !paths.contains_key(&resonance.path)
            || resonance.shared_resonance.is_empty()
            || resonance.effect_program.is_empty()
        {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &resonance.key);
        }
    }
    for interplay in &catalog.interplays {
        if !paths.contains_key(&interplay.main_path)
            || !paths.contains_key(&interplay.sub_path)
            || interplay.main_path == interplay.sub_path
            || interplay.thresholds.is_empty()
            || interplay.effect_program.is_empty()
        {
            return fail(SwarmDisasterUniqueErrorKind::Reference, &interplay.key);
        }
    }
    for bonus in &catalog.bonuses {
        if bonus.effect_program.is_empty() {
            return fail(SwarmDisasterUniqueErrorKind::Metadata, &bonus.key);
        }
    }
    Ok(())
}

fn sequential(
    values: impl Iterator<Item = u32>,
    key: &str,
) -> Result<(), SwarmDisasterUniqueError> {
    if values.enumerate().any(|(index, value)| {
        u32::try_from(index)
            .ok()
            .and_then(|index| index.checked_add(1))
            != Some(value)
    }) {
        return fail(SwarmDisasterUniqueErrorKind::Identifier, key);
    }
    Ok(())
}

fn unique<'a>(
    mut values: impl Iterator<Item = &'a str>,
    key: &str,
) -> Result<(), SwarmDisasterUniqueError> {
    let mut found = BTreeSet::new();
    if values.any(|value| !found.insert(value)) {
        return fail(SwarmDisasterUniqueErrorKind::Duplicate, key);
    }
    Ok(())
}

fn unique_values<T: Ord>(values: &[T]) -> bool {
    values.iter().collect::<BTreeSet<_>>().len() == values.len()
}

fn fail<T>(kind: SwarmDisasterUniqueErrorKind, key: &str) -> Result<T, SwarmDisasterUniqueError> {
    Err(SwarmDisasterUniqueError {
        kind,
        key: key.into(),
    })
}
