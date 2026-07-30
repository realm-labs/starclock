# Currency Wars audit schema and generated reader

Batch `G12-P3-B4` completes the independent Sora schema with eight review
tables for sources, exact-once coverage, research gaps, semantic fixture
families, review cases, reconciliation receipts, normalized manifest and pack
index.

The full project has 102 unique tables partitioned exactly across:

- 60 sheets in `CurrencyWars.xlsx`;
- 32 sheets in `CurrencyWarsBindings.xlsx`; and
- ten sheets in `CurrencyWarsReview.xlsx`.

Checksum-bound Sora 0.3.0 generates the schema lock, the three workbook
templates and a Rust reader only below `config/currency-wars-generated/`.
The reader is an isolated generated artifact and is not added to the workspace,
linked into a runtime crate or used to lower handlers.

The batch verifier regenerates the lock, workbook list and Rust reader in an
ignored Goal-specific scratch directory. Schema-lock and reader bytes must
match exactly; workbook ZIP bytes are intentionally not compared because Sora
0.3.0 writes variable archive metadata. P3-B5/P3-B6 perform semantic and visual
workbook verification.

The generated schema-lock digest is
`4f9d91b7ed5b28c838fc4f0c1078c147f24d6b0cba32889c62e0e4713839e484`;
the canonical generated-reader tree digest is
`bf3a5dccd6beb8a0a4b4b7e00c327806d8ca695025fde0f00147a464ba695021`.
