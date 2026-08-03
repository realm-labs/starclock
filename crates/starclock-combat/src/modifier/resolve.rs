//! Pure staged stat resolution with deterministic dependency-cycle faults.

use crate::{
    formula::model::CombatElement,
    modifier::model::StatQuerySubject::{self, Actor, Applier, CurrentTarget, EventTarget, Owner},
    rule::{
        evaluate::BattleQueryReader,
        model::{RuleEventFacts, RuleEventKind, RuleEventPoint, ValueExpr},
    },
};
use std::{cell::RefCell, collections::BTreeMap};

use crate::{
    ActionId, EffectCategory, EffectDefinitionId, EventId, LifeState, ModifierInstanceId,
    PresenceState, Rounding, RuleInstanceId, Scalar, UnitId, WaveInstanceId,
    rule::{
        evaluate::{RuleEvaluationError, StatQueryReader, evaluate_value, stat_query_error},
        model::{RuleCause, RuleEvaluationInput, RuleOccurrence, RuleValue},
    },
};

use super::model;
use super::model::{
    ActiveModifier, FormulaModifierQuery, FormulaPurpose, FormulaStage, LifeFilter,
    ModifierAggregation, ModifierDefinition, ModifierFilter, ModifierQueryContext, PresenceFilter,
    SnapshotPolicy, StatKind, StatQuery,
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
    shields: Option<&'a BTreeMap<UnitId, Scalar>>,
    effect_stacks: Option<&'a BTreeMap<(UnitId, EffectDefinitionId), i64>>,
    effect_category_stacks: Option<&'a BTreeMap<(UnitId, EffectCategory), i64>>,
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
            shields: None,
            effect_stacks: None,
            effect_category_stacks: None,
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

    /// Supplies current effective shield values for dynamic modifier expressions.
    #[must_use]
    pub const fn with_shields(mut self, shields: &'a BTreeMap<UnitId, Scalar>) -> Self {
        self.shields = Some(shields);
        self
    }

    /// Supplies current aggregate effect stacks for dynamic modifier expressions.
    #[must_use]
    pub const fn with_effect_category_stacks(
        mut self,
        values: &'a BTreeMap<(UnitId, EffectCategory), i64>,
    ) -> Self {
        self.effect_category_stacks = Some(values);
        self
    }

    /// Supplies current aggregate stacks keyed by effect definition.
    #[must_use]
    pub const fn with_effect_stacks(
        mut self,
        values: &'a BTreeMap<(UnitId, EffectDefinitionId), i64>,
    ) -> Self {
        self.effect_stacks = Some(values);
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

    /// Resolves one non-stat formula stage for one explicitly selected subject.
    pub fn query_formula(
        &self,
        query: FormulaModifierQuery,
        context: &ModifierQueryContext,
    ) -> Result<Scalar, ModifierQueryError> {
        *self.context.borrow_mut() = context.clone();
        let mut groups = BTreeMap::<_, Vec<_>>::new();
        for instance in self.instances {
            let definition = self
                .registry
                .definition(instance.definition)
                .ok_or(ModifierQueryError::MissingDefinition(instance.instance))?;
            if instance.subject == query.subject
                && definition.stage == query.stage
                && definition.purpose == query.purpose
                && matches_filters(definition, instance, context)
            {
                let value = self.value(instance, definition, context)?;
                groups
                    .entry(definition.stacking_group)
                    .or_default()
                    .push((instance, definition, value));
            }
        }
        let mut group_values = Vec::with_capacity(groups.len());
        for (group_id, mut values) in groups {
            sort_group(&mut values);
            let group = self
                .registry
                .group(group_id)
                .expect("registry checked group");
            group_values.push(self.aggregate_group(group, &values)?);
        }
        let value = sum(group_values.into_iter())?;
        apply_bounds(
            value,
            self.instances.iter().filter_map(|instance| {
                let definition = self.registry.definition(instance.definition)?;
                (instance.subject == query.subject
                    && definition.stage == query.stage
                    && definition.purpose == query.purpose
                    && definition.cap_stage == query.stage
                    && matches_filters(definition, instance, context))
                .then_some(definition)
            }),
        )
    }

    /// Evaluates one modifier expression against current inputs for snapshot capture.
    pub(crate) fn capture_value(
        &self,
        instance: &ActiveModifier,
        definition: &ModifierDefinition,
    ) -> Result<Scalar, ModifierQueryError> {
        self.evaluate_expression(instance, &definition.value, SnapshotPolicy::Dynamic)
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
                    && (definition.purpose == FormulaPurpose::Stat
                        || definition.purpose == query.purpose)
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
                sort_group(&mut values);
                let group = self
                    .registry
                    .group(group_id)
                    .expect("registry checked group");
                stage_values.push(self.aggregate_group(group, &values)?);
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
                    Rounding::NearestTiesEven,
                ),
                FormulaStage::FinalMultiply => {
                    result.checked_mul(combined, Rounding::NearestTiesEven)
                }
                _ => unreachable!(),
            }
            .map_err(|_| ModifierQueryError::Numeric)?;
            result = apply_bounds(
                result,
                self.instances.iter().filter_map(|instance| {
                    let definition = self.registry.definition(instance.definition)?;
                    (instance.subject == query.subject
                        && definition.stat == query.stat
                        && (definition.purpose == FormulaPurpose::Stat
                            || definition.purpose == query.purpose)
                        && definition.stage <= stage
                        && definition.cap_stage == stage
                        && matches_filters(definition, instance, context))
                    .then_some(definition)
                }),
            )?;
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
        self.evaluate_expression(instance, &definition.value, definition.snapshot)
    }

    fn evaluate_expression(
        &self,
        instance: &ActiveModifier,
        expression: &ValueExpr,
        policy: SnapshotPolicy,
    ) -> Result<Scalar, ModifierQueryError> {
        let snapshot_reader = SnapshotReader {
            resolver: self,
            instance,
            policy,
        };
        let reader: &dyn StatQueryReader = if policy == SnapshotPolicy::Dynamic {
            self
        } else {
            &snapshot_reader
        };
        let event_facts = RuleEventFacts {
            point: Some(RuleEventPoint::RuleStateChanged),
            ..RuleEventFacts::default()
        };
        let battle_reader = (self.shields.is_some()
            || self.effect_stacks.is_some()
            || self.effect_category_stacks.is_some())
        .then_some(ModifierBattleQuery {
            shields: self.shields,
            effect_stacks: self.effect_stacks,
            effect_category_stacks: self.effect_category_stacks,
        });
        let input = RuleEvaluationInput {
            event_kind: RuleEventKind::Rule,
            event_facts: &event_facts,
            cause: RuleCause {
                parent_event: None,
                root_command: None,
                action: instance.application_action,
                phase: None,
                hit: None,
                owner: Some(instance.owner),
                actor: Some(instance.owner),
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
            rule_owner: Some(instance.owner),
            source_tags: &[],
            slots: &instance.slots,
            selectors: &[],
            stat_reader: Some(reader),
            ability_parameter_reader: None,
            resource_reader: None,
            battle_query_reader: battle_reader
                .as_ref()
                .map(|reader| reader as &dyn BattleQueryReader),
        };
        let value = evaluate_value(expression, input, Some(instance.subject));
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

    fn aggregate_group(
        &self,
        group: &model::ModifierStackingGroup,
        values: &[(&ActiveModifier, &ModifierDefinition, Scalar)],
    ) -> Result<Scalar, ModifierQueryError> {
        if group.aggregation != ModifierAggregation::StrongestByComparator {
            return aggregate(group.aggregation, values);
        }
        let comparator = group
            .comparator
            .as_ref()
            .expect("registry checked comparator");
        let mut winner = None::<(Scalar, usize)>;
        for (index, (instance, definition, _)) in values.iter().enumerate() {
            let key = self.evaluate_expression(instance, comparator, definition.snapshot)?;
            if winner.is_none_or(|current| (key, index) > current) {
                winner = Some((key, index));
            }
        }
        Ok(values[winner.expect("nonempty").1].2)
    }
}

impl StatQueryReader for StatResolver<'_> {
    fn query_stat(
        &self,
        _origin: StatQuerySubject,
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

    fn query_base_stat(
        &self,
        _origin: StatQuerySubject,
        subject: UnitId,
        stat: StatKind,
    ) -> Result<Scalar, RuleEvaluationError> {
        self.bases
            .get(&(subject, stat))
            .copied()
            .ok_or_else(|| stat_query_error(0x205))
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
        origin: StatQuerySubject,
        subject: UnitId,
        stat: StatKind,
        purpose: FormulaPurpose,
    ) -> Result<Scalar, RuleEvaluationError> {
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

    fn query_base_stat(
        &self,
        origin: StatQuerySubject,
        subject: UnitId,
        stat: StatKind,
    ) -> Result<Scalar, RuleEvaluationError> {
        self.resolver.query_base_stat(origin, subject, stat)
    }
}

struct ModifierBattleQuery<'a> {
    shields: Option<&'a BTreeMap<UnitId, Scalar>>,
    effect_stacks: Option<&'a BTreeMap<(UnitId, EffectDefinitionId), i64>>,
    effect_category_stacks: Option<&'a BTreeMap<(UnitId, EffectCategory), i64>>,
}

impl BattleQueryReader for ModifierBattleQuery<'_> {
    fn life_presence(&self, _subject: UnitId) -> Option<(LifeState, PresenceState)> {
        None
    }

    fn has_effect(&self, _subject: UnitId, _effect: EffectDefinitionId) -> bool {
        false
    }

    fn is_frozen(&self, _subject: UnitId) -> bool {
        false
    }

    fn has_weakness(&self, _subject: UnitId, _element: CombatElement) -> bool {
        false
    }

    fn is_broken(&self, _subject: UnitId) -> bool {
        false
    }

    fn current_shield(&self, subject: UnitId) -> Option<Scalar> {
        self.shields?.get(&subject).copied()
    }

    fn effect_stacks(&self, subject: UnitId, effect: EffectDefinitionId) -> Option<i64> {
        Some(
            self.effect_stacks
                .and_then(|values| values.get(&(subject, effect)).copied())
                .unwrap_or(0),
        )
    }

    fn effect_category_stacks(&self, subject: UnitId, category: EffectCategory) -> Option<i64> {
        Some(
            self.effect_category_stacks
                .and_then(|values| values.get(&(subject, category)).copied())
                .unwrap_or(0),
        )
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
        StrongestByComparator => unreachable!("resolved through the authored comparator"),
        UniquePerSource => {
            let mut per_source = BTreeMap::new();
            for value in values {
                per_source.insert(value.0.source, value.2);
            }
            sum(per_source.into_values())
        }
    }
}

