use std::collections::{BTreeMap, BTreeSet};

use crate::{
    battle_materialization::catalog_composition::UniverseBattleCatalogComposition,
    catalog::UniverseCatalog,
    gold_gears_content::{
        EXPECTED_CONTENT_ROWS, GoldAndGearsContentCatalog, GoldAndGearsContentError,
        GoldAndGearsContentErrorKind,
        types::{JsonPayload, StableIndexRow, StableKey},
    },
    gold_gears_generated::SoraConfig,
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../../config/universe-generated/config.sora");
// Goal 08 binds these two released v4.4 identities, while their combat
// definitions are owned by Goal 14 P6 rather than the Standard mode catalog.
const GOLD_ENEMY_IDENTITIES_PENDING_P6: [&str; 23] = [
    "enemy.abundant-ebon-deer-complete.littleboss.02.variant.01",
    "enemy.argenti-complete.littleboss.variant.01",
    "enemy.aurumaton-spectral-envoy-bug.elite.variant.01",
    "enemy.automaton-direwolf-bug.elite.variant.01",
    "enemy.automaton-direwolf-complete.elite.03.variant.01",
    "enemy.automaton-grizzly-bug.elite.variant.01",
    "enemy.automaton-grizzly-complete.elite.03.variant.01",
    "enemy.blaze-out-of-space-bug.elite.variant.01",
    "enemy.cloud-knight-lieutenant-yanqing-complete.littleboss.02.variant.01",
    "enemy.cocolia-complete.littleboss.02.variant.01",
    "enemy.decaying-shadow-bug.elite.variant.01",
    "enemy.frigid-prowler-bug.elite.variant.01",
    "enemy.gepard-complete.littleboss.02.variant.01",
    "enemy.guardian-shadow-bug.elite.variant.01",
    "enemy.ice-out-of-space-bug.elite.variant.01",
    "enemy.searing-prowler-bug.elite.variant.01",
    "enemy.sequence-trotter.minionlv2.05.variant.01",
    "enemy.silvermane-lieutenant.elite.variant.01",
    "enemy.stellaron-hunter-kafka-complete.littleboss.02.variant.01",
    "enemy.swarm-true-sting-complete.littleboss.02.variant.01",
    "enemy.swarm-true-sting-complete.littleboss.variant.01",
    "enemy.svarog-complete.littleboss.02.variant.01",
    "enemy.the-ascended-bug.elite.variant.01",
];

pub(super) fn validate(
    catalog: &GoldAndGearsContentCatalog,
    source: &SoraConfig,
) -> Result<(), GoldAndGearsContentError> {
    denominators(catalog)?;
    unique_and_ordered(catalog)?;
    json_payloads(catalog)?;
    let standard = standard_catalog()?;
    content_references(catalog, source, &standard)?;
    encounter_references(catalog, source, &standard)?;
    mechanic_and_evidence_references(catalog, source)?;
    coverage(catalog, source)?;
    Ok(())
}

fn denominators(catalog: &GoldAndGearsContentCatalog) -> Result<(), GoldAndGearsContentError> {
    let counts = [
        catalog.blessings.len(),
        catalog.blessing_levels.len(),
        catalog.curios.len(),
        catalog.curio_states.len(),
        catalog.occurrences.len(),
        catalog.occurrence_variants.len(),
        catalog.occurrence_choices.len(),
        catalog.services.len(),
        catalog.adventure_outcomes.len(),
        catalog.encounter_groups.len(),
        catalog.encounter_waves.len(),
        catalog.enemy_slots.len(),
        catalog.map_events.len(),
        catalog.block_create_rules.len(),
        catalog.mechanic_rules.len(),
        catalog.source_records.len(),
        catalog.coverage.len(),
        catalog.research_gaps.len(),
        catalog.gap_affected_records.len(),
        catalog.review_fixtures.len(),
        catalog.pack_index.len(),
    ];
    let expected = [
        162, 324, 80, 80, 62, 65, 257, 15, 8, 181, 478, 1_513, 332, 1_091, 1_224, 9_082, 42, 16,
        5_025, 18, 1,
    ];
    require(
        counts == expected && catalog.row_count() == EXPECTED_CONTENT_ROWS,
        GoldAndGearsContentErrorKind::Denominator,
        "21 content table denominators",
    )
}

fn unique_and_ordered(
    catalog: &GoldAndGearsContentCatalog,
) -> Result<(), GoldAndGearsContentError> {
    check_rows(
        catalog.blessings.iter().map(|row| (row.id, &row.key)),
        "blessings",
    )?;
    check_rows(
        catalog.blessing_levels.iter().map(|row| (row.id, &row.key)),
        "blessing levels",
    )?;
    check_rows(
        catalog.curios.iter().map(|row| (row.id, &row.key)),
        "curios",
    )?;
    check_rows(
        catalog.curio_states.iter().map(|row| (row.id, &row.key)),
        "curio states",
    )?;
    check_rows(
        catalog.occurrences.iter().map(|row| (row.id, &row.key)),
        "occurrences",
    )?;
    check_rows(
        catalog
            .occurrence_variants
            .iter()
            .map(|row| (row.id, &row.key)),
        "occurrence variants",
    )?;
    check_rows(
        catalog
            .occurrence_choices
            .iter()
            .map(|row| (row.id, &row.key)),
        "occurrence choices",
    )?;
    check_rows(
        catalog.services.iter().map(|row| (row.id, &row.key)),
        "services",
    )?;
    check_rows(
        catalog
            .adventure_outcomes
            .iter()
            .map(|row| (row.id, &row.key)),
        "adventure outcomes",
    )?;
    check_rows(
        catalog
            .encounter_groups
            .iter()
            .map(|row| (row.id, &row.key)),
        "encounter groups",
    )?;
    check_rows(
        catalog.encounter_waves.iter().map(|row| (row.id, &row.key)),
        "encounter waves",
    )?;
    check_rows(
        catalog.enemy_slots.iter().map(|row| (row.id, &row.key)),
        "enemy slots",
    )?;
    check_rows(
        catalog.map_events.iter().map(|row| (row.id, &row.key)),
        "map events",
    )?;
    check_rows(
        catalog
            .block_create_rules
            .iter()
            .map(|row| (row.id, &row.key)),
        "block create rules",
    )?;
    check_rows(
        catalog.mechanic_rules.iter().map(|row| (row.id, &row.key)),
        "mechanic rules",
    )?;
    check_index(&catalog.source_records, "source records")?;
    check_rows(
        catalog.coverage.iter().map(|row| (row.id, &row.key)),
        "coverage",
    )?;
    check_index(&catalog.research_gaps, "research gaps")?;
    check_index(&catalog.gap_affected_records, "gap affected records")?;
    check_index(&catalog.review_fixtures, "review fixtures")?;
    check_index(&catalog.pack_index, "pack index")
}

fn check_index(rows: &[StableIndexRow], label: &str) -> Result<(), GoldAndGearsContentError> {
    check_rows(rows.iter().map(|row| (row.id, &row.key)), label)
}

fn check_rows<'a>(
    rows: impl Iterator<Item = (i32, &'a StableKey)>,
    label: &str,
) -> Result<(), GoldAndGearsContentError> {
    let mut previous = 0;
    let mut ids = BTreeSet::new();
    let mut keys = BTreeSet::new();
    for (id, key) in rows {
        require(
            id > previous && id > 0 && !key.as_str().trim().is_empty(),
            GoldAndGearsContentErrorKind::Metadata,
            label,
        )?;
        require(
            ids.insert(id) && keys.insert(key.as_str()),
            GoldAndGearsContentErrorKind::Duplicate,
            key.as_str(),
        )?;
        previous = id;
    }
    Ok(())
}

fn json_payloads(catalog: &GoldAndGearsContentCatalog) -> Result<(), GoldAndGearsContentError> {
    for payload in catalog
        .blessing_levels
        .iter()
        .map(|row| &row.parameters)
        .chain(
            catalog
                .curio_states
                .iter()
                .flat_map(|row| row.payloads.iter()),
        )
        .chain(
            catalog
                .occurrence_choices
                .iter()
                .flat_map(|row| row.payloads.iter()),
        )
        .chain(catalog.services.iter().flat_map(|row| row.payloads.iter()))
        .chain(
            catalog
                .adventure_outcomes
                .iter()
                .flat_map(|row| row.payloads.iter()),
        )
        .chain(
            catalog
                .encounter_groups
                .iter()
                .flat_map(|row| row.payloads.iter()),
        )
        .chain(catalog.encounter_waves.iter().map(|row| &row.payload))
        .chain(
            catalog
                .mechanic_rules
                .iter()
                .flat_map(|row| row.payloads.iter()),
        )
    {
        validate_json(payload)?;
    }
    Ok(())
}

fn validate_json(payload: &JsonPayload) -> Result<(), GoldAndGearsContentError> {
    serde_json::from_str::<serde_json::Value>(payload.as_str()).map_err(|_| {
        GoldAndGearsContentError {
            kind: GoldAndGearsContentErrorKind::Json,
            key: "retained payload".into(),
        }
    })?;
    Ok(())
}

fn content_references(
    catalog: &GoldAndGearsContentCatalog,
    source: &SoraConfig,
    standard: &UniverseCatalog,
) -> Result<(), GoldAndGearsContentError> {
    let paths = source
        .gold_gears_path()
        .ordered_rows()
        .map(|row| row.stable_key.as_str())
        .collect::<BTreeSet<_>>();
    let levels = key_set(catalog.blessing_levels.iter().map(|row| &row.key));
    let standard_blessings = standard
        .blessings()
        .iter()
        .map(|row| row.stable_key())
        .collect::<BTreeSet<_>>();
    let standard_levels = standard
        .blessing_levels()
        .iter()
        .map(|row| row.stable_key())
        .collect::<BTreeSet<_>>();
    for row in &catalog.blessings {
        require_ref(paths.contains(row.path.as_str()), &row.path)?;
        require(
            standard_blessings.contains(row.key.as_str()),
            GoldAndGearsContentErrorKind::SharedIdentity,
            row.key.as_str(),
        )?;
        for level in &row.levels {
            require_ref(levels.contains(level.as_str()), level)?;
        }
        require(
            !row.inherited_rules.is_empty(),
            GoldAndGearsContentErrorKind::Reference,
            row.key.as_str(),
        )?;
    }
    let blessing_ids = id_set(catalog.blessings.iter().map(|row| row.id));
    for row in &catalog.blessing_levels {
        require_ref(blessing_ids.contains(&row.blessing_id), &row.key)?;
        require(
            standard_levels.contains(row.key.as_str()) && !row.inherited_rules.is_empty(),
            GoldAndGearsContentErrorKind::SharedIdentity,
            row.key.as_str(),
        )?;
    }

    let rule_keys = key_set(catalog.mechanic_rules.iter().map(|row| &row.key));
    let state_ids = id_set(catalog.curio_states.iter().map(|row| row.id));
    let state_keys = key_set(catalog.curio_states.iter().map(|row| &row.key));
    let standard_curios = standard
        .curios()
        .iter()
        .map(|row| row.stable_key())
        .collect::<BTreeSet<_>>();
    for row in &catalog.curios {
        require_ref(state_ids.contains(&row.initial_state_id), &row.key)?;
        require_ref(rule_keys.contains(row.rule.as_str()), &row.rule)?;
        for state in &row.states {
            require_ref(state_keys.contains(state.as_str()), state)?;
        }
        if row.shared {
            require(
                standard_curios.contains(row.key.as_str()),
                GoldAndGearsContentErrorKind::SharedIdentity,
                row.key.as_str(),
            )?;
        }
    }
    let curio_ids = id_set(catalog.curios.iter().map(|row| row.id));
    for row in &catalog.curio_states {
        require_ref(curio_ids.contains(&row.curio_id), &row.key)?;
        require_ref(rule_keys.contains(row.rule.as_str()), &row.rule)?;
    }

    let occurrence_ids = id_set(catalog.occurrences.iter().map(|row| row.id));
    let occurrence_keys = key_set(catalog.occurrences.iter().map(|row| &row.key));
    let variant_ids = id_set(catalog.occurrence_variants.iter().map(|row| row.id));
    let variant_keys = key_set(catalog.occurrence_variants.iter().map(|row| &row.key));
    let choice_keys = key_set(catalog.occurrence_choices.iter().map(|row| &row.key));
    for row in &catalog.occurrences {
        require_ref(rule_keys.contains(row.rule.as_str()), &row.rule)?;
        for variant in &row.variants {
            require_ref(variant_keys.contains(variant.as_str()), variant)?;
        }
    }
    for row in &catalog.occurrence_variants {
        require_ref(occurrence_ids.contains(&row.occurrence_id), &row.key)?;
        require_ref(rule_keys.contains(row.rule.as_str()), &row.rule)?;
        for occurrence in &row.occurrence_keys {
            require_ref(occurrence_keys.contains(occurrence.as_str()), occurrence)?;
        }
        for choice in &row.choices {
            require_ref(choice_keys.contains(choice.as_str()), choice)?;
        }
    }
    let entry_nodes = source
        .gold_gears_occurrence_variant()
        .ordered_rows()
        .map(|row| row.entry_node_id.as_str())
        .collect::<BTreeSet<_>>();
    for row in &catalog.occurrence_choices {
        require_ref(variant_ids.contains(&row.variant_id), &row.key)?;
        require_ref(rule_keys.contains(row.rule.as_str()), &row.rule)?;
        if let Some(next) = &row.next_node {
            require_ref(
                choice_keys.contains(next.as_str()) || entry_nodes.contains(next.as_str()),
                next,
            )?;
        }
    }

    let standard_services = standard
        .services()
        .iter()
        .map(|row| row.stable_key())
        .collect::<BTreeSet<_>>();
    for row in &catalog.services {
        require_ref(rule_keys.contains(row.rule.as_str()), &row.rule)?;
        if row.shared {
            require(
                standard_services.contains(row.key.as_str()),
                GoldAndGearsContentErrorKind::SharedIdentity,
                row.key.as_str(),
            )?;
        }
    }
    let service_ids = id_set(catalog.services.iter().map(|row| row.id));
    let room_keys = source
        .gold_gears_room()
        .ordered_rows()
        .map(|row| row.stable_key.as_str())
        .collect::<BTreeSet<_>>();
    for row in &catalog.adventure_outcomes {
        require(
            service_ids.contains(&row.downloader_service_id),
            GoldAndGearsContentErrorKind::Reference,
            &format!(
                "{}->service:{}",
                row.key.as_str(),
                row.downloader_service_id
            ),
        )?;
        require_ref(room_keys.contains(row.room.as_str()), &row.room)?;
        require_ref(rule_keys.contains(row.rule.as_str()), &row.rule)?;
    }
    Ok(())
}

fn encounter_references(
    catalog: &GoldAndGearsContentCatalog,
    source: &SoraConfig,
    standard: &UniverseCatalog,
) -> Result<(), GoldAndGearsContentError> {
    let area_keys = source
        .gold_gears_area()
        .ordered_rows()
        .map(|row| row.stable_key.as_str())
        .collect::<BTreeSet<_>>();
    let room_keys = source
        .gold_gears_room()
        .ordered_rows()
        .map(|row| row.stable_key.as_str())
        .collect::<BTreeSet<_>>();
    for row in &catalog.encounter_groups {
        if let Some(room) = &row.parent_room {
            require_ref(room_keys.contains(room.as_str()), room)?;
        }
        for area in &row.areas {
            require_ref(area_keys.contains(area.as_str()), area)?;
        }
    }
    let group_ids = id_set(catalog.encounter_groups.iter().map(|row| row.id));
    let wave_ids = id_set(catalog.encounter_waves.iter().map(|row| row.id));
    let slot_keys = key_set(catalog.enemy_slots.iter().map(|row| &row.key));
    for row in &catalog.encounter_waves {
        require_ref(group_ids.contains(&row.group_id), &row.key)?;
        for slot in &row.slots {
            require_ref(slot_keys.contains(slot.as_str()), slot)?;
        }
    }
    let boss_keys = source
        .gold_gears_boss_choice()
        .ordered_rows()
        .map(|row| row.stable_key.as_str())
        .collect::<BTreeSet<_>>();
    let core = standard.simulation_catalog();
    let standard_battle_catalog =
        UniverseBattleCatalogComposition::compile(standard).map_err(|_| {
            error(
                GoldAndGearsContentErrorKind::SharedIdentity,
                "Standard battle catalog composition",
            )
        })?;
    let standard_enemy_keys = standard_battle_catalog
        .enemies()
        .iter()
        .map(|enemy| enemy.stable_key())
        .collect::<BTreeSet<_>>();
    let pending_gold_enemy_keys = GOLD_ENEMY_IDENTITIES_PENDING_P6
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut unresolved_enemy_keys = BTreeSet::new();
    for row in &catalog.enemy_slots {
        require_ref(wave_ids.contains(&row.wave_id), &row.key)?;
        if core.enemy_by_stable_key(row.enemy.as_str()).is_none()
            && !standard_enemy_keys.contains(row.enemy.as_str())
            && !pending_gold_enemy_keys.contains(row.enemy.as_str())
        {
            unresolved_enemy_keys.insert(row.enemy.as_str());
        }
        for boss in &row.boss_choices {
            require_ref(boss_keys.contains(boss.as_str()), boss)?;
        }
    }
    require(
        unresolved_enemy_keys.is_empty(),
        GoldAndGearsContentErrorKind::SharedIdentity,
        &unresolved_enemy_keys
            .into_iter()
            .collect::<Vec<_>>()
            .join(","),
    )?;

    let chessboards = source
        .gold_gears_chessboard()
        .ordered_rows()
        .map(|row| row.id)
        .collect::<BTreeSet<_>>();
    let domains = source
        .gold_gears_domain()
        .ordered_rows()
        .map(|row| row.id)
        .collect::<BTreeSet<_>>();
    let beacons = source
        .gold_gears_beacon()
        .ordered_rows()
        .map(|row| row.stable_key.as_str())
        .collect::<BTreeSet<_>>();
    for row in &catalog.map_events {
        require_ref(chessboards.contains(&row.chessboard_id), &row.key)?;
        require(
            row.weight > 0,
            GoldAndGearsContentErrorKind::Metadata,
            row.key.as_str(),
        )?;
    }
    for row in &catalog.block_create_rules {
        require_ref(chessboards.contains(&row.chessboard_id), &row.key)?;
        require_ref(domains.contains(&row.domain_id), &row.key)?;
        require(
            !row.group_id.is_empty()
                && row.create_counts.iter().all(|value| value.weight > 0)
                && row.beacons.iter().all(|value| {
                    value.weight > 0
                        && value
                            .beacon
                            .as_ref()
                            .is_none_or(|beacon| beacons.contains(beacon.as_str()))
                }),
            GoldAndGearsContentErrorKind::Metadata,
            row.key.as_str(),
        )?;
    }
    Ok(())
}

fn mechanic_and_evidence_references(
    catalog: &GoldAndGearsContentCatalog,
    source: &SoraConfig,
) -> Result<(), GoldAndGearsContentError> {
    let fixtures = key_set(catalog.review_fixtures.iter().map(|row| &row.key));
    let owner_keys = owner_keys(catalog, source);
    for row in &catalog.mechanic_rules {
        require_ref(owner_keys.contains(row.owner.as_str()), &row.owner)?;
        for fixture in &row.fixtures {
            require_ref(fixtures.contains(fixture.as_str()), fixture)?;
        }
        require(
            row.disposition.as_ref() == "ReferenceOnly"
                && row.policy_bound
                    == source
                        .gold_gears_mechanic_rule()
                        .get(&row.id)
                        .is_some_and(|raw| raw.policy_bound),
            GoldAndGearsContentErrorKind::Metadata,
            row.key.as_str(),
        )?;
    }
    let gap_ids = id_set(catalog.research_gaps.iter().map(|row| row.id));
    for row in source
        .gold_gears_research_gap_affected_record()
        .ordered_rows()
    {
        require(
            gap_ids.contains(&row.research_gap_id)
                && row.ordinal >= 0
                && !row.file.is_empty()
                && !row.record_stable_key.is_empty(),
            GoldAndGearsContentErrorKind::Reference,
            &row.record_stable_key,
        )?;
    }
    let source_ids = key_set(catalog.source_records.iter().map(|row| &row.key));
    require(
        source_ids.len() == catalog.source_records.len(),
        GoldAndGearsContentErrorKind::Duplicate,
        "source record stable keys",
    )?;
    Ok(())
}

fn owner_keys<'a>(
    catalog: &'a GoldAndGearsContentCatalog,
    source: &'a SoraConfig,
) -> BTreeSet<&'a str> {
    let mut keys = BTreeSet::new();
    for key in catalog
        .blessings
        .iter()
        .map(|row| row.key.as_str())
        .chain(catalog.blessing_levels.iter().map(|row| row.key.as_str()))
        .chain(catalog.curios.iter().map(|row| row.key.as_str()))
        .chain(catalog.curio_states.iter().map(|row| row.key.as_str()))
        .chain(catalog.occurrences.iter().map(|row| row.key.as_str()))
        .chain(
            catalog
                .occurrence_variants
                .iter()
                .map(|row| row.key.as_str()),
        )
        .chain(
            catalog
                .occurrence_choices
                .iter()
                .map(|row| row.key.as_str()),
        )
        .chain(catalog.services.iter().map(|row| row.key.as_str()))
        .chain(
            catalog
                .adventure_outcomes
                .iter()
                .map(|row| row.key.as_str()),
        )
    {
        keys.insert(key);
    }
    macro_rules! add {
        ($table:expr) => {
            for row in $table.ordered_rows() {
                keys.insert(row.stable_key.as_str());
            }
        };
    }
    add!(source.gold_gears_profile());
    add!(source.gold_gears_conundrum_level());
    add!(source.gold_gears_neural_network());
    add!(source.gold_gears_path_boost());
    add!(source.gold_gears_resonance());
    add!(source.gold_gears_resonance_extrapolation());
    add!(source.gold_gears_resonance_interplay());
    add!(source.gold_gears_dice_definition());
    add!(source.gold_gears_dice_face());
    add!(source.gold_gears_knowledge_rule());
    add!(source.gold_gears_trailblaze_bonus());
    keys
}

