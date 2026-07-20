//! Canonical SHA-256 identities for build definitions, catalogs and selections.

use sha2::{Digest, Sha256};
use starclock_combat::{
    AbilityId, Hp, ModifierDefinitionId, ResolvedModifierBinding, RuleBundleId, Speed, StatValue,
    UnitDefinitionId, UnitLevel,
    rule::model::{RuleSource, SourceClass},
};

use crate::{
    catalog::{BuildCatalogRevision, CharacterBuildDefinition},
    light_cone::{CombatPath, LightConeApplicability, LightConeDefinition},
    patch::BuildPatch,
    preset::BuildPreset,
    spec::CombatantBuildSpec,
};

macro_rules! digest_type {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name([u8; 32]);

        impl $name {
            #[must_use]
            pub const fn new(bytes: [u8; 32]) -> Self {
                Self(bytes)
            }
            #[must_use]
            pub const fn bytes(self) -> [u8; 32] {
                self.0
            }
        }
    };
}

digest_type!(
    BuildDefinitionDigest,
    "Digest of one canonical build definition."
);

pub const COMBATANT_BUILD_DIGEST_REVISION: &str = "starclock-combatant-build-v2";
digest_type!(BuildCatalogDigest, "Digest of one canonical build catalog.");
digest_type!(
    CombatantBuildDigest,
    "Digest of one exact normalized selected build."
);

pub(crate) fn character_digest(definition: &CharacterBuildDefinition) -> BuildDefinitionDigest {
    let mut encoder = Encoder::new(b"starclock-build-character-v1");
    encode_character(&mut encoder, definition);
    BuildDefinitionDigest::new(encoder.finish())
}

pub(crate) fn light_cone_digest(definition: &LightConeDefinition) -> BuildDefinitionDigest {
    let mut encoder = Encoder::new(b"starclock-build-light-cone-v1");
    encode_light_cone(&mut encoder, definition);
    BuildDefinitionDigest::new(encoder.finish())
}

pub(crate) fn catalog_digest(
    revision: &BuildCatalogRevision,
    combat_revision: &str,
    combat_digest: [u8; 32],
    characters: &[CharacterBuildDefinition],
    light_cones: &[LightConeDefinition],
    presets: &[BuildPreset],
) -> BuildCatalogDigest {
    let mut encoder = Encoder::new(b"starclock-build-catalog-v1");
    encoder.string(revision.as_str());
    encoder.string(combat_revision);
    encoder.bytes(&combat_digest);
    encoder.len(characters.len());
    for character in characters {
        encoder.bytes(&character_digest(character).bytes());
    }
    encoder.len(light_cones.len());
    for cone in light_cones {
        encoder.bytes(&light_cone_digest(cone).bytes());
    }
    encoder.len(presets.len());
    for preset in presets {
        encoder.u32(preset.id().get());
        encoder.string(preset.name());
        encode_spec(&mut encoder, preset.spec());
    }
    BuildCatalogDigest::new(encoder.finish())
}

pub(crate) fn selected_build_digest(
    catalog: BuildCatalogDigest,
    spec: &CombatantBuildSpec,
) -> CombatantBuildDigest {
    let mut encoder = Encoder::new(COMBATANT_BUILD_DIGEST_REVISION.as_bytes());
    encoder.bytes(&catalog.bytes());
    encode_spec(&mut encoder, spec);
    CombatantBuildDigest::new(encoder.finish())
}

pub(crate) struct ResolvedDigestInput<'a> {
    pub form: UnitDefinitionId,
    pub level: UnitLevel,
    pub maximum_hp: Hp,
    pub attack: StatValue,
    pub defense: StatValue,
    pub speed: Speed,
    pub abilities: &'a [AbilityId],
    pub rules: &'a [RuleBundleId],
    pub modifiers: &'a [ModifierDefinitionId],
    pub modifier_bindings: &'a [ResolvedModifierBinding],
    pub sources: &'a [RuleSource],
}

