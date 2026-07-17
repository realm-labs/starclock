use crate::{
    UnitId,
    catalog::action::{
        HealingDefinition, HpConsumptionDefinition, OrdinaryDamageDefinition, ShieldDefinition,
    },
    id::OperationId,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Operation {
    Damage(DamageOp),
    Heal(HealOp),
    Shield(ShieldOp),
    ConsumeHp(ConsumeHpOp),
}

impl Operation {
    pub(crate) const fn id(&self) -> OperationId {
        match self {
            Self::Damage(operation) => operation.id,
            Self::Heal(operation) => operation.id,
            Self::Shield(operation) => operation.id,
            Self::ConsumeHp(operation) => operation.id,
        }
    }
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
