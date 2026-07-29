# Anomaly Arbitration Sora Release and Visual Review

Goal 13 batch `G13-P3-B6` validates the complete reference-only authoring
surface. It does not admit any table to the Starclock runtime.

## Sora build and load

- Checksum-bound Sora 0.3.0 accepts all four schemas and all three workbooks.
- Two clean workbook generations remain byte-identical.
- Binary and JSON-debug exports regenerate byte-identically.
- `config.sora` SHA-256:
  `a646b66ad0eae515a624d838ea8574f52c6e40588f88127f281b9cd8c40f89f1`.
- Schema fingerprint: `740fa7f2e0010cff`.
- The debug export contains exactly 37 table files and 2,103 rows.
- The isolated generated-reader loader parses the binary and loads every one
  of the 37 generated tables, with 2,103 rows in aggregate.

Actual data build found and corrected two schema-only defects:

1. The ownership enum used `AA` while the frozen normalized vocabulary is
   `AnomalyArbitration`.
2. Sora 0.3.0 generates inconsistent Rust type spellings for leading
   all-capital logical prefixes (`AAOwnership` versus `AaOwnership`). The
   isolated logical prefix is therefore `Arb*`: it remains safely below
   Excel's 32-character validation-title limit and its generated reader
   compiles without hand edits.

The six exact-zero pool audits use
`optional<list<string>>` for `manifest_record_ids`, because their generated
zero proofs intentionally have no active manifest record IDs. Their canonical
payloads and source receipts remain complete.

## Rendered inspection

Disposable review copies expose rows 1–5 of every nonempty table, preserve all
metadata/header rows, and set one landscape fitted print page per sheet.
LibreOfficeDev 26.8 exported:

| Workbook | Pages | PDF SHA-256 |
|---|---:|---|
| `AnomalyArbitration.xlsx` | 17 | `9a512c5f725020ce3195c156cac5042c22eaa543d044904ff083e2d2c2510e1d` |
| `AnomalyArbitrationBindings.xlsx` | 12 | `bb9b212280ab0d4a37d445f1625768325d6c69c8db7c18d9519b3f20091f1211` |
| `AnomalyArbitrationReview.xlsx` | 8 | `924ebdde634ed25612f306e392854fb28173f7765ecd6f775e5160fa431b4e2c` |

`pypdfium2 5.12.1` rendered all 37 pages and Pillow 12.2.0 composed five
contact sheets. Manual inspection of every contact sheet found:

- all metadata, field and type rows visible;
- all inspected data rows aligned under their headers;
- no overlapping cells, clipped table boundaries, unexpected blank sheets or
  extra pages;
- consistent dark-blue headers, alternating data rows and readable wrapping.

The rendered PDFs and PNGs are disposable QA derivatives under
`/tmp/starclock-g13-visual-review-20260730`; their page counts and hashes above
are the retained audit receipt.
