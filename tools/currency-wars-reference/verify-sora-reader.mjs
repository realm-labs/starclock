#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { resolve } from "node:path";

const root = resolve(process.cwd());
const bundle = resolve(process.argv[2] ?? "");
if (!process.argv[2]) {
  throw new Error("usage: verify-sora-reader.mjs <config.sora>");
}

const contract = JSON.parse(
  readFileSync(
    resolve(root, "content-manifests/currency-wars-v1/normalized-schema.json"),
    "utf8",
  ),
);
const expectedRows = contract.files.reduce((sum, file) => {
  const path = resolve(root, "content-reference/currency-wars-v1", file.file);
  return sum + (
    existsSync(path) ? JSON.parse(readFileSync(path, "utf8")).length : 0
  );
}, 0);
const expectedTables = contract.files.length;

const scratch = resolve(root, ".cache/currency-wars-sora-reader");
rmSync(scratch, { force: true, recursive: true });
mkdirSync(scratch, { recursive: true });
copyFileSync(
  resolve(root, "config/sora-golden/reader/Cargo.lock"),
  resolve(scratch, "Cargo.lock"),
);
writeFileSync(
  resolve(scratch, "Cargo.toml"),
  `[package]
name = "starclock-sora-golden-reader"
version = "0.0.0"
edition = "2024"
publish = false

[[bin]]
name = "starclock-sora-golden-reader"
path = "main.rs"

[dependencies]
serde = { version = "=1.0.228", features = ["derive", "rc"] }
zstd = "=0.13.3"

[workspace]
`,
);

const generatedModule = resolve(
  root,
  "config/currency-wars-generated/rust/mod.rs",
).replaceAll("\\", "\\\\");
writeFileSync(
  resolve(scratch, "main.rs"),
  `#[path = "${generatedModule}"]
mod generated;

use generated::SoraConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bundle_path = std::env::args().nth(1).ok_or("missing bundle path")?;
    let expected_tables: usize = std::env::args()
        .nth(2)
        .ok_or("missing expected table count")?
        .parse()?;
    let expected_rows: usize = std::env::args()
        .nth(3)
        .ok_or("missing expected row count")?
        .parse()?;
    let bytes = std::fs::read(bundle_path)?;
    let bundle = generated::runtime::SoraBundle::parse(&bytes)?;
    let config = SoraConfig::from_source(&bundle)?;
    let mut tables = config.tables().collect::<Vec<_>>();
    tables.sort_by_key(|table| table.info().name);
    let rows = tables.iter().map(|table| table.len()).sum::<usize>();
    assert_eq!(tables.len(), expected_tables);
    assert_eq!(rows, expected_rows);
    assert!(tables.iter().all(|table| table.info().name.starts_with("CurrencyWars")));
    println!("loaded {expected_tables} Currency Wars tables with {expected_rows} rows");
    Ok(())
}
`,
);

const output = execFileSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--locked",
    "--manifest-path",
    resolve(scratch, "Cargo.toml"),
    "--",
    bundle,
    String(expectedTables),
    String(expectedRows),
  ],
  {
    cwd: root,
    encoding: "utf8",
    env: {
      ...process.env,
      CARGO_TARGET_DIR: resolve(root, ".cache/currency-wars-sora-reader-target"),
    },
    stdio: ["ignore", "pipe", "inherit"],
  },
);
process.stdout.write(output);
