//! Generated Sora rule/state-slot rows to executable battle Rule IR.

use std::collections::{BTreeMap, BTreeSet};

use starclock_combat::{
    NativeHandlerId, ProgramId, RuleId, SourceDefinitionId, StateSlotDefinitionId, TriggerId,
    rule::model::{
        BattleRuleDefinition, BattleRuleScope, CauseAncestry, Comparison, ConditionExpr,
        EventFilter, EventValueProperty, OnceScope, ReactionPriority, RuleActionKind,
        RuleDamageClass, RuleEventPoint, RuleResourceKind, RuleSource, RuleValue, RuleValueKind,
        SlotPersistence, SlotResetPoint, SlotVisibility, StateSlotDef, TriggerDef, TriggerPhase,
        ValueExpr,
    },
};

use crate::{
    catalog::{
        CatalogLoadError, IdentityDefinition, IdentityKind, LoadMode, domain_fail, require_identity,
    },
    generated::{
        self, SoraConfig, comparison_operator, condition_expression_node, event_pattern,
        once_scope, rule_domain, rule_scope, slot_persistence, slot_reset_point, slot_value_kind,
        slot_visibility, trigger_phase,
    },
};

#[derive(Debug)]
pub(super) struct RuleDataDefinition {
    pub(super) id: RuleId,
    pub(super) runtime: BattleRuleDefinition,
}

pub(super) fn convert(
    config: &SoraConfig,
    mode: LoadMode,
    identities: &BTreeMap<u32, &IdentityDefinition>,
    native_handlers: &BTreeSet<NativeHandlerId>,
) -> Result<Vec<RuleDataDefinition>, CatalogLoadError> {
    let mut rules = config
        .rule_definition()
        .ordered_rows()
        .map(|row| lower_rule(config, row, mode, identities, native_handlers))
        .collect::<Result<Vec<_>, _>>()?;
    rules.sort_unstable_by_key(|rule| rule.id);
    Ok(rules)
}

fn lower_rule(
    config: &SoraConfig,
    row: &generated::rule_definition::RuleDefinition,
    mode: LoadMode,
    identities: &BTreeMap<u32, &IdentityDefinition>,
    native_handlers: &BTreeSet<NativeHandlerId>,
) -> Result<RuleDataDefinition, CatalogLoadError> {
    if row.domain != rule_domain::RuleDomain::Battle {
        return Err(domain_fail(format!(
            "rule {} belongs to the activity domain",
            row.id
        )));
    }
    let native_handler = row
        .native_handler_id
        .map(crate::native_handler_lower::handler_id)
        .transpose()?;
    if native_handler.is_some_and(|handler| !native_handlers.contains(&handler)) {
        return Err(domain_fail(format!(
            "rule {} requires an unregistered native handler",
            row.id
        )));
    }
    let raw_id = positive(row.id, "RuleDefinition.id")?;
    require_identity(identities, raw_id, IdentityKind::Other, mode)?;
    let source = positive(
        row.source_definition_identity_id,
        "RuleDefinition.source_definition_identity_id",
    )?;
    if !identities.contains_key(&source) {
        return Err(domain_fail(format!(
            "rule {} refers to missing source identity {source}",
            row.id
        )));
    }
    let tags = config
        .rule_source_tag()
        .iter()
        .filter(|tag| tag.rule_id == row.id)
        .collect::<Vec<_>>();
    if !tags.is_empty() {
        return Err(domain_fail(format!(
            "rule {} uses string source tags before the stable tag registry boundary",
            row.id
        )));
    }

    let mut slots = config
        .state_slot()
        .ordered_rows()
        .filter(|slot| slot.rule_id == row.id)
        .map(|slot| lower_slot(config, slot))
        .collect::<Result<Vec<_>, _>>()?;
    slots.sort_unstable_by_key(StateSlotDef::id);
    let mut trigger_rows = config
        .rule_trigger()
        .ordered_rows()
        .filter(|trigger| trigger.rule_id == row.id)
        .collect::<Vec<_>>();
    trigger_rows.sort_unstable_by_key(|trigger| trigger.sequence);
    for (offset, trigger) in trigger_rows.iter().enumerate() {
        if trigger.sequence != i32::try_from(offset + 1).expect("trigger bound fits i32") {
            return Err(domain_fail(format!(
                "rule {} has noncontiguous trigger order",
                row.id
            )));
        }
    }
    let triggers = trigger_rows
        .into_iter()
        .map(|trigger| lower_trigger(config, trigger))
        .collect::<Result<Vec<_>, _>>()?;

    let runtime = BattleRuleDefinition::new(
        RuleSource::new(
            SourceDefinitionId::new(source).expect("positive source ID"),
            lower_source_class(row.source_class),
            Vec::new(),
            parse_digest(&row.source_digest_sha256)?,
        ),
        slots,
        triggers,
        native_handler,
    );
    Ok(RuleDataDefinition {
        id: RuleId::new(raw_id).expect("positive rule ID"),
        runtime,
    })
}

