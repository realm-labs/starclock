//! Field-wise account build inheritance over an immutable mapped minimum.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ability::AbilityInvestment,
    spec::{BuildSpecError, CombatantBuildSpec, LightConeLoadout},
};

/// Origin of one field in a resolved account-or-mapped build.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildFieldSource {
    Owned,
    MappedMinimum,
    Combined,
}

/// Why one build was selected at the immutable mode boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildSubstitutionKind {
    Trial,
    StrengthenedOwned,
}

/// Explicit account-side facts that cannot be inferred from opaque contribution IDs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OwnedBuildMinimumFacts {
    contributions_meet_minimum: bool,
}

impl OwnedBuildMinimumFacts {
    #[must_use]
    pub const fn new(contributions_meet_minimum: bool) -> Self {
        Self {
            contributions_meet_minimum,
        }
    }

    #[must_use]
    pub const fn contributions_meet_minimum(self) -> bool {
        self.contributions_meet_minimum
    }
}

/// Auditable receipt for field-wise owned/trial substitution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildSubstitutionReceipt {
    kind: BuildSubstitutionKind,
    progression: BuildFieldSource,
    abilities: BuildFieldSource,
    traces: BuildFieldSource,
    eidolon: BuildFieldSource,
    light_cone: BuildFieldSource,
    contributions: BuildFieldSource,
}

impl BuildSubstitutionReceipt {
    #[must_use]
    pub const fn kind(self) -> BuildSubstitutionKind {
        self.kind
    }

    #[must_use]
    pub const fn progression(self) -> BuildFieldSource {
        self.progression
    }

    #[must_use]
    pub const fn abilities(self) -> BuildFieldSource {
        self.abilities
    }

    #[must_use]
    pub const fn traces(self) -> BuildFieldSource {
        self.traces
    }

    #[must_use]
    pub const fn eidolon(self) -> BuildFieldSource {
        self.eidolon
    }

    #[must_use]
    pub const fn light_cone(self) -> BuildFieldSource {
        self.light_cone
    }

    #[must_use]
    pub const fn contributions(self) -> BuildFieldSource {
        self.contributions
    }
}

/// Exact build input and its substitution receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubstitutedBuild {
    spec: CombatantBuildSpec,
    receipt: BuildSubstitutionReceipt,
}

impl SubstitutedBuild {
    #[must_use]
    pub const fn spec(&self) -> &CombatantBuildSpec {
        &self.spec
    }

    #[must_use]
    pub const fn receipt(&self) -> BuildSubstitutionReceipt {
        self.receipt
    }

    #[must_use]
    pub fn into_spec(self) -> CombatantBuildSpec {
        self.spec
    }
}

