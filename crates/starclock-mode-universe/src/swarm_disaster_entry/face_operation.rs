//! Closed operation vocabulary for released Swarm Audience Die faces.

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum FaceOperation {
    AddMazeBuff,
    AllowMoveToReplicateCell,
    BenefitsBySeed,
    EnterCellTriggerBuff,
    EnterEmptyGetMoney,
    GetMoneyByProtectCell,
    LockMarkCell,
    MoveMarkCellUpgradeReward,
    MoveToSwarmGetBuff,
    ProtectCollaspeCell,
    RandomNeighborSpecialType,
    RandomSetSpecialType,
    ReplicateAllAroundCell,
    ReplicateCellToAround,
    ReplicateLastCell,
    SelectAndToFightCell,
    SelectBuffToEmpty,
    SelectCellGetHelp,
    SelectCellToProtect,
    SelectExceptCellGetHelp,
    SelectMiracleToEmpty,
    SetAroundBlockType,
    SetCellTypeAndTakeReward,
    SetColCanMove,
    SetMarkToRandomCell,
    SetMarkType,
    SetNeighborSpecialType,
    SetSpecialType,
    ToRandomBlockType,
    TriggerMark,
    TrunEmptyToReward,
    TurnEventCellToEmpty,
    TurnFightCellToEmpty,
}

impl FaceOperation {
    pub(super) const fn code(self) -> u64 {
        self as u64 + 1
    }

    pub(super) const fn is_mercy(self) -> bool {
        matches!(
            self,
            Self::SetSpecialType
                | Self::RandomSetSpecialType
                | Self::RandomNeighborSpecialType
                | Self::SetNeighborSpecialType
        )
    }

    pub(super) fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "AddMazeBuff" => Self::AddMazeBuff,
            "AllowMoveToReplicateCell" => Self::AllowMoveToReplicateCell,
            "BenefitsBySeed" => Self::BenefitsBySeed,
            "EnterCellTriggerBuff" => Self::EnterCellTriggerBuff,
            "EnterEmptyGetMoney" => Self::EnterEmptyGetMoney,
            "GetMoneyByProtectCell" => Self::GetMoneyByProtectCell,
            "LockMarkCell" => Self::LockMarkCell,
            "MoveMarkCellUpgradeReward" => Self::MoveMarkCellUpgradeReward,
            "MoveToSwarmGetBuff" => Self::MoveToSwarmGetBuff,
            "ProtectCollaspeCell" => Self::ProtectCollaspeCell,
            "RandomNeighborSpecialType" => Self::RandomNeighborSpecialType,
            "RandomSetSpecialType" => Self::RandomSetSpecialType,
            "ReplicateAllAroundCell" => Self::ReplicateAllAroundCell,
            "ReplicateCellToAround" => Self::ReplicateCellToAround,
            "ReplicateLastCell" => Self::ReplicateLastCell,
            "SelectAndToFightCell" => Self::SelectAndToFightCell,
            "SelectBuffToEmpty" => Self::SelectBuffToEmpty,
            "SelectCellGetHelp" => Self::SelectCellGetHelp,
            "SelectCellToProtect" => Self::SelectCellToProtect,
            "SelectExceptCellGetHelp" => Self::SelectExceptCellGetHelp,
            "SelectMiracleToEmpty" => Self::SelectMiracleToEmpty,
            "SetAroundBlockType" => Self::SetAroundBlockType,
            "SetCellTypeAndTakeReward" => Self::SetCellTypeAndTakeReward,
            "SetColCanMove" => Self::SetColCanMove,
            "SetMarkToRandomCell" => Self::SetMarkToRandomCell,
            "SetMarkType" => Self::SetMarkType,
            "SetNeighborSpecialType" => Self::SetNeighborSpecialType,
            "SetSpecialType" => Self::SetSpecialType,
            "ToRandomBlockType" => Self::ToRandomBlockType,
            "TriggerMark" => Self::TriggerMark,
            "TrunEmptyToReward" => Self::TrunEmptyToReward,
            "TurnEventCellToEmpty" => Self::TurnEventCellToEmpty,
            "TurnFightCellToEmpty" => Self::TurnFightCellToEmpty,
            _ => return None,
        })
    }

    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::AddMazeBuff => "AddMazeBuff",
            Self::AllowMoveToReplicateCell => "AllowMoveToReplicateCell",
            Self::BenefitsBySeed => "BenefitsBySeed",
            Self::EnterCellTriggerBuff => "EnterCellTriggerBuff",
            Self::EnterEmptyGetMoney => "EnterEmptyGetMoney",
            Self::GetMoneyByProtectCell => "GetMoneyByProtectCell",
            Self::LockMarkCell => "LockMarkCell",
            Self::MoveMarkCellUpgradeReward => "MoveMarkCellUpgradeReward",
            Self::MoveToSwarmGetBuff => "MoveToSwarmGetBuff",
            Self::ProtectCollaspeCell => "ProtectCollaspeCell",
            Self::RandomNeighborSpecialType => "RandomNeighborSpecialType",
            Self::RandomSetSpecialType => "RandomSetSpecialType",
            Self::ReplicateAllAroundCell => "ReplicateAllAroundCell",
            Self::ReplicateCellToAround => "ReplicateCellToAround",
            Self::ReplicateLastCell => "ReplicateLastCell",
            Self::SelectAndToFightCell => "SelectAndToFightCell",
            Self::SelectBuffToEmpty => "SelectBuffToEmpty",
            Self::SelectCellGetHelp => "SelectCellGetHelp",
            Self::SelectCellToProtect => "SelectCellToProtect",
            Self::SelectExceptCellGetHelp => "SelectExceptCellGetHelp",
            Self::SelectMiracleToEmpty => "SelectMiracleToEmpty",
            Self::SetAroundBlockType => "SetAroundBlockType",
            Self::SetCellTypeAndTakeReward => "SetCellTypeAndTakeReward",
            Self::SetColCanMove => "SetColCanMove",
            Self::SetMarkToRandomCell => "SetMarkToRandomCell",
            Self::SetMarkType => "SetMarkType",
            Self::SetNeighborSpecialType => "SetNeighborSpecialType",
            Self::SetSpecialType => "SetSpecialType",
            Self::ToRandomBlockType => "ToRandomBlockType",
            Self::TriggerMark => "TriggerMark",
            Self::TrunEmptyToReward => "TrunEmptyToReward",
            Self::TurnEventCellToEmpty => "TurnEventCellToEmpty",
            Self::TurnFightCellToEmpty => "TurnFightCellToEmpty",
        }
    }
}
