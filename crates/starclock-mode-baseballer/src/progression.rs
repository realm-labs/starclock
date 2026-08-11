use std::collections::BTreeSet;
use std::sync::Arc;

use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityDefinitionIdentity, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityEdgeId, ActivityExpression, ActivityGraphDefinition,
    ActivityInstanceId, ActivityMasterSeed, ActivityNodeDefinition, ActivityNodeKind,
    ActivityOperation, ActivityOptionDefinition, ActivityOptionId, ActivityProgramDefinition,
    ActivityProgramId, ActivityRandomPolicies, ActivityScope, ActivitySlotDefinition,
    ActivitySlotId, ActivityStateDefinition, ActivityStateHash, ActivityStateSource,
    ActivityStateVisibility, ActivityTerminalOutcome, ActivityValue, GraphActivity,
    GraphActivityDefinition, GraphActivityNodeProgram, NodeId, ParticipantLock, SectionId,
    SlotCarryPolicy,
};

use crate::{
    BaseballerCatalog, BaseballerProfileId, BaseballerShopUpgrade, BaseballerShopUpgradeId,
    BaseballerShopUpgradeKind,
};

const SECTION: u32 = 1;
const SHOP_NODE: u32 = 1;
const COMPLETED_NODE: u32 = 2;
const PURCHASE_EDGE: u32 = 1;
const FINISH_EDGE: u32 = 2;
const BALANCE_SLOT: u32 = 1;
const UPGRADE_LEVELS_SLOT: u32 = 2;
const FINISH_OPTION: u64 = 9_100_000_001;
const OPTION_BASE: u64 = 4_000_000_000;

/// Restorable profile-owned progression supplied at the Activity boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseballerProgressionSeed {
    pub balance: i64,
    pub upgrade_levels: Box<[(u32, u8)]>,
}

impl BaseballerProgressionSeed {
    #[must_use]
    pub fn empty(balance: i64) -> Self {
        Self {
            balance,
            upgrade_levels: Box::new([]),
        }
    }
}

/// One unresolved purchased MazeBuff at its current cumulative level.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseballerUnresolvedMazeBuff {
    pub source_numeric_id: u32,
    pub level: u8,
    pub maze_buff_id: u32,
    pub parameters: Box<[Box<str>]>,
}

/// Persistable projection from the authoritative Activity state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseballerProgressionSnapshot {
    pub profile: BaseballerProfileId,
    pub balance: i64,
    pub upgrade_levels: Box<[(u32, u8)]>,
    pub initial_weapon_level_bonus: u8,
    pub additional_accessory_slots: u8,
    pub unresolved_maze_buffs: Box<[BaseballerUnresolvedMazeBuff]>,
}

impl BaseballerProgressionSnapshot {
    #[must_use]
    pub fn seed(&self) -> BaseballerProgressionSeed {
        BaseballerProgressionSeed {
            balance: self.balance,
            upgrade_levels: self.upgrade_levels.clone(),
        }
    }

    pub(crate) fn validate_for_catalog(&self, catalog: &BaseballerCatalog) -> bool {
        project_snapshot(
            catalog,
            self.profile,
            self.balance,
            self.upgrade_levels.clone(),
        )
        .is_ok_and(|expected| expected == *self)
    }
}

#[derive(Debug)]
pub struct BaseballerProgressionDefinition {
    catalog: Arc<BaseballerCatalog>,
    profile: BaseballerProfileId,
    activity: Arc<GraphActivityDefinition>,
}

