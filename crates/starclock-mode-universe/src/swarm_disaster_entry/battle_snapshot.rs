//! Immutable current-Activity contribution snapshot for one Swarm battle.

use starclock_activity::{
    ActivityInventoryId, ActivitySlotId, ActivityTransactionState, ActivityValue,
};

use crate::{
    ability_runtime::{AbilityBoundary, AbilityExecutionContext, AbilityProjectionScope},
    battle_contribution::UniverseBattleContributionSet,
    digest::Encoder,
    error::UniverseCatalogLoadError,
    id::BlessingId,
};

use super::{
    SwarmDisasterRuntimeInstance, curio_battle_bridge,
    encounter_runtime::{EncounterRole, EncounterSelection},
    state::{BLESSING_INVENTORY, CONTENT, CURIO_INVENTORY},
    validate::{error as invalid, reference},
};

pub(super) const SWARM_DISASTER_BATTLE_SNAPSHOT_REVISION: &str =
    "swarm-disaster-battle-snapshot-v1";

pub(super) struct CompiledSwarmBattleSnapshot {
    pub(super) shared: UniverseBattleContributionSet,
    pub(super) disarray: (i64, i64, i64),
    pub(super) blessing_count: u16,
    pub(super) curio_count: u16,
    pub(super) interplay_count: u8,
    pub(super) trail_effect_count: u8,
    pub(super) next_battle_face: Option<Box<str>>,
    pub(super) digest: [u8; 32],
}

