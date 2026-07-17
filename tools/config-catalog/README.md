# Catalog fixture verifier

`verify.mjs` materializes two fresh 80-workbook roots from the representative
fixture rows, exports both with pinned Sora 0.3.0 and compares their binary and
diagnostic outputs with the committed golden. Use `--bless` only when a reviewed
schema or fixture change intentionally changes the expected bundle.