impl BaseballerProgressionDefinition {
    pub fn new(
        identity: ActivityDefinitionIdentity,
        catalog: Arc<BaseballerCatalog>,
        profile: BaseballerProfileId,
        seed: BaseballerProgressionSeed,
        participants: ParticipantLock,
    ) -> Result<Self, BaseballerProgressionError> {
        if !catalog.profiles().iter().any(|item| item.id == profile) {
            return Err(error(
                "Baseballer progression profile is not in the catalog",
            ));
        }
        let upgrades = catalog
            .shop_upgrades_for_profile(profile)
            .collect::<Vec<_>>();
        if upgrades.is_empty() {
            return Err(error("Baseballer progression profile has no shop upgrades"));
        }
        validate_seed(&seed, &upgrades)?;
        let graph = progression_graph(upgrades.len())?;
        let state = progression_state(seed, &upgrades)?;
        let program = shop_program(&upgrades)?;
        let activity = GraphActivityDefinition::new(
            identity,
            graph,
            state,
            Arc::new(participants),
            vec![GraphActivityNodeProgram::new(node(SHOP_NODE), program)],
            None,
            ActivityRandomPolicies::new(vec![], vec![]),
        )
        .map_err(debug_error)?;
        Ok(Self {
            catalog,
            profile,
            activity: Arc::new(activity),
        })
    }

    #[must_use]
    pub const fn profile(&self) -> BaseballerProfileId {
        self.profile
    }
}

#[derive(Debug)]
pub struct BaseballerProgression {
    definition: Arc<BaseballerProgressionDefinition>,
    activity: GraphActivity,
}

impl BaseballerProgression {
    pub fn start(
        definition: Arc<BaseballerProgressionDefinition>,
        instance: ActivityInstanceId,
        master_seed: ActivityMasterSeed,
    ) -> Result<Self, BaseballerProgressionError> {
        let activity =
            GraphActivity::start(Arc::clone(&definition.activity), instance, master_seed)
                .map_err(debug_error)?
                .into_activity();
        Ok(Self {
            definition,
            activity,
        })
    }

    #[must_use]
    pub fn state_hash(&self) -> ActivityStateHash {
        self.activity.state_hash()
    }

    #[must_use]
    pub fn canonical_state_bytes(&self) -> Box<[u8]> {
        self.activity.canonical_state_bytes()
    }

    #[must_use]
    pub fn player_view(&self) -> starclock_activity::ActivityPlayerView {
        self.activity.player_view()
    }

    #[must_use]
    pub fn debug_view(&self) -> starclock_activity::ActivityDebugView {
        self.activity.debug_view()
    }

    pub fn purchase(
        &mut self,
        upgrade: BaseballerShopUpgradeId,
    ) -> Result<(), BaseballerProgressionError> {
        let item = self
            .definition
            .catalog
            .shop_upgrade_by_id(upgrade)
            .filter(|item| item.profile == self.definition.profile)
            .ok_or_else(|| error("shop upgrade does not belong to this profile"))?;
        let view = self.activity.player_view();
        let decision = view
            .decision()
            .filter(|decision| decision.kind() == ActivityDecisionKind::Shop)
            .ok_or_else(|| error("Baseballer shop is not currently offered"))?;
        self.activity
            .choose_option(view.state_hash(), decision.id(), purchase_option(item)?)
            .map_err(debug_error)?;
        Ok(())
    }

    pub fn finish(&mut self) -> Result<(), BaseballerProgressionError> {
        let view = self.activity.player_view();
        let decision = view
            .decision()
            .filter(|decision| decision.kind() == ActivityDecisionKind::Shop)
            .ok_or_else(|| error("Baseballer shop is not currently offered"))?;
        self.activity
            .choose_option(view.state_hash(), decision.id(), option(FINISH_OPTION))
            .map_err(debug_error)?;
        Ok(())
    }

    pub fn snapshot(&self) -> Result<BaseballerProgressionSnapshot, BaseballerProgressionError> {
        let balance = integer_slot(&self.activity, slot(BALANCE_SLOT))?;
        let upgrade_levels = counter_slot(&self.activity, slot(UPGRADE_LEVELS_SLOT))?;
        project_snapshot(
            &self.definition.catalog,
            self.definition.profile,
            balance,
            upgrade_levels,
        )
    }
}

