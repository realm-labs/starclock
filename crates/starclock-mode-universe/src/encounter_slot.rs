//! Spatial-free encounter-slot sequencing shared by authored domain profiles.

use crate::id::EncounterMemberId;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum EncounterSlotRequirement {
    Mandatory = 0,
    Optional = 1,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum EncounterSlotPolicy {
    Sequential = 0,
    OneOf = 1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncounterSlotGroup {
    id: u32,
    requirement: EncounterSlotRequirement,
    policy: EncounterSlotPolicy,
    members: Box<[EncounterMemberId]>,
}

impl EncounterSlotGroup {
    pub fn new(
        id: u32,
        requirement: EncounterSlotRequirement,
        policy: EncounterSlotPolicy,
        mut members: Vec<EncounterMemberId>,
    ) -> Result<Self, EncounterSlotPlanError> {
        members.sort_unstable();
        if id == 0 || members.is_empty() || members.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(EncounterSlotPlanError::InvalidGroup);
        }
        Ok(Self {
            id,
            requirement,
            policy,
            members: members.into_boxed_slice(),
        })
    }
    #[must_use]
    pub const fn id(&self) -> u32 {
        self.id
    }
    #[must_use]
    pub const fn requirement(&self) -> EncounterSlotRequirement {
        self.requirement
    }
    #[must_use]
    pub const fn policy(&self) -> EncounterSlotPolicy {
        self.policy
    }
    #[must_use]
    pub fn members(&self) -> &[EncounterMemberId] {
        &self.members
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncounterSlotPlan {
    groups: Box<[EncounterSlotGroup]>,
}

impl EncounterSlotPlan {
    pub fn new(mut groups: Vec<EncounterSlotGroup>) -> Result<Self, EncounterSlotPlanError> {
        groups.sort_by_key(EncounterSlotGroup::id);
        if groups.is_empty() || groups.windows(2).any(|pair| pair[0].id == pair[1].id) {
            return Err(EncounterSlotPlanError::InvalidPlan);
        }
        Ok(Self {
            groups: groups.into_boxed_slice(),
        })
    }
    #[must_use]
    pub fn groups(&self) -> &[EncounterSlotGroup] {
        &self.groups
    }
    #[must_use]
    pub fn begin(&self) -> EncounterSlotProgress<'_> {
        EncounterSlotProgress {
            plan: self,
            group_index: 0,
            sequence_index: 0,
            mandatory_complete: 0,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncounterSlotProgress<'a> {
    plan: &'a EncounterSlotPlan,
    group_index: usize,
    sequence_index: usize,
    mandatory_complete: usize,
}

impl EncounterSlotProgress<'_> {
    #[must_use]
    pub fn available(&self) -> &[EncounterMemberId] {
        let Some(group) = self.plan.groups.get(self.group_index) else {
            return &[];
        };
        match group.policy {
            EncounterSlotPolicy::Sequential => {
                &group.members[self.sequence_index..=self.sequence_index]
            }
            EncounterSlotPolicy::OneOf => &group.members,
        }
    }

    pub fn resolve(&mut self, member: EncounterMemberId) -> Result<(), EncounterSlotPlanError> {
        if !self.available().contains(&member) {
            return Err(EncounterSlotPlanError::MemberNotAvailable);
        }
        let group = &self.plan.groups[self.group_index];
        let complete = group.policy == EncounterSlotPolicy::OneOf
            || self.sequence_index + 1 == group.members.len();
        if complete {
            if group.requirement == EncounterSlotRequirement::Mandatory {
                self.mandatory_complete += 1;
            }
            self.group_index += 1;
            self.sequence_index = 0;
        } else {
            self.sequence_index += 1;
        }
        Ok(())
    }

    pub fn skip_optional(&mut self) -> Result<(), EncounterSlotPlanError> {
        let group = self
            .plan
            .groups
            .get(self.group_index)
            .ok_or(EncounterSlotPlanError::PlanComplete)?;
        if group.requirement != EncounterSlotRequirement::Optional || self.sequence_index != 0 {
            return Err(EncounterSlotPlanError::GroupNotSkippable);
        }
        self.group_index += 1;
        Ok(())
    }

    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.group_index == self.plan.groups.len()
    }

    #[must_use]
    pub fn can_exit(&self) -> bool {
        let required = self
            .plan
            .groups
            .iter()
            .filter(|group| group.requirement == EncounterSlotRequirement::Mandatory)
            .count();
        self.mandatory_complete == required
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncounterSlotPlanError {
    InvalidGroup,
    InvalidPlan,
    MemberNotAvailable,
    GroupNotSkippable,
    PlanComplete,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(raw: u32) -> EncounterMemberId {
        EncounterMemberId::new(raw).expect("test member")
    }

    #[test]
    fn mandatory_sequential_optional_and_one_of_are_explicit() {
        let plan = EncounterSlotPlan::new(vec![
            EncounterSlotGroup::new(
                1,
                EncounterSlotRequirement::Optional,
                EncounterSlotPolicy::OneOf,
                vec![member(1), member(2)],
            )
            .unwrap(),
            EncounterSlotGroup::new(
                2,
                EncounterSlotRequirement::Mandatory,
                EncounterSlotPolicy::Sequential,
                vec![member(3), member(4)],
            )
            .unwrap(),
        ])
        .unwrap();
        let mut progress = plan.begin();
        assert_eq!(progress.available(), [member(1), member(2)]);
        progress.skip_optional().unwrap();
        assert_eq!(progress.available(), [member(3)]);
        assert!(!progress.can_exit());
        progress.resolve(member(3)).unwrap();
        assert_eq!(progress.available(), [member(4)]);
        progress.resolve(member(4)).unwrap();
        assert!(progress.is_complete());
        assert!(progress.can_exit());
    }
}
