# Goal 06 Replay Reconstruction and Surface Parity

This document is normative for `G06-P3-B3`.

## Dynamic verification

Replay-v3 verification never trusts a recorded battle catalog or a cached
materialization as authority. For every `NestedBattleStart` record it advances
the reconstructed Activity to the pending boundary, derives the current
`StandardUniverseBattleSnapshot`, resolves the exact assembly key and compares
the resulting identities before executing the first battle command.

Cache hits are permitted only after exact-key lookup. The cache is scratch
state: a fresh factory produces the same verification result and first
divergence as a warm factory.

The dynamic corruption fixture proves the frozen first-divergence order:

1. component;
2. assembly;
3. combat input;
4. command;
5. event;
6. battle state;
7. nested result;
8. Activity state.

The fixture also counts assembler cache lookups and requires exactly one
snapshot resolution for every replayed nested battle.

## Production surface parity

CLI, Agent and MCP do not own separate battle construction rules.

- The CLI baseline path and incremental Agent path are run with equal world,
  difficulty, seed and stable decision policy. Their authoritative nested
  projection—handoff identity, accepted battle commands, complete event
  payloads, battle state hashes and nested results—must be byte-equivalent.
  Controller component identity and diagnostics are intentionally excluded
  from that comparison.
- MCP drives the same opaque actions to terminal through the Agent registry.
  Its exported replay bytes have the exact Agent replay SHA-256 and pass fresh
  replay-v3 verification.
- Concurrent Agent sessions share immutable catalogs and the bounded assembly
  cache, but retain independent mutable Activity state. Equal sessions produce
  equal final state and replay hashes.

These tests distinguish transport parity from presentation parity: JSON field
layout, MCP envelopes and controller diagnostics may differ, while the
authoritative nested battle projection may not.

The immutable Goal 02 HTTP trace remains unchanged as historical evidence.
Its conformance test freezes the transport boundary and action count, while a
separate current-codec golden freezes state hashes after the declared combat
state codec revision. A codec revision therefore cannot silently rewrite old
evidence or make the active server assert obsolete hash bytes.

## Historical compatibility boundary

The production runtime no longer exposes generic `materialization()` or
`into_parts()` accessors. New execution consumes `into_dynamic_parts()`.

The static catalog remains private inside a deliberately named
`into_replay_v2_compatibility_parts()` path used only to verify released replay
v2 bytes. The Goal 05 evidence generator can read baseline materialization
coverage counts, but cannot obtain the frozen materialization itself.
