//! Line-limit exception: the Activity aggregate keeps command transitions and state invariants in one mutation authority.
use std::{collections::BTreeMap, sync::Arc};

use sha2::{Digest, Sha256};
use starclock_activity::{
    ActivityBattleHandoff, ActivityBattlePreparationRequest, ActivityDebugView,
    ActivityDecisionKind, ActivityDefinitionIdentity, ActivityExpression, ActivityInstanceId,
    ActivityMasterSeed, ActivityOperation, ActivityPlayerView, ActivityProgramDefinition,
    ActivityRandomPolicies, ActivityRngStreams, ActivityRosterLock, ActivityScopePath,
    ActivityStateHash, ActivityValue, AttemptId, BattleBinding, BattleOutcome, BattleResult,
    BattleSequence, BuildDigest, EncounterInitiativePolicy, EncounterPreparationDefinition,
    GraphActivity, GraphActivityBattleResolution, GraphActivityCommandError,
    GraphActivityDefinition, GraphActivityPreparationResolution, LoadoutLockScope, MetricValue,
    OpaqueParticipantBuild, ParticipantId, ParticipantLock, ParticipantLockEntry,
    ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope, PreparedBattleVariant,
    ProjectedValue,
};
use starclock_build::{
    ability::AbilityInvestment,
    output::CompiledBuild,
    spec::CombatantBuildSpec,
    substitution::{BuildSubstitutionReceipt, SubstitutedBuild},
};
use starclock_combat::{ActionValue, BattleSpec, Ratio, TeamSide};

use crate::{
    CurrencyWarsBattleAssembler, CurrencyWarsBattleMaterialization,
    CurrencyWarsBattleOverrideSnapshot, CurrencyWarsBondId, CurrencyWarsBondResolutionContext,
    CurrencyWarsBondSnapshot, CurrencyWarsCatalog, CurrencyWarsContributionSnapshot,
    CurrencyWarsDeployment, CurrencyWarsEmpowermentSnapshot, CurrencyWarsEnemyAffixSelection,
    CurrencyWarsEntryResolution, CurrencyWarsEntryState, CurrencyWarsEquipmentId,
    CurrencyWarsEquipmentLoadout, CurrencyWarsEquipmentSlot, CurrencyWarsFlow, CurrencyWarsGambit,
    CurrencyWarsInvestmentId, CurrencyWarsInvestmentKind, CurrencyWarsNode, CurrencyWarsPosition,
    CurrencyWarsPositionKind, CurrencyWarsRoleId, CurrencyWarsRoleState, CurrencyWarsRoster,
    CurrencyWarsRouteId, advance_team_level,
    contribution::{materialize as materialize_contribution, selected_role_builds},
};

pub struct CurrencyWarsBattlePreparation {
    resolution: GraphActivityPreparationResolution,
    materialization: CurrencyWarsBattleMaterialization,
}

impl CurrencyWarsBattlePreparation {
    #[must_use]
    pub const fn resolution(&self) -> &GraphActivityPreparationResolution {
        &self.resolution
    }

    #[must_use]
    pub const fn materialization(&self) -> &CurrencyWarsBattleMaterialization {
        &self.materialization
    }

    #[must_use]
    pub fn into_materialization(self) -> CurrencyWarsBattleMaterialization {
        self.materialization
    }
}

mod boundary;
mod definition;
mod economy;
mod investment;
mod progression;
mod reward;
mod service;
mod state;

pub use boundary::{
    CurrencyWarsActionValueBudget, CurrencyWarsBattleBoundary, CurrencyWarsBattleBoundaryResolution,
};
pub use economy::CurrencyWarsShopOffer;
pub use service::{
    CurrencyWarsAppliedReward, CurrencyWarsForgeOffer, CurrencyWarsRewardPoolResolution,
    CurrencyWarsSpecialGoodActivation,
};

use self::definition::{activity_state, battle_contract, node_programs, validate_participants};

pub const CURRENCY_WARS_BATTLE_PROGRESS_KEY: &str = "currency_wars_battle_progress";
pub const CURRENCY_WARS_ACTION_VALUE_REMAINING_KEY: &str = "currency_wars_action_value_remaining";

const GOLD: u32 = 1;
const EXPERIENCE: u32 = 2;
const TEAM_LEVEL: u32 = 3;
const SQUAD_HP: u32 = 4;
const LAST_LOSS: u32 = 5;
const LAST_ACTION_VALUE: u32 = 6;
const ROSTER: u32 = 7;
const DEPLOYMENT: u32 = 8;
const BONDS: u32 = 9;
const SHOP_OFFERS: u32 = 10;
pub(super) const INVESTMENTS: u32 = 11;
const LAST_PROGRESS: u32 = 12;
const SHOP_LOCKED: u32 = 13;
const LOCKED_SHOP_OFFERS: u32 = 14;
const BACK_CAPACITY: u32 = 15;
const EQUIPMENT_INVENTORY: u32 = 16;
const EQUIPMENT_LOADOUT: u32 = 17;
const BOND_SELECTIONS: u32 = 18;
const AUGMENT_OFFERS: u32 = 19;
pub(super) const SELECTED_ENHANCEMENTS: u32 = 20;
const SELECTED_ENHANCEMENT_OFFERS: u32 = 21;
pub(super) const SEASON_TALENTS: u32 = 22;
const INVESTMENT_OFFERS: u32 = 23;
const INVESTMENT_REROLLS: u32 = 24;
const INVESTMENT_FAMILY_MASK: u32 = 25;
const INVESTMENT_QUALITY: u32 = 26;
const INVESTMENT_OFFER_WIDTH: u32 = 27;
const ITEM_INVENTORY: u32 = 28;
const FREE_REFRESHES: u32 = 29;
const FORGE_OFFERS: u32 = 30;
const FORGE_ITEM: u32 = 31;
pub(super) const SPECIAL_GOOD_OFFER: u32 = 32;
pub(super) const SPECIAL_GOOD_PURCHASED: u32 = 33;
pub(super) const SPECIAL_GOOD_ACTIVATIONS: u32 = 34;
pub(super) const CURRENT_CHAPTER: u32 = 35;
pub(super) const CURRENT_SECTION: u32 = 36;
pub(super) const TREASURE_TO_TRASH_PLANE: u32 = 37;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRunSetup {
    pub initial_gold: u32,
    pub initial_team_level: u8,
    pub initial_experience: u32,
    pub roster: CurrencyWarsRoster,
    pub deployment: CurrencyWarsDeployment,
    /// Pre-run player choice. Empty input uses the documented deterministic fallback policy.
    pub enemy_affix_ids: Box<[u32]>,
    /// Caller-frozen account builds. Missing roles use their mapped trial minimum.
    pub owned_builds: BTreeMap<CurrencyWarsRoleId, CurrencyWarsOwnedBuildSnapshot>,
}

