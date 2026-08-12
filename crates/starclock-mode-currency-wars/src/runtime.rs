use std::sync::Arc;

use starclock_activity::{
    ActivityBattleHandoff, ActivityBattlePreparationRequest, ActivityBattleResultContract,
    ActivityCondition, ActivityDecisionKind, ActivityDefinitionIdentity, ActivityExpression,
    ActivityInstanceId, ActivityMasterSeed, ActivityMetricProjectionBinding, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivityParticipantCarryDefinition,
    ActivityProgramDefinition, ActivityProgramId, ActivityRandomPolicies, ActivityRngLabel,
    ActivityRosterLock, ActivityScope, ActivityScopePath, ActivitySlotDefinition, ActivitySlotId,
    ActivityStateDefinition, ActivityStateHash, ActivityStateSource, ActivityStateVisibility,
    ActivityValue, AttemptId, BattleBinding, BattleOutcome, BattleResult, BattleResultProjection,
    BattleSequence, EncounterInitiativePolicy, EncounterPreparationDefinition, EnergyCarryPolicy,
    GraphActivity, GraphActivityBattleResolution, GraphActivityDefinition,
    GraphActivityNodeProgram, GraphActivityPreparationResolution, HpCarryPolicy, LifeCarryPolicy,
    MetricSettlementPolicy, MetricValueKind, NodeId, ParticipantId, ParticipantLock,
    PresenceCarryPolicy, ProjectedValue, ProjectionField, ProjectionId, SectionId, SlotCarryPolicy,
    TechniqueContributionDigest,
};
use starclock_combat::BattleSpec;

use crate::{
    CurrencyWarsCatalog, CurrencyWarsDeployment, CurrencyWarsFlow, CurrencyWarsGambit,
    CurrencyWarsInvestmentId, CurrencyWarsNode, CurrencyWarsPosition, CurrencyWarsRoleId,
    CurrencyWarsRoleState, CurrencyWarsRoster, CurrencyWarsRouteId, advance_team_level,
};

pub const CURRENCY_WARS_SQUAD_HP_LOSS_KEY: &str = "currency_wars_squad_hp_loss";
pub const CURRENCY_WARS_ACTION_VALUE_REMAINING_KEY: &str = "currency_wars_action_value_remaining";

const SECTION: u32 = 1;
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
const INVESTMENTS: u32 = 11;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRunSetup {
    pub initial_gold: u32,
    pub initial_team_level: u8,
    pub initial_experience: u32,
    pub roster: CurrencyWarsRoster,
    pub deployment: CurrencyWarsDeployment,
}

impl Default for CurrencyWarsRunSetup {
    fn default() -> Self {
        Self {
            initial_gold: 0,
            initial_team_level: 1,
            initial_experience: 0,
            roster: CurrencyWarsRoster::default(),
            deployment: CurrencyWarsDeployment::default(),
        }
    }
}

#[derive(Debug)]
pub struct CurrencyWarsRunDefinition {
    catalog: Arc<CurrencyWarsCatalog>,
    route: CurrencyWarsRouteId,
    difficulty: u32,
    gambit: CurrencyWarsGambit,
    flow: CurrencyWarsFlow,
    activity: Arc<GraphActivityDefinition>,
    participants: Arc<ParticipantLock>,
    contracts: Box<[Option<Arc<ActivityBattleResultContract>>]>,
}

