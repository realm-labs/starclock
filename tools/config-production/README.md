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
  complete `openpyxl` targets for the Clara Counter charge/presence admission
  expressions and owner selector. It verifies every unowned cell's value,
  type, style, comment and hyperlink, and preserves native sheet settings.
  Review the targets and source workbook hashes before explicitly installing
  them; production workbooks are not overwritten by this command. Install
  only reviewed targets with actual authored changes; the unchanged ValueExpression
  target needs no installation. `--check-counter-guard` checks all five current
  authored rows. This is a
  bounded representative-program safety correction, not full Clara kit parity.
- `author-elation-inputs.py --write-clean <new-root>` creates complete
  `openpyxl` targets for the dedicated Elation authoring capability. It owns
  13 `policy.probe.elation` expressions (IDs 970101–970113) and the unbound
  explicit-input self-damage operation 970201. It refuses conflicting IDs,
  preserves every existing cell's value/type/style/comment/hyperlink and reports
  source SHA-256 values. Review the clean targets and recheck those hashes before
  explicitly installing only `ValueExpression.xlsx` and `Operation.xlsx`.
  It also owns the stat enum choice formula in `ModifierDefinition.xlsx` and
  `SnapshotCapture.xlsx`, copied from the current Sora templates while retaining
  all cells, validation ranges/settings and unrelated validation rules. Review
  and install those two complete clean targets under the same hash checks.
  `--check` verifies the installed rows/dropdowns without writing. These ProjectPolicy
  probes add no character identity, released binding or content coverage.

These tools provide no JSON runtime loader and do not edit `config/data` during
normal verification.
