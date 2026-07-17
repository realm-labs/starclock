# Dependency and Tool Policy

The machine-readable active inventory is
[`policy/dependency-and-tool-policy.json`](../policy/dependency-and-tool-policy.json).
Every package and executable introduced by an implementation batch must be added
there with an exact version, license, purpose, deterministic impact, compile-cost
record and rejected alternatives in the same commit.

## Active baseline

Rust/Cargo 1.97.0, rustfmt 1.9.0, Clippy 0.1.97 and Node 24.15.0 are pinned by
`rust-toolchain.toml` and `.node-version`. The workspace uses resolver v3 and a
committed Cargo lockfile.

The only registry dependency is `fixnum = 0.9.5`, enabled without default
features and with only `i64` and `std`. Its three transitives (`itoa`,
`static_assertions`, and `typenum`) are also recorded and locked. A fresh local
Windows x86-64 check of `starclock-combat` plus those packages took 1,161 ms;
this is a review baseline, not a performance budget.

Sora CLI 0.3.0 is a checksum-bound repository tool installed into an ignored
local tool root by `node tools/sora/install.mjs`. Its crates.io archive, release
tag object/commit, license, 102,500 ms local install observation and capability
golden are recorded separately. Sora's tool graph does not enter the production
workspace. The generated-reader golden has its own exact 19-package lock and
license inventory; its 6,264 ms fresh check is fixture evidence, not approval to
add `serde` or `zstd` to `starclock-data` before `G01-P1-B10`.

## Numeric boundary

Only `starclock-combat/src/numeric.rs` may name `fixnum`. Public consumers see
Starclock's `Scalar` and `Ratio` wrappers, whose constructors and accessors use
signed millionths. No backing type, feature, generic operation trait or float
conversion is public. Checked arithmetic and complete domain wrappers remain
owned by Phase 2; this batch establishes representation privacy without
preempting their formula policy.

## Change rule

A dependency addition or version/feature change must update the inventory,
`Cargo.lock`, compile-cost observation and deterministic-impact review. Numeric,
RNG, codec or generated-data changes also rerun their cross-platform golden
vectors. Git dependencies, floating semver requirements, dynamic plugins and
downloaded binaries without a committed checksum are rejected.
