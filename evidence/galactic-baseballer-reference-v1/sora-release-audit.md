# Goal 16 Sora Reader and Export Audit

`G16-P3-B4` generated the isolated Rust readers, compressed binary bundle and
debug export for the 40-table Goal 16 Sora project. These artifacts remain
Candidate/reference-only and are not imported by any Starclock runtime crate.

## Generated result

| Field | Result |
|---|---|
| Sora | `0.3.0` |
| Tables | 40 |
| Rows | 10,615 |
| Empty tables | 0 |
| Generated Rust files | 42 (`40` tables plus `mod.rs` and `runtime.rs`) |
| Debug JSON files | 40 |
| Bundle bytes | 741,203 |
| Bundle SHA-256 | `3d18f22e3def1c46f0901620769edbf2aa3266af57ca800588b6d18bde7af07a` |
| Reader tree SHA-256 | `927e7077999dabc3389cdb9af6fdbed3e6995438278274e396e29c618a499874` |
| Debug tree SHA-256 | `3dc21701d9b8111a3f66588182b3b40e41682075ed87abe94f5e62456fe42a47` |
| Complete generated tree SHA-256 | `62a780b7aca2bfb7b3ef3188805317a860f577d08d9aa57bde3dde5a23534685` |

The binary export uses Sora's deterministic Zstandard compression at level 9.
Two clean full releases and an additional verifier regeneration matched byte
for byte across the schema lock, templates, readers, binary and every debug
table.

## Row and reader proof

The debug verifier checks all 40 table files independently. For every table,
it proves:

- the table name matches the normalized file's `Gb` stable table name;
- its exact row count matches that normalized family;
- workbook-private integer keys are the contiguous sequence starting at one;
- every exported `stable_key` equals the normalized row ID at the same
  canonical ordinal.

The standalone `reader-loader` crate compiles only the generated Goal 16
reader surface, parses the committed binary through `SoraBundle`, constructs
`SoraConfig`, iterates all 40 table readers, rejects empty/non-Goal-16 tables
and proves the aggregate is exactly 10,615 rows. Its locked direct
dependencies are `serde=1.0.228` and `zstd=0.13.3`; it is an isolated QA
artifact, not a workspace or runtime dependency.

## Reproduction

```text
node tools/galactic-baseballer-reference/generate-sora-release.mjs \
  --root "$PWD" --output <new-output-directory> \
  --python /Users/mikai/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3

node tools/galactic-baseballer-reference/verify-sora-release.mjs \
  --root "$PWD" \
  --python /Users/mikai/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3
```

The verifier also reruns the complete workbook/Sora-foundation checks and the
locked standalone reader.
