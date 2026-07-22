//! Strict lowering for encounters, difficulty bindings and content pools.

use std::collections::BTreeMap;

use crate::definition::DomainKind;
use crate::encounter::{
    BossPhasePolicy, ContentPoolDefinition, ContentPoolEntry, ContentPoolKind,
    DifficultyEnemyBinding, EncounterEnemySlot, EncounterGroupDefinition,
    EncounterMemberDefinition, EncounterPoolDefinition, EncounterSelectionPolicy,
    EncounterWaveDefinition, EnemyRole, FixedEncounterBinding, RoomContentBinding, RoomContentKind,
    WavePolicy, WeightedEncounterBinding,
};
use crate::error::UniverseCatalogLoadError;
use crate::generated::{
    SoraConfig, universe_boss_phase_policy::UniverseBossPhasePolicy,
    universe_coverage_state::UniverseCoverageState, universe_domain_kind::UniverseDomainKind,
    universe_enemy_role::UniverseEnemyRole, universe_mode_owner::UniverseModeOwner,
    universe_pool_kind::UniversePoolKind, universe_room_content_kind::UniverseRoomContentKind,
    universe_selection_policy::UniverseSelectionPolicy, universe_wave_policy::UniverseWavePolicy,
};
use crate::id::{
    ContentPoolId, DifficultyId, EncounterGroupId, EncounterMemberId, EncounterPoolId,
    EncounterWaveId, RoomId,
};
use crate::lowering::{checked_key, checked_source, invalid, localized, reference};
use crate::path_lowering::parse_decimal;

pub(crate) struct EncounterDefinitions {
    pub(crate) groups: Box<[EncounterGroupDefinition]>,
    pub(crate) difficulty_enemies: Box<[DifficultyEnemyBinding]>,
    pub(crate) pools: Box<[EncounterPoolDefinition]>,
    pub(crate) room_content: Box<[RoomContentBinding]>,
    pub(crate) content_pools: Box<[ContentPoolDefinition]>,
}

pub(crate) fn lower(config: &SoraConfig) -> Result<EncounterDefinitions, UniverseCatalogLoadError> {
    validate_evidence(config)?;
    let groups = lower_groups(config)?;
    let difficulty_enemies = lower_difficulty_enemies(config)?;
    let pools = lower_encounter_pools(config)?;
    let room_content = lower_room_content(config)?;
    let content_pools = lower_content_pools(config)?;
    Ok(EncounterDefinitions {
        groups,
        difficulty_enemies,
        pools,
        room_content,
        content_pools,
    })
}

fn validate_evidence(config: &SoraConfig) -> Result<(), UniverseCatalogLoadError> {
    for row in config.universe_content_audit().ordered_rows() {
        if !row.enabled
            || row.mode_owner != UniverseModeOwner::Standard
            || row.coverage_state != UniverseCoverageState::DataReady
            || row.provenance_ids.is_empty()
        {
            return Err(invalid(
                "Universe content evidence is not enabled Standard DataReady content",
            ));
        }
        checked_key(&row.content_stable_key, "Content evidence stable key")?;
        checked_source(&row.source_file, "Content evidence source file")?;
        for source in &row.provenance_ids {
            if config.universe_source_record().get(source).is_none() {
                return Err(reference(
                    "Content evidence provenance reference is unresolved",
                ));
            }
        }
        for source in row.source_ids.as_deref().unwrap_or_default() {
            checked_source(source, "Content evidence source ID")?;
        }
        if row.note.as_deref().is_some_and(|value| value.len() > 2_048) {
            return Err(invalid("Content evidence note is too long"));
        }
    }
    if config.universe_content_audit().len() != 2_201 {
        return Err(reference("Content evidence denominator differs"));
    }
    Ok(())
}