fn validate_seed(
    seed: &BaseballerProgressionSeed,
    upgrades: &[&BaseballerShopUpgrade],
) -> Result<(), BaseballerProgressionError> {
    if seed.balance < 0
        || seed
            .upgrade_levels
            .windows(2)
            .any(|pair| pair[0].0 >= pair[1].0)
    {
        return Err(error("Baseballer progression seed is not canonical"));
    }
    for (id, level) in &seed.upgrade_levels {
        let maximum = upgrades
            .iter()
            .find(|upgrade| upgrade.source_numeric_id == *id)
            .map(|upgrade| upgrade.maximum_level)
            .ok_or_else(|| error("Baseballer progression seed has an unknown upgrade"))?;
        if *level == 0 || *level > maximum {
            return Err(error("Baseballer progression seed level is out of bounds"));
        }
    }
    Ok(())
}

fn progression_graph(
    upgrade_count: usize,
) -> Result<ActivityGraphDefinition, BaseballerProgressionError> {
    let visits = u32::try_from(upgrade_count)
        .ok()
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| error("too many Baseballer shop upgrades"))?;
    let section = SectionId::new(SECTION).expect("section id is non-zero");
    let nodes = vec![
        ActivityNodeDefinition::new(node(SHOP_NODE), section, ActivityNodeKind::Shop, visits)
            .map_err(debug_error)?,
        ActivityNodeDefinition::new(
            node(COMPLETED_NODE),
            section,
            ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
            1,
        )
        .map_err(debug_error)?,
    ];
    let edges = vec![
        ActivityEdgeDefinition::new(
            edge(PURCHASE_EDGE),
            node(SHOP_NODE),
            node(SHOP_NODE),
            ActivityEdgeCondition::OptionSelected,
            0,
            u32::try_from(upgrade_count).map_err(debug_error)?,
        )
        .map_err(debug_error)?,
        ActivityEdgeDefinition::new(
            edge(FINISH_EDGE),
            node(SHOP_NODE),
            node(COMPLETED_NODE),
            ActivityEdgeCondition::OptionSelected,
            1,
            1,
        )
        .map_err(debug_error)?,
    ];
    ActivityGraphDefinition::new(node(SHOP_NODE), nodes, edges, visits + 1).map_err(debug_error)
}

fn progression_state(
    seed: BaseballerProgressionSeed,
    upgrades: &[&BaseballerShopUpgrade],
) -> Result<ActivityStateDefinition, BaseballerProgressionError> {
    let maximum_level = upgrades
        .iter()
        .map(|upgrade| i64::from(upgrade.maximum_level))
        .max()
        .ok_or_else(|| error("Baseballer shop has no upgrades"))?;
    let maximum_entries = u32::try_from(
        upgrades
            .iter()
            .map(|upgrade| upgrade.source_numeric_id)
            .collect::<BTreeSet<_>>()
            .len(),
    )
    .map_err(debug_error)?;
    let levels = seed
        .upgrade_levels
        .iter()
        .map(|(id, level)| (u64::from(*id), i64::from(*level)))
        .collect::<Vec<_>>();
    let slots = vec![
        ActivitySlotDefinition::new_with_policy(
            slot(BALANCE_SLOT),
            ActivityScope::Activity,
            ActivityValue::BoundedInteger(seed.balance),
            Some((0, seed.balance)),
            None,
            vec![],
            SlotCarryPolicy::CarryExact,
            ActivityStateVisibility::Player,
            source(BALANCE_SLOT),
        )
        .map_err(debug_error)?,
        ActivitySlotDefinition::new_with_policy(
            slot(UPGRADE_LEVELS_SLOT),
            ActivityScope::Activity,
            ActivityValue::BoundedCounterMap(levels.into_boxed_slice()),
            Some((0, maximum_level)),
            Some(maximum_entries),
            vec![],
            SlotCarryPolicy::CarryExact,
            ActivityStateVisibility::Player,
            source(UPGRADE_LEVELS_SLOT),
        )
        .map_err(debug_error)?,
    ];
    ActivityStateDefinition::new(slots, vec![], vec![]).map_err(debug_error)
}

