use super::SwarmDisasterEntry;

impl SwarmDisasterEntry {
    #[must_use]
    pub fn new(
        area: impl Into<Box<str>>,
        path: impl Into<Box<str>>,
        audience_die: impl Into<Box<str>>,
        participants: starclock_activity::ParticipantLock,
    ) -> Self {
        Self {
            area: area.into(),
            path: path.into(),
            audience_die: audience_die.into(),
            participants,
            audience_unlocks: Box::new([]),
            dice_control_unlocks: Box::new([]),
            communing_points: Box::new([]),
            unlocked_progression: Box::new([]),
            trailblaze_bonus: None,
        }
    }

    /// Supplies the account's authored Audience Path unlock IDs.
    ///
    /// Unknown or duplicate IDs fail closed, and a selected locked Path must
    /// be present. Destruction is the sole released always-available Path.
    #[must_use]
    pub fn with_audience_unlocks(mut self, unlocks: Vec<String>) -> Self {
        self.audience_unlocks = unlocks
            .into_iter()
            .map(String::into_boxed_str)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        self
    }

    /// Supplies authored unlock IDs for optional Audience Die controls.
    ///
    /// The released catalog currently defines only the `1000022` abandon
    /// unlock. Unknown or duplicate control unlocks fail closed.
    #[must_use]
    pub fn with_dice_control_unlocks(mut self, unlocks: Vec<String>) -> Self {
        self.dice_control_unlocks = unlocks
            .into_iter()
            .map(String::into_boxed_str)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        self
    }

    #[must_use]
    pub fn with_progression(
        mut self,
        communing_points: Vec<(String, u16)>,
        unlocked_progression: Vec<String>,
        trailblaze_bonus: Option<String>,
    ) -> Self {
        self.communing_points = communing_points
            .into_iter()
            .map(|(key, value)| (key.into_boxed_str(), value))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        self.unlocked_progression = unlocked_progression
            .into_iter()
            .map(String::into_boxed_str)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        self.trailblaze_bonus = trailblaze_bonus.map(String::into_boxed_str);
        self
    }
}
