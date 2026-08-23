use std::collections::{BTreeMap, BTreeSet};

use starclock_build::{id::LightConeId, spec::CombatantBuildSpec};
use starclock_combat::Scalar;

use crate::{CurrencyWarsPositionKind, CurrencyWarsRole, CurrencyWarsRoleId};

const LOADOUT_SLOT_RADIX: u64 = 32;
const EQUIPMENT_SLOT_LIMIT: u8 = 3;
const FIRST_SLOTLESS_INDEX: u8 = EQUIPMENT_SLOT_LIMIT + 1;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsEquipmentId(u32);

impl CurrencyWarsEquipmentId {
    #[must_use]
    pub const fn new(raw: u32) -> Option<Self> {
        if raw == 0 { None } else { Some(Self(raw)) }
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsEquipmentCategory {
    Artifacts,
    Basic,
    Craftable,
    Crown,
    Emblem,
    FateEquip,
    GoldTrash,
    Hack,
    Material,
    Other,
    Radiant,
    Support,
    TraitSpecial,
    Trash,
}

impl CurrencyWarsEquipmentCategory {
    /// Released Hacking Components explicitly do not consume character slots.
    #[must_use]
    pub const fn occupies_slot(self) -> bool {
        !matches!(self, Self::Hack)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyWarsEquipmentDressRule {
    Any,
    AllSlotsEmpty,
    Unique,
    RoleOnly(Box<[CurrencyWarsRoleId]>),
    TraitOnly(Box<[u32]>),
    UniqueAndExclusiveTrait(Box<[u32]>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsPropertyContribution {
    pub property: Box<str>,
    pub value: Scalar,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRuntimeEquipment {
    pub id: CurrencyWarsEquipmentId,
    pub category: CurrencyWarsEquipmentCategory,
    pub tags: Box<[u32]>,
    pub dress_rule: CurrencyWarsEquipmentDressRule,
    pub properties: Box<[CurrencyWarsPropertyContribution]>,
    pub ability_name: Option<Box<str>>,
    pub parameters: Box<[Scalar]>,
}

impl CurrencyWarsRuntimeEquipment {
    #[must_use]
    pub fn eligible_for(&self, role: &CurrencyWarsRole) -> bool {
        match &self.dress_rule {
            CurrencyWarsEquipmentDressRule::Any
            | CurrencyWarsEquipmentDressRule::AllSlotsEmpty
            | CurrencyWarsEquipmentDressRule::Unique => true,
            CurrencyWarsEquipmentDressRule::RoleOnly(roles) => roles.contains(&role.id),
            CurrencyWarsEquipmentDressRule::TraitOnly(traits)
            | CurrencyWarsEquipmentDressRule::UniqueAndExclusiveTrait(traits) => traits
                .iter()
                .any(|trait_id| role.trait_ids.contains(trait_id)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEquipmentCategoryLimit {
    pub category: CurrencyWarsEquipmentCategory,
    pub maximum: Option<u8>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsEquipmentSlot {
    role: CurrencyWarsRoleId,
    index: u8,
}

impl CurrencyWarsEquipmentSlot {
    pub fn new(role: CurrencyWarsRoleId, index: u8) -> Result<Self, CurrencyWarsEquipmentError> {
        if index == 0 || u64::from(index) >= LOADOUT_SLOT_RADIX {
            return Err(error(
                "Currency Wars equipment slot is outside the encoded range",
            ));
        }
        Ok(Self { role, index })
    }

    #[must_use]
    pub const fn role(self) -> CurrencyWarsRoleId {
        self.role
    }

    #[must_use]
    pub const fn index(self) -> u8 {
        self.index
    }

    #[must_use]
    pub const fn occupies_slot(self) -> bool {
        self.index <= EQUIPMENT_SLOT_LIMIT
    }

    #[must_use]
    pub const fn encode(self) -> u64 {
        self.role.get() as u64 * LOADOUT_SLOT_RADIX + self.index as u64
    }

    pub fn decode(raw: u64) -> Result<Self, CurrencyWarsEquipmentError> {
        let role = u32::try_from(raw / LOADOUT_SLOT_RADIX)
            .ok()
            .and_then(CurrencyWarsRoleId::new)
            .ok_or_else(|| error("Currency Wars equipment slot role is invalid"))?;
        let index = u8::try_from(raw % LOADOUT_SLOT_RADIX)
            .map_err(|_| error("Currency Wars equipment slot index is invalid"))?;
        Self::new(role, index)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CurrencyWarsEquipmentLoadout {
    slots: BTreeMap<CurrencyWarsEquipmentSlot, CurrencyWarsEquipmentId>,
}

impl CurrencyWarsEquipmentLoadout {
    pub fn decode(values: &[(u64, i64)]) -> Result<Self, CurrencyWarsEquipmentError> {
        let mut slots = BTreeMap::new();
        for &(raw_slot, raw_equipment) in values {
            let slot = CurrencyWarsEquipmentSlot::decode(raw_slot)?;
            let equipment = u32::try_from(raw_equipment)
                .ok()
                .and_then(CurrencyWarsEquipmentId::new)
                .ok_or_else(|| error("Currency Wars equipped equipment ID is invalid"))?;
            if slots.insert(slot, equipment).is_some() {
                return Err(error("Currency Wars equipment slot is duplicated"));
            }
        }
        Ok(Self { slots })
    }

    #[must_use]
    pub fn slots(&self) -> &BTreeMap<CurrencyWarsEquipmentSlot, CurrencyWarsEquipmentId> {
        &self.slots
    }

    pub fn for_role(
        &self,
        role: CurrencyWarsRoleId,
    ) -> impl Iterator<Item = (CurrencyWarsEquipmentSlot, CurrencyWarsEquipmentId)> + '_ {
        self.slots
            .iter()
            .filter(move |(slot, _)| slot.role == role)
            .map(|(slot, equipment)| (*slot, *equipment))
    }

    #[must_use]
    pub fn encoded(&self) -> Box<[(u64, i64)]> {
        self.slots
            .iter()
            .map(|(slot, equipment)| (slot.encode(), i64::from(equipment.get())))
            .collect()
    }

    pub(crate) fn remove_role(&mut self, role: CurrencyWarsRoleId) {
        self.slots.retain(|slot, _| slot.role != role);
    }

    pub(crate) fn equip<'a>(
        &mut self,
        role: &CurrencyWarsRole,
        equipment: &CurrencyWarsRuntimeEquipment,
        category_limit: Option<u8>,
        implant_limit: u8,
        replace: Option<CurrencyWarsEquipmentSlot>,
        definitions: impl Fn(CurrencyWarsEquipmentId) -> Option<&'a CurrencyWarsRuntimeEquipment>,
    ) -> Result<Option<CurrencyWarsEquipmentId>, CurrencyWarsEquipmentError> {
        if !equipment.eligible_for(role) {
            return Err(error(
                "Currency Wars equipment is not eligible for the role",
            ));
        }
        if replace.is_some_and(|slot| slot.role != role.id) {
            return Err(error(
                "Currency Wars replacement slot belongs to another role",
            ));
        }
        let mut candidate = self.clone();
        let replaced = replace.and_then(|slot| candidate.slots.remove(&slot));
        if replace.is_some() && replaced.is_none() {
            return Err(error("Currency Wars replacement slot is empty"));
        }
        let current = candidate.for_role(role.id).collect::<Vec<_>>();
        if (!current.is_empty()
            && matches!(
                equipment.dress_rule,
                CurrencyWarsEquipmentDressRule::AllSlotsEmpty
            ))
            || current.iter().any(|(_, value)| {
                definitions(*value).is_some_and(|definition| {
                    matches!(
                        definition.dress_rule,
                        CurrencyWarsEquipmentDressRule::AllSlotsEmpty
                    )
                })
            })
        {
            return Err(error(
                "Currency Wars equipment requires every role slot to be empty",
            ));
        }
        if current.iter().any(|(_, value)| {
            *value == equipment.id
                && matches!(
                    equipment.dress_rule,
                    CurrencyWarsEquipmentDressRule::Unique
                        | CurrencyWarsEquipmentDressRule::UniqueAndExclusiveTrait(_)
                )
        }) {
            return Err(error("Currency Wars unique equipment is already equipped"));
        }
        let same_category = current
            .iter()
            .filter(|(_, value)| {
                definitions(*value)
                    .is_some_and(|definition| definition.category == equipment.category)
            })
            .count();
        if category_limit.is_some_and(|maximum| same_category >= usize::from(maximum)) {
            return Err(error("Currency Wars equipment category limit is exceeded"));
        }
        if !equipment.category.occupies_slot()
            && current
                .iter()
                .filter(|(slot, _)| !slot.occupies_slot())
                .count()
                >= usize::from(implant_limit)
        {
            return Err(error("Currency Wars role implant slot is occupied"));
        }
        let index = if equipment.category.occupies_slot() {
            (1..=EQUIPMENT_SLOT_LIMIT)
                .find(|index| {
                    !candidate.slots.contains_key(
                        &CurrencyWarsEquipmentSlot::new(role.id, *index)
                            .expect("normal equipment slot is valid"),
                    )
                })
                .ok_or_else(|| error("Currency Wars role already uses all three equipment slots"))?
        } else {
            (FIRST_SLOTLESS_INDEX..u8::try_from(LOADOUT_SLOT_RADIX).expect("radix fits u8"))
                .find(|index| {
                    !candidate.slots.contains_key(
                        &CurrencyWarsEquipmentSlot::new(role.id, *index)
                            .expect("slotless equipment index is valid"),
                    )
                })
                .ok_or_else(|| error("Currency Wars slotless equipment capacity is exceeded"))?
        };
        candidate.slots.insert(
            CurrencyWarsEquipmentSlot::new(role.id, index)?,
            equipment.id,
        );
        *self = candidate;
        Ok(replaced)
    }

    pub(crate) fn unequip(
        &mut self,
        slot: CurrencyWarsEquipmentSlot,
    ) -> Result<CurrencyWarsEquipmentId, CurrencyWarsEquipmentError> {
        self.slots
            .remove(&slot)
            .ok_or_else(|| error("Currency Wars equipment slot is empty"))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurrencyWarsOffFieldEligibility {
    Eidolon {
        role: CurrencyWarsRoleId,
        rank_id: u32,
        rank: u8,
    },
    SignatureLightCone {
        role: CurrencyWarsRoleId,
        light_cone: LightConeId,
        superimposition: u8,
    },
}

impl CurrencyWarsOffFieldEligibility {
    #[must_use]
    pub const fn role(&self) -> CurrencyWarsRoleId {
        match *self {
            Self::Eidolon { role, .. } | Self::SignatureLightCone { role, .. } => role,
        }
    }

    fn matches(&self, role: CurrencyWarsRoleId, build: &CombatantBuildSpec) -> bool {
        match *self {
            Self::Eidolon {
                role: expected,
                rank,
                ..
            } => expected == role && rank <= build.eidolon().get(),
            Self::SignatureLightCone {
                role: expected,
                light_cone,
                superimposition,
            } => {
                expected == role
                    && build.light_cone().is_some_and(|loadout| {
                        loadout.definition() == light_cone
                            && loadout.superimposition().get() == superimposition
                    })
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsOffFieldPayload {
    pub owner_properties: Box<[CurrencyWarsPropertyContribution]>,
    pub all_member_properties: Box<[CurrencyWarsPropertyContribution]>,
    pub modified_skills: Box<[u32]>,
    pub rank_abilities: Box<[Box<str>]>,
    pub parameters: Box<[Scalar]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsOffFieldContributionSnapshot {
    pub role: CurrencyWarsRoleId,
    pub conversion_keys: Box<[Box<str>]>,
    pub owner_properties: Box<[CurrencyWarsPropertyContribution]>,
    pub all_member_properties: Box<[CurrencyWarsPropertyContribution]>,
    pub modified_skills: Box<[u32]>,
    pub rank_abilities: Box<[Box<str>]>,
}

pub(crate) fn resolve_off_field_contributions<'a>(
    role: CurrencyWarsRoleId,
    position: CurrencyWarsPositionKind,
    build: &CombatantBuildSpec,
    conversions: impl IntoIterator<
        Item = (
            &'a str,
            &'a CurrencyWarsOffFieldEligibility,
            &'a CurrencyWarsOffFieldPayload,
        ),
    >,
) -> CurrencyWarsOffFieldContributionSnapshot {
    let mut keys = Vec::new();
    let mut owner = Vec::new();
    let mut all = Vec::new();
    let mut skills = BTreeSet::new();
    let mut abilities = BTreeSet::new();
    if position == CurrencyWarsPositionKind::Back {
        for (key, eligibility, payload) in conversions {
            if !eligibility.matches(role, build) {
                continue;
            }
            keys.push(Box::<str>::from(key));
            owner.extend_from_slice(&payload.owner_properties);
            all.extend_from_slice(&payload.all_member_properties);
            skills.extend(payload.modified_skills.iter().copied());
            abilities.extend(payload.rank_abilities.iter().cloned());
        }
    }
    CurrencyWarsOffFieldContributionSnapshot {
        role,
        conversion_keys: keys.into_boxed_slice(),
        owner_properties: owner.into_boxed_slice(),
        all_member_properties: all.into_boxed_slice(),
        modified_skills: skills.into_iter().collect(),
        rank_abilities: abilities.into_iter().collect(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEquipmentError {
    message: Box<str>,
}

impl std::fmt::Display for CurrencyWarsEquipmentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CurrencyWarsEquipmentError {}

fn error(message: &'static str) -> CurrencyWarsEquipmentError {
    CurrencyWarsEquipmentError {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_and_implant_slots_use_independent_released_caps() {
        let role = fixture_role();
        let ordinary =
            [1, 2, 3, 4].map(|id| fixture_equipment(id, CurrencyWarsEquipmentCategory::Radiant));
        let implants = [5, 6].map(|id| fixture_equipment(id, CurrencyWarsEquipmentCategory::Hack));
        let all = ordinary.iter().chain(implants.iter()).collect::<Vec<_>>();
        let definition = |id| all.iter().copied().find(|value| value.id == id);
        let mut loadout = CurrencyWarsEquipmentLoadout::default();

        for equipment in &ordinary[..3] {
            loadout
                .equip(&role, equipment, None, 1, None, definition)
                .unwrap();
        }
        assert!(
            loadout
                .equip(&role, &ordinary[3], None, 1, None, definition)
                .is_err()
        );
        loadout
            .equip(&role, &implants[0], None, 1, None, definition)
            .unwrap();
        assert!(
            loadout
                .equip(&role, &implants[1], None, 1, None, definition)
                .is_err()
        );
        assert_eq!(loadout.for_role(role.id).count(), 4);
    }

    #[test]
    fn trait_and_role_dress_rules_use_explicit_roster_relationships() {
        let role = fixture_role();
        let trait_only = CurrencyWarsRuntimeEquipment {
            dress_rule: CurrencyWarsEquipmentDressRule::TraitOnly(Box::new([1011])),
            ..fixture_equipment(1, CurrencyWarsEquipmentCategory::TraitSpecial)
        };
        let role_only = CurrencyWarsRuntimeEquipment {
            dress_rule: CurrencyWarsEquipmentDressRule::RoleOnly(Box::new([role.id])),
            ..fixture_equipment(2, CurrencyWarsEquipmentCategory::Hack)
        };

        assert!(trait_only.eligible_for(&role));
        assert!(role_only.eligible_for(&role));
    }

    fn fixture_role() -> CurrencyWarsRole {
        CurrencyWarsRole {
            id: CurrencyWarsRoleId::new(1001).unwrap(),
            stable_key: "role.1001".into(),
            avatar_id: 1001,
            rarity: 1,
            build_mapping_id: "build.role.1001".into(),
            maximum_star: 3,
            positions: Box::new([CurrencyWarsPositionKind::Back]),
            trait_ids: Box::new([1011]),
            backend_rank_ids: Box::new([100_101]),
        }
    }

    fn fixture_equipment(
        id: u32,
        category: CurrencyWarsEquipmentCategory,
    ) -> CurrencyWarsRuntimeEquipment {
        CurrencyWarsRuntimeEquipment {
            id: CurrencyWarsEquipmentId::new(id).unwrap(),
            category,
            tags: Box::new([]),
            dress_rule: CurrencyWarsEquipmentDressRule::Any,
            properties: Box::new([]),
            ability_name: None,
            parameters: Box::new([]),
        }
    }
}