fn shop_program(
    upgrades: &[&BaseballerShopUpgrade],
) -> Result<ActivityProgramDefinition, BaseballerProgressionError> {
    let mut options = upgrades
        .iter()
        .map(|upgrade| {
            let enabled = purchase_condition(upgrade);
            Ok(ActivityOptionDefinition::new(
                purchase_option(upgrade)?,
                i32::try_from(upgrade.id.get()).map_err(debug_error)?,
                enabled.clone(),
                vec![
                    ActivityOperation::Require(enabled),
                    ActivityOperation::AddToSlot {
                        slot: slot(BALANCE_SLOT),
                        delta: literal(-upgrade.cost),
                    },
                    ActivityOperation::AddCounter {
                        slot: slot(UPGRADE_LEVELS_SLOT),
                        key: u64::from(upgrade.source_numeric_id),
                        delta: literal(1),
                    },
                    ActivityOperation::Traverse(edge(PURCHASE_EDGE)),
                ],
            ))
        })
        .collect::<Result<Vec<_>, BaseballerProgressionError>>()?;
    options.push(ActivityOptionDefinition::new(
        option(FINISH_OPTION),
        i32::MAX,
        always(),
        vec![ActivityOperation::Traverse(edge(FINISH_EDGE))],
    ));
    ActivityProgramDefinition::new(
        ActivityProgramId::new(1).expect("program id is non-zero"),
        vec![ActivityOperation::Offer {
            kind: ActivityDecisionKind::Shop,
            options: options.into_boxed_slice(),
        }],
    )
    .map_err(debug_error)
}

fn purchase_condition(upgrade: &BaseballerShopUpgrade) -> ActivityCondition {
    ActivityCondition::All(
        vec![
            ActivityCondition::Equal(
                ActivityExpression::CounterValue {
                    slot: slot(UPGRADE_LEVELS_SLOT),
                    key: u64::from(upgrade.source_numeric_id),
                },
                literal(i64::from(upgrade.purchase_level - 1)),
            ),
            ActivityCondition::Not(Box::new(ActivityCondition::LessThan(
                ActivityExpression::Slot(slot(BALANCE_SLOT)),
                literal(upgrade.cost),
            ))),
        ]
        .into_boxed_slice(),
    )
}

fn project_snapshot(
    catalog: &BaseballerCatalog,
    profile: BaseballerProfileId,
    balance: i64,
    levels: Box<[(u32, u8)]>,
) -> Result<BaseballerProgressionSnapshot, BaseballerProgressionError> {
    let mut initial_weapon_level_bonus = 0_u8;
    let mut additional_accessory_slots = 0_u8;
    let mut unresolved_maze_buffs = Vec::new();
    for (source_numeric_id, level) in &levels {
        let row = catalog
            .shop_upgrades_for_profile(profile)
            .find(|upgrade| {
                upgrade.source_numeric_id == *source_numeric_id && upgrade.purchase_level == *level
            })
            .ok_or_else(|| error("progression level has no exact shop row"))?;
        match row.kind {
            BaseballerShopUpgradeKind::InitWeaponLevel => {
                initial_weapon_level_bonus = initial_weapon_level_bonus
                    .checked_add(*level)
                    .ok_or_else(|| error("initial weapon level bonus overflow"))?;
            }
            BaseballerShopUpgradeKind::AddAccessorySlot => {
                additional_accessory_slots = additional_accessory_slots
                    .checked_add(*level)
                    .ok_or_else(|| error("additional accessory slots overflow"))?;
            }
            BaseballerShopUpgradeKind::AddMazeBuff => {
                let maze_buff_id = row
                    .maze_buff_id
                    .ok_or_else(|| error("MazeBuff shop row has no MazeBuff identity"))?;
                if row.runtime_binding_exact {
                    return Err(error(
                        "MazeBuff row claims an exact binding but has no shared Combat lowering",
                    ));
                }
                unresolved_maze_buffs.push(BaseballerUnresolvedMazeBuff {
                    source_numeric_id: *source_numeric_id,
                    level: *level,
                    maze_buff_id,
                    parameters: row.maze_buff_parameters.clone(),
                });
            }
        }
    }
    Ok(BaseballerProgressionSnapshot {
        profile,
        balance,
        upgrade_levels: levels,
        initial_weapon_level_bonus,
        additional_accessory_slots,
        unresolved_maze_buffs: unresolved_maze_buffs.into_boxed_slice(),
    })
}

