use crate::{
    UnitId,
    catalog::action::{HealingDefinition, OrdinaryDamageDefinition},
    id::OperationId,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Operation {
    Damage(DamageOp),
    Heal(HealOp),
}

impl Operation {
    pub(crate) const fn id(&self) -> OperationId {
        match self {
            Self::Damage(operation) => operation.id,
            Self::Heal(operation) => operation.id,
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
