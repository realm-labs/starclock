use super::SwarmDisasterUniqueCatalog;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmCountdownRuntimeInput {
    pub(crate) initial: Box<str>,
    pub(crate) warning: Box<str>,
    pub(crate) movement_delta: Box<str>,
    pub(crate) tiers: Box<str>,
    pub(crate) source_constants: Box<str>,
    pub(crate) boss_decay: Box<[SwarmBossDecayRuntimeInput]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmBossDecayRuntimeInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) threshold: Box<str>,
    pub(crate) effect_program: Box<str>,
    pub(crate) enabled: bool,
}

impl SwarmDisasterUniqueCatalog {
    pub(crate) fn countdown_runtime_input(&self) -> Option<SwarmCountdownRuntimeInput> {
        let countdown = self.countdown.first()?;
        Some(SwarmCountdownRuntimeInput {
            initial: countdown.initial.clone(),
            warning: countdown.warning.clone(),
            movement_delta: countdown.movement_delta.clone(),
            tiers: countdown.tiers.clone(),
            source_constants: countdown.source_constants.clone(),
            boss_decay: self
                .boss_decay_levels
                .iter()
                .map(|row| SwarmBossDecayRuntimeInput {
                    id: row.id.0,
                    key: row.key.clone(),
                    threshold: row.threshold.clone(),
                    effect_program: row.effect_program.clone(),
                    enabled: row.enabled,
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        })
    }
}
