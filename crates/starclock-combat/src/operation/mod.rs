//! Closed typed requests for authoritative battle mutation.

mod model;

pub(crate) use model::{
    AddWeaknessOp, ApplyEffectOp, ConsumeHpOp, DamageOp, DetonateDotsOp, HealOp,
    HitOperationScratch, ModifyStateSlotOp, Operation, QueueActionOp, ReduceToughnessOp,
    RemoveEffectsOp, ShieldOp, SuperBreakOp,
};
