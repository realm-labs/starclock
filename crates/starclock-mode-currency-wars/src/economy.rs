use std::collections::BTreeMap;

use crate::{
    CurrencyWarsBondResolutionContext, CurrencyWarsCatalog, CurrencyWarsEquipmentLoadout,
    CurrencyWarsPositionKind, CurrencyWarsRoleId, CurrencyWarsStarRule,
};

const ROLE_STATE_RADIX: u64 = 16;
const BACK_POSITION_OFFSET: u64 = 100;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsRoleState {
    role: CurrencyWarsRoleId,
    star: u8,
}

impl CurrencyWarsRoleState {
    pub fn new(role: CurrencyWarsRoleId, star: u8) -> Result<Self, CurrencyWarsEconomyError> {
        if star == 0 || star >= u8::try_from(ROLE_STATE_RADIX).expect("radix fits u8") {
            return Err(error(
                "Currency Wars role star is outside the encoded range",
            ));
        }
        Ok(Self { role, star })
    }

    #[must_use]
    pub const fn role(self) -> CurrencyWarsRoleId {
        self.role
    }
    #[must_use]
    pub const fn star(self) -> u8 {
        self.star
    }

    #[must_use]
    pub fn encode(self) -> u64 {
        u64::from(self.role.get()) * ROLE_STATE_RADIX + u64::from(self.star)
    }

