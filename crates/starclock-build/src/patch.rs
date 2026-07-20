//! Closed typed build-patch language shared by Traces and Eidolons.

use starclock_combat::{AbilityId, ModifierDefinitionId, RuleBundleId};

/// Build-time changes that can currently be lowered into generic combat
/// definition bindings.
///
/// Resource, state-slot, tag, and phase-program edits remain outside this
/// executable subset until their combat-domain output exists. They must not be
/// approximated by mutating immutable catalog definitions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildPatch {
    AddAbility(AbilityId),
    AddRuleBundle(RuleBundleId),
    RemoveRuleBundle(RuleBundleId),
    AddModifier(ModifierDefinitionId),
    ReplaceAbility {
        old: AbilityId,
        new: AbilityId,
    },
    AdjustAbilityLevel {
        family: AbilityId,
        bonus: i8,
        cap_delta: i8,
    },
}
