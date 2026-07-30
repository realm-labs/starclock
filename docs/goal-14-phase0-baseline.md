# Goal 14 Phase 0 Baseline

Batch `G14-P0-B1` freezes the starting contract for the Gold and Gears
runtime. The machine-readable authority is
[`policy/goal14-foundation.json`](../policy/goal14-foundation.json), verified by
`node tools/goal14/verify-foundation.mjs`.

## Frozen input

- Goals 01–08 are consumed through their immutable completion commits and
  trees, not by reinterpreting historical evidence against the live source
  tree.
- Goal 08 contributes exactly 7,913 source obligations, 1,224 mechanic rules,
  18 semantic fixture families and 16 inherited policy boundaries.
- The Candidate bundle is
  `97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b`;
  the normalized pack is
  `ea2f3a35807b9a7dae39be2d67fb5de955bfad7852718eb1d3393affed5a5623`.
- The merged six-mode Candidate audit covers 46,110 manifest records and all
  15 mode pairs with zero conflicts and zero runtime-enabled modes.

The five Goal 08 reference, manifest, normalized-reference, workbook and
generated-output roots are protected at their starting Git tree identities.
Goal 14 writes new runtime manifests, evidence and tools only to its revisioned
roots unless a later batch follows the explicit authored-data revision path.

## Runtime baseline

The starting Activity, combat, build, replay, rules, Universe-mode, CLI, agent
and MCP crate trees are retained in the foundation policy as historical
baselines. They are not compatibility locks on future Goal 14 implementation;
they make later architectural and interface changes reviewable.

The reusable boundary is the released generic one:

```text
offered ActivityCommand
        |
        v
Activity::apply ----> pending immutable BattleSpec
                              |
                              v
                       Battle::apply
                              |
                              v
                    verified BattleResult
                              |
                              v
                     Activity settlement
```

Gold and Gears remains a profile in `starclock-mode-universe`. Generated Sora
rows lower privately, shared content links by stable identity and digest, and
CLI, agent API and MCP remain adapters over the same authoritative runtime.

## Verification result

The retained execution evidence is
[`execution-baseline.json`](../evidence/gold-and-gears-runtime-v1/foundation/execution-baseline.json).
It records eight immutable prerequisites, the passing merged Candidate audit,
the five protected roots, the nine historical runtime crate trees and the
48-batch execution package. The next batch is `G14-P0-B2`, which generates the
exact source, rule and semantic-fixture dispositions.
