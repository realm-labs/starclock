# Starclock Agent Guidelines

This file applies to the entire Starclock repository. A more specific
`AGENTS.md` may add requirements for its subtree, but it must not weaken this
root contract.

Normative design lives in `docs/`. Starclock maintains only the current source,
data, behavior and generated state summary. Git history is the sole
historical record; the working tree does not preserve completed-goal evidence,
old formats or migration contracts.

## 1. Working Agreement

- Prioritize correctness, determinism, readability, maintainability and
  testability.
- Inspect the relevant design document and existing implementation before
  changing code or data.
- Respect unrelated uncommitted work. Do not overwrite, revert, reformat or
  opportunistically clean up user changes.
- Keep one responsibility-bounded change active. Do not broaden a documentation,
  research or diagnostic request into runtime implementation.
- Use `rg` and `rg --files` for repository search when available.
- Prefer small, reviewable patches. Use `apply_patch` for handwritten files and
  the owning generator for generated artifacts.
- Never hand-edit generated Rust, schema locks, Sora exports, coverage reports
  or other files whose owning generator is documented.
- Concurrent changes use separate branches/worktrees and isolated artifact
  paths. Stop and reconcile shared-file conflicts rather than overwriting
  another task.

## 2. Architecture Ownership

Preserve the dependency and responsibility boundaries defined by
`docs/06-rust-architecture.md`:

- `starclock-combat` owns exactly one battle: resolved combatants, formulas,
  timeline, actions, effects, enemies, waves, events and battle RNG.
- `starclock-build` compiles character progression, equipment and loadout
  definitions into immutable generic `ResolvedCombatantSpec` values.
- `starclock-activity` owns cross-battle graphs, scopes, participants, decisions,
  inventories, clocks, metrics, persistence and `BattleSpec`/`BattleResult`
  handoff.
- `starclock-data` loads Sora bundles and converts generated records into
  validated Starclock-owned immutable definitions.
- `starclock-rules` owns the bounded static registry for exceptional battle and
  activity handlers.
- Mode crates contribute profiles, definitions, generic operations and
  mode-owned handler bundles. They do not fork command processing, formulas,
  timelines, graph execution, RNG, replay or hashing.
- CLI, MCP, AI and presentation adapters depend on domain crates. Domain crates
  never depend on adapters, Bevy, workbooks, wall-clock time or platform UI.

Do not:

- add content IDs, character IDs or mode IDs to shared resolver branches;
- introduce a second battle/activity state machine for a mode;
- let combat query build catalogs, account inventory or progression graphs;
- let a battle mutate live Activity state or an Activity mutate live battle
  state;
- add global mutable registration, runtime scripting or filesystem-discovered
  handlers;
- create cyclic crate/module dependencies to hide misplaced ownership.

Move a shared concept to its lowest truthful owner and keep mode-specific
terminology in the mode layer.

## 3. Commands, State and Determinism

- All battle mutation passes through accepted commands and typed operations.
  All Activity mutation passes through `ActivityCommand` and typed activity
  operations.
- Rejected commands leave authoritative state byte-identical. Accepted commands
  either commit ordered events/operations or enter a documented deterministic
  fault state.
- Every meaningful mutation is represented by a domain event. Triggers enqueue
  bounded reactions; they do not recursively mutate arbitrary state.
- Trigger phase, priority, cause ownership, snapshot policy and once-scope must
  be explicit whenever they affect behavior.
- Authoritative arithmetic uses project domain types over the pinned fixed-point
  backend. Do not use `f32`, `f64`, unchecked overflow, implicit rounding or
  generic approximate comparison in authoritative paths.
- Use named checked arithmetic and explicit rounding at documented formula
  boundaries. Saturation/clamping is allowed only when the rule declares it.
- All randomness uses project-owned labeled RNG streams, integer sampling and
  stable ordered candidate sets. Do not use thread/system RNG, generic shuffle,
  floating probability draws or collection iteration order.
- `HashMap`/`HashSet` may be used for lookup, but their iteration order is never
  authoritative. Sort emitted work by a stable fixed-width domain key.
- Do not serialize or hash `usize`, addresses, platform time, thread scheduling
  or filesystem enumeration order.
