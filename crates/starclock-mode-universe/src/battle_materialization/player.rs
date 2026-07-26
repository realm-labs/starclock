//! Player participant assembly from locked builds, mode contributions and carry.

use starclock_activity::ActivityParticipantCarryState;
use starclock_combat::{
    CombatantSpecDigest, Energy, ParticipantInitialState, ParticipantSource, ParticipantSpec,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, ResolvedModifierBinding, TeamSide,
};

use crate::{
    ability_runtime::AbilityTarget,
    battle_contribution::UniverseBattleContributionSet,
    battle_rule_lowering::{RESONANCE_ABILITY_ID, RuleAttachment},
    battle_technique::CompiledUniverseBattleTechnique,
    catalog::UniverseCatalog,
};

use super::{
    UniverseBattleMaterializationError, UniverseBattleRoster, UniverseBattleRosterEntry,
    materialization_digest::combatant_digest,
};

pub(super) fn player_participants(
    universe: &UniverseCatalog,
    roster: &UniverseBattleRoster,
    contributions: &UniverseBattleContributionSet,
    technique: Option<&CompiledUniverseBattleTechnique>,
    carry: &[ActivityParticipantCarryState],
) -> Result<Vec<ParticipantSpec>, UniverseBattleMaterializationError> {
    roster
        .entries()
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let base = projected_eidolon_combatant(universe, entry, contributions)?;
            let projected_energy = projected_party_energy(&base, contributions)?;
            let mut participant = ParticipantSpec::new(
                TeamSide::Player,
                entry.formation(),
                ParticipantSource::Player,
                apply_party_modifiers(
                    &base,
                    contributions,
                    index == 0,
                    technique.filter(|technique| {
                        technique.definition().participant() == entry.participant()
                    }),
                )?,
            )
            .with_locked_combatant_digest(entry.combatant().digest());
            if let Some(state) = carry
                .iter()
                .find(|state| state.participant() == entry.participant())
            {
                let initial = ParticipantInitialState::new(
                    state.current_hp(),
                    state.maximum_hp(),
                    projected_energy.unwrap_or(state.current_energy()),
                    state.maximum_energy(),
                    state.life(),
                    state.presence(),
                )
                .ok_or(UniverseBattleMaterializationError::InvalidCarry)?;
                participant = participant
                    .with_initial_state(initial)
                    .ok_or(UniverseBattleMaterializationError::InvalidCarry)?;
            }
            Ok(participant)
        })
        .collect()
}

fn projected_eidolon_combatant(
    universe: &UniverseCatalog,
    entry: &UniverseBattleRosterEntry,
    contributions: &UniverseBattleContributionSet,
) -> Result<ResolvedCombatantSpec, UniverseBattleMaterializationError> {
    let levels = contributions.eidolon_resonance_levels();
    if levels == 0 {
        return Ok(entry.combatant().clone());
    }
    let spec = entry
        .build_spec()
        .ok_or(UniverseBattleMaterializationError::MissingBuildSelection)?;
    let core = universe.simulation_catalog();
    let compiled = starclock_build::compiler::LoadoutCompiler
        .compile(core.build_catalog(), core.combat_catalog(), spec)
        .map_err(|_| UniverseBattleMaterializationError::InvalidBuildSelection)?;
    if compiled.combatant().digest() != entry.combatant().digest()
        || compiled.build_digest().bytes() != entry.build_digest().bytes()
    {
        return Err(UniverseBattleMaterializationError::InvalidBuildSelection);
    }
    let effective = spec
        .eidolon()
        .get()
        .saturating_add(levels)
        .min(starclock_build::spec::EidolonLevel::MAX);
    let enhanced = spec.clone().with_eidolon(
        starclock_build::spec::EidolonLevel::new(effective)
            .expect("clamped Eidolon level is valid"),
    );
    starclock_build::compiler::LoadoutCompiler
        .compile(core.build_catalog(), core.combat_catalog(), &enhanced)
        .map(|compiled| compiled.combatant().clone())
        .map_err(|_| UniverseBattleMaterializationError::InvalidBuildSelection)
}

