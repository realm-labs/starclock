use std::collections::{BTreeMap, BTreeSet};

use super::{
    EXPECTED_UNIQUE_ROWS, GoldAndGearsUniqueCatalog, GoldAndGearsUniqueError,
    GoldAndGearsUniqueErrorKind,
    support::fail,
    types::{CanonicalScalar, Identity},
};

pub(super) fn validate(catalog: &GoldAndGearsUniqueCatalog) -> Result<(), GoldAndGearsUniqueError> {
    let counts = [
        (catalog.cognition_ranges.len(), 13, "cognition-ranges"),
        (catalog.secrets.len(), 20, "secrets"),
        (catalog.constants.len(), 22, "mode-constants"),
        (catalog.dice.len(), 12, "custom-dice"),
        (catalog.dice_categories.len(), 4, "dice-categories"),
        (catalog.dice_path_values.len(), 108, "dice-path-values"),
        (catalog.dice_slots.len(), 6, "dice-slots"),
        (catalog.dice_faces.len(), 80, "dice-faces"),
        (catalog.dice_face_tags.len(), 10, "dice-face-tags"),
        (catalog.knowledge_rules.len(), 22, "knowledge-rules"),
        (catalog.neural_nodes.len(), 40, "neural-nodes"),
        (catalog.conundrum_levels.len(), 12, "conundrum-levels"),
        (catalog.trailblaze_bonuses.len(), 5, "trailblaze-bonuses"),
        (catalog.paths.len(), 9, "paths"),
        (catalog.path_boosts.len(), 9, "path-boosts"),
        (catalog.resonances.len(), 36, "resonances"),
        (catalog.extrapolations.len(), 36, "extrapolations"),
        (catalog.interplays.len(), 18, "interplays"),
    ];
    if counts
        .iter()
        .any(|(actual, expected, _)| actual != expected)
        || catalog.row_count() != EXPECTED_UNIQUE_ROWS
    {
        return fail(
            GoldAndGearsUniqueErrorKind::Denominator,
            counts
                .iter()
                .find(|(actual, expected, _)| actual != expected)
                .map_or("unique-total", |(_, _, key)| *key),
        );
    }
    identities(
        &catalog.cognition_ranges,
        |row| row.identity.id.0,
        |row| &row.identity,
        "cognition-ranges",
    )?;
    identities(
        &catalog.secrets,
        |row| row.identity.id.0,
        |row| &row.identity,
        "secrets",
    )?;
    identities(
        &catalog.constants,
        |row| row.identity.id.0,
        |row| &row.identity,
        "mode-constants",
    )?;
    identities(
        &catalog.dice,
        |row| row.identity.id.0,
        |row| &row.identity,
        "custom-dice",
    )?;
    identities(
        &catalog.dice_categories,
        |row| row.identity.id.0,
        |row| &row.identity,
        "dice-categories",
    )?;
    identities(
        &catalog.dice_path_values,
        |row| row.identity.id.0,
        |row| &row.identity,
        "dice-path-values",
    )?;
    identities(
        &catalog.dice_slots,
        |row| row.identity.id.0,
        |row| &row.identity,
        "dice-slots",
    )?;
    identities(
        &catalog.dice_faces,
        |row| row.identity.id.0,
        |row| &row.identity,
        "dice-faces",
    )?;
    identities(
        &catalog.dice_face_tags,
        |row| row.identity.id.0,
        |row| &row.identity,
        "dice-face-tags",
    )?;
    identities(
        &catalog.knowledge_rules,
        |row| row.identity.id.0,
        |row| &row.identity,
        "knowledge-rules",
    )?;
    identities(
        &catalog.neural_nodes,
        |row| row.identity.id.0,
        |row| &row.identity,
        "neural-nodes",
    )?;
    identities(
        &catalog.conundrum_levels,
        |row| row.identity.id.0,
        |row| &row.identity,
        "conundrum-levels",
    )?;
    identities(
        &catalog.trailblaze_bonuses,
        |row| row.identity.id.0,
        |row| &row.identity,
        "trailblaze-bonuses",
    )?;
    identities(
        &catalog.paths,
        |row| row.identity.id.0,
        |row| &row.identity,
        "paths",
    )?;
    identities(
        &catalog.path_boosts,
        |row| row.identity.id.0,
        |row| &row.identity,
        "path-boosts",
    )?;
    identities(
        &catalog.resonances,
        |row| row.identity.id.0,
        |row| &row.identity,
        "resonances",
    )?;
    identities(
        &catalog.extrapolations,
        |row| row.identity.id.0,
        |row| &row.identity,
        "extrapolations",
    )?;
    identities(
        &catalog.interplays,
        |row| row.identity.id.0,
        |row| &row.identity,
        "interplays",
    )?;
    validate_cognition(catalog)?;
    validate_dice(catalog)?;
    validate_progression(catalog)
}

