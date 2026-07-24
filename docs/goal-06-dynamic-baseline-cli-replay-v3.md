# Goal 06 Dynamic Baseline, CLI and Replay v3

This document is normative for `G06-P3-B1`.

## Production baseline boundary

New Standard Universe automatic runs use
`StandardUniverseBaselineRunner::run_to_terminal_dynamic`. At every pending
battle it:

1. asks the shared `StandardUniverseBattleAssembler` to snapshot and
   atomically start the current encounter;
2. passes `StandardUniverseDynamicBattleStart` to a
   `DynamicNestedBattleExecutor`;
3. executes against the exact immutable combat catalog paired with that
   handoff;
4. settles the projected result back into Activity state;
5. repeats assembly for the next battle from the newly settled inventory and
   participant carry.

Execution failure rolls back the just-started Activity boundary. Cache state
is not authoritative.

The original `NestedBattleExecutor` and fixed-catalog verifier remain only for
released replay-v2 and earlier replay-v3 compatibility fixtures. New CLI runs
do not use that path.

## CLI contract

`starclock universe run` now:

- creates a dynamic-only nested executor;
- records through `record_baseline_run_v3`;
- emits a replay-v3 envelope;
- reports `starclock-cli-universe-v3`;
- produces the final Activity hash from current-state per-battle assembly.

`starclock replay verify` recognizes both replay v2 and v3. Replay v2 uses the
historical fixed-catalog verifier. Replay v3 reconstructs every recorded
battle through the shared assembler and compares commands, event payloads,
battle state hashes, results and Activity hashes.

## Replay recording

Dynamic replay recording captures:

- the v3 component-addressed header;
- the assembly and combat-input identity of every handoff;
- every accepted battle command and its controller;
- complete emitted event payloads and state hash after each command;
- the exact nested result identity and digest;
- the Activity state hash after settlement.

The same seed and configuration must reproduce the exact final hash and replay
bytes. A modified trailing byte is rejected by the CLI verifier.

## Compatibility

No released v2 decoder, encoder fixture or static v3 compatibility test was
rewritten. New production recording and historical verification are separate
entry points so current-state assembly cannot silently reinterpret old replay
bytes.
