//! Generated Sora modifier rows to Starclock-owned Rule IR and definitions.

use std::collections::BTreeSet;

use starclock_combat::modifier::model::{
    FormulaPurpose, FormulaStage, LifeFilter, ModifierAggregation, ModifierDefinition,
    ModifierFilter, ModifierStackingGroup, PresenceFilter, SnapshotPolicy, StatKind,
    StatQuerySubject,
};
use starclock_combat::modifier::registry::ModifierRegistry;
use starclock_combat::{
    ModifierDefinitionId, ModifierStackingGroupId, Rounding, Scalar, SelectorId,
    rule::model::{RuleResourceKind, RuleValue, RuleValueKind, ValueExpr},
};

use crate::{
    catalog::{CatalogLoadError, domain_fail, parse_decimal},
    generated::{
        self, SoraConfig, formula_purpose, formula_stage, modifier_aggregation,
        modifier_filter_node, rounding_policy, selector_origin, snapshot_policy, stat_kind,
        value_binary_operator, value_expression_node,
    },
};

pub(super) fn convert(config: &SoraConfig) -> Result<ModifierRegistry, CatalogLoadError> {
    validate_support_rows(config)?;
    let groups = config
        .modifier_stacking_group()
        .ordered_rows()
        .map(|row| {
            if row.comparator_expression_id.is_some()
                && row.aggregation
                    != modifier_aggregation::ModifierAggregation::StrongestByComparator
            {
                return Err(domain_fail(
                    "only strongest modifier groups accept comparators",
                ));
            }
            Ok(ModifierStackingGroup {
                id: group_id(row.id)?,
                aggregation: aggregation(row.aggregation),
            })
        })
        .collect::<Result<Vec<_>, CatalogLoadError>>()?;
    let definitions = config
        .modifier_definition()
        .ordered_rows()
        .map(|row| lower_definition(config, row))
        .collect::<Result<Vec<_>, CatalogLoadError>>()?;
    ModifierRegistry::new(groups, definitions).map_err(domain_fail)
}

fn validate_support_rows(config: &SoraConfig) -> Result<(), CatalogLoadError> {
    for row in config.rule_definition().ordered_rows() {
        if row.native_handler_id.is_some() {
            return Err(domain_fail(format!(
                "probe rule {} requires a native handler",
                row.id
            )));
        }
    }
    for row in config.state_slot().ordered_rows() {
        let initial = integer_literal(config, row.initial_expression_id)?;
        if row.value_kind == generated::slot_value_kind::SlotValueKind::BoundedInteger {
            let minimum = integer_literal(
                config,
                row.minimum_expression_id.ok_or_else(|| {
                    domain_fail(format!("bounded state slot {} lacks a minimum", row.id))
                })?,
            )?;
            let maximum = integer_literal(
                config,
                row.maximum_expression_id.ok_or_else(|| {
                    domain_fail(format!("bounded state slot {} lacks a maximum", row.id))
                })?,
            )?;
            if minimum > initial || initial > maximum {
                return Err(domain_fail(format!(
                    "state slot {} initial value is outside its bounds",
                    row.id
                )));
            }
        }
    }
    Ok(())
}

fn integer_literal(config: &SoraConfig, id: i32) -> Result<i64, CatalogLoadError> {
    match expression(config, id, &mut BTreeSet::new())? {
        ValueExpr::Literal(RuleValue::Integer(value)) => Ok(value),
        _ => Err(domain_fail(format!(
            "state-slot expression {id} is not an integer literal"
        ))),
    }
}

fn lower_definition(
    config: &SoraConfig,
    row: &generated::modifier_definition::ModifierDefinition,
) -> Result<ModifierDefinition, CatalogLoadError> {
    let filters = config
        .modifier_filter()
        .iter()
        .filter(|filter| filter.modifier_id == row.id)
        .map(|filter| lower_filter(&filter.filter))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ModifierDefinition {
        id: modifier_id(row.id)?,
        stat: stat(row.stat),
        stage: stage(row.formula_stage),
        purpose: purpose(row.formula_purpose),
        value: expression(config, row.value_expression_id, &mut BTreeSet::new())?,
        stacking_group: group_id(row.stacking_group_id)?,
        priority: row.priority,
        floor: optional_scalar(config, row.floor_expression_id)?,
        cap: optional_scalar(config, row.cap_expression_id)?,
        cap_stage: stage(row.cap_formula_stage),
        snapshot: snapshot(row.snapshot_policy),
        filters: filters.into_boxed_slice(),
    })
}

