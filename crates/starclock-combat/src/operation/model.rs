use crate::{
    UnitId,
    catalog::action::{
        HealingDefinition, HpConsumptionDefinition, OrdinaryDamageDefinition, ShieldDefinition,
    },
    id::OperationId,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Operation {
    Damage(DamageOp),
    Heal(HealOp),
    Shield(ShieldOp),
    ConsumeHp(ConsumeHpOp),
    AddWeakness(AddWeaknessOp),
    ReduceToughness(ReduceToughnessOp),
    SuperBreak(SuperBreakOp),
}

impl Operation {
    pub(crate) const fn id(&self) -> OperationId {
        match self {
            Self::Damage(operation) => operation.id,
            Self::Heal(operation) => operation.id,
            Self::Shield(operation) => operation.id,
            Self::ConsumeHp(operation) => operation.id,
            Self::AddWeakness(operation) => operation.id,
            Self::ReduceToughness(operation) => operation.id,
            Self::SuperBreak(operation) => operation.id,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct HitOperationScratch {
    pub(crate) effective_reductions: BTreeMap<UnitId, crate::RawToughness>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AddWeaknessOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: crate::catalog::action::WeaknessApplicationDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReduceToughnessOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: crate::ToughnessReductionDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SuperBreakOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: crate::formula::toughness::SuperBreakDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DamageOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) formula: OrdinaryDamageDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HealOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) formula: HealingDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShieldOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) formula: ShieldDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ConsumeHpOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: HpConsumptionDefinition,
}
