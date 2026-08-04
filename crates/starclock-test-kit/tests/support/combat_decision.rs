use starclock_combat::{Battle, Command, Resolution};

pub(crate) fn pass_interrupt_if_offered(battle: &mut Battle) -> Option<Resolution> {
    let command = battle
        .decision()?
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::PassInterruptWindow { .. }))?
        .clone();
    Some(
        battle
            .apply(command)
            .expect("an exactly offered interrupt pass remains legal"),
    )
}
