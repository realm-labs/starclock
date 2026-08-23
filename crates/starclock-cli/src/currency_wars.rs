mod replay;
mod run;

use std::fmt::Write as _;

use starclock_data::currency_wars::{load_currency_wars_catalog, summarize_currency_wars_catalog};

pub fn config_validate(args: &[String]) -> Result<(), CurrencyWarsCliError> {
    let json = optional_json(args, "expected only optional --json")?;
    let catalog = load_currency_wars_catalog().map_err(configuration)?;
    let summary = summarize_currency_wars_catalog(&catalog);
    if json {
        println!(
            "{{\"kind\":\"currency-wars-config-validation\",\"valid\":true,\"routes\":{},\"nodes\":{},\"difficulties\":{},\"roles\":{},\"bonds\":{},\"investments\":{},\"project_policies\":{}}}",
            summary.routes,
            summary.nodes,
            summary.difficulties,
            summary.roles,
            summary.bonds,
            summary.investments,
            summary.policies,
        );
    } else {
        println!(
            "currency-wars config valid routes={} nodes={} difficulties={} roles={} bonds={} investments={} project_policies={}",
            summary.routes,
            summary.nodes,
            summary.difficulties,
            summary.roles,
            summary.bonds,
            summary.investments,
            summary.policies,
        );
    }
    Ok(())
}

pub fn coverage(args: &[String]) -> Result<(), CurrencyWarsCliError> {
    let json = optional_json(args, "expected only optional --json")?;
    let catalog = load_currency_wars_catalog().map_err(configuration)?;
    let summary = summarize_currency_wars_catalog(&catalog);
    if json {
        println!(
            "{{\"kind\":\"currency-wars-coverage\",\"source_obligations\":19250,\"source_terminal\":19250,\"source_pending\":0,\"mechanic_programs\":2367,\"mechanic_terminal\":2367,\"fixture_families\":28,\"project_policies\":12,\"native_handlers\":0,\"routes\":{},\"difficulties\":{},\"roles\":{}}}",
            summary.routes, summary.difficulties, summary.roles,
        );
    } else {
        println!(
            "currency-wars coverage source_obligations=19250 source_terminal=19250 source_pending=0 mechanic_programs=2367 mechanic_terminal=2367 fixtures=28 project_policies=12 native_handlers=0 routes={} difficulties={} roles={}",
            summary.routes, summary.difficulties, summary.roles,
        );
    }
    Ok(())
}

pub fn run(args: &[String]) -> Result<(), CurrencyWarsCliError> {
    run::command(args)
}

pub fn is_replay(bytes: &[u8]) -> bool {
    replay::is_replay(bytes)
}

pub fn verify_replay(bytes: &[u8], json: bool) -> Result<(), CurrencyWarsCliError> {
    replay::verify(bytes, json)
}

pub fn inspect(args: &[String]) -> Result<(), CurrencyWarsCliError> {
    let mut route = None;
    let mut json = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--route" if route.is_none() => {
                index += 1;
                route = Some(
                    args.get(index)
                        .ok_or_else(|| usage("--route requires an unsigned integer"))?
                        .parse::<u32>()
                        .map_err(|_| usage("--route requires an unsigned integer"))?,
                );
            }
            "--json" if !json => json = true,
            _ => return Err(usage("expected --route ID and optional --json")),
        }
        index += 1;
    }
    let route = route.ok_or_else(|| usage("inspect requires --route ID"))?;
    let catalog = load_currency_wars_catalog().map_err(configuration)?;
    let route = catalog
        .routes()
        .iter()
        .find(|candidate| candidate.id.get() == route)
        .ok_or_else(|| usage("unknown Currency Wars route"))?;
    if json {
        let mut output = format!(
            "{{\"kind\":\"currency-wars-route\",\"route_id\":{},\"stable_key\":\"{}\",\"nodes\":[",
            route.id.get(),
            json_escape(&route.stable_key),
        );
        for (index, node) in route.nodes.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            write!(
                output,
                "{{\"id\":{},\"stable_key\":\"{}\",\"plane\":{},\"ordinal\":{},\"kind\":\"{:?}\",\"node_template_id\":{},\"encounter_id\":{},\"penalty_bonus_rule_id\":{},\"basic_gold_reward\":{},\"next_node_id\":{}}}",
                node.id.get(),
                json_escape(&node.stable_key),
                node.plane,
                node.ordinal,
                node.kind,
                node.node_template_id,
                node.encounter.get(),
                optional_u32(node.penalty_bonus_rule_id),
                optional_u32(node.basic_gold_reward),
                node.next.map_or_else(|| "null".to_owned(), |id| id.get().to_string()),
            )
            .expect("writing to a String cannot fail");
        }
        output.push_str("]}");
        println!("{output}");
    } else {
        println!(
            "currency-wars route={} stable_key={} nodes={}",
            route.id.get(),
            route.stable_key,
            route.nodes.len(),
        );
        for node in &route.nodes {
            println!(
                "node={} plane={} ordinal={} kind={:?} template={} encounter={} penalty_bonus={} gold={} next={}",
                node.id.get(),
                node.plane,
                node.ordinal,
                node.kind,
                node.node_template_id,
                node.encounter.get(),
                optional_u32(node.penalty_bonus_rule_id),
                optional_u32(node.basic_gold_reward),
                node.next
                    .map_or_else(|| "none".to_owned(), |id| id.get().to_string()),
            );
        }
    }
    Ok(())
}