fn lower_groups(
    config: &SoraConfig,
) -> Result<Box<[EncounterGroupDefinition]>, UniverseCatalogLoadError> {
    let enemies = group_wave_enemies(config)?;
    let waves = group_waves(config, &enemies)?;
    let members = group_members(config, &waves)?;
    let mut definitions = Vec::with_capacity(config.universe_encounter_group().len());
    for row in config.universe_encounter_group().ordered_rows() {
        let values = members.get(&row.id).cloned().unwrap_or_default();
        if values.is_empty() {
            return Err(reference("Encounter group has no member"));
        }
        definitions.push(EncounterGroupDefinition::new(
            group_id(row.id, "Encounter group")?,
            checked_key(&row.stable_key, "Encounter group stable key")?,
            checked_source(&row.source_group_id, "Encounter source group ID")?,
            match row.wave_policy {
                UniverseWavePolicy::SingleWave => WavePolicy::SingleWave,
                UniverseWavePolicy::AuthoredSequentialWaves => WavePolicy::AuthoredSequentialWaves,
            },
            match row.boss_phase_policy {
                UniverseBossPhasePolicy::EnemyAuthoredLifecycle => {
                    BossPhasePolicy::EnemyAuthoredLifecycle
                }
            },
            localized(
                &row.name_en,
                &row.name_zh_cn,
                &row.summary_en,
                &row.summary_zh_cn,
                "Encounter group",
            )?,
            values.into_boxed_slice(),
        ));
    }
    definitions.sort_by_key(EncounterGroupDefinition::id);
    if definitions.len() != 74
        || config.universe_encounter_member().len() != 173
        || config.universe_encounter_wave().len() != 173
        || config.universe_encounter_wave_enemy().len() != 538
    {
        return Err(reference(
            "Encounter group/member/wave/enemy denominator differs",
        ));
    }
    Ok(definitions.into_boxed_slice())
}

fn group_wave_enemies(
    config: &SoraConfig,
) -> Result<BTreeMap<i32, Vec<EncounterEnemySlot>>, UniverseCatalogLoadError> {
    let mut groups = BTreeMap::new();
    for row in config.universe_encounter_wave_enemy().iter() {
        if config.universe_encounter_wave().get(&row.wave_id).is_none() {
            return Err(reference("Encounter enemy slot wave is unresolved"));
        }
        checked_key(&row.enemy_variant_stable_key, "Encounter enemy variant key")?;
        groups.entry(row.wave_id).or_insert_with(Vec::new).push((
            sequence(row.sequence, "Encounter enemy slot")?,
            EncounterEnemySlot::new(
                checked_source(&row.slot, "Encounter enemy slot")?,
                checked_source(&row.source_monster_id, "Encounter source monster ID")?,
                &row.enemy_variant_stable_key,
            ),
        ));
    }
    ordered(groups, "Encounter enemy slot")
}

fn group_waves(
    config: &SoraConfig,
    enemies: &BTreeMap<i32, Vec<EncounterEnemySlot>>,
) -> Result<BTreeMap<i32, Vec<EncounterWaveDefinition>>, UniverseCatalogLoadError> {
    let mut groups = BTreeMap::new();
    for row in config.universe_encounter_wave().ordered_rows() {
        if config
            .universe_encounter_member()
            .get(&row.member_id)
            .is_none()
        {
            return Err(reference("Encounter wave member is unresolved"));
        }
        let slots = enemies.get(&row.id).cloned().unwrap_or_default();
        if slots.is_empty() {
            return Err(reference("Encounter wave has no enemy slot"));
        }
        groups.entry(row.member_id).or_insert_with(Vec::new).push((
            sequence(row.sequence, "Encounter wave")?,
            EncounterWaveDefinition::new(wave_id(row.id)?, slots.into_boxed_slice()),
        ));
    }
    ordered(groups, "Encounter wave")
}

