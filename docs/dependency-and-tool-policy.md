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

The authoritative numeric dependency is `fixnum = 0.9.5`, enabled without
default features and with only `i64` and `std`. Its three transitives (`itoa`,
`static_assertions`, and `typenum`) are also recorded and locked. A fresh local
Windows x86-64 check of `starclock-combat` plus those packages took 1,161 ms;
this is a review baseline, not a performance budget.

Sora CLI 0.3.0 is a checksum-bound repository tool installed into an ignored
local tool root by `node tools/sora/install.mjs`. Its crates.io archive, release
tag object/commit, license, 102,500 ms local install observation and capability
golden are recorded separately. Sora's tool graph does not enter the production
workspace.

`G01-P1-B10` adds exact `serde = 1.0.228` and `zstd = 0.13.3` dependencies to
`starclock-data` solely because unmodified Sora 0.3.0 production readers require
them. Their 19-package direct/transitive group is reviewed in the active policy;
generated types remain private. A fresh Windows x86-64 check of
`starclock-data`, its generated reader and local dependencies took 18,782 ms.

The standalone workbook bootstrap pins `calamine = 0.35.0` and the documented
`rust_xlsxwriter = 0.96.0` with default features disabled. Its exact 47-package
lock, features, licenses and checksums are in
`policy/workbook-bootstrap-dependencies.json`. Those packages author `.xlsx`
files only and do not enter the production workspace.

## Numeric boundary

Only files under `starclock-combat/src/numeric/` may name `fixnum`. Public
consumers see Starclock's `Scalar`, `Ratio` and unit-specific wrappers, whose
constructors and accessors use signed millionths or integral fixed-width values.
No backing type, feature, generic operation trait or float conversion is public.
`G01-P2-B1` adds checked arithmetic, six explicit rounding policies and typed
numeric faults without changing the reviewed dependency graph.

## Change rule

A dependency addition or version/feature change must update the inventory,
`Cargo.lock`, compile-cost observation and deterministic-impact review. Numeric,
RNG, codec or generated-data changes also rerun their cross-platform golden
vectors. Git dependencies, floating semver requirements, dynamic plugins and
downloaded binaries without a committed checksum are rejected.