fn validate_cognition(catalog: &GoldAndGearsUniqueCatalog) -> Result<(), GoldAndGearsUniqueError> {
    for range in &catalog.cognition_ranges {
        let values = [
            integer(&range.global_minimum)?,
            integer(&range.minimum)?,
            integer(&range.maximum)?,
            integer(&range.global_maximum)?,
        ];
        if range.area_key.is_empty()
            || !range.inclusive
            || range.lifecycle_json.is_empty()
            || !values.windows(2).all(|pair| pair[0] <= pair[1])
        {
            return fail(
                GoldAndGearsUniqueErrorKind::Metadata,
                &range.identity.stable_key,
            );
        }
    }
    let secret_sources = catalog
        .secrets
        .iter()
        .map(|row| row.identity.stable_key.as_ref())
        .collect::<BTreeSet<_>>();
    for secret in &catalog.secrets {
        if secret.area_key.is_empty()
            || secret.area_source.is_empty()
            || secret.plane_layer == 0
            || !secret.inclusive
            || secret.evaluation_boundary.is_empty()
            || secret.condition_hash.is_empty()
            || secret.condition_digest.is_empty()
            || secret.lifecycle_policy.is_empty()
            || integer(&secret.cognition_minimum)? > integer(&secret.cognition_maximum)?
            || secret.origin_minimum.is_empty()
            || secret.origin_maximum.is_empty()
            || secret
                .predecessors
                .iter()
                .chain(secret.next.iter())
                .any(|source| !secret_sources.contains(source.as_ref()))
            || !unique(&secret.predecessors)
            || !unique(&secret.next)
            || (secret.terminal && !secret.next.is_empty())
        {
            return fail(
                GoldAndGearsUniqueErrorKind::Reference,
                &secret.identity.stable_key,
            );
        }
    }
    for constant in &catalog.constants {
        if constant.mechanical_role.is_empty()
            || !matches!(
                constant.value_kind.as_ref(),
                "Decimal" | "Integer" | "IntegerList" | "IntegerMap"
            )
            || constant.values.is_empty()
            || !unique(&constant.values)
        {
            return fail(
                GoldAndGearsUniqueErrorKind::Metadata,
                &constant.identity.stable_key,
            );
        }
    }
    Ok(())
}

