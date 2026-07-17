# Pinned Sora 0.3.0

Install the checksum-bound CLI into the ignored repository-local tool cache:

```sh
node tools/sora/install.mjs
```

The installer downloads the exact crates.io archive, verifies SHA-256 before
invoking Cargo's locked installation, and verifies `sora --version`. It does not
add Sora or its dependency graph to the production Cargo workspace.

Verify the committed capability golden:

```sh
node tools/sora/verify-golden.mjs
```

The golden exercises schema checking, configured builds, direct schema lock,
Excel template generation, read-only and write synchronization, references,
single-field unique indexes, unions, child-table materialization, Rust codegen
and runtime loading, binary `.sora` export and per-table diagnostic JSON. Use
`--bless` only during an explicitly reviewed Sora/tool/schema migration.
