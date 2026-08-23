use crate::{
    EnemyRuntimeProfileDefinition, EnemyRuntimeStatDefinition,
    build_lower::CharacterDataDefinition,
    catalog::{
        CatalogManifest, CatalogSummary, EffectDataDefinition, SimulationCatalog,
        StandardScenarioDefinition,
    },
};
use starclock_combat::modifier::registry::ModifierRegistry;
use starclock_combat::rule::model::BattleRuleDefinition;
use starclock_combat::{AbilityId, EffectDefinitionId, EnemyDefinitionId, RuleId};

impl SimulationCatalog {
    /// Returns the immutable combat catalog compiled from this exact Sora bundle.
    #[must_use]
    pub fn combat_catalog(&self) -> &starclock_combat::catalog::CombatCatalog {
        &self.combat_catalog
    }

    /// Returns the compatible immutable build catalog compiled from the same bundle.
    #[must_use]
    pub fn build_catalog(&self) -> &starclock_build::catalog::BuildCatalog {
        &self.build_catalog
    }

    /// Looks up one complete production character data definition.
    #[must_use]
    pub fn character(
        &self,
        id: starclock_combat::UnitDefinitionId,
    ) -> Option<&CharacterDataDefinition> {
        self.builds
            .characters
            .binary_search_by_key(&id, |character| character.id())
            .ok()
            .map(|index| &self.builds.characters[index])
    }

    /// Looks up the shared stable character form for an upstream avatar locator.
    #[must_use]
    pub fn character_form_for_source_avatar(
        &self,
        source_avatar_id: u32,
    ) -> Option<starclock_combat::UnitDefinitionId> {
        self.builds
            .characters
            .iter()
            .find(|character| character.source_avatar_id() == source_avatar_id)
            .map(CharacterDataDefinition::id)
    }

    /// Resolves one shared Ability by its stable content key.
    ///
    /// Generated identity rows stay private to the data layer; mode catalogs
    /// receive only the stable `AbilityId` used by combat.
    #[must_use]
    pub fn ability_by_stable_key(&self, stable_key: &str) -> Option<AbilityId> {
        let raw = self
            .identities
            .iter()
            .find(|identity| {
                identity.kind == crate::catalog::IdentityKind::Ability
                    && identity.stable_key.as_ref() == stable_key
            })?
            .id;
        let id = AbilityId::new(raw)?;
        self.combat_catalog().ability(id).map(|_| id)
    }

    /// Looks up the shared stable Light Cone identity for an upstream equipment locator.
    #[must_use]
    pub fn light_cone_for_source_equipment(
        &self,
        source_equipment_id: u32,
    ) -> Option<starclock_build::id::LightConeId> {
        self.builds
            .light_cones
            .iter()
            .find(|cone| cone.source_equipment_id() == source_equipment_id)
            .map(crate::light_cone_lower::LightConeDataDefinition::id)
    }

    /// Returns immutable bundle compatibility metadata.
    #[must_use]
    pub const fn manifest(&self) -> &CatalogManifest {
        &self.manifest
    }

    /// Returns immutable Starclock-owned modifier definitions lowered from Sora rows.
    #[must_use]
    pub const fn modifiers(&self) -> &ModifierRegistry {
        &self.combat.modifiers
    }

    /// Looks up one Starclock-owned effect definition lowered from Sora rows.
    #[must_use]
    pub fn effect(&self, id: EffectDefinitionId) -> Option<&EffectDataDefinition> {
        self.combat
            .effects
            .binary_search_by_key(&id, |effect| effect.id())
            .ok()
            .map(|index| &self.combat.effects[index])
    }

    /// Returns the typed semantic tags retained for one lowered ability.
    #[must_use]
    pub fn ability_semantic_tags(
        &self,
        id: AbilityId,
    ) -> Option<starclock_combat::catalog::action::AbilityTags> {
        self.combat
            .abilities
            .iter()
            .find(|ability| ability.id == id)
            .map(|ability| ability.semantic_tags)
    }

    /// Looks up one executable battle rule lowered from Sora rows.
    #[must_use]
    pub fn battle_rule(&self, id: RuleId) -> Option<&BattleRuleDefinition> {
        self.combat
            .rules
            .binary_search_by_key(&id, |rule| rule.id)
            .ok()
            .map(|index| &self.combat.rules[index].runtime)
    }