fn lower_trigger(
    config: &SoraConfig,
    row: &generated::rule_trigger::RuleTrigger,
) -> Result<TriggerDef, CatalogLoadError> {
    let filter = config
        .event_filter()
        .get(&row.filter_id)
        .ok_or_else(|| domain_fail(format!("missing event filter {}", row.filter_id)))?;
    let source = filter
        .source_definition_identity_id
        .map(|id| {
            SourceDefinitionId::new(positive(id, "EventFilter.source_definition_identity_id")?)
                .ok_or_else(|| domain_fail("event-filter source ID is zero"))
        })
        .transpose()?;
    if config.program().get(&row.program_id).is_none() {
        return Err(domain_fail(format!(
            "rule trigger {} refers to missing program {}",
            row.id, row.program_id
        )));
    }
    let event_point = lower_event(&row.event)?;
    Ok(TriggerDef {
        id: TriggerId::new(positive(row.id, "RuleTrigger.id")?).expect("positive trigger ID"),
        event: event_point.kind(),
        event_point,
        phase: lower_trigger_phase(row.phase),
        filter: EventFilter {
            source,
            source_class: filter.source_class.map(lower_source_class),
            owner_selector: filter.owner_selector_id.map(selector).transpose()?,
            actor_selector: filter.actor_selector_id.map(selector).transpose()?,
            applier_selector: filter.applier_selector_id.map(selector).transpose()?,
            target_selector: filter.target_selector_id.map(selector).transpose()?,
            action_kind: filter.action_kind.map(lower_action_kind),
            ability_tag: filter
                .ability_tag
                .as_deref()
                .map(lower_ability_tag)
                .transpose()?,
            element: filter.element.map(crate::effect_lower::lower_element),
            damage_class: filter.damage_class.map(lower_damage_class).transpose()?,
            resource: lower_filter_resource(
                filter.resource_kind,
                filter.character_resource_key.as_deref(),
            )?,
            cause_ancestry: lower_ancestry(filter.cause_ancestry),
            ..EventFilter::default()
        },
        condition: lower_condition(config, row.condition_id, &mut BTreeSet::new())?,
        once_scope: lower_once_scope(row.once_scope)?,
        priority: ReactionPriority::new(i16::try_from(row.priority).map_err(|_| {
            domain_fail(format!("rule trigger {} priority does not fit i16", row.id))
        })?),
        program: ProgramId::new(positive(row.program_id, "RuleTrigger.program_id")?)
            .expect("positive program ID"),
    })
}