/// Immutable account-side Build input captured before the Activity starts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsOwnedBuildSnapshot {
    spec: CombatantBuildSpec,
    combatant: starclock_combat::ResolvedCombatantSpec,
    receipt: BuildSubstitutionReceipt,
    effective_ability_levels: Box<[AbilityInvestment]>,
}

impl CurrencyWarsOwnedBuildSnapshot {
    /// Captures the already-substituted Build and the compiler's final effective levels.
    pub fn new(
        build: SubstitutedBuild,
        compiled: CompiledBuild,
    ) -> Result<Self, CurrencyWarsRuntimeError> {
        if build.spec() != compiled.selected_spec() {
            return Err(error(
                "Currency Wars owned Build does not match its compiled result",
            ));
        }
        let effective_ability_levels = compiled.effective_ability_levels();
        if effective_ability_levels.is_empty()
            || effective_ability_levels
                .windows(2)
                .any(|pair| pair[0].family() >= pair[1].family())
        {
            return Err(error(
                "Currency Wars owned Build effective levels are empty or non-canonical",
            ));
        }
        let receipt = build.receipt();
        Ok(Self {
            spec: build.into_spec(),
            combatant: compiled.combatant().clone(),
            receipt,
            effective_ability_levels: effective_ability_levels.into(),
        })
    }

    #[must_use]
    pub const fn spec(&self) -> &CombatantBuildSpec {
        &self.spec
    }

    #[must_use]
    pub const fn receipt(&self) -> BuildSubstitutionReceipt {
        self.receipt
    }

    #[must_use]
    pub const fn combatant(&self) -> &starclock_combat::ResolvedCombatantSpec {
        &self.combatant
    }

    #[must_use]
    pub fn effective_ability_levels(&self) -> &[AbilityInvestment] {
        &self.effective_ability_levels
    }
}

impl Default for CurrencyWarsRunSetup {
    fn default() -> Self {
        Self {
            initial_gold: 0,
            initial_team_level: 1,
            initial_experience: 0,
            roster: CurrencyWarsRoster::default(),
            deployment: CurrencyWarsDeployment::default(),
            enemy_affix_ids: Box::new([]),
            owned_builds: BTreeMap::new(),
        }
    }
}