    pub fn decode(raw: u64) -> Result<Self, CurrencyWarsEconomyError> {
        let role = u32::try_from(raw / ROLE_STATE_RADIX)
            .ok()
            .and_then(CurrencyWarsRoleId::new)
            .ok_or_else(|| error("Currency Wars encoded role is invalid"))?;
        let star = u8::try_from(raw % ROLE_STATE_RADIX)
            .map_err(|_| error("Currency Wars encoded star is invalid"))?;
        Self::new(role, star)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsPosition {
    kind: CurrencyWarsPositionKind,
    index: u8,
}

impl CurrencyWarsPosition {
    pub fn new(
        kind: CurrencyWarsPositionKind,
        index: u8,
    ) -> Result<Self, CurrencyWarsEconomyError> {
        if index == 0 {
            return Err(error("Currency Wars position index is zero"));
        }
        Ok(Self { kind, index })
    }

    #[must_use]
    pub const fn kind(self) -> CurrencyWarsPositionKind {
        self.kind
    }
    #[must_use]
    pub const fn index(self) -> u8 {
        self.index
    }

    #[must_use]
    pub const fn encode(self) -> u64 {
        match self.kind {
            CurrencyWarsPositionKind::Front => self.index as u64,
            CurrencyWarsPositionKind::Back => BACK_POSITION_OFFSET + self.index as u64,
        }
    }

    pub fn decode(raw: u64) -> Result<Self, CurrencyWarsEconomyError> {
        if raw > BACK_POSITION_OFFSET {
            Self::new(
                CurrencyWarsPositionKind::Back,
                u8::try_from(raw - BACK_POSITION_OFFSET)
                    .map_err(|_| error("Currency Wars back position is invalid"))?,
            )
        } else {
            Self::new(
                CurrencyWarsPositionKind::Front,
                u8::try_from(raw).map_err(|_| error("Currency Wars front position is invalid"))?,
            )
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CurrencyWarsRoster {
    states: BTreeMap<CurrencyWarsRoleState, u32>,
}

impl CurrencyWarsRoster {
    /// Builds an already-resolved roster.
    ///
    /// Counts at a star level that has an authored combination rule must be
    /// below that rule's threshold. Runtime acquisition is the boundary that
    /// performs automatic combination.
    pub fn new(
        catalog: &CurrencyWarsCatalog,
        values: impl IntoIterator<Item = (CurrencyWarsRoleState, u32)>,
    ) -> Result<Self, CurrencyWarsEconomyError> {
        let mut states = BTreeMap::new();
        for (state, count) in values {
            let role = catalog
                .role(state.role)
                .ok_or_else(|| error("Currency Wars roster role is missing"))?;
            if count == 0 || state.star > role.maximum_star || states.insert(state, count).is_some()
            {
                return Err(error("Currency Wars roster state is invalid or duplicated"));
            }
        }
        if catalog.star_rules().iter().any(|rule| {
            states
                .get(&CurrencyWarsRoleState {
                    role: rule.role,
                    star: rule.input_star,
                })
                .is_some_and(|count| *count >= u32::from(rule.required_copies))
        }) {
            return Err(error(
                "Currency Wars roster contains an unresolved star combination",
            ));
        }
        Ok(Self { states })
    }

    #[must_use]
    pub fn states(&self) -> &BTreeMap<CurrencyWarsRoleState, u32> {
        &self.states
    }

    #[must_use]
    pub fn count(&self, state: CurrencyWarsRoleState) -> u32 {
        self.states.get(&state).copied().unwrap_or_default()
    }

    #[must_use]
    pub fn total_units(&self) -> u32 {
        self.states.values().copied().sum()
    }

    #[must_use]
    pub fn owns_role(&self, role: CurrencyWarsRoleId) -> bool {
        self.states.keys().any(|state| state.role == role)
    }

    #[must_use]
    pub fn reached_maximum_star(
        &self,
        catalog: &CurrencyWarsCatalog,
        role: CurrencyWarsRoleId,
    ) -> bool {
        catalog.role(role).is_some_and(|definition| {
            self.count(CurrencyWarsRoleState {
                role,
                star: definition.maximum_star,
            }) > 0
        })
    }

    pub fn base_copy_count(
        &self,
        catalog: &CurrencyWarsCatalog,
        role: CurrencyWarsRoleId,
    ) -> Result<u32, CurrencyWarsEconomyError> {
        self.states
            .iter()
            .filter(|(state, _)| state.role == role)
            .try_fold(0_u32, |total, (state, count)| {
                let copies = catalog
                    .star_copy_count(role, state.star)
                    .ok_or_else(|| error("Currency Wars roster star state is missing"))?;
                copies
                    .checked_mul(*count)
                    .and_then(|value| total.checked_add(value))
                    .ok_or_else(|| error("Currency Wars roster base-copy count overflow"))
            })
    }

    pub fn acquire(
        &self,
        catalog: &CurrencyWarsCatalog,
        role: CurrencyWarsRoleId,
    ) -> Result<Self, CurrencyWarsEconomyError> {
        let definition = catalog
            .role(role)
            .ok_or_else(|| error("Currency Wars acquired role is missing"))?;
        let mut states = self.states.clone();
        add(&mut states, CurrencyWarsRoleState::new(role, 1)?, 1)?;
        let rules = catalog.star_rules().iter().filter(|rule| rule.role == role);
        for rule in rules {
            combine(&mut states, rule)?;
        }
        if states
            .keys()
            .any(|state| state.role == role && state.star > definition.maximum_star)
        {
            return Err(error(
                "Currency Wars synthesis exceeded authored maximum star",
            ));
        }
        Ok(Self { states })
    }

    pub fn sell(&self, state: CurrencyWarsRoleState) -> Result<Self, CurrencyWarsEconomyError> {
        let mut states = self.states.clone();
        let count = states
            .get_mut(&state)
            .ok_or_else(|| error("Currency Wars sold role state is not owned"))?;
        *count = count
            .checked_sub(1)
            .ok_or_else(|| error("Currency Wars sold role count underflow"))?;
        if *count == 0 {
            states.remove(&state);
        }
        Ok(Self { states })
    }

    #[must_use]
    pub fn encoded(&self) -> Box<[(u64, i64)]> {
        self.states
            .iter()
            .map(|(state, count)| (state.encode(), i64::from(*count)))
            .collect()
    }
}

fn combine(
    states: &mut BTreeMap<CurrencyWarsRoleState, u32>,
    rule: &CurrencyWarsStarRule,
) -> Result<(), CurrencyWarsEconomyError> {
    let input = CurrencyWarsRoleState::new(rule.role, rule.input_star)?;
    let count = states.get(&input).copied().unwrap_or_default();
    let required = u32::from(rule.required_copies);
    let outputs = count / required;
    if outputs == 0 {
        return Ok(());
    }
    let remainder = count % required;
    if remainder == 0 {
        states.remove(&input);
    } else {
        states.insert(input, remainder);
    }
    add(
        states,
        CurrencyWarsRoleState::new(rule.role, rule.output_star)?,
        outputs,
    )
}

fn add(
    states: &mut BTreeMap<CurrencyWarsRoleState, u32>,
    state: CurrencyWarsRoleState,
    count: u32,
) -> Result<(), CurrencyWarsEconomyError> {
    let current = states.get(&state).copied().unwrap_or_default();
    let updated = current
        .checked_add(count)
        .ok_or_else(|| error("Currency Wars roster count overflow"))?;
    states.insert(state, updated);
    Ok(())
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CurrencyWarsDeployment {
    positions: BTreeMap<CurrencyWarsPosition, CurrencyWarsRoleState>,
}

impl CurrencyWarsDeployment {
    pub fn new(
        catalog: &CurrencyWarsCatalog,
        roster: &CurrencyWarsRoster,
        team_level: u8,
        values: impl IntoIterator<Item = (CurrencyWarsPosition, CurrencyWarsRoleState)>,
    ) -> Result<Self, CurrencyWarsEconomyError> {
        Self::new_with_back_capacity(catalog, roster, team_level, catalog.back_initial(), values)
    }

    pub fn new_with_back_capacity(
        catalog: &CurrencyWarsCatalog,
        roster: &CurrencyWarsRoster,
        team_level: u8,
        back_capacity: u8,
        values: impl IntoIterator<Item = (CurrencyWarsPosition, CurrencyWarsRoleState)>,
    ) -> Result<Self, CurrencyWarsEconomyError> {
        let positions = values.into_iter().collect::<BTreeMap<_, _>>();
        let deployment = Self { positions };
        deployment.validate_with_back_capacity(catalog, roster, team_level, back_capacity)?;
        Ok(deployment)
    }

    #[must_use]
    pub fn positions(&self) -> &BTreeMap<CurrencyWarsPosition, CurrencyWarsRoleState> {
        &self.positions
    }

    pub fn deploy(
        &self,
        catalog: &CurrencyWarsCatalog,
        roster: &CurrencyWarsRoster,
        team_level: u8,
        back_capacity: u8,
        position: CurrencyWarsPosition,
        state: CurrencyWarsRoleState,
    ) -> Result<Self, CurrencyWarsEconomyError> {
        let mut positions = self.positions.clone();
        positions.insert(position, state);
        let deployment = Self { positions };
        deployment.validate_with_back_capacity(catalog, roster, team_level, back_capacity)?;
        Ok(deployment)
    }

    pub fn undeploy(
        &self,
        catalog: &CurrencyWarsCatalog,
        roster: &CurrencyWarsRoster,
        team_level: u8,
        back_capacity: u8,
        position: CurrencyWarsPosition,
    ) -> Result<Self, CurrencyWarsEconomyError> {
        let mut positions = self.positions.clone();
        if positions.remove(&position).is_none() {
            return Err(error("Currency Wars position is empty"));
        }
        let deployment = Self { positions };
        deployment.validate_with_back_capacity(catalog, roster, team_level, back_capacity)?;
        Ok(deployment)
    }

    pub fn relocate(
        &self,
        catalog: &CurrencyWarsCatalog,
        roster: &CurrencyWarsRoster,
        team_level: u8,
        back_capacity: u8,
        from: CurrencyWarsPosition,
        to: CurrencyWarsPosition,
    ) -> Result<Self, CurrencyWarsEconomyError> {
        if self.positions.contains_key(&to) {
            return Err(error("Currency Wars relocation destination is occupied"));
        }
        let mut positions = self.positions.clone();
        let state = positions
            .remove(&from)
            .ok_or_else(|| error("Currency Wars relocation source is empty"))?;
        positions.insert(to, state);
        let deployment = Self { positions };
        deployment.validate_with_back_capacity(catalog, roster, team_level, back_capacity)?;
        Ok(deployment)
    }

    #[must_use]
    pub fn reconcile_roster(&self, roster: &CurrencyWarsRoster) -> Self {
        let mut used = BTreeMap::<CurrencyWarsRoleState, u32>::new();
        let positions = self
            .positions
            .iter()
            .filter_map(|(position, state)| {
                let count = used.entry(*state).or_default();
                if *count < roster.count(*state) {
                    *count += 1;
                    Some((*position, *state))
                } else {
                    None
                }
            })
            .collect();
        Self { positions }
    }

    #[must_use]
    pub fn reconcile_acquisition(
        &self,
        roster: &CurrencyWarsRoster,
        acquired_role: CurrencyWarsRoleId,
    ) -> Self {
        let mut available = roster.states.clone();
        let mut positions = BTreeMap::new();
        let mut displaced = Vec::new();
        for (position, state) in &self.positions {
            if take_one(&mut available, *state) {
                positions.insert(*position, *state);
            } else {
                displaced.push((*position, *state));
            }
        }
        let mut replacements = available
            .iter()
            .filter(|(state, count)| state.role == acquired_role && **count > 0)
            .map(|(state, _)| *state)
            .collect::<Vec<_>>();
        replacements.sort_by_key(|state| std::cmp::Reverse(state.star));
        for (position, previous) in displaced {
            if previous.role != acquired_role {
                continue;
            }
            if let Some(replacement) = replacements
                .iter()
                .copied()
                .find(|state| take_one(&mut available, *state))
            {
                positions.insert(position, replacement);
            }
        }
        Self { positions }
    }

    pub fn validate(
        &self,
        catalog: &CurrencyWarsCatalog,
        roster: &CurrencyWarsRoster,
        team_level: u8,
    ) -> Result<(), CurrencyWarsEconomyError> {
        self.validate_with_back_capacity(catalog, roster, team_level, catalog.back_initial())
    }

    pub fn validate_with_back_capacity(
        &self,
        catalog: &CurrencyWarsCatalog,
        roster: &CurrencyWarsRoster,
        team_level: u8,
        back_capacity: u8,
    ) -> Result<(), CurrencyWarsEconomyError> {
        self.validate_with_capacities(
            catalog,
            roster,
            team_level,
            back_capacity,
            u16::from(catalog.economy_catalog().rules().team_size.bench_authored),
        )
    }

    /// Validates a non-shop service grant against the separately authored
    /// overflow waiting-area boundary.
    pub(crate) fn validate_service_overflow(
        &self,
        catalog: &CurrencyWarsCatalog,
        roster: &CurrencyWarsRoster,
        team_level: u8,
        back_capacity: u8,
    ) -> Result<(), CurrencyWarsEconomyError> {
        self.validate_with_capacities(
            catalog,
            roster,
            team_level,
            back_capacity,
            catalog.economy_catalog().rules().team_size.bench_overflow,
        )
    }

    fn validate_with_capacities(
        &self,
        catalog: &CurrencyWarsCatalog,
        roster: &CurrencyWarsRoster,
        team_level: u8,
        back_capacity: u8,
        bench_capacity: u16,
    ) -> Result<(), CurrencyWarsEconomyError> {
        if back_capacity < catalog.back_initial() || back_capacity > catalog.back_cap() {
            return Err(error(
                "Currency Wars back capacity is outside its authored range",
            ));
        }
        let team_size = catalog.economy_catalog().rules().team_size;
        if bench_capacity < u16::from(team_size.bench_authored)
            || bench_capacity > team_size.bench_overflow
        {
            return Err(error(
                "Currency Wars bench capacity is outside its authored range",
            ));
        }
        let level = catalog
            .team_level(team_level)
            .ok_or_else(|| error("Currency Wars team level is missing"))?;
        if self.positions.len() > usize::from(level.field_cap) {
            return Err(error(
                "Currency Wars deployed team exceeds the current level cap",
            ));
        }
        let mut used = BTreeMap::<CurrencyWarsRoleState, u32>::new();
        for (position, state) in &self.positions {
            catalog
                .role(state.role)
                .ok_or_else(|| error("Currency Wars deployed role is missing"))?;
            let cap = match position.kind {
                CurrencyWarsPositionKind::Front => catalog.front_cap(),
                CurrencyWarsPositionKind::Back => back_capacity,
            };
            if position.index > cap {
                return Err(error("Currency Wars position exceeds its authored cap"));
            }
            *used.entry(*state).or_default() += 1;
        }
        if used
            .iter()
            .any(|(state, count)| *count > roster.count(*state))
        {
            return Err(error("Currency Wars deployment uses an unowned role state"));
        }
        let deployed = u32::try_from(self.positions.len())
            .map_err(|_| error("Currency Wars deployment count overflow"))?;
        if roster.total_units().saturating_sub(deployed) > u32::from(bench_capacity) {
            return Err(error("Currency Wars bench exceeds the authored cap"));
        }
        Ok(())
    }

    pub fn position_ability_active(
        &self,
        catalog: &CurrencyWarsCatalog,
        position: CurrencyWarsPosition,
    ) -> Result<bool, CurrencyWarsEconomyError> {
        let state = self
            .positions
            .get(&position)
            .ok_or_else(|| error("Currency Wars position is empty"))?;
        let role = catalog
            .role(state.role)
            .ok_or_else(|| error("Currency Wars deployed role is missing"))?;
        Ok(role.positions.contains(&position.kind))
    }

    pub fn validate_battle_ready(
        &self,
        catalog: &CurrencyWarsCatalog,
    ) -> Result<(), CurrencyWarsEconomyError> {
        let front = self
            .positions
            .keys()
            .filter(|position| position.kind == CurrencyWarsPositionKind::Front)
            .count();
        if front < usize::from(catalog.front_minimum()) {
            return Err(error(
                "Currency Wars battle requires the authored front minimum",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn bond_levels(&self, catalog: &CurrencyWarsCatalog) -> Box<[(u64, i64)]> {
        catalog
            .bond_catalog()
            .resolve(
                self,
                &CurrencyWarsEquipmentLoadout::default(),
                &CurrencyWarsBondResolutionContext::default(),
            )
            .active_bonds
            .iter()
            .map(|bond| (u64::from(bond.id.get()), i64::from(bond.level)))
            .collect()
    }

    #[must_use]
    pub fn encoded(&self) -> Box<[(u64, i64)]> {
        self.positions
            .iter()
            .map(|(position, state)| {
                (
                    position.encode(),
                    i64::try_from(state.encode()).expect("role state fits i64"),
                )
            })
            .collect()
    }
}

fn take_one(
    available: &mut BTreeMap<CurrencyWarsRoleState, u32>,
    state: CurrencyWarsRoleState,
) -> bool {
    let Some(count) = available.get_mut(&state) else {
        return false;
    };
    if *count == 0 {
        return false;
    }
    *count -= 1;
    true
}

pub fn advance_team_level(
    catalog: &CurrencyWarsCatalog,
    mut level: u8,
    mut experience: u32,
) -> Result<(u8, u32), CurrencyWarsEconomyError> {
    loop {
        let definition = catalog
            .team_level(level)
            .ok_or_else(|| error("Currency Wars team level is missing"))?;
        let Some(required) = definition.experience_to_next else {
            return Ok((level, experience));
        };
        if experience < required {
            return Ok((level, experience));
        }
        experience -= required;
        level = level
            .checked_add(1)
            .ok_or_else(|| error("Currency Wars team level overflow"))?;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEconomyError {
    message: Box<str>,
}

impl std::fmt::Display for CurrencyWarsEconomyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CurrencyWarsEconomyError {}

fn error(message: &'static str) -> CurrencyWarsEconomyError {
    CurrencyWarsEconomyError {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{CurrencyWarsPositionKind, CurrencyWarsRoleId, catalog::tests_support};

    use super::{
        CurrencyWarsDeployment, CurrencyWarsPosition, CurrencyWarsRoleState, CurrencyWarsRoster,
        advance_team_level,
    };

    #[test]
    fn three_equal_copies_combine_in_stable_star_order() {
        let catalog = tests_support::catalog();
        let role = CurrencyWarsRoleId::new(1001).unwrap();
        let mut roster = CurrencyWarsRoster::default();
        for _ in 0..9 {
            roster = roster.acquire(&catalog, role).unwrap();
        }
        assert_eq!(
            roster.count(CurrencyWarsRoleState::new(role, 3).unwrap()),
            1
        );
        assert_eq!(roster.total_units(), 1);
    }

    #[test]
    fn resolved_rosters_reject_pending_star_combinations() {
        let catalog = tests_support::catalog();
        let role = CurrencyWarsRoleId::new(1001).unwrap();
        let state = CurrencyWarsRoleState::new(role, 1).unwrap();

        assert!(CurrencyWarsRoster::new(&catalog, [(state, 3)]).is_err());
    }

    #[test]
    fn maximum_star_overflow_stays_explicit_without_an_unauthored_state() {
        let catalog = tests_support::catalog();
        let role = CurrencyWarsRoleId::new(1001).unwrap();
        let maximum = CurrencyWarsRoleState::new(role, 3).unwrap();
        let base = CurrencyWarsRoleState::new(role, 1).unwrap();
        let mut roster = CurrencyWarsRoster::default();
        for _ in 0..10 {
            roster = roster.acquire(&catalog, role).unwrap();
        }

        assert_eq!(roster.count(maximum), 1);
        assert_eq!(roster.count(base), 1);
        assert_eq!(roster.base_copy_count(&catalog, role).unwrap(), 10);
        assert!(roster.reached_maximum_star(&catalog, role));
    }

    #[test]
    fn field_and_bench_caps_apply_after_automatic_combination() {
        let catalog = tests_support::catalog();
        let role = CurrencyWarsRoleId::new(1001).unwrap();
        let base = CurrencyWarsRoleState::new(role, 1).unwrap();
        let maximum = CurrencyWarsRoleState::new(role, 3).unwrap();
        let front = CurrencyWarsPosition::new(CurrencyWarsPositionKind::Front, 1).unwrap();

        let exact_bench = CurrencyWarsRoster::new(&catalog, [(maximum, 9)]).unwrap();
        assert!(CurrencyWarsDeployment::new(&catalog, &exact_bench, 1, []).is_ok());
        let overflow = CurrencyWarsRoster::new(&catalog, [(maximum, 10)]).unwrap();
        assert!(CurrencyWarsDeployment::new(&catalog, &overflow, 1, []).is_err());

        let exact_field = CurrencyWarsRoster::new(&catalog, [(maximum, 10)]).unwrap();
        assert!(CurrencyWarsDeployment::new(&catalog, &exact_field, 1, [(front, maximum)]).is_ok());

        let full_before_merge =
            CurrencyWarsRoster::new(&catalog, [(base, 2), (maximum, 7)]).unwrap();
        assert_eq!(full_before_merge.total_units(), 9);
        let merged = full_before_merge.acquire(&catalog, role).unwrap();
        assert_eq!(merged.total_units(), 8);
        assert!(CurrencyWarsDeployment::new(&catalog, &merged, 1, []).is_ok());
    }

    #[test]
    fn service_grants_use_the_separate_authored_overflow_boundary() {
        let catalog = tests_support::catalog();
        let role = CurrencyWarsRoleId::new(1001).unwrap();
        let maximum = CurrencyWarsRoleState::new(role, 3).unwrap();
        let deployment = CurrencyWarsDeployment::default();
        let exact_overflow = CurrencyWarsRoster::new(&catalog, [(maximum, 100)]).unwrap();
        let beyond_overflow = CurrencyWarsRoster::new(&catalog, [(maximum, 101)]).unwrap();

        assert!(
            deployment
                .validate_service_overflow(&catalog, &exact_overflow, 1, 6)
                .is_ok()
        );
        assert!(
            deployment
                .validate_service_overflow(&catalog, &beyond_overflow, 1, 6)
                .is_err()
        );
        assert!(
            deployment
                .validate_with_back_capacity(&catalog, &exact_overflow, 1, 6)
                .is_err()
        );
    }

    #[test]
    fn off_position_roles_are_legal_but_their_position_ability_is_inactive() {
        let catalog = tests_support::catalog();
        let role = CurrencyWarsRoleId::new(1001).unwrap();
        let state = CurrencyWarsRoleState::new(role, 1).unwrap();
        let roster = CurrencyWarsRoster::new(&catalog, [(state, 1)]).unwrap();
        let front = CurrencyWarsPosition::new(CurrencyWarsPositionKind::Front, 1).unwrap();
        let back_six = CurrencyWarsPosition::new(CurrencyWarsPositionKind::Back, 6).unwrap();
        let back_seven = CurrencyWarsPosition::new(CurrencyWarsPositionKind::Back, 7).unwrap();

        let preferred =
            CurrencyWarsDeployment::new(&catalog, &roster, 1, [(front, state)]).unwrap();
        assert!(preferred.position_ability_active(&catalog, front).unwrap());
        assert!(preferred.validate_battle_ready(&catalog).is_ok());

        let off_position =
            CurrencyWarsDeployment::new(&catalog, &roster, 1, [(back_six, state)]).unwrap();
        assert!(
            !off_position
                .position_ability_active(&catalog, back_six)
                .unwrap()
        );
        assert!(off_position.validate_battle_ready(&catalog).is_err());
        assert!(CurrencyWarsDeployment::new(&catalog, &roster, 1, [(back_seven, state)]).is_err());
        assert!(
            CurrencyWarsDeployment::new_with_back_capacity(
                &catalog,
                &roster,
                1,
                9,
                [(back_seven, state)],
            )
            .is_ok()
        );
    }

    #[test]
    fn experience_consumes_each_authored_threshold() {
        let catalog = tests_support::catalog();
        assert_eq!(advance_team_level(&catalog, 1, 5).unwrap(), (3, 1));
    }
}