/// Resolves an optional owned build over one authored mapped minimum.
///
/// The caller snapshots account state before this function runs. Contribution
/// quality is explicit because contribution IDs are intentionally opaque to the
/// generic compiler. This function never queries or mutates account state.
pub fn substitute_owned_or_trial(
    owned: Option<(&CombatantBuildSpec, OwnedBuildMinimumFacts)>,
    mapped: &CombatantBuildSpec,
) -> Result<SubstitutedBuild, BuildSubstitutionError> {
    let Some((owned, facts)) = owned else {
        return Ok(SubstitutedBuild {
            spec: mapped.clone(),
            receipt: BuildSubstitutionReceipt {
                kind: BuildSubstitutionKind::Trial,
                progression: BuildFieldSource::MappedMinimum,
                abilities: BuildFieldSource::MappedMinimum,
                traces: BuildFieldSource::MappedMinimum,
                eidolon: BuildFieldSource::MappedMinimum,
                light_cone: BuildFieldSource::MappedMinimum,
                contributions: BuildFieldSource::MappedMinimum,
            },
        });
    };
    if owned.form() != mapped.form() {
        return Err(BuildSubstitutionError::FormMismatch);
    }

    let (level, promotion, progression) =
        if owned.level() >= mapped.level() && owned.promotion() >= mapped.promotion() {
            (owned.level(), owned.promotion(), BuildFieldSource::Owned)
        } else {
            (
                mapped.level(),
                mapped.promotion(),
                BuildFieldSource::MappedMinimum,
            )
        };
    let (ability_levels, abilities) = merge_abilities(owned, mapped);
    let (traces, trace_source) = merge_traces(owned, mapped);
    let (eidolon, eidolon_source) = if owned.eidolon() >= mapped.eidolon() {
        (owned.eidolon(), BuildFieldSource::Owned)
    } else {
        (mapped.eidolon(), BuildFieldSource::MappedMinimum)
    };
    let (light_cone, light_cone_source) = select_light_cone(owned, mapped);
    let (contributions, contribution_source) = if facts.contributions_meet_minimum() {
        (owned.contributions().to_vec(), BuildFieldSource::Owned)
    } else {
        (
            mapped.contributions().to_vec(),
            BuildFieldSource::MappedMinimum,
        )
    };

    let mut spec = CombatantBuildSpec::new(owned.form(), level, promotion)
        .with_ability_levels(ability_levels)
        .map_err(BuildSubstitutionError::InvalidSpec)?
        .with_traces(traces)
        .map_err(BuildSubstitutionError::InvalidSpec)?
        .with_eidolon(eidolon)
        .with_contributions(contributions)
        .map_err(BuildSubstitutionError::InvalidSpec)?
        .with_relic_stats(if facts.contributions_meet_minimum() {
            owned.relic_stats()
        } else {
            mapped.relic_stats()
        });
    if let Some(loadout) = light_cone {
        spec = spec.with_light_cone(loadout);
    }
    Ok(SubstitutedBuild {
        spec,
        receipt: BuildSubstitutionReceipt {
            kind: BuildSubstitutionKind::StrengthenedOwned,
            progression,
            abilities,
            traces: trace_source,
            eidolon: eidolon_source,
            light_cone: light_cone_source,
            contributions: contribution_source,
        },
    })
}

fn merge_abilities(
    owned: &CombatantBuildSpec,
    mapped: &CombatantBuildSpec,
) -> (Vec<AbilityInvestment>, BuildFieldSource) {
    let mut levels = BTreeMap::new();
    for investment in mapped.ability_levels() {
        levels.insert(investment.family(), *investment);
    }
    for investment in owned.ability_levels() {
        levels
            .entry(investment.family())
            .and_modify(|selected| {
                if investment.invested() > selected.invested() {
                    *selected = *investment;
                }
            })
            .or_insert(*investment);
    }
    let selected = levels.into_values().collect::<Vec<_>>();
    let source = field_source(
        selected.as_slice() == owned.ability_levels(),
        selected.as_slice() == mapped.ability_levels(),
    );
    (selected, source)
}

fn merge_traces(
    owned: &CombatantBuildSpec,
    mapped: &CombatantBuildSpec,
) -> (Vec<crate::id::TraceNodeId>, BuildFieldSource) {
    let selected = owned
        .traces()
        .iter()
        .chain(mapped.traces())
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let source = field_source(
        selected.as_slice() == owned.traces(),
        selected.as_slice() == mapped.traces(),
    );
    (selected, source)
}

fn field_source(matches_owned: bool, matches_mapped: bool) -> BuildFieldSource {
    match (matches_owned, matches_mapped) {
        (true, false) => BuildFieldSource::Owned,
        (false, true) => BuildFieldSource::MappedMinimum,
        _ => BuildFieldSource::Combined,
    }
}

fn select_light_cone(
    owned: &CombatantBuildSpec,
    mapped: &CombatantBuildSpec,
) -> (Option<LightConeLoadout>, BuildFieldSource) {
    let Some(minimum) = mapped.light_cone() else {
        return (owned.light_cone(), BuildFieldSource::Owned);
    };
    match owned.light_cone() {
        Some(candidate)
            if candidate.level() >= minimum.level()
                && candidate.promotion() >= minimum.promotion() =>
        {
            (Some(candidate), BuildFieldSource::Owned)
        }
        _ => (Some(minimum), BuildFieldSource::MappedMinimum),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildSubstitutionError {
    FormMismatch,
    InvalidSpec(BuildSpecError),
}

impl std::fmt::Display for BuildSubstitutionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid build substitution: {self:?}")
    }
}

