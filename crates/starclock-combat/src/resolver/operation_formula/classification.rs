//! Operation-entry damage labels derived from immutable definitions and captured effects.

use super::{FormulaInputs, damage_purpose, formula_source, invariant_fault};
use crate::{
    AbilityId,
    battle::fault::BattleFault,
    catalog::{CombatCatalog, action::AbilityTag, definition::AbilityDefinition},
    damage::{DamageProducer, DamageSemantic, DamageSemantics},
    event::cause::{Cause, CauseActor},
    formula::model::DamageClass,
    modifier::model::FormulaPurpose,
    resolver::transaction::Transaction,
};

impl FormulaInputs {
    pub(in crate::resolver) fn damage_semantics(
        &self,
        catalog: &CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        class: DamageClass,
        ultimate_semantics: bool,
    ) -> Result<DamageSemantics, BattleFault> {
        if class == DamageClass::Dot {
            return Ok(DamageSemantics::NONE);
        }
        let source = formula_source(txn, cause, damage_purpose(class))?;
        let producer = damage_producer(txn, cause, damage_purpose(class))?;
        let native_follow_up = class == DamageClass::Direct
            && !ultimate_semantics
            && cause
                .source_definition()
                .and_then(|id| AbilityId::new(id.get()))
                .and_then(|id| catalog.ability(id))
                .and_then(AbilityDefinition::action)
                .is_some_and(|action| action.tags().contains(AbilityTag::FollowUp));
        let mut labels = if native_follow_up {
            DamageSemantics::new(DamageSemantic::FollowUp)
        } else {
            DamageSemantics::NONE
        };
        for ((holder, definition), stacks) in &self.effect_stacks {
            if *holder != source || *stacks == 0 {
                continue;
            }
            if let Some(entry) = catalog.effect(*definition).and_then(|effect| {
                effect.damage_classifications().iter().find(|entry| {
                    entry.class == class
                        && entry.producer.is_none_or(|required| required == producer)
                })
            }) {
                labels = labels.union(entry.semantics);
            }
        }
        Ok(labels)
    }
}

pub(super) fn damage_producer(
    txn: &Transaction<'_>,
    cause: Cause,
    purpose: FormulaPurpose,
) -> Result<DamageProducer, BattleFault> {
    let actor = if matches!(
        purpose,
        FormulaPurpose::Dot | FormulaPurpose::Break | FormulaPurpose::SuperBreak
    ) && cause.applier().is_some()
    {
        cause.applier().map(CauseActor::Unit)
    } else {
        cause.actor()
    };
    let unit = match actor {
        Some(CauseActor::Unit(unit)) => unit,
        Some(CauseActor::TimelineActor(actor)) => {
            let actor = txn
                .state
                .actors
                .get(actor)
                .ok_or_else(|| invariant_fault(43))?;
            let Some(unit) = actor.unit else {
                return Ok(DamageProducer::UnitlessActor);
            };
            unit
        }
        None => formula_source(txn, cause, purpose)?,
    };
    Ok(if txn.state.links.for_unit(unit).is_some() {
        DamageProducer::LinkedUnit
    } else {
        DamageProducer::OriginalUnit
    })
}
