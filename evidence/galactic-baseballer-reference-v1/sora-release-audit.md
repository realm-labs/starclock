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
| Bundle bytes | 741,194 |
| Bundle SHA-256 | `82e600ee2b8aaaaeada82810f223d4f45e193833eb4151c939a5e51950f78848` |
| Reader tree SHA-256 | `f568092007c1a3e3b7618b62527e5981a0d4c7480104be954da0380d8911697e` |
| Debug tree SHA-256 | `37c7819de3de866be87732f8a7d81aafc753cf79a12e77fdbe26dbac27fc795a` |
| Complete generated tree SHA-256 | `fad82dc324258451c71039130f5f5043b12360efd80716d81e02ed4ffaf18e03` |

The binary export uses Sora's deterministic Zstandard compression at level 9.
Two clean full releases and an additional verifier regeneration matched byte
for byte across the schema lock, templates, readers, binary and every debug
table.

P4-B2 regenerated the release after making the shared-base field required on
both Profile rows. The current fingerprints above supersede the P3-B4
Candidate-preparation fingerprints; table and row denominators are unchanged.

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