impl CurrencyWarsRunDefinition {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        identity: ActivityDefinitionIdentity,
        catalog: Arc<CurrencyWarsCatalog>,
        route: CurrencyWarsRouteId,
        difficulty: u32,
        gambit: CurrencyWarsGambit,
        participants: ParticipantLock,
        setup: CurrencyWarsRunSetup,
    ) -> Result<Self, CurrencyWarsRuntimeError> {
        let route_definition = catalog
            .route(route)
            .ok_or_else(|| error("Currency Wars route is not in the catalog"))?;
        if !catalog
            .difficulties()
            .iter()
            .any(|candidate| candidate.source_id == difficulty)
        {
            return Err(error("Currency Wars difficulty is not in the catalog"));
        }
        validate_participants(&participants)?;
        setup
            .deployment
            .validate(&catalog, &setup.roster, setup.initial_team_level)
            .map_err(debug_error)?;
        let (level, experience) =
            advance_team_level(&catalog, setup.initial_team_level, setup.initial_experience)
                .map_err(debug_error)?;
        let flow = CurrencyWarsFlow::compile(route_definition).map_err(debug_error)?;
        let state = activity_state(&catalog, &setup, level, experience)?;
        let programs = node_programs(route_definition, &flow)?;
        let participant_ids = participant_ids(&participants);
        let contracts = route_definition
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                node.kind
                    .battle()
                    .then(|| battle_contract(&participant_ids, index).map(Arc::new))
                    .transpose()
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
            flow,
            activity,
            participants,
            contracts: contracts.into_boxed_slice(),
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
}

#[derive(Debug)]
pub struct CurrencyWarsRun {
    definition: Arc<CurrencyWarsRunDefinition>,
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
    pub fn player_view(&self) -> starclock_activity::ActivityPlayerView {
        self.activity.player_view()
    }