fn integer_slot(
    activity: &GraphActivity,
    id: ActivitySlotId,
) -> Result<i64, BaseballerProgressionError> {
    activity
        .player_view()
        .slots()
        .iter()
        .find(|entry| entry.id() == id)
        .and_then(|entry| match entry.value() {
            ActivityValue::BoundedInteger(value) => Some(*value),
            _ => None,
        })
        .ok_or_else(|| error("Baseballer progression integer slot is missing"))
}

fn counter_slot(
    activity: &GraphActivity,
    id: ActivitySlotId,
) -> Result<Box<[(u32, u8)]>, BaseballerProgressionError> {
    let view = activity.player_view();
    let values = view
        .slots()
        .iter()
        .find(|entry| entry.id() == id)
        .and_then(|entry| match entry.value() {
            ActivityValue::BoundedCounterMap(values) => Some(values.as_ref()),
            _ => None,
        })
        .ok_or_else(|| error("Baseballer progression counter slot is missing"))?;
    values
        .iter()
        .map(|(key, value)| {
            Ok((
                u32::try_from(*key).map_err(debug_error)?,
                u8::try_from(*value).map_err(debug_error)?,
            ))
        })
        .collect::<Result<Vec<_>, BaseballerProgressionError>>()
        .map(Vec::into_boxed_slice)
}

fn purchase_option(
    upgrade: &BaseballerShopUpgrade,
) -> Result<ActivityOptionId, BaseballerProgressionError> {
    OPTION_BASE
        .checked_add(u64::from(upgrade.id.get()))
        .and_then(ActivityOptionId::new)
        .ok_or_else(|| error("Baseballer shop option identity overflow"))
}

fn literal(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn always() -> ActivityCondition {
    ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(true)))
}

fn node(raw: u32) -> NodeId {
    NodeId::new(raw).expect("node id is non-zero")
}

fn edge(raw: u32) -> ActivityEdgeId {
    ActivityEdgeId::new(raw).expect("edge id is non-zero")
}

fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("slot id is non-zero")
}

fn option(raw: u64) -> ActivityOptionId {
    ActivityOptionId::new(raw).expect("option id is non-zero")
}

