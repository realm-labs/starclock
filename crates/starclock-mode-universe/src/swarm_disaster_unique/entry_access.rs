use super::SwarmDisasterUniqueCatalog;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SwarmDisasterEntrySelection {
    pub(crate) path_id: u32,
    pub(crate) audience_die_id: u32,
}

impl SwarmDisasterUniqueCatalog {
    pub(crate) fn entry_selection(
        &self,
        path_key: &str,
        die_key: &str,
    ) -> Option<SwarmDisasterEntrySelection> {
        let path = self
            .paths
            .iter()
            .find(|row| row.shared_path.as_ref() == path_key)?;
        let die = self
            .audience_dice
            .iter()
            .find(|row| row.id == path.audience_die && row.key.as_ref() == die_key)?;
        Some(SwarmDisasterEntrySelection {
            path_id: path.id.0,
            audience_die_id: die.id.0,
        })
    }

    pub(crate) fn communing_dimension(&self, key: &str) -> Option<(u32, u16)> {
        self.communing_dimensions
            .iter()
            .find(|row| row.key.as_ref() == key)
            .map(|row| (row.id.0, row.maximum))
    }

    pub(crate) fn progression_key(&self, key: &str) -> Option<u64> {
        self.trail_nodes
            .iter()
            .find(|row| row.key.as_ref() == key)
            .map(|row| 0x1000_0000_u64 + u64::from(row.id.0))
            .or_else(|| {
                self.cabinets
                    .iter()
                    .find(|row| row.key.as_ref() == key)
                    .map(|row| 0x2000_0000_u64 + u64::from(row.id.0))
            })
            .or_else(|| {
                self.interplays
                    .iter()
                    .find(|row| row.key.as_ref() == key)
                    .map(|row| 0x3000_0000_u64 + u64::from(row.id.0))
            })
    }

    pub(crate) fn trailblaze_bonus_id(&self, key: &str) -> Option<u32> {
        self.bonuses
            .iter()
            .find(|row| row.key.as_ref() == key)
            .map(|row| row.id.0)
    }

    pub(crate) fn initial_countdown(&self) -> Option<i64> {
        self.countdown.first()?.initial.parse().ok()
    }
}
