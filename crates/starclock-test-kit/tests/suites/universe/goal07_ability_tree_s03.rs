use std::{
    collections::BTreeSet,
    sync::{Arc, OnceLock},
};

use starclock_activity::{
    BuildDigest, LoadoutLockScope, OpaqueParticipantBuild, ParticipantId, ParticipantLock,
    ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope,
};
use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};
use starclock_mode_universe::{
    ability_runtime::{
        AbilityBoundary, AbilityExecutionContext, AbilityProjectionScope, AbilityRuntimeCatalog,
        AbilityTarget,
    },
    blessing_runtime::BlessingRuntimeCatalog,
    catalog::UniverseCatalog,
    entry::{StandardUniverseEntry, StandardUniverseProfile},
    path_runtime::PathRuntimeCatalog,
    progression::AbilityOperation,
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] =
    include_bytes!("../../../../../config/universe-generated/config.sora");
const RECORDS: [&str; 10] = [
    "universe.ability-tree.39",
    "universe.ability-tree.4",
    "universe.ability-tree.40",
    "universe.ability-tree.41",
    "universe.ability-tree.42",
    "universe.ability-tree.5",
    "universe.ability-tree.6",
    "universe.ability-tree.7",
    "universe.ability-tree.8",
    "universe.ability-tree.9",
];

fn catalog() -> Arc<UniverseCatalog> {
    static CATALOG: OnceLock<Arc<UniverseCatalog>> = OnceLock::new();
    Arc::clone(CATALOG.get_or_init(|| {
        let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
        UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe catalog")
    }))
}

#[test]
fn goal07_p2_m01_s03_executes_every_assigned_rule_and_operation_fixture() {
    let catalog = catalog();
    let selected = catalog
        .ability_tree_nodes()
        .iter()
        .filter(|node| RECORDS.contains(&node.stable_key()))
        .map(|node| node.id())
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), RECORDS.len());
    let runtime = AbilityRuntimeCatalog::compile(&catalog).expect("Ability runtime");
    let contexts = [
        AbilityExecutionContext::run_start(),
        AbilityExecutionContext::new(
            AbilityProjectionScope::Run,
            AbilityBoundary::AfterBattle,
            14,
            false,
        ),
        AbilityExecutionContext::new(
            AbilityProjectionScope::Battle,
            AbilityBoundary::BattleStart,
            14,
            false,
        ),
    ];
    let projections =
        contexts.map(|context| runtime.project(&selected, context).expect("projection"));
    let executed = projections
        .iter()
        .flat_map(|projection| projection.applied_effects())
        .map(|effect| effect.source())
        .collect::<BTreeSet<_>>();
    assert_eq!(executed, selected.iter().copied().collect());
    let operations = projections
        .iter()
        .flat_map(|projection| projection.applied_effects())
        .map(|effect| effect.operation())
        .collect::<BTreeSet<_>>();
    assert!(
        BTreeSet::from([AbilityOperation::Set, AbilityOperation::UnlockFormationSlot])
            .is_subset(&operations)
    );

    let run_start = &projections[0];
    assert_raw(run_start, AbilityTarget::ServiceReviver, 1_000_000);
    assert_raw(
        run_start,
        AbilityTarget::ServiceReviverRestoredHpRatio,
        1_000_000,
    );
    assert_raw(run_start, AbilityTarget::RunConsumableUse, 1_000_000);

    let after_battle = &projections[1];
    assert_raw(after_battle, AbilityTarget::RunPathResonance, 1_000_000);

    let battle = &projections[2];
    assert_raw(battle, AbilityTarget::PartyDefenseFlat, 65_000_000);
    assert_raw(battle, AbilityTarget::PartyMaximumHpFlat, 60_000_000);
    assert_raw(battle, AbilityTarget::PartyAttackFlat, 40_000_000);
    assert_raw(battle, AbilityTarget::PartyEffectHitRateRatio, 80_000);
    assert_raw(battle, AbilityTarget::PathResonanceDamageRatio, 300_000);
}

