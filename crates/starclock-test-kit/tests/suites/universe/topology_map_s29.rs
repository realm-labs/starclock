use std::{
    collections::BTreeSet,
    sync::{Arc, OnceLock},
};

use sha2::{Digest, Sha256};
use starclock_mode_universe::catalog::UniverseCatalog;

const CORE_BUNDLE: &[u8] = include_bytes!("../../../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] =
    include_bytes!("../../../../../config/universe-generated/config.sora");
const SOURCE_MAPS: [u32; 4] = [901, 902, 903, 904];

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
fn goal07_p5_m15_s29_materializes_four_complete_topology_maps() {
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
            .eq(512_u32..=579)
    );
    assert_eq!(
        selected
            .iter()
            .map(|topology| topology.nodes().len())
            .sum::<usize>(),
        68
    );
    assert_eq!(
        selected
            .iter()
            .flat_map(|topology| topology.nodes())
            .map(|node| node.outgoing().len())
            .sum::<usize>(),
        84
    );

    let mut hasher = Sha256::new();
    text(&mut hasher, "starclock-goal07-p5-m15-s29-topology-maps-v1");
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
            205, 180, 254, 69, 224, 107, 185, 65, 16, 73, 115, 100, 135, 167, 208, 184, 111, 150,
            208, 171, 38, 216, 98, 114, 36, 15, 117, 122, 78, 134, 14, 228,
        ]
    );
}

#[test]
fn goal07_p5_m15_s29_keeps_every_edge_inside_one_bounded_map() {
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