    /// Returns deterministic catalog counts without exposing transport rows.
    #[must_use]
    pub fn summary(&self) -> CatalogSummary {
        CatalogSummary {
            identity_count: self.identities.len(),
            enabled_identity_count: self
                .identities
                .iter()
                .filter(|identity| identity.enabled)
                .count(),
            ability_count: self.combat.abilities.len(),
            hit_plan_count: self.combat.hit_plans.len(),
            character_count: self.builds.len(),
            light_cone_count: self.builds.light_cones.len(),
            effect_count: self.combat.effects.len(),
            ai_graph_count: self.encounters.ai_graphs.len(),
            enemy_count: self.encounters.enemies.len(),
            encounter_count: self.encounters.encounters.len(),
            standard_profile_count: self.standard.profiles.len(),
            standard_scenario_count: self.standard.scenarios.len(),
        }
    }

    /// Looks up one validated finite enemy AI graph.
    #[must_use]
    pub fn ai_graph(
        &self,
        id: starclock_combat::AiGraphId,
    ) -> Option<&starclock_combat::catalog::encounter::AiGraphDefinition> {
        self.encounters.ai_graph(id)
    }

    /// Looks up one validated mechanically distinct enemy definition.
    #[must_use]
    pub fn enemy(
        &self,
        id: starclock_combat::EnemyDefinitionId,
    ) -> Option<&starclock_combat::catalog::definition::EnemyDefinition> {
        self.encounters.enemy(id)
    }

    /// Resolves one mechanically distinct enemy by its authored stable key.
    ///
    /// Stable-key lookup stays in the data layer so generated identity rows do
    /// not leak into mode or combat APIs.
    #[must_use]
    pub fn enemy_by_stable_key(
        &self,
        stable_key: &str,
    ) -> Option<&starclock_combat::catalog::definition::EnemyDefinition> {
        let raw = self
            .identities
            .iter()
            .find(|identity| identity.stable_key.as_ref() == stable_key)?
            .id;
        self.enemy(starclock_combat::EnemyDefinitionId::new(raw)?)
    }

    pub(crate) fn enemy_stable_definitions(
        &self,
    ) -> impl Iterator<
        Item = (
            &str,
            &starclock_combat::catalog::definition::EnemyDefinition,
        ),
    > {
        self.identities.iter().filter_map(|identity| {
            let definition = self.enemy(EnemyDefinitionId::new(identity.id)?)?;
            Some((identity.stable_key.as_ref(), definition))
        })
    }

    /// Looks up reviewed runtime statistics for one enemy, level and difficulty.
    #[must_use]
    pub fn enemy_runtime_stat(
        &self,
        variant: starclock_combat::EnemyDefinitionId,
        level: starclock_combat::UnitLevel,
        difficulty_key: &str,
    ) -> Option<&EnemyRuntimeStatDefinition> {
        self.encounters.enemy_stat(variant, level, difficulty_key)
    }

    /// Returns the nearest reviewed level row for one enemy and difficulty.
    /// Ties select the lower level so the fallback is deterministic.
    #[must_use]
    pub fn nearest_enemy_runtime_stat(
        &self,
        variant: starclock_combat::EnemyDefinitionId,
        level: starclock_combat::UnitLevel,
        difficulty_key: &str,
    ) -> Option<&EnemyRuntimeStatDefinition> {
        self.encounters
            .enemy_stats
            .iter()
            .filter(|row| row.variant() == variant && row.difficulty_key() == difficulty_key)
            .min_by_key(|row| (row.level().get().abs_diff(level.get()), row.level().get()))
    }

    /// Looks up the reviewed rank, weakness and Toughness profile for an enemy.
    #[must_use]
    pub fn enemy_runtime_profile(
        &self,
        variant: starclock_combat::EnemyDefinitionId,
    ) -> Option<&EnemyRuntimeProfileDefinition> {
        self.encounters.enemy_profile(variant)
    }

    /// Looks up one validated ordered encounter definition.
    #[must_use]
    pub fn encounter(
        &self,
        id: starclock_combat::EncounterId,
    ) -> Option<&starclock_combat::catalog::definition::EncounterDefinition> {
        self.encounters.encounter(id)
    }

    /// Looks up one ordinary Standard profile lowered from Sora rows.
    #[must_use]
    pub fn standard_profile(
        &self,
        id: starclock_mode_standard::StandardProfileId,
    ) -> Option<starclock_mode_standard::StandardProfile> {
        self.standard.profile(id)
    }

    /// Looks up one reproducible Standard scenario descriptor.
    #[must_use]
    pub fn standard_scenario(
        &self,
        id: starclock_mode_standard::StandardScenarioId,
    ) -> Option<&StandardScenarioDefinition> {
        self.standard.scenario(id)
    }
}