- Canonical codecs and state/config hashes describe only the current tree and
  may change intentionally with the implementation. Update current goldens in
  the same change. Do not add compatibility migrations, legacy decoders,
  revision branches or historical golden evidence.

The normative rules are in `docs/09-determinism-and-numerics.md`,
`docs/10-lifecycle-and-resolution.md` and
`docs/16-replay-cli-and-engine-integration.md`.

## 4. Configuration and Content Data

- Excel `.xlsx` workbooks are the editable production authoring surface.
- Sora 0.3.0 is the only schema validation, code-generation and production
  export authority.
- JSON may be deterministic research/bootstrap or debug output. Runtime code
  must never load normalized JSON or Excel directly.
- Edit production workbooks only through the documented Python `openpyxl`
  authoring path. Generate complete clean targets; do not patch an `.xlsx` as a
  ZIP or overwrite a designer-edited workbook implicitly.
- Preserve canonical decimal strings. Do not round authoritative values through
  JavaScript numbers, Python floats or Excel floating cells.
- Schema, workbook, generated readers, debug export and binary `.sora` bundle
  changes travel together and pass deterministic drift checks.
- Stable Starclock IDs are runtime/content identity. Upstream numeric IDs remain
  source locators unless an explicit contract says otherwise.
- Shared content requires proven profile reachability. Source-table reuse,
  matching names or adjacent ID ranges are not membership evidence.
- Runtime configuration identity binds the exact current inputs needed for
  deterministic execution; it is not a compatibility promise for old inputs.

Generated or raw source material belongs only in its declared cache/output
boundary. Do not commit bulk upstream prose, assets, ability programs or source
repositories.

## 5. Current Content Authoring

- Use released/public evidence only. Reject leaks, beta dumps, previews,
  NDA-bound material and announced-but-unavailable content.
- Evidence priority is pinned released structured data, official released text,
  reproducible observations, then independent public cross-checks.
- Every factual row records its exact source repository revision or URL/access
  date, game version, path/page, row locator, evidence digest, quality and note.
  These locators are current gameplay provenance, not project-version history.
  Do not build historical project release-evidence packages or completion
  snapshots in the repository.
- Preserve exact factual values and relationships. Write short independent
  bilingual summaries instead of copying long descriptions or dialogue.
- Approximation is field-level and explicit. Record the unavailable fact,
  selected deterministic policy, alternatives, rationale, affected tests,
  confidence and replacement condition.
- Never present a `ProjectPolicy` or inferred behavior as observed parity.
- Completeness comes from current reference manifests with exact-once
  accounting, not historical Goal receipts, Wiki totals or a large source table.
- Keep content ownership, runtime disposition, accuracy, provenance and
  coverage as separate dimensions.
- Story presentation, account rewards, achievements and assets remain excluded
  unless the current task explicitly includes a mechanically relevant locator.

Follow `docs/sources.md`,
`docs/content-reference/authoring-contract.md` and
`docs/15-content-data-and-coverage.md`.

## 6. Rust Structure and File Size

- A handwritten `.rs` file must not exceed 1,200 physical lines without a
  specific documented exception. Begin planning a split around 800 lines.
- `lib.rs` and `mod.rs` should normally stay below 200 lines and contain
  declarations and a small deliberate facade, not implementation bulk.
- Generated, vendored and mechanically produced files are exempt only when the
  path/header makes ownership obvious.
- Split by responsibility, not arbitrary line ranges. Avoid catch-all modules
  such as `common`, `utils`, `helpers`, `manager` or `misc`.
- Keep tests close to the responsibility they verify; move large integration
  behavior to crate-level `tests/`.
- Do not use `super::super` or deeper parent-relative paths. Cross two or more
  module boundaries through a stable `crate::` path at the import site.
- Put stable module-path qualification in clear `use` declarations near the
  top of a module, then use the imported local names in signatures and bodies.
  This applies to inline `crate::...`, `super::...` and `self::...` paths; a
  `use super::...` or `use self::...` declaration is fine, but the qualified
  path should not be repeated at a call site or in a type signature merely to
  avoid an import.
- Resolve import-name conflicts with descriptive aliases. Keep an inline path
  only when the source distinction must remain visible and an alias would make
  that distinction less clear, or when Rust requires a path for a visibility
  boundary such as `pub(in crate::...)`.
