use std::collections::{BTreeMap, BTreeSet};

use crate::{
    swarm_disaster_structural::SwarmDisasterStructuralCatalog,
    swarm_disaster_unique::SwarmDisasterUniqueCatalog,
};

use super::{
    EXPECTED_CONTENT_ROWS, SwarmDisasterContentCatalog, SwarmDisasterContentError,
    SwarmDisasterContentErrorKind,
};

pub(super) fn catalog(
    catalog: &SwarmDisasterContentCatalog,
    structural: &SwarmDisasterStructuralCatalog,
    unique: &SwarmDisasterUniqueCatalog,
) -> Result<(), SwarmDisasterContentError> {
    validate_denominators(catalog)?;
    validate_keys(catalog)?;
    validate_topology(catalog, structural, unique)?;
    validate_inventory(catalog, structural, unique)?;
    validate_encounters(catalog, structural)?;
    validate_rules(catalog)
}

fn validate_denominators(
    catalog: &SwarmDisasterContentCatalog,
) -> Result<(), SwarmDisasterContentError> {
    let counts = [
        (catalog.map_events.len(), 349, "map-events"),
        (catalog.block_rules.len(), 1_212, "block-rules"),
        (
            catalog.topology_consequences.len(),
            13,
            "topology-consequences",
        ),
        (catalog.blessings.len(), 144, "blessings"),
        (catalog.blessing_levels.len(), 288, "blessing-levels"),
        (catalog.pool_memberships.len(), 184, "pool-memberships"),
        (catalog.curios.len(), 66, "curios"),
        (catalog.curio_states.len(), 66, "curio-states"),
        (catalog.curio_rules.len(), 66, "curio-rules"),
        (catalog.occurrences.len(), 75, "occurrences"),
        (catalog.occurrence_variants.len(), 57, "occurrence-variants"),
        (catalog.occurrence_choices.len(), 308, "occurrence-choices"),
        (catalog.services.len(), 15, "services"),
        (catalog.adventure_outcomes.len(), 6, "adventure-outcomes"),
        (catalog.currencies.len(), 1, "currencies"),
        (catalog.service_rules.len(), 19, "service-rules"),
        (catalog.encounter_groups.len(), 179, "encounter-groups"),
        (catalog.encounter_waves.len(), 347, "encounter-waves"),
        (catalog.enemy_slots.len(), 1_070, "enemy-slots"),
        (catalog.boss_pools.len(), 15, "boss-pools"),
        (catalog.mechanic_rules.len(), 23, "mechanic-rules"),
        (catalog.audit.source_records, 8_139, "source-records"),
        (catalog.audit.coverage_rows, 6_963, "coverage"),
        (catalog.audit.research_gaps, 31, "research-gaps"),
        (catalog.audit.affected_rows, 5_560, "gap-affected"),
        (catalog.audit.fixtures, 23, "fixtures"),
        (catalog.audit.receipts, 609, "receipts"),
        (catalog.audit.manifest_rows, 1, "manifest"),
        (catalog.audit.pack_rows, 63, "pack-index"),
    ];
    if catalog.bundle.table_count() != 65
        || catalog.row_count() != EXPECTED_CONTENT_ROWS
        || counts
            .iter()
            .any(|(actual, expected, _)| actual != expected)
    {
        let key = counts
            .iter()
            .find(|(actual, expected, _)| actual != expected)
            .map_or("content-total", |(_, _, key)| *key);
        return fail(SwarmDisasterContentErrorKind::Denominator, key);
    }
    Ok(())
}

fn validate_keys(catalog: &SwarmDisasterContentCatalog) -> Result<(), SwarmDisasterContentError> {
    macro_rules! unique_table {
        ($field:ident) => {
            unique(
                catalog.$field.iter().map(|row| row.key.as_ref()),
                stringify!($field),
            )?;
        };
    }
    unique_table!(map_events);
    unique_table!(block_rules);
    unique_table!(topology_consequences);
    unique_table!(blessings);
    unique_table!(blessing_levels);
    unique_table!(pool_memberships);
    unique_table!(curios);
    unique_table!(curio_states);
    unique_table!(curio_rules);
    unique_table!(occurrences);
    unique_table!(occurrence_variants);
    unique_table!(occurrence_choices);
    unique_table!(services);
    unique_table!(adventure_outcomes);
    unique_table!(currencies);
    unique_table!(service_rules);
    unique_table!(encounter_groups);
    unique_table!(encounter_waves);
    unique_table!(enemy_slots);
    unique_table!(boss_pools);
    unique_table!(mechanic_rules);
    unique_table!(review_fixtures);
    Ok(())
}

