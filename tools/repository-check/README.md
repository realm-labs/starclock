# Repository checks

Run the complete pinned local gate from the repository root:

```sh
node tools/repository-check/run.mjs
```

The runner verifies the reviewed dependency/tool inventory, CI platform and
evidence contract, crate dependency direction, handwritten Rust line limits,
deliberate public re-exports, generated-artifact drift, formatting, Clippy and
all workspace tests. The Rust
source policy excludes generated or vendored trees only when their exact path,
kind and reason are committed in `policy/repository-checks.json`.

Phase 0's ignored third-party source caches are not clean-checkout inputs. When
those caches are present, extend the same gate with their hash and full prepared
pack regeneration proof:

```sh
node tools/repository-check/run.mjs --with-source-cache
```

CI calls this runner rather than copying its command list. Native and
compile-only target claims are defined in `policy/ci-matrix.json` and documented
in `docs/ci-platform-matrix.md`. Later batches
extend `policy/generated-drift.json` as Sora schemas and golden artifacts land.
