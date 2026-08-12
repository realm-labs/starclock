use std::collections::{BTreeMap, BTreeSet};

use crate::{
    CurrencyWarsCatalog, CurrencyWarsPositionKind, CurrencyWarsRoleId, CurrencyWarsStarRule,
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
            combine(&mut states, *rule)?;
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
    rule: CurrencyWarsStarRule,
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
        let positions = values.into_iter().collect::<BTreeMap<_, _>>();
        let deployment = Self { positions };
        deployment.validate(catalog, roster, team_level)?;
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
        position: CurrencyWarsPosition,
        state: CurrencyWarsRoleState,
    ) -> Result<Self, CurrencyWarsEconomyError> {
        let mut positions = self.positions.clone();
        positions.insert(position, state);
        let deployment = Self { positions };
        deployment.validate(catalog, roster, team_level)?;
        Ok(deployment)
    }

    pub fn undeploy(
        &self,
        catalog: &CurrencyWarsCatalog,
        roster: &CurrencyWarsRoster,
        team_level: u8,
        position: CurrencyWarsPosition,
    ) -> Result<Self, CurrencyWarsEconomyError> {
        let mut positions = self.positions.clone();
        if positions.remove(&position).is_none() {
            return Err(error("Currency Wars position is empty"));
        }
        let deployment = Self { positions };
        deployment.validate(catalog, roster, team_level)?;
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

    pub fn validate(
        &self,
        catalog: &CurrencyWarsCatalog,
        roster: &CurrencyWarsRoster,
        team_level: u8,
    ) -> Result<(), CurrencyWarsEconomyError> {
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
            let role = catalog
                .role(state.role)
                .ok_or_else(|| error("Currency Wars deployed role is missing"))?;
            if !role.positions.contains(&position.kind) {
                return Err(error(
                    "Currency Wars role cannot use the selected position kind",
                ));
            }
            let cap = match position.kind {
                CurrencyWarsPositionKind::Front => catalog.front_cap(),
                CurrencyWarsPositionKind::Back => catalog.back_cap(),
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
        if roster.total_units().saturating_sub(deployed) > u32::from(level.bench_cap) {
            return Err(error("Currency Wars bench exceeds the authored cap"));
        }
        Ok(())
    }

    #[must_use]
    pub fn bond_levels(&self, catalog: &CurrencyWarsCatalog) -> Box<[(u64, i64)]> {
        let deployed_roles = self
            .positions
            .values()
            .map(|state| state.role)
            .collect::<BTreeSet<_>>();
        catalog
            .bonds()
            .iter()
            .filter_map(|bond| {
                let count = bond
                    .members
                    .iter()
                    .filter(|role| deployed_roles.contains(role))
                    .count();
                bond.levels
                    .iter()
                    .filter(|level| usize::from(level.threshold) <= count)
                    .max_by_key(|level| level.threshold)
                    .map(|level| (u64::from(bond.id.get()), i64::from(level.level)))
            })
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
    use crate::{CurrencyWarsRoleId, catalog::tests_support};

    use super::{CurrencyWarsRoleState, CurrencyWarsRoster, advance_team_level};

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
    fn experience_consumes_each_authored_threshold() {
        let catalog = tests_support::catalog();
        assert_eq!(advance_team_level(&catalog, 1, 5).unwrap(), (3, 1));
    }
}
