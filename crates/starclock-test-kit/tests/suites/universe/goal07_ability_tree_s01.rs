use std::{
    collections::BTreeSet,
    sync::{Arc, OnceLock},
};

use starclock_activity::ActivitySlotId;
use starclock_mode_universe::{
    ability_runtime::{
        AbilityBoundary, AbilityExecutionContext, AbilityProjectionScope, AbilityRuntimeCatalog,
        AbilityTarget,
    },
    catalog::UniverseCatalog,
    progression::AbilityOperation,
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] =
    include_bytes!("../../../../../config/universe-generated/config.sora");
const RECORDS: [&str; 16] = [
    "universe.ability-tree.1",
    "universe.ability-tree.10",
    "universe.ability-tree.11",
    "universe.ability-tree.12",
    "universe.ability-tree.13",
    "universe.ability-tree.14",
    "universe.ability-tree.15",
    "universe.ability-tree.16",
    "universe.ability-tree.17",
    "universe.ability-tree.18",
    "universe.ability-tree.19",
    "universe.ability-tree.2",
    "universe.ability-tree.20",
    "universe.ability-tree.21",
    "universe.ability-tree.22",
    "universe.ability-tree.23",
];

fn catalog() -> &'static UniverseCatalog {
    static CATALOG: OnceLock<Arc<UniverseCatalog>> = OnceLock::new();
    CATALOG
        .get_or_init(|| {
            let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
            UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe catalog")
        })
        .as_ref()
}

#[test]
fn goal07_p2_m01_s01_executes_every_assigned_rule_and_operation_fixture() {
    let catalog = catalog();
    let nodes = catalog
        .ability_tree_nodes()
        .iter()
        .filter(|node| RECORDS.contains(&node.stable_key()))
        .collect::<Vec<_>>();
    assert_eq!(nodes.len(), RECORDS.len());
    assert_eq!(
        nodes
            .iter()
            .map(|node| node.stable_key())
            .collect::<BTreeSet<_>>(),
        RECORDS.into_iter().collect()
    );
    assert!(nodes.iter().all(|node| {
        node.rule_key()
            == format!(
                "universe.rule.ability-tree.{}",
                node.stable_key().rsplit('.').next().unwrap()
            )
            && !node.effects().is_empty()
    }));

    let selected = nodes.iter().map(|node| node.id()).collect::<Vec<_>>();
    let runtime = AbilityRuntimeCatalog::compile(catalog).expect("Ability runtime");
    let contexts = [
        AbilityExecutionContext::run_start(),
        AbilityExecutionContext::new(
            AbilityProjectionScope::Run,
            AbilityBoundary::AfterBattle,
            14,
            true,
        ),
        AbilityExecutionContext::new(
            AbilityProjectionScope::Battle,
            AbilityBoundary::BattleStart,
            14,
            true,
        ),
        AbilityExecutionContext::new(
            AbilityProjectionScope::Battle,
            AbilityBoundary::EnterEliteOrBossDomain,
            14,
            true,
        ),
    ];
    let projections =
        contexts.map(|context| runtime.project(&selected, context).expect("projection"));
    let executed_sources = projections
        .iter()
        .flat_map(|projection| projection.applied_effects())
        .map(|effect| effect.source())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        executed_sources,
        selected.iter().copied().collect::<BTreeSet<_>>()
    );
    let operations = projections
        .iter()
        .flat_map(|projection| projection.applied_effects())
        .map(|effect| effect.operation())
        .collect::<BTreeSet<_>>();
    assert!(
        BTreeSet::from([
            AbilityOperation::AddChoice,
            AbilityOperation::AddCurrency,
            AbilityOperation::AddLimit,
            AbilityOperation::AddStat,
            AbilityOperation::Enable,
            AbilityOperation::Unlock,
        ])
        .is_subset(&operations)
    );

    let run_start = &projections[0];
    let mut current = run_start
        .values()
        .iter()
        .map(|value| {
            (
                value.target().activity_key(),
                value.value().raw_six_decimal(),
            )
        })
        .collect::<Vec<_>>();
    current.sort_unstable_by_key(|entry| entry.0);
    let delta = runtime
        .project_activity_delta_operations(
            &selected,
            contexts[1],
            ActivitySlotId::new(19).unwrap(),
            &current,
        )
        .expect("post-battle delta");
    assert_eq!(
        delta
            .projection()
            .value(AbilityTarget::FirstBattleBlessingCount)
            .and_then(|value| value.integral()),
        Some(1)
    );
    assert!(!delta.operations().is_empty());
}
