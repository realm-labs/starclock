use crate::{
    UnitId,
    catalog::action::{TargetInvalidationPolicy, UnitTargetSelector},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TargetCommitment {
    pub(crate) selector: UnitTargetSelector,
    pub(crate) invalidation: TargetInvalidationPolicy,
    pub(crate) primary: Option<UnitId>,
    pub(crate) targets: Box<[UnitId]>,
}