pub(super) fn expression(
    config: &SoraConfig,
    id: i32,
    stack: &mut BTreeSet<i32>,
) -> Result<ValueExpr, CatalogLoadError> {
    if !stack.insert(id) {
        return Err(domain_fail(format!("value expression cycle at {id}")));
    }
    let row = config
        .value_expression()
        .get(&id)
        .ok_or_else(|| domain_fail(format!("missing value expression {id}")))?;
    use value_expression_node::ValueExpressionNode as Node;
    let result = match &row.node {
        Node::IntegerLiteral { value } => ValueExpr::Literal(RuleValue::Integer(i64::from(*value))),
        Node::ScalarLiteral { value_decimal }
        | Node::RatioLiteral { value_decimal }
        | Node::ProbabilityLiteral { value_decimal } => ValueExpr::Literal(RuleValue::Scalar(
            Scalar::from_scaled(parse_decimal(value_decimal)?),
        )),
        Node::StableIdLiteral { identity_id } => {
            ValueExpr::Literal(RuleValue::StableId(u64::from(positive(*identity_id)?)))
        }
        Node::BooleanLiteral { value } => ValueExpr::Literal(RuleValue::Boolean(*value)),
        Node::ReadStateSlot { state_slot_id } => ValueExpr::Slot(
            starclock_combat::StateSlotDefinitionId::new(positive(*state_slot_id)?)
                .expect("positive ID"),
        ),
        Node::AbilityParameter { parameter_key } => ValueExpr::AbilityParameter {
            key: parameter_key.clone().into_boxed_str(),
            kind: value_kind(row.result_kind)?,
        },
        Node::ReadResource {
            subject_selector_id,
            resource_kind,
            character_resource_key,
        } => ValueExpr::ReadResource {
            selector: selector(*subject_selector_id)?,
            resource: lower_resource(*resource_kind, character_resource_key.as_deref())?,
        },
        Node::QueryStat {
            subject_selector_id,
            stat: query_stat,
            formula_purpose,
        } => ValueExpr::QueryStat {
            subject: query_subject(config, *subject_selector_id)?,
            stat: stat(*query_stat),
            purpose: purpose(*formula_purpose),
        },
        Node::SelectorCount { selector_id } => ValueExpr::SelectorCount(selector(*selector_id)?),
        Node::ReadEventProperty { property } => {
            ValueExpr::ReadEventProperty(crate::rule_lower::lower_event_property(*property))
        }
        Node::SelectorSum {
            selector_id,
            value_expression_id,
        } => ValueExpr::SelectorSum {
            selector: selector(*selector_id)?,
            value: Box::new(expression(config, *value_expression_id, stack)?),
        },
        Node::CheckedBinary {
            operator,
            left_expression_id,
            right_expression_id,
            rounding,
        } => {
            let left = Box::new(expression(config, *left_expression_id, stack)?);
            let right = Box::new(expression(config, *right_expression_id, stack)?);
            use value_binary_operator::ValueBinaryOperator as Operator;
            match operator {
                Operator::CheckedAdd => ValueExpr::Add(left, right),
                Operator::CheckedSubtract => ValueExpr::Subtract(left, right),
                Operator::CheckedMultiply => ValueExpr::Multiply {
                    lhs: left,
                    rhs: right,
                    rounding: round(*rounding),
                },
                Operator::CheckedDivide => ValueExpr::Divide {
                    lhs: left,
                    rhs: right,
                    rounding: round(*rounding),
                },
                Operator::Minimum => ValueExpr::Minimum(left, right),
                Operator::Maximum => ValueExpr::Maximum(left, right),
            }
        }
        Node::Clamp {
            value_expression_id,
            minimum_expression_id,
            maximum_expression_id,
        } => ValueExpr::Clamp {
            value: Box::new(expression(config, *value_expression_id, stack)?),
            minimum: Box::new(expression(config, *minimum_expression_id, stack)?),
            maximum: Box::new(expression(config, *maximum_expression_id, stack)?),
        },
        Node::Negate {
            operand_expression_id,
        } => ValueExpr::Negate(Box::new(expression(config, *operand_expression_id, stack)?)),
        Node::Choose {
            condition_id,
            when_true_expression_id,
            when_false_expression_id,
        } => ValueExpr::Choose {
            condition: Box::new(crate::rule_lower::lower_condition(
                config,
                *condition_id,
                &mut BTreeSet::new(),
            )?),
            when_true: Box::new(expression(config, *when_true_expression_id, stack)?),
            when_false: Box::new(expression(config, *when_false_expression_id, stack)?),
        },
        Node::Convert {
            operand_expression_id,
            target_kind,
            rounding,
        } => ValueExpr::Convert {
            value: Box::new(expression(config, *operand_expression_id, stack)?),
            target: value_kind(*target_kind)?,
            rounding: round(*rounding),
        },
    };
    stack.remove(&id);
    Ok(result)
}

