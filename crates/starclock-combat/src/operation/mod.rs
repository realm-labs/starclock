//! Closed typed requests for authoritative battle mutation.

mod model;

pub(crate) use model::{
    AddWeaknessOp, ApplyEffectOp, ChangePresenceOp, ConsumeHpOp, CreateCountdownOp, DamageOp,
    DetonateDotsOp, EncounterLifecycleOp, EnemyPhaseOp, HealOp, HitOperationScratch,
    ModifyStateSlotOp, ModifyTeamResourceOp, Operation, QueueActionOp, QueueRuleActionOp,
    ReduceToughnessOp, RemoveEffectsOp, ReviveOp, ShieldOp, SummonLinkedOp, SuperBreakOp,
    TransformOp, UnitLifecycleOp,
};
