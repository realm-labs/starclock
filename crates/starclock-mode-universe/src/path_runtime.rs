//! Runtime Path selection, Resonance availability and Formation contributions.

use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityInventoryId, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivitySlotId, ActivityValue,
};

use crate::{
    blessing_runtime::BlessingContributionSet,
    catalog::UniverseCatalog,
    digest::Encoder,
    id::{PathId, ResonanceId},
    path::{ExactParameter, ResonanceDefinition, ResonanceKind},
};

pub const PATH_RUNTIME_REVISION: &str = "standard-universe-path-runtime-v1";
pub const FORMATION_SELECTION_THRESHOLDS: [u8; 3] = [6, 10, 14];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResonanceRuleContribution {
    id: ResonanceId,
    kind: ResonanceKind,
    source_binding_key: Box<str>,
    rule_key: Box<str>,
    mechanic_tags: Box<[Box<str>]>,
    parameters: Box<[ExactParameter]>,
    energy_max: ExactParameter,
    initial_energy: ExactParameter,
}

impl ResonanceRuleContribution {
    fn from_definition(definition: &ResonanceDefinition) -> Self {
        Self {
            id: definition.id(),
            kind: definition.kind(),
            source_binding_key: definition.source_binding_key().into(),
            rule_key: definition.rule_key().into(),
            mechanic_tags: definition.mechanic_tags().to_vec().into_boxed_slice(),
            parameters: definition.parameters().to_vec().into_boxed_slice(),
            energy_max: definition.energy_max(),
            initial_energy: definition.initial_energy(),
        }
    }

    #[must_use]
    pub const fn id(&self) -> ResonanceId {
        self.id
    }
    #[must_use]
    pub const fn kind(&self) -> ResonanceKind {
        self.kind
    }
    #[must_use]
    pub fn source_binding_key(&self) -> &str {
        &self.source_binding_key
    }
    #[must_use]
    pub fn rule_key(&self) -> &str {
        &self.rule_key
    }
    #[must_use]
    pub fn mechanic_tags(&self) -> &[Box<str>] {
        &self.mechanic_tags
    }
    #[must_use]
    pub fn parameters(&self) -> &[ExactParameter] {
        &self.parameters
    }
    #[must_use]
    pub const fn energy_max(&self) -> ExactParameter {
        self.energy_max
    }
    #[must_use]
    pub const fn initial_energy(&self) -> ExactParameter {
        self.initial_energy
    }