fn lower_resource(
    kind: generated::resource_kind::ResourceKind,
    key: Option<&str>,
) -> Result<RuleResourceKind, CatalogLoadError> {
    use generated::resource_kind::ResourceKind as V;
    match (kind, key) {
        (V::Energy, None) => Ok(RuleResourceKind::Energy),
        (V::SkillPoints, None) => Ok(RuleResourceKind::SkillPoints),
        (V::CharacterResource, Some(key)) => Ok(RuleResourceKind::Character(key.into())),
        (V::TeamResource, Some(key)) => Ok(RuleResourceKind::Team(key.into())),
        _ => Err(domain_fail(
            "resource expression kind/key combination is invalid",
        )),
    }
}

fn optional_scalar(
    config: &SoraConfig,
    id: Option<i32>,
) -> Result<Option<Scalar>, CatalogLoadError> {
    id.map(|id| match expression(config, id, &mut BTreeSet::new())? {
        ValueExpr::Literal(RuleValue::Scalar(value)) => Ok(value),
        _ => Err(domain_fail(format!(
            "bound expression {id} is not a scalar literal"
        ))),
    })
    .transpose()
}

fn query_subject(config: &SoraConfig, id: i32) -> Result<StatQuerySubject, CatalogLoadError> {
    let origin = config
        .selector()
        .get(&id)
        .ok_or_else(|| domain_fail(format!("missing stat-query selector {id}")))?
        .origin;
    use selector_origin::SelectorOrigin as Origin;
    match origin {
        Origin::Owner => Ok(StatQuerySubject::Owner),
        Origin::Actor => Ok(StatQuerySubject::Actor),
        Origin::Applier => Ok(StatQuerySubject::Applier),
        Origin::PrimaryTarget => Ok(StatQuerySubject::EventTarget),
        Origin::CurrentSubject => Ok(StatQuerySubject::CurrentTarget),
        _ => Err(domain_fail(format!(
            "selector {id} is not a single query subject"
        ))),
    }
}

