//! Closed typed requests for authoritative battle mutation.

mod model;

pub(crate) use model::{
    AddWeaknessFromAlliedElementsOp, AddWeaknessOp, ApplyEffectOp, ChangePresenceOp, ConsumeHpOp,
    CreateCountdownOp, DamageOp, DetonateDotsOp, EncounterLifecycleOp, EnemyPhaseOp, ForceBreakOp,
    HealOp, HitOperationScratch, ModifyStateSlotOp, ModifyTeamResourceOp, Operation, QueueActionOp,
    QueueRuleActionOp, ReduceToughnessOp, RemoveEffectsOp, RemoveShieldsOp, ReviveOp, ShieldOp,
    SummonLinkedOp, SuperBreakOp, TransformOp, UnitLifecycleOp,
};