fn source(raw: u32) -> ActivityStateSource {
    ActivityStateSource::new(u64::from(raw)).expect("state source is non-zero")
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseballerProgressionError {
    message: Box<str>,
}

impl std::fmt::Display for BaseballerProgressionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for BaseballerProgressionError {}

fn error(message: &str) -> BaseballerProgressionError {
    BaseballerProgressionError {
        message: message.into(),
    }
}

fn debug_error(error: impl std::fmt::Debug) -> BaseballerProgressionError {
    BaseballerProgressionError {
        message: format!("{error:?}").into_boxed_str(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use starclock_activity::{
        ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
        ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, BuildDigest,
        LoadoutLockScope, OpaqueParticipantBuild, ParticipantId, ParticipantLock,
        ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope,
    };
    use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};

    use super::{
        BaseballerProgression, BaseballerProgressionDefinition, BaseballerProgressionSeed,
    };
    use crate::{
        BaseballerCatalog, BaseballerRun, BaseballerRunDefinition, BaseballerScoreRule,
        BaseballerShopUpgrade, BaseballerShopUpgradeId, BaseballerShopUpgradeKind,
        catalog::tests_support,
    };

    #[test]
    fn purchase_commits_balance_level_and_effect_atomically() {
        let mut progression = progression(500);

        progression.purchase(upgrade_id(1)).unwrap();
        let snapshot = progression.snapshot().unwrap();

        assert_eq!(snapshot.balance, 400);
        assert_eq!(snapshot.upgrade_levels.as_ref(), &[(90, 1)]);
        assert_eq!(snapshot.initial_weapon_level_bonus, 1);
    }

    #[test]
    fn rejected_purchase_leaves_state_byte_identical() {
        let mut progression = progression(50);
        let before = progression.canonical_state_bytes();

        assert!(progression.purchase(upgrade_id(1)).is_err());
        assert_eq!(progression.canonical_state_bytes(), before);
    }

    #[test]
    fn purchased_initial_level_seeds_the_stage_inventory() {
        let catalog = Arc::new(catalog());
        let progression_definition = Arc::new(
            BaseballerProgressionDefinition::new(
                identity(),
                Arc::clone(&catalog),
                tests_support::profile_id(),
                BaseballerProgressionSeed::empty(500),
                participants(),
            )
            .unwrap(),
        );
        let mut progression = BaseballerProgression::start(
            progression_definition,
            ActivityInstanceId::new(2).unwrap(),
            ActivityMasterSeed::from_u64(2),
        )
        .unwrap();
        progression.purchase(upgrade_id(1)).unwrap();
        let snapshot = progression.snapshot().unwrap();
        let equipment = catalog.equipment()[0].id;
        let run_definition = Arc::new(
            BaseballerRunDefinition::new_with_progression(
                identity(),
                catalog,
                tests_support::stage_id(),
                BaseballerScoreRule::new(
                    1,
                    vec![1, 1, 1, 1],
                    vec![1, 1, 1, 1, 1],
                    100,
                    0,
                    vec![0, 1, 2, 3, 4],
                )
                .unwrap(),
                participants(),
                Some(&snapshot),
            )
            .unwrap(),
        );
        let run = BaseballerRun::start(
            run_definition,
            ActivityInstanceId::new(3).unwrap(),
            ActivityMasterSeed::from_u64(3),
        )
        .unwrap();

        assert_eq!(run.equipment_level(equipment), 2);
    }

    fn progression(balance: i64) -> BaseballerProgression {
        let definition = Arc::new(
            BaseballerProgressionDefinition::new(
                identity(),
                Arc::new(catalog()),
                tests_support::profile_id(),
                BaseballerProgressionSeed::empty(balance),
                participants(),
            )
            .unwrap(),
        );
        BaseballerProgression::start(
            definition,
            ActivityInstanceId::new(1).unwrap(),
            ActivityMasterSeed::from_u64(1),
        )
        .unwrap()
    }

    fn catalog() -> BaseballerCatalog {
        let base = tests_support::catalog();
        BaseballerCatalog::new_with_shop_upgrades(
            base.profiles().to_vec(),
            base.stages().to_vec(),
            base.stage_periods().to_vec(),
            base.equipment().to_vec(),
            base.recipes().to_vec(),
            vec![BaseballerShopUpgrade {
                id: upgrade_id(1),
                stable_key: "initial-level-1".into(),
                profile: tests_support::profile_id(),
                source_numeric_id: 90,
                purchase_level: 1,
                maximum_level: 1,
                kind: BaseballerShopUpgradeKind::InitWeaponLevel,
                currency_key: "gold".into(),
                cost: 100,
                maze_buff_id: None,
                maze_buff_parameters: Box::new([]),
                shop_parameter_values: Box::new(["1".into()]),
                runtime_binding_exact: true,
            }],
        )
        .unwrap()
    }

    fn upgrade_id(raw: u32) -> BaseballerShopUpgradeId {
        BaseballerShopUpgradeId::new(raw).unwrap()
    }

    fn identity() -> ActivityDefinitionIdentity {
        ActivityDefinitionIdentity::new(
            ActivityDefinitionId::new(2).unwrap(),
            ActivityDefinitionDigest::new([5; 32]).unwrap(),
            ActivityConfigDigest::new([6; 32]).unwrap(),
        )
    }

    fn participants() -> ParticipantLock {
        let policy = ParticipantPolicy::new(
            1,
            1,
            4,
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
                CombatantSpecDigest::new([7; 32]).unwrap(),
                BuildDigest::new([8; 32]).unwrap(),
                ParticipantSourceKind::FixedResolved,
            )
            .unwrap(),
        )
        .unwrap();
        ParticipantLock::seal(policy, vec![entry]).unwrap()
    }
}