fn lower_event(value: &event_pattern::EventPattern) -> Result<RuleEventPoint, CatalogLoadError> {
    use event_pattern::EventPattern as V;
    Ok(match value {
        V::Battle { point } => match point {
            generated::battle_event_point::BattleEventPoint::Started => {
                RuleEventPoint::BattleStarted
            }
            generated::battle_event_point::BattleEventPoint::Won => RuleEventPoint::BattleWon,
            generated::battle_event_point::BattleEventPoint::Lost => RuleEventPoint::BattleLost,
            generated::battle_event_point::BattleEventPoint::Faulted => {
                RuleEventPoint::BattleFaulted
            }
        },
        V::Wave { point } => boundary(
            *point,
            RuleEventPoint::WaveStarted,
            RuleEventPoint::WaveEnded,
        ),
        V::Turn { point } => boundary(
            *point,
            RuleEventPoint::TurnStarted,
            RuleEventPoint::TurnEnded,
        ),
        V::Action { point } => match point {
            generated::action_event_point::ActionEventPoint::Declared => {
                RuleEventPoint::ActionDeclared
            }
            generated::action_event_point::ActionEventPoint::Started => {
                RuleEventPoint::ActionStarted
            }
            generated::action_event_point::ActionEventPoint::Resolved => {
                RuleEventPoint::ActionResolved
            }
        },
        V::Hit { point } => boundary(*point, RuleEventPoint::HitStarted, RuleEventPoint::HitEnded),
        V::Damage { point } => match point {
            generated::damage_event_point::DamageEventPoint::Calculated => {
                RuleEventPoint::DamageCalculated
            }
            generated::damage_event_point::DamageEventPoint::Applied => {
                RuleEventPoint::DamageApplied
            }
        },
        V::Effect { point } => match point {
            generated::effect_event_point::EffectEventPoint::Applied => {
                RuleEventPoint::EffectApplied
            }
            generated::effect_event_point::EffectEventPoint::Removed => {
                RuleEventPoint::EffectRemoved
            }
            generated::effect_event_point::EffectEventPoint::Refreshed => {
                RuleEventPoint::EffectRefreshed
            }
            generated::effect_event_point::EffectEventPoint::StacksChanged => {
                RuleEventPoint::EffectStacksChanged
            }
        },
        V::Unit { point } => match point {
            generated::unit_event_point::UnitEventPoint::Downed => RuleEventPoint::UnitDowned,
            generated::unit_event_point::UnitEventPoint::Defeated => RuleEventPoint::UnitDefeated,
            generated::unit_event_point::UnitEventPoint::Revived => RuleEventPoint::UnitRevived,
            generated::unit_event_point::UnitEventPoint::Transformed => {
                RuleEventPoint::UnitTransformed
            }
        },
        V::EncounterTransition {} => RuleEventPoint::EncounterTransition,
        V::TimelineChanged {} => RuleEventPoint::TimelineChanged,
        V::HpChanged {} => RuleEventPoint::HpChanged,
        V::ToughnessChanged {} => RuleEventPoint::ToughnessChanged,
        V::WeaknessBroken {} => RuleEventPoint::WeaknessBroken,
        V::HealApplied {} => RuleEventPoint::HealApplied,
        V::ShieldChanged {} => RuleEventPoint::ShieldChanged,
        V::ResourceChanged {} => RuleEventPoint::ResourceChanged,
        V::PresenceChanged {} => RuleEventPoint::PresenceChanged,
        V::RuleStateChanged {} => RuleEventPoint::RuleStateChanged,
        V::InformationalRule {} => RuleEventPoint::InformationalRule,
        V::DecisionRequested {} => RuleEventPoint::DecisionRequested,
        V::FaultRaised {} => RuleEventPoint::FaultRaised,
    })
}

fn boundary(
    point: generated::boundary_event_point::BoundaryEventPoint,
    started: RuleEventPoint,
    ended: RuleEventPoint,
) -> RuleEventPoint {
    match point {
        generated::boundary_event_point::BoundaryEventPoint::Started => started,
        generated::boundary_event_point::BoundaryEventPoint::Ended => ended,
    }
}

fn selector(value: i32) -> Result<starclock_combat::SelectorId, CatalogLoadError> {
    Ok(
        starclock_combat::SelectorId::new(positive(value, "EventFilter.selector")?)
            .expect("positive selector ID"),
    )
}

