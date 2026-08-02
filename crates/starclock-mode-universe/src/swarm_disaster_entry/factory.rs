use std::sync::Arc;

use crate::{
    error::UniverseCatalogLoadError, swarm_disaster_content::SwarmDisasterContentCatalog,
    swarm_disaster_structural::SwarmDisasterStructuralCatalog,
    swarm_disaster_unique::SwarmDisasterUniqueCatalog,
};

use super::{
    SwarmDisasterEntry, SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance, audience,
    audience_rule_runtime, battle_enemy_catalog, boss_rule_runtime, communing,
    communing_rule_runtime, content_runtime, countdown, curio_rule_runtime, dice_control,
    disarray_rule_runtime, encounter_rule_runtime, encounter_runtime, face_effect, map_overlay,
    occurrence_rule_runtime, occurrence_runtime, path_rule_runtime, path_runtime,
    pathstrider_progress, plane_transition, profile_rule_runtime, progression_rule_runtime,
    runtime_coverage, semantic_fixture_runtime, service_adventure_runtime, service_rule_runtime,
    state, topology_rule_runtime, trail,
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
        let boss_rules = Arc::new(boss_rule_runtime::BossRuleRuntimeCatalog::compile([
            mechanic_rule(&content, "boss-choice-consequence")?,
            mechanic_rule(&content, "final-boss-consequence")?,
        ])?);
        let encounter_rule = Arc::new(
            encounter_rule_runtime::EncounterRuleRuntimeCatalog::compile(mechanic_rule(
                &content,
                "encounter-selection",
            )?)?,
        );
        let encounters = Arc::new(encounter_runtime::EncounterRuntimeCatalog::compile(
            content.encounter_runtime_input()?,
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
        let path_rules = Arc::new(path_rule_runtime::PathRuleRuntimeCatalog::compile([
            mechanic_rule(&content, "path-and-propagation-unlock")?,
            mechanic_rule(&content, "resonance-interplay")?,
        ])?);
        let profile_rule = Arc::new(profile_rule_runtime::ProfileRuleRuntimeCatalog::compile(
            content
                .mechanic_rule_runtime_input("profile-entry")
                .ok_or_else(|| error("Profile-entry mechanic rule is missing"))?,
            &path_runtime,
        )?);
        let content_runtime = Arc::new(content_runtime::ContentRuntimeCatalog::compile(
            content.inventory_runtime_input(),
        )?);
        let battle_catalog = Arc::new(
            battle_enemy_catalog::SwarmBattleCatalogComposition::compile(
                &encounters,
                content_runtime.standard(),
                content_runtime
                    .standard()
                    .simulation_catalog()
                    .combat_catalog(),
            )?,
        );
        let curio_rules = Arc::new(curio_rule_runtime::CurioRuleRuntimeCatalog::compile(
            mechanic_rule(&content, "curio-lifecycle")?,
        )?);
        let occurrences = Arc::new(occurrence_runtime::OccurrenceRuntimeCatalog::compile(
            &content.interaction_runtime_input(),
        )?);
        let occurrence_rules = Arc::new(
            occurrence_rule_runtime::OccurrenceRuleRuntimeCatalog::compile(mechanic_rule(
                &content,
                "occurrence-choice",
            )?)?,
        );
        let service_adventure = Arc::new(
            service_adventure_runtime::ServiceAdventureRuntimeCatalog::compile(
                &content.interaction_runtime_input(),
                content_runtime.standard(),
            )?,
        );
        let service_rules = Arc::new(service_rule_runtime::ServiceRuleRuntimeCatalog::compile(
            mechanic_rule(&content, "service-and-adventure")?,
        )?);
        let semantic_fixtures =
            Arc::new(semantic_fixture_runtime::SemanticFixtureRuntimeCatalog::compile(&content)?);
        runtime_coverage::RuntimeCoverageCatalog::compile(&content, semantic_fixtures.digest())?;
        Ok(Self {
            structural: Arc::new(structural),
            unique: Arc::new(unique),
            content: Arc::new(content),
            map,
            countdown,
            disarray_rules,
            transitions,
            boss_rules,
            encounter_rule,
            encounters,
            audience,
            audience_rules,
            dice_controls,
            face_effects,
            occurrences,
            occurrence_rules,
            service_adventure,
            service_rules,
            semantic_fixtures,
            communing,
            communing_rules,
            content_runtime,
            curio_rules,
            trail,
            path_runtime,
            path_rules,
            pathstrider,
            progression_rules,
            profile_rule,
            topology_rules,
            battle_catalog,
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
        let path_runtime = self.path_rules.select(
            &self.path_runtime,
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
        let encounter_runtime = encounter_runtime::CompiledEncounterRuntime::compile(
            Arc::clone(&self.encounters),
            self.structural
                .encounter_runtime_input(area.id)
                .ok_or_else(|| reference("Swarm encounter structural input is incomplete"))?,
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
            boss_rules: Arc::clone(&self.boss_rules),
            encounter_rule: Arc::clone(&self.encounter_rule),
            encounter_runtime,
            audience,
            audience_rules: Arc::clone(&self.audience_rules),
            dice_controls,
            face_effects: Arc::clone(&self.face_effects),
            occurrences: Arc::clone(&self.occurrences),
            occurrence_rules: Arc::clone(&self.occurrence_rules),
            service_adventure: Arc::clone(&self.service_adventure),
            service_rules: Arc::clone(&self.service_rules),
            communing: Arc::clone(&self.communing),
            communing_rules: Arc::clone(&self.communing_rules),
            content_runtime: Arc::clone(&self.content_runtime),
            curio_rules: Arc::clone(&self.curio_rules),
            trail,
            path_runtime,
            path_rules: Arc::clone(&self.path_rules),
            pathstrider: Arc::clone(&self.pathstrider),
            progression_rules: Arc::clone(&self.progression_rules),
            profile_rule: Arc::clone(&self.profile_rule),
            topology_rules: Arc::clone(&self.topology_rules),
            battle_catalog: Arc::clone(&self.battle_catalog),
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
