//! Transactional target revalidation and journaled repeated-hit draws.

use crate::{
    battle::fault::BattleFault,
    catalog::action::TargetRelation,
    rng::types::DrawPurpose,
    target::{model::TargetCommitment, select},
};

use super::transaction::{Transaction, action_fault};

impl Transaction<'_> {
    pub(super) fn resolve_hit_targets(
        &mut self,
        actor: crate::UnitId,
        commitment: &mut TargetCommitment,
    ) -> Result<Box<[crate::UnitId]>, BattleFault> {
        let rng = &mut self.state.rng;
        let journal = &mut self.journal;
        select::resolve_for_hit(
            &self.state.units,
            &self.state.formations,
            actor,
            commitment,
            |count| {
                let before = rng.draw_count();
                let selected = rng
                    .choose_index(DrawPurpose::BOUNCE_TARGET, count)
                    .map_err(|_| select::TargetError::ChoiceFailed)?
                    .ok_or(select::TargetError::ChoiceFailed)?;
                for index in before..rng.draw_count() {
                    journal.rng_draw(index, DrawPurpose::BOUNCE_TARGET.code());
                }
                usize::try_from(selected.value()).map_err(|_| select::TargetError::ChoiceFailed)
            },
        )
        .map_err(|_| action_fault(32))
    }

    pub(super) fn draw_bounce_target(
        &mut self,
        actor: crate::UnitId,
        relation: TargetRelation,
    ) -> Result<crate::UnitId, BattleFault> {
        let side = self
            .state
            .units
            .get(actor)
            .ok_or_else(|| action_fault(33))?
            .side;
        let pool = select::stable_pool(&self.state.units, &self.state.formations, side, relation);
        let count = u32::try_from(pool.len()).map_err(|_| action_fault(34))?;
        if count == 0 {
            return Err(action_fault(35));
        }
        let before = self.state.rng.draw_count();
        let selected = self
            .state
            .rng
            .choose_index(DrawPurpose::BOUNCE_TARGET, count)
            .map_err(|_| action_fault(36))?
            .ok_or_else(|| action_fault(37))?;
        for index in before..self.state.rng.draw_count() {
            self.journal
                .rng_draw(index, DrawPurpose::BOUNCE_TARGET.code());
        }
        let index = usize::try_from(selected.value()).map_err(|_| action_fault(38))?;
        pool.get(index).copied().ok_or_else(|| action_fault(39))
    }
}
