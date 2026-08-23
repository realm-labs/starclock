//! Enemy-affix selection at the immutable run-definition boundary.

use sha2::{Digest, Sha256};
use starclock_combat::Scalar;

use crate::{
    CurrencyWarsDifficulty, CurrencyWarsEncounterCatalog, CurrencyWarsEnemyAffix,
    CurrencyWarsEnemyAffixDefinition, CurrencyWarsNodeKind,
};

pub const ENEMY_AFFIX_SELECTION_POLICY_ID: &str = "currency-wars.enemy-affix-selection-policy.v1";
pub const ENEMY_AFFIX_SELECTION_REPLACEMENT_CONDITION: &str = "Replace when released executable evidence publishes the GridFight Division/Stage candidate pool, player-choice boundary and ordering algorithm.";

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsEnemyAffixSelectionSource {
    Explicit,
    DeterministicProjectPolicy,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsEnemyAffixExecutionOwner {
    PrebattleStats,
    BattleRule,
    ActivityBoundary,
}

/// Closed semantic identity for every released Currency Wars enemy Affix.
///
/// IDs are interpreted once at this mode-owned compiler boundary. Shared
/// combat and Activity code consume the semantic value and never branch on a
/// Currency Wars content ID.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsEnemyAffixSemantic {
    BossEnhancement,
    FollowerEnhancement,
    FirstPlaneEnhancement,
    SecondPlaneEnhancement,
    ThirdPlaneEnhancement,
    HeavyFootfall,
    ExtraStrike,
    BlazingVengeance,
    CarriedByInertia,
    PreyOnTheWeak,
    FrontendShutdown,
    BackendShutdown,
    Enervation,
    ShowdownImpending,
    SatisfyingBrawl,
    CryogenicHibernation,
    LeadByExample,
    GetOutOfJailFreeCard,
    BeyondEndurance,
    SelfDefense,
    EnergyDisappearance,
    CriticalConundrum,
    PurityOfFleshAndMind,
    RapidCooling,
    SynchronizedAction,
    PermanentTrauma,
    FightOrFlightResponse,
    LostLuck,
    MagmaBombardment,
    TimeAssassin,
    CurbedWind,
    CurbedFire,
    CurbedIce,
    CurbedLightning,
    CurbedPhysical,
    CurbedQuantum,
    CurbedImaginary,
    SpeedAlternation,
    ItsATrap,
    EmergencyHemostasis,
    GrowingPains,
    TreasureToTrash,
    BadStart,
    ThickSkinned,
    BluntTheEdge,
    ConnectionsFirst,
    DifferentialTreatment,
    ExpensiveTaste,
    RustingTreasury,
    MeAlone,
    CheapTaste,
}

