# G19-P3-B4 — Review and Sora Foundation Audit

The seven review tables contain 1,914 source receipts, 419 content and exact-zero
pool audit rows, 1,904 exact-once coverage receipts, thirteen bounded research
policies, eleven peer reconciliation receipts, 56 semantic fixtures and
seventeen pack-file index rows. Together with the 41 gameplay tables, the four
workbooks will carry 5,934 unique rows.

Pinned `sora 0.3.0` generated the complete schema lock, four clean templates and
50 isolated Rust reader files (48 table readers plus `mod.rs` and `runtime.rs`).
An independent clean generation produced a byte-identical tree with digest
`96478ee7e015ebbeea8fcfe9be4b713a01bc19355b143fdc2c439581a02277c2`.
The generated readers remain reference artifacts and are not imported by any
runtime crate.

Focused commands:

```text
fnm exec --using 24.15.0 node tools/fate-star-rail-night-reference/verify-sora-tables.mjs --root . --batch G19-P3-B4
fnm exec --using 24.15.0 node tools/fate-star-rail-night-reference/verify-sora-foundation.mjs --root . --python python3
```

Result: 48 tables / 5,934 rows, four templates, 50 reader files and zero
regeneration drift.
