use core::num::NonZeroU32;

use starclock_activity::{
    Activity, ActivityDefinitionId, ActivityInstanceId, ActivityMasterSeed, ActivityPhase,
    ActivitySpec, ProjectionField, TerminalOutcome,
};
use starclock_combat::catalog::encounter::WaveTransitionPolicy;

macro_rules! id_type {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(NonZeroU32);

        impl $name {
            #[must_use]
            pub const fn new(raw: u32) -> Option<Self> {
                match NonZeroU32::new(raw) {
                    Some(value) => Some(Self(value)),
                    None => None,
                }
            }

            #[must_use]
            pub const fn get(self) -> u32 {
                self.0.get()
            }
        }
    };
}

id_type!(
    StandardProfileId,
    "Stable authored Standard profile identity."
);
id_type!(
    StandardScenarioId,
    "Stable authored Standard scenario identity."
);
id_type!(
    StandardBindingId,
    "Stable authored Standard battle-binding identity."
);

/// Exact terminal outcome authored for a reproducible Standard scenario.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum StandardExpectedOutcome {
    Won = 0,
    Lost = 1,
    Faulted = 2,
}

impl StandardExpectedOutcome {
    const fn terminal(self) -> TerminalOutcome {
        match self {
            Self::Won => TerminalOutcome::Complete,
            Self::Lost => TerminalOutcome::Failed,
            Self::Faulted => TerminalOutcome::Faulted,
        }
    }
}

/// Ordinary one-team profile. Unsupported mode layers are absent by construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StandardProfile {
    id: StandardProfileId,
    activity: ActivityDefinitionId,
    maximum_party_size: u8,
    default_wave_transition: WaveTransitionPolicy,
}

