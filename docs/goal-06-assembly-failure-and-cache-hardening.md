# Goal 06 Assembly Failure and Cache Hardening

This document is normative for `G06-P2-B4`.

## Failure boundary

Dynamic battle assembly is a read/validate/compile operation followed by one
atomic Activity publication. Failures before publication must leave the
Activity's canonical bytes unchanged. Those bytes include the pending battle,
RNG state and draw counters, replay-visible state, participant carry and all
current Standard Universe inventory.

`StandardUniverseBattleAssembler` rejects these cases explicitly:

- no prepared pending battle;
- a snapshot whose source hash is not the current Activity state hash;
- a technique definition not registered by the immutable battle template;
- an encounter or preparation variant absent from the resolved overlay;
- an invalid replacement binding;
- a contribution, carry or encounter count above the configured assembly
  budget;
- a mismatched materialization key or poisoned cache.

The pending-battle API checks for a pending battle before projecting the
snapshot. Callers therefore receive `MissingPendingBattle` independently of
which run mechanics are not yet selectable.

## Assembly budget

`BattleAssemblyBudget` bounds selected rule bindings, modifiers, participant
carry entries and compiled encounter bindings. The production defaults are:

| Dimension | Maximum |
|---|---:|
| Rule bindings | 1,024 |
| Modifiers | 64 |
| Carry entries | 8 |
| Encounter bindings | 512 |

These are service-protection limits, not gameplay caps. A release that needs a
larger validated content pack must revise the policy deliberately. Exceeding a
limit returns `BudgetExceeded`; it never truncates or partially assembles
content.

## Cache behavior

The assembly cache is bounded FIFO and keyed by the complete
`BattleAssemblyKey`. A hit can only return a materialization that validates
against that exact key. Eviction order and metrics are diagnostic state:
neither participates in battle or Activity hashes.

A cache miss may compile and cache an immutable materialization without
changing authoritative Activity state. Invalid definitions and budget
failures are rejected before cache insertion.

## Retry contract

After stale-snapshot, invalid-definition or budget failure, retrying the same
pending battle with a fresh snapshot and valid production policy must produce
the same handoff that a first successful attempt would have produced. The
failed attempt consumes no Activity RNG draw and leaves canonical state bytes
identical.

The `dynamic_battle_assembly` integration fixture freezes:

- miss-then-hit behavior for the same current snapshot;
- bounded eviction across distinct current Activity snapshots;
- exact canonical-state preservation after stale, invalid and budget errors;
- successful retry after those failures;
- execution of the returned handoff with its paired immutable combat catalog.
