# Goal 06 Phase 0 Baseline

Phase 0 freezes the contracts that govern the identity, replay, cache,
performance and release migration. Verify them with:

```text
node tools/goal06/verify-phase0.mjs
```

## Replay compatibility

Released component-addressed replay v2 remains format version 2/schema 1 and
retains its decoder plus Standard Universe verifier. New production output
will use v3 and bind, per nested battle:

1. consumed component root;
2. `AssemblyDigest`;
3. combat-input codec revision;
4. `CombatInputDigest`;
5. handoff identity;
6. result identity.

First divergence order is component, assembly, combat input, command, event,
state, result and Activity. Unknown records fail closed. Frozen v2 bytes and
evidence are not regenerated.

## Performance workloads

Six release workloads cover:

- 10,000 combat-input digest computations;
- cold assembly for all 33 World/difficulty entries;
- 10,000 warm representative assembly lookups;
- 256 cache eviction/invariance transitions;
- 16 concurrent sessions sharing one immutable catalog; and
- the 33-entry real replay-v3 run matrix.

The factory composes its catalog once and a battle composes it zero times.
Cache fields remain non-authoritative. A material regression of 20% requires
explicit review; focused acceptance has a 180-second maximum.

## Dependency baseline

Phase 0 adds no Rust dependency. Identity uses the existing SHA-256 primitive,
the bounded cache uses standard-library collections and replay v3 reuses the
project canonical codec. `Cargo.lock` and the workspace manifest hashes are
frozen in `policy/goal06-dependency-baseline.json`.

Any later dependency change requires an exact pin, license inventory,
deterministic-impact review and affected cross-platform goldens.

## Release scaffold

The release contract starts as `Scaffold` over five phases and 18 batches. It
requires all five prior Goal snapshots and reserves the terminal revisions:

- `combat-input-v1`;
- `battle-assembly-v1`;
- `component-addressed-v3`;
- `standard-universe-dynamic-assembly-v1`.

Promotion to `Released` belongs only to `G06-P4-B3`.

