use starclock_replay::{codec::CanonicalSink, digest::Sha256Sink};

const EVENT_BUNDLE: &[u8] = include_bytes!("../../../config/event-runtime-generated/config.sora");

pub fn config_validate(args: &[String]) -> Result<(), EventCliError> {
    let json = match args {
        [] => false,
        [flag] if flag == "--json" => true,
        _ => {
            return Err(EventCliError::usage("expected only optional --json"));
        }
    };
    let baseballer = starclock_data::event::galactic_baseballer().map_err(configuration)?;
    let fate = starclock_data::event::fate_star_rail_night().map_err(configuration)?;
    let bundle_sha256 = hex(event_bundle_digest());
    if json {
        println!(
            "{{\"kind\":\"event-config-validation\",\"valid\":true,\"bundle_sha256\":\"{bundle_sha256}\",\"modes\":[{{\"mode\":\"galactic-baseballer\",\"profiles\":{},\"stages\":{},\"stage_periods\":{},\"equipment\":{},\"recipes\":{},\"shop_upgrades\":{},\"strategies\":{},\"team_bonuses\":{},\"policies\":{}}},{{\"mode\":\"fate-star-rail-night\",\"boards\":{},\"owners\":{},\"decks\":{},\"deck_recommendations\":{},\"cards\":{},\"story_fights\":{},\"challenge_fights\":{},\"map_fights\":{},\"policies\":{}}}]}}",
            baseballer.catalog.profiles().len(),
            baseballer.catalog.stages().len(),
            baseballer.catalog.stage_periods().len(),
            baseballer.catalog.equipment().len(),
            baseballer.catalog.recipes().len(),
            baseballer.catalog.shop_upgrades().len(),
            baseballer.catalog.strategies().len(),
            baseballer.catalog.team_bonuses().len(),
            baseballer.policies.len(),
            fate.catalog.boards().len(),
            fate.catalog.owners().len(),
            fate.catalog.decks().len(),
            fate.catalog.recommendations().len(),
            fate.catalog.cards().len(),
            fate.catalog.story_fights().len(),
            fate.catalog.challenge_fights().len(),
            fate.catalog.map_fights().len(),
            fate.policies.len(),
        );
    } else {
        println!(
            "event config valid bundle_sha256={bundle_sha256} baseballer=({} profiles,{} stages,{} stage periods,{} equipment,{} recipes,{} shop upgrades,{} strategies,{} team bonuses,{} policies) fate=({} boards,{} owners,{} decks,{} recommendations,{} cards,{} story fights,{} challenge fights,{} map fights,{} policies)",
            baseballer.catalog.profiles().len(),
            baseballer.catalog.stages().len(),
            baseballer.catalog.stage_periods().len(),
            baseballer.catalog.equipment().len(),
            baseballer.catalog.recipes().len(),
            baseballer.catalog.shop_upgrades().len(),
            baseballer.catalog.strategies().len(),
            baseballer.catalog.team_bonuses().len(),
            baseballer.policies.len(),
            fate.catalog.boards().len(),
            fate.catalog.owners().len(),
            fate.catalog.decks().len(),
            fate.catalog.recommendations().len(),
            fate.catalog.cards().len(),
            fate.catalog.story_fights().len(),
            fate.catalog.challenge_fights().len(),
            fate.catalog.map_fights().len(),
            fate.policies.len(),
        );
    }
    Ok(())
}

fn event_bundle_digest() -> [u8; 32] {
    let mut digest = Sha256Sink::new();
    digest.write(EVENT_BUNDLE);
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

fn configuration(error: impl std::fmt::Display) -> EventCliError {
    EventCliError {
        kind: EventCliErrorKind::Configuration,
        message: error.to_string().into_boxed_str(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EventCliErrorKind {
    Usage,
    Configuration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventCliError {
    kind: EventCliErrorKind,
    message: Box<str>,
}

impl EventCliError {
    fn usage(message: &str) -> Self {
        Self {
            kind: EventCliErrorKind::Usage,
            message: message.into(),
        }
    }

    pub const fn exit_code(&self) -> u8 {
        match self.kind {
            EventCliErrorKind::Usage => 2,
            EventCliErrorKind::Configuration => 3,
        }
    }
}

impl std::fmt::Display for EventCliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            EventCliErrorKind::Usage => write!(formatter, "usage error: {}", self.message),
            EventCliErrorKind::Configuration => {
                write!(formatter, "event configuration error: {}", self.message)
            }
        }
    }
}

impl std::error::Error for EventCliError {}
