# Standard Universe runtime interface

This document describes the interface implemented by the current tree. It is
not a compatibility or migration contract for old binaries, replays, hashes or
API payloads.

## Ownership

| Owner | Responsibility |
|---|---|
| `starclock-activity` | Generic graph definitions, authoritative Activity state, commands, decisions, events, battle handoff and read-only views |
| `starclock-mode-universe` | Immutable Universe catalogs, validated bundle lowering, mode profiles, battle assembly and mode-owned handlers |
| `starclock-data` | Core combat/build bundle loading and conversion into Starclock-owned definitions |
| `starclock-replay` | The single canonical replay header, records, encoding and verification |
| `starclock-ai` | Deterministic selection from exactly offered commands |
| adapters | Session ownership, opaque action binding, transport and presentation |

`starclock-combat` does not depend on Universe mode code. Mode crates compile
content into shared Activity and combat operations; they do not own alternate
state machines, RNG implementations, replay formats or hash algorithms.

## Activity boundary

An Activity exposes exactly one authoritative boundary after construction and
after every accepted command:

- a decision with canonically ordered exact offers;
- an immutable battle handoff;
- or a terminal outcome.

Commands carry the expected state hash and the identity of an exact current
offer. Callers cannot submit arbitrary currency changes, inventory values, RNG
results, enemy identities, graph destinations or rule programs. Rejected input
does not change state, RNG counters, command sequence or the offered boundary.

Automatic programs settle inside one transaction until the next external
boundary. Adapters never drive hidden intermediate operations.

## Events and failures

Committed mutations produce ordered typed events for lifecycle, decisions,
state, inventory, graph traversal, battle handoff/results, RNG audit and
terminal settlement. Every event has an `ActivityCause` identifying the
command and relevant definition, node, attempt, source, option or battle.

Invalid external input returns a typed rejection. Evaluation, overflow or
invariant failures either roll back before mutation or commit the documented
deterministic fault state. Undocumented partial state is forbidden.

## Catalog and configuration identity

Production loading accepts validated `.sora` bundles. Generated readers and
workbook vocabulary remain private to the data/mode lowering boundary. Runtime
code does not load JSON, TSV or Excel.

The runtime binds the exact current inputs as a sorted
`ConfigurationComponentSet`. Each component has a stable kind, key and
cryptographic digest. The component root is derived from that set; textual
revision selectors are not part of the public runtime interface.

## Replay

There is one replay format in `starclock-replay::format`. Its header contains:

- the current game environment;
- the exact configuration component set;
- the entry specification and seed;
- and the bounded record count.

Activity replays contain accepted Activity commands, expected states,
nested-battle identities/results and optional controller diagnostics. Battle
replays contain accepted battle commands and expected states. Verification
reconstructs execution from the current tree and requires an exact component
set. Unsupported old bytes are rejected; the repository has no legacy decoder,
migration selector or alternate current format.

## Adapter contract

CLI, Agent API and MCP expose only current mode/profile identities and opaque
offered actions. They do not expose schema selectors, runtime revisions,
executor revisions or compatibility fields. All authoritative numeric values
remain exact domain values or canonical decimal strings at transport
boundaries.

Large seeded matrices and full gameplay/replay checks remain available as
explicit exhaustive tests. Ordinary development uses focused package tests and
does not run complete gameplay simulations by default.