impl StandardProfile {
    /// Creates an ordinary profile with exactly one player team and no implicit
    /// clock, score or seasonal-rule layer.
    #[must_use]
    pub const fn new(
        id: StandardProfileId,
        activity: ActivityDefinitionId,
        maximum_party_size: u8,
        default_wave_transition: WaveTransitionPolicy,
    ) -> Option<Self> {
        if maximum_party_size >= 1 && maximum_party_size <= 4 {
            Some(Self {
                id,
                activity,
                maximum_party_size,
                default_wave_transition,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn id(self) -> StandardProfileId {
        self.id
    }
    #[must_use]
    pub const fn activity(self) -> ActivityDefinitionId {
        self.activity
    }
    #[must_use]
    pub const fn player_team_count(self) -> u8 {
        1
    }
    #[must_use]
    pub const fn maximum_party_size(self) -> u8 {
        self.maximum_party_size
    }
    #[must_use]
    pub const fn default_wave_transition(self) -> WaveTransitionPolicy {
        self.default_wave_transition
    }

    pub fn validate_activity(self, activity: &ActivitySpec) -> Result<(), StandardProfileError> {
        if activity.identity().id() != self.activity {
            return Err(StandardProfileError::ActivityIdentityMismatch);
        }
        let participant_policy = activity.participants().policy();
        if participant_policy.team_count() != 1 {
            return Err(StandardProfileError::MultiplePlayerTeams);
        }
        if participant_policy.minimum_team_size() == 0
            || participant_policy.maximum_team_size() > self.maximum_party_size
        {
            return Err(StandardProfileError::PartySizeMismatch);
        }
        let fields = activity.projection().fields();
        let expected = [
            ProjectionField::Outcome,
            ProjectionField::FinalStateHash,
            ProjectionField::EventDigest,
            ProjectionField::TerminalFault,
        ];
        if fields != expected {
            return Err(StandardProfileError::NonStandardProjection);
        }
        Ok(())
    }
}

/// Authored binding identity paired with one already validated generic Activity spec.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandardActivityBinding {
    id: StandardBindingId,
    activity: ActivitySpec,
}

impl StandardActivityBinding {
    #[must_use]
    pub const fn new(id: StandardBindingId, activity: ActivitySpec) -> Self {
        Self { id, activity }
    }
    #[must_use]
    pub const fn id(&self) -> StandardBindingId {
        self.id
    }
    #[must_use]
    pub const fn activity(&self) -> &ActivitySpec {
        &self.activity
    }
}

/// Reproducible Standard scenario with all cross-row links resolved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandardScenario {
    id: StandardScenarioId,
    profile: StandardProfile,
    binding: StandardActivityBinding,
    master_seed: u64,
    expected_outcome: StandardExpectedOutcome,
}

impl StandardScenario {
    pub fn new(
        id: StandardScenarioId,
        profile: StandardProfile,
        binding: StandardActivityBinding,
        master_seed_hex: &str,
        expected_outcome: StandardExpectedOutcome,
    ) -> Result<Self, StandardScenarioError> {
        profile
            .validate_activity(binding.activity())
            .map_err(StandardScenarioError::Profile)?;
        let master_seed =
            parse_seed(master_seed_hex).ok_or(StandardScenarioError::InvalidMasterSeed)?;
        Ok(Self {
            id,
            profile,
            binding,
            master_seed,
            expected_outcome,
        })
    }

    #[must_use]
    pub const fn id(&self) -> StandardScenarioId {
        self.id
    }
    #[must_use]
    pub const fn profile(&self) -> StandardProfile {
        self.profile
    }
    #[must_use]
    pub const fn binding(&self) -> &StandardActivityBinding {
        &self.binding
    }
    #[must_use]
    pub const fn master_seed(&self) -> u64 {
        self.master_seed
    }
    #[must_use]
    pub const fn expected_outcome(&self) -> StandardExpectedOutcome {
        self.expected_outcome
    }

    #[must_use]
    pub fn instantiate(&self, instance: ActivityInstanceId) -> StandardActivity {
        StandardActivity {
            profile: self.profile.id,
            scenario: self.id,
            binding: self.binding.id,
            expected_outcome: self.expected_outcome,
            activity: Activity::new(
                self.binding.activity.clone(),
                instance,
                ActivityMasterSeed::from_u64(self.master_seed),
            ),
        }
    }
}

/// Running Standard wrapper. All mutation remains in the generic Activity aggregate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandardActivity {
    profile: StandardProfileId,
    scenario: StandardScenarioId,
    binding: StandardBindingId,
    expected_outcome: StandardExpectedOutcome,
    activity: Activity,
}

impl StandardActivity {
    #[must_use]
    pub const fn profile_id(&self) -> StandardProfileId {
        self.profile
    }
    #[must_use]
    pub const fn scenario_id(&self) -> StandardScenarioId {
        self.scenario
    }
    #[must_use]
    pub const fn binding_id(&self) -> StandardBindingId {
        self.binding
    }
    #[must_use]
    pub const fn expected_outcome(&self) -> StandardExpectedOutcome {
        self.expected_outcome
    }
    #[must_use]
    pub const fn activity(&self) -> &Activity {
        &self.activity
    }
    #[must_use]
    pub const fn activity_mut(&mut self) -> &mut Activity {
        &mut self.activity
    }
    #[must_use]
    pub fn into_activity(self) -> Activity {
        self.activity
    }

    pub fn verify_terminal(&self) -> Result<(), StandardTerminalError> {
        match self.activity.phase() {
            ActivityPhase::Terminal(actual) if actual == self.expected_outcome.terminal() => Ok(()),
            ActivityPhase::Terminal(actual) => Err(StandardTerminalError::OutcomeMismatch {
                expected: self.expected_outcome,
                actual,
            }),
            _ => Err(StandardTerminalError::NotTerminal),
        }
    }
}

fn parse_seed(value: &str) -> Option<u64> {
    if value.len() != 16 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    u64::from_str_radix(value, 16).ok()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardProfileError {
    ActivityIdentityMismatch,
    MultiplePlayerTeams,
    PartySizeMismatch,
    NonStandardProjection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardScenarioError {
    Profile(StandardProfileError),
    InvalidMasterSeed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardTerminalError {
    NotTerminal,
    OutcomeMismatch {
        expected: StandardExpectedOutcome,
        actual: TerminalOutcome,
    },
}
