//! Linked-unit and countdown lowering from generated Sora rows.

use std::collections::BTreeMap;

use starclock_combat::{
    AbilityId, ActionGauge, CombatantSpecDigest, CountdownCatalogDefinition, CountdownDefinition,
    FormationIndex, Hp, LinkedEntityKind, LinkedOwnerScaling, LinkedStatScaling,
    LinkedUnitCatalogDefinition, LinkedUnitDefinition, OwnerLinkPolicy, PresenceState, Ratio,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, Scalar, SourceDefinitionId, Speed,
    UnitDefinitionId, UnitLevel, WaveLinkPolicy,
};

use crate::{
    catalog::{
        CatalogLoadError, IdentityDefinition, IdentityKind, LoadMode, domain_fail, parse_decimal,
        positive, require_identity, valid_sha256,
    },
    generated::{
        SoraConfig, linked_unit_kind, owner_link_policy, presence_state, wave_link_policy,
    },
};

type LifecycleDefinitions = (
    Box<[LinkedUnitCatalogDefinition]>,
    Box<[CountdownCatalogDefinition]>,
);

pub(super) fn lower(
    config: &SoraConfig,
    identities: &BTreeMap<u32, &IdentityDefinition>,
    mode: LoadMode,
) -> Result<LifecycleDefinitions, CatalogLoadError> {
    let linked = config
        .linked_unit_definition()
        .ordered_rows()
        .map(|row| lower_linked(row, identities, mode))
        .collect::<Result<Vec<_>, _>>()?;
    let countdowns = config
        .countdown_definition()
        .ordered_rows()
        .map(lower_countdown)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((linked.into_boxed_slice(), countdowns.into_boxed_slice()))
}