pub(crate) fn resolved_spec_digest(input: ResolvedDigestInput<'_>) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-resolved-combatant-v2");
    encoder.u32(input.form.get());
    encoder.u8(input.level.get());
    encoder.i64(input.maximum_hp.get());
    encoder.i64(input.attack.scaled());
    encoder.i64(input.defense.scaled());
    encoder.i64(input.speed.scaled());
    encoder.ids(input.abilities.iter().map(|id| id.get()));
    encoder.ids(input.rules.iter().map(|id| id.get()));
    encoder.ids(input.modifiers.iter().map(|id| id.get()));
    encoder.len(input.modifier_bindings.len());
    for binding in input.modifier_bindings {
        encoder.u32(binding.definition().get());
        encoder.u32(binding.source().get());
    }
    encoder.len(input.sources.len());
    for source in input.sources {
        encode_source(&mut encoder, source);
    }
    encoder.finish()
}

fn encode_character(encoder: &mut Encoder, definition: &CharacterBuildDefinition) {
    encoder.u32(definition.form().get());
    encoder.u8(path_tag(definition.path()));
    encode_source(encoder, definition.source());
    encoder.len(definition.stat_rows().len());
    for row in definition.stat_rows() {
        encoder.u8(row.level().get());
        encoder.u8(row.promotion().get());
        encoder.i64(row.maximum_hp().get());
        encoder.i64(row.attack().scaled());
        encoder.i64(row.defense().scaled());
        encoder.i64(row.speed().scaled());
    }
    encoder.ids(definition.abilities().iter().map(|id| id.get()));
    encoder.ids(definition.rule_bundles().iter().map(|id| id.get()));
    encoder.ids(definition.modifiers().iter().map(|id| id.get()));
    encoder.len(definition.ability_levels().len());
    for table in definition.ability_levels() {
        encoder.u32(table.family().get());
        encoder.u8(table.invested_cap().get());
        encoder.len(table.rows().len());
        for row in table.rows() {
            encoder.u8(row.effective().get());
            encoder.u32(row.resolved_ability().get());
        }
    }
    match definition.trace_graph() {
        None => encoder.u8(0),
        Some(graph) => {
            encoder.u8(1);
            encoder.len(graph.nodes().len());
            for node in graph.nodes() {
                encoder.u32(node.id().get());
                encode_source(encoder, node.source());
                encoder.ids(node.prerequisites().iter().map(|id| id.get()));
                encoder.u8(node.promotion_requirement().get());
                encode_patches(encoder, node.patches());
            }
        }
    }
    encoder.len(definition.eidolons().ranks().len());
    for rank in definition.eidolons().ranks() {
        encoder.u32(rank.id().get());
        encode_source(encoder, rank.source());
        encoder.u8(rank.rank().get());
        encode_patches(encoder, rank.patches());
    }
}

fn encode_light_cone(encoder: &mut Encoder, definition: &LightConeDefinition) {
    encoder.u32(definition.id().get());
    encode_source(encoder, definition.source());
    encoder.u8(path_tag(definition.path()));
    encoder.u8(match definition.applicability() {
        LightConeApplicability::MatchingPath => 0,
        LightConeApplicability::Always => 1,
        LightConeApplicability::BaseStatsOnly => 2,
    });
    encoder.len(definition.stats().len());
    for row in definition.stats() {
        encoder.u8(row.level().get());
        encoder.u8(row.promotion().get());
        encoder.i64(row.maximum_hp().get());
        encoder.i64(row.attack().scaled());
        encoder.i64(row.defense().scaled());
    }
    encoder.len(definition.passive_ranks().len());
    for rank in definition.passive_ranks() {
        encoder.u8(rank.rank().get());
        encode_patches(encoder, rank.patches());
    }
}

