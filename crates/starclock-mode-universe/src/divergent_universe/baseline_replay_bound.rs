//! Current replay for caller-rebuilt immutable profiles, not serialized graphs.
use super::{
    DivergentUniverseRecordedRun, DivergentUniverseReplayError, DivergentUniverseReplayReport,
    seal_transcript, verify_flow,
};
use crate::digest::Encoder;
use crate::divergent_universe::{
    DivergentUniverseBaselineError, DivergentUniverseBaselineFixture,
    DivergentUniverseBaselineFixtureError, DivergentUniverseBaselinePolicy,
    DivergentUniverseBaselineStep, DivergentUniverseFlowInstance,
};
use starclock_activity::GraphActivity;
use starclock_replay::{
    component::{
        ConfigurationComponentIdentity, ConfigurationComponentKind, ConfigurationComponentSet,
    },
    digest::ComponentDigest,
    format::decode_replay,
};

impl DivergentUniverseFlowInstance {
    /// Seal a caller-driven terminal transcript for this bound position profile.
    /// The exact supplied policy is bound in the Controller component. The
    /// standard current envelope is used; no graph/program/equipment operations
    /// are serialized as authority. Sealing does not prove the transcript: use
    /// verify_bound_replay against a freshly reconstructed trusted profile.
    /// Unbound flows and foreign source components reject without mutation.
    pub fn record_bound_transcript(
        &self,
        fixture: &DivergentUniverseBaselineFixture,
        activity: &GraphActivity,
        seed: u64,
        steps: Vec<DivergentUniverseBaselineStep>,
        policy: &DivergentUniverseBaselinePolicy,
    ) -> Result<DivergentUniverseRecordedRun, DivergentUniverseReplayError> {
        self.require_bound_replay_profile(fixture)?;
        if !self
            .position_battles
            .as_ref()
            .is_some_and(|profile| profile.matches(activity))
        {
            return Err(DivergentUniverseReplayError::Baseline(
                DivergentUniverseBaselineError::DefinitionMismatch,
            ));
        }
        seal_transcript(
            self,
            activity,
            seed,
            steps,
            bound_components(fixture, self, policy)?,
        )
    }

    /// Reapply encoded accepted choices on a fresh Activity and real nested
    /// battles using this caller-rebuilt, already authenticated position flow.
    /// Exact components, entry inputs and policy are checked before commands;
    /// every score/event/hash/result is regenerated and compared at its first
    /// boundary. The input profile and any caller Activity are never mutated.
    /// This does not reconstruct a position recipe from untrusted bytes and
    /// does not establish complete-run parity or default position admission.
    pub fn verify_bound_replay(
        &self,
        bytes: &[u8],
        fixture: &DivergentUniverseBaselineFixture,
        policy: &DivergentUniverseBaselinePolicy,
    ) -> Result<DivergentUniverseReplayReport, DivergentUniverseReplayError> {
        self.require_bound_replay_profile(fixture)?;
        let decoded = decode_replay(bytes).map_err(DivergentUniverseReplayError::Replay)?;
        // Mandatory current entry shape is validated even for empty/malformed
        // transcripts; it never supplies or overrides a trusted profile.
        verify_flow(
            &decoded,
            fixture,
            self,
            policy,
            bound_components(fixture, self, policy)?,
        )
    }

    fn require_bound_replay_profile(
        &self,
        fixture: &DivergentUniverseBaselineFixture,
    ) -> Result<(), DivergentUniverseReplayError> {
        if self.position_battles.is_none() || !self.has_runtime_battle_route() {
            return Err(DivergentUniverseReplayError::Baseline(
                DivergentUniverseBaselineError::MissingBattleRoute,
            ));
        }
        if !self.weighted_curio.matches_factory(fixture.factory())
            || self.component_digest
                != fixture
                    .factory()
                    .bundle_identity()
                    .component_digest()
                    .bytes()
        {
            return Err(DivergentUniverseReplayError::Baseline(
                DivergentUniverseBaselineError::DefinitionMismatch,
            ));
        }
        Ok(())
    }
}

fn bound_components(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    policy: &DivergentUniverseBaselinePolicy,
) -> Result<ConfigurationComponentSet, DivergentUniverseReplayError> {
    let base = fixture
        .components(flow)
        .map_err(DivergentUniverseReplayError::Fixture)?;
    let invalid =
        || DivergentUniverseReplayError::Fixture(DivergentUniverseBaselineFixtureError::Component);
    let mut components = base.components().to_vec();
    let controller = components
        .iter_mut()
        .find(|component| component.kind() == ConfigurationComponentKind::Controller)
        .ok_or_else(invalid)?;
    let mut encoder = Encoder::new(b"starclock.du.bound-position-controller.current");
    encoder.digest(controller.digest().bytes());
    encoder.digest(policy.configuration_digest());
    *controller = ConfigurationComponentIdentity::new(
        controller.kind(),
        controller.id(),
        ComponentDigest::new(encoder.finish()),
    )
    .map_err(|_| invalid())?;
    ConfigurationComponentSet::new(components).map_err(|_| invalid())
}
