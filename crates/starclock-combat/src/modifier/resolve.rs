//! Pure staged stat resolution with deterministic dependency-cycle faults.

use std::{cell::RefCell, collections::BTreeMap};

use crate::{
    ActionId, EventId, ModifierInstanceId, RuleInstanceId, Scalar, UnitId, WaveInstanceId,
    rule::{
        evaluate::{RuleEvaluationError, StatQueryReader, evaluate_value, stat_query_error},
        model::{RuleCause, RuleEvaluationInput, RuleOccurrence, RuleValue},
    },
};

use super::model::{
    ActiveModifier, FormulaPurpose, FormulaStage, LifeFilter, ModifierAggregation,
    ModifierDefinition, ModifierFilter, ModifierQueryContext, PresenceFilter, SnapshotPolicy,
    StatKind, StatQuery,
};
use super::registry::ModifierRegistry;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModifierQueryError {
    MissingBase(StatQuery),
    MissingDefinition(ModifierInstanceId),
    InvalidSnapshot(ModifierInstanceId),
    InvalidValue(ModifierInstanceId),
    Numeric,
    StatQueryCycle(Box<[StatQuery]>),
}

impl core::fmt::Display for ModifierQueryError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "modifier query failed: {self:?}")
    }
}

impl std::error::Error for ModifierQueryError {}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct CacheKey {
    query: StatQuery,
    context: ModifierQueryContext,
}

pub struct StatResolver<'a> {
    registry: &'a ModifierRegistry,
    bases: &'a BTreeMap<(UnitId, StatKind), Scalar>,
    instances: &'a [ActiveModifier],
    context: RefCell<ModifierQueryContext>,
    stack: RefCell<Vec<StatQuery>>,
    deferred_error: RefCell<Option<ModifierQueryError>>,
    cache: RefCell<BTreeMap<CacheKey, Scalar>>,
    cache_enabled: bool,
}

impl<'a> StatResolver<'a> {
    #[must_use]
    pub fn new(
        registry: &'a ModifierRegistry,
        bases: &'a BTreeMap<(UnitId, StatKind), Scalar>,
        instances: &'a [ActiveModifier],
    ) -> Self {
        Self {
            registry,
            bases,
            instances,
            context: RefCell::default(),
            stack: RefCell::default(),
            deferred_error: RefCell::default(),
            cache: RefCell::default(),
            cache_enabled: true,
        }
    }

    #[must_use]
    pub const fn without_cache(mut self) -> Self {
        self.cache_enabled = false;
        self
    }

    pub fn query(
        &self,
        query: StatQuery,
        context: &ModifierQueryContext,
    ) -> Result<Scalar, ModifierQueryError> {
        *self.context.borrow_mut() = context.clone();
        self.resolve(query, context)
    }

    fn resolve(
        &self,
        query: StatQuery,
        context: &ModifierQueryContext,
    ) -> Result<Scalar, ModifierQueryError> {
        let key = CacheKey {
            query,
            context: context.clone(),
        };
        if self.cache_enabled
            && let Some(value) = self.cache.borrow().get(&key)
        {
            return Ok(*value);
        }
        {
            let mut stack = self.stack.borrow_mut();
            if let Some(index) = stack.iter().position(|active| *active == query) {
                let mut cycle = stack[index..].to_vec();
                cycle.push(query);
                return Err(ModifierQueryError::StatQueryCycle(cycle.into_boxed_slice()));
            }
            stack.push(query);
        }
        let result = self.resolve_inner(query, context);
        self.stack.borrow_mut().pop();
        if let Ok(value) = result
            && self.cache_enabled
        {
            self.cache.borrow_mut().insert(key, value);
        }
        result
    }

