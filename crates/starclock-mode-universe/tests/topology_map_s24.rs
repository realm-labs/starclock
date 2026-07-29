use std::{
    collections::BTreeSet,
    sync::{Arc, OnceLock},
};

use sha2::{Digest, Sha256};
use starclock_mode_universe::catalog::UniverseCatalog;

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");
const SOURCE_MAPS: [u32; 5] = [302, 303, 304, 305, 401];

fn catalog() -> &'static UniverseCatalog {
    static CATALOG: OnceLock<Arc<UniverseCatalog>> = OnceLock::new();
    CATALOG
        .get_or_init(|| {
            let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
            UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe catalog")
        })
        .as_ref()
}

fn text(hasher: &mut Sha256, value: &str) {
    hasher.update(u64::try_from(value.len()).unwrap().to_le_bytes());
    hasher.update(value.as_bytes());
}

#[test]
fn goal07_p5_m15_s24_materializes_five_complete_topology_maps() {
    let selected = catalog()
        .topologies()
        .iter()
        .filter(|topology| SOURCE_MAPS.contains(&topology.source_map_id()))
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), SOURCE_MAPS.len());
    assert!(
        selected
            .iter()
            .map(|topology| topology.source_map_id())
            .eq(SOURCE_MAPS)
    );
    assert!(selected.iter().all(|topology| topology.nodes().len() == 17));
    assert!(selected.iter().all(|topology| {
        topology
            .nodes()
            .iter()
            .map(|node| node.outgoing().len())
            .sum::<usize>()
            == 21
    }));
    assert!(
        selected
            .iter()
            .all(|topology| topology.terminals().len() == 1)
    );
    assert!(
        selected
            .iter()
            .flat_map(|topology| topology.nodes())
            .map(|node| node.id().get())
            .eq(87_u32..=171)
    );
    assert_eq!(
        selected
            .iter()
            .map(|topology| topology.nodes().len())
            .sum::<usize>(),
        85
    );
    assert_eq!(
        selected
            .iter()
            .flat_map(|topology| topology.nodes())
            .map(|node| node.outgoing().len())
            .sum::<usize>(),
        105
    );

    let mut hasher = Sha256::new();
    text(&mut hasher, "starclock-goal07-p5-m15-s24-topology-maps-v1");
    for topology in selected {
        hasher.update(topology.id().get().to_le_bytes());
        hasher.update(topology.source_map_id().to_le_bytes());
        hasher.update(topology.start().get().to_le_bytes());
        for terminal in topology.terminals() {
            hasher.update(terminal.get().to_le_bytes());
        }
        for node in topology.nodes() {
            hasher.update(node.id().get().to_le_bytes());
            text(&mut hasher, node.stable_key());
            hasher.update(node.source_node_id().to_le_bytes());
            hasher.update(u64::try_from(node.outgoing().len()).unwrap().to_le_bytes());
            for target in node.outgoing() {
                hasher.update(target.get().to_le_bytes());
            }
        }
    }
    let digest: [u8; 32] = hasher.finalize().into();
    assert_eq!(
        digest,
        [
            67, 23, 55, 130, 124, 160, 59, 61, 22, 145, 191, 110, 143, 133, 167, 171, 253, 52, 239,
            152, 132, 193, 103, 68, 163, 108, 191, 12, 77, 241, 190, 233,
        ]
    );
}

#[test]
fn goal07_p5_m15_s24_keeps_every_edge_inside_one_bounded_map() {
    for topology in catalog()
        .topologies()
        .iter()
        .filter(|topology| SOURCE_MAPS.contains(&topology.source_map_id()))
    {
        assert_eq!(
            topology
                .node(topology.start())
                .expect("start node")
                .source_node_id(),
            1
        );
        let node_ids = topology
            .nodes()
            .iter()
            .map(|node| node.id())
            .collect::<BTreeSet<_>>();
        assert!(
            topology
                .nodes()
                .iter()
                .flat_map(|node| node.outgoing())
                .all(|target| node_ids.contains(target))
        );
        assert!(topology.terminals().iter().all(|terminal| {
            topology
                .node(*terminal)
                .is_some_and(|node| node.is_terminal())
        }));
    }
}