fn lower_linked(
    row: &crate::generated::linked_unit_definition::LinkedUnitDefinition,
    identities: &BTreeMap<u32, &IdentityDefinition>,
    mode: LoadMode,
) -> Result<LinkedUnitCatalogDefinition, CatalogLoadError> {
    let raw = positive(row.id, "LinkedUnitDefinition.id")?;
    require_identity(identities, raw, IdentityKind::Character, mode)?;
    let source = positive(
        row.source_definition_identity_id,
        "LinkedUnitDefinition.source_definition_identity_id",
    )?;
    let mut abilities = row
        .ability_ids
        .iter()
        .map(|value| {
            AbilityId::new(positive(*value, "LinkedUnitDefinition.ability_ids")?)
                .ok_or_else(|| domain_fail("linked ability ID is zero"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    abilities.sort_unstable();
    if abilities.is_empty() || abilities.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(domain_fail("linked abilities are empty or duplicated"));
    }
    let action = row
        .action_ability_id
        .map(|value| {
            AbilityId::new(positive(value, "LinkedUnitDefinition.action_ability_id")?)
                .ok_or_else(|| domain_fail("linked action ability ID is zero"))
        })
        .transpose()?;
    let bindings =
        ResolvedDefinitionBindings::new(abilities, Vec::new(), Vec::new()).map_err(domain_fail)?;
    let digest = CombatantSpecDigest::new(decode_sha256(&row.combatant_digest_sha256)?)
        .ok_or_else(|| domain_fail("linked combatant digest cannot be all zero"))?;
    let form = UnitDefinitionId::new(raw).expect("positive linked definition ID");
    let prototype = ResolvedCombatantSpec::new(
        form,
        UnitLevel::new(80).expect("level 80 is valid"),
        Hp::new(1).expect("one HP is valid"),
        Speed::from_scaled(1_000_000).expect("one Speed is valid"),
        bindings,
        digest,
    )
    .map_err(domain_fail)?;
    let definition = LinkedUnitDefinition::new(
        prototype,
        SourceDefinitionId::new(source).expect("positive linked source ID"),
        FormationIndex::new(
            u8::try_from(row.formation_index)
                .map_err(|_| domain_fail("linked formation index exceeds u8"))?,
        )
        .ok_or_else(|| domain_fail("linked formation index exceeds battle domain"))?,
        linked_kind(row.kind),
        presence(row.presence),
        action,
        ActionGauge::from_scaled(parse_decimal(&row.initial_gauge_decimal)?)
            .map_err(domain_fail)?,
        owner_policy(row.owner_defeat_policy),
        owner_policy(row.owner_departure_policy),
        wave_policy(row.wave_policy),
    )
    .ok_or_else(|| domain_fail("invalid linked-unit lifecycle definition"))?
    .with_owner_scaling(LinkedOwnerScaling::new(
        scaling(&row.hp_owner_ratio_decimal, &row.hp_flat_decimal)?,
        scaling(&row.atk_owner_ratio_decimal, &row.atk_flat_decimal)?,
        scaling(&row.def_owner_ratio_decimal, &row.def_flat_decimal)?,
        scaling(&row.spd_owner_ratio_decimal, &row.spd_flat_decimal)?,
    ));
    LinkedUnitCatalogDefinition::new(form, definition)
        .ok_or_else(|| domain_fail("linked unit form differs from its catalog ID"))
}

fn lower_countdown(
    row: &crate::generated::countdown_definition::CountdownDefinition,
) -> Result<CountdownCatalogDefinition, CatalogLoadError> {
    let code = positive(row.code, "CountdownDefinition.code")?;
    let ability = AbilityId::new(positive(row.ability_id, "CountdownDefinition.ability_id")?)
        .expect("positive countdown ability ID");
    let mut definition = CountdownDefinition::new(
        ability,
        ActionGauge::from_scaled(parse_decimal(&row.initial_gauge_decimal)?)
            .map_err(domain_fail)?,
        Speed::from_scaled(parse_decimal(&row.speed_decimal)?).map_err(domain_fail)?,
        owner_policy(row.owner_defeat_policy),
        owner_policy(row.owner_departure_policy),
        wave_policy(row.wave_policy),
    );
    if row.end_transformation {
        definition = definition.with_end_transformation();
    }
    CountdownCatalogDefinition::new(code, definition)
        .ok_or_else(|| domain_fail("countdown code is zero"))
}

fn scaling(ratio: &str, flat: &str) -> Result<LinkedStatScaling, CatalogLoadError> {
    Ok(LinkedStatScaling::new(
        Ratio::from_scaled(parse_decimal(ratio)?),
        Scalar::from_scaled(parse_decimal(flat)?),
    ))
}

fn linked_kind(value: linked_unit_kind::LinkedUnitKind) -> LinkedEntityKind {
    match value {
        linked_unit_kind::LinkedUnitKind::Summon => LinkedEntityKind::Summon,
        linked_unit_kind::LinkedUnitKind::Memosprite => LinkedEntityKind::Memosprite,
        linked_unit_kind::LinkedUnitKind::SharedActor => LinkedEntityKind::SharedActor,
    }
}

fn presence(value: presence_state::PresenceState) -> PresenceState {
    match value {
        presence_state::PresenceState::Present => PresenceState::Present,
        presence_state::PresenceState::Reserved => PresenceState::Reserved,
        presence_state::PresenceState::Departed => PresenceState::Departed,
        presence_state::PresenceState::Untargetable => PresenceState::Untargetable,
        presence_state::PresenceState::Linked => PresenceState::Linked,
        presence_state::PresenceState::Transformed => PresenceState::Transformed,
    }
}

fn owner_policy(value: owner_link_policy::OwnerLinkPolicy) -> OwnerLinkPolicy {
    match value {
        owner_link_policy::OwnerLinkPolicy::Persist => OwnerLinkPolicy::Persist,
        owner_link_policy::OwnerLinkPolicy::Depart => OwnerLinkPolicy::Depart,
        owner_link_policy::OwnerLinkPolicy::Defeat => OwnerLinkPolicy::Defeat,
    }
}

fn wave_policy(value: wave_link_policy::WaveLinkPolicy) -> WaveLinkPolicy {
    match value {
        wave_link_policy::WaveLinkPolicy::Persist => WaveLinkPolicy::Persist,
        wave_link_policy::WaveLinkPolicy::ResetGauge => WaveLinkPolicy::ResetGauge,
        wave_link_policy::WaveLinkPolicy::Depart => WaveLinkPolicy::Depart,
    }
}

fn decode_sha256(value: &str) -> Result<[u8; 32], CatalogLoadError> {
    if !valid_sha256(value) {
        return Err(domain_fail(
            "linked combatant digest is not lowercase SHA-256",
        ));
    }
    let mut output = [0; 32];
    for (index, byte) in output.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .map_err(|_| domain_fail("linked combatant digest is invalid"))?;
    }
    Ok(output)
}
