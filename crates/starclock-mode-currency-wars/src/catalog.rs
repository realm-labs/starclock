use std::collections::BTreeSet;
use std::num::{NonZeroU32, NonZeroU64};

use starclock_combat::{EncounterId, Ratio};

use crate::battle_override::CurrencyWarsBattleOverrideEnvironment;
use crate::{
    CurrencyWarsAugmentCatalog, CurrencyWarsBattleOverrideRoleBuild,
    CurrencyWarsBattleOverrideSnapshot, CurrencyWarsBlessingFormulaCatalog, CurrencyWarsBond,
    CurrencyWarsBondCatalog, CurrencyWarsBondMember, CurrencyWarsBuildCatalog,
    CurrencyWarsContentCatalog, CurrencyWarsContentReference, CurrencyWarsCrossInvestmentCatalog,
    CurrencyWarsDecimal, CurrencyWarsDeployment, CurrencyWarsEconomyCatalog,
    CurrencyWarsEmpowermentCatalog, CurrencyWarsEmpowermentCatalogError,
    CurrencyWarsEmpowermentSnapshot, CurrencyWarsEncounterCatalog, CurrencyWarsEquipmentDressRule,
    CurrencyWarsFlowCatalog, CurrencyWarsForgeTarget, CurrencyWarsMechanicProgramDisposition,
    CurrencyWarsOccurrenceCatalog, CurrencyWarsOffFieldEligibility, CurrencyWarsOfferCostRule,
    CurrencyWarsOfferFallback, CurrencyWarsProgressionCatalog, CurrencyWarsProgressionCatalogError,
    CurrencyWarsProgressionModifiers, CurrencyWarsProgressionProjection, CurrencyWarsRewardKind,
    CurrencyWarsRoleOverrideCatalog, CurrencyWarsRunPosition, CurrencyWarsServiceCatalog,
    CurrencyWarsServiceConstantValue, CurrencyWarsStarOverflowRule, CurrencyWarsStarStateOwner,
    CurrencyWarsTeamLevelTransition, CurrencyWarsTransactionChange,
    equipment_category_from_selector,
};

macro_rules! id_type_u32 {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(NonZeroU32);

        impl $name {
            #[must_use]
            pub const fn new(raw: u32) -> Option<Self> {
                match NonZeroU32::new(raw) {
                    Some(value) => Some(Self(value)),
                    None => None,
                }
            }

            #[must_use]
            pub const fn get(self) -> u32 {
                self.0.get()
            }
        }
    };
}

id_type_u32!(CurrencyWarsRouteId);
id_type_u32!(CurrencyWarsNodeId);
id_type_u32!(CurrencyWarsRoleId);
id_type_u32!(CurrencyWarsBondId);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsInvestmentId(NonZeroU64);

