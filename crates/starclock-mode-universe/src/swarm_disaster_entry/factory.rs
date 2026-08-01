use std::sync::Arc;

use crate::{
    error::UniverseCatalogLoadError, swarm_disaster_content::SwarmDisasterContentCatalog,
    swarm_disaster_structural::SwarmDisasterStructuralCatalog,
    swarm_disaster_unique::SwarmDisasterUniqueCatalog,
};

use super::{
    SwarmDisasterEntry, SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance, audience,
    audience_rule_runtime, communing, communing_rule_runtime, content_runtime, countdown,
    dice_control, disarray_rule_runtime, face_effect, map_overlay, occurrence_runtime,
    path_runtime, pathstrider_progress, plane_transition, profile_rule_runtime,
    progression_rule_runtime, service_adventure_runtime, state, topology_rule_runtime, trail,
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
        let topology_rules = Arc::new(topology_rule_runtime::TopologyRuleRuntimeCatalog::compile(
            [
                mechanic_rule(&content, "beacon-copy-and-blanking")?,
                mechanic_rule(&content, "domain-replacement")?,
                mechanic_rule(&content, "topology-event-order")?,
                mechanic_rule(&content, "topology-generation")?,
            ],
        )?);
        let countdown = Arc::new(countdown::CountdownRuntimeCatalog::compile(
            unique
                .countdown_runtime_input()
                .ok_or_else(|| error("Swarm Countdown runtime input is missing"))?,
        )?);
        let disarray_rules = Arc::new(disarray_rule_runtime::DisarrayRuleRuntimeCatalog::compile(
            [
                mechanic_rule(&content, "boss-decay-stack")?,
                mechanic_rule(&content, "countdown-lifecycle")?,
                mechanic_rule(&content, "planar-disarray-transition")?,
            ],
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
        let audience_rules = Arc::new(audience_rule_runtime::AudienceRuleRuntimeCatalog::compile(
            [
                mechanic_rule(&content, "audience-die-passive")?,
                mechanic_rule(&content, "dice-face-targeting")?,
                mechanic_rule(&content, "dice-roll-reroll-cheat")?,
            ],
        )?);
        let communing = Arc::new(communing::CommuningRuntimeCatalog::compile(
            unique.communing_runtime_input(),
        )?);
        let communing_rules = Arc::new(
            communing_rule_runtime::CommuningRuleRuntimeCatalog::compile([
                mechanic_rule(&content, "communing-choice")?,
                mechanic_rule(&content, "communing-dimension-points")?,
            ])?,
        );
        let trail = Arc::new(trail::TrailRuntimeCatalog::compile(
            unique.trail_runtime_input(),
        )?);
        let pathstrider = Arc::new(pathstrider_progress::PathstriderRuntimeCatalog::compile(
            unique.pathstrider_runtime_input(),
        )?);
        let progression_rules = Arc::new(
            progression_rule_runtime::ProgressionRuleRuntimeCatalog::compile([
                mechanic_rule(&content, "communing-trail-effect")?,
                mechanic_rule(&content, "pathstrider-progress")?,
            ])?,
        );
        let path_runtime = Arc::new(path_runtime::PathRuntimeCatalog::compile(
            unique.path_runtime_input(),
            &pathstrider,
        )?);
        let profile_rule = Arc::new(profile_rule_runtime::ProfileRuleRuntimeCatalog::compile(
            content
                .mechanic_rule_runtime_input("profile-entry")
                .ok_or_else(|| error("Profile-entry mechanic rule is missing"))?,
            &path_runtime,
        )?);
        let content_runtime = Arc::new(content_runtime::ContentRuntimeCatalog::compile(
            content.inventory_runtime_input(),
        )?);
        let occurrences = Arc::new(occurrence_runtime::OccurrenceRuntimeCatalog::compile(
            &content.interaction_runtime_input(),
        )?);
        let service_adventure = Arc::new(
            service_adventure_runtime::ServiceAdventureRuntimeCatalog::compile(
                &content.interaction_runtime_input(),
                content_runtime.standard(),
            )?,
        );
        Ok(Self {
            structural: Arc::new(structural),
            unique: Arc::new(unique),
            content: Arc::new(content),
            map,
            countdown,
            disarray_rules,
            transitions,
            audience,
            audience_rules,
            dice_controls,
            face_effects,
            occurrences,
            service_adventure,
            communing,
            communing_rules,
            content_runtime,
            trail,
            path_runtime,
            pathstrider,
            progression_rules,
            profile_rule,
            topology_rules,
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
        let audience = self.audience_rules.select(
            &self.audience,
            &entry.path,
            &entry.audience_die,
            &entry.audience_unlocks,
        )?;
        let dice_controls = self.dice_controls.select(&entry.dice_control_unlocks)?;
        let mut audience_state = audience.state_values()?.into_vec();
        audience_state.extend(dice_controls.state_values());
        audience_state.sort_unstable_by_key(|(key, _)| *key);
        let communing = canonical_communing(&self.unique, &entry.communing_points)?;
        let trail = self.progression_rules.select_trail(
            &self.trail,
            &entry.unlocked_progression,
            &communing,
        )?;
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
        let path_runtime = self.path_runtime.select(
            &entry.path,
            audience.unlock_id(),
            entry.trailblaze_bonus.as_deref(),
        )?;
        let countdown = self
            .unique
            .initial_countdown()
            .ok_or_else(|| error("invalid Countdown initial value"))?;
        let currency = self
            .content
            .initial_currency()
            .ok_or_else(|| error("invalid initial currency"))?;
        let topology = self.topology_rules.compile_topology(
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
            disarray_rules: Arc::clone(&self.disarray_rules),
            transitions: Arc::clone(&self.transitions),
            audience,
            audience_rules: Arc::clone(&self.audience_rules),
            dice_controls,
            face_effects: Arc::clone(&self.face_effects),
            occurrences: Arc::clone(&self.occurrences),
            service_adventure: Arc::clone(&self.service_adventure),
            communing: Arc::clone(&self.communing),
            communing_rules: Arc::clone(&self.communing_rules),
            content_runtime: Arc::clone(&self.content_runtime),
            trail,
            path_runtime,
            pathstrider: Arc::clone(&self.pathstrider),
            progression_rules: Arc::clone(&self.progression_rules),
            profile_rule: Arc::clone(&self.profile_rule),
            topology_rules: Arc::clone(&self.topology_rules),
        })
    }
}

fn mechanic_rule(
    content: &SwarmDisasterContentCatalog,
    family: &str,
) -> Result<
    crate::swarm_disaster_content::mechanic_access::MechanicRuleRuntimeInput,
    UniverseCatalogLoadError,
> {
    content
        .mechanic_rule_runtime_input(family)
        .ok_or_else(|| error("Swarm topology mechanic rule is missing"))
}