fn coverage(
    catalog: &GoldAndGearsContentCatalog,
    source: &SoraConfig,
) -> Result<(), GoldAndGearsContentError> {
    let gaps = key_set(catalog.research_gaps.iter().map(|row| &row.key));
    let mut required = 0_i32;
    let mut accounted = 0_i32;
    let mut data_ready = 0_i32;
    let mut categories = BTreeMap::new();
    for row in &catalog.coverage {
        require(
            row.required >= 0
                && row.accounted == row.required
                && row.data_ready <= row.accounted
                && categories
                    .insert(row.category.as_ref(), row.required)
                    .is_none(),
            GoldAndGearsContentErrorKind::Coverage,
            row.key.as_str(),
        )?;
        for gap in &row.blocking_gaps {
            require_ref(gaps.contains(gap.as_str()), gap)?;
        }
        required = required
            .checked_add(row.required)
            .ok_or_else(|| error(GoldAndGearsContentErrorKind::Coverage, "required overflow"))?;
        accounted = accounted
            .checked_add(row.accounted)
            .ok_or_else(|| error(GoldAndGearsContentErrorKind::Coverage, "accounted overflow"))?;
        data_ready = data_ready.checked_add(row.data_ready).ok_or_else(|| {
            error(
                GoldAndGearsContentErrorKind::Coverage,
                "data-ready overflow",
            )
        })?;
    }
    let manifest = source
        .gold_gears_manifest()
        .get(&1)
        .ok_or_else(|| error(GoldAndGearsContentErrorKind::Coverage, "missing manifest"))?;
    require(
        required == manifest.frozen_source_obligations
            && accounted == required
            && data_ready <= accounted,
        GoldAndGearsContentErrorKind::Coverage,
        "coverage totals",
    )
}

