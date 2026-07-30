//! Production Activity and battle projections for Auxiliary Conundrum rules.

use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityInventoryId, ActivityOperation,
    ActivityProgramDefinition, ActivityProgramId, ActivityRngStreams, ActivitySlotId,
    ActivityTransactionState, ActivityValue,
};

use super::{
    GoldAndGearsConundrumEffect, GoldAndGearsCurioCategory, GoldAndGearsCurioId,
    GoldAndGearsCurioOfferContext, GoldAndGearsCurioOfferSource, GoldAndGearsEntryError,
    GoldAndGearsRuntimeInstance,
    state_layout::{
        CONUNDRUM_AUXILIARY_KEY, CONUNDRUM_SLOT, CURIO_INVENTORY,
        DEFERRED_CONUNDRUM_PLANE_APPLIED_BASE, DEFERRED_CONUNDRUM_RULE_APPLIED_BASE,
        DEFERRED_CONUNDRUM_RULE_VALUE_BASE, DEFERRED_EFFECTS_SLOT,
    },
};

/// Revision of the six-rule Auxiliary Conundrum execution boundary.
pub const GOLD_AND_GEARS_AUXILIARY_CONUNDRUM_RULE_REVISION: &str =
    "gold-and-gears-auxiliary-conundrum-rule-runtime-v1";

const START_PROGRAM_BASE: u32 = 0x4f10_0000;
const PLANE_PROGRAM_BASE: u32 = 0x4f20_0000;

/// Immutable battle input produced by one active Auxiliary rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoldAndGearsAuxiliaryBattleContribution {
    ThirdPlaneFormationExtrapolation {
        source_rule: Box<str>,
        count: u8,
    },
    SecondPlaneBossPhaseThree {
        source_rule: Box<str>,
        encounter_groups: Box<[Box<str>]>,
    },
    EffectiveBlessingsPerPath {
        source_rule: Box<str>,
        delta: i64,
        minimum: u8,
    },
}

impl GoldAndGearsAuxiliaryBattleContribution {
    #[must_use]
    pub fn source_rule(&self) -> &str {
        match self {
            Self::ThirdPlaneFormationExtrapolation { source_rule, .. }
            | Self::SecondPlaneBossPhaseThree { source_rule, .. }
            | Self::EffectiveBlessingsPerPath { source_rule, .. } => source_rule,
        }
    }
}

/// One cumulative initialization program and its immutable battle inputs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsAuxiliaryConundrumExecution {
    source_rules: Box<[Box<str>]>,
    program: ActivityProgramDefinition,
    battle_contributions: Box<[GoldAndGearsAuxiliaryBattleContribution]>,
}

impl GoldAndGearsAuxiliaryConundrumExecution {
    #[must_use]
    pub fn source_rules(&self) -> &[Box<str>] {
        &self.source_rules
    }

    #[must_use]
    pub const fn program(&self) -> &ActivityProgramDefinition {
        &self.program
    }

    #[must_use]
    pub fn battle_contributions(&self) -> &[GoldAndGearsAuxiliaryBattleContribution] {
        &self.battle_contributions
    }
}

/// One exact plane-entry Negative Curio selection and guarded Activity program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsAuxiliaryPlaneEntryExecution {
    source_rule: Box<str>,
    plane_layer: u8,
    selected_curio: GoldAndGearsCurioId,
    program: ActivityProgramDefinition,
}

impl GoldAndGearsAuxiliaryPlaneEntryExecution {
    #[must_use]
    pub fn source_rule(&self) -> &str {
        &self.source_rule
    }

    #[must_use]
    pub const fn plane_layer(&self) -> u8 {
        self.plane_layer
    }

    #[must_use]
    pub const fn selected_curio(&self) -> GoldAndGearsCurioId {
        self.selected_curio
    }

    #[must_use]
    pub const fn program(&self) -> &ActivityProgramDefinition {
        &self.program
    }
}