fn group_members(
    config: &SoraConfig,
    waves: &BTreeMap<i32, Vec<EncounterWaveDefinition>>,
) -> Result<BTreeMap<i32, Vec<EncounterMemberDefinition>>, UniverseCatalogLoadError> {
    let mut groups = BTreeMap::new();
    for row in config.universe_encounter_member().ordered_rows() {
        if config
            .universe_encounter_group()
            .get(&row.group_id)
            .is_none()
        {
            return Err(reference("Encounter member group is unresolved"));
        }
        let weight = parse_decimal(&row.weight_decimal)?;
        if weight.coefficient() <= 0 {
            return Err(invalid("Encounter member weight must be positive"));
        }
        let authored_waves = waves.get(&row.id).cloned().unwrap_or_default();
        if authored_waves.is_empty() {
            return Err(reference("Encounter member has no wave"));
        }
        groups.entry(row.group_id).or_insert_with(Vec::new).push((
            sequence(row.sequence, "Encounter member")?,
            EncounterMemberDefinition::new(
                member_id(row.id)?,
                checked_source(&row.source_rogue_monster_id, "Rogue monster ID")?,
                checked_source(&row.source_primary_monster_id, "Primary monster ID")?,
                checked_source(&row.source_stage_id, "Encounter stage ID")?,
                weight,
                positive(row.stage_level, "Encounter stage level")?,
                positive(row.hard_level_group, "Encounter hard-level group")?,
                source_list(
                    row.stage_ability_ids.as_deref(),
                    "Encounter stage ability ID",
                )?,
                row.drop_type
                    .as_deref()
                    .map(|value| checked_source(value, "Encounter drop type").map(Into::into))
                    .transpose()?,
                authored_waves.into_boxed_slice(),
            ),
        ));
    }
    ordered(groups, "Encounter member")
}

fn lower_difficulty_enemies(
    config: &SoraConfig,
) -> Result<Box<[DifficultyEnemyBinding]>, UniverseCatalogLoadError> {
    let mut grouped = BTreeMap::new();
    for row in config.universe_difficulty_enemy().iter() {
        if config
            .universe_difficulty()
            .get(&row.difficulty_id)
            .is_none()
        {
            return Err(reference("Difficulty enemy parent is unresolved"));
        }
        checked_key(
            &row.enemy_variant_stable_key,
            "Difficulty enemy variant key",
        )?;
        grouped
            .entry(row.difficulty_id)
            .or_insert_with(Vec::new)
            .push((
                sequence(row.sequence, "Difficulty enemy")?,
                DifficultyEnemyBinding::new(
                    difficulty_id(row.difficulty_id)?,
                    match row.role {
                        UniverseEnemyRole::Boss => EnemyRole::Boss,
                        UniverseEnemyRole::Elite => EnemyRole::Elite,
                    },
                    checked_source(&row.source_monster_id, "Difficulty source monster ID")?,
                    &row.enemy_variant_stable_key,
                    positive(row.level, "Difficulty enemy level")?,
                ),
            ));
    }
    let values = ordered(grouped, "Difficulty enemy")?
        .into_values()
        .flatten()
        .collect::<Vec<_>>();
    if values.len() != 182 {
        return Err(reference("Difficulty enemy denominator differs"));
    }
    Ok(values.into_boxed_slice())
}