fn apply_party_modifiers(
    base: &ResolvedCombatantSpec,
    contributions: &UniverseBattleContributionSet,
    first_player: bool,
    technique: Option<&CompiledUniverseBattleTechnique>,
) -> Result<ResolvedCombatantSpec, UniverseBattleMaterializationError> {
    let mut modifier_ids = base.modifiers().to_vec();
    modifier_ids.extend(
        contributions
            .modifiers()
            .iter()
            .map(|binding| binding.definition().id),
    );
    modifier_ids.sort_unstable();
    if modifier_ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(UniverseBattleMaterializationError::ContributionCollision);
    }
    let mut sources = base.sources().to_vec();
    sources.extend(
        contributions
            .modifiers()
            .iter()
            .map(|binding| binding.source().clone()),
    );
    sources.sort_unstable_by_key(|source| source.definition());
    if sources
        .windows(2)
        .any(|pair| pair[0].definition() == pair[1].definition())
    {
        return Err(UniverseBattleMaterializationError::ContributionCollision);
    }
    let mut modifier_bindings = base.modifier_bindings().to_vec();
    modifier_bindings.extend(contributions.modifiers().iter().map(|binding| {
        ResolvedModifierBinding::new(binding.definition().id, binding.source().definition())
    }));
    modifier_bindings.sort_unstable_by_key(|binding| binding.definition());
    let mut rule_bundles = base.rule_bundles().to_vec();
    rule_bundles.extend(
        contributions
            .executable_rules()
            .iter()
            .filter(|rule| {
                rule.attachment() == RuleAttachment::EveryPlayer
                    || first_player && rule.attachment() == RuleAttachment::FirstPlayer
            })
            .map(|rule| rule.bundle().id()),
    );
    if let Some(technique) = technique {
        rule_bundles.push(technique.bundle().id());
    }
    rule_bundles.sort_unstable();
    if rule_bundles.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(UniverseBattleMaterializationError::ContributionCollision);
    }
    let mut abilities = base.abilities().to_vec();
    if first_player && let Some(resonance) = contributions.resonance() {
        abilities.push(RESONANCE_ABILITY_ID);
        abilities.extend(
            resonance
                .auxiliary_abilities()
                .iter()
                .map(starclock_combat::catalog::definition::AbilityDefinition::id),
        );
    }
    abilities.sort_unstable();
    if abilities.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(UniverseBattleMaterializationError::ContributionCollision);
    }
    let digest = combatant_digest(base, contributions, technique);
    let mut resolved = ResolvedCombatantSpec::new(
        base.form(),
        base.level(),
        base.maximum_hp(),
        base.speed(),
        ResolvedDefinitionBindings::new(abilities, rule_bundles, modifier_ids)
            .map_err(|_| UniverseBattleMaterializationError::InvalidCombatant)?,
        CombatantSpecDigest::new(digest).expect("SHA-256 digest is non-zero"),
    )
    .map_err(|_| UniverseBattleMaterializationError::InvalidCombatant)?
    .with_base_attack_defense(base.base_attack(), base.base_defense())
    .with_energy(
        projected_party_energy(base, contributions)?.unwrap_or(base.current_energy()),
        base.maximum_energy(),
    )
    .map_err(|_| UniverseBattleMaterializationError::InvalidCombatant)?
    .with_toughness(
        base.rank(),
        base.weaknesses().to_vec(),
        base.toughness_layers().to_vec(),
    )
    .map_err(|_| UniverseBattleMaterializationError::InvalidCombatant)?;
    resolved = resolved
        .with_sources(sources)
        .map_err(|_| UniverseBattleMaterializationError::InvalidCombatant)?;
    resolved
        .with_modifier_bindings(modifier_bindings)
        .map_err(|_| UniverseBattleMaterializationError::InvalidCombatant)
}

fn projected_party_energy(
    base: &ResolvedCombatantSpec,
    contributions: &UniverseBattleContributionSet,
) -> Result<Option<Energy>, UniverseBattleMaterializationError> {
    let ratio = [
        AbilityTarget::PartyEnergy,
        AbilityTarget::PartyInitialEnergy,
    ]
    .into_iter()
    .find_map(|target| {
        contributions
            .boundary_values()
            .iter()
            .find(|value| value.target() == target)
            .map(|value| value.value().raw_six_decimal())
    });
    let Some(ratio) = ratio else {
        return Ok(None);
    };
    if !(0..=1_000_000).contains(&ratio) {
        return Err(UniverseBattleMaterializationError::InvalidCombatant);
    }
    let scaled = base
        .maximum_energy()
        .scaled()
        .checked_mul(ratio)
        .and_then(|value| value.checked_div(1_000_000))
        .ok_or(UniverseBattleMaterializationError::InvalidCombatant)?;
    Energy::from_scaled(scaled)
        .map(Some)
        .map_err(|_| UniverseBattleMaterializationError::InvalidCombatant)
}
