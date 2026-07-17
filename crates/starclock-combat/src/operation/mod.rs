//! Closed typed requests for authoritative battle mutation.

mod model;

pub(crate) use model::{
    AddWeaknessOp, ConsumeHpOp, DamageOp, HealOp, HitOperationScratch, Operation,
    ReduceToughnessOp, ShieldOp, SuperBreakOp,
};
