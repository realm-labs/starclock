//! Preserve immutable build inputs while appending mode-owned passive bindings.
use super::DivergentUniverseBattleAssemblyError;
use crate::digest::CanonicalDigestBuilder;
use starclock_combat::{
    CombatantSpecDigest, ParticipantSpec, ResolvedCombatantSpec, ResolvedDefinitionBindings,
    ResolvedModifierBinding, RuleBundleId, rule::model::RuleSource,
};

#[derive(Default)]
pub(super) struct PassiveBindings {
    pub(super) modifiers: Vec<ResolvedModifierBinding>,
    pub(super) rule_bundles: Vec<RuleBundleId>,
    pub(super) sources: Vec<RuleSource>,
}

pub(super) fn bind_passives(
    player: &ParticipantSpec,
    added: &PassiveBindings,
    assembly_digest: [u8; 32],
) -> Result<ParticipantSpec, DivergentUniverseBattleAssemblyError> {
    let invalid = || DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants;
    let base = player.combatant();
    let mut modifiers = base.modifiers().to_vec();
    modifiers.extend(added.modifiers.iter().map(|binding| binding.definition()));
    let mut bindings = base.modifier_bindings().to_vec();
    bindings.extend_from_slice(&added.modifiers);
    let mut bundles = base.rule_bundles().to_vec();
    bundles.extend_from_slice(&added.rule_bundles);
    let mut sources = base.sources().to_vec();
    sources.extend_from_slice(&added.sources);
    modifiers.sort_unstable();
    bindings.sort_by_key(|binding| binding.definition());
    bundles.sort_unstable();
    sources.sort_by_key(RuleSource::definition);
    // Do not deduplicate collisions: the typed spec/catalog must reject them.
    let mut digest = CanonicalDigestBuilder::new();
    digest.update(b"starclock.divergent-universe.curio-attached-combatant");
    digest.update(assembly_digest);
    digest.update(base.digest().bytes());
    for modifier in &modifiers {
        digest.update(modifier.get().to_le_bytes());
    }
    digest.update(b"rule-bundles");
    for bundle in &bundles {
        digest.update(bundle.get().to_le_bytes());
    }
    let combatant = ResolvedCombatantSpec::new(
        base.form(),
        base.level(),
        base.maximum_hp(),
        base.speed(),
        ResolvedDefinitionBindings::new(base.abilities().to_vec(), bundles, modifiers)
            .map_err(|_| invalid())?,
        CombatantSpecDigest::new(digest.finalize()).ok_or_else(invalid)?,
    )
    .map_err(|_| invalid())?
    .with_base_attack_defense(base.base_attack(), base.base_defense())
    .with_base_effect_stats(base.base_effect_hit_rate(), base.base_effect_resistance())
    .with_build_bonuses(base.build_bonuses())
    .with_energy(base.current_energy(), base.maximum_energy())
    .map_err(|_| invalid())?
    .with_toughness(
        base.rank(),
        base.weaknesses().to_vec(),
        base.toughness_layers().to_vec(),
    )
    .map_err(|_| invalid())?
    .with_sources(sources)
    .map_err(|_| invalid())?
    .with_modifier_bindings(bindings)
    .map_err(|_| invalid())?;
    let mut output = ParticipantSpec::new(
        player.side(),
        player.formation(),
        player.source(),
        combatant,
    )
    .with_wave(player.wave())
    .ok_or_else(invalid)?
    .with_locked_combatant_digest(player.locked_combatant_digest());
    if let Some(initial) = player.initial_state() {
        output = output.with_initial_state(initial).ok_or_else(invalid)?;
    }
    Ok(output)
}
