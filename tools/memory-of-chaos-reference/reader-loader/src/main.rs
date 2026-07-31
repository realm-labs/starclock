#![forbid(unsafe_code)]

#[path = "../../../../config/memory-of-chaos-generated/readers/rust/mod.rs"]
mod generated;

use generated::{runtime::SoraBundle, SoraConfig};

const EXPECTED_TABLES: usize = 27;
const EXPECTED_ROWS: usize = 1_521;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let path = arguments.next().ok_or("bundle path missing")?;
    if arguments.next().is_some() {
        return Err("unexpected bundle-loader argument".into());
    }
    let bytes = std::fs::read(path)?;
    let bundle = SoraBundle::parse(&bytes)?;
    let config = SoraConfig::from_source(&bundle)?;
    let mut tables = config
        .tables()
        .map(|table| (table.info().name, table.len()))
        .collect::<Vec<_>>();
    tables.sort_unstable_by_key(|(name, _)| *name);
    if tables.len() != EXPECTED_TABLES {
        return Err(format!("expected {EXPECTED_TABLES} tables, loaded {}", tables.len()).into());
    }
    if let Some((name, _)) = tables.iter().find(|(_, rows)| *rows == 0) {
        return Err(format!("{name}: generated reader loaded no rows").into());
    }
    let rows = tables.iter().map(|(_, count)| count).sum::<usize>();
    if rows != EXPECTED_ROWS {
        return Err(format!("expected {EXPECTED_ROWS} rows, loaded {rows}").into());
    }
    println!(
        "Memory of Chaos bundle loaded through all {EXPECTED_TABLES} generated tables ({EXPECTED_ROWS} rows)."
    );
    Ok(())
}
