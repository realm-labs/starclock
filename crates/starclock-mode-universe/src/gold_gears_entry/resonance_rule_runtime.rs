//! Terminal dispatch for shared Resonance, Gold Interplay and Extrapolation rules.

use starclock_combat::{
    SourceDefinitionId,
    rule::model::{RuleSource, SourceClass},
};

use crate::{
    catalog::UniverseCatalog,
    digest::Encoder,
    gold_gears_unique::{GoldAndGearsUniqueCatalog, Resonance},
    path::{ExactParameter, ResonanceKind},
    rule::MechanicRuleKind,
};

use super::{
    GoldAndGearsEntryError, GoldAndGearsExtrapolationPolarity, GoldAndGearsExtrapolationSelection,
    GoldAndGearsResonanceContribution, GoldAndGearsResonanceKind, GoldAndGearsResonanceSet,
    api::{GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance},
    progression_runtime::ProgressionRuntimeCatalog,
};

const SOURCE_BASE: u32 = 0x7f30_0000;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoldAndGearsResonanceRuleKind {
    SharedResonance = 0,
    Interplay = 1,
    Extrapolation = 2,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoldAndGearsResonanceRuleOwnership {
    GoldAndGears = 0,
    Shared = 1,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoldAndGearsResonanceRuleAccuracy {
    ExactPublic = 0,
    ProjectPolicy = 1,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoldAndGearsResonanceCombatAttachment {
    PlayerOwner = 0,
    RelativeToEnemyOwner = 1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsResonanceRuleBinding {
    rule_id: Box<str>,
    owner_id: Box<str>,
    binding_key: Box<str>,
    parameter_count: u16,
    kind: GoldAndGearsResonanceRuleKind,
    ownership: GoldAndGearsResonanceRuleOwnership,
    accuracy: GoldAndGearsResonanceRuleAccuracy,
}

impl GoldAndGearsResonanceRuleBinding {
    #[must_use]
    pub fn rule_id(&self) -> &str {
        &self.rule_id
    }

    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    #[must_use]
    pub fn binding_key(&self) -> &str {
        &self.binding_key
    }

    #[must_use]
    pub const fn parameter_count(&self) -> u16 {
        self.parameter_count
    }

    #[must_use]
    pub const fn kind(&self) -> GoldAndGearsResonanceRuleKind {
        self.kind
    }

    #[must_use]
    pub const fn ownership(&self) -> GoldAndGearsResonanceRuleOwnership {
        self.ownership
    }

    #[must_use]
    pub const fn accuracy(&self) -> GoldAndGearsResonanceRuleAccuracy {
        self.accuracy
    }

    #[must_use]
    pub const fn executor(&self) -> &'static str {
        match self.ownership {
            GoldAndGearsResonanceRuleOwnership::GoldAndGears => "CombatRuleIr",
            GoldAndGearsResonanceRuleOwnership::Shared => "ReleasedSharedExecutor",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsResonanceCombatBinding {
    terminal: GoldAndGearsResonanceRuleBinding,
    contribution: GoldAndGearsResonanceContribution,
    attachment: GoldAndGearsResonanceCombatAttachment,
    source: RuleSource,
}

impl GoldAndGearsResonanceCombatBinding {
    #[must_use]
    pub const fn terminal(&self) -> &GoldAndGearsResonanceRuleBinding {
        &self.terminal
    }

    #[must_use]
    pub const fn contribution(&self) -> &GoldAndGearsResonanceContribution {
        &self.contribution
    }

    #[must_use]
    pub const fn attachment(&self) -> GoldAndGearsResonanceCombatAttachment {
        self.attachment
    }

    #[must_use]
    pub const fn source(&self) -> &RuleSource {
        &self.source
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsResonanceCombatSet {
    bindings: Box<[GoldAndGearsResonanceCombatBinding]>,
    digest: [u8; 32],
}

impl GoldAndGearsResonanceCombatSet {
    #[must_use]
    pub fn bindings(&self) -> &[GoldAndGearsResonanceCombatBinding] {
        &self.bindings
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

#[derive(Clone, Debug)]
pub(super) struct GoldAndGearsResonanceRuleRuntimeCatalog {
    bindings: Box<[GoldAndGearsResonanceRuleBinding]>,
    digest: [u8; 32],
}

impl GoldAndGearsResonanceRuleRuntimeCatalog {
    pub(super) fn compile(
        unique: &GoldAndGearsUniqueCatalog,
        standard: &UniverseCatalog,
        progression: &ProgressionRuntimeCatalog,
    ) -> Result<Self, GoldAndGearsEntryError> {
        let mut bindings = Vec::with_capacity(90);
        for resonance in &unique.resonances {
            bindings.push(shared_binding(resonance, standard, progression)?);
        }
        for interplay in &unique.interplays {
            let contribution = progression
                .resonance_contribution(&interplay.identity.stable_key)
                .ok_or(GoldAndGearsEntryError::InvalidResonanceRuleRuntime)?;
            bindings.push(gold_binding(
                &interplay.rule_contribution,
                &interplay.identity.stable_key,
                contribution,
                GoldAndGearsResonanceRuleKind::Interplay,
                GoldAndGearsResonanceRuleAccuracy::ExactPublic,
            )?);
        }
        for extrapolation in &unique.extrapolations {
            let inherited = unique
                .resonances
                .iter()
                .find(|resonance| resonance.identity.id.0 == extrapolation.shared_resonance_id)
                .ok_or(GoldAndGearsEntryError::InvalidResonanceRuleRuntime)?;
            let shared = standard
                .resonances()
                .iter()
                .find(|resonance| resonance.stable_key() == inherited.identity.stable_key.as_ref())
                .ok_or(GoldAndGearsEntryError::InvalidResonanceRuleRuntime)?;
            let contribution = progression
                .resonance_contribution(&extrapolation.identity.stable_key)
                .ok_or(GoldAndGearsEntryError::InvalidResonanceRuleRuntime)?;
            let authored_parameters = contribution.parameters_scaled().collect::<Vec<_>>();
            if shared.source_binding_key() != contribution.binding_key()
                || shared
                    .parameters()
                    .iter()
                    .map(exact_scaled)
                    .collect::<Option<Vec<_>>>()
                    != Some(authored_parameters.clone())
                || !matches!(
                    (
                        extrapolation.enhanced,
                        extrapolation.shared_resonance_kind.as_ref(),
                        shared.kind(),
                        contribution.kind(),
                    ),
                    (
                        false,
                        "Resonance",
                        ResonanceKind::Resonance,
                        GoldAndGearsResonanceKind::Resonance,
                    ) | (
                        true,
                        "Formation",
                        ResonanceKind::Formation,
                        GoldAndGearsResonanceKind::Formation,
                    )
                )
            {
                return Err(GoldAndGearsEntryError::InvalidResonanceRuleRuntime);
            }
            bindings.push(gold_binding(
                &extrapolation.rule_contribution,
                &extrapolation.identity.stable_key,
                contribution,
                GoldAndGearsResonanceRuleKind::Extrapolation,
                GoldAndGearsResonanceRuleAccuracy::ProjectPolicy,
            )?);
        }
        bindings.sort_unstable_by(|left, right| left.rule_id.cmp(&right.rule_id));
        if bindings.len() != 90
            || bindings
                .windows(2)
                .any(|pair| pair[0].rule_id >= pair[1].rule_id)
            || count_kind(&bindings, GoldAndGearsResonanceRuleKind::SharedResonance) != 36
            || count_kind(&bindings, GoldAndGearsResonanceRuleKind::Interplay) != 18
            || count_kind(&bindings, GoldAndGearsResonanceRuleKind::Extrapolation) != 36
        {
            return Err(GoldAndGearsEntryError::InvalidResonanceRuleRuntime);
        }
        let digest = execution_digest(&bindings);
        Ok(Self {
            bindings: bindings.into_boxed_slice(),
            digest,
        })
    }

    pub(super) fn bindings(&self) -> &[GoldAndGearsResonanceRuleBinding] {
        &self.bindings
    }

    pub(super) const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    fn project(
        &self,
        contributions: &[GoldAndGearsResonanceContribution],
        attachment: GoldAndGearsResonanceCombatAttachment,
    ) -> Result<GoldAndGearsResonanceCombatSet, GoldAndGearsEntryError> {
        let mut output = Vec::with_capacity(contributions.len());
        for contribution in contributions {
            let (index, terminal) = self
                .bindings
                .iter()
                .enumerate()
                .find(|(_, binding)| binding.owner_id.as_ref() == contribution.source())
                .ok_or(GoldAndGearsEntryError::InvalidResonanceRuleRuntime)?;
            if terminal.binding_key.as_ref() != contribution.binding_key()
                || usize::from(terminal.parameter_count) != contribution.parameters_scaled().len()
                || !attachment_allows(attachment, terminal.kind, contribution.kind())
            {
                return Err(GoldAndGearsEntryError::InvalidResonanceRuleRuntime);
            }
            let source_id = SourceDefinitionId::new(
                SOURCE_BASE
                    + u32::try_from(index + 1)
                        .map_err(|_| GoldAndGearsEntryError::InvalidResonanceRuleRuntime)?,
            )
            .ok_or(GoldAndGearsEntryError::InvalidResonanceRuleRuntime)?;
            output.push(GoldAndGearsResonanceCombatBinding {
                terminal: terminal.clone(),
                contribution: contribution.clone(),
                attachment,
                source: RuleSource::new(
                    source_id,
                    SourceClass::Mode,
                    vec![],
                    source_digest(terminal, contribution, attachment),
                ),
            });
        }
        output.sort_unstable_by(|left, right| left.terminal.rule_id.cmp(&right.terminal.rule_id));
        if output
            .windows(2)
            .any(|pair| pair[0].terminal.rule_id >= pair[1].terminal.rule_id)
        {
            return Err(GoldAndGearsEntryError::InvalidResonanceRuleRuntime);
        }
        let digest = combat_set_digest(&output);
        Ok(GoldAndGearsResonanceCombatSet {
            bindings: output.into_boxed_slice(),
            digest,
        })
    }
}

impl GoldAndGearsRuntimeFactory {
    #[must_use]
    pub fn resonance_rule_bindings(&self) -> &[GoldAndGearsResonanceRuleBinding] {
        self.content_runtime.resonance_rules.bindings()
    }

    #[must_use]
    pub fn resonance_execution_digest(&self) -> [u8; 32] {
        self.content_runtime.resonance_rules.digest()
    }
}

impl GoldAndGearsRuntimeInstance {
    #[must_use]
    pub fn resonance_rule_bindings(&self) -> &[GoldAndGearsResonanceRuleBinding] {
        self.content_runtime.resonance_rules.bindings()
    }

    #[must_use]
    pub fn resonance_execution_digest(&self) -> [u8; 32] {
        self.content_runtime.resonance_rules.digest()
    }

    pub fn compile_resonance_combat_set(
        &self,
        set: &GoldAndGearsResonanceSet,
    ) -> Result<GoldAndGearsResonanceCombatSet, GoldAndGearsEntryError> {
        let contributions = set
            .resonance()
            .into_iter()
            .chain(set.formations())
            .chain(set.interplays())
            .cloned()
            .collect::<Vec<_>>();
        self.content_runtime.resonance_rules.project(
            &contributions,
            GoldAndGearsResonanceCombatAttachment::PlayerOwner,
        )
    }

    pub fn compile_extrapolation_combat_set(
        &self,
        selection: &GoldAndGearsExtrapolationSelection,
    ) -> Result<GoldAndGearsResonanceCombatSet, GoldAndGearsEntryError> {
        if selection.polarity() != GoldAndGearsExtrapolationPolarity::RelativeToEnemyOwner {
            return Err(GoldAndGearsEntryError::InvalidResonanceRuleRuntime);
        }
        self.content_runtime.resonance_rules.project(
            selection.contributions(),
            GoldAndGearsResonanceCombatAttachment::RelativeToEnemyOwner,
        )
    }
}

fn shared_binding(
    resonance: &Resonance,
    standard: &UniverseCatalog,
    progression: &ProgressionRuntimeCatalog,
) -> Result<GoldAndGearsResonanceRuleBinding, GoldAndGearsEntryError> {
    let released = standard
        .resonances()
        .iter()
        .find(|candidate| candidate.stable_key() == resonance.identity.stable_key.as_ref())
        .ok_or(GoldAndGearsEntryError::InvalidResonanceRuleRuntime)?;
    let contribution = progression
        .resonance_contribution(&resonance.identity.stable_key)
        .ok_or(GoldAndGearsEntryError::InvalidResonanceRuleRuntime)?;
    let parameters = contribution.parameters_scaled().collect::<Vec<_>>();
    if resonance.inherited_rule_ids.len() != 1
        || resonance.inherited_rule_ids[0].as_ref() != released.rule_key()
        || contribution.binding_key() != released.source_binding_key()
        || !matches!(
            contribution.kind(),
            GoldAndGearsResonanceKind::Resonance | GoldAndGearsResonanceKind::Formation
        )
        || released
            .parameters()
            .iter()
            .map(exact_scaled)
            .collect::<Option<Vec<_>>>()
            != Some(parameters.clone())
        || standard.mechanic_rules().iter().all(|rule| {
            rule.stable_key() != released.rule_key()
                || rule.source_record_key() != released.stable_key()
                || rule.kind() != MechanicRuleKind::PathResonance
        })
    {
        return Err(GoldAndGearsEntryError::InvalidResonanceRuleRuntime);
    }
    Ok(GoldAndGearsResonanceRuleBinding {
        rule_id: released.rule_key().into(),
        owner_id: released.stable_key().into(),
        binding_key: released.source_binding_key().into(),
        parameter_count: u16::try_from(parameters.len())
            .map_err(|_| GoldAndGearsEntryError::InvalidResonanceRuleRuntime)?,
        kind: GoldAndGearsResonanceRuleKind::SharedResonance,
        ownership: GoldAndGearsResonanceRuleOwnership::Shared,
        accuracy: GoldAndGearsResonanceRuleAccuracy::ExactPublic,
    })
}

fn gold_binding(
    rule_id: &str,
    owner_id: &str,
    contribution: &GoldAndGearsResonanceContribution,
    kind: GoldAndGearsResonanceRuleKind,
    accuracy: GoldAndGearsResonanceRuleAccuracy,
) -> Result<GoldAndGearsResonanceRuleBinding, GoldAndGearsEntryError> {
    if rule_id.is_empty()
        || !owner_id.starts_with("gold-gears.")
        || contribution.source() != owner_id
        || !contribution.binding_key().starts_with("StageAbility_")
        || !matches!(
            (kind, contribution.kind()),
            (
                GoldAndGearsResonanceRuleKind::Interplay,
                GoldAndGearsResonanceKind::Interplay,
            ) | (
                GoldAndGearsResonanceRuleKind::Extrapolation,
                GoldAndGearsResonanceKind::Resonance | GoldAndGearsResonanceKind::Formation,
            )
        )
    {
        return Err(GoldAndGearsEntryError::InvalidResonanceRuleRuntime);
    }
    Ok(GoldAndGearsResonanceRuleBinding {
        rule_id: rule_id.into(),
        owner_id: owner_id.into(),
        binding_key: contribution.binding_key().into(),
        parameter_count: u16::try_from(contribution.parameters_scaled().len())
            .map_err(|_| GoldAndGearsEntryError::InvalidResonanceRuleRuntime)?,
        kind,
        ownership: GoldAndGearsResonanceRuleOwnership::GoldAndGears,
        accuracy,
    })
}

fn attachment_allows(
    attachment: GoldAndGearsResonanceCombatAttachment,
    rule_kind: GoldAndGearsResonanceRuleKind,
    contribution_kind: GoldAndGearsResonanceKind,
) -> bool {
    matches!(
        (attachment, rule_kind, contribution_kind),
        (
            GoldAndGearsResonanceCombatAttachment::PlayerOwner,
            GoldAndGearsResonanceRuleKind::SharedResonance,
            GoldAndGearsResonanceKind::Resonance | GoldAndGearsResonanceKind::Formation,
        ) | (
            GoldAndGearsResonanceCombatAttachment::PlayerOwner,
            GoldAndGearsResonanceRuleKind::Interplay,
            GoldAndGearsResonanceKind::Interplay,
        ) | (
            GoldAndGearsResonanceCombatAttachment::RelativeToEnemyOwner,
            GoldAndGearsResonanceRuleKind::Extrapolation,
            GoldAndGearsResonanceKind::Resonance | GoldAndGearsResonanceKind::Formation,
        )
    )
}

fn count_kind(
    bindings: &[GoldAndGearsResonanceRuleBinding],
    kind: GoldAndGearsResonanceRuleKind,
) -> usize {
    bindings
        .iter()
        .filter(|binding| binding.kind == kind)
        .count()
}

fn exact_scaled(value: &ExactParameter) -> Option<i64> {
    if value.scale() > 6 {
        return None;
    }
    value
        .coefficient()
        .checked_mul(10_i64.pow(u32::from(6 - value.scale())))
}

fn execution_digest(bindings: &[GoldAndGearsResonanceRuleBinding]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-gold-gears-resonance-execution");
    encoder.u32(bindings.len() as u32);
    for binding in bindings {
        encoder.text(&binding.rule_id);
        encoder.text(&binding.owner_id);
        encoder.text(&binding.binding_key);
        encoder.u32(u32::from(binding.parameter_count));
        encoder.u8(binding.kind as u8);
        encoder.u8(binding.ownership as u8);
        encoder.u8(binding.accuracy as u8);
    }
    encoder.finish()
}

fn source_digest(
    terminal: &GoldAndGearsResonanceRuleBinding,
    contribution: &GoldAndGearsResonanceContribution,
    attachment: GoldAndGearsResonanceCombatAttachment,
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-gold-gears-resonance-combat-source-v1");
    encoder.text(&terminal.rule_id);
    encoder.text(&terminal.owner_id);
    encoder.text(contribution.binding_key());
    encoder.u8(contribution.kind() as u8);
    encoder.u8(attachment as u8);
    let parameters = contribution.parameters_scaled().collect::<Vec<_>>();
    encoder.u32(parameters.len() as u32);
    for value in parameters {
        encoder.i64(value);
    }
    encoder.finish()
}

fn combat_set_digest(bindings: &[GoldAndGearsResonanceCombatBinding]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-gold-gears-resonance-combat-set-v1");
    encoder.u32(bindings.len() as u32);
    for binding in bindings {
        encoder.text(&binding.terminal.rule_id);
        encoder.text(&binding.terminal.owner_id);
        encoder.u8(binding.attachment as u8);
        encoder.digest(binding.source.digest());
    }
    encoder.finish()
}
