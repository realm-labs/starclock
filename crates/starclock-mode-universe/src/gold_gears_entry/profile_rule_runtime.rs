//! Production execution for the five profile-entry mechanic rules.

use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityInventoryId, ActivityOperation,
    ActivityProgramDefinition, ActivityProgramId, ActivityRngStreams, ActivitySlotId,
    ActivityTransactionState, ActivityValue,
};

use crate::id::BlessingId;

use super::{
    GoldAndGearsCurioCategory, GoldAndGearsCurioId, GoldAndGearsCurioOfferContext,
    GoldAndGearsCurioOfferSource, GoldAndGearsEntryError, GoldAndGearsRuntimeInstance,
    GoldAndGearsTrailblazeOffer,
    state_layout::{
        BLESSING_INVENTORY, CURIO_INVENTORY, DEFERRED_EFFECTS_SLOT,
        DEFERRED_PROFILE_RULE_APPLIED_BASE,
    },
};

pub const GOLD_AND_GEARS_PROFILE_RULE_RUNTIME_REVISION: &str =
    "gold-and-gears-profile-entry-rule-runtime-v1";

struct CanonicalBlessingInventory {
    counts: Vec<(BlessingId, u32)>,
    ids: Vec<BlessingId>,
}

struct CanonicalCurioInventory {
    counts: Vec<(GoldAndGearsCurioId, u32)>,
    ids: Vec<GoldAndGearsCurioId>,
}

/// One fully bound profile-entry rule program and its deterministic selections.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsProfileRuleExecution {
    source_rule: Box<str>,
    source_bonus: Box<str>,
    event_id: u32,
    selected_blessings: Box<[BlessingId]>,
    selected_curios: Box<[GoldAndGearsCurioId]>,
    program: ActivityProgramDefinition,
}

impl GoldAndGearsProfileRuleExecution {
    #[must_use]
    pub fn source_rule(&self) -> &str {
        &self.source_rule
    }

    #[must_use]
    pub fn source_bonus(&self) -> &str {
        &self.source_bonus
    }

    #[must_use]
    pub const fn event_id(&self) -> u32 {
        self.event_id
    }

    #[must_use]
    pub fn selected_blessings(&self) -> &[BlessingId] {
        &self.selected_blessings
    }

    #[must_use]
    pub fn selected_curios(&self) -> &[GoldAndGearsCurioId] {
        &self.selected_curios
    }

    #[must_use]
    pub const fn program(&self) -> &ActivityProgramDefinition {
        &self.program
    }
}

impl GoldAndGearsRuntimeInstance {
    /// Compiles the selected Trailblaze Bonus into one guarded Activity program.
    ///
    /// `owned_blessings` and `owned_curios` are an exact authoritative
    /// inventory snapshot. The generated program rechecks every catalog
    /// inventory entry and its exactly-once marker. The caller must compile
    /// and apply it inside the same authoritative Activity/RNG transaction so
    /// a stale snapshot also rolls back the selection draws.
    pub fn compile_profile_entry_rule(
        &self,
        state: &ActivityTransactionState,
        owned_blessings: &[(BlessingId, u32)],
        owned_curios: &[(GoldAndGearsCurioId, u32)],
        rng: &mut ActivityRngStreams,
    ) -> Result<GoldAndGearsProfileRuleExecution, GoldAndGearsEntryError> {
        let plan = self
            .trailblaze_bonus_plan()
            .ok_or(GoldAndGearsEntryError::MissingProfileEntryRule)?;
        validate_rule_binding(plan.source_bonus(), plan.source_rule(), plan.event_id())?;
        let blessings = canonical_blessings(self, owned_blessings)?;
        let curios = canonical_curios(self, owned_curios)?;
        let marker = profile_rule_marker(plan.event_id());
        if counter(state, DEFERRED_EFFECTS_SLOT, marker)? != 0 {
            return Err(GoldAndGearsEntryError::ProfileEntryRuleAlreadyApplied);
        }
        rng.transact(|working| {
            let (selected_blessings, selected_curios) = self.select_profile_rule_offers(
                plan.offers(),
                &blessings.ids,
                &curios.ids,
                working,
            )?;
            let program = self.compile_profile_rule_program(
                plan.event_id(),
                plan.immediate_program(),
                &blessings.counts,
                &curios.counts,
                &selected_blessings,
                &selected_curios,
            )?;
            program
                .validate_against(self.state_definition(), self.graph_definition())
                .map_err(|_| GoldAndGearsEntryError::InvalidProfileEntryRule)?;
            Ok(GoldAndGearsProfileRuleExecution {
                source_rule: plan.source_rule().into(),
                source_bonus: plan.source_bonus().into(),
                event_id: plan.event_id(),
                selected_blessings: selected_blessings.into_boxed_slice(),
                selected_curios: selected_curios.into_boxed_slice(),
                program,
            })
        })
    }

