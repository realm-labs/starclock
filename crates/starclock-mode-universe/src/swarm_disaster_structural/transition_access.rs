use crate::definition::RecommendedElement;

use super::SwarmDisasterStructuralCatalog;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmBossChoiceRuntimeInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) source_id: Box<str>,
    pub(crate) display_level: u16,
    pub(crate) enemy_variant_id: Box<str>,
    pub(crate) weakness_elements: Box<[RecommendedElement]>,
}

impl SwarmDisasterStructuralCatalog {
    pub(crate) fn boss_choice_runtime_input(&self) -> Box<[SwarmBossChoiceRuntimeInput]> {
        self.boss_choices
            .iter()
            .map(|row| SwarmBossChoiceRuntimeInput {
                id: row.id.0,
                key: row.stable_key.clone(),
                source_id: row.source_id.clone(),
                display_level: row.display_level,
                enemy_variant_id: row.enemy_variant_id.clone(),
                weakness_elements: row.weakness_elements.clone(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}