fn validate_dice(catalog: &GoldAndGearsUniqueCatalog) -> Result<(), GoldAndGearsUniqueError> {
    let categories = catalog
        .dice_categories
        .iter()
        .map(|row| (row.identity.id, row))
        .collect::<BTreeMap<_, _>>();
    let dice_by_id = catalog
        .dice
        .iter()
        .map(|row| (row.identity.id, row))
        .collect::<BTreeMap<_, _>>();
    let dice_by_key = catalog
        .dice
        .iter()
        .map(|row| (row.identity.stable_key.as_ref(), row))
        .collect::<BTreeMap<_, _>>();
    let dice_by_source = catalog
        .dice
        .iter()
        .map(|row| (row.identity.source_id.as_ref(), row))
        .collect::<BTreeMap<_, _>>();
    let slots_by_key = catalog
        .dice_slots
        .iter()
        .map(|row| (row.identity.stable_key.as_ref(), row))
        .collect::<BTreeMap<_, _>>();
    let slots_by_source = catalog
        .dice_slots
        .iter()
        .map(|row| (row.identity.source_id.as_ref(), row))
        .collect::<BTreeMap<_, _>>();
    let faces_by_source = catalog
        .dice_faces
        .iter()
        .map(|row| (row.identity.source_id.as_ref(), row))
        .collect::<BTreeMap<_, _>>();
    let face_tags = catalog
        .dice_face_tags
        .iter()
        .map(|row| row.identity.stable_key.as_ref())
        .collect::<BTreeSet<_>>();
    for category in &catalog.dice_categories {
        if category.sort == 0 {
            return fail(
                GoldAndGearsUniqueErrorKind::Ordering,
                &category.identity.stable_key,
            );
        }
    }
    for die in &catalog.dice {
        let Some(category) = categories.get(&die.category) else {
            return fail(
                GoldAndGearsUniqueErrorKind::Reference,
                &die.identity.stable_key,
            );
        };
        if category.identity.source_id != die.category_source
            || die.sort == 0
            || (die.initial_effects.is_empty() && die.passive_effects.is_empty())
            || die.available_by_default == die.unlock_id.is_some()
            || die.common_face_sources.is_empty()
            || die.default_face_sources.is_empty()
            || !die.default_face_sources.contains(&die.ultra_face_source)
            || die
                .common_face_sources
                .iter()
                .chain(die.default_face_sources.iter())
                .chain(die.suggestive_face_sources.iter())
                .chain(die.recommended_face_sources.iter())
                .any(|source| !faces_by_source.contains_key(source.as_ref()))
        {
            return fail(
                GoldAndGearsUniqueErrorKind::Reference,
                &die.identity.stable_key,
            );
        }
    }
    for slot in &catalog.dice_slots {
        if usize::from(slot.index) > catalog.dice_slots.len()
            || slot.base_max_rarity > slot.upgraded_max_rarity
            || slot
                .extra_max_rarity
                .is_some_and(|rarity| rarity > slot.upgraded_max_rarity)
        {
            return fail(
                GoldAndGearsUniqueErrorKind::Metadata,
                &slot.identity.stable_key,
            );
        }
    }
    for face in &catalog.dice_faces {
        if face.item_id.is_empty()
            || face.rarity == 0
            || face.allowed_slot_keys.len() != face.allowed_slot_sources.len()
            || face.allowed_dice_keys.len() != face.allowed_dice_sources.len()
            || face.no_target_behavior.is_empty()
            || face.target_policy_json.is_empty()
            || face
                .allowed_slot_keys
                .iter()
                .zip(face.allowed_slot_sources.iter())
                .any(|(key, source)| {
                    slots_by_key.get(key.as_ref()).map(|row| row.identity.id)
                        != slots_by_source
                            .get(source.as_ref())
                            .map(|row| row.identity.id)
                })
            || face
                .allowed_dice_keys
                .iter()
                .zip(face.allowed_dice_sources.iter())
                .any(|(key, source)| {
                    dice_by_key.get(key.as_ref()).map(|row| row.identity.id)
                        != dice_by_source
                            .get(source.as_ref())
                            .map(|row| row.identity.id)
                })
            || face
                .filter_tag_sources
                .iter()
                .any(|key| !face_tags.contains(key.as_ref()))
            || !unique(&face.allowed_slot_keys)
            || !unique(&face.allowed_dice_keys)
            || !unique(&face.mechanical_codes)
            || !unique(&face.effect_ids)
            || !unique(&face.filter_tag_sources)
        {
            return fail(
                GoldAndGearsUniqueErrorKind::Reference,
                &face.identity.stable_key,
            );
        }
        let _ = (
            face.sort,
            face.activation_stage,
            face.parameters.len(),
            face.universal_dice_eligibility,
        );
    }
    for tag in &catalog.dice_face_tags {
        if tag.sort == 0 || tag.mechanical_code.is_empty() || tag.replacement_condition.is_empty() {
            return fail(
                GoldAndGearsUniqueErrorKind::Metadata,
                &tag.identity.stable_key,
            );
        }
    }
    let faces = catalog
        .dice_faces
        .iter()
        .map(|row| (row.identity.id, row))
        .collect::<BTreeMap<_, _>>();
    for rule in &catalog.knowledge_rules {
        let Some(face) = faces.get(&rule.dice_face) else {
            return fail(
                GoldAndGearsUniqueErrorKind::Reference,
                &rule.identity.stable_key,
            );
        };
        if face.identity.source_id != rule.identity.source_id
            || rule.operation.is_empty()
            || rule.trigger_boundary.is_empty()
            || rule.target_scope.is_empty()
            || rule.selection_mode.is_empty()
            || rule.knowledge_access.is_empty()
            || rule.target_policy_json.is_empty()
            || rule.simultaneous_policy_json.is_empty()
            || rule.dice_interactions_json.is_empty()
        {
            return fail(
                GoldAndGearsUniqueErrorKind::Metadata,
                &rule.identity.stable_key,
            );
        }
        let _ = (rule.parameters.len(), rule.activation_stage);
    }
    let paths = catalog
        .paths
        .iter()
        .map(|row| {
            (
                (
                    row.identity.stable_key.as_ref(),
                    row.identity.source_id.as_ref(),
                ),
                row.identity.id,
            )
        })
        .collect::<BTreeMap<_, _>>();
    for value in &catalog.dice_path_values {
        let Some(die) = dice_by_id.get(&value.dice) else {
            return fail(
                GoldAndGearsUniqueErrorKind::Reference,
                &value.identity.stable_key,
            );
        };
        if die.identity.source_id != value.dice_source
            || !paths.contains_key(&(value.path_key.as_ref(), value.path_source.as_ref()))
            || value.boost_stat.is_empty()
            || value.trigger_interval.is_empty()
            || value.boost_value.0.is_empty()
            || value.boost_unit.as_ref() != "SourceRatioFormattedAsPercent"
        {
            return fail(
                GoldAndGearsUniqueErrorKind::Reference,
                &value.identity.stable_key,
            );
        }
        let _ = value.parameters.len();
    }
    Ok(())
}