fn lower_action_kind(value: generated::action_kind::ActionKind) -> RuleActionKind {
    use generated::action_kind::ActionKind as V;
    match value {
        V::Basic => RuleActionKind::Basic,
        V::Skill => RuleActionKind::Skill,
        V::Ultimate => RuleActionKind::Ultimate,
        V::Talent => RuleActionKind::Talent,
        V::TechniqueEntry => RuleActionKind::TechniqueEntry,
        V::FollowUp => RuleActionKind::FollowUp,
        V::Counter => RuleActionKind::Counter,
        V::Summon => RuleActionKind::Summon,
        V::Memosprite => RuleActionKind::Memosprite,
        V::Enemy => RuleActionKind::Enemy,
        V::ExtraTurn => RuleActionKind::ExtraTurn,
        V::Scripted => RuleActionKind::Scripted,
    }
}

fn lower_ability_tag(
    value: &str,
) -> Result<starclock_combat::catalog::action::AbilityTag, CatalogLoadError> {
    use starclock_combat::catalog::action::AbilityTag as V;
    match value {
        "Attack" => Ok(V::Attack),
        "Basic" => Ok(V::Basic),
        "Skill" => Ok(V::Skill),
        "Ultimate" => Ok(V::Ultimate),
        "FollowUp" => Ok(V::FollowUp),
        "Counter" => Ok(V::Counter),
        "Summon" => Ok(V::Summon),
        "Memosprite" => Ok(V::Memosprite),
        "AdditionalDamage" => Ok(V::AdditionalDamage),
        "Joint" => Ok(V::Joint),
        "ElationSkill" => Ok(V::ElationSkill),
        _ => Err(domain_fail(format!("unknown ability tag {value}"))),
    }
}

fn lower_damage_class(
    value: generated::damage_class::DamageClass,
) -> Result<RuleDamageClass, CatalogLoadError> {
    use generated::damage_class::DamageClass as V;
    Ok(match value {
        V::Ordinary => RuleDamageClass::Ordinary,
        V::Dot => RuleDamageClass::Dot,
        V::Break => RuleDamageClass::Break,
        V::SuperBreak => RuleDamageClass::SuperBreak,
        V::Additional => RuleDamageClass::Additional,
        V::Joint => RuleDamageClass::Joint,
        V::Elation => RuleDamageClass::Elation,
        V::TrueDamage => RuleDamageClass::TrueDamage,
    })
}

fn lower_filter_resource(
    kind: Option<generated::resource_kind::ResourceKind>,
    key: Option<&str>,
) -> Result<Option<RuleResourceKind>, CatalogLoadError> {
    use generated::resource_kind::ResourceKind as V;
    match (kind, key) {
        (None, None) => Ok(None),
        (Some(V::Energy), None) => Ok(Some(RuleResourceKind::Energy)),
        (Some(V::SkillPoints), None) => Ok(Some(RuleResourceKind::SkillPoints)),
        (Some(V::CharacterResource), Some(key)) => {
            Ok(Some(RuleResourceKind::Character(key.into())))
        }
        (Some(V::TeamResource), Some(key)) => Ok(Some(RuleResourceKind::Team(key.into()))),
        _ => Err(domain_fail(
            "event-filter resource kind/key combination is invalid",
        )),
    }
}

fn lower_ancestry(value: generated::cause_ancestry::CauseAncestry) -> CauseAncestry {
    use generated::cause_ancestry::CauseAncestry as V;
    match value {
        V::Any => CauseAncestry::Any,
        V::RootCommand => CauseAncestry::RootCommand,
        V::DirectParent => CauseAncestry::DirectParent,
        V::SameAction => CauseAncestry::SameAction,
        V::SamePhase => CauseAncestry::SamePhase,
        V::SameHit => CauseAncestry::SameHit,
    }
}