impl CurrencyWarsRunSetup {
    pub fn participant_lock(
        &self,
        catalog: &CurrencyWarsCatalog,
    ) -> Result<ParticipantLock, CurrencyWarsRuntimeError> {
        let entries = self
            .deployment
            .positions()
            .iter()
            .filter(|(position, _)| position.kind() == CurrencyWarsPositionKind::Front)
            .map(|(position, state)| {
                let (combatant, source) = self.owned_builds.get(&state.role()).map_or_else(
                    || {
                        catalog
                            .build_catalog()
                            .trial_build(state.role())
                            .map(|trial| (&trial.combatant, ParticipantSourceKind::Trial))
                            .ok_or_else(|| {
                                error("Currency Wars participant trial Build is missing")
                            })
                    },
                    |owned| Ok((owned.combatant(), ParticipantSourceKind::CompiledBuild)),
                )?;
                let mut hash = Sha256::new();
                hash.update(b"starclock.currency-wars.participant-build.v1");
                hash.update(combatant.digest().bytes());
                let build_digest = BuildDigest::new(hash.finalize().into())
                    .ok_or_else(|| error("Currency Wars participant Build digest is zero"))?;
                ParticipantLockEntry::new(
                    ParticipantId::new(state.role().get())
                        .ok_or_else(|| error("Currency Wars participant ID is zero"))?,
                    0,
                    position.index().saturating_sub(1),
                    combatant.form(),
                    OpaqueParticipantBuild::new(combatant.digest(), build_digest, source)
                        .map_err(debug_error)?,
                )
                .map_err(debug_error)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let count = u8::try_from(entries.len()).map_err(debug_error)?;
        let policy = ParticipantPolicy::new(
            1,
            count,
            count,
            ParticipantUniquenessScope::Team,
            LoadoutLockScope::Activity,
        )
        .ok_or_else(|| error("Currency Wars participant policy is invalid"))?;
        ParticipantLock::seal(policy, entries).map_err(debug_error)
    }
}

#[derive(Debug)]
pub struct CurrencyWarsRunDefinition {
    pub(super) catalog: Arc<CurrencyWarsCatalog>,
    pub(super) route: CurrencyWarsRouteId,
    pub(super) difficulty: u32,
    pub(super) gambit: CurrencyWarsGambit,
    entry: CurrencyWarsEntryResolution,
    flow: CurrencyWarsFlow,
    pub(super) activity: Arc<GraphActivityDefinition>,
    boundaries: Box<[Option<CurrencyWarsBattleBoundary>]>,
    pub(super) enemy_affixes: CurrencyWarsEnemyAffixSelection,
    pub(super) owned_builds: BTreeMap<CurrencyWarsRoleId, CurrencyWarsOwnedBuildSnapshot>,
}

impl CurrencyWarsRunDefinition {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        identity: ActivityDefinitionIdentity,
        catalog: Arc<CurrencyWarsCatalog>,
        route: CurrencyWarsRouteId,
        difficulty: u32,
        gambit: CurrencyWarsGambit,
        entry_state: CurrencyWarsEntryState,
        setup: CurrencyWarsRunSetup,
    ) -> Result<Self, CurrencyWarsRuntimeError> {
        let route_definition = catalog
            .route(route)
            .ok_or_else(|| error("Currency Wars route is not in the catalog"))?;
        let entry = catalog
            .flow_catalog()
            .resolve_entry(route, difficulty, gambit, entry_state)
            .map_err(debug_error)?;
        let difficulty_definition = catalog
            .difficulties()
            .iter()
            .find(|candidate| candidate.source_id == difficulty)
            .ok_or_else(|| error("Currency Wars difficulty is not in the catalog"))?;
        let enemy_affixes = CurrencyWarsEnemyAffixSelection::resolve(
            catalog.encounter_catalog(),
            difficulty_definition,
            &setup.enemy_affix_ids,
            enemy_affix_seed(&identity, route, difficulty, gambit),
        )
        .map_err(debug_error)?;
        let participants = setup.participant_lock(&catalog)?;
        validate_participants(&participants)?;
        if setup
            .roster
            .states()
            .keys()
            .any(|state| !catalog.role_available(state.role()))
        {
            return Err(error(
                "Currency Wars initial roster contains a season- or module-excluded role",
            ));
        }
        setup
            .deployment
            .validate(&catalog, &setup.roster, setup.initial_team_level)
            .map_err(debug_error)?;
        for (&role, owned) in &setup.owned_builds {
            let trial = catalog
                .build_catalog()
                .trial_build(role)
                .ok_or_else(|| error("Currency Wars owned Build role is missing"))?;
            if owned.spec.form() != trial.spec.form() {
                return Err(error(
                    "Currency Wars owned Build form does not match its role",
                ));
            }
        }
        let (level, experience) =
            advance_team_level(&catalog, setup.initial_team_level, setup.initial_experience)
                .map_err(debug_error)?;
        let flow = CurrencyWarsFlow::compile(route_definition).map_err(debug_error)?;
        let state = activity_state(&catalog, route_definition, &setup, level, experience)?;
        let programs = node_programs(route_definition, &flow, &enemy_affixes)?;
        let boundaries = route_definition
            .nodes
            .iter()
            .map(|node| {
                if !node.kind.battle() {
                    return Ok(None);
                }
                let penalty_rule_id = node
                    .penalty_bonus_rule_id
                    .ok_or_else(|| error("Currency Wars battle node has no penalty rule"))?;
                let rule = catalog
                    .flow_catalog()
                    .penalty_rule(penalty_rule_id)
                    .ok_or_else(|| error("Currency Wars battle penalty rule is missing"))?;
                let adjustment = enemy_affixes
                    .action_value_adjustment(node.kind)
                    .map_err(debug_error)?;
                CurrencyWarsBattleBoundary::from_penalty_rule(rule)
                    .and_then(|boundary| boundary.with_action_value_adjustment(adjustment))
                    .map(Some)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let participants = Arc::new(participants);
        let activity = Arc::new(
            GraphActivityDefinition::new(
                identity,
                flow.graph().clone(),
                state,
                Arc::clone(&participants),
                programs,
                None,
                ActivityRandomPolicies::new(vec![], vec![]),
            )
            .map_err(debug_error)?,
        );
        Ok(Self {
            catalog,
            route,
            difficulty,
            gambit,
            entry,
            flow,
            activity,
            boundaries: boundaries.into_boxed_slice(),
            enemy_affixes,
            owned_builds: setup.owned_builds,
        })
    }

    #[must_use]
    pub const fn route(&self) -> CurrencyWarsRouteId {
        self.route
    }

    #[must_use]
    pub const fn gambit(&self) -> CurrencyWarsGambit {
        self.gambit
    }

    #[must_use]
    pub const fn difficulty(&self) -> u32 {
        self.difficulty
    }

    #[must_use]
    pub const fn entry(&self) -> CurrencyWarsEntryResolution {
        self.entry
    }

    #[must_use]
    pub const fn enemy_affixes(&self) -> &CurrencyWarsEnemyAffixSelection {
        &self.enemy_affixes
    }

    #[must_use]
    pub fn battle_boundary(&self, route_index: usize) -> Option<&CurrencyWarsBattleBoundary> {
        self.boundaries.get(route_index).and_then(Option::as_ref)
    }
}

#[derive(Debug)]
pub struct CurrencyWarsRun {
    pub(super) definition: Arc<CurrencyWarsRunDefinition>,
    activity: GraphActivity,
}

impl CurrencyWarsRun {
    pub fn start(
        definition: Arc<CurrencyWarsRunDefinition>,
        instance: ActivityInstanceId,
        master_seed: ActivityMasterSeed,
    ) -> Result<Self, CurrencyWarsRuntimeError> {
        let activity =
            GraphActivity::start(Arc::clone(&definition.activity), instance, master_seed)
                .map_err(debug_error)?
                .into_activity();
        let mut run = Self {
            definition,
            activity,
        };
        run.synchronize_current_node_shop()?;
        Ok(run)
    }

    #[must_use]
    pub fn state_hash(&self) -> ActivityStateHash {
        self.activity.state_hash()
    }

    #[must_use]
    pub fn player_view(&self) -> ActivityPlayerView {
        self.activity.player_view()
    }

    #[must_use]
    pub fn debug_view(&self) -> ActivityDebugView {
        self.activity.debug_view()
    }

    #[must_use]
    pub fn gold(&self) -> u32 {
        u32::try_from(self.integer(GOLD)).unwrap_or_default()
    }

    #[must_use]
    pub fn experience(&self) -> u32 {
        u32::try_from(self.integer(EXPERIENCE)).unwrap_or_default()
    }

    #[must_use]
    pub fn team_level(&self) -> u8 {
        u8::try_from(self.integer(TEAM_LEVEL)).unwrap_or_default()
    }

    #[must_use]
    pub fn squad_hp(&self) -> u32 {
        u32::try_from(self.integer(SQUAD_HP)).unwrap_or_default()
    }

    #[must_use]
    pub fn last_squad_hp_loss(&self) -> u32 {
        u32::try_from(self.integer(LAST_LOSS)).unwrap_or_default()
    }

    #[must_use]
    pub fn last_battle_progress(&self) -> Ratio {
        Ratio::from_scaled(self.fixed_scalar(LAST_PROGRESS))
    }

    #[must_use]
    pub fn last_action_value(&self) -> ActionValue {
        ActionValue::from_scaled(self.fixed_scalar(LAST_ACTION_VALUE)).unwrap_or(ActionValue::ZERO)
    }

    pub fn roster(&self) -> Result<CurrencyWarsRoster, CurrencyWarsRuntimeError> {
        let values = self
            .counter_map(ROSTER)?
            .into_iter()
            .map(|(state, count)| {
                Ok((
                    CurrencyWarsRoleState::decode(state).map_err(debug_error)?,
                    u32::try_from(count).map_err(debug_error)?,
                ))
            })
            .collect::<Result<Vec<_>, CurrencyWarsRuntimeError>>()?;
        CurrencyWarsRoster::new(&self.definition.catalog, values).map_err(debug_error)
    }

    pub fn deployment(&self) -> Result<CurrencyWarsDeployment, CurrencyWarsRuntimeError> {
        let roster = self.roster()?;
        CurrencyWarsDeployment::new_with_back_capacity(
            &self.definition.catalog,
            &roster,
            self.team_level(),
            self.back_capacity(),
            self.counter_map(DEPLOYMENT)?
                .into_iter()
                .map(|(position, state)| {
                    Ok((
                        CurrencyWarsPosition::decode(position).map_err(debug_error)?,
                        CurrencyWarsRoleState::decode(u64::try_from(state).map_err(debug_error)?)
                            .map_err(debug_error)?,
                    ))
                })
                .collect::<Result<Vec<_>, CurrencyWarsRuntimeError>>()?,
        )
        .map_err(debug_error)
    }

    #[must_use]
    pub fn back_capacity(&self) -> u8 {
        u8::try_from(self.integer(BACK_CAPACITY)).unwrap_or_default()
    }

    pub fn deploy(
        &mut self,
        position: CurrencyWarsPosition,
        state: CurrencyWarsRoleState,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let roster = self.roster()?;
        let deployment = self
            .deployment()?
            .deploy(
                &self.definition.catalog,
                &roster,
                self.team_level(),
                self.back_capacity(),
                position,
                state,
            )
            .map_err(debug_error)?;
        self.apply_roster_state(103, &roster, &deployment, 0)
    }

    pub fn undeploy(
        &mut self,
        position: CurrencyWarsPosition,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let roster = self.roster()?;
        let deployment = self
            .deployment()?
            .undeploy(
                &self.definition.catalog,
                &roster,
                self.team_level(),
                self.back_capacity(),
                position,
            )
            .map_err(debug_error)?;
        self.apply_roster_state(104, &roster, &deployment, 0)
    }

    pub fn relocate(
        &mut self,
        from: CurrencyWarsPosition,
        to: CurrencyWarsPosition,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let roster = self.roster()?;
        let deployment = self
            .deployment()?
            .relocate(
                &self.definition.catalog,
                &roster,
                self.team_level(),
                self.back_capacity(),
                from,
                to,
            )
            .map_err(debug_error)?;
        self.apply_roster_state(105, &roster, &deployment, 0)
    }

    pub fn empowerment_snapshot(
        &self,
    ) -> Result<CurrencyWarsEmpowermentSnapshot, CurrencyWarsRuntimeError> {
        self.definition
            .catalog
            .empowerment_snapshot(&self.deployment()?)
            .map_err(debug_error)
    }

    pub fn bond_snapshot(&self) -> Result<CurrencyWarsBondSnapshot, CurrencyWarsRuntimeError> {
        self.bond_snapshot_for(&self.deployment()?, &self.equipment_loadout()?)
    }

    pub fn battle_override_snapshot(
        &self,
    ) -> Result<CurrencyWarsBattleOverrideSnapshot, CurrencyWarsRuntimeError> {
        let deployment = self.deployment()?;
        let loadout = self.equipment_loadout()?;
        let bonds = self.bond_snapshot_for(&deployment, &loadout)?;
        let builds = selected_role_builds(self, &deployment)?;
        let season_id = self
            .definition
            .catalog
            .flow_catalog()
            .difficulties()
            .binary_search_by_key(&self.definition.difficulty, |value| value.source_id)
            .ok()
            .map(|index| self.definition.catalog.flow_catalog().difficulties()[index].season_id)
            .ok_or_else(|| error("Currency Wars difficulty is missing"))?;
        self.definition
            .catalog
            .battle_override_snapshot(
                &deployment,
                &builds
                    .iter()
                    .map(|build| build.override_build)
                    .collect::<Vec<_>>(),
                &bonds.battle_event_ids,
                season_id,
                self.current_battle_boundary()?.lethal_rescue_action_value(),
            )
            .map_err(debug_error)
    }

    /// Freezes every battle-visible Activity contribution at this command boundary.
    pub fn contribution_snapshot(
        &self,
    ) -> Result<CurrencyWarsContributionSnapshot, CurrencyWarsRuntimeError> {
        materialize_contribution(self)
    }

    pub fn select_bond_subtrait(
        &mut self,
        parent: CurrencyWarsBondId,
        child: CurrencyWarsBondId,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let deployment = self.deployment()?;
        let loadout = self.equipment_loadout()?;
        let mut context = self.bond_context()?;
        if !self.definition.catalog.bond_catalog().selection_eligible(
            parent,
            child,
            &deployment,
            &loadout,
            &context,
        ) {
            return Err(error(
                "Currency Wars sub-Bond selection is not currently eligible",
            ));
        }
        context.selected_subtraits.insert(parent, child);
        let snapshot =
            self.definition
                .catalog
                .bond_catalog()
                .resolve(&deployment, &loadout, &context);
        if !snapshot.selected_subtraits.contains(&(parent, child)) {
            return Err(error("Currency Wars sub-Bond parent is not active"));
        }
        self.apply_state(112, bond_operations(&snapshot))
    }

    pub fn receive_equipment(
        &mut self,
        equipment: CurrencyWarsEquipmentId,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        self.definition
            .catalog
            .build_catalog()
            .runtime_equipment(equipment)
            .ok_or_else(|| error("Currency Wars received equipment is missing"))?;
        let mut inventory = self.equipment_inventory()?;
        let count = inventory.entry(equipment).or_default();
        *count = count
            .checked_add(1)
            .ok_or_else(|| error("Currency Wars equipment inventory count overflow"))?;
        self.apply_state(
            109,
            vec![set_counter_map(
                EQUIPMENT_INVENTORY,
                encode_equipment_inventory(&inventory),
            )],
        )
    }

    pub fn equipment_loadout(
        &self,
    ) -> Result<CurrencyWarsEquipmentLoadout, CurrencyWarsRuntimeError> {
        CurrencyWarsEquipmentLoadout::decode(&self.counter_map(EQUIPMENT_LOADOUT)?)
            .map_err(debug_error)
    }

    pub fn equip(
        &mut self,
        role: CurrencyWarsRoleId,
        equipment: CurrencyWarsEquipmentId,
        replace: Option<CurrencyWarsEquipmentSlot>,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let role_definition = self
            .definition
            .catalog
            .role(role)
            .ok_or_else(|| error("Currency Wars equipment role is missing"))?;
        if !self.roster()?.owns_role(role) {
            return Err(error("Currency Wars equipment role is not owned"));
        }
        let build = self.definition.catalog.build_catalog();
        let definition = build
            .runtime_equipment(equipment)
            .ok_or_else(|| error("Currency Wars equipped equipment is missing"))?;
        let mut inventory = self.equipment_inventory()?;
        remove_equipment_inventory(&mut inventory, equipment)?;
        let mut loadout = self.equipment_loadout()?;
        let replaced = loadout
            .equip(
                role_definition,
                definition,
                build.equipment_category_limit(definition.category),
                build.character_implant_slot_limit(),
                replace,
                |id| build.runtime_equipment(id),
            )
            .map_err(debug_error)?;
        if let Some(replaced) = replaced {
            add_equipment_inventory(&mut inventory, replaced)?;
        }
        self.apply_equipment_state(110, &inventory, &loadout)
    }

    pub fn unequip(
        &mut self,
        slot: CurrencyWarsEquipmentSlot,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let mut inventory = self.equipment_inventory()?;
        let mut loadout = self.equipment_loadout()?;
        let equipment = loadout.unequip(slot).map_err(debug_error)?;
        add_equipment_inventory(&mut inventory, equipment)?;
        self.apply_equipment_state(111, &inventory, &loadout)
    }

    pub fn choose_investment(
        &mut self,
        investment: CurrencyWarsInvestmentId,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let definition = self
            .definition
            .catalog
            .investment(investment)
            .ok_or_else(|| error("Currency Wars investment is missing"))?;
        if definition.kind == CurrencyWarsInvestmentKind::Enhancement {
            return self.choose_enhancement(investment);
        }
        if definition.kind == CurrencyWarsInvestmentKind::Augment {
            return self.choose_augment(investment, None);
        }
        if self
            .definition
            .catalog
            .cross_investment_catalog()
            .investment(investment)
            .is_some()
        {
            return self.choose_typed_investment(investment);
        }
        if definition.runtime_binding_exact {
            return Err(error(
                "Currency Wars exact investment binding requires a typed handler",
            ));
        }
        self.apply_state(
            106,
            vec![ActivityOperation::InsertOrderedId {
                slot: slot(INVESTMENTS),
                id: investment.get(),
            }],
        )
    }

    pub fn continue_supply(&mut self) -> Result<(), CurrencyWarsRuntimeError> {
        let view = self.activity.player_view();
        let decision = view
            .decision()
            .filter(|decision| decision.kind() == ActivityDecisionKind::Shop)
            .ok_or_else(|| error("Currency Wars supply node is not active"))?;
        self.activity
            .choose_option(view.state_hash(), decision.id(), supply_option())
            .map_err(debug_error)?;
        self.synchronize_current_node_shop()?;
        Ok(())
    }

    pub fn continue_plane(&mut self) -> Result<(), CurrencyWarsRuntimeError> {
        let transition = self
            .definition
            .flow
            .plane_transition(self.activity.current_node())
            .ok_or_else(|| error("Currency Wars Plane transition is not active"))?;
        let view = self.activity.player_view();
        let decision = view
            .decision()
            .filter(|decision| decision.kind() == ActivityDecisionKind::Route)
            .ok_or_else(|| error("Currency Wars Plane transition is not offered"))?;
        self.activity
            .choose_option(
                view.state_hash(),
                decision.id(),
                plane_option(transition.to_plane),
            )
            .map_err(debug_error)?;
        self.synchronize_current_node_shop()?;
        Ok(())
    }

    #[must_use]
    pub fn current_plane(&self) -> Option<u8> {
        self.definition
            .flow
            .route_index(self.activity.current_node())
            .and_then(|index| {
                self.definition
                    .catalog
                    .route(self.definition.route)
                    .and_then(|route| route.nodes.get(index))
                    .map(|node| node.plane)
            })
            .or_else(|| {
                self.definition
                    .flow
                    .plane_transition(self.activity.current_node())
                    .map(|transition| transition.from_plane)
            })
    }

    #[must_use]
    pub fn current_node(&self) -> Option<&CurrencyWarsNode> {
        self.current_route_node().ok().map(|(_, node)| node)
    }

    pub fn engage_current_node(
        &mut self,
        attempt: AttemptId,
        assembler: &mut CurrencyWarsBattleAssembler,
    ) -> Result<CurrencyWarsBattlePreparation, CurrencyWarsRuntimeError> {
        self.validate_current_battle_entry()?;
        let contribution = self.contribution_snapshot()?;
        let difficulty_level = contribution
            .augment_enemy_difficulty_add
            .iter()
            .try_fold(
                contribution
                    .difficulty
                    .enemy_scaling
                    .enemy_difficulty_level
                    .checked_add(u16::from(contribution.binary_enemy_difficulty_add))
                    .ok_or_else(|| error("Currency Wars binary enemy difficulty overflow"))?,
                |value, (_, additional)| value.checked_add(u16::from(*additional)),
            )
            .ok_or_else(|| error("Currency Wars enemy difficulty overflow"))?;
        let encounter = self.definition.catalog.encounter_catalog();
        let scaling = encounter
            .enemy_scaling(contribution.node.plane, difficulty_level)
            .ok_or_else(|| error("Currency Wars enemy scaling row is missing"))?;
        let materialization = assembler
            .materialize(&contribution, encounter, scaling)
            .map_err(debug_error)?;
        let resolution = self.engage_battle_spec(attempt, materialization.battle_spec().clone())?;
        Ok(CurrencyWarsBattlePreparation {
            resolution,
            materialization,
        })
    }

    fn engage_battle_spec(
        &mut self,
        attempt: AttemptId,
        battle: BattleSpec,
    ) -> Result<GraphActivityPreparationResolution, CurrencyWarsRuntimeError> {
        self.validate_current_battle_entry()?;
        let (index, node) = self.current_route_node()?;
        if !node.kind.battle() || battle.encounter() != node.encounter {
            return Err(error(
                "battle encounter does not match the current Currency Wars node",
            ));
        }
        let view = self.activity.player_view();
        let decision = view
            .decision()
            .filter(|decision| decision.kind() == ActivityDecisionKind::Encounter)
            .ok_or_else(|| error("Currency Wars encounter is not currently offered"))?;
        let option = encounter_option(index)?;
        let path = ActivityScopePath::new(self.activity.instance())
            .enter_section(section(node.plane)?)
            .and_then(|path| path.enter_node(self.activity.current_node()))
            .and_then(|path| path.enter_attempt(attempt))
            .map_err(debug_error)?;
        let participants = self.participant_lock_for_battle(&battle)?;
        let lock = participants.digest();
        let preparation = Arc::new(
            EncounterPreparationDefinition::new(
                preparation_option(index)?,
                EncounterInitiativePolicy::PlayerControlled,
                lock,
                0,
                vec![],
                vec![PreparedBattleVariant::new(
                    vec![],
                    contribution(node),
                    BattleBinding::new(battle, seed_label(node), lock).map_err(debug_error)?,
                )],
            )
            .map_err(debug_error)?,
        );
        let roster = ActivityRosterLock::new(path, participants).map_err(debug_error)?;
        self.activity
            .engage_encounter(
                view.state_hash(),
                decision.id(),
                option,
                ActivityBattlePreparationRequest::new(
                    path,
                    roster,
                    BattleSequence::new(u32::try_from(index + 1).map_err(debug_error)?)
                        .ok_or_else(|| error("Currency Wars battle sequence is zero"))?,
                    0,
                    preparation,
                ),
            )
            .map_err(debug_error)
    }

    fn validate_current_battle_entry(&self) -> Result<(), CurrencyWarsRuntimeError> {
        self.deployment()?
            .validate_battle_ready(&self.definition.catalog)
            .map_err(debug_error)?;
        let (_, node) = self.current_route_node()?;
        if !node.kind.battle() {
            return Err(error("Currency Wars current node is not a battle"));
        }
        let view = self.activity.player_view();
        if view
            .decision()
            .is_none_or(|decision| decision.kind() != ActivityDecisionKind::Encounter)
        {
            return Err(error("Currency Wars encounter is not currently offered"));
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn engage_current_node_fixture(
        &mut self,
        attempt: AttemptId,
        battle: BattleSpec,
    ) -> Result<GraphActivityPreparationResolution, CurrencyWarsRuntimeError> {
        self.engage_battle_spec(attempt, battle)
    }

    pub fn choose_prepared_battle(&mut self) -> Result<(), CurrencyWarsRuntimeError> {
        let (index, _) = self.current_route_node()?;
        self.activity
            .choose_preparation_option(self.state_hash(), preparation_option(index)?)
            .map_err(debug_error)?;
        Ok(())
    }

    pub fn start_pending_battle(
        &mut self,
    ) -> Result<ActivityBattleHandoff, CurrencyWarsRuntimeError> {
        let (index, _) = self.current_route_node()?;
        let participants = self.current_front_participant_ids()?;
        let contract = Arc::new(battle_contract(&participants, index)?);
        self.activity
            .start_pending_battle(self.state_hash(), contract)
            .map_err(debug_error)
    }

    fn participant_lock_for_battle(
        &self,
        battle: &BattleSpec,
    ) -> Result<ParticipantLock, CurrencyWarsRuntimeError> {
        let deployment = self.deployment()?;
        let mut entries = Vec::new();
        for (position, role) in deployment
            .positions()
            .iter()
            .filter(|(position, _)| position.kind() == CurrencyWarsPositionKind::Front)
        {
            let formation = position.index().saturating_sub(1);
            let participant = battle
                .participants()
                .iter()
                .find(|participant| {
                    participant.side() == TeamSide::Player
                        && participant.formation().get() == formation
                })
                .ok_or_else(|| error("Currency Wars front role is missing from BattleSpec"))?;
            let mut hash = Sha256::new();
            hash.update(b"starclock.currency-wars.participant-build.v1");
            hash.update(participant.locked_combatant_digest().bytes());
            let build = BuildDigest::new(hash.finalize().into())
                .ok_or_else(|| error("Currency Wars participant Build digest is zero"))?;
            let source = if self.definition.owned_builds.contains_key(&role.role()) {
                ParticipantSourceKind::CompiledBuild
            } else {
                ParticipantSourceKind::Trial
            };
            entries.push(
                ParticipantLockEntry::new(
                    ParticipantId::new(role.role().get())
                        .ok_or_else(|| error("Currency Wars participant ID is zero"))?,
                    0,
                    formation,
                    participant.combatant().form(),
                    OpaqueParticipantBuild::new(
                        participant.locked_combatant_digest(),
                        build,
                        source,
                    )
                    .map_err(debug_error)?,
                )
                .map_err(debug_error)?,
            );
        }
        let count = u8::try_from(entries.len()).map_err(debug_error)?;
        let policy = ParticipantPolicy::new(
            1,
            count,
            count,
            ParticipantUniquenessScope::Team,
            LoadoutLockScope::Attempt,
        )
        .ok_or_else(|| error("Currency Wars attempt participant policy is invalid"))?;
        ParticipantLock::seal(policy, entries).map_err(debug_error)
    }

    fn current_front_participant_ids(
        &self,
    ) -> Result<Vec<ParticipantId>, CurrencyWarsRuntimeError> {
        self.deployment()?
            .positions()
            .iter()
            .filter(|(position, _)| position.kind() == CurrencyWarsPositionKind::Front)
            .map(|(_, role)| {
                ParticipantId::new(role.role().get())
                    .ok_or_else(|| error("Currency Wars participant ID is zero"))
            })
            .collect()
    }

    pub fn current_battle_boundary(
        &self,
    ) -> Result<&CurrencyWarsBattleBoundary, CurrencyWarsRuntimeError> {
        let (index, _) = self.current_route_node()?;
        self.definition
            .battle_boundary(index)
            .ok_or_else(|| error("Currency Wars battle boundary is missing"))
    }

    pub fn submit_battle_result(
        &mut self,
        result: BattleResult,
    ) -> Result<GraphActivityBattleResolution, CurrencyWarsRuntimeError> {
        self.submit_battle_result_with_follow_up(result, economy::settlement_shop_operations)
    }

    fn submit_battle_result_with_follow_up(
        &mut self,
        result: BattleResult,
        generate: impl FnOnce(
            &CurrencyWarsRunDefinition,
            &ActivityPlayerView,
            &mut ActivityRngStreams,
        ) -> Result<Vec<ActivityOperation>, GraphActivityCommandError>,
    ) -> Result<GraphActivityBattleResolution, CurrencyWarsRuntimeError> {
        let (index, node) = self.current_route_node()?;
        let outcome = result
            .values()
            .iter()
            .find_map(|value| match value {
                ProjectedValue::Outcome(outcome) => Some(*outcome),
                _ => None,
            })
            .ok_or_else(|| error("Currency Wars result outcome is missing"))?;
        let boundary = self
            .definition
            .battle_boundary(index)
            .ok_or_else(|| error("Currency Wars battle boundary is missing"))?;
        let resolved = if outcome == BattleOutcome::Faulted {
            None
        } else {
            let progress = result
                .values()
                .iter()
                .find_map(|value| match value {
                    ProjectedValue::Metric {
                        key,
                        value: MetricValue::Ratio(raw),
                    } if key.as_ref() == CURRENCY_WARS_BATTLE_PROGRESS_KEY => {
                        Some(Ratio::from_scaled(*raw))
                    }
                    _ => None,
                })
                .ok_or_else(|| error("Currency Wars battle progress is missing"))?;
            let remaining_action_value = result
                .values()
                .iter()
                .find_map(|value| match value {
                    ProjectedValue::Metric {
                        key,
                        value: MetricValue::ActionValue(raw),
                    } if key.as_ref() == CURRENCY_WARS_ACTION_VALUE_REMAINING_KEY => {
                        Some(ActionValue::from_scaled(*raw).map_err(debug_error))
                    }
                    _ => None,
                })
                .ok_or_else(|| error("Currency Wars remaining action value is missing"))??;
            Some(boundary.resolve(outcome, progress, remaining_action_value)?)
        };
        let mut operations = if matches!(outcome, BattleOutcome::Won | BattleOutcome::Lost) {
            let reward = self
                .definition
                .catalog
                .experience_reward(self.definition.gambit, node);
            let total = self
                .experience()
                .checked_add(reward)
                .ok_or_else(|| error("Currency Wars experience overflow"))?;
            let (level, experience) =
                advance_team_level(&self.definition.catalog, self.team_level(), total)
                    .map_err(debug_error)?;
            let interest = self
                .definition
                .catalog
                .battle_interest(self.definition.gambit, self.gold());
            let gold = node
                .basic_gold_reward
                .unwrap_or_default()
                .checked_add(interest)
                .ok_or_else(|| error("Currency Wars battle Gold reward overflow"))?;
            vec![
                add_integer(GOLD, i64::from(gold)),
                set_integer(TEAM_LEVEL, i64::from(level)),
                set_integer(EXPERIENCE, i64::from(experience)),
            ]
        } else {
            vec![]
        };
        match outcome {
            BattleOutcome::Won => operations.push(set_integer(LAST_LOSS, 0)),
            BattleOutcome::Lost => {
                let loss = resolved
                    .expect("lost Currency Wars battle has a boundary resolution")
                    .squad_hp_loss();
                operations.extend([
                    ActivityOperation::SetSlot {
                        slot: slot(SQUAD_HP),
                        value: ActivityExpression::Maximum(
                            Box::new(literal_integer(0)),
                            Box::new(ActivityExpression::Subtract(
                                Box::new(ActivityExpression::Slot(slot(SQUAD_HP))),
                                Box::new(literal_integer(i64::from(loss))),
                            )),
                        ),
                    },
                    set_integer(LAST_LOSS, i64::from(loss)),
                ]);
            }
            BattleOutcome::Faulted => {}
            BattleOutcome::Finalized => unreachable!("finalized outcome was rejected"),
        }
        let boundary =
            ActivityProgramDefinition::new(program_id(107), operations).map_err(debug_error)?;
        let definition = Arc::clone(&self.definition);
        let resolution = self
            .activity
            .submit_pending_battle_result_with_generated_follow_up(
                self.state_hash(),
                result,
                Some(&boundary),
                program_id(109),
                move |view, rng| generate(&definition, view, rng),
            )
            .map_err(debug_error)?;
        Ok(resolution)
    }

    #[cfg(test)]
    pub(crate) fn submit_battle_result_with_rejected_follow_up_fixture(
        &mut self,
        result: BattleResult,
    ) -> Result<GraphActivityBattleResolution, CurrencyWarsRuntimeError> {
        use starclock_activity::{
            ActivityRngLabel, GraphActivityCommandError, GraphActivityRuntimeError,
        };

        self.submit_battle_result_with_follow_up(result, |_, _, rng| {
            rng.choose_index(ActivityRngLabel::Shop, 999, 3)
                .map_err(GraphActivityCommandError::Rng)?;
            Err(GraphActivityCommandError::Runtime(
                GraphActivityRuntimeError::InvalidBoundaryProgram,
            ))
        })
    }

    fn current_route_node(&self) -> Result<(usize, &CurrencyWarsNode), CurrencyWarsRuntimeError> {
        let index = self
            .definition
            .flow
            .route_index(self.activity.current_node())
            .ok_or_else(|| error("Currency Wars is not at a route node"))?;
        let route = self
            .definition
            .catalog
            .route(self.definition.route)
            .expect("Currency Wars route was validated");
        Ok((index, &route.nodes[index]))
    }

    fn apply_roster_state(
        &mut self,
        id: u32,
        roster: &CurrencyWarsRoster,
        deployment: &CurrencyWarsDeployment,
        gold_delta: i64,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let mut inventory = self.equipment_inventory()?;
        let mut loadout = self.equipment_loadout()?;
        let removed_roles = loadout
            .slots()
            .keys()
            .map(|slot| slot.role())
            .filter(|role| !roster.owns_role(*role))
            .collect::<std::collections::BTreeSet<_>>();
        for role in removed_roles {
            let equipment = loadout
                .for_role(role)
                .map(|(_, equipment)| equipment)
                .collect::<Vec<_>>();
            loadout.remove_role(role);
            for equipment in equipment {
                add_equipment_inventory(&mut inventory, equipment)?;
            }
        }
        let snapshot = self.bond_snapshot_for(deployment, &loadout)?;
        let mut operations = vec![
            set_counter_map(ROSTER, roster.encoded()),
            set_counter_map(DEPLOYMENT, deployment.encoded()),
            set_counter_map(EQUIPMENT_INVENTORY, encode_equipment_inventory(&inventory)),
            set_counter_map(EQUIPMENT_LOADOUT, loadout.encoded()),
        ];
        operations.extend(bond_operations(&snapshot));
        if gold_delta != 0 {
            operations.push(add_integer(GOLD, gold_delta));
        }
        self.apply_state(id, operations)
    }

    pub(super) fn equipment_inventory(
        &self,
    ) -> Result<BTreeMap<CurrencyWarsEquipmentId, u32>, CurrencyWarsRuntimeError> {
        self.counter_map(EQUIPMENT_INVENTORY)?
            .into_iter()
            .map(|(raw_id, raw_count)| {
                let id = u32::try_from(raw_id)
                    .ok()
                    .and_then(CurrencyWarsEquipmentId::new)
                    .ok_or_else(|| error("Currency Wars equipment inventory ID is invalid"))?;
                let count = u32::try_from(raw_count)
                    .map_err(|_| error("Currency Wars equipment inventory count is invalid"))?;
                Ok((id, count))
            })
            .collect()
    }

    fn apply_equipment_state(
        &mut self,
        id: u32,
        inventory: &BTreeMap<CurrencyWarsEquipmentId, u32>,
        loadout: &CurrencyWarsEquipmentLoadout,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let snapshot = self.bond_snapshot_for(&self.deployment()?, loadout)?;
        let mut operations = vec![
            set_counter_map(EQUIPMENT_INVENTORY, encode_equipment_inventory(inventory)),
            set_counter_map(EQUIPMENT_LOADOUT, loadout.encoded()),
        ];
        operations.extend(bond_operations(&snapshot));
        self.apply_state(id, operations)
    }

    fn bond_context(&self) -> Result<CurrencyWarsBondResolutionContext, CurrencyWarsRuntimeError> {
        let selected_subtraits = self
            .counter_map(BOND_SELECTIONS)?
            .into_iter()
            .map(|(raw_parent, raw_child)| {
                let parent = u32::try_from(raw_parent)
                    .ok()
                    .and_then(CurrencyWarsBondId::new)
                    .ok_or_else(|| error("Currency Wars selected parent Bond ID is invalid"))?;
                let child = u32::try_from(raw_child)
                    .ok()
                    .and_then(CurrencyWarsBondId::new)
                    .ok_or_else(|| error("Currency Wars selected sub-Bond ID is invalid"))?;
                Ok((parent, child))
            })
            .collect::<Result<BTreeMap<_, _>, CurrencyWarsRuntimeError>>()?;
        Ok(CurrencyWarsBondResolutionContext {
            selected_subtraits,
            module_id: self
                .definition
                .catalog
                .flow_catalog()
                .profile_module_source_id(),
            ..CurrencyWarsBondResolutionContext::default()
        })
    }

    fn bond_snapshot_for(
        &self,
        deployment: &CurrencyWarsDeployment,
        loadout: &CurrencyWarsEquipmentLoadout,
    ) -> Result<CurrencyWarsBondSnapshot, CurrencyWarsRuntimeError> {
        Ok(self.definition.catalog.bond_catalog().resolve(
            deployment,
            loadout,
            &self.bond_context()?,
        ))
    }

    fn apply_state(
        &mut self,
        id: u32,
        operations: Vec<ActivityOperation>,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let program =
            ActivityProgramDefinition::new(program_id(id), operations).map_err(debug_error)?;
        self.activity
            .apply_boundary_program(self.state_hash(), &program)
            .map_err(debug_error)?;
        Ok(())
    }

    fn integer(&self, raw: u32) -> i64 {
        self.value(raw)
            .and_then(|value| match value {
                ActivityValue::BoundedInteger(value) => Ok(value),
                _ => Err(error("Currency Wars integer slot has the wrong type")),
            })
            .unwrap_or_default()
    }

    fn fixed_scalar(&self, raw: u32) -> i64 {
        self.value(raw)
            .and_then(|value| match value {
                ActivityValue::FixedScalar(value) => Ok(value),
                _ => Err(error("Currency Wars fixed-scalar slot has the wrong type")),
            })
            .unwrap_or_default()
    }

    fn counter_map(&self, raw: u32) -> Result<Vec<(u64, i64)>, CurrencyWarsRuntimeError> {
        self.value(raw).and_then(|value| match value {
            ActivityValue::BoundedCounterMap(values) => Ok(values.to_vec()),
            _ => Err(error("Currency Wars counter slot has the wrong type")),
        })
    }

    pub(super) fn ordered_ids(&self, raw: u32) -> Result<Box<[u64]>, CurrencyWarsRuntimeError> {
        self.value(raw).and_then(|value| match value {
            ActivityValue::OrderedIdSet(values) => Ok(values),
            _ => Err(error("Currency Wars ordered-ID slot has the wrong type")),
        })
    }

    fn value(&self, raw: u32) -> Result<ActivityValue, CurrencyWarsRuntimeError> {
        self.activity
            .player_view()
            .slots()
            .iter()
            .find(|entry| entry.id() == slot(raw))
            .map(|entry| entry.value().clone())
            .ok_or_else(|| error("Currency Wars state slot is missing"))
    }
}

use self::state::{
    add_equipment_inventory, add_integer, always, bond_operations, checkpoint_option, contribution,
    encode_equipment_inventory, encounter_option, literal_integer, plane_option,
    preparation_option, program_id, remove_equipment_inventory, section, seed_label,
    set_counter_map, set_integer, set_ordered_ids, set_value, slot, source, supply_option,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRuntimeError {
    message: Box<str>,
}

impl std::fmt::Display for CurrencyWarsRuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CurrencyWarsRuntimeError {}

fn enemy_affix_seed(
    identity: &ActivityDefinitionIdentity,
    route: CurrencyWarsRouteId,
    difficulty: u32,
    gambit: CurrencyWarsGambit,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"starclock.currency-wars.enemy-affix-run.v1");
    hash.update(identity.definition_digest().bytes());
    hash.update(identity.config_digest().bytes());
    hash.update(route.get().to_le_bytes());
    hash.update(difficulty.to_le_bytes());
    hash.update([match gambit {
        CurrencyWarsGambit::Standard => 1,
        CurrencyWarsGambit::Overclock => 2,
    }]);
    hash.finalize().into()
}

pub(super) fn error(message: &str) -> CurrencyWarsRuntimeError {
    CurrencyWarsRuntimeError {
        message: message.into(),
    }
}

pub(super) fn debug_error(value: impl std::fmt::Debug) -> CurrencyWarsRuntimeError {
    CurrencyWarsRuntimeError {
        message: format!("{value:?}").into_boxed_str(),
    }
}
