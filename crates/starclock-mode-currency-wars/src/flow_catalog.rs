use std::collections::{BTreeMap, BTreeSet};

use starclock_combat::{EncounterId, Ratio};

use crate::{
    CurrencyWarsDifficulty, CurrencyWarsGambit, CurrencyWarsNodeId, CurrencyWarsNodeKind,
    CurrencyWarsRankBoundary, CurrencyWarsRankProgression, CurrencyWarsRankProgressionKey,
    CurrencyWarsRoute, CurrencyWarsRouteId, CurrencyWarsSharedBattleBase,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsProfile {
    pub stable_key: Box<str>,
    pub entry_ids: Box<[Box<str>]>,
    pub module_id: Box<str>,
    pub gambits: Box<[CurrencyWarsGambit]>,
    pub initial_resource_ids: Box<[Box<str>]>,
    pub finish_condition_ids: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsModule {
    pub stable_key: Box<str>,
    pub source_id: u32,
    pub season_id: u16,
    pub sub_season_id: u16,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsEntryKind {
    GuideData,
    GuideTab,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsUnlockCondition {
    PlayerLevel(u32),
    CompleteOneStandardGambit,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsEntryRule {
    StandardDifficultyBoundedByHighestRank,
    StandardVictoryMayAdvanceAndDefeatPreservesRank,
    OverclockDifficultyBoundedByHighestStandardRank,
    OverclockCompletionPreservesRank,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEntry {
    pub stable_key: Box<str>,
    pub kind: CurrencyWarsEntryKind,
    pub module_id: Box<str>,
    pub unlocks: Box<[CurrencyWarsUnlockCondition]>,
    pub gambits: Box<[CurrencyWarsGambit]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsGambitDefinition {
    pub stable_key: Box<str>,
    pub gambit: CurrencyWarsGambit,
    pub unlocks: Box<[CurrencyWarsUnlockCondition]>,
    pub entry_rules: Box<[CurrencyWarsEntryRule]>,
    pub initial_resource_ids: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBattleStageFinish {
    pub stage_rule_id: u32,
    pub total_turns: u32,
    pub threshold_position: Ratio,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBattlePenaltyRule {
    pub source_id: u32,
    pub progress_values: Box<[u32]>,
    pub hp_progress_values: Box<[u32]>,
    pub threshold_percent: Option<u8>,
    pub threshold_fail_extra_squad_hp_loss: u32,
    pub base_squad_hp_loss: u32,
    pub progress_penalty_coefficient: u32,
    pub total_turns: u32,
    pub lethal_rescue_action_value_ratio: Ratio,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyWarsFinishRule {
    SettlementRank {
        left_inclusive: Option<u32>,
        right_inclusive: Option<u32>,
        rank_type: Option<Box<str>>,
    },
    BattleStage(CurrencyWarsBattleStageFinish),
    BattlePenalty(CurrencyWarsBattlePenaltyRule),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsFinishCondition {
    pub stable_key: Box<str>,
    pub rule: CurrencyWarsFinishRule,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsAreaGroup {
    pub stable_key: Box<str>,
    pub routes: Box<[CurrencyWarsRouteId]>,
    pub selection_policy: CurrencyWarsAreaSelectionPolicy,
    pub transition_rules: Box<[CurrencyWarsRouteTransitionRule]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsAreaSelectionPolicy {
    CompleteGridFightStageRouteClosure,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsRouteTransitionRule {
    AuthoredChapterAndSectionOrder,
    GambitMembershipUnresolved,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsLayer {
    pub stable_key: Box<str>,
    pub route: CurrencyWarsRouteId,
    pub plane: u8,
    pub nodes: Box<[CurrencyWarsNodeId]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRoom {
    pub stable_key: Box<str>,
    pub kind: CurrencyWarsNodeKind,
    pub reachability: CurrencyWarsRoomReachability,
    pub stage_refs: Box<[EncounterId]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsRoomReachability {
    DirectGridFightNodeType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsDomainComposition {
    pub stable_key: Box<str>,
    pub kind: CurrencyWarsNodeKind,
    pub room_ids: Box<[Box<str>]>,
    pub selection_policy: CurrencyWarsDomainSelectionPolicy,
    pub fallback: CurrencyWarsDomainFallback,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsDomainSelectionPolicy {
    ExactNodeType,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsDomainFallback {
    RejectUnknownNodeType,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsStateCarryRule {
    pub stable_key: Box<str>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsStateResetRule {
    pub stable_key: Box<str>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsTransitionKind {
    NextSection,
    PlaneTerminal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsStageFlow {
    pub stable_key: Box<str>,
    pub profile_id: Box<str>,
    pub nodes: Box<[CurrencyWarsNodeId]>,
    pub next: Option<Box<str>>,
    pub transition: CurrencyWarsTransitionKind,
    pub carry_rules: Box<[CurrencyWarsStateCarryRule]>,
    pub reset_rules: Box<[CurrencyWarsStateResetRule]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsFlowCatalog {
    profile: CurrencyWarsProfile,
    modules: Box<[CurrencyWarsModule]>,
    entries: Box<[CurrencyWarsEntry]>,
    gambits: Box<[CurrencyWarsGambitDefinition]>,
    finish_conditions: Box<[CurrencyWarsFinishCondition]>,
    area_group: CurrencyWarsAreaGroup,
    routes: Box<[CurrencyWarsRoute]>,
    difficulties: Box<[CurrencyWarsDifficulty]>,
    layers: Box<[CurrencyWarsLayer]>,
    rooms: Box<[CurrencyWarsRoom]>,
    domain_compositions: Box<[CurrencyWarsDomainComposition]>,
    stage_flow: Box<[CurrencyWarsStageFlow]>,
    rank_progression: Box<[CurrencyWarsRankProgression]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsFlowCatalogParts {
    pub profile: CurrencyWarsProfile,
    pub modules: Vec<CurrencyWarsModule>,
    pub entries: Vec<CurrencyWarsEntry>,
    pub gambits: Vec<CurrencyWarsGambitDefinition>,
    pub finish_conditions: Vec<CurrencyWarsFinishCondition>,
    pub area_group: CurrencyWarsAreaGroup,
    pub routes: Vec<CurrencyWarsRoute>,
    pub difficulties: Vec<CurrencyWarsDifficulty>,
    pub layers: Vec<CurrencyWarsLayer>,
    pub rooms: Vec<CurrencyWarsRoom>,
    pub domain_compositions: Vec<CurrencyWarsDomainComposition>,
    pub stage_flow: Vec<CurrencyWarsStageFlow>,
    pub rank_progression: Vec<CurrencyWarsRankProgression>,
}

impl CurrencyWarsFlowCatalog {
    pub fn new(
        mut parts: CurrencyWarsFlowCatalogParts,
    ) -> Result<Self, CurrencyWarsFlowCatalogError> {
        parts.modules.sort_by_key(|value| value.source_id);
        parts
            .entries
            .sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        parts.gambits.sort_by_key(|value| value.gambit);
        parts
            .finish_conditions
            .sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        parts.routes.sort_by_key(|value| value.id);
        parts.difficulties.sort_by_key(|value| value.source_id);
        parts.layers.sort_by_key(|value| (value.route, value.plane));
        parts.rooms.sort_by_key(|value| value.kind);
        parts.domain_compositions.sort_by_key(|value| value.kind);
        parts
            .stage_flow
            .sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        parts.rank_progression.sort_by_key(|value| value.key);
        validate(&parts)?;
        Ok(Self {
            profile: parts.profile,
            modules: parts.modules.into_boxed_slice(),
            entries: parts.entries.into_boxed_slice(),
            gambits: parts.gambits.into_boxed_slice(),
            finish_conditions: parts.finish_conditions.into_boxed_slice(),
            area_group: parts.area_group,
            routes: parts.routes.into_boxed_slice(),
            difficulties: parts.difficulties.into_boxed_slice(),
            layers: parts.layers.into_boxed_slice(),
            rooms: parts.rooms.into_boxed_slice(),
            domain_compositions: parts.domain_compositions.into_boxed_slice(),
            stage_flow: parts.stage_flow.into_boxed_slice(),
            rank_progression: parts.rank_progression.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn profile(&self) -> &CurrencyWarsProfile {
        &self.profile
    }

    #[must_use]
    pub fn modules(&self) -> &[CurrencyWarsModule] {
        &self.modules
    }

    #[must_use]
    pub fn profile_module_source_id(&self) -> u32 {
        self.modules
            .iter()
            .find(|module| module.stable_key == self.profile.module_id)
            .expect("Currency Wars profile module was validated")
            .source_id
    }

    #[must_use]
    pub fn entries(&self) -> &[CurrencyWarsEntry] {
        &self.entries
    }

    #[must_use]
    pub fn gambits(&self) -> &[CurrencyWarsGambitDefinition] {
        &self.gambits
    }

    #[must_use]
    pub fn finish_conditions(&self) -> &[CurrencyWarsFinishCondition] {
        &self.finish_conditions
    }

    #[must_use]
    pub fn penalty_rule(&self, source_id: u32) -> Option<&CurrencyWarsBattlePenaltyRule> {
        self.finish_conditions
            .iter()
            .find_map(|condition| match &condition.rule {
                CurrencyWarsFinishRule::BattlePenalty(rule) if rule.source_id == source_id => {
                    Some(rule)
                }
                _ => None,
            })
    }

    #[must_use]
    pub const fn area_group(&self) -> &CurrencyWarsAreaGroup {
        &self.area_group
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
    pub fn layers(&self) -> &[CurrencyWarsLayer] {
        &self.layers
    }

    #[must_use]
    pub fn rooms(&self) -> &[CurrencyWarsRoom] {
        &self.rooms
    }

    #[must_use]
    pub fn domain_compositions(&self) -> &[CurrencyWarsDomainComposition] {
        &self.domain_compositions
    }

    #[must_use]
    pub fn stage_flow(&self) -> &[CurrencyWarsStageFlow] {
        &self.stage_flow
    }

    #[must_use]
    pub fn rank_progression(&self) -> &[CurrencyWarsRankProgression] {
        &self.rank_progression
    }

    #[must_use]
    pub fn stage_battle_base(
        &self,
        encounter: EncounterId,
    ) -> Option<CurrencyWarsSharedBattleBase> {
        self.shared_battle_base(CurrencyWarsRankProgressionKey::StageBase(encounter))
    }

    #[must_use]
    pub fn level_battle_base(
        &self,
        plane: u8,
        section: u8,
    ) -> Option<CurrencyWarsSharedBattleBase> {
        self.shared_battle_base(CurrencyWarsRankProgressionKey::LevelBase { plane, section })
    }

    #[must_use]
    pub fn binary_difficulty_addition(&self, rule: u8, quality: u8) -> Option<u8> {
        let key = CurrencyWarsRankProgressionKey::BinaryDifficulty { rule, quality };
        let progression = self.rank_progression_entry(key)?;
        match &progression.boundary {
            CurrencyWarsRankBoundary::BinaryDifficultyAddition {
                enemy_difficulty_level_add,
            } => Some(*enemy_difficulty_level_add),
            _ => None,
        }
    }

    #[must_use]
    pub fn binary_node_perform_level(&self, rule: u32) -> Option<(u8, u8)> {
        let progression =
            self.rank_progression_entry(CurrencyWarsRankProgressionKey::BinaryNode(rule))?;
        match &progression.boundary {
            CurrencyWarsRankBoundary::BinaryNodePerformLevel {
                quality,
                perform_level,
            } => Some((*quality, *perform_level)),
            _ => None,
        }
    }

    fn shared_battle_base(
        &self,
        key: CurrencyWarsRankProgressionKey,
    ) -> Option<CurrencyWarsSharedBattleBase> {
        let progression = self.rank_progression_entry(key)?;
        match &progression.boundary {
            CurrencyWarsRankBoundary::SharedBattleBase { attack, hp } => {
                Some(CurrencyWarsSharedBattleBase {
                    attack: *attack,
                    hp: *hp,
                })
            }
            _ => None,
        }
    }

    fn rank_progression_entry(
        &self,
        key: CurrencyWarsRankProgressionKey,
    ) -> Option<&CurrencyWarsRankProgression> {
        self.rank_progression
            .binary_search_by_key(&key, |value| value.key)
            .ok()
            .map(|index| &self.rank_progression[index])
    }

    #[must_use]
    pub fn route(&self, id: CurrencyWarsRouteId) -> Option<&CurrencyWarsRoute> {
        self.routes
            .binary_search_by_key(&id, |value| value.id)
            .ok()
            .map(|index| &self.routes[index])
    }

    #[cfg(test)]
    pub(crate) fn test_fixture(
        routes: Vec<CurrencyWarsRoute>,
        difficulties: Vec<CurrencyWarsDifficulty>,
    ) -> Self {
        let route_ids = routes.iter().map(|route| route.id).collect::<Vec<_>>();
        let layers = routes
            .iter()
            .flat_map(|route| {
                (1..=3).filter_map(|plane| {
                    let nodes = route
                        .nodes
                        .iter()
                        .filter(|node| node.plane == plane)
                        .map(|node| node.id)
                        .collect::<Vec<_>>();
                    (!nodes.is_empty()).then(|| CurrencyWarsLayer {
                        stable_key: format!("layer.{}.{}", route.id.get(), plane).into(),
                        route: route.id,
                        plane,
                        nodes: nodes.into_boxed_slice(),
                    })
                })
            })
            .collect();
        let stage_flow = routes
            .iter()
            .flat_map(|route| route.nodes.iter())
            .map(|node| CurrencyWarsStageFlow {
                stable_key: format!("flow.{}", node.id.get()).into(),
                profile_id: "profile".into(),
                nodes: Box::new([node.id]),
                next: node.next.map(|next| format!("flow.{}", next.get()).into()),
                transition: if node.next.is_some() {
                    CurrencyWarsTransitionKind::NextSection
                } else {
                    CurrencyWarsTransitionKind::PlaneTerminal
                },
                carry_rules: Box::new([]),
                reset_rules: Box::new([]),
            })
            .collect();
        let rank_progression = std::iter::once(CurrencyWarsRankProgression {
            stable_key: "rank.1".into(),
            key: CurrencyWarsRankProgressionKey::Division {
                season: 1,
                level: 1,
            },
            boundary: CurrencyWarsRankBoundary::GambitDifficulty {
                maximum_standard: 1,
                maximum_overclock: 1,
                reward_quest_fields_excluded: true,
            },
            enemy_affix_ids: Box::new([]),
        })
        .chain(
            routes
                .iter()
                .flat_map(|route| route.nodes.iter())
                .map(|node| (node.plane, node.ordinal))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .map(|(plane, section)| CurrencyWarsRankProgression {
                    stable_key: format!("rank.level-base.{plane}.{section}").into(),
                    key: CurrencyWarsRankProgressionKey::LevelBase { plane, section },
                    boundary: CurrencyWarsRankBoundary::SharedBattleBase {
                        attack: 100,
                        hp: 100,
                    },
                    enemy_affix_ids: Box::new([]),
                }),
        )
        .collect();
        Self::new(CurrencyWarsFlowCatalogParts {
            profile: CurrencyWarsProfile {
                stable_key: "profile".into(),
                entry_ids: Box::new(["entry".into()]),
                module_id: "module".into(),
                gambits: Box::new([CurrencyWarsGambit::Standard, CurrencyWarsGambit::Overclock]),
                initial_resource_ids: Box::new([]),
                finish_condition_ids: (0..=5)
                    .map(|index| format!("finish.{index}").into())
                    .chain(std::iter::once("finish.penalty.90301".into()))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            },
            modules: vec![CurrencyWarsModule {
                stable_key: "module".into(),
                source_id: 1,
                season_id: 1,
                sub_season_id: 1,
            }],
            entries: vec![CurrencyWarsEntry {
                stable_key: "entry".into(),
                kind: CurrencyWarsEntryKind::GuideData,
                module_id: "module".into(),
                unlocks: Box::new([CurrencyWarsUnlockCondition::PlayerLevel(21)]),
                gambits: Box::new([CurrencyWarsGambit::Standard, CurrencyWarsGambit::Overclock]),
            }],
            gambits: vec![
                CurrencyWarsGambitDefinition {
                    stable_key: "gambit.standard".into(),
                    gambit: CurrencyWarsGambit::Standard,
                    unlocks: Box::new([]),
                    entry_rules: Box::new([
                        CurrencyWarsEntryRule::StandardDifficultyBoundedByHighestRank,
                    ]),
                    initial_resource_ids: Box::new([]),
                },
                CurrencyWarsGambitDefinition {
                    stable_key: "gambit.overclock".into(),
                    gambit: CurrencyWarsGambit::Overclock,
                    unlocks: Box::new([CurrencyWarsUnlockCondition::CompleteOneStandardGambit]),
                    entry_rules: Box::new([
                        CurrencyWarsEntryRule::OverclockDifficultyBoundedByHighestStandardRank,
                    ]),
                    initial_resource_ids: Box::new([]),
                },
            ],
            finish_conditions: vec![
                CurrencyWarsFinishCondition {
                    stable_key: "finish.0".into(),
                    rule: CurrencyWarsFinishRule::SettlementRank {
                        left_inclusive: None,
                        right_inclusive: None,
                        rank_type: None,
                    },
                },
                settlement_fixture(1, 1, 39, "B"),
                settlement_fixture(2, 40, 69, "A"),
                settlement_fixture(3, 70, 89, "S"),
                settlement_fixture(4, 90, 99, "SS"),
                settlement_fixture(5, 100, 9_999_999, "SSS"),
                CurrencyWarsFinishCondition {
                    stable_key: "finish.penalty.90301".into(),
                    rule: CurrencyWarsFinishRule::BattlePenalty(CurrencyWarsBattlePenaltyRule {
                        source_id: 90_301,
                        progress_values: Box::new([2, 3, 10, 15, 0, 0]),
                        hp_progress_values: Box::new([0, 0, 0, 0, 100, 100]),
                        threshold_percent: None,
                        threshold_fail_extra_squad_hp_loss: 0,
                        base_squad_hp_loss: 5,
                        progress_penalty_coefficient: 100,
                        total_turns: 18,
                        lethal_rescue_action_value_ratio: Ratio::from_scaled(250_000),
                    }),
                },
            ],
            area_group: CurrencyWarsAreaGroup {
                stable_key: "area-group".into(),
                routes: route_ids.into_boxed_slice(),
                selection_policy:
                    CurrencyWarsAreaSelectionPolicy::CompleteGridFightStageRouteClosure,
                transition_rules: Box::new([
                    CurrencyWarsRouteTransitionRule::AuthoredChapterAndSectionOrder,
                    CurrencyWarsRouteTransitionRule::GambitMembershipUnresolved,
                ]),
            },
            routes,
            difficulties,
            layers,
            rooms: vec![
                CurrencyWarsRoom {
                    stable_key: "room.monster".into(),
                    kind: CurrencyWarsNodeKind::Monster,
                    reachability: CurrencyWarsRoomReachability::DirectGridFightNodeType,
                    stage_refs: Box::new([]),
                },
                CurrencyWarsRoom {
                    stable_key: "room.supply".into(),
                    kind: CurrencyWarsNodeKind::Supply,
                    reachability: CurrencyWarsRoomReachability::DirectGridFightNodeType,
                    stage_refs: Box::new([]),
                },
            ],
            domain_compositions: vec![
                CurrencyWarsDomainComposition {
                    stable_key: "domain.monster".into(),
                    kind: CurrencyWarsNodeKind::Monster,
                    room_ids: Box::new(["room.monster".into()]),
                    selection_policy: CurrencyWarsDomainSelectionPolicy::ExactNodeType,
                    fallback: CurrencyWarsDomainFallback::RejectUnknownNodeType,
                },
                CurrencyWarsDomainComposition {
                    stable_key: "domain.supply".into(),
                    kind: CurrencyWarsNodeKind::Supply,
                    room_ids: Box::new(["room.supply".into()]),
                    selection_policy: CurrencyWarsDomainSelectionPolicy::ExactNodeType,
                    fallback: CurrencyWarsDomainFallback::RejectUnknownNodeType,
                },
            ],
            stage_flow,
            rank_progression,
        })
        .expect("Currency Wars test flow catalog is valid")
    }
}

#[cfg(test)]
fn settlement_fixture(index: u8, left: u32, right: u32, rank: &str) -> CurrencyWarsFinishCondition {
    CurrencyWarsFinishCondition {
        stable_key: format!("finish.{index}").into(),
        rule: CurrencyWarsFinishRule::SettlementRank {
            left_inclusive: Some(left),
            right_inclusive: Some(right),
            rank_type: Some(rank.into()),
        },
    }
}

fn validate(parts: &CurrencyWarsFlowCatalogParts) -> Result<(), CurrencyWarsFlowCatalogError> {
    unique(parts.modules.iter().map(|value| value.stable_key.as_ref()))?;
    unique(parts.entries.iter().map(|value| value.stable_key.as_ref()))?;
    unique(parts.gambits.iter().map(|value| value.gambit))?;
    unique(
        parts
            .finish_conditions
            .iter()
            .map(|value| value.stable_key.as_ref()),
    )?;
    unique(parts.routes.iter().map(|value| value.id))?;
    unique(parts.difficulties.iter().map(|value| value.source_id))?;
    unique(parts.layers.iter().map(|value| (value.route, value.plane)))?;
    unique(parts.rooms.iter().map(|value| value.stable_key.as_ref()))?;
    unique(parts.rooms.iter().map(|value| value.kind))?;
    unique(
        parts
            .domain_compositions
            .iter()
            .map(|value| value.stable_key.as_ref()),
    )?;
    unique(
        parts
            .stage_flow
            .iter()
            .map(|value| value.stable_key.as_ref()),
    )?;
    unique(parts.rank_progression.iter().map(|value| value.key))?;
    validate_finish_conditions(&parts.finish_conditions)?;

    if parts
        .modules
        .iter()
        .any(|module| module.source_id == 0 || module.season_id == 0 || module.sub_season_id == 0)
        || parts
            .gambits
            .iter()
            .any(|gambit| gambit.entry_rules.is_empty())
    {
        return Err(error(
            "Currency Wars profile/module/Gambit definition is invalid",
        ));
    }

    let module_ids = parts
        .modules
        .iter()
        .map(|value| value.stable_key.as_ref())
        .collect::<BTreeSet<_>>();
    let entry_ids = parts
        .entries
        .iter()
        .map(|value| value.stable_key.as_ref())
        .collect::<BTreeSet<_>>();
    let gambits = parts
        .gambits
        .iter()
        .map(|value| value.gambit)
        .collect::<BTreeSet<_>>();
    let finish_ids = parts
        .finish_conditions
        .iter()
        .map(|value| value.stable_key.as_ref())
        .collect::<BTreeSet<_>>();
    let route_ids = parts
        .routes
        .iter()
        .map(|value| value.id)
        .collect::<BTreeSet<_>>();
    let node_ids = parts
        .routes
        .iter()
        .flat_map(|route| route.nodes.iter().map(|node| node.id))
        .collect::<BTreeSet<_>>();
    let room_ids = parts
        .rooms
        .iter()
        .map(|value| value.stable_key.as_ref())
        .collect::<BTreeSet<_>>();
    let room_kinds = parts
        .rooms
        .iter()
        .map(|value| (value.stable_key.as_ref(), value.kind))
        .collect::<BTreeMap<_, _>>();
    let domains = parts
        .domain_compositions
        .iter()
        .map(|value| (value.stable_key.as_ref(), value))
        .collect::<BTreeMap<_, _>>();

    if !module_ids.contains(parts.profile.module_id.as_ref())
        || parts
            .profile
            .entry_ids
            .iter()
            .any(|id| !entry_ids.contains(id.as_ref()))
        || parts
            .profile
            .gambits
            .iter()
            .any(|gambit| !gambits.contains(gambit))
        || parts
            .profile
            .finish_condition_ids
            .iter()
            .any(|id| !finish_ids.contains(id.as_ref()))
    {
        return Err(error("Currency Wars profile reference is invalid"));
    }
    if parts.entries.iter().any(|entry| {
        !module_ids.contains(entry.module_id.as_ref())
            || entry.gambits.iter().any(|gambit| !gambits.contains(gambit))
    }) {
        return Err(error("Currency Wars entry reference is invalid"));
    }
    if parts
        .area_group
        .routes
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        != route_ids
    {
        return Err(error("Currency Wars area-group route closure is invalid"));
    }
    let layer_ids = parts
        .layers
        .iter()
        .map(|layer| (layer.stable_key.as_ref(), layer))
        .collect::<BTreeMap<_, _>>();
    if parts.layers.iter().any(|layer| {
        !route_ids.contains(&layer.route)
            || layer.plane == 0
            || layer.nodes.is_empty()
            || layer.nodes.iter().any(|node| !node_ids.contains(node))
    }) {
        return Err(error("Currency Wars layer reference is invalid"));
    }
    let layered_nodes = parts
        .layers
        .iter()
        .flat_map(|layer| layer.nodes.iter().copied())
        .collect::<Vec<_>>();
    if layered_nodes.len() != node_ids.len()
        || layered_nodes.iter().copied().collect::<BTreeSet<_>>() != node_ids
    {
        return Err(error("Currency Wars layer-node closure is invalid"));
    }
    for route in &parts.routes {
        let route_layers = route
            .layer_ids
            .iter()
            .map(|id| {
                layer_ids
                    .get(id.as_ref())
                    .copied()
                    .filter(|layer| layer.route == route.id)
                    .ok_or_else(|| error("Currency Wars route layer is invalid"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let expected_nodes = route_layers
            .iter()
            .flat_map(|layer| layer.nodes.iter().copied())
            .collect::<Vec<_>>();
        if expected_nodes != route.nodes.iter().map(|node| node.id).collect::<Vec<_>>() {
            return Err(error("Currency Wars route layer order is invalid"));
        }
        for node in &route.nodes {
            let layer = layer_ids
                .get(node.layer_id.as_ref())
                .ok_or_else(|| error("Currency Wars node layer is invalid"))?;
            let domain = domains
                .get(node.domain_composition_id.as_ref())
                .ok_or_else(|| error("Currency Wars node domain is invalid"))?;
            if layer.route != route.id
                || layer.plane != node.plane
                || !layer.nodes.contains(&node.id)
                || domain.kind != node.kind
                || room_kinds.get(node.room_id.as_ref()) != Some(&node.kind)
                || !domain
                    .room_ids
                    .iter()
                    .any(|room| room.as_ref() == node.room_id.as_ref())
            {
                return Err(error("Currency Wars node topology reference is invalid"));
            }
        }
    }
    if parts.domain_compositions.iter().any(|composition| {
        composition.room_ids.is_empty()
            || composition
                .room_ids
                .iter()
                .any(|room| !room_ids.contains(room.as_ref()))
            || composition
                .room_ids
                .iter()
                .any(|room| room_kinds.get(room.as_ref()) != Some(&composition.kind))
    }) {
        return Err(error("Currency Wars domain composition is invalid"));
    }
    let flow_ids = parts
        .stage_flow
        .iter()
        .map(|value| value.stable_key.as_ref())
        .collect::<BTreeSet<_>>();
    let flow_by_node = parts
        .stage_flow
        .iter()
        .flat_map(|flow| {
            flow.nodes
                .iter()
                .map(move |node| (*node, flow.stable_key.as_ref()))
        })
        .collect::<BTreeMap<_, _>>();
    if parts.stage_flow.iter().any(|flow| {
        flow.profile_id.as_ref() != parts.profile.stable_key.as_ref()
            || flow.nodes.is_empty()
            || flow.nodes.iter().any(|node| !node_ids.contains(node))
            || flow
                .next
                .as_deref()
                .is_some_and(|next| !flow_ids.contains(next))
            || (flow.next.is_some()
                != matches!(flow.transition, CurrencyWarsTransitionKind::NextSection))
    }) {
        return Err(error("Currency Wars stage-flow reference is invalid"));
    }
    if flow_by_node.len() != node_ids.len()
        || flow_by_node.keys().copied().collect::<BTreeSet<_>>() != node_ids
    {
        return Err(error("Currency Wars stage-flow node closure is invalid"));
    }
    for node in parts.routes.iter().flat_map(|route| route.nodes.iter()) {
        let flow_id = flow_by_node[&node.id];
        let flow = parts
            .stage_flow
            .iter()
            .find(|flow| flow.stable_key.as_ref() == flow_id)
            .expect("flow-by-node values come from stage flow");
        let expected_next = node.next.map(|next| flow_by_node[&next]);
        if flow.next.as_deref() != expected_next {
            return Err(error("Currency Wars stage-flow chain is invalid"));
        }
    }
    for finish in &parts.finish_conditions {
        match &finish.rule {
            CurrencyWarsFinishRule::SettlementRank {
                left_inclusive,
                right_inclusive,
                rank_type,
            } => {
                if left_inclusive
                    .zip(*right_inclusive)
                    .is_some_and(|(left, right)| left > right)
                    || (left_inclusive.is_some() != right_inclusive.is_some())
                    || (left_inclusive.is_some() != rank_type.is_some())
                {
                    return Err(error("Currency Wars settlement rank is invalid"));
                }
            }
            CurrencyWarsFinishRule::BattleStage(stage) => {
                if stage.stage_rule_id == 0
                    || stage.total_turns == 0
                    || !(Ratio::ZERO..=Ratio::ONE).contains(&stage.threshold_position)
                {
                    return Err(error("Currency Wars battle-stage finish is invalid"));
                }
            }
            CurrencyWarsFinishRule::BattlePenalty(_) => {}
        }
    }
    if parts.rank_progression.iter().any(|progression| {
        !matches!(
            (&progression.key, &progression.boundary),
            (
                CurrencyWarsRankProgressionKey::Division { .. },
                CurrencyWarsRankBoundary::GambitDifficulty { .. }
            ) | (
                CurrencyWarsRankProgressionKey::LevelBase { .. }
                    | CurrencyWarsRankProgressionKey::StageBase(_),
                CurrencyWarsRankBoundary::SharedBattleBase { .. }
            ) | (
                CurrencyWarsRankProgressionKey::BinaryDifficulty { .. },
                CurrencyWarsRankBoundary::BinaryDifficultyAddition { .. }
            ) | (
                CurrencyWarsRankProgressionKey::BinaryNode(_),
                CurrencyWarsRankBoundary::BinaryNodePerformLevel { .. }
            )
        )
    }) {
        return Err(error("Currency Wars rank progression shape is invalid"));
    }
    Ok(())
}

fn validate_finish_conditions(
    conditions: &[CurrencyWarsFinishCondition],
) -> Result<(), CurrencyWarsFlowCatalogError> {
    let mut ranges = conditions
        .iter()
        .filter_map(|condition| match &condition.rule {
            CurrencyWarsFinishRule::SettlementRank {
                left_inclusive: Some(left),
                right_inclusive: Some(right),
                rank_type: Some(rank_type),
            } => Some((*left, *right, rank_type.as_ref())),
            _ => None,
        })
        .collect::<Vec<_>>();
    ranges.sort_unstable_by_key(|(left, _, _)| *left);
    let defaults = conditions
        .iter()
        .filter(|condition| {
            matches!(
                condition.rule,
                CurrencyWarsFinishRule::SettlementRank {
                    left_inclusive: None,
                    right_inclusive: None,
                    rank_type: None,
                }
            )
        })
        .count();
    let malformed_settlement = conditions.iter().any(|condition| match &condition.rule {
        CurrencyWarsFinishRule::SettlementRank {
            left_inclusive,
            right_inclusive,
            rank_type,
        } => {
            let default =
                left_inclusive.is_none() && right_inclusive.is_none() && rank_type.is_none();
            let bounded = left_inclusive
                .zip(*right_inclusive)
                .is_some_and(|(left, right)| left <= right)
                && rank_type.as_deref().is_some_and(|rank| !rank.is_empty());
            !default && !bounded
        }
        CurrencyWarsFinishRule::BattleStage(stage) => {
            stage.stage_rule_id == 0
                || stage.total_turns == 0
                || stage.threshold_position <= Ratio::ZERO
                || stage.threshold_position > Ratio::ONE
        }
        CurrencyWarsFinishRule::BattlePenalty(rule) => {
            rule.source_id == 0
                || rule.progress_values.len() != 6
                || rule.hp_progress_values.len() != 6
                || rule.threshold_percent.is_some_and(|value| value > 100)
                || rule.total_turns == 0
                || rule.lethal_rescue_action_value_ratio <= Ratio::ZERO
                || rule.lethal_rescue_action_value_ratio > Ratio::ONE
        }
    });
    let penalty_ids = conditions
        .iter()
        .filter_map(|condition| match &condition.rule {
            CurrencyWarsFinishRule::BattlePenalty(rule) => Some(rule.source_id),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let penalty_count = conditions
        .iter()
        .filter(|condition| matches!(condition.rule, CurrencyWarsFinishRule::BattlePenalty(_)))
        .count();
    let contiguous = ranges.first().is_some_and(|(left, _, _)| *left == 1)
        && ranges.windows(2).all(|pair| {
            pair[0]
                .1
                .checked_add(1)
                .is_some_and(|next| next == pair[1].0)
        });
    let rank_types = ranges
        .iter()
        .map(|(_, _, rank)| *rank)
        .collect::<BTreeSet<_>>();
    if defaults != 1
        || ranges.is_empty()
        || !contiguous
        || rank_types.len() != ranges.len()
        || penalty_ids.len() != penalty_count
        || malformed_settlement
    {
        return Err(error("Currency Wars finish-condition partition is invalid"));
    }
    Ok(())
}

fn unique<T: Ord>(values: impl Iterator<Item = T>) -> Result<(), CurrencyWarsFlowCatalogError> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err(error("Currency Wars flow catalog identity is duplicated"));
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsFlowCatalogError {
    message: Box<str>,
}

impl std::fmt::Display for CurrencyWarsFlowCatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CurrencyWarsFlowCatalogError {}

fn error(message: &'static str) -> CurrencyWarsFlowCatalogError {
    CurrencyWarsFlowCatalogError {
        message: message.into(),
    }
}