    fn resolve_inner(
        &self,
        query: StatQuery,
        context: &ModifierQueryContext,
    ) -> Result<Scalar, ModifierQueryError> {
        let authored_base = *self
            .bases
            .get(&(query.subject, query.stat))
            .ok_or(ModifierQueryError::MissingBase(query))?;
        let mut result = authored_base;
        for stage in [
            FormulaStage::BaseAdd,
            FormulaStage::PercentOfBase,
            FormulaStage::Flat,
            FormulaStage::FinalAdd,
            FormulaStage::FinalMultiply,
        ] {
            let mut groups = BTreeMap::<_, Vec<_>>::new();
            for instance in self.instances {
                let definition = self
                    .registry
                    .definition(instance.definition)
                    .ok_or(ModifierQueryError::MissingDefinition(instance.instance))?;
                if instance.subject == query.subject
                    && definition.stat == query.stat
                    && definition.purpose == query.purpose
                    && definition.stage == stage
                    && matches_filters(definition, instance, context)
                {
                    let value = self.value(instance, definition, context)?;
                    groups
                        .entry(definition.stacking_group)
                        .or_default()
                        .push((instance, definition, value));
                }
            }
            let mut stage_values = Vec::new();
            for (group_id, mut values) in groups {
                values.sort_by_key(|(instance, definition, _)| {
                    (
                        definition.priority,
                        instance.source,
                        instance.insertion_sequence,
                        instance.instance,
                    )
                });
                let policy = self
                    .registry
                    .group(group_id)
                    .expect("registry checked group")
                    .aggregation;
                stage_values.push(aggregate(policy, &values)?);
            }
            let combined = combine_stage(stage, &stage_values)?;
            result = match stage {
                FormulaStage::BaseAdd | FormulaStage::Flat | FormulaStage::FinalAdd => {
                    result.checked_add(combined)
                }
                FormulaStage::PercentOfBase => result.checked_mul(
                    Scalar::ONE
                        .checked_add(combined)
                        .map_err(|_| ModifierQueryError::Numeric)?,
                    crate::Rounding::NearestTiesEven,
                ),
                FormulaStage::FinalMultiply => {
                    result.checked_mul(combined, crate::Rounding::NearestTiesEven)
                }
                _ => unreachable!(),
            }
            .map_err(|_| ModifierQueryError::Numeric)?;
            for (_, definition, _) in
                groups_for_bounds(self.instances, self.registry, query, stage, context)
            {
                if definition.cap_stage == stage {
                    if let Some(floor) = definition.floor {
                        result = result.max(floor);
                    }
                    if let Some(cap) = definition.cap {
                        result = result.min(cap);
                    }
                }
            }
        }
        Ok(result)
    }

    fn value(
        &self,
        instance: &ActiveModifier,
        definition: &ModifierDefinition,
        _context: &ModifierQueryContext,
    ) -> Result<Scalar, ModifierQueryError> {
        match definition.snapshot {
            SnapshotPolicy::OnApplication
            | SnapshotPolicy::OnActionStart
            | SnapshotPolicy::OnPhaseStart
            | SnapshotPolicy::OnHitStart
            | SnapshotPolicy::RecomputeOnStackChange => {
                return instance
                    .captured_value
                    .ok_or(ModifierQueryError::InvalidSnapshot(instance.instance));
            }
            SnapshotPolicy::Dynamic
            | SnapshotPolicy::SourceSnapshotTargetDynamic
            | SnapshotPolicy::SourceDynamicTargetSnapshot
            | SnapshotPolicy::ExplicitFields => {}
        }
        let snapshot_reader = SnapshotReader {
            resolver: self,
            instance,
            policy: definition.snapshot,
        };
        let reader: &dyn StatQueryReader = if definition.snapshot == SnapshotPolicy::Dynamic {
            self
        } else {
            &snapshot_reader
        };
        let input = RuleEvaluationInput {
            event_kind: crate::rule::model::RuleEventKind::Rule,
            cause: RuleCause {
                owner: Some(instance.owner),
                actor: None,
                applier: Some(instance.owner),
                target: Some(instance.subject),
                source: Some(instance.source),
            },
            occurrence: RuleOccurrence {
                rule_instance: RuleInstanceId::new(instance.instance.get()).expect("nonzero"),
                event: EventId::new(1).expect("nonzero"),
                hit: None,
                target: Some(instance.subject),
                ability: None,
                action: instance.application_action.or(ActionId::new(1)),
                turn_event: None,
                wave: WaveInstanceId::new(1).expect("nonzero"),
            },
            source_tags: &[],
            slots: &instance.slots,
            selectors: &[],
            stat_reader: Some(reader),
        };
        let value = evaluate_value(&definition.value, input, Some(instance.subject));
        if let Some(error) = self.deferred_error.borrow_mut().take() {
            return Err(error);
        }
        match value.map_err(|_| ModifierQueryError::InvalidValue(instance.instance))? {
            RuleValue::Scalar(value) => Ok(value),
            RuleValue::Integer(value) => {
                Scalar::checked_from_integer(value).map_err(|_| ModifierQueryError::Numeric)
            }
            _ => Err(ModifierQueryError::InvalidValue(instance.instance)),
        }
    }
}

impl StatQueryReader for StatResolver<'_> {
    fn query_stat(
        &self,
        _origin: crate::modifier::model::StatQuerySubject,
        subject: UnitId,
        stat: StatKind,
        purpose: FormulaPurpose,
    ) -> Result<Scalar, RuleEvaluationError> {
        let query = StatQuery {
            subject,
            stat,
            purpose,
        };
        let context = self.context.borrow().clone();
        self.resolve(query, &context).map_err(|error| {
            *self.deferred_error.borrow_mut() = Some(error);
            stat_query_error(0x203)
        })
    }
}

struct SnapshotReader<'a, 'b> {
    resolver: &'a StatResolver<'b>,
    instance: &'a ActiveModifier,
    policy: SnapshotPolicy,
}