    pub fn initial_action_state(&self) -> Result<ResonanceActionState, PathRuntimeError> {
        if self.kind != ResonanceKind::Resonance {
            return Err(PathRuntimeError::FormationHasNoAction);
        }
        ResonanceActionState::new(self.initial_energy, self.energy_max)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathPassiveContribution {
    path: PathId,
    stable_key: Box<str>,
    buff_type: u32,
    unlock_policy_key: Box<str>,
}

impl PathPassiveContribution {
    #[must_use]
    pub const fn path(&self) -> PathId {
        self.path
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn buff_type(&self) -> u32 {
        self.buff_type
    }
    #[must_use]
    pub fn unlock_policy_key(&self) -> &str {
        &self.unlock_policy_key
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathContributionSet {
    passive: PathPassiveContribution,
    selected_path_blessings: u8,
    resonance: Option<ResonanceRuleContribution>,
    formations: Box<[ResonanceRuleContribution]>,
    next_formation_threshold: Option<u8>,
    digest: [u8; 32],
}

impl PathContributionSet {
    #[must_use]
    pub const fn passive(&self) -> &PathPassiveContribution {
        &self.passive
    }
    #[must_use]
    pub const fn selected_path_blessings(&self) -> u8 {
        self.selected_path_blessings
    }
    #[must_use]
    pub const fn resonance(&self) -> Option<&ResonanceRuleContribution> {
        self.resonance.as_ref()
    }
    #[must_use]
    pub fn formations(&self) -> &[ResonanceRuleContribution] {
        &self.formations
    }
    #[must_use]
    pub const fn next_formation_threshold(&self) -> Option<u8> {
        self.next_formation_threshold
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResonanceEnergy(i64);

impl ResonanceEnergy {
    pub fn from_scaled(coefficient: i64, scale: u8) -> Result<Self, PathRuntimeError> {
        if coefficient < 0 || scale > 6 {
            return Err(PathRuntimeError::InvalidEnergy);
        }
        let multiplier = 10_i64
            .checked_pow(u32::from(6 - scale))
            .ok_or(PathRuntimeError::EnergyOverflow)?;
        coefficient
            .checked_mul(multiplier)
            .map(Self)
            .ok_or(PathRuntimeError::EnergyOverflow)
    }
    #[must_use]
    pub const fn raw_six_decimal(self) -> i64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResonanceActionState {
    energy: ResonanceEnergy,
    maximum: ResonanceEnergy,
}

impl ResonanceActionState {
    fn new(initial: ExactParameter, maximum: ExactParameter) -> Result<Self, PathRuntimeError> {
        let initial = energy(initial)?;
        let maximum = energy(maximum)?;
        if maximum.0 == 0 || initial > maximum {
            return Err(PathRuntimeError::InvalidEnergy);
        }
        Ok(Self {
            energy: initial,
            maximum,
        })
    }
    #[must_use]
    pub const fn energy(self) -> ResonanceEnergy {
        self.energy
    }
    #[must_use]
    pub const fn maximum(self) -> ResonanceEnergy {
        self.maximum
    }
    #[must_use]
    pub const fn can_activate(self) -> bool {
        self.energy.0 >= self.maximum.0
    }
    pub fn gain(&mut self, amount: ResonanceEnergy) -> Result<ResonanceEnergy, PathRuntimeError> {
        let before = self.energy.0;
        self.energy.0 = self
            .energy
            .0
            .checked_add(amount.0)
            .ok_or(PathRuntimeError::EnergyOverflow)?
            .min(self.maximum.0);
        Ok(ResonanceEnergy(self.energy.0 - before))
    }
    pub fn activate(&mut self) -> Result<ResonanceEnergy, PathRuntimeError> {
        if !self.can_activate() {
            return Err(PathRuntimeError::ResonanceNotReady);
        }
        self.energy.0 = self
            .energy
            .0
            .checked_sub(self.maximum.0)
            .ok_or(PathRuntimeError::EnergyOverflow)?;
        Ok(self.maximum)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PathRuntimeDefinition {
    passive: PathPassiveContribution,
    blessings: Box<[crate::id::BlessingId]>,
    resonance: ResonanceRuleContribution,
    formations: [ResonanceRuleContribution; 3],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathRuntimeCatalog {
    definitions: Box<[PathRuntimeDefinition]>,
    digest: [u8; 32],
}

impl PathRuntimeCatalog {
    pub fn compile(catalog: &UniverseCatalog) -> Result<Self, PathRuntimeError> {
        let mut definitions = Vec::with_capacity(catalog.paths().len());
        for path in catalog.paths() {
            let resonance = catalog
                .resonance(path.resonance())
                .ok_or(PathRuntimeError::MissingResonance(path.resonance()))?;
            if resonance.path() != path.id()
                || resonance.kind() != ResonanceKind::Resonance
                || resonance.threshold() != 3
                || resonance.energy_max().coefficient() <= 0
            {
                return Err(PathRuntimeError::InvalidResonance(resonance.id()));
            }
            let formations = path
                .formations()
                .iter()
                .map(|id| {
                    let definition = catalog
                        .resonance(*id)
                        .ok_or(PathRuntimeError::MissingResonance(*id))?;
                    if definition.path() != path.id()
                        || definition.kind() != ResonanceKind::Formation
                        || definition.threshold() != 0
                    {
                        return Err(PathRuntimeError::InvalidFormation(*id));
                    }
                    Ok(ResonanceRuleContribution::from_definition(definition))
                })
                .collect::<Result<Vec<_>, PathRuntimeError>>()?
                .try_into()
                .map_err(|_| PathRuntimeError::InvalidPath(path.id()))?;
            definitions.push(PathRuntimeDefinition {
                passive: PathPassiveContribution {
                    path: path.id(),
                    stable_key: path.stable_key().into(),
                    buff_type: path.buff_type(),
                    unlock_policy_key: path.unlock_policy_key().into(),
                },
                blessings: path.blessings().to_vec().into_boxed_slice(),
                resonance: ResonanceRuleContribution::from_definition(resonance),
                formations,
            });
        }
        definitions.sort_by_key(|definition| definition.passive.path);
        if definitions.len() != 9
            || definitions
                .windows(2)
                .any(|pair| pair[0].passive.path == pair[1].passive.path)
        {
            return Err(PathRuntimeError::InvalidDenominator);
        }
        let digest = catalog_digest(&definitions);
        Ok(Self {
            definitions: definitions.into_boxed_slice(),
            digest,
        })
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.definitions.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    pub fn contributions(
        &self,
        selected: PathId,
        blessings: &BlessingContributionSet,
        formations: &[(ResonanceId, u32)],
    ) -> Result<PathContributionSet, PathRuntimeError> {
        let definition = self
            .definition(selected)
            .ok_or(PathRuntimeError::UnknownPath(selected))?;
        let blessing_count = blessings
            .entries()
            .iter()
            .filter(|entry| entry.path() == selected)
            .count();
        let blessing_count =
            u8::try_from(blessing_count).map_err(|_| PathRuntimeError::TooManyPathBlessings)?;
        let mut selected_formations = Vec::new();
        for (id, stacks) in formations {
            if *stacks != 1 {
                return Err(PathRuntimeError::InvalidFormationStack(*id));
            }
            let formation = definition
                .formations
                .iter()
                .find(|candidate| candidate.id == *id)
                .ok_or(PathRuntimeError::FormationPathMismatch(*id))?;
            selected_formations.push(formation.clone());
        }
        selected_formations.sort_by_key(ResonanceRuleContribution::id);
        if selected_formations
            .windows(2)
            .any(|pair| pair[0].id == pair[1].id)
            || selected_formations.len() > 3
            || selected_formations.len() > unlocked_formation_slots(blessing_count)
        {
            return Err(PathRuntimeError::InvalidFormationSelection);
        }
        let resonance = (blessing_count >= 3).then(|| definition.resonance.clone());
        let next_formation_threshold = FORMATION_SELECTION_THRESHOLDS
            .get(selected_formations.len())
            .copied();
        let passive = definition.passive.clone();
        let digest = contribution_digest(
            &passive,
            blessing_count,
            resonance.as_ref(),
            &selected_formations,
            next_formation_threshold,
        );
        Ok(PathContributionSet {
            passive,
            selected_path_blessings: blessing_count,
            resonance,
            formations: selected_formations.into_boxed_slice(),
            next_formation_threshold,
            digest,
        })
    }

    pub(crate) fn formation_selection_options(
        &self,
        bindings: FormationSelectionBindings,
        skip_option: ActivityOptionId,
        option_id: impl Fn(ResonanceId) -> ActivityOptionId,
        settlement: &[ActivityOperation],
    ) -> Vec<ActivityOptionDefinition> {
        let mut options = Vec::with_capacity(28);
        let mut due = Vec::with_capacity(self.definitions.len());
        for (path_priority, definition) in self.definitions.iter().enumerate() {
            let path_due = formation_due(definition, bindings);
            due.push(path_due.clone());
            for (formation_priority, formation) in definition.formations.iter().enumerate() {
                let enabled = ActivityCondition::All(
                    vec![
                        path_due.clone(),
                        equals(
                            ActivityExpression::InventoryCount {
                                inventory: bindings.formation_inventory,
                                content: u64::from(formation.id.get()),
                            },
                            0,
                        ),
                    ]
                    .into_boxed_slice(),
                );
                let mut operations = vec![ActivityOperation::AddInventory {
                    inventory: bindings.formation_inventory,
                    content: u64::from(formation.id.get()),
                    count: integer(1),
                }];
                operations.extend_from_slice(settlement);
                options.push(ActivityOptionDefinition::new(
                    option_id(formation.id),
                    (path_priority * 3 + formation_priority) as i32,
                    enabled,
                    operations,
                ));
            }
        }
        options.push(ActivityOptionDefinition::new(
            skip_option,
            i32::MAX,
            ActivityCondition::Not(Box::new(ActivityCondition::Any(due.into_boxed_slice()))),
            settlement.to_vec(),
        ));
        options
    }

    fn definition(&self, id: PathId) -> Option<&PathRuntimeDefinition> {
        self.definitions
            .binary_search_by_key(&id, |definition| definition.passive.path)
            .ok()
            .map(|index| &self.definitions[index])
    }
}

#[derive(Clone, Copy)]
pub(crate) struct FormationSelectionBindings {
    pub(crate) selected_path_slot: ActivitySlotId,
    pub(crate) path_blessing_count_slot: ActivitySlotId,
    pub(crate) formation_inventory: ActivityInventoryId,
}

fn formation_due(
    definition: &PathRuntimeDefinition,
    bindings: FormationSelectionBindings,
) -> ActivityCondition {
    let selected_count = definition
        .formations
        .iter()
        .map(|formation| ActivityExpression::InventoryCount {
            inventory: bindings.formation_inventory,
            content: u64::from(formation.id.get()),
        })
        .reduce(|left, right| ActivityExpression::Add(Box::new(left), Box::new(right)))
        .expect("every Path has three Formations");
    let stages = FORMATION_SELECTION_THRESHOLDS
        .iter()
        .enumerate()
        .map(|(selected, threshold)| {
            ActivityCondition::All(
                vec![
                    ActivityCondition::Not(Box::new(ActivityCondition::LessThan(
                        ActivityExpression::CounterValue {
                            slot: bindings.path_blessing_count_slot,
                            key: u64::from(definition.passive.path.get()),
                        },
                        integer(i64::from(*threshold)),
                    ))),
                    equals(selected_count.clone(), selected as i64),
                ]
                .into_boxed_slice(),
            )
        })
        .collect::<Vec<_>>();
    ActivityCondition::All(
        vec![
            equals_optional(
                bindings.selected_path_slot,
                u64::from(definition.passive.path.get()),
            ),
            ActivityCondition::Any(stages.into_boxed_slice()),
        ]
        .into_boxed_slice(),
    )
}

fn unlocked_formation_slots(blessings: u8) -> usize {
    FORMATION_SELECTION_THRESHOLDS
        .iter()
        .filter(|threshold| blessings >= **threshold)
        .count()
}

fn energy(value: ExactParameter) -> Result<ResonanceEnergy, PathRuntimeError> {
    ResonanceEnergy::from_scaled(value.coefficient(), value.scale())
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn equals(expression: ActivityExpression, value: i64) -> ActivityCondition {
    ActivityCondition::Equal(expression, integer(value))
}

fn equals_optional(slot: ActivitySlotId, value: u64) -> ActivityCondition {
    ActivityCondition::Equal(
        ActivityExpression::Slot(slot),
        ActivityExpression::Literal(ActivityValue::OptionalId(Some(value))),
    )
}

fn catalog_digest(definitions: &[PathRuntimeDefinition]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-path-runtime-catalog-v1");
    encoder.text(PATH_RUNTIME_REVISION);
    encoder.u32(definitions.len() as u32);
    for definition in definitions {
        encoder.u32(definition.passive.path.get());
        encoder.text(&definition.passive.stable_key);
        encoder.u32(definition.passive.buff_type);
        encoder.text(&definition.passive.unlock_policy_key);
        encoder.u32(definition.blessings.len() as u32);
        for blessing in &definition.blessings {
            encoder.u32(blessing.get());
        }
        encode_rule(&mut encoder, &definition.resonance);
        for formation in &definition.formations {
            encode_rule(&mut encoder, formation);
        }
    }
    for threshold in FORMATION_SELECTION_THRESHOLDS {
        encoder.u8(threshold);
    }
    encoder.finish()
}

fn contribution_digest(
    passive: &PathPassiveContribution,
    blessing_count: u8,
    resonance: Option<&ResonanceRuleContribution>,
    formations: &[ResonanceRuleContribution],
    next_threshold: Option<u8>,
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-path-contribution-set-v1");
    encoder.u32(passive.path.get());
    encoder.text(&passive.stable_key);
    encoder.u32(passive.buff_type);
    encoder.text(&passive.unlock_policy_key);
    encoder.u8(blessing_count);
    encoder.u8(u8::from(resonance.is_some()));
    if let Some(resonance) = resonance {
        encode_rule(&mut encoder, resonance);
    }
    encoder.u32(formations.len() as u32);
    for formation in formations {
        encode_rule(&mut encoder, formation);
    }
    encoder.u8(next_threshold.unwrap_or(0));
    encoder.finish()
}

fn encode_rule(encoder: &mut Encoder, contribution: &ResonanceRuleContribution) {
    encoder.u32(contribution.id.get());
    encoder.u8(contribution.kind as u8);
    encoder.text(&contribution.source_binding_key);
    encoder.text(&contribution.rule_key);
    encoder.i64(contribution.energy_max.coefficient());
    encoder.u8(contribution.energy_max.scale());
    encoder.i64(contribution.initial_energy.coefficient());
    encoder.u8(contribution.initial_energy.scale());
    encoder.u32(contribution.mechanic_tags.len() as u32);
    for tag in &contribution.mechanic_tags {
        encoder.text(tag);
    }
    encoder.u32(contribution.parameters.len() as u32);
    for parameter in &contribution.parameters {
        encoder.i64(parameter.coefficient());
        encoder.u8(parameter.scale());
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathRuntimeError {
    InvalidDenominator,
    InvalidPath(PathId),
    UnknownPath(PathId),
    MissingResonance(ResonanceId),
    InvalidResonance(ResonanceId),
    InvalidFormation(ResonanceId),
    FormationPathMismatch(ResonanceId),
    InvalidFormationStack(ResonanceId),
    InvalidFormationSelection,
    TooManyPathBlessings,
    FormationHasNoAction,
    InvalidEnergy,
    EnergyOverflow,
    ResonanceNotReady,
}