fn lower_filter(
    node: &modifier_filter_node::ModifierFilterNode,
) -> Result<ModifierFilter, CatalogLoadError> {
    use modifier_filter_node::ModifierFilterNode as Node;
    Ok(match node {
        Node::AbilityTag { tag } => ModifierFilter::AbilityTag(tag.clone().into_boxed_str()),
        Node::DamageTag { tag } => ModifierFilter::DamageTag(tag.clone().into_boxed_str()),
        Node::Element { element } => ModifierFilter::Element(*element as u8),
        Node::Action { action_kind } => ModifierFilter::Action(*action_kind as u8),
        Node::Life { life } => ModifierFilter::Life(match life {
            generated::life_predicate::LifePredicate::Any => LifeFilter::Any,
            generated::life_predicate::LifePredicate::Alive => LifeFilter::Alive,
            generated::life_predicate::LifePredicate::Downed => LifeFilter::Downed,
            generated::life_predicate::LifePredicate::Defeated => LifeFilter::Defeated,
        }),
        Node::Presence { presence } => ModifierFilter::Presence(match presence {
            generated::presence_predicate::PresencePredicate::Any => PresenceFilter::Any,
            generated::presence_predicate::PresencePredicate::Present => PresenceFilter::Present,
            generated::presence_predicate::PresencePredicate::Reserved => PresenceFilter::Reserved,
            generated::presence_predicate::PresencePredicate::Departed => PresenceFilter::Departed,
            generated::presence_predicate::PresencePredicate::Untargetable => {
                PresenceFilter::Untargetable
            }
            generated::presence_predicate::PresencePredicate::Linked => PresenceFilter::Linked,
            generated::presence_predicate::PresencePredicate::Transformed => {
                PresenceFilter::Transformed
            }
        }),
        Node::Source { source_class } => ModifierFilter::Source(source_class_value(*source_class)),
        Node::Target { target_selector_id } => {
            ModifierFilter::Target(selector(*target_selector_id)?)
        }
    })
}