impl StatQueryReader for SnapshotReader<'_, '_> {
    fn query_stat(
        &self,
        origin: crate::modifier::model::StatQuerySubject,
        subject: UnitId,
        stat: StatKind,
        purpose: FormulaPurpose,
    ) -> Result<Scalar, RuleEvaluationError> {
        use crate::modifier::model::StatQuerySubject::{
            Actor, Applier, CurrentTarget, EventTarget, Owner,
        };
        let should_capture = match self.policy {
            SnapshotPolicy::SourceSnapshotTargetDynamic => {
                matches!(origin, Owner | Actor | Applier)
            }
            SnapshotPolicy::SourceDynamicTargetSnapshot => {
                matches!(origin, EventTarget | CurrentTarget)
            }
            SnapshotPolicy::ExplicitFields => self.instance.captured_stats.iter().any(|entry| {
                entry.0
                    == StatQuery {
                        subject,
                        stat,
                        purpose,
                    }
            }),
            _ => false,
        };
        if should_capture {
            let query = StatQuery {
                subject,
                stat,
                purpose,
            };
            return self
                .instance
                .captured_stats
                .binary_search_by_key(&query, |entry| entry.0)
                .ok()
                .map(|index| self.instance.captured_stats[index].1)
                .ok_or_else(|| {
                    *self.resolver.deferred_error.borrow_mut() =
                        Some(ModifierQueryError::InvalidSnapshot(self.instance.instance));
                    stat_query_error(0x204)
                });
        }
        self.resolver.query_stat(origin, subject, stat, purpose)
    }
}

fn aggregate(
    policy: ModifierAggregation,
    values: &[(&ActiveModifier, &ModifierDefinition, Scalar)],
) -> Result<Scalar, ModifierQueryError> {
    use ModifierAggregation::*;
    match policy {
        Sum => sum(values.iter().map(|value| value.2)),
        Product => product(values.iter().map(|value| value.2)),
        Maximum => Ok(values.iter().map(|value| value.2).max().expect("nonempty")),
        Minimum => Ok(values.iter().map(|value| value.2).min().expect("nonempty")),
        Latest | ReplaceGroup => Ok(values.last().expect("nonempty").2),
        Earliest => Ok(values.first().expect("nonempty").2),
        StrongestByComparator => Ok(values
            .iter()
            .max_by_key(|value| (value.2.scaled().unsigned_abs(), value.0.instance))
            .expect("nonempty")
            .2),
        UniquePerSource => {
            let mut per_source = BTreeMap::new();
            for value in values {
                per_source.insert(value.0.source, value.2);
            }
            sum(per_source.into_values())
        }
    }
}

fn sum(mut values: impl Iterator<Item = Scalar>) -> Result<Scalar, ModifierQueryError> {
    values.try_fold(Scalar::ZERO, |left, right| {
        left.checked_add(right)
            .map_err(|_| ModifierQueryError::Numeric)
    })
}

fn product(mut values: impl Iterator<Item = Scalar>) -> Result<Scalar, ModifierQueryError> {
    values.try_fold(Scalar::ONE, |left, right| {
        left.checked_mul(right, crate::Rounding::NearestTiesEven)
            .map_err(|_| ModifierQueryError::Numeric)
    })
}

fn combine_stage(stage: FormulaStage, values: &[Scalar]) -> Result<Scalar, ModifierQueryError> {
    if stage == FormulaStage::FinalMultiply {
        product(values.iter().copied())
    } else {
        sum(values.iter().copied())
    }
}

fn groups_for_bounds<'a>(
    instances: &'a [ActiveModifier],
    registry: &'a ModifierRegistry,
    query: StatQuery,
    stage: FormulaStage,
    context: &'a ModifierQueryContext,
) -> impl Iterator<Item = (&'a ActiveModifier, &'a ModifierDefinition, ())> {
    instances.iter().filter_map(move |instance| {
        let definition = registry.definition(instance.definition)?;
        (instance.subject == query.subject
            && definition.stat == query.stat
            && definition.purpose == query.purpose
            && definition.stage == stage
            && matches_filters(definition, instance, context))
        .then_some((instance, definition, ()))
    })
}

fn matches_filters(
    definition: &ModifierDefinition,
    instance: &ActiveModifier,
    context: &ModifierQueryContext,
) -> bool {
    definition.filters.iter().all(|filter| match filter {
        ModifierFilter::AbilityTag(tag) => context.ability_tags.binary_search(tag).is_ok(),
        ModifierFilter::DamageTag(tag) => context.damage_tags.binary_search(tag).is_ok(),
        ModifierFilter::Element(value) => context.element == Some(*value),
        ModifierFilter::Action(value) => context.action_kind == Some(*value),
        ModifierFilter::Life(LifeFilter::Any) | ModifierFilter::Presence(PresenceFilter::Any) => {
            true
        }
        ModifierFilter::Life(value) => context.life == Some(*value),
        ModifierFilter::Presence(value) => context.presence == Some(*value),
        ModifierFilter::Source(value) => {
            instance.source_class == *value
                && context.source_class.is_none_or(|actual| actual == *value)
        }
        ModifierFilter::Target(value) => context
            .matched_target_selectors
            .binary_search(value)
            .is_ok(),
    })
}
