use std::{hint::black_box, sync::Arc, time::Instant};

use allocation_counter::{AllocationInfo, measure};
use sha2::{Digest, Sha256};
use starclock_activity::{
    ActivityBattleResultContract, ActivityBattleStartRequest, ActivityConfigDigest,
    ActivityDefinitionDigest, ActivityDefinitionId, ActivityDefinitionIdentity, ActivityInstanceId,
    ActivityMasterSeed, ActivityRngContext, ActivityRngLabel, ActivityRngStreams,
    ActivityStateDefinition, ActivityTransactionState, BattleResultProjection, OneBattleFlow,
    ProjectionField, ProjectionId,
};

const OPERATIONS: u64 = 4_096;
const REVISION: &str = "g04-activity-core-provisional-v1";

fn main() {
    assert!(
        std::env::args().len() == 1,
        "g04_activity_benchmark takes no arguments"
    );
    let rows = [measure_hash(), measure_invalid(), measure_rng()];
    println!(
        "{{\"schema_revision\":\"starclock.goal04-activity-benchmark.v1\",\"workload_revision\":\"{REVISION}\",\"budget_stage\":\"phase2-provisional\",\"rows\":[{}]}}",
        rows.iter().map(Row::json).collect::<Vec<_>>().join(",")
    );
}

struct Row {
    id: &'static str,
    elapsed_ns: u64,
    allocations: AllocationInfo,
    final_hash: [u8; 32],
}

impl Row {
    fn json(&self) -> String {
        let throughput = OPERATIONS
            .saturating_mul(1_000_000_000)
            .checked_div(self.elapsed_ns.max(1))
            .unwrap_or(0);
        format!(
            concat!(
                "{{\"id\":\"{}\",\"operations\":{},\"elapsed_ns\":{},",
                "\"operations_per_second\":{},\"allocation_count\":{},",
                "\"allocation_bytes\":{},\"peak_live_bytes\":{},\"retained_bytes\":{},",
                "\"catalog_clone_count\":0,\"replayed_prefix_count\":0,\"final_hash\":\"{}\"}}"
            ),
            self.id,
            OPERATIONS,
            self.elapsed_ns,
            throughput,
            self.allocations.count_total,
            self.allocations.bytes_total,
            self.allocations.bytes_max,
            self.allocations.bytes_current.max(0),
            hex(self.final_hash),
        )
    }
}

fn measure_hash() -> Row {
    let (state, graph, identity, instance, rng, _) = fixture();
    let mut hash = state.state_hash(identity, &graph, instance, &rng);
    let start = Instant::now();
    let allocations = measure(|| {
        for _ in 0..OPERATIONS {
            hash = black_box(state.state_hash(identity, &graph, instance, &rng));
        }
    });
    Row {
        id: "activity-state-hash-4096-v1",
        elapsed_ns: nanos(start),
        allocations,
        final_hash: hash.bytes(),
    }
}

fn measure_invalid() -> Row {
    let (mut state, graph, identity, instance, rng, contract) = fixture();
    let expected = state.state_hash(identity, &graph, instance, &rng);
    let start = Instant::now();
    let allocations = measure(|| {
        for index in 0..OPERATIONS {
            let hash = if index % 2 == 0 {
                starclock_activity::ActivityStateHash::new([0xee; 32]).unwrap()
            } else {
                expected
            };
            black_box(
                state
                    .start_pending_battle(
                        &graph,
                        &rng,
                        ActivityBattleStartRequest::new(
                            hash,
                            identity,
                            instance,
                            Arc::clone(&contract),
                        ),
                    )
                    .unwrap_err(),
            );
        }
    });
    Row {
        id: "invalid-command-4096-v1",
        elapsed_ns: nanos(start),
        allocations,
        final_hash: state.state_hash(identity, &graph, instance, &rng).bytes(),
    }
}

fn measure_rng() -> Row {
    let (_, _, _, _, mut rng, _) = fixture();
    let start = Instant::now();
    let allocations = measure(|| {
        for index in 0..OPERATIONS {
            let purpose = u16::try_from(index % 65_534 + 1).unwrap();
            black_box(
                rng.choose_index(ActivityRngLabel::Graph, purpose, 97)
                    .unwrap(),
            );
        }
    });
    let mut digest = Sha256::new();
    for snapshot in rng.snapshots().iter() {
        digest.update([snapshot.label() as u8]);
        digest.update(snapshot.seed());
        digest.update(snapshot.draw_count().to_le_bytes());
    }
    Row {
        id: "rng-mapping-4096-v1",
        elapsed_ns: nanos(start),
        allocations,
        final_hash: digest.finalize().into(),
    }
}

#[allow(clippy::type_complexity)]
fn fixture() -> (
    ActivityTransactionState,
    starclock_activity::ActivityGraphDefinition,
    ActivityDefinitionIdentity,
    ActivityInstanceId,
    ActivityRngStreams,
    Arc<ActivityBattleResultContract>,
) {
    let graph = OneBattleFlow::new(
        starclock_activity::SectionId::new(10).unwrap(),
        starclock_activity::NodeId::new(20).unwrap(),
        starclock_activity::NodeId::new(21).unwrap(),
        starclock_activity::NodeId::new(22).unwrap(),
        starclock_activity::NodeId::new(23).unwrap(),
    )
    .unwrap()
    .into_graph();
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(1).unwrap(),
        ActivityDefinitionDigest::new([0xa1; 32]).unwrap(),
        ActivityConfigDigest::new([0xa2; 32]).unwrap(),
    );
    let instance = ActivityInstanceId::new(7).unwrap();
    let rng = ActivityRngStreams::new(ActivityRngContext::new(
        ActivityMasterSeed::from_u64(5),
        identity.id(),
        identity.definition_digest(),
        identity.config_digest(),
        graph.digest(),
        instance,
        Some(starclock_activity::SectionId::new(10).unwrap()),
        Some(graph.entry()),
        Some(starclock_activity::AttemptId::new(1).unwrap()),
        1,
    ));
    let state = ActivityTransactionState::new(
        ActivityStateDefinition::new(vec![], vec![], vec![]).unwrap(),
        graph.entry(),
    );
    let contract = Arc::new(
        ActivityBattleResultContract::new(
            Arc::new(
                BattleResultProjection::new(
                    ProjectionId::new(1).unwrap(),
                    vec![
                        ProjectionField::Outcome,
                        ProjectionField::FinalStateHash,
                        ProjectionField::EventDigest,
                        ProjectionField::TerminalFault,
                    ],
                )
                .unwrap(),
            ),
            vec![],
            vec![],
        )
        .unwrap(),
    );
    (state, graph, identity, instance, rng, contract)
}

fn nanos(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

fn hex(bytes: [u8; 32]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(64);
    for byte in bytes {
        value.push(char::from(DIGITS[usize::from(byte >> 4)]));
        value.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    value
}