fn lower_trigger_phase(value: trigger_phase::TriggerPhase) -> TriggerPhase {
    use trigger_phase::TriggerPhase as V;
    match value {
        V::Before => TriggerPhase::Before,
        V::Replace => TriggerPhase::Replace,
        V::AfterMutation => TriggerPhase::AfterMutation,
        V::AfterDefeatSettlement => TriggerPhase::AfterDefeatSettlement,
        V::AfterEvent => TriggerPhase::AfterEvent,
        V::AfterAction => TriggerPhase::AfterAction,
        V::Boundary => TriggerPhase::Boundary,
    }
}

fn lower_once_scope(value: once_scope::OnceScope) -> Result<OnceScope, CatalogLoadError> {
    use once_scope::OnceScope as V;
    Ok(match value {
        V::Event => OnceScope::Event,
        V::Hit => OnceScope::Hit,
        V::TargetWithinHit => OnceScope::TargetWithinHit,
        V::Ability => OnceScope::Ability,
        V::Action => OnceScope::Action,
        V::Turn => OnceScope::Turn,
        V::Wave => OnceScope::Wave,
        V::Battle => OnceScope::Battle,
        V::Attempt | V::Node | V::Section | V::Activity => {
            return Err(domain_fail(
                "activity once-scope entered the combat boundary",
            ));
        }
    })
}

pub(super) fn lower_condition(
    config: &SoraConfig,
    id: i32,
    visiting: &mut BTreeSet<i32>,
) -> Result<ConditionExpr, CatalogLoadError> {
    if !visiting.insert(id) {
        return Err(domain_fail(format!("condition expression {id} is cyclic")));
    }
    let row = config
        .condition_expression()
        .get(&id)
        .ok_or_else(|| domain_fail(format!("missing condition expression {id}")))?;
    use condition_expression_node::ConditionExpressionNode as V;
    let condition = match &row.node {
        V::Constant { value } => ConditionExpr::Literal(*value),
        V::Compare {
            left_expression_id,
            comparison,
            right_expression_id,
        } => ConditionExpr::Compare {
            lhs: Box::new(crate::modifier_lower::expression(
                config,
                *left_expression_id,
                &mut BTreeSet::new(),
            )?),
            operator: lower_comparison(*comparison),
            rhs: Box::new(crate::modifier_lower::expression(
                config,
                *right_expression_id,
                &mut BTreeSet::new(),
            )?),
        },
        V::All { condition_ids } => ConditionExpr::All(
            condition_ids
                .iter()
                .map(|id| lower_condition(config, *id, visiting))
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice(),
        ),
        V::Any { condition_ids } => ConditionExpr::Any(
            condition_ids
                .iter()
                .map(|id| lower_condition(config, *id, visiting))
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice(),
        ),
        V::Not { condition_id } => {
            ConditionExpr::Not(Box::new(lower_condition(config, *condition_id, visiting)?))
        }
        V::LifePresence {
            selector_id,
            life,
            presence,
        } => ConditionExpr::LifePresence {
            selector: selector(*selector_id)?,
            life: lower_life_predicate(*life),
            presence: lower_presence_predicate(*presence),
        },
        V::ResourceBounds {
            resource_expression_id,
            minimum_expression_id,
            maximum_expression_id,
        } => ConditionExpr::All(
            vec![
                ConditionExpr::Compare {
                    lhs: Box::new(crate::modifier_lower::expression(
                        config,
                        *resource_expression_id,
                        &mut BTreeSet::new(),
                    )?),
                    operator: Comparison::GreaterOrEqual,
                    rhs: Box::new(crate::modifier_lower::expression(
                        config,
                        *minimum_expression_id,
                        &mut BTreeSet::new(),
                    )?),
                },
                ConditionExpr::Compare {
                    lhs: Box::new(crate::modifier_lower::expression(
                        config,
                        *resource_expression_id,
                        &mut BTreeSet::new(),
                    )?),
                    operator: Comparison::LessOrEqual,
                    rhs: Box::new(crate::modifier_lower::expression(
                        config,
                        *maximum_expression_id,
                        &mut BTreeSet::new(),
                    )?),
                },
            ]
            .into_boxed_slice(),
        ),
        V::EffectExists {
            selector_id,
            effect_id,
        } => ConditionExpr::EffectExists {
            selector: selector(*selector_id)?,
            effect: starclock_combat::EffectDefinitionId::new(positive(
                *effect_id,
                "ConditionExpression.effect_id",
            )?)
            .expect("positive effect ID"),
        },
        V::HasWeakness {
            selector_id,
            element,
        } => ConditionExpr::HasWeakness {
            selector: selector(*selector_id)?,
            element: crate::effect_lower::lower_element(*element),
        },
        V::IsBroken { selector_id } => ConditionExpr::IsBroken(selector(*selector_id)?),
        V::SelectorCardinality {
            selector_id,
            minimum_count,
            maximum_count,
        } => ConditionExpr::All(
            vec![
                ConditionExpr::SelectorCardinality {
                    selector: selector(*selector_id)?,
                    operator: Comparison::GreaterOrEqual,
                    count: u16::try_from(*minimum_count)
                        .map_err(|_| domain_fail("selector minimum does not fit u16"))?,
                },
                ConditionExpr::SelectorCardinality {
                    selector: selector(*selector_id)?,
                    operator: Comparison::LessOrEqual,
                    count: u16::try_from(*maximum_count)
                        .map_err(|_| domain_fail("selector maximum does not fit u16"))?,
                },
            ]
            .into_boxed_slice(),
        ),
        V::EventPropertyCompare {
            property,
            comparison,
            expected_expression_id,
        } => ConditionExpr::Compare {
            lhs: Box::new(ValueExpr::ReadEventProperty(lower_event_property(
                *property,
            ))),
            operator: lower_comparison(*comparison),
            rhs: Box::new(crate::modifier_lower::expression(
                config,
                *expected_expression_id,
                &mut BTreeSet::new(),
            )?),
        },
        _ => {
            return Err(domain_fail(format!(
                "condition expression {id} exceeds the current executable boundary"
            )));
        }
    };
    visiting.remove(&id);
    Ok(condition)
}