impl CurrencyWarsInvestmentId {
    #[must_use]
    pub const fn new(raw: u64) -> Option<Self> {
        match NonZeroU64::new(raw) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsGambit {
    Standard,
    Overclock,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsNodeKind {
    Monster,
    CampMonster,
    EliteBranch,
    Boss,
    Supply,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsPositionKind {
    Front,
    Back,
}

impl CurrencyWarsNodeKind {
    #[must_use]
    pub const fn battle(self) -> bool {
        !matches!(self, Self::Supply)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsNode {
    pub id: CurrencyWarsNodeId,
    pub stable_key: Box<str>,
    pub plane: u8,
    pub ordinal: u8,
    pub kind: CurrencyWarsNodeKind,
    pub layer_id: Box<str>,
    pub domain_composition_id: Box<str>,
    pub room_id: Box<str>,
    pub node_template_id: u32,
    pub encounter: EncounterId,
    pub parameter_ids: Box<[u32]>,
    pub penalty_bonus_rule_id: Option<u32>,
    pub basic_gold_reward: Option<u32>,
    pub next: Option<CurrencyWarsNodeId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRoute {
    pub id: CurrencyWarsRouteId,
    pub stable_key: Box<str>,
    pub map_entry_id: u32,
    pub difficulty_ids: Box<[u32]>,
    pub layer_ids: Box<[Box<str>]>,
    pub nodes: Box<[CurrencyWarsNode]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsDifficulty {
    pub source_id: u32,
    pub stable_key: Box<str>,
    pub season_id: u16,
    pub division_level: u8,
    pub progress: u16,
    pub standard_score_rule: u32,
    pub overclock_score_rule: u32,
    pub weekly_score_modifier: starclock_combat::Ratio,
    pub experience_modifier: starclock_combat::Ratio,
    pub enemy_scaling_refs: Box<[Box<str>]>,
    pub enemy_scaling: CurrencyWarsDifficultyEnemyScaling,
    pub enemy_affix_choice_counts: Box<[u8]>,
    pub binary_difficulty_rule: Option<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsDifficultyEnemyScaling {
    pub enemy_difficulty_level: u16,
    pub level_base_hp_ratio: starclock_combat::Scalar,
    pub level_base_attack_ratio: starclock_combat::Scalar,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRole {
    pub id: CurrencyWarsRoleId,
    pub stable_key: Box<str>,
    pub avatar_id: u32,
    pub rarity: u8,
    pub build_mapping_id: Box<str>,
    pub maximum_star: u8,
    pub positions: Box<[CurrencyWarsPositionKind]>,
    pub trait_ids: Box<[u32]>,
    pub backend_rank_ids: Box<[u32]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsOfferLevel {
    pub level: u8,
    pub candidates: Box<[CurrencyWarsRoleId]>,
    pub rarity_weights: [u32; 5],
    pub cost_rule: CurrencyWarsOfferCostRule,
    pub fallback: CurrencyWarsOfferFallback,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsPriceRule {
    pub stable_key: Box<str>,
    pub rarity: u8,
    pub star_levels: Box<[u8]>,
    pub buy_by_star: Box<[u32]>,
    pub sell_by_star: Box<[u32]>,
    pub ordered_changes: Box<[CurrencyWarsTransactionChange]>,
}

impl CurrencyWarsPriceRule {
    #[must_use]
    pub fn buy(&self, star: u8) -> Option<u32> {
        star.checked_sub(1)
            .and_then(|index| self.buy_by_star.get(usize::from(index)))
            .copied()
    }

    #[must_use]
    pub fn sell(&self, star: u8) -> Option<u32> {
        star.checked_sub(1)
            .and_then(|index| self.sell_by_star.get(usize::from(index)))
            .copied()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsAuthoredProperty {
    pub property: Box<str>,
    pub value: Option<CurrencyWarsDecimal>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsTeamLevel {
    pub level: u8,
    pub field_cap: u8,
    pub bench_cap: u8,
    pub experience_to_next: Option<u32>,
    pub transition: CurrencyWarsTeamLevelTransition,
    pub properties: Box<[CurrencyWarsAuthoredProperty]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsStarRule {
    pub stable_key: Box<str>,
    pub role: CurrencyWarsRoleId,
    pub input_star: u8,
    pub required_copies: u8,
    pub output_star: u8,
    pub overflow: CurrencyWarsStarOverflowRule,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsInvestmentKind {
    Augment,
    Enhancement,
    Orb,
    Portal,
    Projection,
    Talent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsInvestment {
    pub id: CurrencyWarsInvestmentId,
    pub stable_key: Box<str>,
    pub kind: CurrencyWarsInvestmentKind,
    pub effect_ids: Box<[Box<str>]>,
    pub source_id: Box<str>,
    pub references: Box<[CurrencyWarsContentReference]>,
    pub attributes_json: Box<str>,
    pub runtime_binding_exact: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsPolicy {
    pub id: Box<str>,
    pub field: Box<str>,
    pub known_facts: Box<[Box<str>]>,
    pub selected_behavior: Box<str>,
    pub alternatives: Box<[Box<str>]>,
    pub replacement_condition: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsCatalog {
    flow: CurrencyWarsFlowCatalog,
    economy: CurrencyWarsEconomyCatalog,
    build: CurrencyWarsBuildCatalog,
    empowerment: CurrencyWarsEmpowermentCatalog,
    content: CurrencyWarsContentCatalog,
    encounter: CurrencyWarsEncounterCatalog,
    roles: Box<[CurrencyWarsRole]>,
    bonds: CurrencyWarsBondCatalog,
    blessing_formula: CurrencyWarsBlessingFormulaCatalog,
    occurrences: CurrencyWarsOccurrenceCatalog,
    services: CurrencyWarsServiceCatalog,
    augments: CurrencyWarsAugmentCatalog,
    cross_investments: CurrencyWarsCrossInvestmentCatalog,
    progression: CurrencyWarsProgressionCatalog,
    progression_modifiers: [CurrencyWarsProgressionModifiers; 2],
    role_overrides: CurrencyWarsRoleOverrideCatalog,
    investments: Box<[CurrencyWarsInvestment]>,
    policies: Box<[CurrencyWarsPolicy]>,
    front_cap: u8,
    back_cap: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsCatalogParts {
    pub flow: CurrencyWarsFlowCatalog,
    pub economy: CurrencyWarsEconomyCatalog,
    pub build: CurrencyWarsBuildCatalog,
    pub empowerment: CurrencyWarsEmpowermentCatalog,
    pub content: CurrencyWarsContentCatalog,
    pub encounter: CurrencyWarsEncounterCatalog,
    pub roles: Vec<CurrencyWarsRole>,
    pub bonds: CurrencyWarsBondCatalog,
    pub blessing_formula: CurrencyWarsBlessingFormulaCatalog,
    pub occurrences: CurrencyWarsOccurrenceCatalog,
    pub services: CurrencyWarsServiceCatalog,
    pub augments: CurrencyWarsAugmentCatalog,
    pub cross_investments: CurrencyWarsCrossInvestmentCatalog,
    pub progression: CurrencyWarsProgressionCatalog,
    pub role_overrides: CurrencyWarsRoleOverrideCatalog,
    pub investments: Vec<CurrencyWarsInvestment>,
    pub policies: Vec<CurrencyWarsPolicy>,
    pub front_cap: u8,
    pub back_cap: u8,
}

impl CurrencyWarsCatalog {
    pub fn new(mut parts: CurrencyWarsCatalogParts) -> Result<Self, CurrencyWarsCatalogError> {
        parts.roles.sort_by_key(|value| value.id);
        parts.investments.sort_by_key(|value| value.id);
        parts.policies.sort_by(|left, right| left.id.cmp(&right.id));
        validate(&parts)?;
        let progression_modifiers = progression_modifiers(&parts)?;
        Ok(Self {
            flow: parts.flow,
            economy: parts.economy,
            build: parts.build,
            empowerment: parts.empowerment,
            content: parts.content,
            encounter: parts.encounter,
            roles: parts.roles.into_boxed_slice(),
            bonds: parts.bonds,
            blessing_formula: parts.blessing_formula,
            occurrences: parts.occurrences,
            services: parts.services,
            augments: parts.augments,
            cross_investments: parts.cross_investments,
            progression: parts.progression,
            progression_modifiers,
            role_overrides: parts.role_overrides,
            investments: parts.investments.into_boxed_slice(),
            policies: parts.policies.into_boxed_slice(),
            front_cap: parts.front_cap,
            back_cap: parts.back_cap,
        })
    }

    #[must_use]
    pub fn routes(&self) -> &[CurrencyWarsRoute] {
        self.flow.routes()
    }
    #[must_use]
    pub fn difficulties(&self) -> &[CurrencyWarsDifficulty] {
        self.flow.difficulties()
    }
    #[must_use]
    pub const fn flow_catalog(&self) -> &CurrencyWarsFlowCatalog {
        &self.flow
    }
    #[must_use]
    pub fn roles(&self) -> &[CurrencyWarsRole] {
        &self.roles
    }
    #[must_use]
    pub fn offers(&self) -> &[CurrencyWarsOfferLevel] {
        self.economy.offers()
    }
    #[must_use]
    pub fn prices(&self) -> &[CurrencyWarsPriceRule] {
        self.economy.prices()
    }
    #[must_use]
    pub fn team_levels(&self) -> &[CurrencyWarsTeamLevel] {
        self.economy.team_levels()
    }
    #[must_use]
    pub fn star_rules(&self) -> &[CurrencyWarsStarRule] {
        self.economy.star_rules()
    }
    #[must_use]
    pub const fn economy_catalog(&self) -> &CurrencyWarsEconomyCatalog {
        &self.economy
    }
    #[must_use]
    pub fn bonds(&self) -> &[CurrencyWarsBond] {
        self.bonds.bonds()
    }

    #[must_use]
    pub const fn build_catalog(&self) -> &CurrencyWarsBuildCatalog {
        &self.build
    }

    #[must_use]
    pub const fn empowerment_catalog(&self) -> &CurrencyWarsEmpowermentCatalog {
        &self.empowerment
    }

    #[must_use]
    pub const fn content_catalog(&self) -> &CurrencyWarsContentCatalog {
        &self.content
    }

    #[must_use]
    pub const fn encounter_catalog(&self) -> &CurrencyWarsEncounterCatalog {
        &self.encounter
    }

    #[must_use]
    pub const fn bond_catalog(&self) -> &CurrencyWarsBondCatalog {
        &self.bonds
    }
    #[must_use]
    pub const fn blessing_formula_catalog(&self) -> &CurrencyWarsBlessingFormulaCatalog {
        &self.blessing_formula
    }
    #[must_use]
    pub const fn occurrence_catalog(&self) -> &CurrencyWarsOccurrenceCatalog {
        &self.occurrences
    }
    #[must_use]
    pub const fn service_catalog(&self) -> &CurrencyWarsServiceCatalog {
        &self.services
    }
    #[must_use]
    pub const fn augment_catalog(&self) -> &CurrencyWarsAugmentCatalog {
        &self.augments
    }
    #[must_use]
    pub const fn cross_investment_catalog(&self) -> &CurrencyWarsCrossInvestmentCatalog {
        &self.cross_investments
    }
    #[must_use]
    pub const fn progression_catalog(&self) -> &CurrencyWarsProgressionCatalog {
        &self.progression
    }

    pub fn progression_projection(
        &self,
        difficulty: &CurrencyWarsDifficulty,
        gambit: CurrencyWarsGambit,
        position: CurrencyWarsRunPosition,
    ) -> Result<Option<CurrencyWarsProgressionProjection<'_>>, CurrencyWarsProgressionCatalogError>
    {
        self.progression.project(
            difficulty,
            gambit,
            position,
            self.progression_modifiers[gambit as usize],
        )
    }

    #[must_use]
    pub fn active_season(&self) -> u16 {
        let module = self.flow.profile_module_source_id();
        self.flow
            .modules()
            .iter()
            .find(|candidate| candidate.source_id == module)
            .expect("Currency Wars profile module was validated")
            .season_id
    }

    #[must_use]
    pub fn role_available(&self, role: CurrencyWarsRoleId) -> bool {
        self.progression.role_available(
            self.active_season(),
            self.flow.profile_module_source_id(),
            role,
        )
    }

    #[must_use]
    pub fn rank_role_candidates(
        &self,
        candidates: impl IntoIterator<Item = CurrencyWarsRoleId>,
    ) -> Box<[CurrencyWarsRoleId]> {
        self.progression
            .rank_role_candidates(self.active_season(), candidates)
    }
    #[must_use]
    pub const fn role_override_catalog(&self) -> &CurrencyWarsRoleOverrideCatalog {
        &self.role_overrides
    }
    #[must_use]
    pub fn investments(&self) -> &[CurrencyWarsInvestment] {
        &self.investments
    }
    #[must_use]
    pub fn policies(&self) -> &[CurrencyWarsPolicy] {
        &self.policies
    }
    #[must_use]
    pub const fn initial_squad_hp(&self) -> u32 {
        self.economy.squad_hp().initial
    }
    #[must_use]
    pub const fn refresh_cost(&self) -> u32 {
        self.economy.rules().refresh.gold_cost
    }
    #[must_use]
    pub const fn cards_per_refresh(&self) -> u8 {
        self.economy.rules().refresh.cards_per_refresh
    }
    #[must_use]
    pub const fn copies_per_role(&self, rarity: u8) -> Option<u32> {
        let Some(index) = rarity.checked_sub(1) else {
            return None;
        };
        if index >= 5 {
            return None;
        }
        Some(self.economy.rules().refresh.copies_per_role_by_rarity[index as usize])
    }
    #[must_use]
    pub const fn role_offer_initial_weight(&self) -> u32 {
        self.economy.rules().refresh.role_initial_weight
    }

    #[must_use]
    pub const fn maximum_stolen_same_card(&self, rarity: u8) -> Option<u32> {
        let index = match rarity.checked_sub(1) {
            Some(value) if value < 5 => value,
            _ => return None,
        };
        Some(
            self.economy
                .rules()
                .refresh
                .maximum_stolen_same_card_by_rarity[index as usize],
        )
    }

    #[must_use]
    pub const fn stolen_pool_refund_initial_purchase(&self) -> u32 {
        self.economy
            .rules()
            .refresh
            .stolen_pool_refund_initial_purchase
    }

    #[must_use]
    pub const fn stolen_pool_refund_sell(&self) -> u32 {
        self.economy.rules().refresh.stolen_pool_refund_sell
    }

    #[must_use]
    pub const fn stolen_pool_refund_hold(&self) -> u32 {
        self.economy.rules().refresh.stolen_pool_refund_hold
    }
    #[must_use]
    pub fn battle_interest(&self, gambit: CurrencyWarsGambit, gold: u32) -> u32 {
        let rules = &self.economy.rules().interest;
        let maximum = match gambit {
            CurrencyWarsGambit::Standard => rules.standard_maximum,
            CurrencyWarsGambit::Overclock => rules.overclock_maximum,
        };
        (gold / rules.deposit_per_interest).min(maximum)
    }
    #[must_use]
    pub const fn direct_experience_cost(&self) -> u32 {
        self.economy.rules().experience.direct_level_up_gold
    }
    #[must_use]
    pub const fn direct_experience_gain(&self) -> u32 {
        self.economy.rules().experience.direct_level_up_experience
    }
    #[must_use]
    pub const fn front_cap(&self) -> u8 {
        self.front_cap
    }
    #[must_use]
    pub const fn front_minimum(&self) -> u8 {
        self.economy.rules().team_size.front_minimum
    }
    #[must_use]
    pub const fn back_initial(&self) -> u8 {
        self.economy.rules().team_size.back_initial
    }
    #[must_use]
    pub const fn back_cap(&self) -> u8 {
        self.back_cap
    }

    #[must_use]
    pub fn route(&self, id: CurrencyWarsRouteId) -> Option<&CurrencyWarsRoute> {
        self.flow.route(id)
    }

    #[must_use]
    pub fn role(&self, id: CurrencyWarsRoleId) -> Option<&CurrencyWarsRole> {
        self.roles
            .binary_search_by_key(&id, |value| value.id)
            .ok()
            .map(|index| &self.roles[index])
    }

    #[must_use]
    pub fn offer(&self, level: u8) -> Option<&CurrencyWarsOfferLevel> {
        self.economy
            .offers()
            .binary_search_by_key(&level, |value| value.level)
            .ok()
            .map(|index| &self.economy.offers()[index])
    }

    #[must_use]
    pub fn team_level(&self, level: u8) -> Option<&CurrencyWarsTeamLevel> {
        self.economy
            .team_levels()
            .binary_search_by_key(&level, |value| value.level)
            .ok()
            .map(|index| &self.economy.team_levels()[index])
    }

    #[must_use]
    pub fn price(&self, rarity: u8) -> Option<&CurrencyWarsPriceRule> {
        self.economy
            .prices()
            .binary_search_by_key(&rarity, |value| value.rarity)
            .ok()
            .map(|index| &self.economy.prices()[index])
    }

    #[must_use]
    pub fn star_copy_count(&self, role: CurrencyWarsRoleId, star: u8) -> Option<u32> {
        let owner = CurrencyWarsStarStateOwner::Role(role);
        self.economy
            .star_states()
            .binary_search_by_key(&(owner, star), |value| (value.owner, value.star))
            .ok()
            .map(|index| self.economy.star_states()[index].copy_count)
    }

    pub fn empowerment_snapshot(
        &self,
        deployment: &CurrencyWarsDeployment,
    ) -> Result<CurrencyWarsEmpowermentSnapshot, CurrencyWarsEmpowermentCatalogError> {
        self.empowerment
            .resolve(deployment, &self.roles, self.economy.star_states())
    }

    pub fn battle_override_snapshot(
        &self,
        deployment: &CurrencyWarsDeployment,
        builds: &[CurrencyWarsBattleOverrideRoleBuild],
        battle_event_ids: &[u32],
        season_id: u16,
        lethal_rescue_action_value: starclock_combat::ActionValue,
    ) -> Result<CurrencyWarsBattleOverrideSnapshot, CurrencyWarsEmpowermentCatalogError> {
        self.empowerment.battle_override_snapshot(
            deployment,
            &self.roles,
            builds,
            battle_event_ids,
            self.economy.star_states(),
            CurrencyWarsBattleOverrideEnvironment {
                season_id,
                lethal_rescue_action_value,
            },
        )
    }

    #[must_use]
    pub fn investment(&self, id: CurrencyWarsInvestmentId) -> Option<&CurrencyWarsInvestment> {
        self.investments
            .binary_search_by_key(&id, |value| value.id)
            .ok()
            .map(|index| &self.investments[index])
    }

    #[must_use]
    pub fn experience_reward(&self, gambit: CurrencyWarsGambit, node: &CurrencyWarsNode) -> u32 {
        self.economy
            .experience_reward(gambit, node.plane, node.kind == CurrencyWarsNodeKind::Boss)
    }
}

fn validate(parts: &CurrencyWarsCatalogParts) -> Result<(), CurrencyWarsCatalogError> {
    if parts.flow.routes().is_empty()
        || parts.roles.is_empty()
        || parts.economy.offers().len() != 10
        || parts.economy.prices().len() != 5
        || parts.economy.team_levels().len() != 10
        || parts.build.mappings().len() != parts.roles.len()
        || parts.empowerment.empowerments().is_empty()
        || parts.content.records().is_empty()
        || parts.encounter.mechanic_programs.is_empty()
        || parts.bonds.bonds().is_empty()
        || parts.augments.augments().is_empty()
        || parts.economy.squad_hp().initial == 0
        || parts.economy.rules().refresh.cards_per_refresh == 0
        || parts.front_cap == 0
        || parts.back_cap == 0
        || parts.front_cap != parts.economy.rules().team_size.front_maximum
        || parts.back_cap != parts.economy.rules().team_size.back_maximum
    {
        return Err(error("Currency Wars catalog denominators are invalid"));
    }
    let special_avatar_world_level =
        required_integer_constant(&parts.services, "GridFight_SpecialAvatarWorldLevel")?;
    if parts
        .build
        .trial_builds()
        .iter()
        .any(|trial| u32::from(trial.world_level) != special_avatar_world_level)
    {
        return Err(error(
            "Currency Wars trial Build world level does not match its released constant",
        ));
    }
    progression_modifiers(parts)?;
    unique(
        parts.flow.routes().iter().map(|value| value.id),
        "duplicate route",
    )?;
    unique(parts.roles.iter().map(|value| value.id), "duplicate role")?;
    unique(
        parts.investments.iter().map(|value| value.id),
        "duplicate investment",
    )?;
    unique(
        parts
            .flow
            .routes()
            .iter()
            .flat_map(|route| route.nodes.iter().map(|node| node.id)),
        "duplicate node",
    )?;
    let role_ids = parts
        .roles
        .iter()
        .map(|role| role.id)
        .collect::<BTreeSet<_>>();
    let node_ids = parts
        .flow
        .routes()
        .iter()
        .flat_map(|route| route.nodes.iter().map(|node| node.id))
        .collect::<BTreeSet<_>>();
    for route in parts.flow.routes() {
        if route.nodes.is_empty() {
            return Err(error("Currency Wars route is empty"));
        }
        for (index, node) in route.nodes.iter().enumerate() {
            let expected = route
                .nodes
                .get(index + 1)
                .filter(|next| next.plane == node.plane)
                .map(|value| value.id);
            if node.next != expected || node.plane == 0 || node.ordinal == 0 {
                return Err(error("Currency Wars route node chain is invalid"));
            }
            if !node_ids.contains(&node.id) {
                return Err(error("Currency Wars route node identity is invalid"));
            }
        }
    }
    for role in &parts.roles {
        if !(1..=5).contains(&role.rarity)
            || role.maximum_star == 0
            || role.positions.is_empty()
            || role.positions.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(error("Currency Wars role rarity/star bound is invalid"));
        }
        let Some(mapping) = parts
            .build
            .mappings()
            .iter()
            .find(|mapping| mapping.role == role.id)
        else {
            return Err(error("Currency Wars role Build mapping is missing"));
        };
        if mapping.stable_key != role.build_mapping_id {
            return Err(error(
                "Currency Wars role Build mapping reference is invalid",
            ));
        }
        let states = parts
            .economy
            .star_states()
            .iter()
            .filter(|state| state.owner == CurrencyWarsStarStateOwner::Role(role.id))
            .collect::<Vec<_>>();
        if states.len() != usize::from(role.maximum_star)
            || states.iter().enumerate().any(|(index, state)| {
                state.star != u8::try_from(index + 1).expect("maximum star fits u8")
            })
        {
            return Err(error("Currency Wars role star-state closure is incomplete"));
        }
        let mut expected_copies = 1_u32;
        for star in 1..=role.maximum_star {
            let state = states[usize::from(star - 1)];
            if state.copy_count != expected_copies {
                return Err(error("Currency Wars role star copy count is invalid"));
            }
            if star < role.maximum_star {
                let rule = parts
                    .economy
                    .star_rules()
                    .iter()
                    .find(|rule| rule.role == role.id && rule.input_star == star)
                    .ok_or_else(|| error("Currency Wars role star rule is missing"))?;
                expected_copies = expected_copies
                    .checked_mul(u32::from(rule.required_copies))
                    .ok_or_else(|| error("Currency Wars role star copy count overflow"))?;
            }
        }
    }
    for state in parts
        .economy
        .star_states()
        .iter()
        .filter(|state| matches!(state.owner, CurrencyWarsStarStateOwner::Role(_)))
    {
        let owner = match state.owner {
            CurrencyWarsStarStateOwner::Role(role) => role,
            CurrencyWarsStarStateOwner::Servant { .. } => unreachable!(),
        };
        let role = parts
            .roles
            .iter()
            .find(|role| role.id == owner)
            .ok_or_else(|| error("Currency Wars Empowerment star role is missing"))?;
        for (position, execution, display) in [
            (
                CurrencyWarsPositionKind::Front,
                &state.front_execution_skill_ids,
                &state.front_display_skill_ids,
            ),
            (
                CurrencyWarsPositionKind::Back,
                &state.back_execution_skill_ids,
                &state.back_display_skill_ids,
            ),
        ] {
            if execution.iter().any(|skill_id| {
                !parts
                    .empowerment
                    .empowerments()
                    .iter()
                    .any(|value| value.position == position && value.skill_id == Some(*skill_id))
            }) || display.iter().any(|skill_id| {
                !parts
                    .empowerment
                    .empowerments()
                    .iter()
                    .any(|value| value.position == position && value.skill_id == Some(*skill_id))
            }) {
                return Err(error("Currency Wars Empowerment star skill is missing"));
            }
        }
        if role.positions.iter().any(|position| {
            !parts
                .empowerment
                .empowerments()
                .iter()
                .any(|value| value.avatar_id == Some(role.id.get()) && value.position == *position)
        }) {
            return Err(error("Currency Wars Empowerment role display is missing"));
        }
    }
    for conversion in parts.build.off_field_conversions() {
        match conversion.eligibility {
            CurrencyWarsOffFieldEligibility::Eidolon {
                role,
                rank_id,
                rank,
            } => {
                let definition = parts
                    .roles
                    .iter()
                    .find(|definition| definition.id == role)
                    .ok_or_else(|| error("Currency Wars off-field rank role is missing"))?;
                if rank == 0
                    || definition.backend_rank_ids.get(usize::from(rank - 1)) != Some(&rank_id)
                {
                    return Err(error(
                        "Currency Wars off-field backend rank join is invalid",
                    ));
                }
            }
            CurrencyWarsOffFieldEligibility::SignatureLightCone { role, .. } => {
                if !role_ids.contains(&role) {
                    return Err(error("Currency Wars off-field Light Cone role is missing"));
                }
            }
        }
    }
    for equipment in parts
        .build
        .equipment()
        .iter()
        .filter_map(|definition| definition.runtime.as_ref())
    {
        if let CurrencyWarsEquipmentDressRule::RoleOnly(roles) = &equipment.dress_rule
            && roles.iter().any(|role| !role_ids.contains(role))
        {
            return Err(error("Currency Wars equipment role eligibility is invalid"));
        }
    }
    for offer in parts.economy.offers() {
        if offer.candidates.is_empty()
            || offer.candidates.iter().any(|id| !role_ids.contains(id))
            || offer.rarity_weights.iter().all(|weight| *weight == 0)
        {
            return Err(error("Currency Wars roster offer is invalid"));
        }
    }
    for price in parts.economy.prices() {
        if price.rarity == 0 || price.buy_by_star.len() != 4 || price.sell_by_star.len() != 4 {
            return Err(error("Currency Wars transaction price is invalid"));
        }
    }
    for (index, level) in parts.economy.team_levels().iter().enumerate() {
        if usize::from(level.level) != index + 1
            || level.field_cap == 0
            || level.bench_cap == 0
            || (level.level < 10) != level.experience_to_next.is_some()
        {
            return Err(error("Currency Wars team level progression is invalid"));
        }
    }
    let mut rule_keys = BTreeSet::new();
    if parts.economy.star_states().iter().any(|state| {
        matches!(state.owner, CurrencyWarsStarStateOwner::Role(role) if !role_ids.contains(&role))
    }) {
        return Err(error("Currency Wars star state role is invalid"));
    }
    for rule in parts.economy.star_rules() {
        if !role_ids.contains(&rule.role)
            || rule.required_copies < 2
            || rule.output_star != rule.input_star.saturating_add(1)
            || !rule_keys.insert((rule.role, rule.input_star))
        {
            return Err(error("Currency Wars star combination rule is invalid"));
        }
    }
    for bond in parts.bonds.bonds() {
        if bond.levels.is_empty()
            || !strictly_increasing(bond.levels.iter().map(|level| level.threshold))
            || bond.members.iter().any(|member| match member {
                CurrencyWarsBondMember::RosterRole(role) => !role_ids.contains(role),
                CurrencyWarsBondMember::ExternalAuthoredRole(role) => role_ids.contains(role),
            })
        {
            return Err(error("Currency Wars bond definition is invalid"));
        }
    }
    let bond_ids = parts
        .bonds
        .bonds()
        .iter()
        .map(|bond| bond.id)
        .collect::<BTreeSet<_>>();
    if parts
        .encounter
        .mechanic_programs()
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedBondBattlePolicy(policy) => {
                Some(policy)
            }
            _ => None,
        })
        .any(|policy| policy.bond_ids.iter().any(|bond| !bond_ids.contains(bond)))
    {
        return Err(error("Currency Wars Bond battle policy binding is invalid"));
    }
    if parts
        .empowerment
        .empowerments()
        .iter()
        .filter_map(|empowerment| empowerment.avatar_id)
        .any(|avatar_id| !parts.roles.iter().any(|role| role.id.get() == avatar_id))
    {
        return Err(error("Currency Wars Empowerment avatar is invalid"));
    }
    validate_service_closure(parts, &role_ids)?;
    Ok(())
}

fn progression_modifiers(
    parts: &CurrencyWarsCatalogParts,
) -> Result<[CurrencyWarsProgressionModifiers; 2], CurrencyWarsCatalogError> {
    let percentage = |name| {
        required_integer_constant(&parts.services, name).and_then(|value| {
            i64::from(value)
                .checked_mul(1_000_000)
                .map(Ratio::from_scaled)
                .ok_or_else(|| error("Currency Wars progression percentage overflowed"))
        })
    };
    let standard = CurrencyWarsProgressionModifiers {
        weekly_score: Ratio::from_scaled(100_000_000),
        experience: Ratio::from_scaled(100_000_000),
        talent_points: Ratio::from_scaled(100_000_000),
    };
    let overclock = CurrencyWarsProgressionModifiers {
        weekly_score: percentage("GridFight_OCSeasonWeeklyScoreRatio")?,
        experience: percentage("GridFight_OCSeasonExpRatio")?,
        talent_points: percentage("GridFight_OCTalentPointRatio")?,
    };
    Ok([standard, overclock])
}

fn required_integer_constant(
    services: &CurrencyWarsServiceCatalog,
    name: &'static str,
) -> Result<u32, CurrencyWarsCatalogError> {
    match services.constant(name) {
        Some(CurrencyWarsServiceConstantValue::Integer(value)) => Ok(*value),
        Some(CurrencyWarsServiceConstantValue::IntegerArray(_)) | None => {
            Err(error("Currency Wars required integer constant is missing"))
        }
    }
}

fn validate_service_closure(
    parts: &CurrencyWarsCatalogParts,
    role_ids: &BTreeSet<CurrencyWarsRoleId>,
) -> Result<(), CurrencyWarsCatalogError> {
    let equipment = parts
        .build
        .equipment()
        .iter()
        .filter_map(|definition| definition.runtime.as_ref())
        .collect::<Vec<_>>();
    let equipment_ids = equipment
        .iter()
        .map(|definition| definition.id)
        .collect::<BTreeSet<_>>();
    for reward in parts.services.rewards() {
        let valid = match &reward.kind {
            CurrencyWarsRewardKind::DefaultCurrency
            | CurrencyWarsRewardKind::Refresh
            | CurrencyWarsRewardKind::Experience => true,
            CurrencyWarsRewardKind::Item { item, count } => {
                *count > 0 && parts.services.item(*item).is_some()
            }
            CurrencyWarsRewardKind::Orb(source_id) => {
                parts.cross_investments.orbs().iter().any(|orb| {
                    orb.source_id
                        .split('.')
                        .next()
                        .and_then(|value| value.parse::<u32>().ok())
                        == Some(*source_id)
                })
            }
            CurrencyWarsRewardKind::RandomRole { rarity, star } => {
                *star > 0
                    && parts
                        .roles
                        .iter()
                        .any(|role| role.rarity == *rarity && *star <= role.maximum_star)
            }
            CurrencyWarsRewardKind::SpecificAvatar { avatar_id, star } => {
                parts.roles.iter().any(|role| {
                    role.avatar_id == *avatar_id && *star > 0 && *star <= role.maximum_star
                })
            }
            CurrencyWarsRewardKind::RandomEquipmentByCategory(selector) => {
                equipment_category_from_selector(*selector).is_some()
            }
            CurrencyWarsRewardKind::RandomEquipmentByFunction(tag) => equipment
                .iter()
                .any(|definition| definition.tags.contains(tag)),
            CurrencyWarsRewardKind::SpecificAvatarWithEquipment {
                avatar_id,
                star,
                equipment,
            } => {
                parts.roles.iter().any(|role| {
                    role.avatar_id == *avatar_id && *star > 0 && *star <= role.maximum_star
                }) && !equipment.is_empty()
                    && equipment.iter().all(|id| equipment_ids.contains(id))
            }
            CurrencyWarsRewardKind::SpecificAvatarWithRandomEquipment {
                avatar_id,
                star,
                category_selector,
                count,
            } => {
                parts.roles.iter().any(|role| {
                    role.avatar_id == *avatar_id && *star > 0 && *star <= role.maximum_star
                }) && *count > 0
                    && equipment_category_from_selector(*category_selector).is_some()
            }
        };
        if !valid {
            return Err(error("Currency Wars reward execution closure is invalid"));
        }
    }
    if parts.services.recipes().iter().any(|recipe| {
        !equipment_ids.contains(&recipe.output)
            || recipe.inputs.iter().any(|id| !equipment_ids.contains(id))
    }) || parts.services.upgrades().iter().any(|upgrade| {
        !equipment_ids.contains(&upgrade.source) || !equipment_ids.contains(&upgrade.output)
    }) || parts.services.forge_services().iter().any(|service| {
        parts.services.item(service.item).is_none()
            || match service.target {
                CurrencyWarsForgeTarget::Equipment => {
                    equipment
                        .iter()
                        .filter(|definition| definition.category == service.category)
                        .count()
                        < usize::from(service.offer_count)
                }
                CurrencyWarsForgeTarget::Role { rarity, star } => {
                    star == 0
                        || parts
                            .roles
                            .iter()
                            .filter(|role| role.rarity == rarity && star <= role.maximum_star)
                            .count()
                            < usize::from(service.offer_count)
                }
                CurrencyWarsForgeTarget::Expert { minimum, maximum } => {
                    minimum == 0
                        || minimum > maximum
                        || parts
                            .roles
                            .iter()
                            .filter(|role| role.rarity >= minimum && role.rarity <= maximum)
                            .count()
                            < usize::from(service.offer_count)
                }
            }
    }) {
        return Err(error("Currency Wars service execution closure is invalid"));
    }
    if parts
        .build
        .recommendations()
        .iter()
        .flat_map(|recommendation| recommendation.roles.iter())
        .any(|role| !role_ids.contains(role))
    {
        return Err(error(
            "Currency Wars recommendation role closure is invalid",
        ));
    }
    Ok(())
}

fn unique<T: Ord>(
    values: impl Iterator<Item = T>,
    message: &'static str,
) -> Result<(), CurrencyWarsCatalogError> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err(error(message));
        }
    }
    Ok(())
}

fn strictly_increasing(values: impl Iterator<Item = u8>) -> bool {
    let mut previous = None;
    for value in values {
        if previous.is_some_and(|old| value <= old) {
            return false;
        }
        previous = Some(value);
    }
    true
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsCatalogError {
    message: Box<str>,
}

impl std::fmt::Display for CurrencyWarsCatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CurrencyWarsCatalogError {}

fn error(message: &'static str) -> CurrencyWarsCatalogError {
    CurrencyWarsCatalogError {
        message: message.into(),
    }
}

#[cfg(test)]
#[path = "catalog_test_support.rs"]
pub(crate) mod tests_support;
