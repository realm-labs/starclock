use starclock_ai::{
    CurrencyWarsReplayGambit, CurrencyWarsReplayRequest, decode_currency_wars_replay_request,
    encode_currency_wars_replay, verify_currency_wars_replay,
};

use super::{
    CurrencyWarsCliError, replay_error,
    run::{CliGambit, Execution, RunOptions},
};

pub(super) fn encode(
    options: &RunOptions,
    execution: &Execution,
) -> Result<Vec<u8>, CurrencyWarsCliError> {
    encode_currency_wars_replay(
        replay_request(options),
        execution.replay_identity,
        &execution.report,
    )
    .map_err(replay_error)
}

pub(super) fn is_replay(bytes: &[u8]) -> bool {
    decode_currency_wars_replay_request(bytes).is_ok()
}

pub(super) fn verify(bytes: &[u8], json: bool) -> Result<(), CurrencyWarsCliError> {
    let request = decode_currency_wars_replay_request(bytes).map_err(replay_error)?;
    let options = RunOptions::from_replay(
        request.route_id(),
        request.difficulty_id(),
        match request.gambit() {
            CurrencyWarsReplayGambit::Standard => CliGambit::Standard,
            CurrencyWarsReplayGambit::Overclock => CliGambit::Overclock,
        },
        request.seed(),
    );
    let execution = super::run::execute(&options)?;
    verify_currency_wars_replay(bytes, request, execution.replay_identity, &execution.report)
        .map_err(replay_error)?;
    let report = execution.report;
    let battle_commands = report
        .battles()
        .iter()
        .map(|battle| battle.trace().len())
        .sum::<usize>();
    if json {
        println!(
            "{{\"kind\":\"replay-verify\",\"entry\":\"currency-wars\",\"configuration_components\":9,\"activity_actions\":{},\"nested_battles\":{},\"battle_commands\":{},\"terminal\":\"completed\",\"state_hash\":\"{}\"}}",
            report.activity_steps(),
            report.battles().len(),
            battle_commands,
            super::hex(report.final_state_hash().bytes()),
        );
    } else {
        println!(
            "currency-wars replay verified configuration_components=9 activity_actions={} nested_battles={} battle_commands={} terminal=completed hash={}",
            report.activity_steps(),
            report.battles().len(),
            battle_commands,
            super::hex(report.final_state_hash().bytes()),
        );
    }
    Ok(())
}

pub(super) const fn replay_request(options: &RunOptions) -> CurrencyWarsReplayRequest {
    CurrencyWarsReplayRequest::new(
        options.route,
        options.difficulty,
        match options.gambit {
            CliGambit::Standard => CurrencyWarsReplayGambit::Standard,
            CliGambit::Overclock => CurrencyWarsReplayGambit::Overclock,
        },
        options.seed,
    )
}
