# Repository checks

The default local gate is incremental and change-aware:

```sh
node tools/repository-check/run.mjs
```

It always checks dependency direction, source/visibility policy and formatting.
For Rust changes it runs Clippy and every lib/bin harness for directly changed
workspace packages, maps those packages to the relevant centralized
`starclock-test-kit` suite, then compiles reverse dependants. The same bounded
dispatcher used by the full gate retains the smaller change-aware package
scope. Cargo uses the shared workspace `target` directory
and normal incremental compilation. A successful Rust scope is cached under
ignored `.cache/repository-check/`; an identical source/toolchain fingerprint
skips repeated Rust execution. Quick harness timings are written under ignored
`.cache/repository-check/quick-test-timings.json`.

The warm-cache budget is 180 seconds. Override it with
`STARCLOCK_QUICK_BUDGET_SECONDS`, use `--no-cache` to ignore a prior receipt,
or use `--no-budget` for a first cold bootstrap. `--all-rust` selects every
workspace package while retaining the quick target set (`lib`, `bin`, and
tests; no release benchmarks).

The quick gate deliberately defers generated-data regeneration, historical
release evidence, strict performance samples and cross-platform claims. Run
the complete cache-aware gate before a merge or whenever a deferred input is
reported:

```sh
node tools/repository-check/run.mjs --full
```

The full profile compiles all test harnesses once and dispatches the independent
binaries with bounded process-level parallelism instead of Cargo's serial
target loop. Integration behavior is concentrated in five explicit test-kit
suites (`combat`, `activity`, `universe`, `adapter`, and `exhaustive`) plus five
CLI executable-boundary tests. `STARCLOCK_TEST_JOBS` defaults to at most two
processes and `STARCLOCK_TEST_THREADS` receives the remaining CPU budget, so
parallelism happens primarily inside the coarse suites. By default the
three measured memory-heavy harnesses run first, one at a time, with the full
thread budget; the remaining harnesses then use the bounded process pool. Explicit job/thread
environment overrides disable this automatic exclusive phase. The test profile uses `opt-level=1`
because deterministic simulation/replay tests are execution-heavy; this keeps
hot loops fast without paying release-profile compile costs. Doctests remain a separate Cargo
phase. Artifact validators run in artifact-only mode during this gate, so their
embedded focused Cargo tests are not repeated after the complete workspace
suite. Standalone goal validators retain their focused tests. The former
75-integration-binary baseline is now 10 integration targets; timing reports are
written under ignored `.cache/repository-check/`.

The double-generation check for the three production Universe workbooks stores
a content-addressed receipt in the same ignored cache. It is reusable only when
the normalized data, schemas, committed exports, authoring tools, Sora binary,
loader and Python/openpyxl identity all match. Set
`STARCLOCK_NO_ARTIFACT_CACHE=1` to force the expensive regeneration.

When ignored third-party source caches are available, include their hash and
prepared-pack regeneration proof:

```sh
node tools/repository-check/run.mjs --with-source-cache
```

CI automatically selects the full profile through `CI=true`. Isolated release
acceptance sets `STARCLOCK_REPOSITORY_PROFILE=full` and still uses a fresh
target with incremental compilation disabled. Local quick/full runs never
delete `target`.

Completed Goals are checked through `policy/release-snapshots.json`. The gate
loads their status, release policy and evidence from the recorded completion
commit/tree; it does not compare historical source hashes with the evolving
working tree. Current compatibility remains owned by current tests and current
generated-data validation. Historical architecture/property/security/clean
reports and committed CI matrix evidence remain available through their
standalone verifiers, but the current repository gate does not regenerate them
from current source.