fn lower_life_predicate(
    value: generated::life_predicate::LifePredicate,
) -> Option<starclock_combat::LifeState> {
    use generated::life_predicate::LifePredicate as V;
    match value {
        V::Any => None,
        V::Alive => Some(starclock_combat::LifeState::Alive),
        V::Downed => Some(starclock_combat::LifeState::Downed),
        V::Defeated => Some(starclock_combat::LifeState::Defeated),
    }
}

fn lower_presence_predicate(
    value: generated::presence_predicate::PresencePredicate,
) -> Option<starclock_combat::PresenceState> {
    use generated::presence_predicate::PresencePredicate as V;
    match value {
        V::Any => None,
        V::Present => Some(starclock_combat::PresenceState::Present),
        V::Reserved => Some(starclock_combat::PresenceState::Reserved),
        V::Departed => Some(starclock_combat::PresenceState::Departed),
        V::Untargetable => Some(starclock_combat::PresenceState::Untargetable),
        V::Linked => Some(starclock_combat::PresenceState::Linked),
        V::Transformed => Some(starclock_combat::PresenceState::Transformed),
    }
}

pub(super) fn lower_event_property(
    value: generated::event_value_property::EventValueProperty,
) -> EventValueProperty {
    use generated::event_value_property::EventValueProperty as V;
    match value {
        V::OwnerId => EventValueProperty::OwnerId,
        V::ActorId => EventValueProperty::ActorId,
        V::ApplierId => EventValueProperty::ApplierId,
        V::SourceDefinitionId => EventValueProperty::SourceDefinitionId,
        V::PrimaryTargetId => EventValueProperty::PrimaryTargetId,
        V::DamageAmount => EventValueProperty::DamageAmount,
        V::HpChangeAmount => EventValueProperty::HpChangeAmount,
        V::ResourceDelta => EventValueProperty::ResourceDelta,
        V::StackCount => EventValueProperty::StackCount,
        V::HitIndex => EventValueProperty::HitIndex,
    }
}

