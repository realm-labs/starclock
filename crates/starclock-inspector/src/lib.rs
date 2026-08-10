//! ID-only, read-only battle inspection snapshots and diagnostic frames.
//!
//! This crate owns no presentation metadata and performs no configuration
//! lookup. A UI may join its typed IDs against an independently loaded catalog
//! whose digest matches the captured battle identity.

#![forbid(unsafe_code)]

mod diff;
mod frame;
mod history;
mod snapshot;

pub use diff::{BattleSnapshotDiff, RuntimeEntityRef, SnapshotSection};
pub use frame::{InspectorFrame, InspectorFrameError};
pub use history::{InspectorHistory, InspectorHistoryError, MAX_RETAINED_INSPECTOR_FRAMES};
pub use snapshot::{
    ActionBoundarySnapshot, ActionFrameSnapshot, ActiveTurnSnapshot, BattleClockSnapshot,
    BattleIdentitySnapshot, BattleSnapshot, BreakEffectSnapshot, CharacterResourceSnapshot,
    EffectSnapshot, EncounterSnapshot, FormationSnapshot, LinkSnapshot, ModifierSnapshot,
    PendingExtraTurnSnapshot, PendingReactionSnapshot, PreparedActionSnapshot,
    RuleInstanceSnapshot, SequenceCursorsSnapshot, ShieldSnapshot, TeamResourceSnapshot,
    TeamSnapshot, TemporaryWeaknessSnapshot, TimelineActorSnapshot, ToughnessLayerSnapshot,
    TransformationSnapshot, UnitSnapshot,
};

#[cfg(test)]
mod tests {
    use starclock_combat::{BattleDiagnostics, Command, TeamSide};

    use super::{
        BattleSnapshot, BattleSnapshotDiff, InspectorFrame, InspectorHistory, RuntimeEntityRef,
    };

    fn battle(seed: u64) -> starclock_data::standard::StandardBattle {
        starclock_data::standard::instantiate(starclock_data::standard::SCENARIOS[0].0, Some(seed))
            .expect("production Standard fixture loads")
    }

    fn start_command(battle: &starclock_combat::Battle) -> Command {
        Command::StartBattle {
            decision: battle.decision().expect("initial decision").id(),
        }
    }

    #[test]
    fn owned_snapshot_captures_a_stable_boundary_and_diff() {
        let mut fixture = battle(77);
        let battle = fixture.battle_mut();
        let before = BattleSnapshot::capture(battle);
        let initial_frame = InspectorFrame::initial(battle);
        assert_eq!(before.state_hash, battle.state_hash());
        assert_eq!(before.committed_revision, 0);
        assert_eq!(before.teams[0].side, TeamSide::Player);
        assert_eq!(before.teams[1].side, TeamSide::Enemy);

        let mut diagnostics = BattleDiagnostics::new();
        let resolution = battle
            .apply_inspected(start_command(battle), &mut diagnostics)
            .expect("start command resolves");
        let frame = InspectorFrame::after_inspected_resolution(battle, &resolution, &diagnostics)
            .expect("resolution matches battle boundary");
        assert_eq!(frame.snapshot.state_hash, resolution.state_hash());
        assert_eq!(frame.snapshot.committed_revision, 1);
        assert_eq!(frame.events.as_ref(), resolution.events());

        let mut history = InspectorHistory::new(1).expect("bounded history");
        history.push(initial_frame);
        history.push(frame.clone());
        assert_eq!(history.len(), 1);
        assert_eq!(history.dropped(), 1);
        assert_eq!(
            &history.frames().next().expect("latest frame").snapshot,
            &frame.snapshot
        );

        let diff = BattleSnapshotDiff::between(&before, &frame.snapshot);
        assert_ne!(diff.before_hash, diff.after_hash);
        assert!(
            diff.changed
                .iter()
                .any(|entity| matches!(entity, RuntimeEntityRef::Unit(_)))
                || !diff.added.is_empty()
                || !diff.changed_sections.is_empty()
        );
    }

    #[test]
    fn inspected_and_plain_resolution_are_semantically_identical() {
        let mut plain_fixture = battle(91);
        let mut inspected_fixture = battle(91);
        let plain = plain_fixture.battle_mut();
        let inspected = inspected_fixture.battle_mut();
        let plain_resolution = plain
            .apply(start_command(plain))
            .expect("plain start resolves");
        let plain_frame = InspectorFrame::after_resolution(plain, &plain_resolution)
            .expect("plain resolution matches battle boundary");
        assert!(plain_frame.diagnostics.is_empty());
        let mut diagnostics = BattleDiagnostics::new();
        let inspected_resolution = inspected
            .apply_inspected(start_command(inspected), &mut diagnostics)
            .expect("inspected start resolves");

        assert_eq!(plain_resolution, inspected_resolution);
        assert_eq!(plain.state_hash(), inspected.state_hash());
        assert_eq!(plain.decision(), inspected.decision());
        assert_eq!(
            plain.view().rng_draw_count(),
            inspected.view().rng_draw_count()
        );
        assert_eq!(
            BattleSnapshot::capture(plain),
            BattleSnapshot::capture(inspected)
        );
    }
}
