# Current-tree checks

Rust tests are built and executed by Cargo directly. The repository does not
maintain a second JavaScript test scheduler, historical Goal gates or release
snapshot verification.

During development, run the affected package and an optional test filter:

```sh
cargo test -p starclock-combat shield
```

Before completing an ordinary Rust change, validate the affected package:

```sh
cargo fmt --all -- --check
cargo clippy -p <affected-package> --all-targets --all-features -- -D warnings
cargo test -p <affected-package>
```

CI runs `cargo test --workspace`. Use that locally only for shared-boundary
changes or an explicit merge check.

When configuration, workbooks, generated readers or current content data
change, also run:

```sh
node tools/repository-check/verify-data.mjs
```

Large seeded matrices, broad property corpora and performance workloads are
explicit exhaustive checks. They are not part of the default edit-test loop.
Run the current seeded matrices explicitly with:

```sh
cargo test -p starclock-mode-universe seeded_run_tests::frozen_matrix -- --ignored
```