#[test]
fn selected_tree_materializes_generic_run_capabilities_and_roster_safe_topology() {
    let catalog = catalog();
    let profile = StandardUniverseProfile::new(Arc::clone(&catalog));
    let world = &catalog.worlds()[0];
    let empty = profile
        .compile(StandardUniverseEntry::new(
            world.id(),
            world.difficulties()[0],
            participants(1),
            vec![],
        ))
        .expect("entry without Ability Tree");
    let empty_capabilities = empty.initial_run_capabilities().unwrap();
    assert_eq!(empty_capabilities.formation_slots(), 0);
    assert!(!empty_capabilities.reviver());
    assert_eq!(empty_capabilities.reviver_restored_hp_ratio().scaled(), 0);
    assert!(!empty_capabilities.consumable_use());

    let selected = catalog
        .ability_tree_nodes()
        .iter()
        .map(|node| node.id())
        .collect::<Vec<_>>();
    let complete = profile
        .compile(StandardUniverseEntry::new(
            world.id(),
            world.difficulties()[0],
            participants(11),
            selected,
        ))
        .expect("complete Ability Tree entry");
    let capabilities = complete.initial_run_capabilities().unwrap();
    assert_eq!(capabilities.formation_slots(), 3);
    assert!(capabilities.reviver());
    assert_eq!(capabilities.reviver_restored_hp_ratio().scaled(), 1_000_000);
    assert!(capabilities.consumable_use());

    let sources = complete
        .abstract_interactions()
        .iter()
        .map(|binding| binding.source_content_id())
        .filter(|source| source.starts_with("universe.service.reviver.participant."))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        sources,
        BTreeSet::from([
            "universe.service.reviver.participant.11",
            "universe.service.reviver.participant.12",
            "universe.service.reviver.participant.13",
            "universe.service.reviver.participant.14",
        ])
    );
}

#[test]
fn formation_selection_obeys_both_blessing_threshold_and_ability_capacity() {
    let catalog = catalog();
    let path = &catalog.paths()[0];
    let blessings = BlessingRuntimeCatalog::compile(&catalog)
        .unwrap()
        .contributions_from_owned(
            &path
                .blessings()
                .iter()
                .take(14)
                .map(|id| (*id, 1))
                .collect::<Vec<_>>(),
        )
        .unwrap();
    let formations = path
        .formations()
        .iter()
        .map(|id| (*id, 1))
        .collect::<Vec<_>>();
    let runtime = PathRuntimeCatalog::compile(&catalog).unwrap();
    assert!(
        runtime
            .contributions_with_formation_slots(path.id(), &blessings, &formations[..1], 0)
            .is_err()
    );
    assert_eq!(
        runtime
            .contributions_with_formation_slots(path.id(), &blessings, &formations[..1], 1)
            .unwrap()
            .formations()
            .len(),
        1
    );
    assert!(
        runtime
            .contributions_with_formation_slots(path.id(), &blessings, &formations[..2], 1)
            .is_err()
    );
    assert_eq!(
        runtime
            .contributions_with_formation_slots(path.id(), &blessings, &formations, 3)
            .unwrap()
            .formations()
            .len(),
        3
    );
}

fn participants(first: u32) -> ParticipantLock {
    let policy = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .unwrap();
    let entries = (0_u8..4)
        .map(|formation| {
            let participant = ParticipantId::new(first + u32::from(formation)).unwrap();
            let digest_byte = u8::try_from(first + u32::from(formation)).unwrap();
            ParticipantLockEntry::new(
                participant,
                0,
                formation,
                UnitDefinitionId::new(20_001 + u32::from(formation)).unwrap(),
                OpaqueParticipantBuild::new(
                    CombatantSpecDigest::new([digest_byte; 32]).unwrap(),
                    BuildDigest::new([digest_byte.wrapping_add(32); 32]).unwrap(),
                    "goal07-ability-tree-s03",
                    ParticipantSourceKind::CompiledBuild,
                )
                .unwrap(),
            )
            .unwrap()
        })
        .collect();
    ParticipantLock::seal(policy, entries).unwrap()
}

fn assert_raw(
    projection: &starclock_mode_universe::ability_runtime::AbilityRuntimeProjection,
    target: AbilityTarget,
    expected: i64,
) {
    assert_eq!(
        projection
            .value(target)
            .map(|value| value.raw_six_decimal()),
        Some(expected)
    );
}
