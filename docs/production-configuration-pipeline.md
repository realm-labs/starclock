# Production configuration pipeline

Goal 01 batch `G01-P1-B10` installs the authoritative Excel/Sora production
path described by document 07. It does not promote incomplete content: the
initial workbooks contain the frozen 283 identities and provenance bindings as
disabled `Cataloged`, `Documented` or `Researching` rows, while every executable
domain table remains empty.

## Authoritative paths

- `config/project.toml` is the sole production Sora project.
- `config/schema/*.toml` defines every production source as `.xlsx`.
- `config/data/*.xlsx` is the designer-authoritative source root.
- `config/generated/config.sora` is the only runtime bundle.
- `config/generated/rust` is private pinned Sora 0.3.0 reader code.
- `config/generated/debug-json` is deterministic review/test evidence only.
- `config/generated/templates` is the header/schema projection, never live data.

There are exactly 80 source workbooks, one per table. The initial five populated
tables are `SourceRecord`, `EvidenceRecord`, `ContentIdentity`,
`ContentEvidenceBinding` and singleton `ConfigManifest`. All other tables have
schema-owned headers and zero rows.

## Deterministic bootstrap

The one-time command is:

```powershell
node tools/config-production/bootstrap.mjs --output config/data
```

It verifies reference-pack digest
`0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a`
and goal-manifest digest
`e2188c7844d678253c98d569db017dbad7101541cf502aba4c2eb80c0435bf19`.
It asks pinned Sora for current Excel projections, then invokes the standalone
`rust_xlsxwriter = 0.96.0` materializer. The materializer reads that projection
with pinned `calamine = 0.35.0`, sorts frozen identities and writes canonical
cell strings.

Transport IDs are assigned by lexical content key. Every identity fact retains
its prepared reference key and pack evidence. Initial coverage state is
preserved, but `enabled = false`; identity staging cannot pretend to be
`DataReady` content.

The command refuses any existing output directory and has no merge/force flag.
Normal verification generates fresh roots under `.cache`, exports each through
Sora and compares binary and diagnostic bytes; it never writes `config/data`.

## Runtime conversion boundary

`starclock-data::bundle::inspect` accepts bytes and calls only the generated
`SoraBundle` reader. It returns Starclock-owned manifest metadata and counts;
generated rows and Sora errors do not appear in public signatures. Domain
catalog construction follows in `G01-P1-B11`.

The boundary has no filesystem or JSON parser. A regression test passes
diagnostic JSON bytes and requires Sora's magic check to reject them. Production
`serde = 1.0.228` and `zstd = 0.13.3` dependencies exist solely to compile the
unmodified reader and remain private to `starclock-data`.

## Reproduction and golden

Run:

```powershell
node tools/config-production/generate-bootstrap-policy.mjs
node tools/config-production/verify.mjs
```

The verifier checks the exact 47-package authoring-tool lock, proves read-only
`excel-sync`, proves no-overwrite without byte changes, creates two independent
bootstrap roots, exports both through Sora, and compares their `.sora` and debug
tables. It then compares schema lock, formatted Rust, binary and diagnostic
artifacts with the 274-file golden. Raw `.xlsx` ZIP bytes are not runtime IDs.

The stable output digest is
`120fe8235e582e825e38a7bb2a6887e0648b2408261e0d4628c4f1800e64089e`.
It contains 283 disabled identities, zero enabled identities and zero executable
domain rows, so B10 changes no Goal 01 `DataReady` coverage.