fn encode_spec(encoder: &mut Encoder, spec: &CombatantBuildSpec) {
    encoder.u32(spec.form().get());
    encoder.u8(spec.level().get());
    encoder.u8(spec.promotion().get());
    encoder.len(spec.ability_levels().len());
    for investment in spec.ability_levels() {
        encoder.u32(investment.family().get());
        encoder.u8(investment.invested().get());
    }
    encoder.ids(spec.traces().iter().map(|id| id.get()));
    encoder.u8(spec.eidolon().get());
    match spec.light_cone() {
        None => encoder.u8(0),
        Some(cone) => {
            encoder.u8(1);
            encoder.u32(cone.definition().get());
            encoder.u8(cone.level().get());
            encoder.u8(cone.promotion().get());
            encoder.u8(cone.superimposition().get());
        }
    }
    encoder.string(spec.relic_boundary().revision());
    encoder.len(spec.relic_boundary().piece_count());
}

fn encode_patches(encoder: &mut Encoder, patches: &[BuildPatch]) {
    encoder.len(patches.len());
    for patch in patches {
        match *patch {
            BuildPatch::AddAbility(id) => {
                encoder.u8(0);
                encoder.u32(id.get());
            }
            BuildPatch::AddRuleBundle(id) => {
                encoder.u8(1);
                encoder.u32(id.get());
            }
            BuildPatch::RemoveRuleBundle(id) => {
                encoder.u8(2);
                encoder.u32(id.get());
            }
            BuildPatch::AddModifier(id) => {
                encoder.u8(3);
                encoder.u32(id.get());
            }
            BuildPatch::ReplaceAbility { old, new } => {
                encoder.u8(4);
                encoder.u32(old.get());
                encoder.u32(new.get());
            }
            BuildPatch::AdjustAbilityLevel {
                family,
                bonus,
                cap_delta,
            } => {
                encoder.u8(5);
                encoder.u32(family.get());
                encoder.u8(bonus.to_be_bytes()[0]);
                encoder.u8(cap_delta.to_be_bytes()[0]);
            }
        }
    }
}

fn encode_source(encoder: &mut Encoder, source: &RuleSource) {
    encoder.u32(source.definition().get());
    encoder.u8(source_class_tag(source.class()));
    encoder.ids(source.tags().iter().map(|id| id.get()));
    encoder.bytes(&source.digest());
}

const fn path_tag(path: CombatPath) -> u8 {
    match path {
        CombatPath::Destruction => 0,
        CombatPath::Hunt => 1,
        CombatPath::Erudition => 2,
        CombatPath::Harmony => 3,
        CombatPath::Nihility => 4,
        CombatPath::Preservation => 5,
        CombatPath::Abundance => 6,
        CombatPath::Remembrance => 7,
        CombatPath::Elation => 8,
    }
}

const fn source_class_tag(class: SourceClass) -> u8 {
    match class {
        SourceClass::Unit => 0,
        SourceClass::Ability => 1,
        SourceClass::Effect => 2,
        SourceClass::Equipment => 3,
        SourceClass::Progression => 4,
        SourceClass::Enemy => 5,
        SourceClass::Encounter => 6,
        SourceClass::Activity => 7,
        SourceClass::Mode => 8,
        SourceClass::Synthetic => 9,
    }
}

struct Encoder(Sha256);

impl Encoder {
    fn new(domain: &[u8]) -> Self {
        let mut value = Self(Sha256::new());
        value.bytes(domain);
        value
    }
    fn finish(self) -> [u8; 32] {
        self.0.finalize().into()
    }
    fn bytes(&mut self, value: &[u8]) {
        self.u64(u64::try_from(value.len()).expect("canonical input length fits u64"));
        self.0.update(value);
    }
    fn string(&mut self, value: &str) {
        self.bytes(value.as_bytes());
    }
    fn len(&mut self, value: usize) {
        self.u64(u64::try_from(value).expect("canonical collection length fits u64"));
    }
    fn ids(&mut self, values: impl ExactSizeIterator<Item = u32>) {
        self.len(values.len());
        for value in values {
            self.u32(value);
        }
    }
    fn u8(&mut self, value: u8) {
        self.0.update([value]);
    }
    fn u32(&mut self, value: u32) {
        self.0.update(value.to_be_bytes());
    }
    fn u64(&mut self, value: u64) {
        self.0.update(value.to_be_bytes());
    }
    fn i64(&mut self, value: i64) {
        self.0.update(value.to_be_bytes());
    }
}
