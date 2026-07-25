# Goal 07 shared capability and content gates

`G07-P1-B6` closes the shared-mechanic phase and establishes the evidence
boundary for all 104 generated content partitions. It does not claim that a
Standard Universe record is implemented. It proves that the reusable combat
capabilities needed by those records are executable and defines what a later
content commit must prove.

## Shared capability matrix

The machine matrix in `policy/goal07-shared-capability-gate.json` binds each of
the five Phase 1 capability families to:

- its accepted policy and focused verifier;
- named tests that execute authoritative state/event behavior;
- formal `.xlsx` probes where a new configuration transport boundary was
  introduced; and
- the corresponding Sora diagnostic rows.

There are 15 runtime probe markers and six formal Excel rows. The Excel rows
cover selector predicates, an authored strongest comparator, a lifecycle slot
reset, forced Break, Action delay and a persistent extra turn. They are
checked using their Python `openpyxl` author scripts and pinned Sora 0.3.0
output. Trigger timing is exercised directly through production Rule IR and
does not need a behavior-neutral configuration row.

These probes close reusable capability only. They do not promote any of the
2,201 content records or 786 Standard Universe rules.

## Native-handler admission

The current `native-registry-v1` remains explicitly empty. A historical
`StaticNativeHandler` planning label or `native_review_candidate` flag is not
an admission.

A later partition may admit at most one handler, and only after recording all
of the following in the static audit and formal Sora row:

1. stable ID/key, battle/activity domain and version;
2. canonical argument-schema digest;
3. determinism note and owning partition;
4. a written reason the typed IR is unreasonable or insufficient;
5. a removal condition;
6. source and runtime evidence; and
7. an equivalence fixture proving the handler emits ordinary typed operations.

Compiled registration metadata and enabled Excel/Sora metadata must match
exactly. The handler cannot mutate state, emit private events or call RNG
outside the supplied deterministic interfaces.

## Partition completion receipt

The frozen assignment manifest remains immutable. Completion state is derived
separately from ordered receipts under
`evidence/standard-universe-mechanics-complete-v1/partitions/`.

Each receipt must exactly cover the partition's assigned:

- content records;
- mechanic rules;
- semantic fixtures;
- enemy variants; and
- encounter members.

Every entry records its terminal runtime and accuracy disposition, workbook
row evidence and provenance evidence. Rules additionally name formal
definition keys and runtime execution evidence. Assigned semantic fixtures
must name a real Rust/CLI/replay/scenario golden and a marker present in that
artifact. The authoring section binds the edited `.xlsx` files, explicit
`openpyxl` commands, Sora bundle and Sora golden by SHA-256.

The following claims are nonterminal and are rejected:
`TypedEvaluator`, `TypedPlan`, `WorkbookOnly`, `RouteOnly`,
`EffectPlanOnly`, `RetainedApproximation` and
`StaticNativeHandlerCandidate`.

Receipts are accepted only in manifest order. A later receipt cannot exist
before its dependency. This prevents parallel placeholder work from creating
false aggregate completion.

## Progress surfaces

The assignment ledger continues to show the frozen 104-batch plan.
`tools/goal07/generate-content-progress.mjs` derives a separate JSON and
Markdown progress ledger from validated receipt presence. The focused
partition verifier is:

```text
node tools/goal07/verify-content-partition.mjs --partition <batch-id>
```

`G07-P1-B6` verifies the first partition with `--expect-pending`; every later
partition replaces that state with a complete receipt and regenerates the
progress ledger in the same atomic commit.

## Focused verification

```text
node tools/goal07/verify-phase1-b6.mjs
node tools/repository-check/verify-native-handlers.mjs
node tools/goal07/generate-content-progress.mjs --check
node tools/goal07/verify-content-partition.mjs --partition G07-P2-M01-S01 --expect-pending
```
