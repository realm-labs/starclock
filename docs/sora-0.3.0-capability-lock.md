# Sora 0.3.0 Capability Lock

Goal batch `G01-P1-B4` executed the pinned CLI before any production schema
family was authored. The crates.io `sora-cli-0.3.0.crate` archive is bound to
SHA-256 `90d373102de6a0d7969ebdee51d4ac01ba25c7f2c34661cf581a2c8ead57763a`.
The annotated upstream tag object is
`fe12080ffd94b9f2e9cbabc9b7564152cc27aeeb`, peeled to commit
`4afadfb41fbb05868d44eb1d727e0e8575d803dc`. Installation uses
`node tools/sora/install.mjs`, Cargo's bundled lock, and an ignored
repository-local tool root.

## Proven surface

The committed fixture under `config/sora-golden` proves:

- `check`, configured `build --clean`, direct `schema-lock`, `gen` and `export`;
- `excel-template`, read-only `excel-sync`, and `excel-sync --write` while
  retaining separate template/data roots;
- a primary-key reference, single-field unique secondary index, tagged union
  and ordered child-table materialization;
- formatted Rust model/reader generation, compilation and loading of the
  emitted `.sora` bundle;
- byte-stable schema lock, formatted Rust, binary bundle and per-table
  `json-debug` output;
- semantic Excel template drift using the workbook file list and a read-only
  synchronization report rather than unstable ZIP metadata.

The current stable output digest is
`3068755483a02de85271dd531c5707a6b8e0e08270a782dd60a34a2b3e965f09`.
Run `node tools/sora/verify-golden.mjs` to reproduce it.

## Locked 0.3.0 constraints

- `format = "required"` does not find `rustfmt.exe` on Windows because Sora's
  probe checks a suffixless filename. Configure `format = "never"`, then run
  the pinned repository `rustfmt` step explicitly.
- `build --clean --project project.toml` fails after an output exists because a
  bare filename has an empty parent path. Use `./project.toml` or a path such as
  `config/project.toml`.
- Sora accepts unsigned schema primitives, but its generated Sora Rust runtime
  has no unsigned `SoraDecode` implementations. Author transport IDs/order as
  bounded positive `i32` and convert them into Starclock's unsigned newtypes at
  the data/domain boundary.
- Generated Rust references `serde` derives and `zstd` even for an uncompressed
  native bundle. The golden reader therefore has a standalone exact lock and
  license inventory. Production `starclock-data` dependencies remain a later
  `G01-P1-B10` decision.
- `json-debug` is the diagnostic exporter spelling. It is not a runtime input.
- References target a map table's primary key. Combined indexes are validation
  constraints; only single-field generated lookup helpers are assumed.
- Sora exposes floating schema primitives, but authoritative Starclock content
  continues to use canonical decimal strings or scaled integers.

Primary evidence is the [Sora v0.3.0 release](https://github.com/realm-labs/sora/releases/tag/v0.3.0),
the [Sora versioning policy](https://realm-labs.github.io/sora/versioning.html),
and the executed committed fixture. Architecture examples do not override these
observations.
