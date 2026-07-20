# Activity replay and controller diagnostics

Goal 01 batch `G01-P6-B4` fills the version-1 replay envelope without changing
its header, digest, framing or unknown-record policy. Activity verification is
one-shot and linear: it reconstructs one fresh `Activity`, applies each accepted
command once, and compares the declared canonical state hash immediately.

## Record layout

Each accepted Activity command owns exactly three authoritative records. An
optional `ControllerDiagnostic` may precede that group.

| Command | Canonical record order |
|---|---|
| Start battle | accepted Activity command, expected Activity state, nested battle start identity |
| Submit result | nested battle end digest, accepted Activity command with the complete declared result projection, expected Activity state |

The nested start carries the exact Activity scope, battle sequence, definition,
configuration, participant-lock and BattleSpec digests, and derived battle seed.
The nested end repeats the submitted result digest. A mismatch identifies the
first affected Activity command; malformed order, extra records and incomplete
Activities are hard failures.

Command payloads are explicitly versioned. The submitted result codec covers
every declared projection family, including the stable terminal-fault tuple and
typed metric values. Replay reconstruction is the only reason combat exposes a
fault constructor from stable parts; platform error strings never enter it.

## Identity and diagnostics

Verification binds the Activity entry to the exact configuration digest,
profile ID, authored definition ID and digest, and BattleSpec digest before any
command executes. A low-level battle entry cannot be interpreted as an Activity
replay.

Controller diagnostics contain a versioned controller family, decision
sequence, selected canonical ordinal, optional draw count and bounded integer
scores. Scores are canonicalized by ordinal. The selected ordinal must be the
maximum total with the lowest ordinal winning ties. Diagnostics are validated
and counted but never participate in authoritative Activity state or hashes;
only the recorded accepted command does.

The golden integration fixture records one complete Standard-shaped Activity,
pins its replay-byte digest, verifies both nested boundaries and both controller
families, and mutates state/start/end payloads to prove first-divergence errors.