fn lower_comparison(value: comparison_operator::ComparisonOperator) -> Comparison {
    use comparison_operator::ComparisonOperator as V;
    match value {
        V::Equal => Comparison::Equal,
        V::NotEqual => Comparison::NotEqual,
        V::Less => Comparison::Less,
        V::LessOrEqual => Comparison::LessOrEqual,
        V::Greater => Comparison::Greater,
        V::GreaterOrEqual => Comparison::GreaterOrEqual,
    }
}

fn lower_slot(
    config: &SoraConfig,
    row: &generated::state_slot::StateSlot,
) -> Result<StateSlotDef, CatalogLoadError> {
    let id = StateSlotDefinitionId::new(positive(row.id, "StateSlot.id")?)
        .expect("positive state-slot ID");
    let kind = lower_value_kind(row.value_kind);
    let initial = literal(config, row.initial_expression_id, "initial")?;
    let minimum = row
        .minimum_expression_id
        .map(|expr| literal(config, expr, "minimum"))
        .transpose()?;
    let maximum = row
        .maximum_expression_id
        .map(|expr| literal(config, expr, "maximum"))
        .transpose()?;
    for (name, value) in [
        ("initial", Some(&initial)),
        ("minimum", minimum.as_ref()),
        ("maximum", maximum.as_ref()),
    ] {
        if value.is_some_and(|value| value.kind() != kind) {
            return Err(domain_fail(format!(
                "state slot {} {name} value does not match its declared kind",
                row.id
            )));
        }
    }
    if !bounds_contain(&initial, minimum.as_ref(), maximum.as_ref()) {
        return Err(domain_fail(format!(
            "state slot {} initial value is outside its bounds",
            row.id
        )));
    }
    let mut reset_rows = config
        .state_slot_reset()
        .iter()
        .filter(|reset| reset.state_slot_id == row.id)
        .collect::<Vec<_>>();
    reset_rows.sort_unstable_by_key(|reset| reset.sequence);
    for (offset, reset) in reset_rows.iter().enumerate() {
        if reset.sequence != i32::try_from(offset + 1).expect("reset bound fits i32") {
            return Err(domain_fail(format!(
                "state slot {} has noncontiguous reset order",
                row.id
            )));
        }
    }
    let reset_points = reset_rows
        .into_iter()
        .map(|reset| lower_reset(reset.reset_point))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(
        StateSlotDef::new(id, kind, lower_scope(row.owner_scope)?, initial)
            .with_optional_bounds(minimum, maximum)
            .with_policy(
                lower_visibility(row.visibility),
                lower_persistence(row.persistence),
            )
            .with_reset_points(reset_points),
    )
}

fn literal(
    config: &SoraConfig,
    expression_id: i32,
    role: &str,
) -> Result<RuleValue, CatalogLoadError> {
    match crate::modifier_lower::expression(config, expression_id, &mut BTreeSet::new())? {
        ValueExpr::Literal(value) => Ok(value),
        _ => Err(domain_fail(format!(
            "state-slot {role} expression {expression_id} is contextual"
        ))),
    }
}

fn bounds_contain(
    initial: &RuleValue,
    minimum: Option<&RuleValue>,
    maximum: Option<&RuleValue>,
) -> bool {
    match initial {
        RuleValue::Integer(value) => {
            minimum.is_none_or(|bound| matches!(bound, RuleValue::Integer(bound) if value >= bound))
                && maximum.is_none_or(
                    |bound| matches!(bound, RuleValue::Integer(bound) if value <= bound),
                )
        }
        RuleValue::Scalar(value) => {
            minimum.is_none_or(|bound| matches!(bound, RuleValue::Scalar(bound) if value >= bound))
                && maximum
                    .is_none_or(|bound| matches!(bound, RuleValue::Scalar(bound) if value <= bound))
        }
        _ => minimum.is_none() && maximum.is_none(),
    }
}

