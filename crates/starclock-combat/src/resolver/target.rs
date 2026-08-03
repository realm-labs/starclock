//! Transactional target revalidation and journaled repeated-hit draws.

use crate::{
    ActionGauge, EffectDefinitionId, FormationIndex, Hp, LifeState, PresenceState, SelectorId,
    SourceDefinitionId, TeamSide, UnitId,
    battle::{fault::BattleFault, state::BattleState},
    catalog::{
        CombatCatalog,
        action::TargetRelation,
        definition::SelectorDefinition,
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
    formula::model::CombatElement,
    modifier::{
        model::{FormulaPurpose, StatQuerySubject},
        resolve::StatResolver,
    },
    rng::types::DrawPurpose,
    rule::{
        evaluate::{compare, compare_values, evaluate_value},
        model::{RuleEvaluationInput, RuleValue},
    },
    target::{model::TargetCommitment, select},
};

use super::selector_snapshot;
use super::transaction::{Transaction, action_fault};

pub(super) enum RuleSelectorResolution {
    Selected(Box<[UnitId]>),
    Skip,
    CancelRemaining,
}

pub(super) fn ordered_rule_selectors(
    catalog: &CombatCatalog,
    requested: &[SelectorId],
) -> Result<Vec<SelectorId>, BattleFault> {
    fn visit(
        catalog: &CombatCatalog,
        id: SelectorId,
        visiting: &mut std::collections::BTreeSet<SelectorId>,
        visited: &mut std::collections::BTreeSet<SelectorId>,
        output: &mut Vec<SelectorId>,
    ) -> Result<(), BattleFault> {
        if visited.contains(&id) {
            return Ok(());
        }
        if !visiting.insert(id) {
            return Err(action_fault(133));
        }
        let selector = catalog
            .selector(id)
            .and_then(SelectorDefinition::rule_units)
            .ok_or_else(|| action_fault(134))?;
        for dependency in selector.dependencies() {
            visit(catalog, dependency, visiting, visited, output)?;
        }
        visiting.remove(&id);
        visited.insert(id);
        output.push(id);
        Ok(())
    }

    let mut output = Vec::new();
    let mut visiting = std::collections::BTreeSet::new();
    let mut visited = std::collections::BTreeSet::new();
    for id in requested {
        visit(catalog, *id, &mut visiting, &mut visited, &mut output)?;
    }
    Ok(output)
}

impl Transaction<'_> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn resolve_rule_selector(
        &mut self,
        catalog: &CombatCatalog,
        selector: &RuleUnitSelector,
        owner: UnitId,
        actor: UnitId,
        source: Option<UnitId>,
        applier: Option<UnitId>,
        primary: Option<UnitId>,
        current_subject: Option<UnitId>,
        event_order: &[UnitId],
        input: RuleEvaluationInput<'_>,
    ) -> Result<RuleSelectorResolution, BattleFault> {
        let snapshot = self.selector_snapshot(
            selector.reference(),
            input.occurrence.event,
            input.cause.action,
        );
        if selector.reference() != RuleSelectorReference::CurrentState && snapshot.is_none() {
            return Err(action_fault(135));
        }
        let snapshot_bases = snapshot
            .as_deref()
            .map(selector_snapshot::RuleSelectorSnapshot::stat_bases)
            .transpose()
            .map_err(|_| action_fault(136))?;
        let snapshot_shields = snapshot
            .as_deref()
            .map(selector_snapshot::RuleSelectorSnapshot::shield_values);
        let snapshot_reader = snapshot_bases
            .as_ref()
            .zip(snapshot.as_deref())
            .zip(snapshot_shields.as_ref())
            .map(|((bases, snapshot), shields)| {
                StatResolver::new(catalog.modifier_registry(), bases, &snapshot.modifiers)
                    .with_shields(shields)
            });
        let mut input = input;
        if let Some(reader) = &snapshot_reader {
            input.stat_reader = Some(reader);
        }
        let snapshot = snapshot.as_deref();
        let owner_side = selector_unit(self.state, snapshot, owner)
            .ok_or_else(|| action_fault(120))?
            .side;
        let anchored = match selector.origin() {
            RuleSelectorOrigin::Source => source,
            RuleSelectorOrigin::Owner => Some(owner),
            RuleSelectorOrigin::Actor => Some(actor),
            RuleSelectorOrigin::Applier => applier,
            RuleSelectorOrigin::PrimaryTarget => primary,
            RuleSelectorOrigin::CurrentSubject => current_subject.or(primary),
            RuleSelectorOrigin::Team
            | RuleSelectorOrigin::Encounter
            | RuleSelectorOrigin::EventTargets => None,
        };
        let on_selected_side = |side| match selector.side() {
            RuleSelectorSide::Same => side == owner_side,
            RuleSelectorSide::Opposing => side != owner_side,
            RuleSelectorSide::Any => true,
        };
        let direct = anchored.filter(|unit| {
            selector_unit(self.state, snapshot, *unit)
                .is_some_and(|state| on_selected_side(state.side))
        });
        let use_direct = direct.is_some()
            && (matches!(
                selector.origin(),
                RuleSelectorOrigin::PrimaryTarget | RuleSelectorOrigin::CurrentSubject
            ) && !matches!(
                selector.choice(),
                RuleSelectorChoice::PrimaryPlusAdjacent | RuleSelectorChoice::AdjacentToPrimary
            ) && !selector
                .predicates()
                .contains(&RuleSelectorPredicate::AdjacentToPrimary)
                || selector.side() == RuleSelectorSide::Same
                    && selector.choice() == RuleSelectorChoice::First);
        let mut pool = if selector.origin() == RuleSelectorOrigin::EventTargets {
            event_order.to_vec()
        } else if use_direct {
            direct.into_iter().collect::<Vec<_>>()
        } else {
            selector_unit_ids(self.state, snapshot)
                .into_iter()
                .filter(|id| {
                    selector_unit(self.state, snapshot, *id)
                        .is_some_and(|unit| on_selected_side(unit.side))
                })
                .collect::<Vec<_>>()
        };
        pool.retain(|id| {
            selector_unit(self.state, snapshot, *id).is_some_and(|unit| {
                let life = match selector.life() {
                    RuleLifePredicate::Any => true,
                    RuleLifePredicate::Alive => unit.life == LifeState::Alive,
                    RuleLifePredicate::Downed => unit.life == LifeState::Downed,
                    RuleLifePredicate::Defeated => unit.life == LifeState::Defeated,
                };
                let presence = match selector.presence() {
                    RulePresencePredicate::Any => true,
                    RulePresencePredicate::Present => unit.presence == PresenceState::Present,
                    RulePresencePredicate::Reserved => unit.presence == PresenceState::Reserved,
                    RulePresencePredicate::Departed => unit.presence == PresenceState::Departed,
                    RulePresencePredicate::Untargetable => {
                        unit.presence == PresenceState::Untargetable
                    }
                    RulePresencePredicate::Linked => unit.presence == PresenceState::Linked,
                    RulePresencePredicate::Transformed => {
                        unit.presence == PresenceState::Transformed
                    }
                };
                life && presence
            })
        });
        for predicate in selector.predicates() {
            pool.retain(|id| match predicate {
                RuleSelectorPredicate::FormationRange { minimum, maximum } => {
                    selector_unit(self.state, snapshot, *id)
                        .is_some_and(|unit| (*minimum..=*maximum).contains(&unit.formation.get()))
                }
                RuleSelectorPredicate::AdjacentToPrimary => primary
                    .and_then(|primary| selector_unit(self.state, snapshot, primary))
                    .zip(selector_unit(self.state, snapshot, *id))
                    .is_some_and(|(primary, candidate)| {
                        primary.formation.get().abs_diff(candidate.formation.get()) == 1
                    }),
                RuleSelectorPredicate::HasMark(effect)
                | RuleSelectorPredicate::HasEffect(effect) => {
                    selector_has_effect(self.state, snapshot, *id, *effect)
                }
                RuleSelectorPredicate::HasWeakness(element) => {
                    selector_unit(self.state, snapshot, *id)
                        .is_some_and(|unit| unit.weaknesses.binary_search(element).is_ok())
                }
                RuleSelectorPredicate::LacksWeakness(element) => {
                    selector_unit(self.state, snapshot, *id)
                        .is_some_and(|unit| unit.weaknesses.binary_search(element).is_err())
                }
                RuleSelectorPredicate::HasTag(tag) => {
                    selector_has_tag(self.state, snapshot, *id, *tag)
                }
                RuleSelectorPredicate::OwnedBy(owner_selector) => {
                    let owners = input
                        .selectors
                        .binary_search_by_key(owner_selector, |result| result.selector)
                        .ok()
                        .map(|index| input.selectors[index].units)
                        .unwrap_or_default();
                    selector_owner(self.state, snapshot, *id)
                        .is_some_and(|owner| owners.contains(&owner))
                }
                RuleSelectorPredicate::Excludes(excluded_selector) => {
                    let excluded = input
                        .selectors
                        .binary_search_by_key(excluded_selector, |result| result.selector)
                        .ok()
                        .map(|index| input.selectors[index].units)
                        .unwrap_or_default();
                    !excluded.contains(id)
                }
                RuleSelectorPredicate::StatCompare {
                    stat,
                    comparison,
                    value,
                } => {
                    let Some(reader) = input.stat_reader else {
                        return false;
                    };
                    let Ok(lhs) = reader.query_stat(
                        StatQuerySubject::CurrentTarget,
                        *id,
                        *stat,
                        FormulaPurpose::Stat,
                    ) else {
                        return false;
                    };
                    let Ok(rhs) = evaluate_value(value, input, Some(*id)) else {
                        return false;
                    };
                    compare(&RuleValue::Scalar(lhs), *comparison, &rhs).unwrap_or(false)
                }
            });
        }
        if matches!(
            selector.ordering(),
            RuleSelectorOrdering::StatAscending | RuleSelectorOrdering::StatDescending
        ) {
            let expression = selector.weight().ok_or_else(|| action_fault(126))?;
            let mut keyed = pool
                .drain(..)
                .map(|id| {
                    evaluate_value(expression, input, Some(id))
                        .map(|value| (id, value))
                        .map_err(|_| action_fault(128))
                })
                .collect::<Result<Vec<_>, _>>()?;
            fallible_sort(&mut keyed, |left, right| {
                let ordering = compare_values(&left.1, &right.1).map_err(|_| action_fault(129))?;
                let ordering = if selector.ordering() == RuleSelectorOrdering::StatDescending {
                    ordering.reverse()
                } else {
                    ordering
                };
                Ok(ordering.then_with(|| left.0.cmp(&right.0)))
            })?;
            pool = keyed.into_iter().map(|value| value.0).collect();
        }
        match selector.ordering() {
            RuleSelectorOrdering::Formation => pool.sort_unstable_by_key(|id| {
                selector_unit(self.state, snapshot, *id)
                    .map(|unit| (unit.side as u8, unit.formation.get(), id.get()))
            }),
            RuleSelectorOrdering::Timeline => pool.sort_unstable_by_key(|id| {
                let gauge = selector_unit(self.state, snapshot, *id).and_then(|unit| unit.gauge);
                (
                    gauge.is_none(),
                    gauge.map_or(i64::MAX, |value| value.scaled()),
                    id.get(),
                )
            }),
            RuleSelectorOrdering::HpRatioAscending | RuleSelectorOrdering::HpRatioDescending => {
                pool.sort_unstable_by(|left, right| {
                    let left =
                        selector_unit(self.state, snapshot, *left).expect("candidate exists");
                    let right =
                        selector_unit(self.state, snapshot, *right).expect("candidate exists");
                    let ratio_ordering = (i128::from(left.current_hp.get())
                        * i128::from(right.maximum_hp.get()))
                    .cmp(&(i128::from(right.current_hp.get()) * i128::from(left.maximum_hp.get())));
                    let ratio_ordering =
                        if selector.ordering() == RuleSelectorOrdering::HpRatioDescending {
                            ratio_ordering.reverse()
                        } else {
                            ratio_ordering
                        };
                    ratio_ordering.then_with(|| left.id.cmp(&right.id))
                });
            }
            RuleSelectorOrdering::EventOrder => pool.sort_unstable_by_key(|id| {
                (
                    event_order
                        .iter()
                        .position(|candidate| candidate == id)
                        .unwrap_or(usize::MAX),
                    id.get(),
                )
            }),
            RuleSelectorOrdering::StableId => pool.sort_unstable(),
            RuleSelectorOrdering::StatAscending | RuleSelectorOrdering::StatDescending => {}
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
                let index = selector_unit(self.state, snapshot, primary)
                    .expect("candidate exists")
                    .formation
                    .get();
                pool.into_iter()
                    .filter(|id| {
                        selector_unit(self.state, snapshot, *id)
                            .is_some_and(|unit| unit.formation.get().abs_diff(index) <= 1)
                    })
                    .take(maximum)
                    .collect()
            }
            RuleSelectorChoice::AdjacentToPrimary => {
                let Some(primary) = primary else {
                    return self.finish_rule_selector(selector, Vec::new());
                };
                let index = selector_unit(self.state, snapshot, primary)
                    .ok_or_else(|| action_fault(125))?
                    .formation
                    .get();
                pool.into_iter()
                    .filter(|id| {
                        selector_unit(self.state, snapshot, *id)
                            .is_some_and(|unit| unit.formation.get().abs_diff(index) == 1)
                    })
                    .take(maximum)
                    .collect()
            }
            RuleSelectorChoice::RngUniform => {
                self.draw_rule_targets(selector, pool, None, maximum)?
            }
            RuleSelectorChoice::RngWeighted => {
                let expression = selector.weight().ok_or_else(|| action_fault(126))?;
                let weights = pool
                    .iter()
                    .map(|id| {
                        evaluate_value(expression, input, Some(*id))
                            .map_err(|_| action_fault(130))
                            .and_then(rule_weight)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                self.draw_rule_targets(selector, pool, Some(weights), maximum)?
            }
        };
        selected.truncate(maximum);
        self.finish_rule_selector(selector, selected)
    }

    fn draw_rule_targets(
        &mut self,
        selector: &RuleUnitSelector,
        mut pool: Vec<UnitId>,
        mut weights: Option<Vec<u64>>,
        maximum: usize,
    ) -> Result<Vec<UnitId>, BattleFault> {
        let purpose = selector
            .rng_purpose()
            .and_then(rule_draw_purpose)
            .ok_or_else(|| action_fault(121))?;
        let mut output = Vec::new();
        while !pool.is_empty() && output.len() < maximum {
            let before = self.state.rng.draw_count();
            let selected = if let Some(values) = weights.as_deref() {
                self.state
                    .rng
                    .choose_weighted(purpose, values)
                    .map_err(|_| action_fault(123))?
                    .map(|value| u64::from(value.index()))
            } else {
                self.state
                    .rng
                    .choose_index(
                        purpose,
                        u32::try_from(pool.len()).map_err(|_| action_fault(122))?,
                    )
                    .map_err(|_| action_fault(123))?
                    .map(|value| value.value())
            };
            let Some(selected) = selected else {
                break;
            };
            for index in before..self.state.rng.draw_count() {
                self.journal.rng_draw(index, purpose.code());
            }
            let index = usize::try_from(selected).map_err(|_| action_fault(125))?;
            output.push(pool[index]);
            if !selector.repeated() {
                pool.remove(index);
                if let Some(values) = &mut weights {
                    values.remove(index);
                }
            }
        }
        Ok(output)
    }

    fn finish_rule_selector(
        &self,
        selector: &RuleUnitSelector,
        selected: Vec<UnitId>,
    ) -> Result<RuleSelectorResolution, BattleFault> {
        if selected.len() < usize::from(selector.minimum()) {
            match selector.empty_pool() {
                RuleEmptyPoolPolicy::Fault => Err(action_fault(127)),
                RuleEmptyPoolPolicy::NoOp => Ok(RuleSelectorResolution::Selected(Box::new([]))),
                RuleEmptyPoolPolicy::Skip => Ok(RuleSelectorResolution::Skip),
                RuleEmptyPoolPolicy::CancelRemaining => Ok(RuleSelectorResolution::CancelRemaining),
            }
        } else {
            Ok(RuleSelectorResolution::Selected(
                selected.into_boxed_slice(),
            ))
        }
    }
    pub(super) fn resolve_hit_targets(
        &mut self,
        actor: UnitId,
        commitment: &mut TargetCommitment,
    ) -> Result<Box<[UnitId]>, BattleFault> {
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
        actor: UnitId,
        relation: TargetRelation,
    ) -> Result<UnitId, BattleFault> {
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

fn rule_weight(value: RuleValue) -> Result<u64, BattleFault> {
    match value {
        RuleValue::Integer(value) => u64::try_from(value).map_err(|_| action_fault(131)),
        RuleValue::Scalar(value) => u64::try_from(value.scaled()).map_err(|_| action_fault(131)),
        _ => Err(action_fault(132)),
    }
}

#[derive(Clone, Copy)]
struct SelectorUnitFacts<'a> {
    id: UnitId,
    side: TeamSide,
    formation: FormationIndex,
    life: LifeState,
    presence: PresenceState,
    current_hp: Hp,
    maximum_hp: Hp,
    gauge: Option<ActionGauge>,
    weaknesses: &'a [CombatElement],
}

fn selector_unit<'a>(
    state: &'a BattleState,
    snapshot: Option<&'a selector_snapshot::RuleSelectorSnapshot>,
    id: UnitId,
) -> Option<SelectorUnitFacts<'a>> {
    if let Some(snapshot) = snapshot {
        let unit = snapshot.units.get(&id)?;
        return Some(SelectorUnitFacts {
            id,
            side: unit.side,
            formation: unit.formation,
            life: unit.life,
            presence: unit.presence,
            current_hp: unit.current_hp,
            maximum_hp: unit.maximum_hp,
            gauge: unit.gauge,
            weaknesses: &unit.weaknesses,
        });
    }
    let unit = state.units.get(id)?;
    Some(SelectorUnitFacts {
        id,
        side: unit.side,
        formation: unit.formation,
        life: unit.life,
        presence: unit.presence,
        current_hp: unit.current_hp,
        maximum_hp: unit.maximum_hp,
        gauge: state
            .actors
            .id_for_owner(id)
            .and_then(|actor| state.actors.get(actor))
            .map(|actor| actor.gauge),
        weaknesses: &unit.weaknesses,
    })
}