fn lower_encounter_pools(
    config: &SoraConfig,
) -> Result<Box<[EncounterPoolDefinition]>, UniverseCatalogLoadError> {
    let fixed = group_fixed(config)?;
    let weighted = group_weighted(config)?;
    let mut definitions = Vec::new();
    for row in config.universe_encounter_pool().ordered_rows() {
        if config.universe_room().get(&row.room_id).is_none() {
            return Err(reference("Encounter pool room is unresolved"));
        }
        definitions.push(EncounterPoolDefinition::new(
            pool_id(row.id)?,
            checked_key(&row.stable_key, "Encounter pool stable key")?,
            room_id(row.room_id)?,
            domain_kind(row.domain_kind),
            checked_source(&row.map_entrance, "Encounter map entrance")?,
            match row.selection_policy {
                UniverseSelectionPolicy::SelectExactConditionKeyThenWeightedStableOrder => {
                    EncounterSelectionPolicy::ExactConditionThenWeightedStableOrder
                }
                UniverseSelectionPolicy::ResolveWorldDifficultyBossEliteBinding => {
                    EncounterSelectionPolicy::WorldDifficultyBossEliteBinding
                }
                UniverseSelectionPolicy::SelectConditionKeyThenResolveGroupOrDifficultyBinding => {
                    EncounterSelectionPolicy::ConditionThenGroupOrDifficultyBinding
                }
            },
            checked_key(
                &row.source_primary_condition_key,
                "Encounter primary condition",
            )?,
            localized(
                &row.name_en,
                &row.name_zh_cn,
                &row.summary_en,
                &row.summary_zh_cn,
                "Encounter pool",
            )?,
            fixed
                .get(&row.id)
                .cloned()
                .unwrap_or_default()
                .into_boxed_slice(),
            weighted
                .get(&row.id)
                .cloned()
                .unwrap_or_default()
                .into_boxed_slice(),
        ));
    }
    definitions.sort_by_key(EncounterPoolDefinition::id);
    if definitions.len() != 92
        || config.universe_encounter_pool_fixed().len() != 36
        || config.universe_encounter_pool_group().len() != 174
    {
        return Err(reference("Encounter pool/fixed/group denominator differs"));
    }
    Ok(definitions.into_boxed_slice())
}

fn group_fixed(
    config: &SoraConfig,
) -> Result<BTreeMap<i32, Vec<FixedEncounterBinding>>, UniverseCatalogLoadError> {
    let mut groups = BTreeMap::new();
    for row in config.universe_encounter_pool_fixed().iter() {
        require_pool(config, row.pool_id)?;
        groups.entry(row.pool_id).or_insert_with(Vec::new).push((
            sequence(row.sequence, "Fixed encounter binding")?,
            FixedEncounterBinding::new(
                checked_key(&row.condition_key, "Fixed encounter condition")?,
                checked_source(&row.source_content_id, "Fixed encounter source content")?,
            ),
        ));
    }
    ordered(groups, "Fixed encounter binding")
}
fn group_weighted(
    config: &SoraConfig,
) -> Result<BTreeMap<i32, Vec<WeightedEncounterBinding>>, UniverseCatalogLoadError> {
    let mut groups = BTreeMap::new();
    for row in config.universe_encounter_pool_group().iter() {
        require_pool(config, row.pool_id)?;
        if config
            .universe_encounter_group()
            .get(&row.group_id)
            .is_none()
        {
            return Err(reference("Encounter pool group is unresolved"));
        }
        let weight = parse_decimal(&row.weight_decimal)?;
        if weight.coefficient() <= 0 {
            return Err(invalid("Encounter pool weight must be positive"));
        }
        groups.entry(row.pool_id).or_insert_with(Vec::new).push((
            sequence(row.sequence, "Weighted encounter binding")?,
            WeightedEncounterBinding::new(
                checked_key(&row.condition_key, "Weighted encounter condition")?,
                group_id(row.group_id, "Encounter pool group")?,
                weight,
            ),
        ));
    }
    ordered(groups, "Weighted encounter binding")
}