fn lower_value_kind(value: slot_value_kind::SlotValueKind) -> RuleValueKind {
    use slot_value_kind::SlotValueKind as V;
    match value {
        V::BoundedInteger => RuleValueKind::Integer,
        V::FixedScalar => RuleValueKind::Scalar,
        V::Boolean => RuleValueKind::Boolean,
        V::StableId => RuleValueKind::StableId,
        V::OptionalId => RuleValueKind::OptionalStableId,
        V::OrderedIdSet => RuleValueKind::OrderedStableIdSet,
    }
}

fn lower_scope(value: rule_scope::RuleScope) -> Result<BattleRuleScope, CatalogLoadError> {
    use rule_scope::RuleScope as V;
    Ok(match value {
        V::Battle => BattleRuleScope::Battle,
        V::Wave => BattleRuleScope::Wave,
        V::Turn => BattleRuleScope::Turn,
        V::Action => BattleRuleScope::Action,
        V::Hit => BattleRuleScope::Hit,
        V::Activity | V::Section | V::Node | V::Attempt => {
            return Err(domain_fail(
                "activity-owned state slot entered the combat boundary",
            ));
        }
    })
}

fn lower_reset(
    value: slot_reset_point::SlotResetPoint,
) -> Result<SlotResetPoint, CatalogLoadError> {
    use slot_reset_point::SlotResetPoint as V;
    Ok(match value {
        V::BattleStart => SlotResetPoint::BattleStart,
        V::WaveStart => SlotResetPoint::WaveStart,
        V::TurnStart => SlotResetPoint::TurnStart,
        V::ActionStart => SlotResetPoint::ActionStart,
        V::HitStart => SlotResetPoint::HitStart,
        V::TurnEnd => SlotResetPoint::TurnEnd,
        V::ActionEnd => SlotResetPoint::ActionEnd,
        V::WaveEnd => SlotResetPoint::WaveEnd,
        V::BattleEnd => SlotResetPoint::BattleEnd,
        V::ActivityStart | V::SectionStart | V::NodeStart | V::AttemptStart => {
            return Err(domain_fail(
                "activity-owned reset entered the combat boundary",
            ));
        }
    })
}

fn lower_visibility(value: slot_visibility::SlotVisibility) -> SlotVisibility {
    use slot_visibility::SlotVisibility as V;
    match value {
        V::Private => SlotVisibility::Private,
        V::Owner => SlotVisibility::Owner,
        V::Team => SlotVisibility::Team,
        V::Public => SlotVisibility::Public,
    }
}

fn lower_persistence(value: slot_persistence::SlotPersistence) -> SlotPersistence {
    use slot_persistence::SlotPersistence as V;
    match value {
        V::OwnerLifetime => SlotPersistence::OwnerLifetime,
        V::ScopeLifetime => SlotPersistence::ScopeLifetime,
        V::ExplicitReset => SlotPersistence::ExplicitReset,
    }
}

fn lower_source_class(
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

fn parse_digest(value: &str) -> Result<[u8; 32], CatalogLoadError> {
    if value.len() != 64 {
        return Err(domain_fail("rule source digest is not lowercase SHA-256"));
    }
    let mut digest = [0_u8; 32];
    for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
        let pair = std::str::from_utf8(chunk)
            .ok()
            .and_then(|pair| u8::from_str_radix(pair, 16).ok())
            .ok_or_else(|| domain_fail("rule source digest is not lowercase SHA-256"))?;
        if chunk.iter().any(|byte| byte.is_ascii_uppercase()) {
            return Err(domain_fail("rule source digest is not lowercase SHA-256"));
        }
        digest[index] = pair;
    }
    Ok(digest)
}

fn positive(value: i32, field: &str) -> Result<u32, CatalogLoadError> {
    u32::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| domain_fail(format!("{field} must be positive")))
}
