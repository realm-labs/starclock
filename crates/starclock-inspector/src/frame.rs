use starclock_combat::{
    Battle, BattleDiagnostics, BattleEvent, CommandId, DiagnosticRecord, Resolution,
};

use crate::BattleSnapshot;

/// Rejected frame assembly caused by mixing unrelated resolution/state values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InspectorFrameError {
    BoundaryMismatch,
}

impl core::fmt::Display for InspectorFrameError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("inspector resolution does not match the captured battle boundary")
    }
}

impl std::error::Error for InspectorFrameError {}

/// One owned stable-boundary frame suitable for caching or later transport.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectorFrame {
    pub root_command: Option<CommandId>,
    pub snapshot: BattleSnapshot,
    pub events: Box<[BattleEvent]>,
    pub diagnostics: Box<[DiagnosticRecord]>,
    pub diagnostics_truncated: bool,
}

impl InspectorFrame {
    /// Captures a boundary before any accepted command has resolved.
    #[must_use]
    pub fn initial(battle: &Battle) -> Self {
        Self {
            root_command: None,
            snapshot: BattleSnapshot::capture(battle),
            events: Box::new([]),
            diagnostics: Box::new([]),
            diagnostics_truncated: false,
        }
    }

    /// Captures a regular post-command boundary without resolver diagnostics.
    pub fn after_resolution(
        battle: &Battle,
        resolution: &Resolution,
    ) -> Result<Self, InspectorFrameError> {
        let snapshot = matching_snapshot(battle, resolution)?;
        Ok(Self {
            root_command: Some(resolution.root_command()),
            snapshot,
            events: resolution.events().into(),
            diagnostics: Box::new([]),
            diagnostics_truncated: false,
        })
    }

    /// Captures an inspected post-command boundary and its resolver diagnostics.
    pub fn after_inspected_resolution(
        battle: &Battle,
        resolution: &Resolution,
        diagnostics: &BattleDiagnostics,
    ) -> Result<Self, InspectorFrameError> {
        let snapshot = matching_snapshot(battle, resolution)?;
        if diagnostics.root_command() != Some(resolution.root_command())
            || diagnostics.committed_revision() != Some(resolution.committed_revision())
            || diagnostics.state_hash() != Some(resolution.state_hash())
        {
            return Err(InspectorFrameError::BoundaryMismatch);
        }
        Ok(Self {
            root_command: Some(resolution.root_command()),
            snapshot,
            events: resolution.events().into(),
            diagnostics: diagnostics.records().into(),
            diagnostics_truncated: diagnostics.truncated(),
        })
    }
}

fn matching_snapshot(
    battle: &Battle,
    resolution: &Resolution,
) -> Result<BattleSnapshot, InspectorFrameError> {
    let snapshot = BattleSnapshot::capture(battle);
    if snapshot.state_hash != resolution.state_hash()
        || snapshot.committed_revision != resolution.committed_revision()
        || snapshot.rng_draw_count != resolution.rng_draw_count()
        || snapshot.phase != resolution.phase()
    {
        return Err(InspectorFrameError::BoundaryMismatch);
    }
    Ok(snapshot)
}