- Use domain newtypes and enums to prevent illegal/interchangeable states.
  Avoid untyped string/value maps when legal keys and operations are known.
- Keep pure formulas separate from mutation, event collection, reaction
  scheduling and adapter code.

## 7. Visibility, APIs, Errors and Lints

- Use the narrowest visibility: private, then `pub(super)`, `pub(crate)`, and
  finally `pub` for a genuine external API.
- Do not use `pub use` by default. It is allowed only for a small intentional
  facade, a private-layout abstraction or a generated integration requirement.
- List public re-exports explicitly. Do not use wildcard re-exports, project
  preludes or re-export chains merely to shorten paths.
- Public APIs document invariants, timing, ownership and failure behavior.
- Invalid commands, authored data and replay inputs return typed errors.
- Production code must not `unwrap()` or `expect()` recoverable external or
  state-dependent conditions. An invariant `expect()` explains the invariant.
- Unsafe Rust is forbidden by the workspace lint policy.
- Fix Clippy findings. Do not hide them with macros, cfg tricks, lowered lint
  levels or broad `allow`/`expect` attributes.
- A local lint suppression is allowed only for a concrete false positive or
  external/protocol constraint, at the smallest item, with an adjacent reason.
- New dependencies require an exact pin, license/tool policy update, dependency
  direction review and deterministic/compile-cost assessment.

## 8. Tests and Verification

Tests must be proportional to the change:

- formulas use table-driven boundary vectors;
- state machines cover valid and rejected transitions;
- bug fixes include regression tests;
- cross-module behavior uses integration tests;
- RNG and codecs use stable golden vectors;
- replay/golden tests bind current configuration digests and compare current
  canonical hashes from one shared state manifest;
- Sora readers load real generated bundles;
- tests do not depend on wall clock, unseeded randomness, filesystem order or
  thread scheduling.

Node and Python are data-authoring tools, not Rust test orchestrators. When a
data validator is needed, use the toolchain pinned by that validator.

Run focused Rust tests during development:

```text
cargo test -p <affected-package> [test-filter]
```

Before completing an ordinary Rust change, run Cargo directly for the affected
package:

```text
cargo fmt --all -- --check
cargo clippy -p <affected-package> --all-targets -- -D warnings
cargo test -p <affected-package>
```

Do not run the whole workspace repeatedly after a responsibility-local edit.
CI runs `cargo test --workspace`. Run it locally only for shared-boundary
changes, before merge when requested, or when focused results indicate wider
impact.

Run current Sora/workbook/data validators only when their owned inputs change.
Large seeded matrices, property corpora and performance workloads are explicit
exhaustive checks; they do not block the default edit-test loop.

Run the Rust exhaustive suite explicitly with:

```text
cargo test -p starclock-test-kit --features exhaustive --test exhaustive_suite
```

Documentation-only changes do not require Rust compilation, but applicable
links and document-specific validation must still pass.

Never claim an unexecuted check passed. If a toolchain, dependency or external
source prevents a required command, report the exact command, error and
substitute checks.

## 9. Current-State Maintenance

- `docs/state.md` is the human-readable summary of the current tree.
- `policy/state.json` is its machine-readable counterpart and contains
  no schema version, Goal ID, completion batch or historical result.
- Update current code, data, tests, goldens and the state summary
  together when their facts change.
- Replace obsolete current values instead of retaining legacy branches or
  migration paths. Git history records what changed.

## 10. Git, Commits and Cache Safety

- Do not stage, commit, push, branch, delete or rewrite history unless the user
  explicitly authorizes it.
- Every commit follows Conventional Commits:
  `<type>(<optional-scope>): <imperative lowercase description>`.
- Keep commits focused. Do not mix unrelated formatting, refactoring,
  documentation or generated drift.
- Never use destructive Git commands to discard user work.
- Preserve the shared workspace `target` directory and incremental caches.
  Do not use unscoped `cargo clean` as routine maintenance.
- CI, benchmarks, coverage and incompatible build classes use their documented
  isolated target/cache paths.
- Never delete the global Cargo registry/Git cache, a user home directory,
  arbitrary `CARGO_TARGET_DIR`, raw source cache or historical release evidence
  through generic cleanup.