impl GoldAndGearsRuntimeInstance {
    /// Compiles every active cumulative Auxiliary rule into one guarded
    /// Activity program plus immutable battle inputs.
    pub fn compile_auxiliary_conundrum_rules(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<Option<GoldAndGearsAuxiliaryConundrumExecution>, GoldAndGearsEntryError> {
        let selected = self.auxiliary_conundrum();
        if selected == 0 {
            return Ok(None);
        }
        if counter(state, CONUNDRUM_SLOT, CONUNDRUM_AUXILIARY_KEY)? != i64::from(selected) {
            return Err(GoldAndGearsEntryError::AuxiliaryConundrumStateMismatch);
        }
        let mut source_rules = Vec::new();
        let mut operations = Vec::new();
        let mut battle_contributions = Vec::new();
        for contribution in self.conundrum_contributions() {
            let Some(level) = auxiliary_level(contribution.source_level()) else {
                continue;
            };
            if level > selected {
                return Err(GoldAndGearsEntryError::InvalidAuxiliaryConundrumRule);
            }
            let source_rule = format!("gold-gears.rule.conundrum.auxiliary.{level}");
            let marker = rule_marker(level);
            if counter(state, DEFERRED_EFFECTS_SLOT, marker)? != 0 {
                return Err(GoldAndGearsEntryError::AuxiliaryConundrumRuleAlreadyApplied);
            }
            operations.push(require_counter(DEFERRED_EFFECTS_SLOT, marker, 0));
            lower_effect(
                level,
                &source_rule,
                contribution.effect(),
                &mut operations,
                &mut battle_contributions,
            )?;
            operations.push(add_counter(DEFERRED_EFFECTS_SLOT, marker, 1));
            source_rules.push(source_rule.into_boxed_str());
        }
        if source_rules.len() != usize::from(selected) {
            return Err(GoldAndGearsEntryError::InvalidAuxiliaryConundrumRule);
        }
        let program = ActivityProgramDefinition::new(
            ActivityProgramId::new(START_PROGRAM_BASE + u32::from(selected))
                .expect("reserved program identity is non-zero"),
            operations,
        )
        .map_err(|_| GoldAndGearsEntryError::InvalidAuxiliaryConundrumRule)?;
        program
            .validate_against(self.state_definition(), self.graph_definition())
            .map_err(|_| GoldAndGearsEntryError::InvalidAuxiliaryConundrumRule)?;
        Ok(Some(GoldAndGearsAuxiliaryConundrumExecution {
            source_rules: source_rules.into_boxed_slice(),
            program,
            battle_contributions: battle_contributions.into_boxed_slice(),
        }))
    }

    /// Selects the exact level-five Negative Curio for one plane-entry
    /// transaction and compiles its guarded acquisition program.
    ///
    /// The caller must compile and apply this program inside the same
    /// authoritative Activity/RNG transaction.
    pub fn compile_auxiliary_conundrum_plane_entry(
        &self,
        state: &ActivityTransactionState,
        plane_layer: u8,
        owned_curios: &[(GoldAndGearsCurioId, u32)],
        rng: &mut ActivityRngStreams,
    ) -> Result<GoldAndGearsAuxiliaryPlaneEntryExecution, GoldAndGearsEntryError> {
        if self.auxiliary_conundrum() < 5 || !(1..=3).contains(&plane_layer) {
            return Err(GoldAndGearsEntryError::InvalidAuxiliaryConundrumRule);
        }
        let owned = canonical_curios(self, owned_curios)?;
        let marker = plane_marker(plane_layer);
        if counter(state, DEFERRED_EFFECTS_SLOT, marker)? != 0 {
            return Err(GoldAndGearsEntryError::AuxiliaryConundrumRuleAlreadyApplied);
        }
        rng.transact(|working| {
            let context = GoldAndGearsCurioOfferContext::full_category(
                GoldAndGearsCurioOfferSource::AuxiliaryConundrum,
                GoldAndGearsCurioCategory::Negative,
            )
            .ok_or(GoldAndGearsEntryError::InvalidAuxiliaryConundrumRule)?;
            let selected = self.select_curios(&context, &owned.ids, 1, working)?;
            let [selected] = selected.as_ref() else {
                return Err(GoldAndGearsEntryError::InvalidAuxiliaryConundrumRule);
            };
            let mut operations = vec![require_counter(DEFERRED_EFFECTS_SLOT, marker, 0)];
            operations.extend(self.curio_inventory_guards(&owned.counts));
            operations.extend(
                self.compile_curio_acquisition(selected.id())?
                    .operations()
                    .iter()
                    .cloned(),
            );
            operations.push(add_counter(DEFERRED_EFFECTS_SLOT, marker, 1));
            let program = ActivityProgramDefinition::new(
                ActivityProgramId::new(PLANE_PROGRAM_BASE + u32::from(plane_layer))
                    .expect("reserved program identity is non-zero"),
                operations,
            )
            .map_err(|_| GoldAndGearsEntryError::InvalidAuxiliaryConundrumRule)?;
            program
                .validate_against(self.state_definition(), self.graph_definition())
                .map_err(|_| GoldAndGearsEntryError::InvalidAuxiliaryConundrumRule)?;
            Ok(GoldAndGearsAuxiliaryPlaneEntryExecution {
                source_rule: "gold-gears.rule.conundrum.auxiliary.5".into(),
                plane_layer,
                selected_curio: selected.id(),
                program,
            })
        })
    }

    fn curio_inventory_guards(
        &self,
        owned: &[(GoldAndGearsCurioId, u32)],
    ) -> Vec<ActivityOperation> {
        let inventory =
            ActivityInventoryId::new(CURIO_INVENTORY).expect("static inventory is non-zero");
        self.curio_definitions()
            .iter()
            .map(|definition| {
                let id = definition.id();
                let expected = owned
                    .binary_search_by_key(&id, |(candidate, _)| *candidate)
                    .ok()
                    .map_or(0, |index| owned[index].1);
                ActivityOperation::Require(ActivityCondition::Equal(
                    ActivityExpression::InventoryCount {
                        inventory,
                        content: u64::from(id.get()),
                    },
                    integer(i64::from(expected)),
                ))
            })
            .collect()
    }
}

fn lower_effect(
    level: u8,
    source_rule: &str,
    effect: &GoldAndGearsConundrumEffect,
    operations: &mut Vec<ActivityOperation>,
    battle: &mut Vec<GoldAndGearsAuxiliaryBattleContribution>,
) -> Result<(), GoldAndGearsEntryError> {
    match (level, effect) {
        (1, GoldAndGearsConundrumEffect::FormationExtrapolationCount(count)) if *count == 1 => {
            record_values(level, &[*count as i64], operations);
            battle.push(
                GoldAndGearsAuxiliaryBattleContribution::ThirdPlaneFormationExtrapolation {
                    source_rule: source_rule.into(),
                    count: *count,
                },
            );
        }
        (2, GoldAndGearsConundrumEffect::SecondPlaneBossPhaseThree(groups))
            if groups.len() == 12 =>
        {
            record_values(level, &[12], operations);
            battle.push(
                GoldAndGearsAuxiliaryBattleContribution::SecondPlaneBossPhaseThree {
                    source_rule: source_rule.into(),
                    encounter_groups: groups.clone(),
                },
            );
        }
        (3, GoldAndGearsConundrumEffect::BlessingResetCost(cost)) if *cost == 20 => {
            record_values(level, &[*cost], operations);
        }
        (
            4,
            GoldAndGearsConundrumEffect::InitialResources {
                countdown_delta: -1,
                dice_reroll_delta: -1,
                cosmic_fragment_delta: -100,
            },
        ) => record_values(level, &[1, 1, 100], operations),
        (5, GoldAndGearsConundrumEffect::NegativeCuriosPerPlane(count)) if *count == 1 => {
            record_values(level, &[*count as i64], operations);
        }
        (
            6,
            GoldAndGearsConundrumEffect::EffectiveBlessingsPerPath {
                delta: -1,
                minimum: 0,
            },
        ) => {
            record_values(level, &[1, 0], operations);
            battle.push(
                GoldAndGearsAuxiliaryBattleContribution::EffectiveBlessingsPerPath {
                    source_rule: source_rule.into(),
                    delta: -1,
                    minimum: 0,
                },
            );
        }
        _ => return Err(GoldAndGearsEntryError::InvalidAuxiliaryConundrumRule),
    }
    Ok(())
}

fn record_values(level: u8, values: &[i64], operations: &mut Vec<ActivityOperation>) {
    for (index, value) in values.iter().enumerate() {
        let key = DEFERRED_CONUNDRUM_RULE_VALUE_BASE
            + u64::from(level) * 16
            + u64::try_from(index).expect("bounded value index");
        operations.push(require_counter(DEFERRED_EFFECTS_SLOT, key, 0));
        operations.push(add_counter(DEFERRED_EFFECTS_SLOT, key, *value));
    }
}

fn canonical_curios(
    runtime: &GoldAndGearsRuntimeInstance,
    values: &[(GoldAndGearsCurioId, u32)],
) -> Result<CanonicalCurios, GoldAndGearsEntryError> {
    let mut counts = values.to_vec();
    counts.sort_unstable_by_key(|entry| entry.0);
    if counts.windows(2).any(|pair| pair[0].0 == pair[1].0)
        || counts.iter().any(|(id, count)| {
            *count != 1
                || runtime
                    .curio_definitions()
                    .iter()
                    .all(|definition| definition.id() != *id)
        })
    {
        return Err(GoldAndGearsEntryError::InvalidCurioInventory);
    }
    Ok(CanonicalCurios {
        ids: counts.iter().map(|entry| entry.0).collect(),
        counts,
    })
}

struct CanonicalCurios {
    ids: Vec<GoldAndGearsCurioId>,
    counts: Vec<(GoldAndGearsCurioId, u32)>,
}

fn auxiliary_level(source: &str) -> Option<u8> {
    source
        .strip_prefix("gold-gears.conundrum-level.auxiliary.")?
        .parse()
        .ok()
}

fn rule_marker(level: u8) -> u64 {
    DEFERRED_CONUNDRUM_RULE_APPLIED_BASE + u64::from(level)
}

fn plane_marker(plane_layer: u8) -> u64 {
    DEFERRED_CONUNDRUM_PLANE_APPLIED_BASE + u64::from(plane_layer)
}

fn counter(
    state: &ActivityTransactionState,
    slot: u32,
    key: u64,
) -> Result<i64, GoldAndGearsEntryError> {
    let Some(ActivityValue::BoundedCounterMap(values)) =
        state.slot(ActivitySlotId::new(slot).expect("static slot is non-zero"))
    else {
        return Err(GoldAndGearsEntryError::InvalidActivityState);
    };
    Ok(values
        .binary_search_by_key(&key, |entry| entry.0)
        .ok()
        .map_or(0, |index| values[index].1))
}

fn require_counter(slot: u32, key: u64, expected: i64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(
        ActivityExpression::CounterValue {
            slot: ActivitySlotId::new(slot).expect("static slot is non-zero"),
            key,
        },
        integer(expected),
    ))
}

fn add_counter(slot: u32, key: u64, delta: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: ActivitySlotId::new(slot).expect("static slot is non-zero"),
        key,
        delta: integer(delta),
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