fn stat(value: stat_kind::StatKind) -> StatKind {
    use stat_kind::StatKind as V;
    match value {
        V::Hp => StatKind::Hp,
        V::Atk => StatKind::Atk,
        V::Def => StatKind::Def,
        V::Spd => StatKind::Spd,
        V::CritRate => StatKind::CritRate,
        V::CritDamage => StatKind::CritDamage,
        V::EffectHitRate => StatKind::EffectHitRate,
        V::EffectResistance => StatKind::EffectResistance,
        V::BreakEffect => StatKind::BreakEffect,
        V::EnergyRegenerationRate => StatKind::EnergyRegenerationRate,
        V::OutgoingHealing => StatKind::OutgoingHealing,
        V::IncomingHealing => StatKind::IncomingHealing,
        V::ShieldStrength => StatKind::ShieldStrength,
        V::Aggro => StatKind::Aggro,
        V::ToughnessDamage => StatKind::ToughnessDamage,
    }
}
fn stage(value: formula_stage::FormulaStage) -> FormulaStage {
    use formula_stage::FormulaStage as V;
    match value {
        V::BaseAdd => FormulaStage::BaseAdd,
        V::PercentOfBase => FormulaStage::PercentOfBase,
        V::Flat => FormulaStage::Flat,
        V::FinalAdd => FormulaStage::FinalAdd,
        V::FinalMultiply => FormulaStage::FinalMultiply,
        V::Crit => FormulaStage::Crit,
        V::DamageBoost => FormulaStage::DamageBoost,
        V::Weaken => FormulaStage::Weaken,
        V::Defense => FormulaStage::Defense,
        V::Resistance => FormulaStage::Resistance,
        V::Vulnerability => FormulaStage::Vulnerability,
        V::Mitigation => FormulaStage::Mitigation,
        V::Broken => FormulaStage::Broken,
        V::Healing => FormulaStage::Healing,
        V::Shield => FormulaStage::Shield,
        V::Probability => FormulaStage::Probability,
    }
}
fn purpose(value: formula_purpose::FormulaPurpose) -> FormulaPurpose {
    use formula_purpose::FormulaPurpose as V;
    match value {
        V::Stat => FormulaPurpose::Stat,
        V::OrdinaryDamage => FormulaPurpose::OrdinaryDamage,
        V::Dot => FormulaPurpose::Dot,
        V::Break => FormulaPurpose::Break,
        V::SuperBreak => FormulaPurpose::SuperBreak,
        V::AdditionalDamage => FormulaPurpose::AdditionalDamage,
        V::JointDamage => FormulaPurpose::JointDamage,
        V::ElationDamage => FormulaPurpose::ElationDamage,
        V::TrueDamage => FormulaPurpose::TrueDamage,
        V::Healing => FormulaPurpose::Healing,
        V::Shield => FormulaPurpose::Shield,
        V::EffectChance => FormulaPurpose::EffectChance,
        V::Aggro => FormulaPurpose::Aggro,
        V::ActionOrder => FormulaPurpose::ActionOrder,
    }
}
fn aggregation(value: modifier_aggregation::ModifierAggregation) -> ModifierAggregation {
    use modifier_aggregation::ModifierAggregation as V;
    match value {
        V::Sum => ModifierAggregation::Sum,
        V::Product => ModifierAggregation::Product,
        V::Maximum => ModifierAggregation::Maximum,
        V::Minimum => ModifierAggregation::Minimum,
        V::Latest => ModifierAggregation::Latest,
        V::Earliest => ModifierAggregation::Earliest,
        V::StrongestByComparator => ModifierAggregation::StrongestByComparator,
        V::UniquePerSource => ModifierAggregation::UniquePerSource,
        V::ReplaceGroup => ModifierAggregation::ReplaceGroup,
    }
}
fn snapshot(value: snapshot_policy::SnapshotPolicy) -> SnapshotPolicy {
    use snapshot_policy::SnapshotPolicy as V;
    match value {
        V::Dynamic => SnapshotPolicy::Dynamic,
        V::OnApplication => SnapshotPolicy::OnApplication,
        V::OnActionStart => SnapshotPolicy::OnActionStart,
        V::OnPhaseStart => SnapshotPolicy::OnPhaseStart,
        V::OnHitStart => SnapshotPolicy::OnHitStart,
        V::SourceSnapshotTargetDynamic => SnapshotPolicy::SourceSnapshotTargetDynamic,
        V::SourceDynamicTargetSnapshot => SnapshotPolicy::SourceDynamicTargetSnapshot,
        V::RecomputeOnStackChange => SnapshotPolicy::RecomputeOnStackChange,
        V::ExplicitFields => SnapshotPolicy::ExplicitFields,
    }
}
fn round(value: rounding_policy::RoundingPolicy) -> Rounding {
    use rounding_policy::RoundingPolicy as V;
    match value {
        V::Floor => Rounding::Floor,
        V::Ceil => Rounding::Ceil,
        V::TowardZero => Rounding::TowardZero,
        V::AwayFromZero => Rounding::AwayFromZero,
        V::NearestTiesAway => Rounding::NearestTiesAway,
        V::NearestTiesEven => Rounding::NearestTiesEven,
    }
}
fn source_class_value(
    value: generated::source_class::SourceClass,
) -> starclock_combat::rule::model::SourceClass {
    use generated::source_class::SourceClass as V;
    use starclock_combat::rule::model::SourceClass as D;
    match value {
        V::Unit => D::Unit,
        V::Ability => D::Ability,
        V::Effect => D::Effect,
        V::Equipment => D::Equipment,
        V::Progression => D::Progression,
        V::Enemy => D::Enemy,
        V::Encounter => D::Encounter,
        V::Activity => D::Activity,
        V::Mode => D::Mode,
        V::Synthetic => D::Synthetic,
    }
}
fn value_kind(value: generated::value_kind::ValueKind) -> Result<RuleValueKind, CatalogLoadError> {
    use generated::value_kind::ValueKind as V;
    match value {
        V::Integer => Ok(RuleValueKind::Integer),
        V::Scalar | V::Ratio | V::Probability => Ok(RuleValueKind::Scalar),
        V::StableId => Ok(RuleValueKind::StableId),
        V::Boolean => Ok(RuleValueKind::Boolean),
    }
}
fn positive(value: i32) -> Result<u32, CatalogLoadError> {
    u32::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| domain_fail("domain ID must be positive"))
}
fn modifier_id(value: i32) -> Result<ModifierDefinitionId, CatalogLoadError> {
    Ok(ModifierDefinitionId::new(positive(value)?).expect("positive ID"))
}
fn group_id(value: i32) -> Result<ModifierStackingGroupId, CatalogLoadError> {
    Ok(ModifierStackingGroupId::new(positive(value)?).expect("positive ID"))
}
fn selector(value: i32) -> Result<SelectorId, CatalogLoadError> {
    Ok(SelectorId::new(positive(value)?).expect("positive ID"))
}