    fn select_profile_rule_offers(
        &self,
        offers: &[GoldAndGearsTrailblazeOffer],
        owned_blessings: &[BlessingId],
        owned_curios: &[GoldAndGearsCurioId],
        rng: &mut ActivityRngStreams,
    ) -> Result<(Vec<BlessingId>, Vec<GoldAndGearsCurioId>), GoldAndGearsEntryError> {
        let mut blessings = Vec::new();
        let mut curios = Vec::new();
        for offer in offers {
            match offer {
                GoldAndGearsTrailblazeOffer::Blessing {
                    choice_count: 1,
                    minimum_rarity: 1,
                    maximum_rarity: 2,
                } => blessings.push(
                    self.select_trailblaze_blessing(owned_blessings, rng)?
                        .ok_or(GoldAndGearsEntryError::InvalidProfileEntryRule)?,
                ),
                GoldAndGearsTrailblazeOffer::Curio { choice_count: 1 } => {
                    curios.extend(self.select_profile_curios(
                        GoldAndGearsCurioCategory::Normal,
                        owned_curios,
                        1,
                        rng,
                    )?);
                }
                GoldAndGearsTrailblazeOffer::CurioCategory { category, count: 1 } => {
                    curios.extend(self.select_profile_curios(
                        curio_category(category)?,
                        owned_curios,
                        1,
                        rng,
                    )?);
                }
                _ => return Err(GoldAndGearsEntryError::InvalidProfileEntryRule),
            }
        }
        if blessings.len() + curios.len()
            != offers
                .iter()
                .map(|offer| match offer {
                    GoldAndGearsTrailblazeOffer::Blessing { choice_count, .. }
                    | GoldAndGearsTrailblazeOffer::Curio { choice_count } => {
                        usize::from(*choice_count)
                    }
                    GoldAndGearsTrailblazeOffer::CurioCategory { count, .. } => usize::from(*count),
                })
                .sum::<usize>()
        {
            return Err(GoldAndGearsEntryError::InvalidProfileEntryRule);
        }
        Ok((blessings, curios))
    }

