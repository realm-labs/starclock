use std::{collections::BTreeMap, sync::Arc};

use serde_json::Value;
use starclock_activity::{
    ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityTerminalOutcome,
};
use starclock_ai::{
    CURRENCY_WARS_BASELINE_BATTLE_COMMAND_BUDGET, CurrencyWarsBaselineController,
    CurrencyWarsReplayGambit, CurrencyWarsReplayIdentity, CurrencyWarsReplayRequest,
    encode_currency_wars_replay, verify_currency_wars_replay,
};
use starclock_data::{
    currency_wars::load_currency_wars_catalog_candidate, load_currency_wars_battle_resources,
};
use starclock_mode_currency_wars::{
    CurrencyWarsBattleAssembler, CurrencyWarsDeployment, CurrencyWarsEntryState,
    CurrencyWarsGambit, CurrencyWarsPosition, CurrencyWarsPositionKind, CurrencyWarsRoleId,
    CurrencyWarsRoleState, CurrencyWarsRoster, CurrencyWarsRun, CurrencyWarsRunDefinition,
    CurrencyWarsRunSetup,
};

const MATRIX: &str =
    include_str!("../../../content-manifests/currency-wars-runtime-v1/coverage-and-release.json");

#[test]
#[ignore = "explicit Goal 21 generated legal matrix"]
fn generated_legal_matrix_completes_real_battles_and_fresh_replay() {
    let manifest: Value = serde_json::from_str(MATRIX).expect("generated matrix parses");
    let rows = manifest["complete_runs"]
        .as_array()
        .expect("matrix rows exist");
    assert_eq!(rows.len(), 97);

    let candidate = load_currency_wars_catalog_candidate().expect("production catalog loads");
    let catalog_identity = candidate.identity().clone();
    let catalog = Arc::new(candidate.into_catalog());
    let resources =
        Arc::new(load_currency_wars_battle_resources(&catalog).expect("battle resources load"));
    let replay_identity = CurrencyWarsReplayIdentity::new(
        catalog_identity.schema_digest().bytes(),
        catalog_identity.configuration_digest().bytes(),
        catalog_identity.content_digest().bytes(),
        resources.digest(),
        resources.combat().digest().bytes(),
    );
    let definition_identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(31).expect("definition ID is non-zero"),
        ActivityDefinitionDigest::new(catalog_identity.content_digest().bytes())
            .expect("content digest is non-zero"),
        ActivityConfigDigest::new(catalog_identity.configuration_digest().bytes())
            .expect("configuration digest is non-zero"),
    );
    let matrix_filter = std::env::var("STARCLOCK_CURRENCY_WARS_MATRIX_ID").ok();
    let battle_command_budget = std::env::var("STARCLOCK_CURRENCY_WARS_BATTLE_COMMAND_BUDGET")
        .map(|value| value.parse::<u32>().expect("debug battle budget is a u32"))
        .unwrap_or(CURRENCY_WARS_BASELINE_BATTLE_COMMAND_BUDGET);
    let mut failures = Vec::new();
    let mut executed = 0_usize;

    for (index, row) in rows.iter().enumerate() {
        let matrix_id = text(row, "id");
        if matrix_filter
            .as_deref()
            .is_some_and(|filter| filter != matrix_id)
        {
            continue;
        }
        executed += 1;
        eprintln!("{matrix_id}: executing");
        let route_id = stable_id(row, "route_id");
        let difficulty_id = stable_id(row, "difficulty_id");
        let seed = number(row, "seed");
        let team_level = u8::try_from(number(row, "team_level")).expect("team level fits u8");
        let gambit = match text(row, "gambit_id") {
            "currency-wars.gambit.standard" => CurrencyWarsGambit::Standard,
            "currency-wars.gambit.overclock" => CurrencyWarsGambit::Overclock,
            other => panic!("{matrix_id}: unsupported Gambit {other}"),
        };
        let route = catalog
            .routes()
            .iter()
            .find(|route| route.id.get() == route_id)
            .unwrap_or_else(|| panic!("{matrix_id}: route is absent"));
        let difficulty = catalog
            .difficulties()
            .iter()
            .find(|difficulty| difficulty.source_id == difficulty_id)
            .unwrap_or_else(|| panic!("{matrix_id}: difficulty is absent"));
        let roster_rows = row["roster"].as_array().expect("matrix roster exists");
        let role_states = roster_rows
            .iter()
            .map(|role| {
                CurrencyWarsRoleState::new(
                    CurrencyWarsRoleId::new(stable_id(role, "role_id"))
                        .expect("role ID is non-zero"),
                    u8::try_from(number(role, "star")).expect("star fits u8"),
                )
                .expect("matrix role state is valid")
            })
            .collect::<Vec<_>>();
        let roster =
            CurrencyWarsRoster::new(&catalog, role_states.iter().copied().map(|role| (role, 1)))
                .unwrap_or_else(|error| panic!("{matrix_id}: roster rejected: {error:?}"));
        let mut front = 0_u8;
        let mut back = 0_u8;
        let positions = roster_rows
            .iter()
            .zip(role_states.iter().copied())
            .map(|(row, role)| {
                let kind = match text(row, "position") {
                    "Back" => CurrencyWarsPositionKind::Back,
                    "Front" | "FrontBackCandidate" => CurrencyWarsPositionKind::Front,
                    other => panic!("{matrix_id}: unsupported position {other}"),
                };
                let index = match kind {
                    CurrencyWarsPositionKind::Front => {
                        front += 1;
                        front
                    }
                    CurrencyWarsPositionKind::Back => {
                        back += 1;
                        back
                    }
                };
                (
                    CurrencyWarsPosition::new(kind, index).expect("matrix position is valid"),
                    role,
                )
            });
        let deployment = CurrencyWarsDeployment::new(&catalog, &roster, team_level, positions)
            .unwrap_or_else(|error| panic!("{matrix_id}: deployment rejected: {error:?}"));
        let progress = &row["required_progression"];
        let entry = CurrencyWarsEntryState::new(
            21,
            number(progress, "completed_standard_gambits") != 0,
            u8::try_from(number(progress, "highest_standard_rank")).expect("rank fits u8"),
        );
        let definition = Arc::new(
            CurrencyWarsRunDefinition::new(
                definition_identity,
                Arc::clone(&catalog),
                route.id,
                difficulty.source_id,
                gambit,
                entry,
                CurrencyWarsRunSetup {
                    initial_gold: 0,
                    initial_team_level: team_level,
                    initial_experience: 0,
                    roster,
                    deployment,
                    enemy_affix_ids: Box::new([]),
                    owned_builds: BTreeMap::new(),
                },
            )
            .unwrap_or_else(|error| panic!("{matrix_id}: definition rejected: {error:?}")),
        );
        let mut run = CurrencyWarsRun::start(
            definition,
            ActivityInstanceId::new(u64::try_from(index + 1).expect("matrix index fits u64"))
                .expect("instance ID is non-zero"),
            ActivityMasterSeed::from_u64(seed),
        )
        .unwrap_or_else(|error| panic!("{matrix_id}: run start rejected: {error:?}"));
        let mut assembler = CurrencyWarsBattleAssembler::new(Arc::clone(&resources), 16)
            .expect("assembler is valid");
        let controller =
            CurrencyWarsBaselineController::with_limits(1_024, battle_command_budget, None)
                .expect("matrix budgets are non-zero");
        let report = match controller.run_to_terminal(&mut run, &mut assembler) {
            Ok(report) => report,
            Err(error) => {
                eprintln!("{matrix_id}: error={error:?}");
                failures.push(format!("{matrix_id}: {error:?}"));
                continue;
            }
        };
        eprintln!(
            "{matrix_id}: terminal={:?}, battles={}, commands={}",
            report.terminal(),
            report.battles().len(),
            report
                .battles()
                .iter()
                .map(|battle| battle.trace().len())
                .sum::<usize>(),
        );
        if report.terminal() == ActivityTerminalOutcome::Faulted {
            for (battle_index, battle) in report.battles().iter().enumerate() {
                if battle.outcome() != starclock_activity::BattleOutcome::Faulted {
                    continue;
                }
                let fault = battle
                    .trace()
                    .iter()
                    .flat_map(|entry| entry.events())
                    .rev()
                    .find(|event| {
                        matches!(event.kind(), starclock_combat::BattleEventKind::Fault(_))
                    })
                    .map(starclock_combat::BattleEvent::kind);
                eprintln!(
                    "{matrix_id}: faulted battle={}, command={:?}, fault={fault:?}",
                    battle_index + 1,
                    battle.trace().last().map(|entry| entry.command()),
                );
                if let Some(entry) = battle.trace().last() {
                    for event in entry.events() {
                        eprintln!("{matrix_id}: fault trace event={:?}", event.kind());
                    }
                }
            }
            failures.push(format!("{matrix_id}: unexpected Faulted terminal"));
            continue;
        }
        assert!(
            matches!(
                report.terminal(),
                ActivityTerminalOutcome::Completed | ActivityTerminalOutcome::Failed
            ),
            "{matrix_id}: unexpected terminal {:?}",
            report.terminal(),
        );
        assert!(!report.battles().is_empty(), "{matrix_id}");
        let replay_request = CurrencyWarsReplayRequest::new(
            route_id,
            difficulty_id,
            match gambit {
                CurrencyWarsGambit::Standard => CurrencyWarsReplayGambit::Standard,
                CurrencyWarsGambit::Overclock => CurrencyWarsReplayGambit::Overclock,
            },
            seed,
        );
        let replay = encode_currency_wars_replay(replay_request, replay_identity, &report)
            .unwrap_or_else(|error| panic!("{matrix_id}: replay encode failed: {error:?}"));
        verify_currency_wars_replay(&replay, replay_request, replay_identity, &report)
            .unwrap_or_else(|error| panic!("{matrix_id}: replay verify failed: {error:?}"));
    }
    assert_eq!(executed, matrix_filter.as_ref().map_or(rows.len(), |_| 1));
    assert!(
        failures.is_empty(),
        "matrix failures:\n{}",
        failures.join("\n")
    );
}

fn text<'a>(row: &'a Value, field: &str) -> &'a str {
    row[field]
        .as_str()
        .unwrap_or_else(|| panic!("{field} is text"))
}

fn number(row: &Value, field: &str) -> u64 {
    row[field]
        .as_u64()
        .unwrap_or_else(|| panic!("{field} is u64"))
}

fn stable_id(row: &Value, field: &str) -> u32 {
    u32::try_from(
        text(row, field)
            .rsplit('.')
            .next()
            .expect("stable ID has a suffix")
            .parse::<u64>()
            .expect("stable ID suffix is numeric"),
    )
    .expect("stable ID fits u32")
}
