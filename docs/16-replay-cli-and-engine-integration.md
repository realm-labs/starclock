# Replay, CLI and engine integration

Replay is a deterministic trace for the current tree. It is not a long-term
save format and has no backward-compatibility or migration contract.

## Current replay

Public component-addressed Activity replay uses
`starclock_replay::current`. Standard Universe, Gold and Gears and Swarm
Disaster all write and read through this single public entry point. Callers do
not choose a replay generation.

A replay binds the exact current inputs consumed by execution:

- ordered configuration component identities and digests;
- entry definition and immutable spec/build identities;
- explicit seed and controller identity;
- accepted Activity and battle commands in sequence;
- nested battle start, command, event, state and result boundaries;
- expected Activity and battle state hashes.

Rejected command attempts are diagnostics, not part of the authoritative
accepted stream. Unknown records, malformed lengths, wrong component digests,
invalid commands, truncation and trailing bytes fail closed.

Verification reconstructs a fresh current Activity/battle, reapplies accepted
commands and reports the first semantic divergence in this order: component,
assembly, combat input, command, event, state, result, Activity.

Old replay bytes are rejected. Changing the codec, rules, data, hashes or
payload layout replaces the current format and current goldens; it does not add
a legacy decoder or migration path.

## CLI

The `starclock` CLI is a current adapter over domain crates. Its relevant
surfaces are configuration validation, coverage inspection, battle or Universe
execution, and replay verification. JSON output is deterministic and uses
canonical strings for authoritative numbers and hashes. Human output is a
readable projection of the same result.

`starclock challenge config validate [--json]` loads the current challenge
bundle, lowers Memory of Chaos, Pure Fiction and Apocalyptic Shadow, and
compiles each mode-owned combat catalog over the production catalog. It is a
read-only preflight for adapters and UI; it does not synthesize player builds
or mutate an Activity.

CLI flags may select current content, seeds and output paths. They do not alter
rounding, RNG mapping, event ordering or transaction semantics. Exit codes
distinguish usage, configuration, replay divergence, invalid scenarios,
simulation faults and adapter I/O failures.

## Agent and MCP adapters

Agent API and MCP own sessions, authorization, transport framing and public
projections. They do not own battle or Activity mutation. Replay exported from
an adapter is verified by the same current domain path used by the CLI.

## Engine adapters

A presentation engine sends domain commands and consumes events and read-only
views. Frame time, animation timing and engine entity IDs never enter
authoritative state. Save/account/network migration is outside the current
replay boundary.

## Testing

Focused replay tests run with Cargo for the affected package. Default tests
cover current round trips, malformed input and first-divergence behavior.
Large corruption corpora, concurrent sessions, TCP load traces and seeded mode
matrices are explicit exhaustive checks rather than edit-loop gates.