    #[must_use]
    pub fn debug_view(&self) -> starclock_activity::ActivityDebugView {
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
        CurrencyWarsDeployment::new(
            &self.definition.catalog,
            &roster,
            self.team_level(),
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

    pub fn refresh_shop(&mut self) -> Result<Box<[CurrencyWarsRoleId]>, CurrencyWarsRuntimeError> {
        let cost = self.definition.catalog.refresh_cost();
        if self.gold() < cost {
            return Err(error("Currency Wars refresh requires more Gold"));
        }
        let offer = self
            .definition
            .catalog
            .offer(self.team_level())
            .ok_or_else(|| error("Currency Wars offer level is missing"))?;
        let candidates = offer
            .candidates
            .iter()
            .map(|role| {
                let definition = self
                    .definition
                    .catalog
                    .role(*role)
                    .ok_or_else(|| error("Currency Wars offer role is missing"))?;
                let weight = offer.rarity_weights[usize::from(definition.rarity - 1)];
                Ok((
                    ActivityOptionDefinition::new(
                        shop_option(*role)?,
                        i32::try_from(role.get()).map_err(debug_error)?,
                        always(),
                        vec![ActivityOperation::InsertOrderedId {
                            slot: slot(SHOP_OFFERS),
                            id: u64::from(role.get()),
                        }],
                    ),
                    u64::from(weight),
                ))
            })
            .filter(
                |candidate: &Result<_, CurrencyWarsRuntimeError>| match candidate {
                    Ok((_, weight)) => *weight > 0,
                    Err(_) => true,
                },
            )
            .collect::<Result<Vec<_>, _>>()?;
        let width = u16::from(self.definition.catalog.cards_per_refresh());
        let resolution = self
            .activity
            .apply_random_option_boundary(
                self.state_hash(),
                program_id(100),
                ActivityRngLabel::Reward,
                1,
                2,
                width,
                width,
                &[
                    ActivityOperation::AddToSlot {
                        slot: slot(GOLD),
                        delta: literal_integer(-i64::from(cost)),
                    },
                    ActivityOperation::SetOrderedIdSet {
                        slot: slot(SHOP_OFFERS),
                        values: Box::new([]),
                    },
                ],
                &candidates,
            )
            .map_err(debug_error)?;
        resolution
            .selected_options()
            .iter()
            .map(|option| role_from_shop_option(*option))
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }

    pub fn buy_role(&mut self, role: CurrencyWarsRoleId) -> Result<(), CurrencyWarsRuntimeError> {
        if !self.shop_offers()?.contains(&role) {
            return Err(error("Currency Wars role is not in the current shop offer"));
        }
        let definition = self
            .definition
            .catalog
            .role(role)
            .ok_or_else(|| error("Currency Wars role is missing"))?;
        let cost = self
            .definition
            .catalog
            .price(definition.rarity)
            .and_then(|price| price.buy(1))
            .ok_or_else(|| error("Currency Wars role buy price is missing"))?;
        if self.gold() < cost {
            return Err(error("Currency Wars role purchase requires more Gold"));
        }
        let roster = self
            .roster()?
            .acquire(&self.definition.catalog, role)
            .map_err(debug_error)?;
        let deployment = self.deployment()?.reconcile_roster(&roster);
        deployment
            .validate(&self.definition.catalog, &roster, self.team_level())
            .map_err(debug_error)?;
        let offers = self
            .shop_offers()?
            .into_iter()
            .filter(|candidate| *candidate != role)
            .map(|candidate| u64::from(candidate.get()))
            .collect::<Vec<_>>();
        self.apply_state(
            101,
            vec![
                add_integer(GOLD, -i64::from(cost)),
                set_counter_map(ROSTER, roster.encoded()),
                set_counter_map(DEPLOYMENT, deployment.encoded()),
                set_counter_map(BONDS, deployment.bond_levels(&self.definition.catalog)),
                set_ordered_ids(SHOP_OFFERS, offers.into_boxed_slice()),
            ],
        )
    }

    pub fn sell_role(
        &mut self,
        state: CurrencyWarsRoleState,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let definition = self
            .definition
            .catalog
            .role(state.role())
            .ok_or_else(|| error("Currency Wars role is missing"))?;
        let price = self
            .definition
            .catalog
            .price(definition.rarity)
            .and_then(|rule| rule.sell(state.star()))
            .ok_or_else(|| error("Currency Wars role sell price is missing"))?;
        let roster = self.roster()?.sell(state).map_err(debug_error)?;
        let deployment = self.deployment()?.reconcile_roster(&roster);
        self.apply_roster_state(102, &roster, &deployment, i64::from(price))
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
                position,
            )
            .map_err(debug_error)?;
        self.apply_roster_state(104, &roster, &deployment, 0)
    }

    pub fn buy_experience(&mut self) -> Result<(), CurrencyWarsRuntimeError> {
        let cost = self.definition.catalog.direct_experience_cost();
        if self.gold() < cost {
            return Err(error("Currency Wars level purchase requires more Gold"));
        }
        let total = self
            .experience()
            .checked_add(self.definition.catalog.direct_experience_gain())
            .ok_or_else(|| error("Currency Wars experience overflow"))?;
        let (level, experience) =
            advance_team_level(&self.definition.catalog, self.team_level(), total)
                .map_err(debug_error)?;
        self.apply_state(
            105,
            vec![
                add_integer(GOLD, -i64::from(cost)),
                set_integer(TEAM_LEVEL, i64::from(level)),
                set_integer(EXPERIENCE, i64::from(experience)),
            ],
        )
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
        Ok(())
    }

    pub fn engage_current_node(
        &mut self,
        attempt: AttemptId,
        battle: BattleSpec,
    ) -> Result<GraphActivityPreparationResolution, CurrencyWarsRuntimeError> {
        if self.deployment()?.positions().is_empty() {
            return Err(error("Currency Wars battle requires a deployed role"));
        }
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
        let lock = self.definition.participants.digest();
        let preparation = Arc::new(
            EncounterPreparationDefinition::new(
                preparation_option(index)?,
                EncounterInitiativePolicy::PlayerControlled,
                lock,
                0,
                vec![],
                vec![starclock_activity::PreparedBattleVariant::new(
                    vec![],
                    contribution(node),
                    BattleBinding::new(battle, seed_label(node), lock).map_err(debug_error)?,
                )],
            )
            .map_err(debug_error)?,
        );
        let path = ActivityScopePath::new(self.activity.instance())
            .enter_section(section())
            .and_then(|path| path.enter_node(self.activity.current_node()))
            .and_then(|path| path.enter_attempt(attempt))
            .map_err(debug_error)?;
        let roster = ActivityRosterLock::new(
            ActivityScopePath::new(self.activity.instance())
                .enter_section(section())
                .map_err(debug_error)?,
            (*self.definition.participants).clone(),
        )
        .map_err(debug_error)?;
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
        let contract = self
            .definition
            .contracts
            .get(index)
            .and_then(Option::as_ref)
            .ok_or_else(|| error("Currency Wars battle contract is missing"))?;
        self.activity
            .start_pending_battle(self.state_hash(), Arc::clone(contract))
            .map_err(debug_error)
    }

    pub fn submit_battle_result(
        &mut self,
        result: BattleResult,
    ) -> Result<GraphActivityBattleResolution, CurrencyWarsRuntimeError> {
        let (_, node) = self.current_route_node()?;
        let outcome = result
            .values()
            .iter()
            .find_map(|value| match value {
                ProjectedValue::Outcome(outcome) => Some(*outcome),
                _ => None,
            })
            .ok_or_else(|| error("Currency Wars result outcome is missing"))?;
        let operations = match outcome {
            BattleOutcome::Won | BattleOutcome::Finalized => {
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
                vec![
                    add_integer(GOLD, i64::from(node.basic_gold_reward.unwrap_or_default())),
                    set_integer(TEAM_LEVEL, i64::from(level)),
                    set_integer(EXPERIENCE, i64::from(experience)),
                    set_integer(LAST_LOSS, 0),
                ]
            }
            BattleOutcome::Lost => vec![
                ActivityOperation::SetSlot {
                    slot: slot(SQUAD_HP),
                    value: ActivityExpression::Maximum(
                        Box::new(literal_integer(0)),
                        Box::new(ActivityExpression::Subtract(
                            Box::new(ActivityExpression::Slot(slot(SQUAD_HP))),
                            Box::new(ActivityExpression::Slot(slot(LAST_LOSS))),
                        )),
                    ),
                },
                set_integer(LAST_LOSS, 0),
            ],
            BattleOutcome::Faulted => vec![],
        };
        let boundary =
            ActivityProgramDefinition::new(program_id(107), operations).map_err(debug_error)?;
        self.activity
            .submit_pending_battle_result_with_boundary_program(
                self.state_hash(),
                result,
                Some(&boundary),
            )
            .map_err(debug_error)
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

    fn shop_offers(&self) -> Result<Vec<CurrencyWarsRoleId>, CurrencyWarsRuntimeError> {
        self.value(SHOP_OFFERS).and_then(|value| match value {
            ActivityValue::OrderedIdSet(values) => values
                .iter()
                .map(|raw| {
                    u32::try_from(*raw)
                        .ok()
                        .and_then(CurrencyWarsRoleId::new)
                        .ok_or_else(|| error("Currency Wars shop role ID is invalid"))
                })
                .collect(),
            _ => Err(error("Currency Wars shop slot has the wrong type")),
        })
    }

    fn apply_roster_state(
        &mut self,
        id: u32,
        roster: &CurrencyWarsRoster,
        deployment: &CurrencyWarsDeployment,
        gold_delta: i64,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let mut operations = vec![
            set_counter_map(ROSTER, roster.encoded()),
            set_counter_map(DEPLOYMENT, deployment.encoded()),
            set_counter_map(BONDS, deployment.bond_levels(&self.definition.catalog)),
        ];
        if gold_delta != 0 {
            operations.push(add_integer(GOLD, gold_delta));
        }
        self.apply_state(id, operations)
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

    fn counter_map(&self, raw: u32) -> Result<Vec<(u64, i64)>, CurrencyWarsRuntimeError> {
        self.value(raw).and_then(|value| match value {
            ActivityValue::BoundedCounterMap(values) => Ok(values.to_vec()),
            _ => Err(error("Currency Wars counter slot has the wrong type")),
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

fn activity_state(
    catalog: &CurrencyWarsCatalog,
    setup: &CurrencyWarsRunSetup,
    level: u8,
    experience: u32,
) -> Result<ActivityStateDefinition, CurrencyWarsRuntimeError> {
    let roles = u32::try_from(catalog.roles().len()).map_err(debug_error)?;
    let investments = u32::try_from(catalog.investments().len()).map_err(debug_error)?;
    let slots = vec![
        integer_slot(GOLD, i64::from(setup.initial_gold), i64::MAX)?,
        integer_slot(EXPERIENCE, i64::from(experience), i64::MAX)?,
        integer_slot(TEAM_LEVEL, i64::from(level), 10)?,
        integer_slot(SQUAD_HP, i64::from(catalog.initial_squad_hp()), i64::MAX)?,
        integer_slot(LAST_LOSS, 0, i64::MAX)?,
        integer_slot(LAST_ACTION_VALUE, 0, i64::MAX)?,
        map_slot(
            ROSTER,
            setup.roster.encoded(),
            0,
            i64::from(u32::MAX),
            roles.saturating_mul(4),
        )?,
        map_slot(
            DEPLOYMENT,
            setup.deployment.encoded(),
            1,
            i64::MAX,
            u32::from(catalog.front_cap()) + u32::from(catalog.back_cap()),
        )?,
        map_slot(
            BONDS,
            setup.deployment.bond_levels(catalog),
            0,
            i64::from(u8::MAX),
            u32::try_from(catalog.bonds().len()).map_err(debug_error)?,
        )?,
        set_slot(SHOP_OFFERS, Box::new([]), roles)?,
        set_slot(INVESTMENTS, Box::new([]), investments)?,
    ];
    ActivityStateDefinition::new(slots, vec![], vec![]).map_err(debug_error)
}

fn node_programs(
    route: &crate::CurrencyWarsRoute,
    flow: &CurrencyWarsFlow,
) -> Result<Vec<GraphActivityNodeProgram>, CurrencyWarsRuntimeError> {
    let mut programs = Vec::new();
    for (index, route_node) in route.nodes.iter().enumerate() {
        let action = flow
            .activity_node(index)
            .ok_or_else(|| error("Currency Wars action node is missing"))?;
        let (kind, option) = if route_node.kind.battle() {
            (ActivityDecisionKind::Encounter, encounter_option(index)?)
        } else {
            (ActivityDecisionKind::Shop, supply_option())
        };
        programs.push(GraphActivityNodeProgram::new(
            action,
            offer_program(action, kind, option, vec![])?,
        ));
        if let Some(loss) = flow.loss_node(index) {
            let next = route
                .nodes
                .get(index + 1)
                .and_then(|_| flow.activity_node(index + 1))
                .unwrap_or(flow.completed());
            let continue_edge = edge_to(flow, loss, next)?;
            let fail_edge = edge_to(flow, loss, flow.failed())?;
            let operations = vec![ActivityOperation::Conditional {
                condition: ActivityCondition::LessThan(
                    ActivityExpression::Slot(slot(SQUAD_HP)),
                    literal_integer(1),
                ),
                if_true: Box::new([ActivityOperation::Traverse(fail_edge)]),
                if_false: Box::new([ActivityOperation::Traverse(continue_edge)]),
            }];
            programs.push(GraphActivityNodeProgram::new(
                loss,
                offer_program(
                    loss,
                    ActivityDecisionKind::Checkpoint,
                    checkpoint_option(index)?,
                    operations,
                )?,
            ));
        }
    }
    Ok(programs)
}

fn offer_program(
    node: NodeId,
    kind: ActivityDecisionKind,
    option: ActivityOptionId,
    operations: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, CurrencyWarsRuntimeError> {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(node.get()).expect("Currency Wars node program ID is non-zero"),
        vec![ActivityOperation::Offer {
            kind,
            options: Box::new([ActivityOptionDefinition::new(
                option,
                1,
                always(),
                operations,
            )]),
        }],
    )
    .map_err(debug_error)
}

fn edge_to(
    flow: &CurrencyWarsFlow,
    from: NodeId,
    to: NodeId,
) -> Result<starclock_activity::ActivityEdgeId, CurrencyWarsRuntimeError> {
    flow.graph()
        .edges()
        .iter()
        .find(|edge| edge.from() == from && edge.to() == to)
        .map(|edge| edge.id())
        .ok_or_else(|| error("Currency Wars checkpoint edge is missing"))
}

fn battle_contract(
    participants: &[ParticipantId],
    index: usize,
) -> Result<ActivityBattleResultContract, CurrencyWarsRuntimeError> {
    let mut fields = vec![
        ProjectionField::Outcome,
        ProjectionField::FinalStateHash,
        ProjectionField::EventDigest,
        ProjectionField::TerminalFault,
    ];
    fields.extend(
        participants
            .iter()
            .copied()
            .map(ProjectionField::ParticipantState),
    );
    fields.extend([
        ProjectionField::Metric {
            key: CURRENCY_WARS_SQUAD_HP_LOSS_KEY.into(),
            kind: MetricValueKind::BoundedInteger,
        },
        ProjectionField::Metric {
            key: CURRENCY_WARS_ACTION_VALUE_REMAINING_KEY.into(),
            kind: MetricValueKind::BoundedInteger,
        },
    ]);
    let projection = Arc::new(
        BattleResultProjection::new(
            ProjectionId::new(u32::try_from(index + 1).map_err(debug_error)?)
                .ok_or_else(|| error("Currency Wars projection ID is zero"))?,
            fields,
        )
        .map_err(debug_error)?,
    );
    ActivityBattleResultContract::new(
        projection,
        participants
            .iter()
            .copied()
            .map(|participant| {
                ActivityParticipantCarryDefinition::new(
                    participant,
                    HpCarryPolicy::CarryExact,
                    EnergyCarryPolicy::CarryExact,
                    LifeCarryPolicy::CarryExact,
                    PresenceCarryPolicy::CarryExact,
                )
            })
            .collect(),
        vec![
            metric(CURRENCY_WARS_SQUAD_HP_LOSS_KEY, LAST_LOSS)?,
            metric(CURRENCY_WARS_ACTION_VALUE_REMAINING_KEY, LAST_ACTION_VALUE)?,
        ],
    )
    .map_err(debug_error)
}

fn metric(
    key: &str,
    raw: u32,
) -> Result<ActivityMetricProjectionBinding, CurrencyWarsRuntimeError> {
    ActivityMetricProjectionBinding::new(
        key,
        MetricValueKind::BoundedInteger,
        slot(raw),
        MetricSettlementPolicy::Replace,
    )
    .ok_or_else(|| error("Currency Wars metric binding is invalid"))
}

fn validate_participants(participants: &ParticipantLock) -> Result<(), CurrencyWarsRuntimeError> {
    if participants.policy().team_count() != 1 || participant_ids(participants).is_empty() {
        return Err(error("Currency Wars runtime requires one non-empty team"));
    }
    Ok(())
}

fn participant_ids(participants: &ParticipantLock) -> Vec<ParticipantId> {
    participants
        .entries()
        .iter()
        .filter(|entry| entry.team_index() == 0)
        .map(|entry| entry.participant())
        .collect()
}

fn integer_slot(
    raw: u32,
    initial: i64,
    maximum: i64,
) -> Result<ActivitySlotDefinition, CurrencyWarsRuntimeError> {
    ActivitySlotDefinition::new_with_policy(
        slot(raw),
        ActivityScope::Activity,
        ActivityValue::BoundedInteger(initial),
        Some((0, maximum)),
        None,
        vec![],
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Player,
        source(raw),
    )
    .map_err(debug_error)
}

fn map_slot(
    raw: u32,
    initial: Box<[(u64, i64)]>,
    minimum: i64,
    maximum: i64,
    maximum_entries: u32,
) -> Result<ActivitySlotDefinition, CurrencyWarsRuntimeError> {
    ActivitySlotDefinition::new_with_policy(
        slot(raw),
        ActivityScope::Activity,
        ActivityValue::BoundedCounterMap(initial),
        Some((minimum, maximum)),
        Some(maximum_entries.max(1)),
        vec![],
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Player,
        source(raw),
    )
    .map_err(debug_error)
}

fn set_slot(
    raw: u32,
    initial: Box<[u64]>,
    maximum_entries: u32,
) -> Result<ActivitySlotDefinition, CurrencyWarsRuntimeError> {
    ActivitySlotDefinition::new_with_policy(
        slot(raw),
        ActivityScope::Activity,
        ActivityValue::OrderedIdSet(initial),
        None,
        Some(maximum_entries.max(1)),
        vec![],
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Player,
        source(raw),
    )
    .map_err(debug_error)
}

fn set_integer(raw: u32, value: i64) -> ActivityOperation {
    set_value(raw, ActivityValue::BoundedInteger(value))
}

fn set_value(raw: u32, value: ActivityValue) -> ActivityOperation {
    ActivityOperation::SetSlot {
        slot: slot(raw),
        value: ActivityExpression::Literal(value),
    }
}

fn set_counter_map(raw: u32, values: Box<[(u64, i64)]>) -> ActivityOperation {
    ActivityOperation::SetCounterMap {
        slot: slot(raw),
        values,
    }
}

fn set_ordered_ids(raw: u32, values: Box<[u64]>) -> ActivityOperation {
    ActivityOperation::SetOrderedIdSet {
        slot: slot(raw),
        values,
    }
}

fn add_integer(raw: u32, delta: i64) -> ActivityOperation {
    ActivityOperation::AddToSlot {
        slot: slot(raw),
        delta: literal_integer(delta),
    }
}

fn literal_integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn always() -> ActivityCondition {
    ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(true)))
}

fn shop_option(role: CurrencyWarsRoleId) -> Result<ActivityOptionId, CurrencyWarsRuntimeError> {
    1_000_000_000_u64
        .checked_add(u64::from(role.get()))
        .and_then(ActivityOptionId::new)
        .ok_or_else(|| error("Currency Wars shop option ID overflow"))
}

fn role_from_shop_option(
    option: ActivityOptionId,
) -> Result<CurrencyWarsRoleId, CurrencyWarsRuntimeError> {
    option
        .get()
        .checked_sub(1_000_000_000)
        .and_then(|raw| u32::try_from(raw).ok())
        .and_then(CurrencyWarsRoleId::new)
        .ok_or_else(|| error("Currency Wars shop option is invalid"))
}

fn encounter_option(index: usize) -> Result<ActivityOptionId, CurrencyWarsRuntimeError> {
    2_000_000_000_u64
        .checked_add(u64::try_from(index).map_err(debug_error)?)
        .and_then(ActivityOptionId::new)
        .ok_or_else(|| error("Currency Wars encounter option ID overflow"))
}

fn preparation_option(index: usize) -> Result<ActivityOptionId, CurrencyWarsRuntimeError> {
    3_000_000_000_u64
        .checked_add(u64::try_from(index).map_err(debug_error)?)
        .and_then(ActivityOptionId::new)
        .ok_or_else(|| error("Currency Wars preparation option ID overflow"))
}

fn checkpoint_option(index: usize) -> Result<ActivityOptionId, CurrencyWarsRuntimeError> {
    4_000_000_000_u64
        .checked_add(u64::try_from(index).map_err(debug_error)?)
        .and_then(ActivityOptionId::new)
        .ok_or_else(|| error("Currency Wars checkpoint option ID overflow"))
}

fn supply_option() -> ActivityOptionId {
    ActivityOptionId::new(5_000_000_000).expect("Currency Wars supply option ID is non-zero")
}

fn program_id(raw: u32) -> ActivityProgramId {
    ActivityProgramId::new(raw).expect("Currency Wars boundary program ID is non-zero")
}

fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("Currency Wars slot ID is non-zero")
}

fn source(raw: u32) -> ActivityStateSource {
    ActivityStateSource::new(u64::from(raw)).expect("Currency Wars state source is non-zero")
}

fn section() -> SectionId {
    SectionId::new(SECTION).expect("Currency Wars section ID is non-zero")
}

fn contribution(node: &CurrencyWarsNode) -> TechniqueContributionDigest {
    let mut bytes = [0_u8; 32];
    bytes[..4].copy_from_slice(&node.id.get().to_le_bytes());
    TechniqueContributionDigest::new(bytes).expect("Currency Wars node contribution is non-zero")
}

fn seed_label(node: &CurrencyWarsNode) -> Box<str> {
    format!("currency-wars/node/{}", node.id.get()).into_boxed_str()
}

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

fn error(message: &str) -> CurrencyWarsRuntimeError {
    CurrencyWarsRuntimeError {
        message: message.into(),
    }
}

fn debug_error(value: impl std::fmt::Debug) -> CurrencyWarsRuntimeError {
    CurrencyWarsRuntimeError {
        message: format!("{value:?}").into_boxed_str(),
    }
}
