use starclock_activity::{
    ActivityCondition, ActivityEdgeId, ActivityExpression, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivityScope, ActivitySlotDefinition,
    ActivitySlotId, ActivityStateSource, ActivityStateVisibility, ActivityValue, SlotCarryPolicy,
};

use crate::{
    BaseballerCatalog, BaseballerEquipment, BaseballerEquipmentId, BaseballerEquipmentKind,
    BaseballerProfile, BaseballerProgressionSnapshot, BaseballerRecipe, BaseballerRecipeInputKind,
    BaseballerStage,
};

const ACQUIRE_SUFFIX: u64 = 1;
const UPGRADE_SUFFIX: u64 = 2;
const OPTION_MULTIPLIER: u64 = 4;
const SKIP_OPTION: u64 = 9_000_000_001;

/// Activity slots used by one Baseballer stage inventory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BaseballerInventoryBindings {
    pub levels: ActivitySlotId,
    pub used_weapon_slots: ActivitySlotId,
    pub unlocked_weapon_slots: ActivitySlotId,
    pub used_accessory_slots: ActivitySlotId,
    pub unlocked_accessory_slots: ActivitySlotId,
}

impl BaseballerInventoryBindings {
    pub fn definitions(
        self,
        catalog: &BaseballerCatalog,
        profile: &BaseballerProfile,
        stage: &BaseballerStage,
    ) -> Result<Vec<ActivitySlotDefinition>, BaseballerInventoryError> {
        self.definitions_with_progression(catalog, profile, stage, None)
    }

