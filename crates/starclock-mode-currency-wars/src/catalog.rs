use std::collections::BTreeSet;
use std::num::{NonZeroU32, NonZeroU64};

use starclock_combat::EncounterId;

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
    pub node_template_id: u32,
    pub encounter: EncounterId,
    pub penalty_bonus_rule_id: Option<u32>,
    pub basic_gold_reward: Option<u32>,
    pub next: Option<CurrencyWarsNodeId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRoute {
    pub id: CurrencyWarsRouteId,
    pub stable_key: Box<str>,
    pub nodes: Box<[CurrencyWarsNode]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsDifficulty {
    pub source_id: u32,
    pub stable_key: Box<str>,
    pub season_id: u16,
    pub progress: u16,
    pub standard_score_rule: u32,
    pub overclock_score_rule: u32,
    pub enemy_scaling_refs: Box<[Box<str>]>,
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsOfferLevel {
    pub level: u8,
    pub candidates: Box<[CurrencyWarsRoleId]>,
    pub rarity_weights: [u32; 5],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsPriceRule {
    pub rarity: u8,
    pub buy_by_star: Box<[u32]>,
    pub sell_by_star: Box<[u32]>,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsTeamLevel {
    pub level: u8,
    pub field_cap: u8,
    pub bench_cap: u8,
    pub experience_to_next: Option<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsStarRule {
    pub role: CurrencyWarsRoleId,
    pub input_star: u8,
    pub required_copies: u8,
    pub output_star: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBondLevel {
    pub level: u8,
    pub threshold: u8,
    pub effect_ids: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBond {
    pub id: CurrencyWarsBondId,
    pub stable_key: Box<str>,
    pub members: Box<[CurrencyWarsRoleId]>,
    pub levels: Box<[CurrencyWarsBondLevel]>,
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
    routes: Box<[CurrencyWarsRoute]>,
    difficulties: Box<[CurrencyWarsDifficulty]>,
    roles: Box<[CurrencyWarsRole]>,
    offers: Box<[CurrencyWarsOfferLevel]>,
    prices: Box<[CurrencyWarsPriceRule]>,
    team_levels: Box<[CurrencyWarsTeamLevel]>,
    star_rules: Box<[CurrencyWarsStarRule]>,
    bonds: Box<[CurrencyWarsBond]>,
    investments: Box<[CurrencyWarsInvestment]>,
    policies: Box<[CurrencyWarsPolicy]>,
    initial_squad_hp: u32,
    refresh_cost: u32,
    cards_per_refresh: u8,
    direct_experience_cost: u32,
    direct_experience_gain: u32,
    standard_wave_experience: u32,
    standard_boss_experience: u32,
    overclock_wave_experience: [u32; 3],
    overclock_boss_experience: [u32; 3],
    front_cap: u8,
    back_cap: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsCatalogParts {
    pub routes: Vec<CurrencyWarsRoute>,
    pub difficulties: Vec<CurrencyWarsDifficulty>,
    pub roles: Vec<CurrencyWarsRole>,
    pub offers: Vec<CurrencyWarsOfferLevel>,
    pub prices: Vec<CurrencyWarsPriceRule>,
    pub team_levels: Vec<CurrencyWarsTeamLevel>,
    pub star_rules: Vec<CurrencyWarsStarRule>,
    pub bonds: Vec<CurrencyWarsBond>,
    pub investments: Vec<CurrencyWarsInvestment>,
    pub policies: Vec<CurrencyWarsPolicy>,
    pub initial_squad_hp: u32,
    pub refresh_cost: u32,
    pub cards_per_refresh: u8,
    pub direct_experience_cost: u32,
    pub direct_experience_gain: u32,
    pub standard_wave_experience: u32,
    pub standard_boss_experience: u32,
    pub overclock_wave_experience: [u32; 3],
    pub overclock_boss_experience: [u32; 3],
    pub front_cap: u8,
    pub back_cap: u8,
}

impl CurrencyWarsCatalog {
    pub fn new(mut parts: CurrencyWarsCatalogParts) -> Result<Self, CurrencyWarsCatalogError> {
        parts.routes.sort_by_key(|value| value.id);
        parts.difficulties.sort_by_key(|value| value.source_id);
        parts.roles.sort_by_key(|value| value.id);
        parts.offers.sort_by_key(|value| value.level);
        parts.prices.sort_by_key(|value| value.rarity);
        parts.team_levels.sort_by_key(|value| value.level);
        parts
            .star_rules
            .sort_by_key(|value| (value.role, value.input_star));
        parts.bonds.sort_by_key(|value| value.id);
        parts.investments.sort_by_key(|value| value.id);
        parts.policies.sort_by(|left, right| left.id.cmp(&right.id));
        validate(&parts)?;
        Ok(Self {
            routes: parts.routes.into_boxed_slice(),
            difficulties: parts.difficulties.into_boxed_slice(),
            roles: parts.roles.into_boxed_slice(),
            offers: parts.offers.into_boxed_slice(),
            prices: parts.prices.into_boxed_slice(),
            team_levels: parts.team_levels.into_boxed_slice(),
            star_rules: parts.star_rules.into_boxed_slice(),
            bonds: parts.bonds.into_boxed_slice(),
            investments: parts.investments.into_boxed_slice(),
            policies: parts.policies.into_boxed_slice(),
            initial_squad_hp: parts.initial_squad_hp,
            refresh_cost: parts.refresh_cost,
            cards_per_refresh: parts.cards_per_refresh,
            direct_experience_cost: parts.direct_experience_cost,
            direct_experience_gain: parts.direct_experience_gain,
            standard_wave_experience: parts.standard_wave_experience,
            standard_boss_experience: parts.standard_boss_experience,
            overclock_wave_experience: parts.overclock_wave_experience,
            overclock_boss_experience: parts.overclock_boss_experience,
            front_cap: parts.front_cap,
            back_cap: parts.back_cap,
        })
    }

    #[must_use]
    pub fn routes(&self) -> &[CurrencyWarsRoute] {
        &self.routes
    }
    #[must_use]
    pub fn difficulties(&self) -> &[CurrencyWarsDifficulty] {
        &self.difficulties
    }
    #[must_use]
    pub fn roles(&self) -> &[CurrencyWarsRole] {
        &self.roles
    }
    #[must_use]
    pub fn offers(&self) -> &[CurrencyWarsOfferLevel] {
        &self.offers
    }
    #[must_use]
    pub fn prices(&self) -> &[CurrencyWarsPriceRule] {
        &self.prices
    }
    #[must_use]
    pub fn team_levels(&self) -> &[CurrencyWarsTeamLevel] {
        &self.team_levels
    }
    #[must_use]
    pub fn star_rules(&self) -> &[CurrencyWarsStarRule] {
        &self.star_rules
    }
    #[must_use]
    pub fn bonds(&self) -> &[CurrencyWarsBond] {
        &self.bonds
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
        self.initial_squad_hp
    }
    #[must_use]
    pub const fn refresh_cost(&self) -> u32 {
        self.refresh_cost
    }
    #[must_use]
    pub const fn cards_per_refresh(&self) -> u8 {
        self.cards_per_refresh
    }
    #[must_use]
    pub const fn direct_experience_cost(&self) -> u32 {
        self.direct_experience_cost
    }
    #[must_use]
    pub const fn direct_experience_gain(&self) -> u32 {
        self.direct_experience_gain
    }
    #[must_use]
    pub const fn front_cap(&self) -> u8 {
        self.front_cap
    }
    #[must_use]
    pub const fn back_cap(&self) -> u8 {
        self.back_cap
    }

    #[must_use]
    pub fn route(&self, id: CurrencyWarsRouteId) -> Option<&CurrencyWarsRoute> {
        self.routes
            .binary_search_by_key(&id, |value| value.id)
            .ok()
            .map(|index| &self.routes[index])
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
        self.offers
            .binary_search_by_key(&level, |value| value.level)
            .ok()
            .map(|index| &self.offers[index])
    }

    #[must_use]
    pub fn team_level(&self, level: u8) -> Option<CurrencyWarsTeamLevel> {
        self.team_levels
            .binary_search_by_key(&level, |value| value.level)
            .ok()
            .map(|index| self.team_levels[index])
    }

    #[must_use]
    pub fn price(&self, rarity: u8) -> Option<&CurrencyWarsPriceRule> {
        self.prices
            .binary_search_by_key(&rarity, |value| value.rarity)
            .ok()
            .map(|index| &self.prices[index])
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
        match (gambit, node.kind) {
            (CurrencyWarsGambit::Standard, CurrencyWarsNodeKind::Boss) => {
                self.standard_boss_experience
            }
            (CurrencyWarsGambit::Standard, _) => self.standard_wave_experience,
            (CurrencyWarsGambit::Overclock, CurrencyWarsNodeKind::Boss) => {
                self.overclock_boss_experience[usize::from(node.plane.saturating_sub(1).min(2))]
            }
            (CurrencyWarsGambit::Overclock, _) => {
                self.overclock_wave_experience[usize::from(node.plane.saturating_sub(1).min(2))]
            }
        }
    }
}

fn validate(parts: &CurrencyWarsCatalogParts) -> Result<(), CurrencyWarsCatalogError> {
    if parts.routes.is_empty()
        || parts.roles.is_empty()
        || parts.offers.len() != 10
        || parts.prices.len() != 5
        || parts.team_levels.len() != 10
        || parts.initial_squad_hp == 0
        || parts.cards_per_refresh == 0
        || parts.front_cap == 0
        || parts.back_cap == 0
    {
        return Err(error("Currency Wars catalog denominators are invalid"));
    }
    unique(parts.routes.iter().map(|value| value.id), "duplicate route")?;
    unique(parts.roles.iter().map(|value| value.id), "duplicate role")?;
    unique(
        parts.investments.iter().map(|value| value.id),
        "duplicate investment",
    )?;
    unique(
        parts
            .routes
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
        .routes
        .iter()
        .flat_map(|route| route.nodes.iter().map(|node| node.id))
        .collect::<BTreeSet<_>>();
    for route in &parts.routes {
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
    }
    for offer in &parts.offers {
        if offer.candidates.is_empty()
            || offer.candidates.iter().any(|id| !role_ids.contains(id))
            || offer.rarity_weights.iter().all(|weight| *weight == 0)
        {
            return Err(error("Currency Wars roster offer is invalid"));
        }
    }
    for price in &parts.prices {
        if price.rarity == 0 || price.buy_by_star.len() != 4 || price.sell_by_star.len() != 4 {
            return Err(error("Currency Wars transaction price is invalid"));
        }
    }
    for (index, level) in parts.team_levels.iter().enumerate() {
        if usize::from(level.level) != index + 1
            || level.field_cap == 0
            || level.bench_cap == 0
            || (level.level < 10) != level.experience_to_next.is_some()
        {
            return Err(error("Currency Wars team level progression is invalid"));
        }
    }
    let mut rule_keys = BTreeSet::new();
    for rule in &parts.star_rules {
        if !role_ids.contains(&rule.role)
            || rule.required_copies < 2
            || rule.output_star != rule.input_star.saturating_add(1)
            || !rule_keys.insert((rule.role, rule.input_star))
        {
            return Err(error("Currency Wars star combination rule is invalid"));
        }
    }
    for bond in &parts.bonds {
        if bond.levels.is_empty()
            || !strictly_increasing(bond.levels.iter().map(|level| level.threshold))
        {
            return Err(error("Currency Wars bond definition is invalid"));
        }
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
pub(crate) mod tests_support {
    use super::*;

    pub fn catalog() -> CurrencyWarsCatalog {
        let role = CurrencyWarsRoleId::new(1001).unwrap();
        let node_1 = CurrencyWarsNodeId::new(1).unwrap();
        let node_2 = CurrencyWarsNodeId::new(2).unwrap();
        CurrencyWarsCatalog::new(CurrencyWarsCatalogParts {
            routes: vec![CurrencyWarsRoute {
                id: CurrencyWarsRouteId::new(100).unwrap(),
                stable_key: "route.100".into(),
                nodes: vec![
                    CurrencyWarsNode {
                        id: node_1,
                        stable_key: "node.1".into(),
                        plane: 1,
                        ordinal: 1,
                        kind: CurrencyWarsNodeKind::Monster,
                        node_template_id: 10011,
                        encounter: EncounterId::new(70000001).unwrap(),
                        penalty_bonus_rule_id: Some(90301),
                        basic_gold_reward: Some(3),
                        next: Some(node_2),
                    },
                    CurrencyWarsNode {
                        id: node_2,
                        stable_key: "node.2".into(),
                        plane: 1,
                        ordinal: 2,
                        kind: CurrencyWarsNodeKind::Supply,
                        node_template_id: 10012,
                        encounter: EncounterId::new(70000002).unwrap(),
                        penalty_bonus_rule_id: None,
                        basic_gold_reward: None,
                        next: None,
                    },
                ]
                .into_boxed_slice(),
            }],
            difficulties: vec![CurrencyWarsDifficulty {
                source_id: 1,
                stable_key: "difficulty.1".into(),
                season_id: 1,
                progress: 1,
                standard_score_rule: 1,
                overclock_score_rule: 2,
                enemy_scaling_refs: Box::new([]),
            }],
            roles: vec![CurrencyWarsRole {
                id: role,
                stable_key: "role.1001".into(),
                avatar_id: 1001,
                rarity: 1,
                build_mapping_id: "build.1001".into(),
                maximum_star: 3,
                positions: Box::new([CurrencyWarsPositionKind::Front]),
            }],
            offers: (1..=10)
                .map(|level| CurrencyWarsOfferLevel {
                    level,
                    candidates: Box::new([role]),
                    rarity_weights: [100, 0, 0, 0, 0],
                })
                .collect(),
            prices: (1..=5)
                .map(|rarity| CurrencyWarsPriceRule {
                    rarity,
                    buy_by_star: vec![u32::from(rarity), 3, 9, 27].into_boxed_slice(),
                    sell_by_star: vec![u32::from(rarity), 3, 9, 27].into_boxed_slice(),
                })
                .collect(),
            team_levels: (1..=10)
                .map(|level| CurrencyWarsTeamLevel {
                    level,
                    field_cap: level,
                    bench_cap: 9,
                    experience_to_next: (level < 10).then_some(2),
                })
                .collect(),
            star_rules: vec![
                CurrencyWarsStarRule {
                    role,
                    input_star: 1,
                    required_copies: 3,
                    output_star: 2,
                },
                CurrencyWarsStarRule {
                    role,
                    input_star: 2,
                    required_copies: 3,
                    output_star: 3,
                },
            ],
            bonds: vec![CurrencyWarsBond {
                id: CurrencyWarsBondId::new(1).unwrap(),
                stable_key: "bond.1".into(),
                members: Box::new([role]),
                levels: vec![CurrencyWarsBondLevel {
                    level: 1,
                    threshold: 1,
                    effect_ids: Box::new([]),
                }]
                .into_boxed_slice(),
            }],
            investments: vec![CurrencyWarsInvestment {
                id: CurrencyWarsInvestmentId::new(1).unwrap(),
                stable_key: "augment.1".into(),
                kind: CurrencyWarsInvestmentKind::Augment,
                effect_ids: Box::new([]),
                runtime_binding_exact: false,
            }],
            policies: vec![],
            initial_squad_hp: 100,
            refresh_cost: 2,
            cards_per_refresh: 1,
            direct_experience_cost: 4,
            direct_experience_gain: 4,
            standard_wave_experience: 2,
            standard_boss_experience: 10,
            overclock_wave_experience: [6, 8, 12],
            overclock_boss_experience: [10, 12, 16],
            front_cap: 4,
            back_cap: 9,
        })
        .unwrap()
    }
}
