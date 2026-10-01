# Production config tooling

- `bootstrap.mjs --output <new-root>` imports frozen reference identities into
  new `.xlsx` workbooks and refuses an existing root.
- `author-workbooks.py --write|--check` owns the three current
  `ConfigManifest.xlsx` values and keeps the empty Native Handler workbook
  identical to its current schema template.
- `generate-bootstrap-policy.mjs` verifies the standalone calamine/
  rust_xlsxwriter lock and license/checksum inventory.
- `verify.mjs` runs pinned Sora, double-bootstrap reproduction,
  no-overwrite/read-only-sync negatives and compares rebuilt readers/exports
  directly with the current committed generated output.
- `character-probe-promotion.py --write-counter-guard <new-root>` creates
  complete `openpyxl` targets for the two owned Clara Counter admission
  expressions and verifies every unowned cell's value, type and style.
  Review the targets and source workbook hashes before explicitly installing
  them; production workbooks are not overwritten by this command.
  `--check-counter-guard` checks the current authored expressions. This is a
  bounded representative-program safety correction, not full Clara kit parity.

These tools provide no JSON runtime loader and do not edit `config/data` during
normal verification.