fn sort_group(values: &mut [(&ActiveModifier, &ModifierDefinition, Scalar)]) {
    values.sort_by_key(|(instance, definition, _)| {
        (
            definition.priority,
            instance.source,
            instance.insertion_sequence,
            instance.instance,
        )
    });
}

fn sum(mut values: impl Iterator<Item = Scalar>) -> Result<Scalar, ModifierQueryError> {
    values.try_fold(Scalar::ZERO, |left, right| {
        left.checked_add(right)
            .map_err(|_| ModifierQueryError::Numeric)
    })
}

fn product(mut values: impl Iterator<Item = Scalar>) -> Result<Scalar, ModifierQueryError> {
    values.try_fold(Scalar::ONE, |left, right| {
        left.checked_mul(right, Rounding::NearestTiesEven)
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

fn apply_bounds<'a>(
    value: Scalar,
    definitions: impl Iterator<Item = &'a ModifierDefinition>,
) -> Result<Scalar, ModifierQueryError> {
    let mut floor = None::<Scalar>;
    let mut cap = None::<Scalar>;
    for definition in definitions {
        if let Some(value) = definition.floor {
            floor = Some(floor.map_or(value, |current| current.max(value)));
        }
        if let Some(value) = definition.cap {
            cap = Some(cap.map_or(value, |current| current.min(value)));
        }
    }
    if floor.zip(cap).is_some_and(|(floor, cap)| floor > cap) {
        return Err(ModifierQueryError::Numeric);
    }
    let mut bounded = value;
    if let Some(floor) = floor {
        bounded = bounded.max(floor);
    }
    if let Some(cap) = cap {
        bounded = bounded.min(cap);
    }
    Ok(bounded)
}

fn matches_filters(
    definition: &ModifierDefinition,
    instance: &ActiveModifier,
    context: &ModifierQueryContext,
) -> bool {
    if context.formula_subject == Some(model::FormulaSubject::Target)
        && !definition.filters.iter().any(|filter| {
            matches!(
                filter,
                ModifierFilter::FormulaSubject(model::FormulaSubject::Target)
            )
        })
    {
        return false;
    }
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
        ModifierFilter::FormulaSubject(value) => context.formula_subject == Some(*value),
    })
}