fn lower_room_content(
    config: &SoraConfig,
) -> Result<Box<[RoomContentBinding]>, UniverseCatalogLoadError> {
    let mut groups = BTreeMap::new();
    for row in config.universe_room_content().iter() {
        if config.universe_room().get(&row.room_id).is_none() {
            return Err(reference("Room content room is unresolved"));
        }
        let group = row
            .encounter_group_id
            .map(|id| {
                if config.universe_encounter_group().get(&id).is_none() {
                    return Err(reference("Room content encounter group is unresolved"));
                }
                group_id(id, "Room content encounter group")
            })
            .transpose()?;
        let kind = match row.kind {
            UniverseRoomContentKind::EncounterGroup => RoomContentKind::EncounterGroup,
            UniverseRoomContentKind::FixedContent => RoomContentKind::FixedContent,
            UniverseRoomContentKind::ExternalDecision => RoomContentKind::ExternalDecision,
        };
        if (kind == RoomContentKind::EncounterGroup) != group.is_some() {
            return Err(invalid("Room content kind/group contract is inconsistent"));
        }
        groups.entry(row.room_id).or_insert_with(Vec::new).push((
            sequence(row.sequence, "Room content")?,
            RoomContentBinding::new(
                room_id(row.room_id)?,
                checked_key(&row.condition_key, "Room content condition")?,
                checked_source(&row.source_content_id, "Room source content ID")?,
                kind,
                group,
            ),
        ));
    }
    let values = ordered(groups, "Room content")?
        .into_values()
        .flatten()
        .collect::<Vec<_>>();
    if values.len() != 380 {
        return Err(reference("Room content denominator differs"));
    }
    Ok(values.into_boxed_slice())
}

fn lower_content_pools(
    config: &SoraConfig,
) -> Result<Box<[ContentPoolDefinition]>, UniverseCatalogLoadError> {
    let mut entries = BTreeMap::new();
    for row in config.universe_content_pool_entry().iter() {
        let pool = config
            .universe_content_pool()
            .get(&row.pool_id)
            .ok_or_else(|| reference("Content pool entry parent is unresolved"))?;
        validate_content_key(config, pool.kind, &row.content_stable_key)?;
        let weight = parse_decimal(&row.weight_decimal)?;
        if weight.coefficient() <= 0 {
            return Err(invalid("Content pool entry weight must be positive"));
        }
        entries.entry(row.pool_id).or_insert_with(Vec::new).push((
            sequence(row.sequence, "Content pool entry")?,
            ContentPoolEntry::new(
                &row.content_stable_key,
                weight,
                row.condition
                    .as_deref()
                    .map(|value| checked_source(value, "Content pool condition").map(Into::into))
                    .transpose()?,
            ),
        ));
    }
    let entries = ordered(entries, "Content pool entry")?;
    let mut definitions = Vec::new();
    for row in config.universe_content_pool().ordered_rows() {
        let values = entries.get(&row.id).cloned().unwrap_or_default();
        if values.is_empty() {
            return Err(reference("Content pool has no entry"));
        }
        definitions.push(ContentPoolDefinition::new(
            content_pool_id(row.id)?,
            checked_key(&row.stable_key, "Content pool stable key")?,
            pool_kind(row.kind),
            checked_source(&row.ordering, "Content pool ordering")?,
            row.replacement,
            values.into_boxed_slice(),
        ));
    }
    definitions.sort_by_key(ContentPoolDefinition::id);
    if definitions.len() != 23 || config.universe_content_pool_entry().len() != 1_651 {
        return Err(reference("Content pool/entry denominator differs"));
    }
    Ok(definitions.into_boxed_slice())
}

fn validate_content_key(
    config: &SoraConfig,
    kind: UniversePoolKind,
    key: &str,
) -> Result<(), UniverseCatalogLoadError> {
    checked_key(key, "Content pool content key")?;
    let found = match kind {
        UniversePoolKind::Blessing => config.universe_blessing().get_by_stable_key(key).is_some(),
        UniversePoolKind::Curio => config.universe_curio().get_by_stable_key(key).is_some(),
        UniversePoolKind::Occurrence => config
            .universe_occurrence()
            .get_by_stable_key(key)
            .is_some(),
        UniversePoolKind::Encounter => config
            .universe_encounter_group()
            .get_by_stable_key(key)
            .is_some(),
        UniversePoolKind::Shop => {
            config.universe_blessing().get_by_stable_key(key).is_some()
                || config.universe_curio().get_by_stable_key(key).is_some()
        }
        UniversePoolKind::TrailblazeBonus => {
            config.universe_service().get_by_stable_key(key).is_some()
        }
    };
    if found {
        Ok(())
    } else {
        Err(reference(
            "Content pool content key is unresolved for its kind",
        ))
    }
}

