use std::collections::BTreeSet;

use crate::{
    CurrencyWarsDecimal, CurrencyWarsGambit, CurrencyWarsInvestmentId, CurrencyWarsRoleId,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsMazeBuff {
    pub source_id: u32,
    pub stable_key: Box<str>,
    pub series: u16,
    pub level: u8,
    pub maximum_level: u8,
    pub binding_type: Box<str>,
    pub binding_key: Box<str>,
    pub maze_buff_type: Box<str>,
    pub parameters: Box<[CurrencyWarsDecimal]>,
    pub modifier: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsPortalRemark {
    pub en: Box<str>,
    pub zh_cn: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsPortalDefinition {
    pub investment: CurrencyWarsInvestmentId,
    pub source_id: u32,
    pub stable_key: Box<str>,
    pub season_ids: Box<[u16]>,
    pub config_path: Box<str>,
    pub effect_ids: Box<[Box<str>]>,
    pub bonus_ids: Box<[u32]>,
    pub overclock_effective: bool,
    pub in_index: bool,
    pub delayed_bonus_ids: Box<[u32]>,
    pub effect_parameters: Box<[CurrencyWarsDecimal]>,
    pub npc_ids: Box<[u32]>,
    pub remark: Option<CurrencyWarsPortalRemark>,
    pub banned_module_ids: Box<[u32]>,
    pub maze_buffs: Box<[CurrencyWarsMazeBuff]>,
}

impl CurrencyWarsPortalDefinition {
    #[must_use]
    pub fn eligible(&self, season: u16, gambit: CurrencyWarsGambit, module: u32) -> bool {
        self.season_ids.binary_search(&season).is_ok()
            && (gambit == CurrencyWarsGambit::Standard || self.overclock_effective)
            && self.banned_module_ids.binary_search(&module).is_err()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsOrbType {
    White,
    Blue,
    Gold,
    Colorful,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsOrbDisplay {
    pub orb_type: CurrencyWarsOrbType,
    pub icon_path: Box<str>,
    pub prefab_path: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsOrbDefinition {
    pub investment: CurrencyWarsInvestmentId,
    pub source_id: Box<str>,
    pub stable_key: Box<str>,
    pub bonus_id: u32,
    pub orb_type: CurrencyWarsOrbType,
    pub effect_ids: Box<[Box<str>]>,
    pub display: CurrencyWarsOrbDisplay,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsProjectionDefinition {
    pub investment: CurrencyWarsInvestmentId,
    pub source_id: u32,
    pub stable_key: Box<str>,
    pub role: CurrencyWarsRoleId,
    pub unlock_type: Box<str>,
    pub trait_ids: Box<[u32]>,
    pub effect_ids: Box<[Box<str>]>,
    pub maze_buffs: Box<[CurrencyWarsMazeBuff]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsTalentKind {
    Permanent,
    Season,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsTalentDefinition {
    pub investment: Option<CurrencyWarsInvestmentId>,
    pub source_id: u32,
    pub stable_key: Box<str>,
    pub kind: CurrencyWarsTalentKind,
    pub season_id: Option<u16>,
    pub cost: u32,
    pub prerequisites: Box<[u32]>,
    pub successors: Box<[u32]>,
    pub effect_ids: Box<[Box<str>]>,
    pub config_path: Box<str>,
    pub maze_buffs: Box<[CurrencyWarsMazeBuff]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyWarsTypedInvestment {
    Portal(CurrencyWarsPortalDefinition),
    Orb(CurrencyWarsOrbDefinition),
    Projection(CurrencyWarsProjectionDefinition),
    Talent(CurrencyWarsTalentDefinition),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsCrossInvestmentCatalog {
    portals: Box<[CurrencyWarsPortalDefinition]>,
    orbs: Box<[CurrencyWarsOrbDefinition]>,
    projections: Box<[CurrencyWarsProjectionDefinition]>,
    talents: Box<[CurrencyWarsTalentDefinition]>,
    maze_buffs: Box<[CurrencyWarsMazeBuff]>,
}

impl CurrencyWarsCrossInvestmentCatalog {
    pub fn new(
        mut portals: Vec<CurrencyWarsPortalDefinition>,
        mut orbs: Vec<CurrencyWarsOrbDefinition>,
        mut projections: Vec<CurrencyWarsProjectionDefinition>,
        mut talents: Vec<CurrencyWarsTalentDefinition>,
        mut maze_buffs: Vec<CurrencyWarsMazeBuff>,
    ) -> Result<Self, CurrencyWarsCrossInvestmentCatalogError> {
        portals.sort_by_key(|value| value.investment);
        orbs.sort_by_key(|value| value.investment);
        projections.sort_by_key(|value| value.investment);
        talents.sort_by_key(|value| (value.kind, value.source_id));
        maze_buffs.sort_by_key(|value| value.source_id);
        validate(&portals, &orbs, &projections, &talents, &maze_buffs)?;
        Ok(Self {
            portals: portals.into(),
            orbs: orbs.into(),
            projections: projections.into(),
            talents: talents.into(),
            maze_buffs: maze_buffs.into(),
        })
    }

    #[must_use]
    pub fn portals(&self) -> &[CurrencyWarsPortalDefinition] {
        &self.portals
    }
    #[must_use]
    pub fn orbs(&self) -> &[CurrencyWarsOrbDefinition] {
        &self.orbs
    }
    #[must_use]
    pub fn projections(&self) -> &[CurrencyWarsProjectionDefinition] {
        &self.projections
    }
    #[must_use]
    pub fn talents(&self) -> &[CurrencyWarsTalentDefinition] {
        &self.talents
    }
    #[must_use]
    pub fn maze_buffs(&self) -> &[CurrencyWarsMazeBuff] {
        &self.maze_buffs
    }

    #[must_use]
    pub fn investment(&self, id: CurrencyWarsInvestmentId) -> Option<CurrencyWarsTypedInvestment> {
        self.portals
            .iter()
            .find(|value| value.investment == id)
            .cloned()
            .map(CurrencyWarsTypedInvestment::Portal)
            .or_else(|| {
                self.orbs
                    .iter()
                    .find(|value| value.investment == id)
                    .cloned()
                    .map(CurrencyWarsTypedInvestment::Orb)
            })
            .or_else(|| {
                self.projections
                    .iter()
                    .find(|value| value.investment == id)
                    .cloned()
                    .map(CurrencyWarsTypedInvestment::Projection)
            })
            .or_else(|| {
                self.talents
                    .iter()
                    .find(|value| value.investment == Some(id))
                    .cloned()
                    .map(CurrencyWarsTypedInvestment::Talent)
            })
    }

    #[must_use]
    pub fn talent(
        &self,
        kind: CurrencyWarsTalentKind,
        source_id: u32,
    ) -> Option<&CurrencyWarsTalentDefinition> {
        self.talents
            .iter()
            .find(|value| value.kind == kind && value.source_id == source_id)
    }
}

fn validate(
    portals: &[CurrencyWarsPortalDefinition],
    orbs: &[CurrencyWarsOrbDefinition],
    projections: &[CurrencyWarsProjectionDefinition],
    talents: &[CurrencyWarsTalentDefinition],
    buffs: &[CurrencyWarsMazeBuff],
) -> Result<(), CurrencyWarsCrossInvestmentCatalogError> {
    let ids = portals
        .iter()
        .map(|value| value.investment)
        .chain(orbs.iter().map(|value| value.investment))
        .chain(projections.iter().map(|value| value.investment))
        .chain(talents.iter().filter_map(|value| value.investment))
        .collect::<BTreeSet<_>>();
    let count = portals.len()
        + orbs.len()
        + projections.len()
        + talents
            .iter()
            .filter(|value| value.investment.is_some())
            .count();
    if ids.len() != count
        || buffs
            .iter()
            .map(|value| value.source_id)
            .collect::<BTreeSet<_>>()
            .len()
            != buffs.len()
    {
        return Err(error("Currency Wars cross-investment identity is invalid"));
    }
    for talent in talents {
        if talent
            .prerequisites
            .iter()
            .chain(talent.successors.iter())
            .any(|id| {
                talents
                    .iter()
                    .filter(|value| value.kind == talent.kind)
                    .all(|value| value.source_id != *id)
            })
        {
            return Err(error("Currency Wars Talent graph reference is missing"));
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsCrossInvestmentCatalogError {
    message: Box<str>,
}
impl std::fmt::Display for CurrencyWarsCrossInvestmentCatalogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for CurrencyWarsCrossInvestmentCatalogError {}
fn error(message: &'static str) -> CurrencyWarsCrossInvestmentCatalogError {
    CurrencyWarsCrossInvestmentCatalogError {
        message: message.into(),
    }
}
