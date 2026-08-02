# CI verification

CI validates only the current Rust tree. It does not reconstruct completed
goals, compare old release snapshots, upload evidence receipts or execute a
cross-platform compatibility matrix.

The workflow runs Cargo directly:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Focused development uses `cargo test -p <package> [filter]`. Current workbook,
Sora and reference-pack validators are run explicitly when their owned data
changes. Exhaustive seeded matrices, property corpora and benchmarks are also
opt-in checks, not default CI stages.