fn ordered<T>(
    groups: BTreeMap<i32, Vec<(u32, T)>>,
    label: &str,
) -> Result<BTreeMap<i32, Vec<T>>, UniverseCatalogLoadError> {
    groups
        .into_iter()
        .map(|(parent, mut values)| {
            values.sort_by_key(|value| value.0);
            if values
                .iter()
                .map(|value| value.0)
                .ne(1..=values.len() as u32)
            {
                return Err(invalid(format!("{label} sequence is not contiguous")));
            }
            Ok((parent, values.into_iter().map(|value| value.1).collect()))
        })
        .collect()
}
fn source_list(
    values: Option<&[String]>,
    label: &str,
) -> Result<Box<[Box<str>]>, UniverseCatalogLoadError> {
    values
        .unwrap_or_default()
        .iter()
        .map(|value| checked_source(value, label).map(Into::into))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}
fn sequence(raw: i32, label: &str) -> Result<u32, UniverseCatalogLoadError> {
    positive(raw, &format!("{label} sequence"))
}
fn positive(raw: i32, label: &str) -> Result<u32, UniverseCatalogLoadError> {
    u32::try_from(raw)
        .ok()
        .filter(|value| *value != 0)
        .ok_or_else(|| invalid(format!("{label} must be positive")))
}
fn require_pool(config: &SoraConfig, id: i32) -> Result<(), UniverseCatalogLoadError> {
    if config.universe_encounter_pool().get(&id).is_some() {
        Ok(())
    } else {
        Err(reference("Encounter binding pool is unresolved"))
    }
}
fn domain_kind(value: UniverseDomainKind) -> DomainKind {
    match value {
        UniverseDomainKind::CombatPrimary => DomainKind::CombatPrimary,
        UniverseDomainKind::CombatSecondary => DomainKind::CombatSecondary,
        UniverseDomainKind::Occurrence => DomainKind::Occurrence,
        UniverseDomainKind::Encounter => DomainKind::Encounter,
        UniverseDomainKind::Respite => DomainKind::Respite,
        UniverseDomainKind::Elite => DomainKind::Elite,
        UniverseDomainKind::Boss => DomainKind::Boss,
        UniverseDomainKind::Transaction => DomainKind::Transaction,
        UniverseDomainKind::Adventure => DomainKind::Adventure,
    }
}
fn pool_kind(value: UniversePoolKind) -> ContentPoolKind {
    match value {
        UniversePoolKind::Blessing => ContentPoolKind::Blessing,
        UniversePoolKind::Curio => ContentPoolKind::Curio,
        UniversePoolKind::Occurrence => ContentPoolKind::Occurrence,
        UniversePoolKind::Encounter => ContentPoolKind::Encounter,
        UniversePoolKind::Shop => ContentPoolKind::Shop,
        UniversePoolKind::TrailblazeBonus => ContentPoolKind::TrailblazeBonus,
    }
}
macro_rules! id_fn {
    ($fn:ident,$ty:ty,$label:literal) => {
        fn $fn(raw: i32) -> Result<$ty, UniverseCatalogLoadError> {
            u32::try_from(raw)
                .ok()
                .and_then(<$ty>::new)
                .ok_or_else(|| invalid(concat!($label, " ID must be positive")))
        }
    };
}
id_fn!(member_id, EncounterMemberId, "Encounter member");
id_fn!(wave_id, EncounterWaveId, "Encounter wave");
id_fn!(pool_id, EncounterPoolId, "Encounter pool");
id_fn!(content_pool_id, ContentPoolId, "Content pool");
id_fn!(difficulty_id, DifficultyId, "Difficulty");
id_fn!(room_id, RoomId, "Room");
fn group_id(raw: i32, label: &str) -> Result<EncounterGroupId, UniverseCatalogLoadError> {
    u32::try_from(raw)
        .ok()
        .and_then(EncounterGroupId::new)
        .ok_or_else(|| invalid(format!("{label} ID must be positive")))
}
