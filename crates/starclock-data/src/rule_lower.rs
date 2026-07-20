//! Generated Sora rule/state-slot rows to executable battle Rule IR.

use std::collections::{BTreeMap, BTreeSet};

use starclock_combat::{
    NativeHandlerId, RuleId, SourceDefinitionId, StateSlotDefinitionId,
    rule::model::{
        BattleRuleDefinition, BattleRuleScope, RuleSource, RuleValue, RuleValueKind,
        SlotPersistence, SlotResetPoint, SlotVisibility, StateSlotDef, ValueExpr,
    },
};

use crate::{
    catalog::{
        CatalogLoadError, IdentityDefinition, IdentityKind, LoadMode, domain_fail, require_identity,
    },
    generated::{
        self, SoraConfig, rule_domain, rule_scope, slot_persistence, slot_reset_point,
        slot_value_kind, slot_visibility,
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
) -> Result<Vec<RuleDataDefinition>, CatalogLoadError> {
    let mut rules = config
        .rule_definition()
        .ordered_rows()
        .map(|row| lower_rule(config, row, mode, identities))
        .collect::<Result<Vec<_>, _>>()?;
    rules.sort_unstable_by_key(|rule| rule.id);
    Ok(rules)
}

fn lower_rule(
    config: &SoraConfig,
    row: &generated::rule_definition::RuleDefinition,
    mode: LoadMode,
    identities: &BTreeMap<u32, &IdentityDefinition>,
) -> Result<RuleDataDefinition, CatalogLoadError> {
    if row.domain != rule_domain::RuleDomain::Battle {
        return Err(domain_fail(format!(
            "rule {} belongs to the activity domain",
            row.id
        )));
    }
    if row.native_handler_id.is_some() {
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

    let runtime = BattleRuleDefinition::new(
        RuleSource::new(
            SourceDefinitionId::new(source).expect("positive source ID"),
            lower_source_class(row.source_class),
            Vec::new(),
            parse_digest(&row.source_digest_sha256)?,
        ),
        slots,
        Vec::new(),
        None::<NativeHandlerId>,
    );
    Ok(RuleDataDefinition {
        id: RuleId::new(raw_id).expect("positive rule ID"),
        runtime,
    })
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