fn validate_progression(
    catalog: &GoldAndGearsUniqueCatalog,
) -> Result<(), GoldAndGearsUniqueError> {
    validate_neural(catalog)?;
    validate_conundrum(catalog)?;
    let paths = catalog
        .paths
        .iter()
        .map(|row| (row.identity.id, row))
        .collect::<BTreeMap<_, _>>();
    let boosts = catalog
        .path_boosts
        .iter()
        .map(|row| (row.identity.id, row))
        .collect::<BTreeMap<_, _>>();
    let dice_values = catalog
        .dice_path_values
        .iter()
        .map(|row| row.identity.stable_key.as_ref())
        .collect::<BTreeSet<_>>();
    for path in &catalog.paths {
        let Some(boost) = boosts.get(&path.path_boost) else {
            return fail(
                GoldAndGearsUniqueErrorKind::Reference,
                &path.identity.stable_key,
            );
        };
        if boost.path != path.identity.id
            || path.sort == 0
            || path.buff_type == 0
            || path.shared_resonance_id == 0
            || path.shared_formation_ids.is_empty()
            || path.normal_event_group.is_empty()
            || path.enhanced_event_group.is_empty()
        {
            return fail(
                GoldAndGearsUniqueErrorKind::Reference,
                &path.identity.stable_key,
            );
        }
    }
    for boost in &catalog.path_boosts {
        if !paths.contains_key(&boost.path)
            || boost.aeon_source.is_empty()
            || boost.effect_type.is_empty()
            || boost.ability_name.is_empty()
            || boost.target_team.is_empty()
            || boost.target_property.is_empty()
            || boost.boost_stat.is_empty()
            || boost.stacking.is_empty()
            || boost.value_conversion.is_empty()
            || boost.dice_path_value_keys.is_empty()
            || boost
                .dice_path_value_keys
                .iter()
                .any(|key| !dice_values.contains(key.as_ref()))
            || boost.allowed_increments.is_empty()
            || boost.rule_contribution.is_empty()
        {
            return fail(
                GoldAndGearsUniqueErrorKind::Reference,
                &boost.identity.stable_key,
            );
        }
    }
    for resonance in &catalog.resonances {
        if !paths.contains_key(&resonance.path)
            || resonance.resonance_kind.is_empty()
            || (resonance.resonance_kind.as_ref() != "Formation" && resonance.threshold == 0)
            || resonance.energy_max.0.is_empty()
            || resonance.initial_energy.0.is_empty()
            || resonance.parameter_values_json.is_empty()
            || resonance.source_modifier.is_empty()
            || resonance.source_binding_type.is_empty()
            || resonance.source_binding_key.is_empty()
        {
            return fail(
                GoldAndGearsUniqueErrorKind::Reference,
                &resonance.identity.stable_key,
            );
        }
        let _ = (
            resonance.mechanic_tags.len(),
            resonance.inherited_rule_ids.len(),
        );
    }
    for extrapolation in &catalog.extrapolations {
        if !paths.contains_key(&extrapolation.path)
            || extrapolation.aeon_source.is_empty()
            || extrapolation.buff_group.is_empty()
            || extrapolation.shared_resonance_id == 0
            || extrapolation.shared_resonance_kind.is_empty()
            || extrapolation.battle_event_type.is_empty()
            || extrapolation.source_modifier.is_empty()
            || extrapolation.source_binding_type.is_empty()
            || extrapolation.source_binding_key.is_empty()
            || extrapolation.source_parameters_json.is_empty()
            || extrapolation.battle_scope.is_empty()
            || extrapolation.controller_policy_json.is_empty()
            || extrapolation.rule_contribution.is_empty()
        {
            return fail(
                GoldAndGearsUniqueErrorKind::Reference,
                &extrapolation.identity.stable_key,
            );
        }
        let _ = extrapolation.enhanced;
    }
    for interplay in &catalog.interplays {
        if !paths.contains_key(&interplay.main_path)
            || !paths.contains_key(&interplay.sub_path)
            || interplay.main_path == interplay.sub_path
            || interplay.main_threshold == 0
            || interplay.sub_threshold == 0
            || interplay.buff_group.is_empty()
            || interplay.shared_maze_buff.is_empty()
            || interplay.source_modifier.is_empty()
            || interplay.source_binding_type.is_empty()
            || interplay.source_binding_key.is_empty()
            || interplay.source_parameters_json.is_empty()
            || interplay.rule_contribution.is_empty()
        {
            return fail(
                GoldAndGearsUniqueErrorKind::Reference,
                &interplay.identity.stable_key,
            );
        }
    }
    for bonus in &catalog.trailblaze_bonuses {
        if bonus.bonus_event.is_empty()
            || bonus.effect_contributions_json.is_empty()
            || bonus.rule_contribution.is_empty()
        {
            return fail(
                GoldAndGearsUniqueErrorKind::Metadata,
                &bonus.identity.stable_key,
            );
        }
    }
    Ok(())
}

