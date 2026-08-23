# Starclock cross-platform evidence

The pinned CI workflow has one full Linux current-tree job and one Currency
Wars native matrix. The current-tree job runs formatting, workspace Clippy and
all workspace tests on `ubuntu-24.04` with Rust 1.97.0.

The Currency Wars matrix executes release binaries on Windows x64
(`windows-2025`), Linux x64 (`ubuntu-24.04`) and macOS ARM64 (`macos-15`). Each
job runs the baseline/replay suite, the complete generated legal matrix and the
frozen native evidence verifier for its exact target. The verifier compares
canonical run, replay, verification, matrix, runtime-contract and exact-coverage
hashes with `policy/currency-wars-native-evidence.json`.

Windows ARM64, Linux ARM64 and macOS x64 are paired compile-only targets. Each
native job runs `cargo check --workspace --all-targets` for its paired target.
This proves compilation of workspace targets and test sources, but does not
claim target-binary execution, replay or numeric parity.

Local macOS ARM64 evidence is committed under
`evidence/currency-wars-runtime-v1/`. Hosted run IDs remain CI-owned evidence;
they are not fabricated in the repository. Verify the local frozen boundary
with:

```text
node tools/currency-wars-runtime/native-evidence.mjs \
  --target aarch64-apple-darwin \
  --check \
  --output evidence/currency-wars-runtime-v1/native-local-macos-arm64.json
```
