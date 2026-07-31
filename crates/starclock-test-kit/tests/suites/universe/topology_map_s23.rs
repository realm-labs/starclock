use std::sync::{Arc, OnceLock};

use sha2::{Digest, Sha256};
use starclock_mode_universe::catalog::UniverseCatalog;

const CORE_BUNDLE: &[u8] = include_bytes!("../../../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] =
    include_bytes!("../../../../../config/universe-generated/config.sora");
const SOURCE_MAPS: [u32; 8] = [1, 101, 2, 200, 201, 3, 300, 301];
const NODE_COUNTS: [usize; 8] = [1, 7, 3, 17, 17, 7, 17, 17];
const EDGE_COUNTS: [usize; 8] = [0, 6, 2, 21, 21, 6, 21, 21];

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
fn goal07_p5_m15_s23_materializes_eight_complete_topology_maps() {
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
    assert!(
        selected
            .iter()
            .map(|topology| topology.nodes().len())
            .eq(NODE_COUNTS)
    );
    assert!(
        selected
            .iter()
            .map(|topology| {
                topology
                    .nodes()
                    .iter()
                    .map(|node| node.outgoing().len())
                    .sum::<usize>()
            })
            .eq(EDGE_COUNTS)
    );
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
            .eq(1_u32..=86)
    );
    assert_eq!(
        selected
            .iter()
            .map(|topology| topology.nodes().len())
            .sum::<usize>(),
        86
    );
    assert_eq!(
        selected
            .iter()
            .flat_map(|topology| topology.nodes())
            .map(|node| node.outgoing().len())
            .sum::<usize>(),
        98
    );

    let mut hasher = Sha256::new();
    text(&mut hasher, "starclock-goal07-p5-m15-s23-topology-maps-v1");
    for topology in selected {
        hasher.update(topology.id().get().to_le_bytes());
        hasher.update(topology.source_map_id().to_le_bytes());
        hasher.update(topology.start().get().to_le_bytes());
        hasher.update(
            u64::try_from(topology.terminals().len())
                .unwrap()
                .to_le_bytes(),
        );
        for terminal in topology.terminals() {
            hasher.update(terminal.get().to_le_bytes());
        }
        hasher.update(u64::try_from(topology.nodes().len()).unwrap().to_le_bytes());
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
            148, 109, 46, 198, 208, 34, 139, 201, 69, 2, 44, 193, 230, 252, 12, 124, 124, 206, 62,
            230, 57, 117, 28, 79, 93, 206, 103, 122, 149, 199, 178, 208,
        ]
    );
}

#[test]
fn goal07_p5_m15_s23_starts_reach_one_terminal_without_cross_map_edges() {
    for topology in catalog()
        .topologies()
        .iter()
        .filter(|topology| SOURCE_MAPS.contains(&topology.source_map_id()))
    {
        let start = topology.node(topology.start()).expect("start node");
        assert_eq!(start.source_node_id(), 1);
        let node_ids = topology
            .nodes()
            .iter()
            .map(|node| node.id())
            .collect::<std::collections::BTreeSet<_>>();
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
