//! Closed typed requests for authoritative battle mutation.

mod model;

pub(crate) use model::{
    AddWeaknessFromAlliedElementsOp, AddWeaknessOp, ApplyEffectOp, ChangePresenceOp, ConsumeHpOp,
    CreateCountdownOp, CreateToughnessLayerOp, DamageOp, DeductActionValueOp, DetonateDotsOp,
    EncounterLifecycleOp, EnemyPhaseOp, ForceBreakOp, HealOp, HitOperationScratch,
    ModifyStateSlotOp, ModifyTeamResourceOp, Operation, QueueActionOp, QueueRuleActionOp,
    ReduceMaximumHpOp, ReduceToughnessOp, RemoveEffectsOp, RemoveShieldsOp, RemoveToughnessLayerOp,
    ReviveOp, ShieldOp, SummonLinkedOp, SuperBreakOp, TransformOp, UnitLifecycleOp,
};