fn validate_topology(
    catalog: &SwarmDisasterContentCatalog,
    structural: &SwarmDisasterStructuralCatalog,
    unique: &SwarmDisasterUniqueCatalog,
) -> Result<(), SwarmDisasterContentError> {
    for event in &catalog.map_events {
        if !structural.contains_chessboard_id(event.chessboard_id)
            || event.trigger.is_empty()
            || event.operations.is_empty()
        {
            return fail(SwarmDisasterContentErrorKind::Reference, &event.key);
        }
    }
    let mut block_order = BTreeSet::new();
    for rule in &catalog.block_rules {
        if !structural.contains_chessboard_id(rule.chessboard_id)
            || !structural.contains_domain_id(rule.domain_id)
            || !block_order.insert((rule.chessboard_id, rule.group.as_ref(), rule.order))
        {
            return fail(SwarmDisasterContentErrorKind::Reference, &rule.key);
        }
    }
    for consequence in &catalog.topology_consequences {
        if !unique.contains_audience_die_id(consequence.audience_die_id)
            || consequence.trigger_kind.is_empty()
            || consequence.scope.is_empty()
            || consequence.operations.is_empty()
        {
            return fail(SwarmDisasterContentErrorKind::Reference, &consequence.key);
        }
    }
    Ok(())
}

fn validate_inventory(
    catalog: &SwarmDisasterContentCatalog,
    structural: &SwarmDisasterStructuralCatalog,
    unique: &SwarmDisasterUniqueCatalog,
) -> Result<(), SwarmDisasterContentError> {
    validate_blessings(catalog, unique)?;
    validate_pool_memberships(catalog, unique)?;
    validate_curios(catalog)?;
    validate_occurrences(catalog)?;
    validate_services(catalog, structural)
}

fn validate_blessings(
    catalog: &SwarmDisasterContentCatalog,
    unique: &SwarmDisasterUniqueCatalog,
) -> Result<(), SwarmDisasterContentError> {
    let blessings = catalog
        .blessings
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    let levels = catalog
        .blessing_levels
        .iter()
        .map(|row| row.key.as_ref())
        .collect::<BTreeSet<_>>();
    let mut listed = BTreeSet::new();
    for blessing in &catalog.blessings {
        if !unique.contains_shared_path(&blessing.path_key)
            || blessing.level_keys.len() != 2
            || blessing.rarity > 3
            || blessing
                .level_keys
                .iter()
                .any(|key| !levels.contains(key.as_ref()) || !listed.insert(key.as_ref()))
        {
            return fail(SwarmDisasterContentErrorKind::Reference, &blessing.key);
        }
    }
    if listed.len() != catalog.blessing_levels.len() {
        return fail(
            SwarmDisasterContentErrorKind::Reference,
            "blessing-level-closure",
        );
    }
    let mut orders = BTreeSet::new();
    for level in &catalog.blessing_levels {
        let Some(blessing) = blessings.get(&level.blessing) else {
            return fail(SwarmDisasterContentErrorKind::Reference, &level.key);
        };
        if level.shared_blessing_key != blessing.shared_key
            || !blessing.level_keys.iter().any(|key| key == &level.key)
            || !orders.insert((level.blessing, level.level))
        {
            return fail(SwarmDisasterContentErrorKind::Reference, &level.key);
        }
    }
    Ok(())
}

