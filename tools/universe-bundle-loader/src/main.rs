#![forbid(unsafe_code)]

#[path = "../../../config/generated/rust/universe_reference/mod.rs"]
mod generated;

use generated::{SoraConfig, runtime::SoraBundle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let path = arguments.next().ok_or("bundle path missing")?;
    let expected_profiles = count(&mut arguments, "profile count")?;
    let expected_worlds = count(&mut arguments, "World count")?;
    let expected_domains = count(&mut arguments, "domain count")?;
    let expected_sources = count(&mut arguments, "source count")?;
    if arguments.next().is_some() {
        return Err("unexpected bundle-loader argument".into());
    }
    let bytes = std::fs::read(path)?;
    let bundle = SoraBundle::parse(&bytes)?;
    let config = SoraConfig::from_source(&bundle)?;
    expect(
        config.universe_profile().len(),
        expected_profiles,
        "UniverseProfile",
    )?;
    expect(
        config.universe_world().len(),
        expected_worlds,
        "UniverseWorld",
    )?;
    expect(
        config.universe_domain().len(),
        expected_domains,
        "UniverseDomain",
    )?;
    expect(
        config.universe_source_record().len(),
        expected_sources,
        "UniverseSourceRecord",
    )?;
    println!(
        "Universe bundle loaded: profiles={expected_profiles} worlds={expected_worlds} domains={expected_domains} sources={expected_sources}."
    );
    Ok(())
}

fn count(
    arguments: &mut impl Iterator<Item = String>,
    name: &'static str,
) -> Result<usize, Box<dyn std::error::Error>> {
    Ok(arguments.next().ok_or(name)?.parse()?)
}

fn expect(
    actual: usize,
    expected: usize,
    table: &'static str,
) -> Result<(), Box<dyn std::error::Error>> {
    if actual != expected {
        return Err(format!("{table}: expected {expected} rows, got {actual}").into());
    }
    Ok(())
}
