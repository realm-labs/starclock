use starclock_combat::{Battle, BattleEvent, BattlePhase, Resolution};

pub(crate) fn advance_boundary_if_offered(battle: &mut Battle) -> Option<Resolution> {
    let command = battle.advance_command()?;
    Some(
        battle
            .apply(command)
            .expect("the current action-boundary continuation remains legal"),
    )
}

pub(crate) fn settle_ready_boundaries(battle: &mut Battle) -> Vec<BattleEvent> {
    let mut events = Vec::new();
    for _ in 0..32 {
        if battle.view().phase() != BattlePhase::ReadyToAdvance {
            return events;
        }
        events.extend_from_slice(
            battle
                .advance()
                .expect("the current stable action boundary advances")
                .events(),
        );
    }
    panic!("fixture exceeded the stable action-boundary settlement bound");
}
