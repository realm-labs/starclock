use std::sync::Arc;

use crate::{
    error::UniverseCatalogLoadError, swarm_disaster_content::SwarmDisasterContentCatalog,
    swarm_disaster_structural::SwarmDisasterStructuralCatalog,
    swarm_disaster_unique::SwarmDisasterUniqueCatalog,
};

use super::{
    SwarmDisasterEntry, SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance, audience,
    communing, countdown, dice_control, face_effect, map_overlay, plane_transition, state,
    topology, trail,
    validate::{
        canonical_communing, canonical_progression, error, reference, validate_participants,
    },
};

impl SwarmDisasterRuntimeFactory {
    pub fn load_candidate(bytes: &[u8]) -> Result<Self, UniverseCatalogLoadError> {
        let structural = SwarmDisasterStructuralCatalog::load(bytes)?;
        let unique = SwarmDisasterUniqueCatalog::load(bytes)?;
        let content = SwarmDisasterContentCatalog::load(bytes, &structural, &unique)?;
        if !structural.has_runtime_profile()
            || structural.bundle_summary() != unique.bundle_summary()
            || structural.bundle_summary() != content.bundle_summary()
        {
            return Err(error("Swarm Disaster runtime profile identity mismatch"));
        }
        let map = Arc::new(map_overlay::MapRuntimeCatalog::compile(
            structural.map_structural_input(),
            content.map_runtime_input()?,
        )?);
        let countdown = Arc::new(countdown::CountdownRuntimeCatalog::compile(
            unique
                .countdown_runtime_input()
                .ok_or_else(|| error("Swarm Countdown runtime input is missing"))?,
        )?);
        let transitions = Arc::new(plane_transition::PlaneTransitionRuntimeCatalog::compile(
            structural.boss_choice_runtime_input(),
        )?);
        let audience_input = unique.audience_runtime_input();
        let face_effects = Arc::new(face_effect::DiceFaceRuntimeCatalog::compile(
            &audience_input.faces,
            &unique.dice_target_runtime_input(),
        )?);
        let audience = Arc::new(audience::AudienceRuntimeCatalog::compile(audience_input)?);
        let dice_controls = Arc::new(dice_control::DiceControlRuntimeCatalog::compile(
            &unique.dice_control_runtime_input(),
        )?);
        let communing = Arc::new(communing::CommuningRuntimeCatalog::compile(
            unique.communing_runtime_input(),
        )?);
        let trail = Arc::new(trail::TrailRuntimeCatalog::compile(
            unique.trail_runtime_input(),
        )?);
        Ok(Self {
            structural: Arc::new(structural),
            unique: Arc::new(unique),
            content: Arc::new(content),
            map,
            countdown,
            transitions,
            audience,
            dice_controls,
            face_effects,
            communing,
            trail,
        })
    }

    pub fn compile_entry(
        &self,
        entry: SwarmDisasterEntry,
    ) -> Result<SwarmDisasterRuntimeInstance, UniverseCatalogLoadError> {
        validate_participants(entry.participants.policy())?;
        let area = self
            .structural
            .entry_area(&entry.area)
            .ok_or_else(|| reference("unknown or non-Formal Swarm area"))?;
        let selection = self
            .unique
            .entry_selection(&entry.path, &entry.audience_die)
            .ok_or_else(|| reference("Path and Audience Die are not a released pair"))?;
        let audience =
            self.audience
                .select(&entry.path, &entry.audience_die, &entry.audience_unlocks)?;
        let dice_controls = self.dice_controls.select(&entry.dice_control_unlocks)?;
        let mut audience_state = audience.state_values()?.into_vec();
        audience_state.extend(dice_controls.state_values());
        audience_state.sort_unstable_by_key(|(key, _)| *key);
        let communing = canonical_communing(&self.unique, &entry.communing_points)?;
        let trail = self.trail.select(&entry.unlocked_progression, &communing)?;
        let progression = canonical_progression(&self.unique, &entry.unlocked_progression)?;
        let bonus = entry
            .trailblaze_bonus
            .as_deref()
            .map(|key| {
                self.unique
                    .trailblaze_bonus_id(key)
                    .ok_or_else(|| reference("unknown Trailblaze bonus"))
            })
            .transpose()?;
        let countdown = self
            .unique
            .initial_countdown()
            .ok_or_else(|| error("invalid Countdown initial value"))?;
        let currency = self
            .content
            .initial_currency()
            .ok_or_else(|| error("invalid initial currency"))?;
        let topology = topology::compile(
            self.structural
                .topology_input(area.id)
                .ok_or_else(|| reference("Swarm topology input is incomplete"))?,
        )?;
        let state = state::compile(state::SwarmStateCompileInput {
            area,
            selection,
            communing: &communing,
            progression: &progression,
            bonus,
            countdown,
            currency,
            audience_state: &audience_state,
        })?
        .with_logical_scopes(topology.scopes);
        Ok(SwarmDisasterRuntimeInstance {
            area: entry.area,
            difficulty: area.difficulty,
            path: entry.path,
            audience_die: entry.audience_die,
            participants: Arc::new(entry.participants),
            trailblaze_bonus: entry.trailblaze_bonus,
            state,
            graph: topology.graph,
            planes: topology.planes,
            map: Arc::clone(&self.map),
            countdown: Arc::clone(&self.countdown),
            transitions: Arc::clone(&self.transitions),
            audience,
            dice_controls,
            face_effects: Arc::clone(&self.face_effects),
            communing: Arc::clone(&self.communing),
            trail,
        })
    }
}