impl SwarmDisasterRuntimeInstance {
    pub(super) fn compile_battle_snapshot(
        &self,
        state: &ActivityTransactionState,
        selection: &EncounterSelection,
    ) -> Result<CompiledSwarmBattleSnapshot, UniverseCatalogLoadError> {
        let blessing_entries = inventory(state, BLESSING_INVENTORY)?
            .into_iter()
            .map(|(raw, count)| {
                let id = u32::try_from(raw)
                    .ok()
                    .and_then(BlessingId::new)
                    .ok_or_else(|| reference("invalid Swarm Blessing inventory identity"))?;
                Ok((id, count))
            })
            .collect::<Result<Vec<_>, UniverseCatalogLoadError>>()?;
        let blessings = self
            .content_runtime
            .blessings
            .contributions_from_owned(&blessing_entries)
            .map_err(|_| reference("invalid Swarm Blessing battle snapshot"))?;
        let selected_path = self
            .content_runtime
            .standard
            .paths()
            .iter()
            .find(|path| path.stable_key() == self.path())
            .ok_or_else(|| reference("missing selected Swarm shared Path"))?;
        // Swarm Resonance Formations are mode-owned automatic contributions.
        // The shared compiler receives no caller-selected Standard formation.
        let path = self
            .content_runtime
            .paths
            .contributions(selected_path.id(), &blessings, &[])
            .map_err(|_| reference("invalid Swarm shared Path contribution"))?;
        let curios = curio_battle_bridge::compile(&self.content_runtime, state)?;
        let abilities = self
            .content_runtime
            .run
            .ability_contributions(&[])
            .map_err(|_| invalid("invalid empty shared Ability contribution"))?;
        let boundary = if matches!(
            selection.role,
            EncounterRole::Elite
                | EncounterRole::FirstPlaneBoss
                | EncounterRole::SecondPlaneBoss
                | EncounterRole::FinalBoss
        ) {
            AbilityBoundary::EnterEliteOrBossDomain
        } else {
            AbilityBoundary::BattleStart
        };
        let projection = self
            .content_runtime
            .abilities
            .project(
                &[],
                AbilityExecutionContext::new(
                    AbilityProjectionScope::Battle,
                    boundary,
                    path.selected_path_blessings(),
                    false,
                ),
            )
            .map_err(|_| invalid("invalid empty shared Ability projection"))?;
        let shared = self
            .content_runtime
            .battle_contributions
            .compile_snapshot(&path, &blessings, &curios, &abilities, &projection)
            .map_err(|_| reference("invalid Swarm shared battle contributions"))?;

        let interplays = self.active_resonance_interplays(state)?;
        let trail = self.communing_trail_battle_effects().collect::<Vec<_>>();
        let next_battle_face = self
            .dice_resolution_face(state)
            .filter(|face| self.dice_face_activation_stage(face) == Some(3));
        let selected_boss =
            boss_plane(selection.role).and_then(|plane| self.selected_boss(state, plane));
        let boss_decay = if selection.role == EncounterRole::FinalBoss {
            self.countdown.selected_boss_decay(state)?
        } else {
            Box::new([])
        };
        let curio_entries = inventory(state, CURIO_INVENTORY)?;
        let curio_state = counter_map(state, CONTENT)?;
        let disarray = self.disarray_modifiers(state)?;
        let digest = snapshot_digest(
            self,
            selection,
            &shared,
            &blessing_entries,
            &curio_entries,
            &curio_state,
            &interplays,
            &trail,
            next_battle_face,
            selected_boss,
            &boss_decay,
            disarray,
        );
        Ok(CompiledSwarmBattleSnapshot {
            shared,
            disarray,
            blessing_count: checked_u16(blessing_entries.len())?,
            curio_count: checked_u16(curio_entries.len())?,
            interplay_count: checked_u8(interplays.len())?,
            trail_effect_count: checked_u8(trail.len())?,
            next_battle_face: next_battle_face.map(Into::into),
            digest,
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn snapshot_digest(
    instance: &SwarmDisasterRuntimeInstance,
    selection: &EncounterSelection,
    shared: &UniverseBattleContributionSet,
    blessings: &[(BlessingId, u32)],
    curios: &[(u64, u32)],
    curio_state: &[(u64, i64)],
    interplays: &[(&str, &str, &str)],
    trail: &[(&str, &str, &str)],
    next_battle_face: Option<&str>,
    selected_boss: Option<&str>,
    boss_decay: &[&super::countdown::BossDecayContribution],
    disarray: (i64, i64, i64),
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.battle-snapshot.v1");
    encoder.text(SWARM_DISASTER_BATTLE_SNAPSHOT_REVISION);
    encoder.digest(shared.digest());
    encoder.digest(instance.path_runtime_digest());
    encoder.digest(instance.communing_trail_digest());
    encoder.u8(instance.difficulty());
    encoder.u32(u32::from(selection.effective_level));
    encoder.u32(u32::try_from(blessings.len()).expect("Blessing inventory is bounded"));
    for (id, count) in blessings {
        encoder.u32(id.get());
        encoder.u32(*count);
    }
    encoder.u32(u32::try_from(curios.len()).expect("Curio inventory is bounded"));
    for (id, count) in curios {
        encoder.u64(*id);
        encoder.u32(*count);
    }
    encoder.u32(u32::try_from(curio_state.len()).expect("Curio state is bounded"));
    for (key, value) in curio_state {
        encoder.u64(*key);
        encoder.i64(*value);
    }
    encoder.u32(u32::try_from(interplays.len()).expect("Interplay set is bounded"));
    for (row, sub_path, binding) in interplays {
        encoder.text(row);
        encoder.text(sub_path);
        encoder.text(binding);
    }
    encoder.u32(u32::try_from(trail.len()).expect("Trail set is bounded"));
    for (node, effect, reference) in trail {
        encoder.text(node);
        encoder.text(effect);
        encoder.text(reference);
        if let Some(parameters) = instance.communing_trail_battle_effect_parameters(reference) {
            let parameters = parameters.collect::<Vec<_>>();
            encoder.u32(u32::try_from(parameters.len()).expect("Trail parameters are bounded"));
            for parameter in parameters {
                encoder.text(parameter);
            }
        } else {
            encoder.u32(0);
        }
    }
    encoder.optional_text(next_battle_face);
    if let Some(face) = next_battle_face {
        for value in instance.dice_face_parameters_scaled(face).unwrap_or(&[]) {
            encoder.i64(*value);
        }
        for effect in instance.dice_face_effect_references(face).unwrap_or(&[]) {
            encoder.u32(*effect);
        }
        encoder.u32(u32::from(
            instance.dice_face_turn_duration(face).unwrap_or(0),
        ));
    }
    if selected_boss.is_some() || !boss_decay.is_empty() {
        encoder.optional_text(selected_boss);
        encoder.u32(u32::try_from(boss_decay.len()).expect("Boss Decay set is bounded"));
        for contribution in boss_decay {
            encoder.text(contribution.key());
            encoder.text(contribution.effect_program());
        }
    }
    encoder.i64(disarray.0);
    encoder.i64(disarray.1);
    encoder.i64(disarray.2);
    encoder.finish()
}

fn inventory(
    state: &ActivityTransactionState,
    raw: u32,
) -> Result<Vec<(u64, u32)>, UniverseCatalogLoadError> {
    let id = ActivityInventoryId::new(raw).expect("static inventory identity is non-zero");
    state
        .inventory_entries(id)
        .map(Iterator::collect)
        .ok_or_else(|| invalid("missing Swarm battle inventory"))
}

fn counter_map(
    state: &ActivityTransactionState,
    raw: u32,
) -> Result<Vec<(u64, i64)>, UniverseCatalogLoadError> {
    let id = ActivitySlotId::new(raw).expect("static slot identity is non-zero");
    match state.slot(id) {
        Some(ActivityValue::BoundedCounterMap(values)) => Ok(values.to_vec()),
        _ => Err(invalid("missing Swarm battle counter map")),
    }
}

fn checked_u16(value: usize) -> Result<u16, UniverseCatalogLoadError> {
    u16::try_from(value).map_err(|_| invalid("Swarm battle snapshot count overflow"))
}

const fn boss_plane(role: EncounterRole) -> Option<u8> {
    match role {
        EncounterRole::Combat | EncounterRole::Elite => None,
        EncounterRole::FirstPlaneBoss => Some(1),
        EncounterRole::SecondPlaneBoss => Some(2),
        EncounterRole::FinalBoss => Some(3),
    }
}

fn checked_u8(value: usize) -> Result<u8, UniverseCatalogLoadError> {
    u8::try_from(value).map_err(|_| invalid("Swarm battle snapshot count overflow"))
}
