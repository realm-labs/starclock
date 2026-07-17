//! Deterministic schema-template to bootstrap-workbook materializer.

use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

use calamine::{Data, Reader, open_workbook_auto};
use rust_xlsxwriter::{Workbook, XlsxError};

const PROJECTION_ROWS: usize = 7;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let [template_root, row_root, output_root] = required_arguments()?;
    if output_root.exists() {
        return Err(format!(
            "refusing to overwrite existing output root {}",
            output_root.display()
        )
        .into());
    }
    let mut templates = files_with_extension(&template_root, "xlsx")?;
    templates.sort();
    if templates.is_empty() {
        return Err("template root contains no .xlsx files".into());
    }
    fs::create_dir_all(&output_root)?;
    for template in templates {
        materialize(&template, &row_root, &output_root)?;
    }
    Ok(())
}

fn required_arguments() -> Result<[PathBuf; 3], Box<dyn std::error::Error>> {
    let mut arguments = env::args_os().skip(1).map(PathBuf::from);
    let values = [
        arguments.next().ok_or("missing template root")?,
        arguments.next().ok_or("missing row root")?,
        arguments.next().ok_or("missing output root")?,
    ];
    if arguments.next().is_some() {
        return Err("usage: starclock-workbook-bootstrap <templates> <rows> <output>".into());
    }
    Ok(values)
}

fn materialize(
    template: &Path,
    row_root: &Path,
    output_root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut source = open_workbook_auto(template)?;
    let sheet_name = source
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| format!("{} has no worksheet", template.display()))?;
    let range = source.worksheet_range(&sheet_name)?;
    let projection = range
        .rows()
        .take(PROJECTION_ROWS)
        .map(|row| row.iter().map(cell_text).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    if projection.len() != PROJECTION_ROWS
        || projection[2].first().map(String::as_str) != Some("#field")
    {
        return Err(format!("{} is not a Sora template", template.display()).into());
    }
    let fields = projection[2].iter().skip(1).cloned().collect::<Vec<_>>();
    let table = template
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or("invalid template name")?;
    let row_file = row_root.join(format!("{table}.tsv"));
    let rows = if row_file.exists() {
        read_rows(&row_file, &fields)?
    } else {
        Vec::new()
    };
    let destination = output_root.join(template.file_name().ok_or("invalid template filename")?);
    write_workbook(&destination, &sheet_name, &projection, &rows)?;
    Ok(())
}

fn read_rows(
    file: &Path,
    fields: &[String],
) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    let source = fs::read_to_string(file)?;
    if source.contains('\r') {
        return Err(format!("{} must use LF line endings", file.display()).into());
    }
    let mut lines = source.lines();
    let header = lines
        .next()
        .ok_or_else(|| format!("{} is empty", file.display()))?;
    let names = header.split('\t').map(str::to_owned).collect::<Vec<_>>();
    let field_columns = fields
        .iter()
        .enumerate()
        .map(|(index, field)| (field, index))
        .collect::<BTreeMap<_, _>>();
    let mut source_columns = Vec::new();
    for name in &names {
        source_columns.push(
            *field_columns
                .get(name)
                .ok_or_else(|| format!("{} has unknown field {name}", file.display()))?,
        );
    }
    let mut result = Vec::new();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let values = line.split('\t').collect::<Vec<_>>();
        if values.len() != names.len() {
            return Err(format!(
                "{} row {} has {} cells, expected {}",
                file.display(),
                offset + 2,
                values.len(),
                names.len()
            )
            .into());
        }
        let mut row = vec![String::new(); fields.len()];
        for (value, column) in values.into_iter().zip(&source_columns) {
            if value.contains(['\n', '\r', '\t']) {
                return Err("TSV cell contains a control separator".into());
            }
            row[*column] = value.to_owned();
        }
        result.push(row);
    }
    Ok(result)
}

fn write_workbook(
    destination: &Path,
    sheet_name: &str,
    projection: &[Vec<String>],
    rows: &[Vec<String>],
) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet().set_name(sheet_name)?;
    for (row, cells) in projection.iter().enumerate() {
        for (column, value) in cells.iter().enumerate() {
            if !value.is_empty() {
                worksheet.write_string(row as u32, column as u16, value)?;
            }
        }
    }
    for (offset, cells) in rows.iter().enumerate() {
        for (column, value) in cells.iter().enumerate() {
            if !value.is_empty() {
                worksheet.write_string(
                    (PROJECTION_ROWS + offset) as u32,
                    (column + 1) as u16,
                    value,
                )?;
            }
        }
    }
    workbook.save(destination)
}

fn cell_text(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(value) => value.clone(),
        Data::Int(value) => value.to_string(),
        Data::Float(value) => value.to_string(),
        Data::Bool(value) => value.to_string(),
        Data::DateTime(value) => value.to_string(),
        Data::DateTimeIso(value) | Data::DurationIso(value) => value.clone(),
        Data::Error(value) => format!("{value:?}"),
    }
}

fn files_with_extension(root: &Path, extension: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    fs::read_dir(root)?
        .filter_map(|entry| match entry {
            Ok(entry)
                if entry.path().extension().and_then(|value| value.to_str()) == Some(extension) =>
            {
                Some(Ok(entry.path()))
            }
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}