fn validate_pool_memberships(
    catalog: &SwarmDisasterContentCatalog,
    unique: &SwarmDisasterUniqueCatalog,
) -> Result<(), SwarmDisasterContentError> {
    let blessings = catalog
        .blessings
        .iter()
        .map(|row| row.shared_key.as_ref())
        .collect::<BTreeSet<_>>();
    let mut members = BTreeSet::new();
    for row in &catalog.pool_memberships {
        let valid = match row.member_kind.as_ref() {
            "Blessing" => blessings.contains(row.member_key.as_ref()),
            "Path" => unique.contains_shared_path(&row.member_key),
            "Resonance" => unique.contains_shared_resonance(&row.member_key),
            "Formation" => row.member_key.starts_with("universe.resonance."),
            _ => false,
        };
        if !valid || !members.insert((row.pool_key.as_ref(), row.member_key.as_ref())) {
            return fail(SwarmDisasterContentErrorKind::Reference, &row.key);
        }
    }
    Ok(())
}

fn validate_curios(catalog: &SwarmDisasterContentCatalog) -> Result<(), SwarmDisasterContentError> {
    let curios = catalog
        .curios
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    let states = catalog
        .curio_states
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    for curio in &catalog.curios {
        if !states
            .get(&curio.initial_state)
            .is_some_and(|state| state.curio == curio.id)
        {
            return fail(SwarmDisasterContentErrorKind::Reference, &curio.key);
        }
    }
    for state in &catalog.curio_states {
        if !curios.contains_key(&state.curio) {
            return fail(SwarmDisasterContentErrorKind::Reference, &state.key);
        }
    }
    for rule in &catalog.curio_rules {
        if !curios.contains_key(&rule.curio)
            || !states
                .get(&rule.state)
                .is_some_and(|state| state.curio == rule.curio)
        {
            return fail(SwarmDisasterContentErrorKind::Reference, &rule.key);
        }
    }
    Ok(())
}

fn validate_occurrences(
    catalog: &SwarmDisasterContentCatalog,
) -> Result<(), SwarmDisasterContentError> {
    let occurrence_keys = catalog
        .occurrences
        .iter()
        .map(|row| row.key.as_ref())
        .collect::<BTreeSet<_>>();
    let variants = catalog
        .occurrence_variants
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    let variant_keys = catalog
        .occurrence_variants
        .iter()
        .map(|row| row.key.as_ref())
        .collect::<BTreeSet<_>>();
    let choice_keys = catalog
        .occurrence_choices
        .iter()
        .map(|row| row.key.as_ref())
        .collect::<BTreeSet<_>>();
    for occurrence in &catalog.occurrences {
        if occurrence.variant_keys.is_empty()
            || occurrence
                .variant_keys
                .iter()
                .any(|key| !variant_keys.contains(key.as_ref()))
        {
            return fail(SwarmDisasterContentErrorKind::Reference, &occurrence.key);
        }
    }
    for variant in &catalog.occurrence_variants {
        if variant.occurrence_keys.is_empty()
            || variant.choice_keys.is_empty()
            || variant
                .occurrence_keys
                .iter()
                .any(|key| !occurrence_keys.contains(key.as_ref()))
            || variant
                .choice_keys
                .iter()
                .any(|key| !choice_keys.contains(key.as_ref()))
        {
            return fail(SwarmDisasterContentErrorKind::Reference, &variant.key);
        }
    }
    let mut order = BTreeSet::new();
    for choice in &catalog.occurrence_choices {
        if !variants
            .get(&choice.variant)
            .is_some_and(|variant| variant.choice_keys.iter().any(|key| key == &choice.key))
            || !order.insert((choice.variant, choice.ordinal))
        {
            return fail(SwarmDisasterContentErrorKind::Ordering, &choice.key);
        }
    }
    Ok(())
}

fn validate_services(
    catalog: &SwarmDisasterContentCatalog,
    structural: &SwarmDisasterStructuralCatalog,
) -> Result<(), SwarmDisasterContentError> {
    let service_keys = catalog
        .services
        .iter()
        .map(|row| row.key.as_ref())
        .collect::<BTreeSet<_>>();
    for rule in &catalog.service_rules {
        if !service_keys.contains(rule.service_key.as_ref())
            && !structural.contains_beacon_key(&rule.service_key)
        {
            return fail(SwarmDisasterContentErrorKind::Reference, &rule.key);
        }
    }
    Ok(())
}