fn validate_neural(catalog: &GoldAndGearsUniqueCatalog) -> Result<(), GoldAndGearsUniqueError> {
    let by_source = catalog
        .neural_nodes
        .iter()
        .map(|row| (row.identity.stable_key.as_ref(), row))
        .collect::<BTreeMap<_, _>>();
    for node in &catalog.neural_nodes {
        if node
            .prerequisites
            .iter()
            .chain(node.next.iter())
            .any(|source| !by_source.contains_key(source.as_ref()))
            || node.prerequisites.iter().any(|source| {
                by_source[source.as_ref()].topological_index >= node.topological_index
            })
            || node.next.iter().any(|source| {
                by_source[source.as_ref()].topological_index <= node.topological_index
            })
            || node.costs_json.is_empty()
            || node.disposition.is_empty()
            || node.effect_domain.is_empty()
            || node.source_parameters_json.is_empty()
            || node.effect_contributions_json.is_empty()
            || node.rule_contribution.is_empty()
            || !unique(&node.prerequisites)
            || !unique(&node.next)
        {
            return fail(
                GoldAndGearsUniqueErrorKind::Ordering,
                &node.identity.stable_key,
            );
        }
        let _ = (node.external_unlocks.len(), node.important);
    }
    Ok(())
}

fn validate_conundrum(catalog: &GoldAndGearsUniqueCatalog) -> Result<(), GoldAndGearsUniqueError> {
    for track in ["Stats", "Auxiliary"] {
        let mut levels = catalog
            .conundrum_levels
            .iter()
            .filter(|row| row.track.as_ref() == track)
            .collect::<Vec<_>>();
        levels.sort_unstable_by_key(|row| row.level);
        if levels.len() != 6
            || levels
                .iter()
                .enumerate()
                .any(|(index, row)| usize::from(row.level) != index + 1)
        {
            return fail(GoldAndGearsUniqueErrorKind::Denominator, track);
        }
    }
    let sources = catalog
        .conundrum_levels
        .iter()
        .map(|row| row.identity.stable_key.as_ref())
        .collect::<BTreeSet<_>>();
    for level in &catalog.conundrum_levels {
        if level.source_type.is_empty()
            || level.track_cap != 6
            || level.total_cap != 12
            || level.total_formula.is_empty()
            || level.unlock_requirement_json.is_empty()
            || level.composition_mode.is_empty()
            || level.active_contributions.is_empty()
            || level
                .replaces_levels
                .iter()
                .any(|source| !sources.contains(source.as_ref()))
            || level.source_parameters_json.is_empty()
            || level.effect_contributions_json.is_empty()
            || level.rule_contribution.is_empty()
        {
            return fail(
                GoldAndGearsUniqueErrorKind::Reference,
                &level.identity.stable_key,
            );
        }
        let _ = (level.source_tag, level.source_sort);
    }
    Ok(())
}

fn identities<T, I>(
    rows: &[T],
    id: impl Fn(&T) -> u32,
    identity: impl Fn(&T) -> &Identity<I>,
    key: &str,
) -> Result<(), GoldAndGearsUniqueError> {
    let mut stable = BTreeSet::new();
    let mut source = BTreeSet::new();
    for (index, row) in rows.iter().enumerate() {
        let expected = u32::try_from(index)
            .ok()
            .and_then(|index| index.checked_add(1));
        let identity = identity(row);
        if Some(id(row)) != expected
            || !stable.insert(identity.stable_key.as_ref())
            || !source.insert(identity.source_id.as_ref())
        {
            return fail(GoldAndGearsUniqueErrorKind::Duplicate, key);
        }
    }
    Ok(())
}

fn integer(value: &CanonicalScalar) -> Result<i64, GoldAndGearsUniqueError> {
    value
        .0
        .parse::<i64>()
        .map_err(|_| super::support::error(GoldAndGearsUniqueErrorKind::Metadata, &value.0))
}

fn unique<T: Ord>(values: &[T]) -> bool {
    values.iter().collect::<BTreeSet<_>>().len() == values.len()
}
