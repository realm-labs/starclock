# Sora 0.6.1 Capability Lock

Goal batch `G21-P0-B1` migrated the current repository toolchain before the
Currency Wars runtime was expanded. The crates.io `sora-cli-0.6.1.crate`
archive is bound to SHA-256
`1e94903eba86640a6c9836ba224c9049152c4d84b6cc57537b79126d124a43af`.
The annotated upstream tag object is
`c7f03adb1e89615d2861bb07bc00abbb564b4c07`, peeled to commit
`207d62ef81042382901fb344ae528e74581abf0e`. Installation uses
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
`562bb18df2dcd327da4520b095555868c40fb7b8600158e3c58b85ab7f85356d`.
Run `node tools/sora/verify-golden.mjs` to reproduce it.

## Locked 0.6.1 constraints

- `format = "required"` does not find `rustfmt.exe` on Windows because Sora's
  probe checks a suffixless filename. Configure `format = "never"`, then run
  the pinned repository `rustfmt` step explicitly.
- `build --clean --project project.toml` fails after an output exists because a
  bare filename has an empty parent path. Use `./project.toml` or a path such as
  `config/project.toml`.
- Sora 0.6.1 requires a stable root `project` identity, declared groups and
  views, and a stable schema-local `id` for every table. Starclock uses one
  default `common` group and one `default` view unless a project documents a
  deliberate projection boundary.
- Generated Sora Rust runtimes now decode unsigned integer primitives. Existing
  signed transport fields remain unchanged until their owning schema performs
  a separate reviewed data-model migration.
- Generated Rust references `serde` derives and `zstd` even for an uncompressed
  native bundle. The golden reader therefore has a standalone exact lock and
  license inventory. Production `starclock-data` dependencies remain a later
  `G01-P1-B10` decision.
- `json-debug` is the diagnostic exporter spelling. It is not a runtime input.
- References target a map table's primary key. Combined indexes are validation
  constraints; only single-field generated lookup helpers are assumed.
- Sora exposes floating schema primitives, but authoritative Starclock content
  continues to use canonical decimal strings or scaled integers.

Primary evidence is the [Sora v0.6.1 release](https://github.com/realm-labs/sora/releases/tag/v0.6.1),
the [Sora versioning policy](https://realm-labs.github.io/sora/versioning.html),
and the executed committed fixture. Architecture examples do not override these
observations.
