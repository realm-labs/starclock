# Isolated audit schema and generated readers

The final schema partition adds eight review-workbook tables for mechanic
rules, sources, reconciliation, coverage, research gaps, fixtures, manifest
identity and pack index. Fixture-to-source and manifest-to-profile links use
typed Sora references.

Pinned Sora 0.3.0 generates a 37-table schema lock, exactly three templates and
an isolated Rust reader beneath `config/anomaly-arbitration-generated/`.
Regeneration compares the lock and reader bytes exactly. Sora template content
is compared member-by-member while excluding only `docProps/core.xml`, whose
creation timestamp is emitted by Sora; the openpyxl authoring step replaces
both document timestamps before canonical workbook output. No generated
reader is wired into a runtime crate.