fn optional_json(args: &[String], message: &str) -> Result<bool, CurrencyWarsCliError> {
    match args {
        [] => Ok(false),
        [flag] if flag == "--json" => Ok(true),
        _ => Err(usage(message)),
    }
}

fn optional_u32(value: Option<u32>) -> String {
    value.map_or_else(|| "null".to_owned(), |value| value.to_string())
}

fn json_escape(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CurrencyWarsCliErrorKind {
    Usage,
    Configuration,
    Simulation,
    Replay,
    Io,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsCliError {
    kind: CurrencyWarsCliErrorKind,
    message: Box<str>,
}

impl CurrencyWarsCliError {
    pub const fn exit_code(&self) -> u8 {
        match self.kind {
            CurrencyWarsCliErrorKind::Usage => 2,
            CurrencyWarsCliErrorKind::Configuration => 3,
            CurrencyWarsCliErrorKind::Replay => 4,
            CurrencyWarsCliErrorKind::Simulation => 6,
            CurrencyWarsCliErrorKind::Io => 7,
        }
    }
}

impl std::fmt::Display for CurrencyWarsCliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            CurrencyWarsCliErrorKind::Usage => write!(formatter, "usage error: {}", self.message),
            CurrencyWarsCliErrorKind::Configuration => {
                write!(
                    formatter,
                    "Currency Wars configuration error: {}",
                    self.message
                )
            }
            CurrencyWarsCliErrorKind::Simulation => {
                write!(
                    formatter,
                    "Currency Wars simulation error: {}",
                    self.message
                )
            }
            CurrencyWarsCliErrorKind::Replay => {
                write!(formatter, "Currency Wars replay error: {}", self.message)
            }
            CurrencyWarsCliErrorKind::Io => {
                write!(formatter, "Currency Wars I/O error: {}", self.message)
            }
        }
    }
}

impl std::error::Error for CurrencyWarsCliError {}

fn usage(message: &str) -> CurrencyWarsCliError {
    CurrencyWarsCliError {
        kind: CurrencyWarsCliErrorKind::Usage,
        message: message.into(),
    }
}

fn configuration(error: impl std::fmt::Display) -> CurrencyWarsCliError {
    CurrencyWarsCliError {
        kind: CurrencyWarsCliErrorKind::Configuration,
        message: error.to_string().into_boxed_str(),
    }
}

fn simulation(message: &str) -> CurrencyWarsCliError {
    CurrencyWarsCliError {
        kind: CurrencyWarsCliErrorKind::Simulation,
        message: message.into(),
    }
}

fn replay_error(error: impl std::fmt::Display) -> CurrencyWarsCliError {
    CurrencyWarsCliError {
        kind: CurrencyWarsCliErrorKind::Replay,
        message: error.to_string().into_boxed_str(),
    }
}

fn io(error: std::io::Error) -> CurrencyWarsCliError {
    CurrencyWarsCliError {
        kind: CurrencyWarsCliErrorKind::Io,
        message: error.to_string().into_boxed_str(),
    }
}

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