impl std::error::Error for BuildSubstitutionError {}

#[cfg(test)]
mod tests {
    use starclock_combat::{AbilityId, UnitDefinitionId, UnitLevel};

    use crate::{
        ability::{AbilityInvestment, AbilityLevel},
        id::{BuildContributionId, TraceNodeId},
        spec::{CombatantBuildSpec, EidolonLevel, PromotionStage},
        substitution::{
            BuildFieldSource, BuildSubstitutionKind, OwnedBuildMinimumFacts,
            substitute_owned_or_trial,
        },
    };

    #[test]
    fn absent_owned_build_selects_trial_without_mutation() {
        let mapped = spec(80, 6, 8, &[1, 2], 0, &[2]);
        let selected = substitute_owned_or_trial(None, &mapped).unwrap();

        assert_eq!(selected.spec(), &mapped);
        assert_eq!(selected.receipt().kind(), BuildSubstitutionKind::Trial);
    }

    #[test]
    fn owned_build_inherits_stronger_fields_and_receives_field_minimums() {
        let owned = spec(70, 5, 10, &[1, 3], 2, &[1]);
        let mapped = spec(80, 6, 8, &[1, 2], 0, &[2]);
        let selected =
            substitute_owned_or_trial(Some((&owned, OwnedBuildMinimumFacts::new(false))), &mapped)
                .unwrap();

        assert_eq!(selected.spec().level().get(), 80);
        assert_eq!(selected.spec().promotion().get(), 6);
        assert_eq!(selected.spec().ability_levels()[0].invested().get(), 10);
        assert_eq!(selected.spec().traces(), &[trace(1), trace(2), trace(3)]);
        assert_eq!(selected.spec().eidolon().get(), 2);
        assert_eq!(selected.spec().contributions(), &[contribution(2)]);
        assert_eq!(
            selected.receipt().progression(),
            BuildFieldSource::MappedMinimum
        );
        assert_eq!(selected.receipt().abilities(), BuildFieldSource::Owned);
        assert_eq!(selected.receipt().traces(), BuildFieldSource::Combined);
        assert_eq!(selected.receipt().eidolon(), BuildFieldSource::Owned);
        assert_eq!(
            selected.receipt().contributions(),
            BuildFieldSource::MappedMinimum
        );
    }

    #[test]
    fn mismatched_forms_are_rejected() {
        let owned = spec(80, 6, 8, &[], 0, &[]);
        let mapped = CombatantBuildSpec::new(form(2), level(80), promotion(6));
        assert!(
            substitute_owned_or_trial(Some((&owned, OwnedBuildMinimumFacts::new(true))), &mapped,)
                .is_err()
        );
    }

    fn spec(
        raw_level: u8,
        raw_promotion: u8,
        ability_level: u8,
        traces: &[u32],
        eidolon: u8,
        contributions: &[u32],
    ) -> CombatantBuildSpec {
        CombatantBuildSpec::new(form(1), level(raw_level), promotion(raw_promotion))
            .with_ability_levels(vec![AbilityInvestment::new(
                ability(1),
                AbilityLevel::new(ability_level).unwrap(),
            )])
            .unwrap()
            .with_traces(traces.iter().copied().map(trace).collect())
            .unwrap()
            .with_eidolon(EidolonLevel::new(eidolon).unwrap())
            .with_contributions(contributions.iter().copied().map(contribution).collect())
            .unwrap()
    }

    fn form(raw: u32) -> UnitDefinitionId {
        UnitDefinitionId::new(raw).unwrap()
    }

    fn level(raw: u8) -> UnitLevel {
        UnitLevel::new(raw).unwrap()
    }

    fn promotion(raw: u8) -> PromotionStage {
        PromotionStage::new(raw).unwrap()
    }

    fn ability(raw: u32) -> AbilityId {
        AbilityId::new(raw).unwrap()
    }

    fn trace(raw: u32) -> TraceNodeId {
        TraceNodeId::new(raw).unwrap()
    }

    fn contribution(raw: u32) -> BuildContributionId {
        BuildContributionId::new(raw).unwrap()
    }
}
