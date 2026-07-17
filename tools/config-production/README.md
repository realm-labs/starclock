# Production config tooling

- `bootstrap.mjs --output <new-root>` imports frozen reference identities into
  new `.xlsx` workbooks and refuses an existing root.
- `generate-bootstrap-policy.mjs` verifies the standalone calamine/
  rust_xlsxwriter lock and license/checksum inventory.
- `verify.mjs` runs pinned Sora, double-bootstrap reproduction,
  no-overwrite/read-only-sync negatives and the generated-reader golden.

These tools provide no JSON runtime loader and do not edit `config/data` during
normal verification.
