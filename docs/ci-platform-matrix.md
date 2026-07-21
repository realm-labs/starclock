# CI Platform Matrix

[`policy/ci-matrix.json`](../policy/ci-matrix.json) is the machine-readable
platform and evidence contract. The committed workflow uses immutable action
commit SHAs, installs Rust 1.97.0 and Node 24.15.0, and retains each successful
profile's JSON evidence for 30 days.

## Native execution

| Profile | Hosted image | Native Rust host | Gate |
|---|---|---|---|
| `windows-x64-native` | `windows-2025` | `x86_64-pc-windows-msvc` | complete repository runner |
| `linux-x64-native` | `ubuntu-24.04` | `x86_64-unknown-linux-gnu` | complete repository runner |
| `macos-arm64-native` | `macos-15` | `aarch64-apple-darwin` | complete repository runner |

These jobs install the checksum-bound Sora 0.3.0 tool and call exactly
`node tools/repository-check/run.mjs`. That runner owns source limits,
dependency direction, generated drift, Sora capability goldens, formatting,
Clippy, native workspace tests and the broad release benchmark smoke ceiling.
A successful evidence record therefore sets
`execution_mode` to `native` and `tests_executed` to `true`.

Each record also lists the six executed golden suites: numeric, RNG, codec,
battle, build and replay. The committed matrix evidence binds those claims to
the exact normalized test-source hashes; the per-run record binds them to the
hosted image, workflow run and checked commit.

## Compile-only coverage

| Profile | Execution host | Checked target |
|---|---|---|
| `windows-arm64-compile` | Windows x64 | `aarch64-pc-windows-msvc` |
| `linux-arm64-compile` | Linux x64 | `aarch64-unknown-linux-gnu` |
| `macos-x64-compile` | macOS ARM64 | `x86_64-apple-darwin` |

These profiles install the target standard library and run `cargo check`.
They never run target binaries or tests, do not install Sora for the target,
and always record `execution_mode: compile-only` and `tests_executed: false`.
Compile success is not runtime, numeric-golden or compatibility evidence.
Its evidence lists every suite as `compiled-not-executed`, never `executed`.
The Linux ARM64 profile installs `gcc-aarch64-linux-gnu` because the workspace's
compression dependency compiles native target code during `cargo check`.

## Evidence boundary

`tools/ci/write-evidence.mjs` rejects a runner whose actual Rust host, Node
platform or architecture differs from the selected profile. It records the
workflow commit/run identity, hosted image metadata, Rust/Cargo/Node versions,
target, CI-policy hash and Sora golden output digest. Native records also prove
the installed Sora version; compile-only records prove that the named target
was installed.

Prepared reference JSON is forced to LF by `.gitattributes`, so its byte-level
pack digest is stable on Windows hosted checkouts. Probe generation uses only
committed fixture rows; when the ignored raw source cache is present it is
hash-checked, but its absence never blocks a clean-checkout golden run.

This batch commits the workflows and locally verifies their static contract. It
does not claim those hosted jobs have run. Goal batch `G01-P8-B2` owns retained
cross-platform numeric, RNG, codec, battle, build and replay evidence from this
matrix.