    pub fn definitions_with_progression(
        self,
        catalog: &BaseballerCatalog,
        profile: &BaseballerProfile,
        stage: &BaseballerStage,
        progression: Option<&BaseballerProgressionSnapshot>,
    ) -> Result<Vec<ActivitySlotDefinition>, BaseballerInventoryError> {
        let (initial_weapon_level_bonus, additional_accessory_slots) =
            progression.map_or((0, 0), |snapshot| {
                (
                    snapshot.initial_weapon_level_bonus,
                    snapshot.additional_accessory_slots,
                )
            });
        if progression.is_some_and(|snapshot| snapshot.profile != profile.id) {
            return Err(BaseballerInventoryError::InvalidProgressionProfile);
        }
        let initial_weapon_level = 1_u8
            .checked_add(initial_weapon_level_bonus)
            .ok_or(BaseballerInventoryError::InvalidInitialInventory)?;
        let unlocked_accessory_slots = profile
            .initially_unlocked_accessory_slots
            .checked_add(additional_accessory_slots)
            .filter(|slots| *slots <= profile.accessory_slots)
            .ok_or(BaseballerInventoryError::InvalidInitialInventory)?;
        if stage.profile != profile.id
            || stage.initial_weapons.len() > usize::from(profile.initially_unlocked_weapon_slots)
            || stage.initial_weapons.iter().any(|id| {
                catalog
                    .equipment_by_id(*id)
                    .is_none_or(|item| item.maximum_level < initial_weapon_level)
            })
        {
            return Err(BaseballerInventoryError::InvalidInitialInventory);
        }
        let mut initial_levels = stage
            .initial_weapons
            .iter()
            .map(|id| (u64::from(id.get()), i64::from(initial_weapon_level)))
            .collect::<Vec<_>>();
        initial_levels.sort_unstable_by_key(|item| item.0);
        if initial_levels.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(BaseballerInventoryError::InvalidInitialInventory);
        }
        let maximum_level = catalog
            .equipment()
            .iter()
            .filter(|item| item.profiles.contains(&profile.id))
            .map(|item| i64::from(item.maximum_level))
            .max()
            .ok_or(BaseballerInventoryError::EmptyEquipmentPool)?;
        let maximum_entries = u32::try_from(
            catalog
                .equipment()
                .iter()
                .filter(|item| item.profiles.contains(&profile.id))
                .count(),
        )
        .map_err(|_| BaseballerInventoryError::TooManyEquipmentEntries)?;
        Ok(vec![
            ActivitySlotDefinition::new_with_policy(
                self.levels,
                ActivityScope::Activity,
                ActivityValue::BoundedCounterMap(initial_levels.into_boxed_slice()),
                Some((0, maximum_level)),
                Some(maximum_entries),
                vec![],
                SlotCarryPolicy::CarryExact,
                ActivityStateVisibility::Player,
                source(self.levels),
            )
            .map_err(debug_error)?,
            integer_slot(
                self.used_weapon_slots,
                i64::try_from(stage.initial_weapons.len())
                    .map_err(|_| BaseballerInventoryError::InvalidInitialInventory)?,
                i64::from(profile.weapon_slots),
            )?,
            integer_slot(
                self.unlocked_weapon_slots,
                i64::from(profile.initially_unlocked_weapon_slots),
                i64::from(profile.weapon_slots),
            )?,
            integer_slot(
                self.used_accessory_slots,
                0,
                i64::from(profile.accessory_slots),
            )?,
            integer_slot(
                self.unlocked_accessory_slots,
                i64::from(unlocked_accessory_slots),
                i64::from(profile.accessory_slots),
            )?,
        ])
    }

    pub fn equipment_options(
        self,
        catalog: &BaseballerCatalog,
        profile: &BaseballerProfile,
        route: ActivityEdgeId,
    ) -> Result<BaseballerInventoryOptions, BaseballerInventoryError> {
        let mut options = Vec::new();
        let mut legal_conditions = Vec::new();
        let mut weights = Vec::new();
        for item in catalog
            .equipment()
            .iter()
            .filter(|item| item.profiles.contains(&profile.id))
        {
            let (used, unlocked) = match item.kind {
                BaseballerEquipmentKind::StandardWeapon
                | BaseballerEquipmentKind::LegendaryWeapon => {
                    (self.used_weapon_slots, self.unlocked_weapon_slots)
                }
                BaseballerEquipmentKind::Accessory => {
                    (self.used_accessory_slots, self.unlocked_accessory_slots)
                }
            };
            let acquire = ActivityCondition::All(
                vec![
                    level_equal(self.levels, item, 0),
                    ActivityCondition::LessThan(
                        ActivityExpression::Slot(used),
                        ActivityExpression::Slot(unlocked),
                    ),
                ]
                .into_boxed_slice(),
            );
            let upgrade = ActivityCondition::All(
                vec![
                    ActivityCondition::LessThan(integer(0), level(self.levels, item)),
                    ActivityCondition::LessThan(
                        level(self.levels, item),
                        integer(i64::from(item.maximum_level)),
                    ),
                ]
                .into_boxed_slice(),
            );
            let acquire_id = equipment_option(item, ACQUIRE_SUFFIX)?;
            let upgrade_id = equipment_option(item, UPGRADE_SUFFIX)?;
            options.push(ActivityOptionDefinition::new(
                acquire_id,
                priority(item, 0)?,
                acquire.clone(),
                settle_with_synthesis(
                    self,
                    catalog,
                    profile,
                    item.id,
                    vec![
                        ActivityOperation::AddCounter {
                            slot: self.levels,
                            key: u64::from(item.id.get()),
                            delta: integer(1),
                        },
                        ActivityOperation::AddToSlot {
                            slot: used,
                            delta: integer(1),
                        },
                    ],
                    route,
                )?,
            ));
            options.push(ActivityOptionDefinition::new(
                upgrade_id,
                priority(item, 1)?,
                upgrade.clone(),
                settle_with_synthesis(
                    self,
                    catalog,
                    profile,
                    item.id,
                    vec![ActivityOperation::AddCounter {
                        slot: self.levels,
                        key: u64::from(item.id.get()),
                        delta: integer(1),
                    }],
                    route,
                )?,
            ));
            legal_conditions.extend([acquire, upgrade]);
            weights.extend([(acquire_id, 1), (upgrade_id, 1)]);
        }
        if options.is_empty() {
            return Err(BaseballerInventoryError::EmptyEquipmentPool);
        }
        let skip = ActivityOptionId::new(SKIP_OPTION).expect("skip option id is non-zero");
        options.push(ActivityOptionDefinition::new(
            skip,
            i32::MAX,
            ActivityCondition::Not(Box::new(ActivityCondition::Any(
                legal_conditions.into_boxed_slice(),
            ))),
            vec![ActivityOperation::Traverse(route)],
        ));
        weights.push((skip, 1));
        Ok(BaseballerInventoryOptions { options, weights })
    }
}

