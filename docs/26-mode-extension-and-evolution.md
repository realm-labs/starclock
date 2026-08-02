# Mode extension

This document defines the current extension boundary for adding gameplay modes.
It does not promise compatibility with earlier source trees or serialized data.

## Core guarantees

Every mode uses the shared domain kernels and preserves:

- command-based mutation and byte-identical rejection;
- checked fixed-point authoritative arithmetic;
- project-owned deterministic RNG and stable ordering;
- immutable validated definitions;
- typed operations and bounded static native handlers;
- separation of combat, build compilation, activity orchestration, adapters and
  presentation;
- no mode-ID or content-ID branches in shared resolvers.

## Ownership

A mode owns its profile, data definitions, generic operation programs and static
handler bundle. Shared mechanics move to the lowest truthful shared crate. A mode
must not fork combat command processing, formulas, timeline, Activity graph
execution, RNG, replay or hashing.

Exceptional behavior is registered through immutable bundles composed before a
run starts. IDs and schemas must be unique and deterministic. Runtime scripting,
filesystem discovery, dynamic libraries and global mutable registration are not
supported.

## Activity tasks and scopes

`BattleSpec` and `BattleResult` are the combat handoff. Activity owns ordered,
bounded child tasks and verifies a result against the task that produced it
before committing state.

Physical lifetimes are:

```text
Activity -> Section -> Node -> Attempt
```

Mode names such as Plane, Stage, Round, Domain or Room are bounded logical scopes
owned by a physical lifetime. Each scope has a stable ID, parent, reset/carry
rules and limits. New terminology does not justify hard-coding another core
lifetime.

## Configuration identity

The current runtime identity is derived from the components consumed by the
selected entry: combat, build, Activity core, mode content partitions and the
composed rule registry. Hashes and replay fixtures describe only the current
tree and may be replaced atomically when current behavior changes.

Production content remains Excel-authored, Sora-validated and loaded from Sora
bundles. JSON is permitted for deterministic research, staging and diagnostics,
not as a parallel production runtime format.

## Adding a mode

Add the smallest set of current artifacts needed by the mode:

1. gameplay reference material and source locators;
2. workbook/schema definitions and Sora conversion;
3. mode profile and generic operation programs;
4. a static handler bundle only for mechanics that cannot be expressed by the
   generic model;
5. focused tests, including deterministic replay when the mode exposes replay;
6. current documentation and current-state coverage.

Do not add release ledgers, migration code, legacy decoders, version-selection
branches or archived validation receipts. Git records earlier project states.
