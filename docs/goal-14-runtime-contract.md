# Goal 14 Runtime Contract

This document summarizes the machine-checked contract frozen by
`G14-P0-B3`. The normative structured form is
[`policy/goal14-runtime-contract.json`](../policy/goal14-runtime-contract.json).

## Catalog and API boundary

`GoldAndGearsRuntimeFactory` is the only production assembly path. It consumes
the core combat bundle, the released shared Universe content component and the
Goal 08 Candidate Sora bundle. Generated Sora records and authoring artifacts
remain private to data loading and mode lowering.

The public mode surface is deliberately small: factory, runtime instance,
entry, catalog identity, coverage and controller identity. It reuses generic
Activity observations, commands, battle handoffs, component identities and
replay compatibility values. No workbook row, generated table type, handler
payload or lowering intermediate is public.

## State and graph model

Gold and Gears uses the existing physical scope hierarchy:

| Authored concept | Generic scope |
|---|---|
| Run | `Activity` |
| Plane | `Section` |
| Board interaction node | `Node` |
| Node visit or retry | `Attempt` |

Three logical classes retain plane-board, board-node-visit and interaction
identity across the physical micrograph. A revisit creates a fresh logical
instance. Battle and shorter scopes remain combat-owned.

Seventeen bounded slot families own entry/loadout, Neural Network, both
Conundrum tracks, Cognition, Secrets, progression, resources, Curio lifecycle,
deferred effects, plane and board state, node domain/beacon state, Knowledge,
node visits and dice resolution. Inventory ownership continues to use generic
Activity inventories.

The chessboard definition is immutable. The compiler validates a superset of
possible nodes and edges; creation, replacement, copying, blanking, collapse,
domain, beacon and Knowledge state live in bounded typed overlays. Legal route
offers filter the static edges against the current overlay. This keeps map
mutation inside `Activity::apply` without editing a live graph definition.

## Commands, events and registries

All player actions lower to the five existing graph commands:
`ChooseOption`, `StartBattle`, `SubmitBattleResult`,
`SubmitExternalOutcome` and `Abandon`. Every command binds the current state
hash and decision ID. Gold and Gears adds no command processor or mode event
log.

Authoritative changes emit existing slot, counter, inventory, modifier,
participant-carry, edge, decision, terminal or fault events. Battle requests,
accepted results and combat terminal faults cross only the declared handoff.

The Activity registry composes `starclock.activity.core` with
`starclock.mode.gold-and-gears`. The combat registry is
`gold-and-gears-combat-rules`. Both begin with zero admitted native handlers;
later admission requires a reviewed batch and ordinary typed output.

## Identity, RNG and replay

The component set contains ten consumed identities: combat, build, Activity
core, profile, Gold content, selected shared Universe content, Activity
handlers, combat rules, encounter overlay and controller. Gold and shared
content have separate digests. No unrelated mode enters the set, and the
released Standard Universe component set remains byte-identical.

The existing eight Activity RNG labels are retained. `Spawn` is the
mode-mechanic stream for a Custom Dice resolution and Knowledge caused by that
same resolution; graph, encounter, reward, shop, occurrence/policy,
external-outcome tests and battles remain isolated. Empty pools consume no
draw, and rejected transactions restore every counter.

Gold reuses the component-addressed replay envelope under
`gold-and-gears-real-battle-replay-v1`, with a distinct entry and component
set. Standard replay bytes are not migrated. Verification reconstructs every
battle and reports the first component, catalog, assembly, Activity command,
battle command, event, battle state, result or Activity-state divergence.

## Failure policy

Catalog errors fail before a run exists. Stale or invalid commands reject
without changing bytes, RNG or hashes. Deterministic execution faults use the
ordinary Activity terminal path. Nested-executor infrastructure failures
restore the pre-start Activity hash and append no report; combat-owned faults
settle only as sealed declared results. Replay verification never mutates a
live session.
