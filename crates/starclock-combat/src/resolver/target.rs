//! Transactional target revalidation and journaled repeated-hit draws.

use crate::{
    battle::fault::BattleFault,
    catalog::action::TargetRelation,
    rng::types::DrawPurpose,
    target::{model::TargetCommitment, select},
};

use super::transaction::{Transaction, action_fault};

impl Transaction<'_> {
    pub(super) fn resolve_rule_selector(
        &mut self,
        selector: &crate::catalog::selector::RuleUnitSelector,
        owner: crate::UnitId,
        actor: crate::UnitId,
        applier: Option<crate::UnitId>,
        primary: Option<crate::UnitId>,
        current_subject: Option<crate::UnitId>,
    ) -> Result<Box<[crate::UnitId]>, BattleFault> {
        use crate::catalog::selector::{
            RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice, RuleSelectorOrdering,
            RuleSelectorOrigin, RuleSelectorSide,
        };
        let owner_side = self
            .state
            .units
            .get(owner)
            .ok_or_else(|| action_fault(120))?
            .side;
        let anchored = match selector.origin() {
            RuleSelectorOrigin::Owner | RuleSelectorOrigin::Source => Some(owner),
            RuleSelectorOrigin::Actor => Some(actor),
            RuleSelectorOrigin::Applier => applier,
            RuleSelectorOrigin::PrimaryTarget => primary,
            RuleSelectorOrigin::CurrentSubject => current_subject.or(primary),
            RuleSelectorOrigin::Team | RuleSelectorOrigin::Encounter => None,
        };
        let on_selected_side = |side| match selector.side() {
            RuleSelectorSide::Same => side == owner_side,
            RuleSelectorSide::Opposing => side != owner_side,
            RuleSelectorSide::Any => true,
        };
        let direct = anchored.filter(|unit| {
            self.state
                .units
                .get(*unit)
                .is_some_and(|state| on_selected_side(state.side))
        });
        let use_direct = direct.is_some()
            && matches!(
                selector.origin(),
                RuleSelectorOrigin::PrimaryTarget | RuleSelectorOrigin::CurrentSubject
            )
            || direct.is_some()
                && selector.side() == RuleSelectorSide::Same
                && selector.choice() == RuleSelectorChoice::First;
        let mut pool = if use_direct {
            direct.into_iter().collect::<Vec<_>>()
        } else {
            self.state
                .units
                .iter_by_id()
                .filter(|unit| on_selected_side(unit.side))
                .map(|unit| unit.id)
                .collect::<Vec<_>>()
        };
        pool.retain(|id| {
            self.state.units.get(*id).is_some_and(|unit| {
                let life = match selector.life() {
                    RuleLifePredicate::Any => true,
                    RuleLifePredicate::Alive => unit.life == crate::LifeState::Alive,
                    RuleLifePredicate::Downed => unit.life == crate::LifeState::Downed,
                    RuleLifePredicate::Defeated => unit.life == crate::LifeState::Defeated,
                };
                let presence = match selector.presence() {
                    RulePresencePredicate::Any => true,
                    RulePresencePredicate::Present => {
                        unit.presence == crate::PresenceState::Present
                    }
                    RulePresencePredicate::Reserved => {
                        unit.presence == crate::PresenceState::Reserved
                    }
                    RulePresencePredicate::Departed => {
                        unit.presence == crate::PresenceState::Departed
                    }
                    RulePresencePredicate::Untargetable => {
                        unit.presence == crate::PresenceState::Untargetable
                    }
                    RulePresencePredicate::Linked => unit.presence == crate::PresenceState::Linked,
                    RulePresencePredicate::Transformed => {
                        unit.presence == crate::PresenceState::Transformed
                    }
                };
                life && presence
            })
        });
        match selector.ordering() {
            RuleSelectorOrdering::Formation => pool.sort_unstable_by_key(|id| {
                self.state
                    .units
                    .get(*id)
                    .map(|unit| (unit.side as u8, unit.formation.get(), id.get()))
            }),
            RuleSelectorOrdering::Timeline => pool.sort_unstable_by_key(|id| {
                self.state
                    .actors
                    .id_for_owner(*id)
                    .and_then(|actor| self.state.actors.get(actor))
                    .map(|state| (state.gauge.scaled(), id.get()))
            }),
            RuleSelectorOrdering::HpRatioAscending | RuleSelectorOrdering::HpRatioDescending => {
                pool.sort_unstable_by(|left, right| {
                    let left = self
                        .state
                        .units
                        .get(*left)
                        .expect("selector candidate exists");
                    let right = self
                        .state
                        .units
                        .get(*right)
                        .expect("selector candidate exists");
                    let ordering = (i128::from(left.current_hp.get())
                        * i128::from(right.maximum_hp.get()))
                    .cmp(&(i128::from(right.current_hp.get()) * i128::from(left.maximum_hp.get())))
                    .then_with(|| left.id.cmp(&right.id));
                    if selector.ordering() == RuleSelectorOrdering::HpRatioDescending {
                        ordering.reverse()
                    } else {
                        ordering
                    }
                });
            }
            RuleSelectorOrdering::StableId
            | RuleSelectorOrdering::EventOrder
            | RuleSelectorOrdering::StatAscending
            | RuleSelectorOrdering::StatDescending => pool.sort_unstable(),
        }
        let maximum = usize::from(selector.maximum());
        let mut selected = match selector.choice() {
            RuleSelectorChoice::All => {
                pool.truncate(maximum);
                pool
            }
            RuleSelectorChoice::First => pool.into_iter().take(1).collect(),
            RuleSelectorChoice::PrimaryPlusAdjacent => {
                let Some(primary) = primary.filter(|value| pool.contains(value)) else {
                    return self.finish_rule_selector(selector, Vec::new());
                };
                let index = self
                    .state
                    .units
                    .get(primary)
                    .expect("candidate exists")
                    .formation
                    .get();
                pool.into_iter()
                    .filter(|id| {
                        self.state
                            .units
                            .get(*id)
                            .is_some_and(|unit| unit.formation.get().abs_diff(index) <= 1)
                    })
                    .take(maximum)
                    .collect()
            }
            RuleSelectorChoice::RngUniform => {
                if pool.is_empty() {
                    Vec::new()
                } else {
                    let purpose = selector
                        .rng_purpose()
                        .and_then(rule_draw_purpose)
                        .ok_or_else(|| action_fault(121))?;
                    let before = self.state.rng.draw_count();
                    let selection = self
                        .state
                        .rng
                        .choose_index(
                            purpose,
                            u32::try_from(pool.len()).map_err(|_| action_fault(122))?,
                        )
                        .map_err(|_| action_fault(123))?
                        .ok_or_else(|| action_fault(124))?;
                    for index in before..self.state.rng.draw_count() {
                        self.journal.rng_draw(index, purpose.code());
                    }
                    vec![pool[usize::try_from(selection.value()).map_err(|_| action_fault(125))?]]
                }
            }
            RuleSelectorChoice::RngWeighted => return Err(action_fault(126)),
        };
        if selector.repeated() && selected.len() == 1 {
            selected.resize(maximum.max(1), selected[0]);
        }
        self.finish_rule_selector(selector, selected)
    }

    fn finish_rule_selector(
        &self,
        selector: &crate::catalog::selector::RuleUnitSelector,
        selected: Vec<crate::UnitId>,
    ) -> Result<Box<[crate::UnitId]>, BattleFault> {
        if selected.len() < usize::from(selector.minimum()) {
            match selector.empty_pool() {
                crate::catalog::selector::RuleEmptyPoolPolicy::Fault => Err(action_fault(127)),
                _ => Ok(Box::new([])),
            }
        } else {
            Ok(selected.into_boxed_slice())
        }
    }
    pub(super) fn resolve_hit_targets(
        &mut self,
        actor: crate::UnitId,
        commitment: &mut TargetCommitment,
    ) -> Result<Box<[crate::UnitId]>, BattleFault> {
        let rng = &mut self.state.rng;
        let journal = &mut self.journal;
        select::resolve_for_hit(
            &self.state.units,
            &self.state.formations,
            actor,
            commitment,
            |count| {
                let before = rng.draw_count();
                let selected = rng
                    .choose_index(DrawPurpose::BOUNCE_TARGET, count)
                    .map_err(|_| select::TargetError::ChoiceFailed)?
                    .ok_or(select::TargetError::ChoiceFailed)?;
                for index in before..rng.draw_count() {
                    journal.rng_draw(index, DrawPurpose::BOUNCE_TARGET.code());
                }
                usize::try_from(selected.value()).map_err(|_| select::TargetError::ChoiceFailed)
            },
        )
        .map_err(|_| action_fault(32))
    }

    pub(super) fn draw_bounce_target(
        &mut self,
        actor: crate::UnitId,
        relation: TargetRelation,
    ) -> Result<crate::UnitId, BattleFault> {
        let side = self
            .state
            .units
            .get(actor)
            .ok_or_else(|| action_fault(33))?
            .side;
        let pool = select::stable_pool(&self.state.units, &self.state.formations, side, relation);
        let count = u32::try_from(pool.len()).map_err(|_| action_fault(34))?;
        if count == 0 {
            return Err(action_fault(35));
        }
        let before = self.state.rng.draw_count();
        let selected = self
            .state
            .rng
            .choose_index(DrawPurpose::BOUNCE_TARGET, count)
            .map_err(|_| action_fault(36))?
            .ok_or_else(|| action_fault(37))?;
        for index in before..self.state.rng.draw_count() {
            self.journal
                .rng_draw(index, DrawPurpose::BOUNCE_TARGET.code());
        }
        let index = usize::try_from(selected.value()).map_err(|_| action_fault(38))?;
        pool.get(index).copied().ok_or_else(|| action_fault(39))
    }
}

fn rule_draw_purpose(key: &str) -> Option<DrawPurpose> {
    match key {
        "bounce-target" => Some(DrawPurpose::BOUNCE_TARGET),
        "aggro-target" => Some(DrawPurpose::AGGRO_TARGET),
        "behavior-choice" => Some(DrawPurpose::BEHAVIOR_CHOICE),
        _ => None,
    }
}