fn settle_with_synthesis(
    bindings: BaseballerInventoryBindings,
    catalog: &BaseballerCatalog,
    profile: &BaseballerProfile,
    trigger: BaseballerEquipmentId,
    prefix: Vec<ActivityOperation>,
    route: ActivityEdgeId,
) -> Result<Vec<ActivityOperation>, BaseballerInventoryError> {
    let candidates = synthesis_candidates_for(catalog, profile, trigger);
    if candidates.is_empty() {
        let mut operations = prefix;
        operations.push(ActivityOperation::Traverse(route));
        return Ok(operations);
    }
    let post_synthesis = synthesis_candidates(
        bindings,
        catalog,
        profile,
        &candidates,
        route,
        vec![ActivityOperation::Traverse(route)],
    )?;
    let mut fallback = prefix;
    fallback.extend(post_synthesis);
    synthesis_candidates(bindings, catalog, profile, &candidates, route, fallback)
}

fn synthesis_candidates_for<'a>(
    catalog: &'a BaseballerCatalog,
    profile: &BaseballerProfile,
    trigger: BaseballerEquipmentId,
) -> Vec<&'a BaseballerRecipe> {
    let mut candidates = catalog
        .recipes()
        .iter()
        .filter(|recipe| {
            recipe.profile == profile.id
                && recipe.inputs.iter().any(|input| {
                    matches!(input.kind, BaseballerRecipeInputKind::Equipment(id) if id == trigger)
                })
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|recipe| (recipe.tier, recipe.id));
    candidates
}

fn synthesis_tail(
    bindings: BaseballerInventoryBindings,
    catalog: &BaseballerCatalog,
    profile: &BaseballerProfile,
    trigger: BaseballerEquipmentId,
    route: ActivityEdgeId,
) -> Result<Vec<ActivityOperation>, BaseballerInventoryError> {
    let candidates = synthesis_candidates_for(catalog, profile, trigger);
    synthesis_candidates(
        bindings,
        catalog,
        profile,
        &candidates,
        route,
        vec![ActivityOperation::Traverse(route)],
    )
}

fn synthesis_candidates(
    bindings: BaseballerInventoryBindings,
    catalog: &BaseballerCatalog,
    profile: &BaseballerProfile,
    candidates: &[&BaseballerRecipe],
    route: ActivityEdgeId,
    fallback: Vec<ActivityOperation>,
) -> Result<Vec<ActivityOperation>, BaseballerInventoryError> {
    let Some((recipe, remaining)) = candidates.split_first() else {
        return Ok(fallback);
    };
    let mut if_true = synthesis_operations(bindings, catalog, recipe)?;
    if_true.extend(synthesis_tail(
        bindings,
        catalog,
        profile,
        recipe.output,
        route,
    )?);
    let if_false = synthesis_candidates(bindings, catalog, profile, remaining, route, fallback)?;
    Ok(vec![ActivityOperation::Conditional {
        condition: synthesis_condition(bindings, catalog, recipe)?,
        if_true: if_true.into_boxed_slice(),
        if_false: if_false.into_boxed_slice(),
    }])
}

fn synthesis_condition(
    bindings: BaseballerInventoryBindings,
    catalog: &BaseballerCatalog,
    recipe: &BaseballerRecipe,
) -> Result<ActivityCondition, BaseballerInventoryError> {
    let output = catalog
        .equipment_by_id(recipe.output)
        .ok_or(BaseballerInventoryError::InvalidSynthesis)?;
    let mut conditions = vec![level_equal(bindings.levels, output, 0)];
    for input in &recipe.inputs {
        let id = match input.kind {
            BaseballerRecipeInputKind::Equipment(id) => id,
            BaseballerRecipeInputKind::AnyStandardWeapon => {
                return Err(BaseballerInventoryError::UnsupportedSynthesisInput);
            }
        };
        let item = catalog
            .equipment_by_id(id)
            .ok_or(BaseballerInventoryError::InvalidSynthesis)?;
        conditions.push(ActivityCondition::Not(Box::new(
            ActivityCondition::LessThan(
                level(bindings.levels, item),
                integer(i64::from(input.required_level)),
            ),
        )));
    }
    Ok(ActivityCondition::All(conditions.into_boxed_slice()))
}

