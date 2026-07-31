#![forbid(unsafe_code)]

#[path = "../../../../config/fate-star-rail-night-generated/readers/rust/mod.rs"]
mod generated;

use generated::{SoraConfig, runtime::SoraBundle};

const EXPECTED_TABLES: usize = 48;
const EXPECTED_ROWS: usize = 5_934;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let path = arguments.next().ok_or("bundle path missing")?;
    if arguments.next().is_some() {
        return Err("unexpected bundle-loader argument".into());
    }
    let bytes = std::fs::read(path)?;
    let bundle = SoraBundle::parse(&bytes)?;
    let config = SoraConfig::from_source(&bundle)?;
    let tables = config
        .tables()
        .map(|table| (table.info().name, table.len()))
        .collect::<Vec<_>>();
    if tables.len() != EXPECTED_TABLES {
        return Err(format!("expected {EXPECTED_TABLES} tables, loaded {}", tables.len()).into());
    }
    if let Some((name, _)) = tables
        .iter()
        .find(|(name, rows)| !name.starts_with("Fsn") || *rows == 0)
    {
        return Err(format!("{name}: missing or non-Fate table").into());
    }
    let rows = tables.iter().map(|(_, count)| count).sum::<usize>();
    if rows != EXPECTED_ROWS {
        return Err(format!("expected {EXPECTED_ROWS} rows, loaded {rows}").into());
    }
    println!(
        "Fate bundle loaded through every generated reader: tables={EXPECTED_TABLES} rows={EXPECTED_ROWS} empty_tables=0."
    );
    Ok(())
}
