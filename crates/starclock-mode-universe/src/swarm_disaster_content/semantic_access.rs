//! Immutable semantic-fixture inputs for production runtime binding.

use super::{ReviewFixtureQuality, SwarmDisasterContentCatalog};

#[derive(Clone, Debug)]
pub(crate) struct SemanticFixtureRuntimeInput {
    pub(crate) key: Box<str>,
    pub(crate) family: Box<str>,
    pub(crate) source_record_keys: Box<[Box<str>]>,
    pub(crate) preconditions: Box<str>,
    pub(crate) input: Box<str>,
    pub(crate) ordered_operations: Box<str>,
    pub(crate) expected_facts: Box<str>,
    pub(crate) quality: ReviewFixtureQuality,
}

impl SwarmDisasterContentCatalog {
    pub(crate) fn semantic_fixture_runtime_inputs(&self) -> Box<[SemanticFixtureRuntimeInput]> {
        self.review_fixtures
            .iter()
            .map(|row| SemanticFixtureRuntimeInput {
                key: row.key.clone(),
                family: row.family.clone(),
                source_record_keys: row.source_record_keys.clone(),
                preconditions: row.preconditions.clone(),
                input: row.input.clone(),
                ordered_operations: row.ordered_operations.clone(),
                expected_facts: row.expected_facts.clone(),
                quality: row.quality,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    pub(crate) fn encounter_fixture_shape(&self) -> Option<(usize, usize, usize, usize)> {
        let group = self
            .encounter_groups
            .iter()
            .filter(|row| row.key.as_ref() == "swarm-disaster.encounter-group.120001")
            .count();
        let wave_key = "swarm-disaster.encounter-wave.120001.1200011.1";
        let wave = self
            .encounter_waves
            .iter()
            .filter(|row| row.key.as_ref() == wave_key)
            .count();
        let slots = self
            .enemy_slots
            .iter()
            .filter(|row| row.wave_key.as_ref() == wave_key)
            .filter(|row| {
                matches!(
                    row.key.as_ref(),
                    "swarm-disaster.encounter-wave.120001.1200011.1.slot.1"
                        | "swarm-disaster.encounter-wave.120001.1200011.1.slot.2"
                        | "swarm-disaster.encounter-wave.120001.1200011.1.slot.3"
                )
            })
            .count();
        let pools = self
            .boss_pools
            .iter()
            .filter(|row| {
                row.key.as_ref()
                    == "swarm-disaster.boss-pool.Difficulty_1.first-plane-boss-alternative"
            })
            .count();
        (group, wave, slots, pools)
            .eq(&(1, 1, 3, 1))
            .then_some((group, wave, slots, pools))
    }
}