fn synthesis_operations(
    bindings: BaseballerInventoryBindings,
    catalog: &BaseballerCatalog,
    recipe: &BaseballerRecipe,
) -> Result<Vec<ActivityOperation>, BaseballerInventoryError> {
    let mut operations = Vec::new();
    for input in recipe.inputs.iter().filter(|input| input.consumed) {
        let id = match input.kind {
            BaseballerRecipeInputKind::Equipment(id) => id,
            BaseballerRecipeInputKind::AnyStandardWeapon => {
                return Err(BaseballerInventoryError::UnsupportedSynthesisInput);
            }
        };
        let item = catalog
            .equipment_by_id(id)
            .ok_or(BaseballerInventoryError::InvalidSynthesis)?;
        operations.push(ActivityOperation::AddCounter {
            slot: bindings.levels,
            key: u64::from(id.get()),
            delta: ActivityExpression::Negate(Box::new(level(bindings.levels, item))),
        });
        operations.push(ActivityOperation::AddToSlot {
            slot: used_slot(bindings, item.kind),
            delta: integer(-1),
        });
    }
    operations.push(ActivityOperation::AddCounter {
        slot: bindings.levels,
        key: u64::from(recipe.output.get()),
        delta: integer(1),
    });
    operations.push(ActivityOperation::AddToSlot {
        slot: bindings.used_weapon_slots,
        delta: integer(1),
    });
    Ok(operations)
}

