use super::*;

const BUNDLE: &[u8] = include_bytes!("../../../../config/gold-and-gears-generated/config.sora");

#[test]
fn lowers_all_remaining_tables_and_closes_cross_catalog_references() {
    let catalog = GoldAndGearsContentCatalog::load(BUNDLE).unwrap();
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
    assert_eq!(
        counts,
        [
            162, 324, 80, 80, 62, 65, 257, 15, 8, 181, 478, 1_513, 332, 1_091, 1_224, 9_082, 42,
            16, 5_025, 18, 1
        ]
    );
    assert_eq!(catalog.row_count(), EXPECTED_CONTENT_ROWS);
    assert!(catalog.map_events.iter().all(|event| event.weight > 0));
    assert!(catalog.block_create_rules.iter().all(|rule| {
        !rule.group_id.is_empty()
            && rule
                .create_counts
                .iter()
                .all(|candidate| candidate.weight > 0)
            && rule.beacons.iter().all(|candidate| candidate.weight > 0)
    }));
    assert_eq!(
        catalog
            .map_events
            .iter()
            .filter(|event| event.trigger == MapEventTrigger::EnterCell)
            .count(),
        221
    );
    assert_eq!(
        catalog
            .map_events
            .iter()
            .filter(|event| event.trigger == MapEventTrigger::EnterRow)
            .count(),
        111
    );
    assert_eq!(
        [
            MapEventEffect::AddActionPoint,
            MapEventEffect::GrantCurio,
            MapEventEffect::GenerateMark,
            MapEventEffect::RandomReplace,
            MapEventEffect::Replace,
            MapEventEffect::Shuffle,
        ]
        .map(|effect| {
            catalog
                .map_events
                .iter()
                .filter(|event| event.effect == effect)
                .count()
        }),
        [81, 30, 80, 31, 80, 30]
    );
}
