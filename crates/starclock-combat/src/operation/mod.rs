//! Closed typed requests for authoritative battle mutation.

mod model;

pub(crate) use model::{
    AddWeaknessOp, ApplyEffectOp, ChangePresenceOp, ConsumeHpOp, DamageOp, DetonateDotsOp, HealOp,
    HitOperationScratch, ModifyStateSlotOp, ModifyTeamResourceOp, Operation, QueueActionOp,
    ReduceToughnessOp, RemoveEffectsOp, ReviveOp, ShieldOp, SummonLinkedOp, SuperBreakOp,
    TransformOp, UnitLifecycleOp,
};