fn selector_unit_ids(
    state: &BattleState,
    snapshot: Option<&selector_snapshot::RuleSelectorSnapshot>,
) -> Vec<UnitId> {
    snapshot.map_or_else(
        || state.units.iter_by_id().map(|unit| unit.id).collect(),
        |snapshot| snapshot.units.keys().copied().collect(),
    )
}

fn selector_has_effect(
    state: &BattleState,
    snapshot: Option<&selector_snapshot::RuleSelectorSnapshot>,
    unit: UnitId,
    definition: EffectDefinitionId,
) -> bool {
    snapshot.map_or_else(
        || {
            state
                .effects
                .iter_by_id()
                .any(|effect| effect.target == unit && effect.definition == definition)
        },
        |snapshot| {
            snapshot
                .effects
                .get(&unit)
                .is_some_and(|effects| effects.iter().any(|effect| effect.definition == definition))
        },
    )
}

fn selector_has_tag(
    state: &BattleState,
    snapshot: Option<&selector_snapshot::RuleSelectorSnapshot>,
    unit: UnitId,
    tag: SourceDefinitionId,
) -> bool {
    snapshot.map_or_else(
        || {
            state
                .effects
                .iter_by_id()
                .filter(|effect| effect.target == unit)
                .any(|effect| effect.tags.binary_search(&tag).is_ok())
        },
        |snapshot| {
            snapshot.effects.get(&unit).is_some_and(|effects| {
                effects
                    .iter()
                    .any(|effect| effect.tags.binary_search(&tag).is_ok())
            })
        },
    )
}