    fn select_profile_curios(
        &self,
        category: GoldAndGearsCurioCategory,
        owned: &[GoldAndGearsCurioId],
        maximum: u16,
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<GoldAndGearsCurioId>, GoldAndGearsEntryError> {
        let context = GoldAndGearsCurioOfferContext::full_category(
            GoldAndGearsCurioOfferSource::TrailblazeBonus,
            category,
        )
        .ok_or(GoldAndGearsEntryError::InvalidProfileEntryRule)?;
        let selected = self.select_curios(&context, owned, maximum, rng)?;
        if selected.len() != usize::from(maximum) {
            return Err(GoldAndGearsEntryError::InvalidProfileEntryRule);
        }
        Ok(selected.iter().map(|candidate| candidate.id()).collect())
    }

    fn compile_profile_rule_program(
        &self,
        event_id: u32,
        immediate: Option<&ActivityProgramDefinition>,
        owned_blessings: &[(BlessingId, u32)],
        owned_curios: &[(GoldAndGearsCurioId, u32)],
        selected_blessings: &[BlessingId],
        selected_curios: &[GoldAndGearsCurioId],
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let mut operations = vec![require_counter(
            DEFERRED_EFFECTS_SLOT,
            profile_rule_marker(event_id),
            0,
        )];
        operations.extend(self.inventory_snapshot_guards(owned_blessings, owned_curios));
        if let Some(immediate) = immediate {
            operations.extend(guarded_immediate_operations(immediate)?);
        }
        for blessing in selected_blessings {
            operations.extend(
                self.compile_blessing_acquisition(*blessing)?
                    .operations()
                    .iter()
                    .cloned(),
            );
        }
        for curio in selected_curios {
            operations.extend(
                self.compile_curio_acquisition(*curio)?
                    .operations()
                    .iter()
                    .cloned(),
            );
        }
        operations.push(add_counter(
            DEFERRED_EFFECTS_SLOT,
            profile_rule_marker(event_id),
            1,
        ));
        ActivityProgramDefinition::new(
            ActivityProgramId::new(
                0x4E00_0000_u32
                    .checked_add(event_id)
                    .ok_or(GoldAndGearsEntryError::InvalidProfileEntryRule)?,
            )
            .ok_or(GoldAndGearsEntryError::InvalidProfileEntryRule)?,
            operations,
        )
        .map_err(|_| GoldAndGearsEntryError::InvalidProfileEntryRule)
    }

    fn inventory_snapshot_guards(
        &self,
        owned_blessings: &[(BlessingId, u32)],
        owned_curios: &[(GoldAndGearsCurioId, u32)],
    ) -> Vec<ActivityOperation> {
        let blessing_inventory =
            ActivityInventoryId::new(BLESSING_INVENTORY).expect("static inventory is non-zero");
        let curio_inventory =
            ActivityInventoryId::new(CURIO_INVENTORY).expect("static inventory is non-zero");
        let mut operations = self
            .content_runtime
            .blessings
            .definitions()
            .iter()
            .map(|definition| {
                let id = definition.blessing();
                let expected = owned_blessings
                    .binary_search_by_key(&id, |(candidate, _)| *candidate)
                    .ok()
                    .map_or(0, |index| owned_blessings[index].1);
                require_inventory(blessing_inventory, u64::from(id.get()), expected)
            })
            .collect::<Vec<_>>();
        operations.extend(self.curio_definitions().iter().map(|definition| {
            let id = definition.id();
            let expected = owned_curios
                .binary_search_by_key(&id, |(candidate, _)| *candidate)
                .ok()
                .map_or(0, |index| owned_curios[index].1);
            require_inventory(curio_inventory, u64::from(id.get()), expected)
        }));
        operations
    }
}

fn canonical_blessings(
    runtime: &GoldAndGearsRuntimeInstance,
    values: &[(BlessingId, u32)],
) -> Result<CanonicalBlessingInventory, GoldAndGearsEntryError> {
    let mut values = values.to_vec();
    values.sort_by_key(|(id, _)| *id);
    if values.windows(2).any(|pair| pair[0].0 == pair[1].0)
        || values.iter().any(|(id, count)| {
            !(1..=2).contains(count) || runtime.content_runtime.blessings.definition(*id).is_none()
        })
    {
        return Err(GoldAndGearsEntryError::InvalidBlessingInventory);
    }
    let ids = values.iter().map(|(id, _)| *id).collect();
    Ok(CanonicalBlessingInventory {
        counts: values,
        ids,
    })
}

fn canonical_curios(
    runtime: &GoldAndGearsRuntimeInstance,
    values: &[(GoldAndGearsCurioId, u32)],
) -> Result<CanonicalCurioInventory, GoldAndGearsEntryError> {
    let mut values = values.to_vec();
    values.sort_by_key(|(id, _)| *id);
    if values.windows(2).any(|pair| pair[0].0 == pair[1].0)
        || values.iter().any(|(id, count)| {
            *count != 1
                || runtime
                    .curio_definitions()
                    .iter()
                    .all(|definition| definition.id() != *id)
        })
    {
        return Err(GoldAndGearsEntryError::InvalidCurioInventory);
    }
    let ids = values.iter().map(|(id, _)| *id).collect();
    Ok(CanonicalCurioInventory {
        counts: values,
        ids,
    })
}

fn validate_rule_binding(
    bonus: &str,
    rule: &str,
    event: u32,
) -> Result<(), GoldAndGearsEntryError> {
    let valid = [
        (
            "gold-gears.trailblaze-bonus.201",
            "gold-gears.rule.trailblaze-bonus.201",
            3010,
        ),
        (
            "gold-gears.trailblaze-bonus.202",
            "gold-gears.rule.trailblaze-bonus.202",
            3020,
        ),
        (
            "gold-gears.trailblaze-bonus.203",
            "gold-gears.rule.trailblaze-bonus.203",
            3030,
        ),
        (
            "gold-gears.trailblaze-bonus.204",
            "gold-gears.rule.trailblaze-bonus.204",
            3040,
        ),
        (
            "gold-gears.trailblaze-bonus.205",
            "gold-gears.rule.trailblaze-bonus.205",
            3050,
        ),
    ];
    valid
        .contains(&(bonus, rule, event))
        .then_some(())
        .ok_or(GoldAndGearsEntryError::InvalidProfileEntryRule)
}

fn curio_category(value: &str) -> Result<GoldAndGearsCurioCategory, GoldAndGearsEntryError> {
    match value {
        "Negative" => Ok(GoldAndGearsCurioCategory::Negative),
        "ErrorCode" => Ok(GoldAndGearsCurioCategory::ErrorCode),
        _ => Err(GoldAndGearsEntryError::InvalidProfileEntryRule),
    }
}

fn guarded_immediate_operations(
    program: &ActivityProgramDefinition,
) -> Result<Vec<ActivityOperation>, GoldAndGearsEntryError> {
    let [ActivityOperation::AddCounter { slot, key, delta }] = program.operations() else {
        return Err(GoldAndGearsEntryError::InvalidProfileEntryRule);
    };
    let ActivityExpression::Literal(ActivityValue::BoundedInteger(delta)) = delta else {
        return Err(GoldAndGearsEntryError::InvalidProfileEntryRule);
    };
    let upper_exclusive = i64::MAX
        .checked_sub(*delta)
        .and_then(|value| value.checked_add(1))
        .ok_or(GoldAndGearsEntryError::InvalidProfileEntryRule)?;
    Ok(vec![
        ActivityOperation::Require(ActivityCondition::LessThan(
            counter_expression(slot.get(), *key),
            integer(upper_exclusive),
        )),
        program.operations()[0].clone(),
    ])
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
        .binary_search_by_key(&key, |(candidate, _)| *candidate)
        .ok()
        .map_or(0, |index| values[index].1))
}

fn profile_rule_marker(event_id: u32) -> u64 {
    DEFERRED_PROFILE_RULE_APPLIED_BASE + u64::from(event_id)
}

fn require_inventory(
    inventory: ActivityInventoryId,
    content: u64,
    expected: u32,
) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(
        ActivityExpression::InventoryCount { inventory, content },
        integer(i64::from(expected)),
    ))
}

fn require_counter(slot: u32, key: u64, expected: i64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(
        counter_expression(slot, key),
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

fn counter_expression(slot: u32, key: u64) -> ActivityExpression {
    ActivityExpression::CounterValue {
        slot: ActivitySlotId::new(slot).expect("static slot is non-zero"),
        key,
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
