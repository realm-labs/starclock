//! Cross-reference checks for lifecycle operations and linked combatants.

use crate::{ProgramId, rule::model::RuleOperationTemplate};

use super::{CatalogBuildError, CombatCatalog, DefinitionKind, require};

pub(super) fn valid_linked_definition(
    catalog: &CombatCatalog,
    linked: &crate::LinkedUnitDefinition,
) -> bool {
    let combatant = linked.combatant();
    catalog.units.get(combatant.form()).is_some()
        && combatant
            .abilities()
            .iter()
            .all(|ability| catalog.abilities.get(*ability).is_some())
        && combatant
            .rule_bundles()
            .iter()
            .all(|bundle| catalog.rule_bundles.get(*bundle).is_some())
        && combatant
            .modifiers()
            .iter()
            .all(|modifier| catalog.modifiers.definition(*modifier).is_some())
        && linked.action_ability().is_none_or(|ability| {
            catalog
                .abilities
                .get(ability)
                .and_then(crate::catalog::definition::AbilityDefinition::action)
                .is_some_and(|action| {
                    matches!(
                        (linked.kind(), action.kind()),
                        (
                            crate::LinkedEntityKind::Summon,
                            crate::catalog::action::AbilityKind::Summon
                        ) | (
                            crate::LinkedEntityKind::Memosprite,
                            crate::catalog::action::AbilityKind::Memosprite
                        )
                    )
                })
        })
}

pub(super) fn validate_program_operation(
    catalog: &CombatCatalog,
    program: ProgramId,
    operation: &RuleOperationTemplate,
) -> Result<(), CatalogBuildError> {
    match operation {
        RuleOperationTemplate::Summon {
            unit_definition, ..
        } => require(
            catalog.linked_units.get(*unit_definition).is_some(),
            DefinitionKind::Program,
            program.get(),
            DefinitionKind::LinkedUnit,
            unit_definition.get(),
        ),
        RuleOperationTemplate::CreateCountdown { code } => require(
            catalog.countdowns.get(*code).is_some(),
            DefinitionKind::Program,
            program.get(),
            DefinitionKind::Countdown,
            *code,
        ),
        RuleOperationTemplate::QueueAction { ability, .. } => require(
            catalog
                .abilities
                .get(*ability)
                .and_then(crate::catalog::definition::AbilityDefinition::action)
                .is_some(),
            DefinitionKind::Program,
            program.get(),
            DefinitionKind::Ability,
            ability.get(),
        ),
        RuleOperationTemplate::Transform {
            replacement_definition,
            ..
        } => require(
            catalog.units.get(*replacement_definition).is_some(),
            DefinitionKind::Program,
            program.get(),
            DefinitionKind::Unit,
            replacement_definition.get(),
        ),
        RuleOperationTemplate::ReplaceAbility {
            old_ability,
            new_ability,
            ..
        } => {
            require(
                catalog.abilities.get(*old_ability).is_some(),
                DefinitionKind::Program,
                program.get(),
                DefinitionKind::Ability,
                old_ability.get(),
            )?;
            require(
                catalog.abilities.get(*new_ability).is_some(),
                DefinitionKind::Program,
                program.get(),
                DefinitionKind::Ability,
                new_ability.get(),
            )
        }
        _ => Ok(()),
    }
}