fn selector_owner(
    state: &BattleState,
    snapshot: Option<&selector_snapshot::RuleSelectorSnapshot>,
    unit: UnitId,
) -> Option<UnitId> {
    snapshot.map_or_else(
        || {
            state
                .links
                .for_unit(unit)
                .filter(|link| link.active)
                .map(|link| link.owner)
        },
        |snapshot| snapshot.owners.get(&unit).copied(),
    )
}

fn fallible_sort<T>(
    values: &mut [T],
    mut compare: impl FnMut(&T, &T) -> Result<core::cmp::Ordering, BattleFault>,
) -> Result<(), BattleFault> {
    for index in 1..values.len() {
        let mut current = index;
        while current > 0
            && compare(&values[current - 1], &values[current])? == core::cmp::Ordering::Greater
        {
            values.swap(current - 1, current);
            current -= 1;
        }
    }
    Ok(())
}

fn rule_draw_purpose(key: &str) -> Option<DrawPurpose> {
    match key {
        "bounce-target" => Some(DrawPurpose::BOUNCE_TARGET),
        "aggro-target" => Some(DrawPurpose::AGGRO_TARGET),
        "behavior-choice" => Some(DrawPurpose::BEHAVIOR_CHOICE),
        "damage-target" => Some(DrawPurpose::DAMAGE_TARGET),
        _ => None,
    }
}