fn validate_encounters(
    catalog: &SwarmDisasterContentCatalog,
    structural: &SwarmDisasterStructuralCatalog,
) -> Result<(), SwarmDisasterContentError> {
    let groups = catalog
        .encounter_groups
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    let group_keys = catalog
        .encounter_groups
        .iter()
        .map(|row| row.key.as_ref())
        .collect::<BTreeSet<_>>();
    let waves = catalog
        .encounter_waves
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    let wave_keys = catalog
        .encounter_waves
        .iter()
        .map(|row| row.key.as_ref())
        .collect::<BTreeSet<_>>();
    let slot_keys = catalog
        .enemy_slots
        .iter()
        .map(|row| row.key.as_ref())
        .collect::<BTreeSet<_>>();
    for group in &catalog.encounter_groups {
        if group.wave_keys.is_empty()
            || group
                .room_key
                .as_ref()
                .is_some_and(|key| !structural.contains_room_key(key))
            || group
                .area_keys
                .iter()
                .any(|key| !structural.contains_area_key(key))
            || group
                .boss_choice_keys
                .iter()
                .any(|key| !structural.contains_boss_choice_key(key))
            || group
                .wave_keys
                .iter()
                .any(|key| !wave_keys.contains(key.as_ref()))
        {
            return fail(SwarmDisasterContentErrorKind::Reference, &group.key);
        }
    }
    for wave in &catalog.encounter_waves {
        if !groups.contains_key(&wave.group)
            || wave.slot_keys.is_empty()
            || wave
                .slot_keys
                .iter()
                .any(|key| !slot_keys.contains(key.as_ref()))
        {
            return fail(SwarmDisasterContentErrorKind::Reference, &wave.key);
        }
    }
    for slot in &catalog.enemy_slots {
        if !waves.get(&slot.wave).is_some_and(|wave| {
            wave.key == slot.wave_key && wave.slot_keys.iter().any(|key| key == &slot.key)
        }) || slot
            .boss_choice_keys
            .iter()
            .any(|key| !structural.contains_boss_choice_key(key))
        {
            return fail(SwarmDisasterContentErrorKind::Reference, &slot.key);
        }
    }
    for pool in &catalog.boss_pools {
        if !structural.contains_difficulty_key(&pool.difficulty_key)
            || !structural.contains_area_id(pool.area_id)
            || pool.candidate_keys.is_empty()
            || pool
                .candidate_keys
                .iter()
                .any(|key| !group_keys.contains(key.as_ref()))
        {
            return fail(SwarmDisasterContentErrorKind::Reference, &pool.key);
        }
    }
    Ok(())
}

fn validate_rules(catalog: &SwarmDisasterContentCatalog) -> Result<(), SwarmDisasterContentError> {
    let mut families = BTreeSet::new();
    let mut fixtures = BTreeSet::new();
    for rule in &catalog.mechanic_rules {
        if rule.triggers.is_empty()
            || rule.fixture_keys.is_empty()
            || rule.disposition.as_ref() != "ReferenceOnly"
            || !families.insert(rule.family_key.as_ref())
            || rule
                .fixture_keys
                .iter()
                .any(|key| !fixtures.insert(key.as_ref()))
        {
            return fail(SwarmDisasterContentErrorKind::Reference, &rule.key);
        }
    }
    if fixtures.len() != usize::from(catalog.audit.fixture_families)
        || catalog.audit.mechanic_rules != 23
    {
        return fail(
            SwarmDisasterContentErrorKind::Reference,
            "rule-fixture-closure",
        );
    }
    Ok(())
}

fn unique<'a>(
    mut values: impl Iterator<Item = &'a str>,
    key: &str,
) -> Result<(), SwarmDisasterContentError> {
    let mut found = BTreeSet::new();
    if values.any(|value| !found.insert(value)) {
        return fail(SwarmDisasterContentErrorKind::Duplicate, key);
    }
    Ok(())
}

fn fail<T>(kind: SwarmDisasterContentErrorKind, key: &str) -> Result<T, SwarmDisasterContentError> {
    Err(SwarmDisasterContentError {
        kind,
        key: key.into(),
    })
}
