use sha2::{Digest, Sha256};
use starclock_combat::{
    CombatantSpecDigest, ModifierDefinitionId, ResolvedCombatantSpec, ResolvedDefinitionBindings,
    ResolvedModifierBinding, RuleBundleId, rule::model::RuleSource,
};

use super::{CurrencyWarsBattleAssemblyError, debug_error, error};

pub(crate) fn attach_rule_bundle(
    base: &ResolvedCombatantSpec,
    bundle: RuleBundleId,
    domain: &[u8],
    overlay_digest: [u8; 32],
) -> Result<ResolvedCombatantSpec, CurrencyWarsBattleAssemblyError> {
    let mut bundles = base.rule_bundles().to_vec();
    match bundles.binary_search(&bundle) {
        Ok(_) => return Err(error("Currency Wars rule bundle is duplicated")),
        Err(index) => bundles.insert(index, bundle),
    }
    let mut hash = Sha256::new();
    hash.update(domain);
    hash.update(base.digest().bytes());
    hash.update(overlay_digest);
    let digest = CombatantSpecDigest::new(hash.finalize().into())
        .ok_or_else(|| error("Currency Wars combatant overlay digest is zero"))?;
    ResolvedCombatantSpec::new(
        base.form(),
        base.level(),
        base.maximum_hp(),
        base.speed(),
        ResolvedDefinitionBindings::new(
            base.abilities().to_vec(),
            bundles,
            base.modifiers().to_vec(),
        )
        .map_err(debug_error)?,
        digest,
    )
    .map_err(debug_error)
    .map(|value| {
        value
            .with_base_attack_defense(base.base_attack(), base.base_defense())
            .with_base_effect_stats(base.base_effect_hit_rate(), base.base_effect_resistance())
            .with_build_bonuses(base.build_bonuses())
    })?
    .with_energy(base.current_energy(), base.maximum_energy())
    .and_then(|value| {
        value.with_toughness(
            base.rank(),
            base.weaknesses().to_vec(),
            base.toughness_layers().to_vec(),
        )
    })
    .and_then(|value| value.with_sources(base.sources().to_vec()))
    .and_then(|value| value.with_modifier_bindings(base.modifier_bindings().to_vec()))
    .map_err(debug_error)
}

pub(crate) fn attach_modifier(
    base: &ResolvedCombatantSpec,
    modifier: ModifierDefinitionId,
    source: RuleSource,
    domain: &[u8],
    overlay_digest: [u8; 32],
) -> Result<ResolvedCombatantSpec, CurrencyWarsBattleAssemblyError> {
    attach_modifier_with_linked_subjects(base, modifier, source, domain, overlay_digest, false)
}

pub(crate) fn attach_modifier_to_linked_subjects(
    base: &ResolvedCombatantSpec,
    modifier: ModifierDefinitionId,
    source: RuleSource,
    domain: &[u8],
    overlay_digest: [u8; 32],
) -> Result<ResolvedCombatantSpec, CurrencyWarsBattleAssemblyError> {
    attach_modifier_with_linked_subjects(base, modifier, source, domain, overlay_digest, true)
}

fn attach_modifier_with_linked_subjects(
    base: &ResolvedCombatantSpec,
    modifier: ModifierDefinitionId,
    source: RuleSource,
    domain: &[u8],
    overlay_digest: [u8; 32],
    linked_subjects: bool,
) -> Result<ResolvedCombatantSpec, CurrencyWarsBattleAssemblyError> {
    let mut modifiers = base.modifiers().to_vec();
    let index = modifiers
        .binary_search(&modifier)
        .map_or_else(|index| index, |_| usize::MAX);
    if index == usize::MAX {
        return Err(error("Currency Wars modifier overlay is duplicated"));
    }
    modifiers.insert(index, modifier);
    let mut sources = base.sources().to_vec();
    match sources.binary_search_by_key(&source.definition(), RuleSource::definition) {
        Ok(index) if sources[index] != source => {
            return Err(error("Currency Wars modifier source identity conflicts"));
        }
        Ok(_) => {}
        Err(index) => sources.insert(index, source.clone()),
    }
    let mut bindings = base.modifier_bindings().to_vec();
    let binding = ResolvedModifierBinding::new(modifier, source.definition());
    bindings.insert(
        index,
        if linked_subjects {
            binding.with_linked_subjects()
        } else {
            binding
        },
    );
    let mut hash = Sha256::new();
    hash.update(domain);
    hash.update(base.digest().bytes());
    hash.update(overlay_digest);
    let digest = CombatantSpecDigest::new(hash.finalize().into())
        .ok_or_else(|| error("Currency Wars combatant modifier digest is zero"))?;
    ResolvedCombatantSpec::new(
        base.form(),
        base.level(),
        base.maximum_hp(),
        base.speed(),
        ResolvedDefinitionBindings::new(
            base.abilities().to_vec(),
            base.rule_bundles().to_vec(),
            modifiers,
        )
        .map_err(debug_error)?,
        digest,
    )
    .map_err(debug_error)
    .map(|value| {
        value
            .with_base_attack_defense(base.base_attack(), base.base_defense())
            .with_base_effect_stats(base.base_effect_hit_rate(), base.base_effect_resistance())
            .with_build_bonuses(base.build_bonuses())
    })?
    .with_energy(base.current_energy(), base.maximum_energy())
    .and_then(|value| {
        value.with_toughness(
            base.rank(),
            base.weaknesses().to_vec(),
            base.toughness_layers().to_vec(),
        )
    })
    .and_then(|value| value.with_sources(sources))
    .and_then(|value| value.with_modifier_bindings(bindings))
    .map_err(debug_error)
}

pub(crate) fn attach_source_tag(
    base: &ResolvedCombatantSpec,
    source: RuleSource,
    domain: &[u8],
    overlay_digest: [u8; 32],
) -> Result<ResolvedCombatantSpec, CurrencyWarsBattleAssemblyError> {
    let mut sources = base.sources().to_vec();
    match sources.binary_search_by_key(&source.definition(), RuleSource::definition) {
        Ok(index) if sources[index] != source => {
            return Err(error("Currency Wars source tag identity conflicts"));
        }
        Ok(_) => return Ok(base.clone()),
        Err(index) => sources.insert(index, source),
    }
    let mut hash = Sha256::new();
    hash.update(domain);
    hash.update(base.digest().bytes());
    hash.update(overlay_digest);
    let digest = CombatantSpecDigest::new(hash.finalize().into())
        .ok_or_else(|| error("Currency Wars combatant source tag digest is zero"))?;
    ResolvedCombatantSpec::new(
        base.form(),
        base.level(),
        base.maximum_hp(),
        base.speed(),
        ResolvedDefinitionBindings::new(
            base.abilities().to_vec(),
            base.rule_bundles().to_vec(),
            base.modifiers().to_vec(),
        )
        .map_err(debug_error)?,
        digest,
    )
    .map_err(debug_error)
    .map(|value| {
        value
            .with_base_attack_defense(base.base_attack(), base.base_defense())
            .with_base_effect_stats(base.base_effect_hit_rate(), base.base_effect_resistance())
            .with_build_bonuses(base.build_bonuses())
    })?
    .with_energy(base.current_energy(), base.maximum_energy())
    .and_then(|value| {
        value.with_toughness(
            base.rank(),
            base.weaknesses().to_vec(),
            base.toughness_layers().to_vec(),
        )
    })
    .and_then(|value| value.with_sources(sources))
    .and_then(|value| value.with_modifier_bindings(base.modifier_bindings().to_vec()))
    .map_err(debug_error)
}
