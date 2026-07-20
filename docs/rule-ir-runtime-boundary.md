# Typed Rule IR runtime boundary

Goal 01 batch `G01-P4-B1` replaces the combat catalog's identity-only rule and
program placeholders with an executable, immutable battle Rule IR. The Sora
transport remains unchanged and production data still enters only through the
generated-reader/domain-lowering boundary.

## Domain model

`starclock-combat::rule` owns closed domain values for:

- generic source class, source identity, ordered tags and source digest;
- integer, fixed scalar, boolean, stable-ID, optional-ID and ordered-ID-set
  values;
- battle/wave/turn/action/hit slots with bounds, visibility, persistence and
  explicit reset points;
- event families, independent trigger phases, signed reaction priorities,
  cheap cause-role filters and contextual conditions;
- event/cause/slot/selector reads, checked arithmetic, comparison, choice,
  clamp, negate and explicit integer/scalar conversion with named rounding;
- finite ordered operation, `If` and bounded `ForEach` program steps; and
- per-event, hit, target-within-hit, ability, action, turn, wave and battle
  once scopes.

Programs currently emit only the foundation operations owned by this batch:
typed slot set/add proposals, informational rule facts, replacement proposals
and validated native-handler invocations. Damage, modifier, effect, timeline,
summon and encounter operation payloads are added by their owning Phase 4
batches rather than frozen prematurely here. Every emission is a resolver
proposal; Rule IR never receives mutable battle state.

## Catalog validation and indexes

The combat catalog rejects zero source digests, unordered tags/slots/triggers,
mistyped initial values or bounds, malformed explicit-reset slots, expression
depth above 64, missing selectors/slots/programs, heterogeneous arithmetic or
comparisons, unsupported conversions, `ForEach` bounds outside `1..=64`,
undeclared handler invocation and replacement programs that reach an ordinary
mutation. Structured child-program references are incorporated into the
existing deterministic cycle check.

Catalog construction compiles immutable trigger groups by event and phase.
Within a group, definition order is priority, source ID, rule ID and trigger
ID. Runtime reaction keys append side, formation, spawn, rule-instance and
insertion identities, so no comparison depends on container layout.

## Read-only evaluation and once scopes

Evaluation accepts only an immutable event/cause projection, ordered source
tags, ordered slot values and already-resolved ordered selector results. It
performs checked arithmetic and returns typed proposals. Missing data, type
errors, numeric errors, invalid conversions and budget exhaustion are stable
numeric error categories.

The evaluator counts steps, emissions and bounded iterations independently.
`TriggerLedger` stores canonical `OnceKey` values. It performs event/filter/
condition matching and complete program evaluation before committing a key;
therefore a failed or budget-exhausted evaluation cannot consume the trigger's
once scope. A missing hit, target, action, turn or wave identity is rejected
instead of being coalesced into a convenient default.

The current battle fixtures bind no executable Rule IR, so existing Phase 3
hashes are intentionally unchanged. `G01-P4-B2` is the first production-path
consumer: it binds rule instances and the Asta modifier probe while retaining
this read-only proposal boundary.

## Native handler registry

`starclock-rules` contains an immutable versioned registry over sorted static
function registrations. A battle handler receives the same read-only
evaluation input plus validated typed arguments and returns ordinary
`RuleEmission` values. It has no mutable battle handle, dynamic library name,
filesystem/network/time access or global mutable registry.

An enabled authoring requirement must match handler domain, stable key, version,
argument-schema digest, determinism note, owner, written IR-insufficiency
decision and removal condition exactly. B1 registered no production/content
handler; its only implementation was a test-local synthetic echo used to prove
that native and equivalent IR produce the same emission shape. `G01-P4-B10`
completed the manifest-wide V1a review: all eight probe scopes remain
expressible through typed IR, the production registry is explicitly empty, and
the universal repository gate rejects scattered content-ID branches. See the
[native-handler and content-branch audit](native-handler-and-content-branch-audit.md).

## Focused evidence

- `cargo test -p starclock-combat --test rule_ir_contract`
- `cargo test -p starclock-rules --all-targets --all-features`
- `cargo check -p starclock-combat -p starclock-rules --all-targets --all-features --target aarch64-pc-windows-msvc`
- `node tools/repository-check/verify-source-policy.mjs`

The focused suites cover typed catalog rejection, deterministic trigger index
order, ordered `ForEach` output, every once scope, cause-role separation,
budget-fault non-mutation, registry audit and IR/native equivalence.
