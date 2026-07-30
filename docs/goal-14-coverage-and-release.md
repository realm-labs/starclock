# Goal 14 Coverage and Release Contract

`G14-P0-B4` freezes the valid seeded matrix, first vertical slice, policy
ownership, performance workloads, CI expectations and release gates. The
machine-readable authority is
[`policy/goal14-coverage-and-release.json`](../policy/goal14-coverage-and-release.json).

## Seeded matrix

The generated matrix contains 25 valid complete-run entries:

- 12 baseline entries cover all five formal difficulties, nine Paths and
  twelve Custom Dice with each die's exact six default faces;
- 12 Difficulty 5 entries cover Stats and Auxiliary Conundrum levels 1–6
  independently; and
- one Difficulty 5 entry covers the valid 6+6 total cap.

Every Conundrum entry carries the prior
`ClearFormalDifficulty:gold-gears.area.405` prerequisite. Level zero is covered
by baseline entries. This is axis and boundary coverage, not a claim that all
5×9×12×7×7 combinations are distinct or legal.

The matrix is generated and checked with:

```text
node tools/goal14/generate-coverage-matrix.mjs
node tools/goal14/generate-coverage-matrix.mjs --check
```

The output is
[`coverage-matrix.json`](../evidence/gold-and-gears-runtime-v1/foundation/coverage-matrix.json).
Each of the 16 inherited policies is assigned to one matrix probe and retains
its exact implementation owner.

## First vertical slice

The first executable slice uses Difficulty 1, Preservation, Custom Dice 101,
its six default faces, no Conundrum and seed 14001. It must cross entry,
three-plane graph construction, a dice/Knowledge resolution, Cognition,
Occurrence or service mutation, a real nested battle, carry settlement and
fresh component-addressed replay verification. A control without the
dice/Knowledge contribution must diverge at the declared mechanic boundary.

This slice proves the production path but cannot satisfy complete-content
coverage. The broader partitions and 25-run matrix remain required.

## Policy, performance and CI

All 16 Goal 08 policy boundaries have one or more Goal 14 owner batches.
Release permits only a versioned executable policy, metadata-only proof,
stronger replacement evidence or a terminal blocker. `InheritedPolicy` and
`AssignedPendingResolution` are not release states.

Seven stable workloads cover catalog lowering, all matrix entries, full-run
replay, trigger-heavy dice/Knowledge work, warm battle assembly, concurrent
shared catalogs and invalid-command/replay corruption. Structural budgets
forbid per-run catalog clones, repeated catalog composition, replay-prefix
reconstruction and warm-assembly allocation. Host timing/allocation ceilings
are frozen after the first executable baseline and then guarded at 120%.

Windows x64, Linux x64 and macOS ARM64 must execute the same Gold goldens.
Windows ARM64, Linux ARM64 and macOS x64 remain compile-only and never count as
runtime evidence.

## Release boundary

The release scaffold requires all 48 batches, 7,913/1,224/18 exact-once
terminal coverage, all 16 terminal policies, fresh matrix replay verification,
surface parity, hardening, performance, native CI, release audits and a clean
checkout. Only `G14-P8-B4` may create the final evidence and register the
immutable completion snapshot.
