use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsGlobalTaskWave {
    Any,
    First,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsGlobalTaskTargetPopulation {
    AllAlliesIncludingUnselectable,
    SelectableAllies,
    UnselectableAllies,
    InvocationSelected,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsGlobalTaskPredicate {
    Any,
    InvocationTrait,
    InvocationTraitWhenEnabled,
    InvocationModifier,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsGlobalTaskFormationOrder {
    Authored,
    Ascending,
    Descending,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsGlobalTaskMaximumTargets {
    All,
    Invocation,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsGlobalTaskPresentationReason {
    EnergyBarPresentation,
    CameraPresentation,
    PursuedDamagePresentationTiming,
    MonsterDropPresentationEffect,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsGlobalTaskNodeCount {
    pub node_type: Box<str>,
    pub count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsGlobalModifierTemplate {
    pub wave: CurrencyWarsGlobalTaskWave,
    pub target_population: CurrencyWarsGlobalTaskTargetPopulation,
    pub predicate: CurrencyWarsGlobalTaskPredicate,
    pub formation_order: CurrencyWarsGlobalTaskFormationOrder,
    pub maximum_targets: CurrencyWarsGlobalTaskMaximumTargets,
    pub modifier_parameter: Box<str>,
    pub predicate_parameter: Option<Box<str>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyWarsGlobalTaskTemplateDefinition {
    ApplyModifier(CurrencyWarsGlobalModifierTemplate),
    PresentationOnly(CurrencyWarsGlobalTaskPresentationReason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsGlobalTaskTemplate {
    pub stable_key: Box<str>,
    pub definition: CurrencyWarsGlobalTaskTemplateDefinition,
    pub node_type_counts: Box<[CurrencyWarsGlobalTaskNodeCount]>,
    pub typed_node_count: u32,
    pub add_modifier_node_count: u32,
    pub ordered_shape_sha256: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsGlobalTaskTemplateLibrary {
    templates: Box<[CurrencyWarsGlobalTaskTemplate]>,
    mechanical_shape_sha256: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsGlobalTaskCandidate {
    pub stable_key: Box<str>,
    pub formation: u16,
    pub selectable: bool,
    pub traits: BTreeSet<Box<str>>,
    pub modifiers: BTreeSet<Box<str>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsGlobalTaskInvocation {
    pub wave_number: u16,
    pub selected_population: Option<CurrencyWarsGlobalTaskTargetPopulation>,
    pub check_predicate: bool,
    pub maximum_targets: Option<u16>,
    pub modifier_name: Box<str>,
    pub predicate_value: Option<Box<str>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsGlobalTaskModifierApplication {
    pub target_key: Box<str>,
    pub modifier_name: Box<str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurrencyWarsGlobalTaskExecutionError {
    InvalidLibrary,
    UnknownTemplate,
    PresentationOnly,
    MissingTargetPopulation,
    InvalidTargetPopulation,
    MissingMaximumTargets,
    MissingModifier,
    MissingPredicateValue,
    InvalidCandidates,
}

impl CurrencyWarsGlobalTaskTemplateLibrary {
    pub fn new(
        mut templates: Vec<CurrencyWarsGlobalTaskTemplate>,
        mechanical_shape_sha256: Box<str>,
    ) -> Result<Self, CurrencyWarsGlobalTaskExecutionError> {
        templates.sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        if templates.is_empty()
            || !valid_sha256(&mechanical_shape_sha256)
            || templates
                .windows(2)
                .any(|pair| pair[0].stable_key == pair[1].stable_key)
            || templates.iter().any(invalid_template)
        {
            return Err(CurrencyWarsGlobalTaskExecutionError::InvalidLibrary);
        }
        Ok(Self {
            templates: templates.into_boxed_slice(),
            mechanical_shape_sha256,
        })
    }

    #[must_use]
    pub fn templates(&self) -> &[CurrencyWarsGlobalTaskTemplate] {
        &self.templates
    }

    #[must_use]
    pub fn mechanical_shape_sha256(&self) -> &str {
        &self.mechanical_shape_sha256
    }

    #[must_use]
    pub fn template(&self, stable_key: &str) -> Option<&CurrencyWarsGlobalTaskTemplate> {
        self.templates
            .binary_search_by(|template| template.stable_key.as_ref().cmp(stable_key))
            .ok()
            .map(|index| &self.templates[index])
    }

    pub fn execute(
        &self,
        stable_key: &str,
        invocation: &CurrencyWarsGlobalTaskInvocation,
        candidates: &[CurrencyWarsGlobalTaskCandidate],
    ) -> Result<
        Box<[CurrencyWarsGlobalTaskModifierApplication]>,
        CurrencyWarsGlobalTaskExecutionError,
    > {
        let template = self
            .template(stable_key)
            .ok_or(CurrencyWarsGlobalTaskExecutionError::UnknownTemplate)?;
        let CurrencyWarsGlobalTaskTemplateDefinition::ApplyModifier(definition) =
            &template.definition
        else {
            return Err(CurrencyWarsGlobalTaskExecutionError::PresentationOnly);
        };
        if invocation.wave_number == 0
            || candidates
                .iter()
                .any(|candidate| candidate.stable_key.is_empty())
            || candidates.iter().enumerate().any(|(index, candidate)| {
                candidates[index + 1..]
                    .iter()
                    .any(|other| other.stable_key == candidate.stable_key)
            })
        {
            return Err(CurrencyWarsGlobalTaskExecutionError::InvalidCandidates);
        }
        if definition.wave == CurrencyWarsGlobalTaskWave::First && invocation.wave_number != 1 {
            return Ok(Box::new([]));
        }
        if invocation.modifier_name.is_empty() {
            return Err(CurrencyWarsGlobalTaskExecutionError::MissingModifier);
        }
        let population = match definition.target_population {
            CurrencyWarsGlobalTaskTargetPopulation::InvocationSelected => invocation
                .selected_population
                .ok_or(CurrencyWarsGlobalTaskExecutionError::MissingTargetPopulation)?,
            population => population,
        };
        if population == CurrencyWarsGlobalTaskTargetPopulation::InvocationSelected {
            return Err(CurrencyWarsGlobalTaskExecutionError::InvalidTargetPopulation);
        }
        let predicate_value = match definition.predicate {
            CurrencyWarsGlobalTaskPredicate::Any => None,
            CurrencyWarsGlobalTaskPredicate::InvocationTraitWhenEnabled
                if !invocation.check_predicate =>
            {
                None
            }
            _ => Some(
                invocation
                    .predicate_value
                    .as_deref()
                    .filter(|value| !value.is_empty())
                    .ok_or(CurrencyWarsGlobalTaskExecutionError::MissingPredicateValue)?,
            ),
        };
        let mut selected = candidates
            .iter()
            .filter(|candidate| population_accepts(population, candidate))
            .filter(|candidate| predicate_accepts(definition.predicate, predicate_value, candidate))
            .collect::<Vec<_>>();
        selected.sort_by(|left, right| match definition.formation_order {
            CurrencyWarsGlobalTaskFormationOrder::Authored
            | CurrencyWarsGlobalTaskFormationOrder::Ascending => {
                (left.formation, &left.stable_key).cmp(&(right.formation, &right.stable_key))
            }
            CurrencyWarsGlobalTaskFormationOrder::Descending => right
                .formation
                .cmp(&left.formation)
                .then_with(|| left.stable_key.cmp(&right.stable_key)),
        });
        let maximum = match definition.maximum_targets {
            CurrencyWarsGlobalTaskMaximumTargets::All => selected.len(),
            CurrencyWarsGlobalTaskMaximumTargets::Invocation => usize::from(
                invocation
                    .maximum_targets
                    .ok_or(CurrencyWarsGlobalTaskExecutionError::MissingMaximumTargets)?,
            ),
        };
        Ok(selected
            .into_iter()
            .take(maximum)
            .map(|candidate| CurrencyWarsGlobalTaskModifierApplication {
                target_key: candidate.stable_key.clone(),
                modifier_name: invocation.modifier_name.clone(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice())
    }
}

fn invalid_template(template: &CurrencyWarsGlobalTaskTemplate) -> bool {
    template.stable_key.is_empty()
        || template.typed_node_count == 0
        || !valid_sha256(&template.ordered_shape_sha256)
        || template.node_type_counts.is_empty()
        || template
            .node_type_counts
            .windows(2)
            .any(|pair| pair[0].node_type >= pair[1].node_type)
        || template
            .node_type_counts
            .iter()
            .any(|count| count.node_type.is_empty() || count.count == 0)
        || template
            .node_type_counts
            .iter()
            .map(|count| count.count)
            .sum::<u32>()
            != template.typed_node_count
        || match &template.definition {
            CurrencyWarsGlobalTaskTemplateDefinition::ApplyModifier(definition) => {
                template.add_modifier_node_count == 0
                    || definition.modifier_parameter.is_empty()
                    || matches!(
                        definition.predicate,
                        CurrencyWarsGlobalTaskPredicate::InvocationTrait
                            | CurrencyWarsGlobalTaskPredicate::InvocationTraitWhenEnabled
                            | CurrencyWarsGlobalTaskPredicate::InvocationModifier
                    ) && definition
                        .predicate_parameter
                        .as_deref()
                        .is_none_or(str::is_empty)
            }
            CurrencyWarsGlobalTaskTemplateDefinition::PresentationOnly(_) => {
                template.add_modifier_node_count != 0
            }
        }
}

fn population_accepts(
    population: CurrencyWarsGlobalTaskTargetPopulation,
    candidate: &CurrencyWarsGlobalTaskCandidate,
) -> bool {
    match population {
        CurrencyWarsGlobalTaskTargetPopulation::AllAlliesIncludingUnselectable => true,
        CurrencyWarsGlobalTaskTargetPopulation::SelectableAllies => candidate.selectable,
        CurrencyWarsGlobalTaskTargetPopulation::UnselectableAllies => !candidate.selectable,
        CurrencyWarsGlobalTaskTargetPopulation::InvocationSelected => false,
    }
}

fn predicate_accepts(
    predicate: CurrencyWarsGlobalTaskPredicate,
    value: Option<&str>,
    candidate: &CurrencyWarsGlobalTaskCandidate,
) -> bool {
    match predicate {
        CurrencyWarsGlobalTaskPredicate::Any => true,
        CurrencyWarsGlobalTaskPredicate::InvocationTrait
        | CurrencyWarsGlobalTaskPredicate::InvocationTraitWhenEnabled => {
            value.is_none_or(|value| candidate.traits.contains(value))
        }
        CurrencyWarsGlobalTaskPredicate::InvocationModifier => {
            value.is_some_and(|value| candidate.modifiers.contains(value))
        }
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
