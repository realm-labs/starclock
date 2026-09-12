//! Immutable fail-closed encounter candidate closure.

use crate::divergent_universe_catalog::DivergentUniverseFlowCatalog;
use crate::divergent_universe_progression_catalog::DivergentUniverseProgressionCatalog;
use std::collections::{BTreeMap, BTreeSet};

macro_rules! stable_id {
    ($name:ident,$prefix:literal) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(Box<str>);
        impl $name {
            pub fn new(v: impl Into<Box<str>>) -> Result<Self, DivergentUniverseEncounterError> {
                let v = v.into();
                if !v.starts_with($prefix) || v.len() == $prefix.len() {
                    Err(error(concat!(stringify!($name), " namespace mismatch")))
                } else {
                    Ok(Self(v))
                }
            }
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}
stable_id!(
    DivergentUniverseEncounterSourceId,
    "divergent-universe.encounter-source."
);
stable_id!(
    DivergentUniverseEncounterGroupId,
    "divergent-universe.encounter-group."
);
stable_id!(
    DivergentUniverseEncounterWaveId,
    "divergent-universe.encounter-wave."
);
stable_id!(
    DivergentUniverseEnemySlotId,
    "divergent-universe.encounter-wave."
);
stable_id!(DivergentUniverseBossPoolId, "divergent-universe.boss-pool.");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEncounterSourceDefinition {
    pub id: DivergentUniverseEncounterSourceId,
    pub parent_id: Box<str>,
    pub parent_kind: Box<str>,
    pub encounter_groups: Box<[DivergentUniverseEncounterGroupId]>,
    pub stage_ids: Box<[Box<str>]>,
    pub map_entry_id: Box<str>,
    pub room_type: Option<Box<str>>,
    pub resolution_state: Box<str>,
    pub replacement_condition: Box<str>,
    pub blocking: bool,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEncounterMember {
    pub npc_monster_id: Box<str>,
    pub source_monster_id: Box<str>,
    pub stage_id: Box<str>,
    pub weight: Box<str>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEncounterGroupDefinition {
    pub id: DivergentUniverseEncounterGroupId,
    pub candidate_stage_ids: Box<[Box<str>]>,
    pub members: Box<[DivergentUniverseEncounterMember]>,
    pub display_binding_ids: Box<[Box<str>]>,
    pub display_roles: Box<[Box<str>]>,
    pub module_id: Box<str>,
    pub area_ids: Box<[Box<str>]>,
    pub difficulty_ids: Box<[Box<str>]>,
    pub selection_policy: Box<str>,
    pub reachability_disposition: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEncounterWaveDefinition {
    pub id: DivergentUniverseEncounterWaveId,
    pub stage_id: Box<str>,
    pub wave_index: u16,
    pub trigger: Box<str>,
    pub enemy_slots: Box<[DivergentUniverseEnemySlotId]>,
    pub hard_level_group: Box<str>,
    pub level: Box<str>,
    pub stage_ability_refs: Box<[Box<str>]>,
    pub reachability_disposition: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEnemySlotDefinition {
    pub id: DivergentUniverseEnemySlotId,
    pub wave: DivergentUniverseEncounterWaveId,
    pub slot_index: u16,
    pub source_slot: Box<str>,
    pub source_monster_id: Box<str>,
    pub monster_id: Box<str>,
    pub enemy_id: Box<str>,
    pub level: Box<str>,
    pub ability_refs: Box<[Box<str>]>,
    pub reachability_disposition: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBossPoolDefinition {
    pub id: DivergentUniverseBossPoolId,
    pub encounter_group: DivergentUniverseEncounterGroupId,
    pub weekly_modifier_id: Box<str>,
    pub source_group_id: Box<str>,
    pub display_slot: Box<str>,
    pub display_variant: Box<str>,
    pub candidate_monster_ids: Box<[Box<str>]>,
    pub candidate_stage_ids: Box<[Box<str>]>,
    pub module_id: Box<str>,
    pub area_id: Box<str>,
    pub difficulty_ids: Box<[Box<str>]>,
    pub selection_policy: Box<str>,
    pub fallback: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEncounterCatalogParts {
    pub sources: Vec<DivergentUniverseEncounterSourceDefinition>,
    pub groups: Vec<DivergentUniverseEncounterGroupDefinition>,
    pub waves: Vec<DivergentUniverseEncounterWaveDefinition>,
    pub slots: Vec<DivergentUniverseEnemySlotDefinition>,
    pub boss_pools: Vec<DivergentUniverseBossPoolDefinition>,
    pub source_obligations: usize,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEncounterCatalog {
    parts: DivergentUniverseEncounterCatalogParts,
}
impl DivergentUniverseEncounterCatalog {
    pub fn new(
        mut p: DivergentUniverseEncounterCatalogParts,
        flow: &DivergentUniverseFlowCatalog,
        progression: &DivergentUniverseProgressionCatalog,
    ) -> Result<Self, DivergentUniverseEncounterError> {
        if p.sources.len() != 877
            || p.groups.len() != 43
            || p.waves.len() != 176
            || p.slots.len() != 385
            || p.boss_pools.len() != 618
        {
            return Err(error("encounter candidate denominator drift"));
        }
        if p.source_obligations != 877 {
            return Err(error("encounter source obligation closure drift"));
        }
        p.sources.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.groups.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.waves.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.slots.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.boss_pools.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        validate(&p, flow, progression)?;
        Ok(Self { parts: p })
    }
    #[must_use]
    pub fn sources(&self) -> &[DivergentUniverseEncounterSourceDefinition] {
        &self.parts.sources
    }
    #[must_use]
    pub fn groups(&self) -> &[DivergentUniverseEncounterGroupDefinition] {
        &self.parts.groups
    }
    #[must_use]
    pub fn waves(&self) -> &[DivergentUniverseEncounterWaveDefinition] {
        &self.parts.waves
    }
    #[must_use]
    pub fn slots(&self) -> &[DivergentUniverseEnemySlotDefinition] {
        &self.parts.slots
    }
    #[must_use]
    pub fn boss_pools(&self) -> &[DivergentUniverseBossPoolDefinition] {
        &self.parts.boss_pools
    }
    #[must_use]
    pub const fn source_obligations(&self) -> usize {
        self.parts.source_obligations
    }
    #[cfg(test)]
    pub(crate) fn into_parts(self) -> DivergentUniverseEncounterCatalogParts {
        self.parts
    }
}
fn validate(
    p: &DivergentUniverseEncounterCatalogParts,
    flow: &DivergentUniverseFlowCatalog,
    progression: &DivergentUniverseProgressionCatalog,
) -> Result<(), DivergentUniverseEncounterError> {
    let groups = p
        .groups
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    let waves = p
        .waves
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    let slots = p
        .slots
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    if groups.len() != 43 || waves.len() != 176 || slots.len() != 385 {
        return Err(error("duplicate encounter candidate identity"));
    }
    let areas = flow
        .areas()
        .iter()
        .map(|x| x.id.as_str())
        .collect::<BTreeSet<_>>();
    let rooms = flow
        .room_candidates()
        .iter()
        .map(|x| x.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut parent_counts = BTreeMap::new();
    for source in &p.sources {
        *parent_counts
            .entry(source.parent_kind.as_ref())
            .or_insert(0usize) += 1;
        if source.runtime_lowered || source.blocking {
            return Err(error("encounter source fail-closed boundary drift"));
        }
        match source.parent_kind.as_ref() {
            "AreaEntry" if areas.contains(source.parent_id.as_ref()) => {}
            "RoomCandidate" if rooms.contains(source.parent_id.as_ref()) => {}
            "SharedStageConfigRoot" => {}
            _ => return Err(error("encounter source parent closure drift")),
        }
    }
    if parent_counts
        != BTreeMap::from([
            ("AreaEntry", 28),
            ("RoomCandidate", 848),
            ("SharedStageConfigRoot", 1),
        ])
    {
        return Err(error("encounter parent split drift"));
    }
    let stage_ids = p
        .waves
        .iter()
        .map(|x| x.stage_id.as_ref())
        .collect::<BTreeSet<_>>();
    if stage_ids.len() != 118
        || p.groups.iter().any(|x| {
            x.runtime_lowered
                || !x.module_id.is_empty()
                || !x.area_ids.is_empty()
                || !x.difficulty_ids.is_empty()
                || x.members.is_empty()
                || x.selection_policy.as_ref() != "DisplayOnlyNoEnabledWeeklySelector"
                || x.reachability_disposition.as_ref() != "UnprovenWeeklyDisplayCandidate"
                || x.candidate_stage_ids.len() != x.members.len()
                || x.members
                    .iter()
                    .any(|m| m.weight.is_empty() || !stage_ids.contains(m.stage_id.as_ref()))
        })
    {
        return Err(error("display encounter group boundary drift"));
    }
    if p.waves.iter().any(|x| {
        x.runtime_lowered
            || x.wave_index < 1
            || x.trigger.as_ref()
                != if x.wave_index == 1 {
                    "BattleStart"
                } else {
                    "PreviousWaveDefeated"
                }
            || x.enemy_slots.is_empty()
            || x.enemy_slots
                .iter()
                .any(|id| slots.get(id).is_none_or(|slot| slot.wave != x.id))
            || x.reachability_disposition.as_ref() != "UnprovenWeeklyDisplayCandidate"
    }) {
        return Err(error("encounter wave/slot closure drift"));
    }
    let referenced_slots = p
        .waves
        .iter()
        .flat_map(|x| x.enemy_slots.iter())
        .collect::<BTreeSet<_>>();
    if referenced_slots.len() != 385
        || p.slots.iter().any(|x| {
            x.runtime_lowered
                || !waves.contains_key(&x.wave)
                || !x.monster_id.starts_with("enemy.")
                || !x.enemy_id.starts_with("enemy.")
                || x.reachability_disposition.as_ref() != "UnprovenWeeklyDisplayCandidate"
        })
    {
        return Err(error("enemy slot exact-once closure drift"));
    }
    let weekly = progression
        .weekly_modifiers()
        .iter()
        .map(|x| x.id.as_str())
        .collect::<BTreeSet<_>>();
    if p.boss_pools.iter().any(|x| {
        x.runtime_lowered
            || !groups.contains_key(&x.encounter_group)
            || !weekly.contains(x.weekly_modifier_id.as_ref())
            || !x.module_id.is_empty()
            || !x.area_id.is_empty()
            || !x.difficulty_ids.is_empty()
            || x.candidate_monster_ids.is_empty()
            || x.candidate_monster_ids.len() != x.candidate_stage_ids.len()
            || x.selection_policy.as_ref() != "DisplayOnlyNoEnabledWeeklySelector"
            || x.fallback.as_ref() != "FailClosedWithoutCurrentWeeklySelector"
    }) {
        return Err(error("display boss-pool boundary drift"));
    }
    Ok(())
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEncounterError {
    message: Box<str>,
}
impl std::fmt::Display for DivergentUniverseEncounterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for DivergentUniverseEncounterError {}
fn error(message: &str) -> DivergentUniverseEncounterError {
    DivergentUniverseEncounterError {
        message: message.into(),
    }
}