fn used_slot(
    bindings: BaseballerInventoryBindings,
    kind: BaseballerEquipmentKind,
) -> ActivitySlotId {
    match kind {
        BaseballerEquipmentKind::StandardWeapon | BaseballerEquipmentKind::LegendaryWeapon => {
            bindings.used_weapon_slots
        }
        BaseballerEquipmentKind::Accessory => bindings.used_accessory_slots,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseballerInventoryOptions {
    pub options: Vec<ActivityOptionDefinition>,
    pub weights: Vec<(ActivityOptionId, u64)>,
}

fn integer_slot(
    id: ActivitySlotId,
    initial: i64,
    maximum: i64,
) -> Result<ActivitySlotDefinition, BaseballerInventoryError> {
    ActivitySlotDefinition::new_with_policy(
        id,
        ActivityScope::Activity,
        ActivityValue::BoundedInteger(initial),
        Some((0, maximum)),
        None,
        vec![],
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Player,
        source(id),
    )
    .map_err(debug_error)
}

fn source(id: ActivitySlotId) -> ActivityStateSource {
    ActivityStateSource::new(u64::from(id.get())).expect("inventory slot source is non-zero")
}

fn level(slot: ActivitySlotId, item: &BaseballerEquipment) -> ActivityExpression {
    ActivityExpression::CounterValue {
        slot,
        key: u64::from(item.id.get()),
    }
}

fn level_equal(slot: ActivitySlotId, item: &BaseballerEquipment, value: i64) -> ActivityCondition {
    ActivityCondition::Equal(level(slot, item), integer(value))
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn equipment_option(
    item: &BaseballerEquipment,
    suffix: u64,
) -> Result<ActivityOptionId, BaseballerInventoryError> {
    u64::from(item.id.get())
        .checked_mul(OPTION_MULTIPLIER)
        .and_then(|value| value.checked_add(suffix))
        .and_then(ActivityOptionId::new)
        .ok_or(BaseballerInventoryError::IdentityOverflow)
}

fn priority(item: &BaseballerEquipment, suffix: i32) -> Result<i32, BaseballerInventoryError> {
    i32::try_from(item.id.get())
        .ok()
        .and_then(|value| value.checked_mul(2))
        .and_then(|value| value.checked_add(suffix))
        .ok_or(BaseballerInventoryError::IdentityOverflow)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BaseballerInventoryError {
    InvalidInitialInventory,
    InvalidProgressionProfile,
    EmptyEquipmentPool,
    TooManyEquipmentEntries,
    IdentityOverflow,
    InvalidSynthesis,
    UnsupportedSynthesisInput,
    InvalidDefinition(Box<str>),
}

impl std::fmt::Display for BaseballerInventoryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInitialInventory => {
                formatter.write_str("invalid initial Baseballer inventory")
            }
            Self::InvalidProgressionProfile => {
                formatter.write_str("Baseballer progression profile does not match the stage")
            }
            Self::EmptyEquipmentPool => formatter.write_str("Baseballer equipment pool is empty"),
            Self::TooManyEquipmentEntries => {
                formatter.write_str("too many Baseballer equipment entries")
            }
            Self::IdentityOverflow => formatter.write_str("Baseballer inventory identity overflow"),
            Self::InvalidSynthesis => formatter.write_str("invalid Baseballer synthesis"),
            Self::UnsupportedSynthesisInput => {
                formatter.write_str("unsupported Baseballer synthesis input")
            }
            Self::InvalidDefinition(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for BaseballerInventoryError {}

fn debug_error(error: impl std::fmt::Debug) -> BaseballerInventoryError {
    BaseballerInventoryError::InvalidDefinition(format!("{error:?}").into_boxed_str())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use starclock_activity::{
        ActivityConfigDigest, ActivityDecisionKind, ActivityDefinitionDigest, ActivityDefinitionId,
        ActivityDefinitionIdentity, ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId,
        ActivityGraphDefinition, ActivityInstanceId, ActivityMasterSeed, ActivityNodeDefinition,
        ActivityNodeKind, ActivityOperation, ActivityOptionId, ActivityProgramDefinition,
        ActivityProgramId, ActivityRandomPolicies, ActivitySlotId, ActivityStateDefinition,
        ActivityTerminalOutcome, BuildDigest, GraphActivity, GraphActivityDefinition,
        GraphActivityNodeProgram, LoadoutLockScope, NodeId, OpaqueParticipantBuild, ParticipantId,
        ParticipantLock, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
        ParticipantUniquenessScope, SectionId,
    };
    use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};

    use super::BaseballerInventoryBindings;
    use crate::catalog::tests_support::{catalog, full_catalog, profile_id, stage_id};

    #[test]
    fn equipment_choices_distinguish_acquisition_from_upgrade() {
        let catalog = catalog();
        let profile = catalog
            .profiles()
            .iter()
            .find(|profile| profile.id == profile_id())
            .unwrap();
        let bindings = bindings();
        let options = bindings
            .equipment_options(&catalog, profile, ActivityEdgeId::new(1).unwrap())
            .unwrap();

        assert!(options.options.iter().any(|option| {
            option
                .operations()
                .iter()
                .any(|operation| matches!(operation, ActivityOperation::AddToSlot { .. }))
        }));
        assert!(options.options.iter().any(|option| {
            option
                .operations()
                .iter()
                .all(|operation| !matches!(operation, ActivityOperation::AddToSlot { .. }))
        }));
    }

    #[test]
    fn stage_initial_weapons_seed_the_level_map() {
        let catalog = catalog();
        let profile = &catalog.profiles()[0];
        let stage = catalog
            .stages()
            .iter()
            .find(|stage| stage.id == stage_id())
            .unwrap();
        let definitions = bindings().definitions(&catalog, profile, stage).unwrap();

        assert_eq!(definitions.len(), 5);
    }

    #[test]
    fn eligible_synthesis_is_atomic_and_precedes_duplicate_upgrade() {
        let catalog = catalog();
        let profile = &catalog.profiles()[0];
        let options = bindings()
            .equipment_options(&catalog, profile, ActivityEdgeId::new(1).unwrap())
            .unwrap();
        let acquire_second = ActivityOptionId::new(9).unwrap();
        let option = options
            .options
            .iter()
            .find(|option| option.id() == acquire_second)
            .unwrap();

        assert!(matches!(
            option.operations().last(),
            Some(ActivityOperation::Conditional { if_true, .. })
                if if_true.iter().any(|operation| matches!(
                    operation,
                    ActivityOperation::AddCounter { key: 3, .. }
                ))
        ));
    }

    #[test]
    fn no_legal_candidate_exposes_only_skip() {
        let catalog = full_catalog();
        let profile = &catalog.profiles()[0];
        let stage = &catalog.stages()[0];
        let bindings = bindings();
        let state = ActivityStateDefinition::new(
            bindings.definitions(&catalog, profile, stage).unwrap(),
            vec![],
            vec![],
        )
        .unwrap();
        let node = NodeId::new(1).unwrap();
        let terminal = NodeId::new(2).unwrap();
        let edge = ActivityEdgeId::new(1).unwrap();
        let graph = ActivityGraphDefinition::new(
            node,
            vec![
                ActivityNodeDefinition::new(
                    node,
                    SectionId::new(1).unwrap(),
                    ActivityNodeKind::Reward,
                    1,
                )
                .unwrap(),
                ActivityNodeDefinition::new(
                    terminal,
                    SectionId::new(1).unwrap(),
                    ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
                    1,
                )
                .unwrap(),
            ],
            vec![
                ActivityEdgeDefinition::new(
                    edge,
                    node,
                    terminal,
                    ActivityEdgeCondition::OptionSelected,
                    1,
                    1,
                )
                .unwrap(),
            ],
            2,
        )
        .unwrap();
        let options = bindings.equipment_options(&catalog, profile, edge).unwrap();
        let program = ActivityProgramDefinition::new(
            ActivityProgramId::new(1).unwrap(),
            vec![ActivityOperation::Offer {
                kind: ActivityDecisionKind::Reward,
                options: options.options.into_boxed_slice(),
            }],
        )
        .unwrap();
        let definition = Arc::new(
            GraphActivityDefinition::new(
                identity(),
                graph,
                state,
                Arc::new(participants()),
                vec![GraphActivityNodeProgram::new(node, program)],
                None,
                ActivityRandomPolicies::default(),
            )
            .unwrap(),
        );
        let run = GraphActivity::start(
            definition,
            ActivityInstanceId::new(1).unwrap(),
            ActivityMasterSeed::from_u64(7),
        )
        .unwrap()
        .into_activity();
        let view = run.player_view();
        let decision = view.decision().unwrap();

        assert_eq!(decision.options().len(), 1);
        assert_eq!(decision.options()[0].id().get(), super::SKIP_OPTION);
    }

    fn bindings() -> BaseballerInventoryBindings {
        BaseballerInventoryBindings {
            levels: ActivitySlotId::new(1).unwrap(),
            used_weapon_slots: ActivitySlotId::new(2).unwrap(),
            unlocked_weapon_slots: ActivitySlotId::new(3).unwrap(),
            used_accessory_slots: ActivitySlotId::new(4).unwrap(),
            unlocked_accessory_slots: ActivitySlotId::new(5).unwrap(),
        }
    }

    fn identity() -> ActivityDefinitionIdentity {
        ActivityDefinitionIdentity::new(
            ActivityDefinitionId::new(1).unwrap(),
            ActivityDefinitionDigest::new([1; 32]).unwrap(),
            ActivityConfigDigest::new([2; 32]).unwrap(),
        )
    }

    fn participants() -> ParticipantLock {
        let policy = ParticipantPolicy::new(
            1,
            1,
            1,
            ParticipantUniquenessScope::Team,
            LoadoutLockScope::Activity,
        )
        .unwrap();
        let entry = ParticipantLockEntry::new(
            ParticipantId::new(1).unwrap(),
            0,
            0,
            UnitDefinitionId::new(1).unwrap(),
            OpaqueParticipantBuild::new(
                CombatantSpecDigest::new([3; 32]).unwrap(),
                BuildDigest::new([4; 32]).unwrap(),
                ParticipantSourceKind::FixedResolved,
            )
            .unwrap(),
        )
        .unwrap();
        ParticipantLock::seal(policy, vec![entry]).unwrap()
    }
}
