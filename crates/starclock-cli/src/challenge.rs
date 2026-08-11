use starclock_data::{
    ApocalypticCombatCatalog, MemoryCombatCatalog, PureFictionCombatCatalog,
    catalog::SimulationCatalog,
};
use starclock_replay::{codec::CanonicalSink, digest::Sha256Sink};

const CHALLENGE_BUNDLE: &[u8] =
    include_bytes!("../../../config/challenge-runtime-generated/config.sora");

pub fn config_validate(
    args: &[String],
    production: &SimulationCatalog,
) -> Result<(), ChallengeCliError> {
    let json = match args {
        [] => false,
        [flag] if flag == "--json" => true,
        _ => return Err(ChallengeCliError::usage("expected only optional --json")),
    };

    let memory = starclock_data::challenge::memory_of_chaos().map_err(configuration)?;
    let memory_definitions =
        starclock_data::challenge::memory_of_chaos_combat_definitions().map_err(configuration)?;
    let memory_catalog =
        MemoryCombatCatalog::compile(&memory_definitions, production).map_err(configuration)?;

    let apocalyptic = starclock_data::challenge::apocalyptic_shadow().map_err(configuration)?;
    let apocalyptic_definitions =
        starclock_data::challenge::apocalyptic_shadow_combat_definitions()
            .map_err(configuration)?;
    let apocalyptic_catalog =
        ApocalypticCombatCatalog::compile(&apocalyptic_definitions, production)
            .map_err(configuration)?;

    let pure_fiction = starclock_data::challenge::pure_fiction().map_err(configuration)?;
    let pure_fiction_definitions =
        starclock_data::challenge::pure_fiction_combat_definitions().map_err(configuration)?;
    let pure_fiction_catalog =
        PureFictionCombatCatalog::compile(&pure_fiction_definitions, production)
            .map_err(configuration)?;
    let anomaly = starclock_data::challenge::anomaly_arbitration().map_err(configuration)?;

    let memory_nodes = memory
        .stages()
        .iter()
        .map(|stage| stage.nodes().len())
        .sum::<usize>();
    let apocalyptic_nodes = apocalyptic
        .stages
        .iter()
        .map(|stage| stage.nodes.len())
        .sum::<usize>();
    let pure_fiction_nodes = pure_fiction
        .stages
        .iter()
        .map(|stage| stage.nodes.len())
        .sum::<usize>();
    let bundle_sha256 = hex(challenge_bundle_digest());

    if json {
        println!(
            "{{\"kind\":\"challenge-config-validation\",\"valid\":true,\"bundle_sha256\":\"{bundle_sha256}\",\"modes\":[{{\"mode\":\"memory-of-chaos\",\"stages\":{},\"nodes\":{memory_nodes},\"encounters\":{},\"policies\":{},\"approximate_enemies\":{}}},{{\"mode\":\"apocalyptic-shadow\",\"stages\":{},\"nodes\":{apocalyptic_nodes},\"encounters\":{},\"policies\":{},\"approximate_enemies\":{}}},{{\"mode\":\"pure-fiction\",\"stages\":{},\"nodes\":{pure_fiction_nodes},\"encounters\":{},\"policies\":{},\"approximate_enemies\":{}}},{{\"mode\":\"anomaly-arbitration\",\"stages\":{},\"nodes\":5,\"encounters\":5,\"policies\":{},\"approximate_enemies\":0}}]}}",
            memory.stages().len(),
            memory_definitions.encounters().len(),
            memory.policies().len(),
            memory_catalog.approximate_enemy_count(),
            apocalyptic.stages.len(),
            apocalyptic_definitions.encounters().len(),
            apocalyptic.policies.len(),
            apocalyptic_catalog.approximate_enemy_count(),
            pure_fiction.stages.len(),
            pure_fiction_definitions.encounters().len(),
            pure_fiction.policies.len(),
            pure_fiction_catalog.approximate_enemy_count(),
            anomaly.stages.len(),
            anomaly.policies.len(),
        );
    } else {
        println!(
            "challenge config valid bundle_sha256={bundle_sha256} memory=({},{} nodes,{} encounters,{} policies,{} approximate enemies) apocalyptic=({},{} nodes,{} encounters,{} policies,{} approximate enemies) pure_fiction=({},{} nodes,{} encounters,{} policies,{} approximate enemies) anomaly=({},5 nodes,5 encounters,{} policies,0 approximate enemies)",
            memory.stages().len(),
            memory_nodes,
            memory_definitions.encounters().len(),
            memory.policies().len(),
            memory_catalog.approximate_enemy_count(),
            apocalyptic.stages.len(),
            apocalyptic_nodes,
            apocalyptic_definitions.encounters().len(),
            apocalyptic.policies.len(),
            apocalyptic_catalog.approximate_enemy_count(),
            pure_fiction.stages.len(),
            pure_fiction_nodes,
            pure_fiction_definitions.encounters().len(),
            pure_fiction.policies.len(),
            pure_fiction_catalog.approximate_enemy_count(),
            anomaly.stages.len(),
            anomaly.policies.len(),
        );
    }
    Ok(())
}

fn challenge_bundle_digest() -> [u8; 32] {
    let mut digest = Sha256Sink::new();
    digest.write(CHALLENGE_BUNDLE);
    digest.finalize().bytes()
}

fn hex(bytes: [u8; 32]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(64);
    for byte in bytes {
        value.push(char::from(DIGITS[usize::from(byte >> 4)]));
        value.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    value
}

fn configuration(error: impl std::fmt::Display) -> ChallengeCliError {
    ChallengeCliError {
        kind: ChallengeCliErrorKind::Configuration,
        message: error.to_string().into_boxed_str(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ChallengeCliErrorKind {
    Usage,
    Configuration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChallengeCliError {
    kind: ChallengeCliErrorKind,
    message: Box<str>,
}

impl ChallengeCliError {
    fn usage(message: &str) -> Self {
        Self {
            kind: ChallengeCliErrorKind::Usage,
            message: message.into(),
        }
    }

    pub const fn exit_code(&self) -> u8 {
        match self.kind {
            ChallengeCliErrorKind::Usage => 2,
            ChallengeCliErrorKind::Configuration => 3,
        }
    }
}

impl std::fmt::Display for ChallengeCliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ChallengeCliErrorKind::Usage => write!(formatter, "usage error: {}", self.message),
            ChallengeCliErrorKind::Configuration => {
                write!(formatter, "challenge configuration error: {}", self.message)
            }
        }
    }
}

impl std::error::Error for ChallengeCliError {}
