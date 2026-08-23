use std::collections::{BTreeMap, BTreeSet};

use starclock_combat::Ratio;

use crate::{
    CurrencyWarsBondId, CurrencyWarsDeployment, CurrencyWarsEquipmentId,
    CurrencyWarsEquipmentLoadout, CurrencyWarsPositionKind, CurrencyWarsRoleId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsBondMember {
    RosterRole(CurrencyWarsRoleId),
    ExternalAuthoredRole(CurrencyWarsRoleId),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsBondSelectionRule {
    DeployedRole(CurrencyWarsRoleId),
    EquippedEquipment(CurrencyWarsEquipmentId),
    GrantedFrontTrait(CurrencyWarsRoleId),
    DefaultModule,
    Module(u32),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsBondActivation {
    GreaterEqualThan,
    ExplicitSubTraitSelection,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsBondRecompute {
    OrderedRosterMutationBeforeBattleProjection,
    ExplicitSubTraitSelectionChange,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsBondPropertyKind {
    AllDamage,
    AllDamageSecondary,
    BackPower,
    DamageOverTime,
    ElementDamage,
    FrontPower,
    Hp,
    Healing,
    InsertDamage,
    LuckChance,
    LuckDamage,
    NormalDamage,
    Shield,
    SkillDamage,
    Speed,
    UltimateDamage,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsBondPropertyScope {
    BondMembers,
    AllDeployed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBondPropertyContribution {
    pub scope: CurrencyWarsBondPropertyScope,
    pub kind: CurrencyWarsBondPropertyKind,
    pub value: Ratio,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsActiveBondProperty {
    pub bond: CurrencyWarsBondId,
    pub targets: Box<[CurrencyWarsRoleId]>,
    pub property: CurrencyWarsBondPropertyContribution,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBondLevel {
    pub stable_key: Box<str>,
    pub source_id: Box<str>,
    pub level: u8,
    pub threshold: u8,
    pub threshold_semantics: Box<str>,
    pub property_bind_type: Box<str>,
    pub property_parameters_json: Box<str>,
    pub properties: Box<[CurrencyWarsBondPropertyContribution]>,
    pub effect_ids: Box<[Box<str>]>,
    pub trait_member_properties_json: Box<str>,
    pub all_member_properties_json: Box<str>,
    pub override_battle_event_properties_json: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBondContribution {
    pub stable_key: Box<str>,
    pub source_id: Box<str>,
    pub level: Option<u8>,
    pub scope: Box<str>,
    pub activation: Box<str>,
    pub ordered_effects: Box<[Box<str>]>,
    pub parameters_json: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBond {
    pub id: CurrencyWarsBondId,
    pub stable_key: Box<str>,
    pub source_id: Box<str>,
    pub parent: Option<CurrencyWarsBondId>,
    pub members: Box<[CurrencyWarsBondMember]>,
    pub selection_rules: Box<[CurrencyWarsBondSelectionRule]>,
    pub activation: CurrencyWarsBondActivation,
    pub recompute: CurrencyWarsBondRecompute,
    pub trait_effect_ids: Box<[u32]>,
    pub battle_event_ids: Box<[u32]>,
    pub levels: Box<[CurrencyWarsBondLevel]>,
    pub contributions: Box<[CurrencyWarsBondContribution]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBondCatalog {
    bonds: Box<[CurrencyWarsBond]>,
    contributions: Box<[CurrencyWarsBondContribution]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsActiveBond {
    pub id: CurrencyWarsBondId,
    pub member_count: u8,
    pub level: u8,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CurrencyWarsBondResolutionContext {
    pub selected_subtraits: BTreeMap<CurrencyWarsBondId, CurrencyWarsBondId>,
    pub additional_member_counts: BTreeMap<CurrencyWarsBondId, u8>,
    pub granted_front_traits: BTreeSet<CurrencyWarsRoleId>,
    pub module_id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBondSnapshot {
    pub active_bonds: Box<[CurrencyWarsActiveBond]>,
    pub selected_subtraits: Box<[(CurrencyWarsBondId, CurrencyWarsBondId)]>,
    pub contributions: Box<[CurrencyWarsBondContribution]>,
    pub properties: Box<[CurrencyWarsActiveBondProperty]>,
    pub trait_effect_ids: Box<[u32]>,
    pub battle_event_ids: Box<[u32]>,
}

impl CurrencyWarsBondCatalog {
    pub fn new(
        mut bonds: Vec<CurrencyWarsBond>,
        mut contributions: Vec<CurrencyWarsBondContribution>,
    ) -> Result<Self, CurrencyWarsBondCatalogError> {
        bonds.sort_by_key(|value| value.id);
        contributions.sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        validate(&bonds, &contributions)?;
        Ok(Self {
            bonds: bonds.into_boxed_slice(),
            contributions: contributions.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn bonds(&self) -> &[CurrencyWarsBond] {
        &self.bonds
    }

    #[must_use]
    pub fn contributions(&self) -> &[CurrencyWarsBondContribution] {
        &self.contributions
    }

    #[must_use]
    pub fn bond(&self, id: CurrencyWarsBondId) -> Option<&CurrencyWarsBond> {
        self.bonds
            .binary_search_by_key(&id, |bond| bond.id)
            .ok()
            .map(|index| &self.bonds[index])
    }

    #[must_use]
    pub fn selection_eligible(
        &self,
        parent: CurrencyWarsBondId,
        child: CurrencyWarsBondId,
        deployment: &CurrencyWarsDeployment,
        loadout: &CurrencyWarsEquipmentLoadout,
        context: &CurrencyWarsBondResolutionContext,
    ) -> bool {
        let Some(child) = self.bond(child) else {
            return false;
        };
        child.parent == Some(parent)
            && child
                .selection_rules
                .iter()
                .any(|rule| explicit_rule_active(*rule, deployment, loadout, context))
    }

    #[must_use]
    pub fn resolve(
        &self,
        deployment: &CurrencyWarsDeployment,
        loadout: &CurrencyWarsEquipmentLoadout,
        context: &CurrencyWarsBondResolutionContext,
    ) -> CurrencyWarsBondSnapshot {
        let deployed_roles = deployment
            .positions()
            .values()
            .map(|state| state.role())
            .collect::<BTreeSet<_>>();
        let mut counts = BTreeMap::new();
        let mut levels = BTreeMap::new();
        for bond in self.bonds.iter().filter(|bond| bond.parent.is_none()) {
            let direct = bond
                .members
                .iter()
                .filter(|member| {
                    matches!(member, CurrencyWarsBondMember::RosterRole(role)
                        if deployed_roles.contains(role))
                })
                .count();
            let count = direct.saturating_add(usize::from(
                context
                    .additional_member_counts
                    .get(&bond.id)
                    .copied()
                    .unwrap_or_default(),
            ));
            let count = u8::try_from(count).unwrap_or(u8::MAX);
            counts.insert(bond.id, count);
            if let Some(level) = active_level(bond, count) {
                levels.insert(bond.id, level.level);
            }
        }

        let mut selections = BTreeMap::new();
        for parent in self.bonds.iter().filter(|bond| bond.parent.is_none()) {
            if !levels.contains_key(&parent.id) {
                continue;
            }
            let children = self
                .bonds
                .iter()
                .filter(|bond| bond.parent == Some(parent.id))
                .collect::<Vec<_>>();
            let automatic = children
                .iter()
                .copied()
                .find(|child| {
                    child.selection_rules.iter().any(|rule| {
                        matches!(rule, CurrencyWarsBondSelectionRule::Module(module)
                            if *module == context.module_id)
                    })
                })
                .or_else(|| {
                    children.iter().copied().find(|child| {
                        child
                            .selection_rules
                            .contains(&CurrencyWarsBondSelectionRule::DefaultModule)
                    })
                });
            let selected = automatic.or_else(|| {
                context
                    .selected_subtraits
                    .get(&parent.id)
                    .and_then(|id| self.bond(*id))
                    .filter(|child| {
                        self.selection_eligible(parent.id, child.id, deployment, loadout, context)
                    })
            });
            if let Some(child) = selected {
                selections.insert(parent.id, child.id);
                let count = counts.get(&parent.id).copied().unwrap_or_default();
                counts.insert(child.id, count);
                if let Some(level) = active_level(child, count) {
                    levels.insert(child.id, level.level);
                }
            }
        }

        let mut active_bonds = Vec::new();
        let mut contributions = Vec::new();
        let mut properties = Vec::new();
        let mut trait_effect_ids = BTreeSet::new();
        let mut battle_event_ids = BTreeSet::new();
        for bond in &self.bonds {
            let Some(&level) = levels.get(&bond.id) else {
                continue;
            };
            let count = counts.get(&bond.id).copied().unwrap_or_default();
            active_bonds.push(CurrencyWarsActiveBond {
                id: bond.id,
                member_count: count,
                level,
            });
            contributions.extend(
                bond.contributions
                    .iter()
                    .filter(|contribution| contribution.level == Some(level))
                    .cloned(),
            );
            if let Some(active) = bond
                .levels
                .iter()
                .find(|candidate| candidate.level == level)
            {
                let member_roles = bond_member_targets(bond, &deployed_roles, &selections, self);
                let all_roles = deployed_roles.iter().copied().collect::<Box<[_]>>();
                properties.extend(active.properties.iter().copied().map(|property| {
                    let targets = match property.scope {
                        CurrencyWarsBondPropertyScope::BondMembers => member_roles.clone(),
                        CurrencyWarsBondPropertyScope::AllDeployed => all_roles.clone(),
                    };
                    CurrencyWarsActiveBondProperty {
                        bond: bond.id,
                        targets,
                        property,
                    }
                }));
            }
            trait_effect_ids.extend(bond.trait_effect_ids.iter().copied());
            battle_event_ids.extend(bond.battle_event_ids.iter().copied());
        }
        CurrencyWarsBondSnapshot {
            active_bonds: active_bonds.into_boxed_slice(),
            selected_subtraits: selections.into_iter().collect(),
            contributions: contributions.into_boxed_slice(),
            properties: properties.into_boxed_slice(),
            trait_effect_ids: trait_effect_ids.into_iter().collect(),
            battle_event_ids: battle_event_ids.into_iter().collect(),
        }
    }
}

fn bond_member_targets(
    bond: &CurrencyWarsBond,
    deployed_roles: &BTreeSet<CurrencyWarsRoleId>,
    selections: &BTreeMap<CurrencyWarsBondId, CurrencyWarsBondId>,
    catalog: &CurrencyWarsBondCatalog,
) -> Box<[CurrencyWarsRoleId]> {
    let owner = bond
        .parent
        .and_then(|parent| catalog.bond(parent))
        .unwrap_or(bond);
    let mut targets = owner
        .members
        .iter()
        .filter_map(|member| match member {
            CurrencyWarsBondMember::RosterRole(role) if deployed_roles.contains(role) => {
                Some(*role)
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    if let Some(selected) = selections.get(&owner.id).and_then(|id| catalog.bond(*id)) {
        targets.extend(
            selected
                .selection_rules
                .iter()
                .filter_map(|rule| match rule {
                    CurrencyWarsBondSelectionRule::DeployedRole(role)
                        if deployed_roles.contains(role) =>
                    {
                        Some(*role)
                    }
                    _ => None,
                }),
        );
    }
    targets.into_iter().collect()
}

fn active_level(bond: &CurrencyWarsBond, count: u8) -> Option<&CurrencyWarsBondLevel> {
    bond.levels
        .iter()
        .filter(|level| level.threshold <= count)
        .max_by_key(|level| level.threshold)
}

fn explicit_rule_active(
    rule: CurrencyWarsBondSelectionRule,
    deployment: &CurrencyWarsDeployment,
    loadout: &CurrencyWarsEquipmentLoadout,
    context: &CurrencyWarsBondResolutionContext,
) -> bool {
    match rule {
        CurrencyWarsBondSelectionRule::DeployedRole(role) => deployment
            .positions()
            .values()
            .any(|state| state.role() == role),
        CurrencyWarsBondSelectionRule::EquippedEquipment(equipment) => {
            loadout.slots().iter().any(|(slot, equipped)| {
                *equipped == equipment
                    && deployment
                        .positions()
                        .values()
                        .any(|state| state.role() == slot.role())
            })
        }
        CurrencyWarsBondSelectionRule::GrantedFrontTrait(role) => {
            context.granted_front_traits.contains(&role)
                && deployment.positions().iter().any(|(position, state)| {
                    position.kind() == CurrencyWarsPositionKind::Front && state.role() == role
                })
        }
        CurrencyWarsBondSelectionRule::DefaultModule | CurrencyWarsBondSelectionRule::Module(_) => {
            false
        }
    }
}

#[cfg(test)]
impl CurrencyWarsBondCatalog {
    pub(crate) fn test_fixture(role: CurrencyWarsRoleId) -> Self {
        let contribution = CurrencyWarsBondContribution {
            stable_key: "bond-contribution.1".into(),
            source_id: "1".into(),
            level: Some(1),
            scope: "fixture".into(),
            activation: "fixture".into(),
            ordered_effects: Box::new([]),
            parameters_json: "{}".into(),
        };
        let subtrait_contribution = CurrencyWarsBondContribution {
            stable_key: "bond-contribution.2".into(),
            source_id: "2".into(),
            level: Some(1),
            scope: "fixture-subtrait".into(),
            activation: "fixture-subtrait".into(),
            ordered_effects: Box::new([]),
            parameters_json: "{}".into(),
        };
        Self::new(
            vec![
                CurrencyWarsBond {
                    id: CurrencyWarsBondId::new(1).unwrap(),
                    stable_key: "bond.1".into(),
                    source_id: "1".into(),
                    parent: None,
                    members: Box::new([CurrencyWarsBondMember::RosterRole(role)]),
                    selection_rules: Box::new([]),
                    activation: CurrencyWarsBondActivation::GreaterEqualThan,
                    recompute:
                        CurrencyWarsBondRecompute::OrderedRosterMutationBeforeBattleProjection,
                    trait_effect_ids: Box::new([11, 30_021]),
                    battle_event_ids: Box::new([12]),
                    levels: Box::new([fixture_level("bond-level.1", "1:1")]),
                    contributions: Box::new([contribution.clone()]),
                },
                CurrencyWarsBond {
                    id: CurrencyWarsBondId::new(2).unwrap(),
                    stable_key: "bond.subtrait.2".into(),
                    source_id: "2".into(),
                    parent: CurrencyWarsBondId::new(1),
                    members: Box::new([]),
                    selection_rules: Box::new([CurrencyWarsBondSelectionRule::DeployedRole(role)]),
                    activation: CurrencyWarsBondActivation::ExplicitSubTraitSelection,
                    recompute: CurrencyWarsBondRecompute::ExplicitSubTraitSelectionChange,
                    trait_effect_ids: Box::new([21]),
                    battle_event_ids: Box::new([22]),
                    levels: Box::new([fixture_level("bond-level.2", "2:1")]),
                    contributions: Box::new([subtrait_contribution.clone()]),
                },
            ],
            vec![contribution, subtrait_contribution],
        )
        .expect("test Bond catalog is valid")
    }
}

#[cfg(test)]
fn fixture_level(stable_key: &'static str, source_id: &'static str) -> CurrencyWarsBondLevel {
    CurrencyWarsBondLevel {
        stable_key: stable_key.into(),
        source_id: source_id.into(),
        level: 1,
        threshold: 1,
        threshold_semantics: "fixture".into(),
        property_bind_type: "fixture".into(),
        property_parameters_json: "[]".into(),
        properties: Box::new([]),
        effect_ids: Box::new([]),
        trait_member_properties_json: "[]".into(),
        all_member_properties_json: "[]".into(),
        override_battle_event_properties_json: "[]".into(),
    }
}

fn validate(
    bonds: &[CurrencyWarsBond],
    contributions: &[CurrencyWarsBondContribution],
) -> Result<(), CurrencyWarsBondCatalogError> {
    if bonds.is_empty() || bonds.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return Err(error("Currency Wars Bond catalog is empty or duplicated"));
    }
    if contributions.is_empty()
        || contributions
            .windows(2)
            .any(|pair| pair[0].stable_key == pair[1].stable_key)
    {
        return Err(error(
            "Currency Wars Bond contribution catalog is empty or duplicated",
        ));
    }
    for bond in bonds {
        if bond.levels.is_empty()
            || bond.contributions.is_empty()
            || bond
                .levels
                .windows(2)
                .any(|pair| pair[0].threshold >= pair[1].threshold)
        {
            return Err(error("Currency Wars Bond progression is invalid"));
        }
        let levels = bond
            .levels
            .iter()
            .map(|value| value.level)
            .collect::<BTreeSet<_>>();
        if levels.len() != bond.levels.len()
            || bond
                .contributions
                .iter()
                .filter_map(|value| value.level)
                .any(|level| !levels.contains(&level))
        {
            return Err(error("Currency Wars Bond contribution level is invalid"));
        }
        let contribution_keys = bond
            .contributions
            .iter()
            .map(|value| value.stable_key.as_ref())
            .collect::<BTreeSet<_>>();
        if contribution_keys.len() != bond.contributions.len() {
            return Err(error("Currency Wars Bond contribution is duplicated"));
        }
        match bond.activation {
            CurrencyWarsBondActivation::GreaterEqualThan
                if bond.parent.is_some() || !bond.selection_rules.is_empty() =>
            {
                return Err(error(
                    "Currency Wars main Bond selection metadata is invalid",
                ));
            }
            CurrencyWarsBondActivation::ExplicitSubTraitSelection
                if bond.parent.is_none()
                    || !bond.members.is_empty()
                    || bond.selection_rules.is_empty() =>
            {
                return Err(error(
                    "Currency Wars sub-Bond selection metadata is invalid",
                ));
            }
            _ => {}
        }
    }
    let ids = bonds.iter().map(|bond| bond.id).collect::<BTreeSet<_>>();
    if bonds
        .iter()
        .filter_map(|bond| bond.parent)
        .any(|parent| !ids.contains(&parent))
    {
        return Err(error("Currency Wars sub-Bond parent is missing"));
    }
    Ok(())
}

impl CurrencyWarsBondCatalog {
    pub fn assemble(
        mut definitions: Vec<CurrencyWarsBondDefinition>,
        levels: Vec<(Box<str>, CurrencyWarsBondLevel)>,
        contributions: Vec<(Option<Box<str>>, CurrencyWarsBondContribution)>,
    ) -> Result<Self, CurrencyWarsBondCatalogError> {
        let mut levels_by_bond = BTreeMap::<Box<str>, Vec<CurrencyWarsBondLevel>>::new();
        for (bond, level) in levels {
            levels_by_bond.entry(bond).or_default().push(level);
        }
        let contribution_parents = definitions
            .iter()
            .flat_map(|definition| {
                definition
                    .contribution_ids
                    .iter()
                    .map(|id| (id.as_ref(), definition.stable_key.as_ref()))
            })
            .collect::<BTreeMap<_, _>>();
        let mut contributions_by_bond =
            BTreeMap::<Box<str>, Vec<CurrencyWarsBondContribution>>::new();
        let all_contributions = contributions
            .iter()
            .map(|(_, contribution)| contribution.clone())
            .collect::<Vec<_>>();
        for (bond, contribution) in contributions {
            if let Some(expected_parent) =
                contribution_parents.get(contribution.stable_key.as_ref())
            {
                let bond = bond.ok_or_else(|| {
                    error("Currency Wars referenced Bond contribution has no parent")
                })?;
                if bond.as_ref() != *expected_parent {
                    return Err(error("Currency Wars Bond contribution parent is invalid"));
                }
                contributions_by_bond
                    .entry(bond)
                    .or_default()
                    .push(contribution);
            }
        }
        let mut bonds = Vec::with_capacity(definitions.len());
        definitions.sort_by_key(|value| value.id);
        for definition in definitions {
            let mut bond_levels = levels_by_bond
                .remove(definition.stable_key.as_ref())
                .ok_or_else(|| error("Currency Wars Bond levels are missing"))?;
            bond_levels.sort_by_key(|value| value.level);
            let mut bond_contributions = contributions_by_bond
                .remove(definition.stable_key.as_ref())
                .ok_or_else(|| error("Currency Wars Bond contributions are missing"))?;
            bond_contributions.sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
            let level_keys = bond_levels
                .iter()
                .map(|value| value.stable_key.as_ref())
                .collect::<BTreeSet<_>>();
            let contribution_keys = bond_contributions
                .iter()
                .map(|value| value.stable_key.as_ref())
                .collect::<BTreeSet<_>>();
            if definition
                .level_ids
                .iter()
                .any(|id| !level_keys.contains(id.as_ref()))
                || definition
                    .contribution_ids
                    .iter()
                    .any(|id| !contribution_keys.contains(id.as_ref()))
                || definition.level_ids.len() != level_keys.len()
                || definition.contribution_ids.len() != contribution_keys.len()
            {
                return Err(error("Currency Wars Bond child reference is invalid"));
            }
            bonds.push(CurrencyWarsBond {
                id: definition.id,
                stable_key: definition.stable_key,
                source_id: definition.source_id,
                parent: definition.parent,
                members: definition.members,
                selection_rules: definition.selection_rules,
                activation: definition.activation,
                recompute: definition.recompute,
                trait_effect_ids: definition.trait_effect_ids,
                battle_event_ids: definition.battle_event_ids,
                levels: bond_levels.into_boxed_slice(),
                contributions: bond_contributions.into_boxed_slice(),
            });
        }
        if !levels_by_bond.is_empty() || !contributions_by_bond.is_empty() {
            return Err(error("Currency Wars Bond child has no parent"));
        }
        CurrencyWarsBondCatalog::new(bonds, all_contributions)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBondDefinition {
    pub id: CurrencyWarsBondId,
    pub stable_key: Box<str>,
    pub source_id: Box<str>,
    pub parent: Option<CurrencyWarsBondId>,
    pub members: Box<[CurrencyWarsBondMember]>,
    pub selection_rules: Box<[CurrencyWarsBondSelectionRule]>,
    pub level_ids: Box<[Box<str>]>,
    pub activation: CurrencyWarsBondActivation,
    pub recompute: CurrencyWarsBondRecompute,
    pub contribution_ids: Box<[Box<str>]>,
    pub trait_effect_ids: Box<[u32]>,
    pub battle_event_ids: Box<[u32]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBondCatalogError {
    message: Box<str>,
}
impl std::fmt::Display for CurrencyWarsBondCatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}
impl std::error::Error for CurrencyWarsBondCatalogError {}
fn error(message: &'static str) -> CurrencyWarsBondCatalogError {
    CurrencyWarsBondCatalogError {
        message: message.into(),
    }
}