impl CurrencyWarsEnemyAffixSemantic {
    #[must_use]
    pub const fn execution_owner(self) -> CurrencyWarsEnemyAffixExecutionOwner {
        match self {
            Self::BossEnhancement
            | Self::FollowerEnhancement
            | Self::FirstPlaneEnhancement
            | Self::SecondPlaneEnhancement
            | Self::ThirdPlaneEnhancement => CurrencyWarsEnemyAffixExecutionOwner::PrebattleStats,
            Self::ShowdownImpending
            | Self::SatisfyingBrawl
            | Self::GrowingPains
            | Self::TreasureToTrash
            | Self::BadStart => CurrencyWarsEnemyAffixExecutionOwner::ActivityBoundary,
            _ => CurrencyWarsEnemyAffixExecutionOwner::BattleRule,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEnemyAffixBehavior {
    pub source_id: u32,
    pub semantic: CurrencyWarsEnemyAffixSemantic,
    pub maze_buff_ids: Box<[u32]>,
    pub parameters: Box<[Scalar]>,
}

impl CurrencyWarsEnemyAffixBehavior {
    /// Compiles one released Affix definition into its closed runtime semantic.
    ///
    /// Non-Affix rows, unknown source IDs and parameter shapes that do not
    /// match the released definition are rejected without producing behavior.
    pub fn compile(
        affix: &CurrencyWarsEnemyAffix,
    ) -> Result<Self, CurrencyWarsEnemyAffixSelectionError> {
        let CurrencyWarsEnemyAffixDefinition::Affix {
            source_id,
            maze_buff_ids,
            parameters,
            ..
        } = &affix.definition
        else {
            return Err(error(
                "Currency Wars selected enemy Affix is not a definition",
            ));
        };
        let (semantic, parameter_count) = semantic(*source_id)
            .ok_or_else(|| error("Currency Wars enemy Affix semantic is missing"))?;
        if parameters.len() != parameter_count {
            return Err(error("Currency Wars enemy Affix semantic shape is invalid"));
        }
        Ok(Self {
            source_id: *source_id,
            semantic,
            maze_buff_ids: maze_buff_ids.clone(),
            parameters: parameters.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEnemyAffixSelection {
    source_ids: Box<[u32]>,
    behaviors: Box<[CurrencyWarsEnemyAffixBehavior]>,
    source: CurrencyWarsEnemyAffixSelectionSource,
}

impl CurrencyWarsEnemyAffixSelection {
    #[cfg(test)]
    pub(crate) fn test_empty() -> Self {
        Self {
            source_ids: Box::new([]),
            behaviors: Box::new([]),
            source: CurrencyWarsEnemyAffixSelectionSource::Explicit,
        }
    }

    pub(crate) fn resolve(
        catalog: &CurrencyWarsEncounterCatalog,
        difficulty: &CurrencyWarsDifficulty,
        requested: &[u32],
        seed: [u8; 32],
    ) -> Result<Self, CurrencyWarsEnemyAffixSelectionError> {
        let count = difficulty
            .enemy_affix_choice_counts
            .iter()
            .try_fold(0_usize, |total, value| {
                total.checked_add(usize::from(*value))
            })
            .ok_or_else(|| error("Currency Wars enemy-affix choice count overflows"))?;
        let mut source_ids = requested.to_vec();
        source_ids.sort_unstable();
        if source_ids.windows(2).any(|pair| pair[0] == pair[1])
            || source_ids
                .iter()
                .any(|id| catalog.enemy_affix_definition(*id).is_none())
        {
            return Err(error(
                "Currency Wars explicit enemy-affix selection is invalid",
            ));
        }
        let source = if source_ids.is_empty() && count > 0 {
            let mut candidates = catalog
                .enemy_affix_definitions()
                .filter_map(|affix| match affix.definition {
                    CurrencyWarsEnemyAffixDefinition::Affix { source_id, .. } => {
                        let mut hash = Sha256::new();
                        hash.update(b"starclock.currency-wars.enemy-affix-selection.v1");
                        hash.update(seed);
                        hash.update(source_id.to_le_bytes());
                        Some((<[u8; 32]>::from(hash.finalize()), source_id))
                    }
                    CurrencyWarsEnemyAffixDefinition::MazeBuff { .. }
                    | CurrencyWarsEnemyAffixDefinition::Scaling(_) => None,
                })
                .collect::<Vec<_>>();
            candidates.sort_unstable();
            if count > candidates.len() {
                return Err(error(
                    "Currency Wars enemy-affix choice count exceeds the candidate pool",
                ));
            }
            source_ids = candidates
                .into_iter()
                .take(count)
                .map(|(_, source_id)| source_id)
                .collect();
            source_ids.sort_unstable();
            CurrencyWarsEnemyAffixSelectionSource::DeterministicProjectPolicy
        } else {
            CurrencyWarsEnemyAffixSelectionSource::Explicit
        };
        if source_ids.len() != count {
            return Err(error(
                "Currency Wars enemy-affix selection count is invalid",
            ));
        }
        let behaviors = source_ids
            .iter()
            .map(|source_id| {
                catalog
                    .enemy_affix_definition(*source_id)
                    .ok_or_else(|| error("Currency Wars selected enemy Affix is missing"))
                    .and_then(CurrencyWarsEnemyAffixBehavior::compile)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            source_ids: source_ids.into_boxed_slice(),
            behaviors: behaviors.into_boxed_slice(),
            source,
        })
    }

    #[must_use]
    pub fn source_ids(&self) -> &[u32] {
        &self.source_ids
    }

    #[must_use]
    pub fn behaviors(&self) -> &[CurrencyWarsEnemyAffixBehavior] {
        &self.behaviors
    }

    pub(crate) fn action_value_adjustment(
        &self,
        node: CurrencyWarsNodeKind,
    ) -> Result<i32, CurrencyWarsEnemyAffixSelectionError> {
        self.behaviors.iter().try_fold(0_i32, |total, behavior| {
            let adjustment = match behavior.semantic {
                CurrencyWarsEnemyAffixSemantic::ShowdownImpending => match node {
                    CurrencyWarsNodeKind::Boss => -integer_parameter(behavior, 1)?,
                    CurrencyWarsNodeKind::EliteBranch => integer_parameter(behavior, 0)?,
                    _ => 0,
                },
                CurrencyWarsEnemyAffixSemantic::SatisfyingBrawl => match node {
                    CurrencyWarsNodeKind::CampMonster => -integer_parameter(behavior, 1)?,
                    CurrencyWarsNodeKind::Boss => integer_parameter(behavior, 0)?,
                    _ => 0,
                },
                _ => 0,
            };
            total.checked_add(adjustment).ok_or_else(|| {
                error("Currency Wars enemy Affix Action Value adjustment overflowed")
            })
        })
    }

    pub(crate) fn growing_pains_gold_loss(
        &self,
        team_level: u8,
    ) -> Result<u32, CurrencyWarsEnemyAffixSelectionError> {
        self.behaviors.iter().try_fold(0_u32, |total, behavior| {
            if behavior.semantic != CurrencyWarsEnemyAffixSemantic::GrowingPains
                || i32::from(team_level) < integer_parameter(behavior, 0)?
            {
                return Ok(total);
            }
            let loss = u32::try_from(integer_parameter(behavior, 1)?)
                .map_err(|_| error("Currency Wars Growing Pains loss is negative"))?;
            total
                .checked_add(loss)
                .ok_or_else(|| error("Currency Wars Growing Pains loss overflowed"))
        })
    }

    pub(crate) fn bad_start_squad_hp_loss(
        &self,
    ) -> Result<u32, CurrencyWarsEnemyAffixSelectionError> {
        self.behaviors.iter().try_fold(0_u32, |total, behavior| {
            if behavior.semantic != CurrencyWarsEnemyAffixSemantic::BadStart {
                return Ok(total);
            }
            let loss = u32::try_from(integer_parameter(behavior, 0)?)
                .map_err(|_| error("Currency Wars Bad Start loss is negative"))?;
            total
                .checked_add(loss)
                .ok_or_else(|| error("Currency Wars Bad Start loss overflowed"))
        })
    }

    pub(crate) fn treasure_to_trash_chance(&self) -> Option<Scalar> {
        self.behaviors
            .iter()
            .find(|behavior| behavior.semantic == CurrencyWarsEnemyAffixSemantic::TreasureToTrash)
            .and_then(|behavior| behavior.parameters.first().copied())
    }

    #[must_use]
    pub const fn source(&self) -> CurrencyWarsEnemyAffixSelectionSource {
        self.source
    }

    #[must_use]
    pub const fn policy_id(&self) -> Option<&'static str> {
        match self.source {
            CurrencyWarsEnemyAffixSelectionSource::Explicit => None,
            CurrencyWarsEnemyAffixSelectionSource::DeterministicProjectPolicy => {
                Some(ENEMY_AFFIX_SELECTION_POLICY_ID)
            }
        }
    }

    #[must_use]
    pub const fn replacement_condition(&self) -> Option<&'static str> {
        match self.source {
            CurrencyWarsEnemyAffixSelectionSource::Explicit => None,
            CurrencyWarsEnemyAffixSelectionSource::DeterministicProjectPolicy => {
                Some(ENEMY_AFFIX_SELECTION_REPLACEMENT_CONDITION)
            }
        }
    }
}

fn integer_parameter(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    index: usize,
) -> Result<i32, CurrencyWarsEnemyAffixSelectionError> {
    let scaled = behavior
        .parameters
        .get(index)
        .ok_or_else(|| error("Currency Wars enemy Affix parameter is missing"))?
        .scaled();
    if scaled % 1_000_000 != 0 {
        return Err(error(
            "Currency Wars enemy Affix parameter is not an integer",
        ));
    }
    i32::try_from(scaled / 1_000_000)
        .map_err(|_| error("Currency Wars enemy Affix integer parameter overflowed"))
}

const fn semantic(source_id: u32) -> Option<(CurrencyWarsEnemyAffixSemantic, usize)> {
    use CurrencyWarsEnemyAffixSemantic as Semantic;
    Some(match source_id {
        1001 => (Semantic::BossEnhancement, 2),
        1002 => (Semantic::FollowerEnhancement, 2),
        1003 => (Semantic::FirstPlaneEnhancement, 2),
        1004 => (Semantic::SecondPlaneEnhancement, 2),
        1005 => (Semantic::ThirdPlaneEnhancement, 2),
        2002 => (Semantic::HeavyFootfall, 1),
        2003 => (Semantic::ExtraStrike, 1),
        2004 => (Semantic::BlazingVengeance, 2),
        2005 => (Semantic::CarriedByInertia, 1),
        2006 => (Semantic::PreyOnTheWeak, 1),
        3001 => (Semantic::FrontendShutdown, 1),
        3002 => (Semantic::BackendShutdown, 1),
        3003 => (Semantic::Enervation, 2),
        3004 => (Semantic::ShowdownImpending, 2),
        3005 => (Semantic::SatisfyingBrawl, 2),
        3006 => (Semantic::CryogenicHibernation, 1),
        3007 => (Semantic::LeadByExample, 2),
        3008 => (Semantic::GetOutOfJailFreeCard, 2),
        4001 => (Semantic::BeyondEndurance, 2),
        4002 => (Semantic::SelfDefense, 1),
        4003 => (Semantic::EnergyDisappearance, 1),
        4005 => (Semantic::CriticalConundrum, 2),
        4006 => (Semantic::PurityOfFleshAndMind, 3),
        4007 => (Semantic::RapidCooling, 2),
        4008 => (Semantic::SynchronizedAction, 1),
        4009 => (Semantic::PermanentTrauma, 2),
        4010 => (Semantic::FightOrFlightResponse, 2),
        4011 => (Semantic::LostLuck, 1),
        4012 => (Semantic::MagmaBombardment, 3),
        4013 => (Semantic::TimeAssassin, 2),
        40140 => (Semantic::CurbedWind, 1),
        40141 => (Semantic::CurbedFire, 1),
        40142 => (Semantic::CurbedIce, 1),
        40143 => (Semantic::CurbedLightning, 1),
        40144 => (Semantic::CurbedPhysical, 1),
        40145 => (Semantic::CurbedQuantum, 1),
        40146 => (Semantic::CurbedImaginary, 1),
        4015 => (Semantic::SpeedAlternation, 2),
        4016 => (Semantic::ItsATrap, 3),
        4017 => (Semantic::EmergencyHemostasis, 3),
        4018 => (Semantic::GrowingPains, 2),
        4019 => (Semantic::TreasureToTrash, 1),
        4020 => (Semantic::BadStart, 1),
        4021 => (Semantic::ThickSkinned, 1),
        4022 => (Semantic::BluntTheEdge, 2),
        4023 => (Semantic::ConnectionsFirst, 2),
        4024 => (Semantic::DifferentialTreatment, 3),
        4025 => (Semantic::ExpensiveTaste, 2),
        4026 => (Semantic::RustingTreasury, 3),
        4027 => (Semantic::MeAlone, 3),
        4028 => (Semantic::CheapTaste, 2),
        _ => return None,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEnemyAffixSelectionError {
    message: Box<str>,
}

impl std::fmt::Display for CurrencyWarsEnemyAffixSelectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CurrencyWarsEnemyAffixSelectionError {}

fn error(message: &'static str) -> CurrencyWarsEnemyAffixSelectionError {
    CurrencyWarsEnemyAffixSelectionError {
        message: message.into(),
    }
}
