# Goal 06 Release Contract

Goal 06 releases the combat-owned input identity, opaque assembly provenance,
component replay v3 and current-Activity per-battle Standard Universe assembly.
The immutable release identity is
`combat-identity-dynamic-assembly-v1`.

## Released boundary

Every production Standard Universe battle now:

1. reads one immutable snapshot of current Activity contributions and carry;
2. resolves the pending encounter through the shared immutable catalog;
3. computes a canonical assembly key and opaque `AssemblyDigest`;
4. constructs `BattleSpec` and lets combat compute `CombatInputDigest`;
5. seals the handoff, catalog and result contract atomically;
6. records and freshly verifies the nested battle through replay v3.

CLI baseline runs, direct Agent sessions, MCP sessions and replay
reconstruction share that boundary. Historical component replay v2 remains
decodable and verifiable, but new production recordings do not emit it.

## Frozen evidence

The release binds:

- six representative inventory, lifecycle, progression, carry and
  provenance transitions;
- the eight ordered first-divergence boundaries;
- 33 complete world/difficulty runs containing 166 nested battles, 956
  accepted battle commands and 1,954 replay actions;
- cache hit, miss, eviction, rollback and concurrent-session fixtures;
- stable-runner performance evidence with an eight-entry default cache and a
  128 MiB retained-memory ceiling;
- the Goal 01 benchmark workload re-bound to the released
  `SCBS-v3/sha256-v4` state-hash oracle without rewriting Goal 01 evidence;
- native execution contracts for Windows x64, Linux x64 and macOS ARM64,
  with three alternate targets explicitly compile-only.

The machine-readable release evidence is generated and verified by:

```text
node tools/goal06/verify-release-contract.mjs . --release
```

The final completion tree is registered separately in
`policy/release-snapshots.json`. Later goals may evolve current source and CI
inventories, but they may not rewrite the Goal 06 completion tree.

## Explicit retained scope

This release does not claim that the 783 retained Standard Universe rules or
73 approximate enemy proxies are complete. Goal 07 owns that content and
accuracy work. Goal 06 proves that later content changes flow through the
correct deterministic battle boundary without adding Universe concepts to
`starclock-combat`.
