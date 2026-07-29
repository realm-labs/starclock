#![forbid(unsafe_code)]

mod generated;

use generated::{runtime::SoraBundle, SoraConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let path = arguments.next().ok_or("bundle path missing")?;
    let expected_tables = count(&mut arguments, "table count")?;
    let expected_rows = count(&mut arguments, "row count")?;
    let expected_empty_tables = count(&mut arguments, "empty-table count")?;
    if arguments.next().is_some() {
        return Err("unexpected bundle-loader argument".into());
    }
    let bytes = std::fs::read(path)?;
    let bundle = SoraBundle::parse(&bytes)?;
    let config = SoraConfig::from_source(&bundle)?;
    let tables = config.tables().collect::<Vec<_>>();
    expect(tables.len(), expected_tables, "table count")?;
    let rows = tables.iter().map(|table| table.len()).sum();
    expect(rows, expected_rows, "row count")?;
    let empty_tables = tables.iter().filter(|table| table.is_empty()).count();
    expect(empty_tables, expected_empty_tables, "empty-table count")?;
    println!(
        "Divergent Universe bundle loaded through every generated reader: \
         tables={expected_tables} rows={expected_rows} \
         empty_tables={expected_empty_tables}."
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
    label: &'static str,
) -> Result<(), Box<dyn std::error::Error>> {
    if actual != expected {
        return Err(format!("{label}: expected {expected}, got {actual}").into());
    }
    Ok(())
}
