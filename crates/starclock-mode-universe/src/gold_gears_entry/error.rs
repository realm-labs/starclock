/// Typed construction failures before a Gold Activity instance can start.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoldAndGearsEntryError {
    InvalidCatalog,
    ParticipantPolicyMismatch,
    UnknownArea(Box<str>),
    GuideArea(Box<str>),
    UnknownPath(Box<str>),
    UnknownDice(Box<str>),
    LockedDice(Box<str>),
    DuplicateUnlockedDice(Box<str>),
    UnknownDiceFace(Box<str>),
    InvalidDiceFaceCount,
    DuplicateDiceFace(Box<str>),
    LockedDiceFace(Box<str>),
    DiceFaceSlotMismatch(Box<str>),
    DiceFaceDiceMismatch(Box<str>),
    DiceFaceRarityMismatch(Box<str>),
    InvalidDiceLoadoutRuntime,
    UnknownNeuralNode(Box<str>),
    DuplicateNeuralNode(Box<str>),
    MissingNeuralPrerequisite {
        node: Box<str>,
        prerequisite: Box<str>,
    },
    UnknownCompletedArea(Box<str>),
    DuplicateCompletedArea(Box<str>),
    InvalidConundrumLevel,
    ConundrumDifficultyMismatch,
    MissingConundrumPrerequisite,
    UnknownTrailblazeBonus(Box<str>),
    MissingCognitionRange,
    InvalidActivityState,
    InvalidPlaneCount,
    MissingPlane(Box<str>),
    MissingChessboard(Box<str>),
    InvalidTopology,
    InvalidMapRuntime,
    UnknownChessboard(Box<str>),
    UnknownDomain(Box<str>),
    UnknownBeacon(Box<str>),
    MissingMapEvent,
    MapCapacityExceeded,
    InvalidCognitionRuntime,
    InvalidCognitionDelta,
    InvalidCognitionState,
    InvalidPlaneLayer,
    InvalidPlaneTransition,
    UnknownBossChoice(Box<str>),
}

impl core::fmt::Display for GoldAndGearsEntryError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "Gold and Gears entry rejected: {self:?}")
    }
}

impl std::error::Error for GoldAndGearsEntryError {}