fn standard_catalog() -> Result<std::sync::Arc<UniverseCatalog>, GoldAndGearsContentError> {
    let core = starclock_data::catalog::load(CORE_BUNDLE)
        .map_err(|_| error(GoldAndGearsContentErrorKind::SharedIdentity, "core catalog"))?;
    UniverseCatalog::load(UNIVERSE_BUNDLE, core).map_err(|_| {
        error(
            GoldAndGearsContentErrorKind::SharedIdentity,
            "Standard Universe catalog",
        )
    })
}

fn key_set<'a>(rows: impl Iterator<Item = &'a StableKey>) -> BTreeSet<&'a str> {
    rows.map(StableKey::as_str).collect()
}

fn id_set(rows: impl Iterator<Item = i32>) -> BTreeSet<i32> {
    rows.collect()
}

fn require_ref(condition: bool, key: &StableKey) -> Result<(), GoldAndGearsContentError> {
    require(
        condition,
        GoldAndGearsContentErrorKind::Reference,
        key.as_str(),
    )
}

fn require(
    condition: bool,
    kind: GoldAndGearsContentErrorKind,
    key: &str,
) -> Result<(), GoldAndGearsContentError> {
    if condition {
        Ok(())
    } else {
        Err(error(kind, key))
    }
}

fn error(kind: GoldAndGearsContentErrorKind, key: &str) -> GoldAndGearsContentError {
    GoldAndGearsContentError {
        kind,
        key: key.into(),
    }
}
