# Currency Wars Coverage and Release Contract

`G21-P0-B5` freezes the bounded complete-run matrix, first vertical slice,
policy ownership, replay identity, performance workloads and native CI target.
The machine-readable authority is
[`content-manifests/currency-wars-runtime-v1/coverage-and-release.json`](../content-manifests/currency-wars-runtime-v1/coverage-and-release.json).

## Seeded matrix

The generated matrix contains 97 complete-run targets, one for every authored
difficulty. Together they cover all 26 routes, both Gambits, all 77 focal
roles, all rarities and position kinds. They use the released maximum rank as
a legal setup; Overclock rows additionally require a prior Standard clear.
Because released route rows publish no Gambit selector, every route/Gambit join
visibly names `VersionedProjectPolicy:route.gambit_membership`.

The complete runs are only one layer of the matrix. Production execution
fixtures assign exact-once targets for all 834 investment identities, 77 roles,
10 team-size states, 10 rank boundaries, 189 star transitions, 152 Bond levels,
653 Bond contributions, 25 encounter groups, five wave records, 306 enemy
slots, 721 affixes, 10 boss pools and 341 battle overrides. The same artifact
assigns all 43 mechanic partitions, 28 semantic fixture families and 12 policy
boundaries. A fixture earns credit only when production-lowered behavior changes
authoritative state or a typed rejection/control proves the boundary; loading
an ID or retaining catalog metadata is insufficient.

Some authored Bond thresholds exceed the number of direct members, and
subtraits use an explicit selection boundary. Released guidance says that Bond
levels depend on deployed matching members, while the released Emblem rule lets
a non-member join a Bond and add to its count. The matrix therefore records
direct members plus required Emblem/trait contributions instead of accepting
the current one-point-per-distinct-role skeleton as complete. See the
[official gameplay guide](https://www.hoyolab.com/article/42136581) and the
[released Bond/Emblem cross-check](https://honkai-star-rail.fandom.com/wiki/Currency_Wars%3A_Zero-Sum_Game/Bonds),
accessed 2026-08-13.

Generate and verify the contract with:

```text
node tools/currency-wars-runtime/generate-coverage-and-release.mjs --check
node tools/currency-wars-runtime/verify-coverage-and-release.mjs
```

## First vertical slice

The selected slice is `G21-VERTICAL-SLICE-01` with seed `21000501`, profile
`currency-wars.profile.v1`, module `currency-wars.module.7100501`, Standard
Gambit, route 100 and difficulty 10101. Its four-role deployment uses roles
1004, 1001 and 1003 to activate Bond 1001 and role 1508 to activate Projection
1508. That Projection contributes the exact all-member
`ExtraAllDamageTypeAddedRatio5 = 0.2` property; an otherwise identical control
must differ at that contribution boundary.

The first battle begins at route node
`currency-wars.node.route.100.chapter.1.section.1`, encounter 70000001. The
production encounter overlay lowers the offered node into real enemies and
waves. `G21-P6-B1` selects an exact Camp and released BattleArea/Stage boundary,
uses StageConfig as the level/wave/formation skeleton, fills each wave without
replacement from the Camp or BossPool GridFightMonster candidates, and applies
the selected monster's exact star and difficulty scaling. The unresolved draw,
boss-identity, enemy-star and FormationWave selectors remain explicit
replaceable project policies rather than inferred parity claims.

The slice is now `ProductionRunExecutedAndFreshReplayed`. P3-B6 loads the
production catalog and executes the selected seed through paid refresh and
purchase, deterministic no-combination proof, deployment and Bond recomputation,
all 23 route nodes, 20 typed battle handoffs/results, one non-victory checkpoint
recovery, both Plane transitions, final completion and SSS settlement.
`G21-P6-B1` materializes every one of those battle boundaries with the
production catalog and validates each immutable `BattleSpec` by constructing a
Battle. The Phase 6 partitions execute battle-visible programs and P7 executes
offered combat commands, nine-component fresh replay and the complete legal
matrix. The slice therefore carries production assembly, combat and replay
evidence rather than a boundary stub.

## Replay, performance and CI

Replay identity uses the nine ordered components from the runtime contract and
binds exact component digests, run selectors, participant lock, accepted
Activity commands, battle assembly, battle commands/events, sealed results and
settlement. Verification must reconstruct fresh immutable production inputs and
report the first divergent component or record.

Eight stable workload shapes cover catalog lowering, all matrix entries, a
complete run plus fresh replay, trigger-heavy investment/Bond/battle work,
10,000 allocation-free warm assembly reads, 16 concurrent sessions over shared
catalogs and 4,096 invalid-command/replay-corruption cases. Timing thresholds
are intentionally deferred until the first executable release-mode baseline;
that baseline is then frozen and guarded at 120%.

Windows x64, Linux x64 and macOS ARM64 must execute identical generated
matrix/replay goldens. Windows ARM64, Linux ARM64 and macOS x64 remain
compile-only; cross-compilation and emulation do not count as native runtime
evidence.

## Current boundary

The runtime admits zero native handlers and is
`RuntimeCoverageCompletePendingNativeRelease`. All 19,250 source obligations,
2,367 programs, 28 semantic fixture families and 12 policies are terminal;
97 legal matrix runs execute production battles and fresh replay, and all eight
performance workloads have a frozen local macOS ARM64 baseline. The repository
now declares Windows x64, Linux x64 and macOS ARM64 native jobs against one
frozen run/replay evidence digest, with the paired ARM64/ARM64/x64 targets
compile-only. Fresh local macOS ARM64 clean-checkout acceptance passes on the
current tree. The hosted Windows x64 and macOS ARM64 jobs are terminal; the
Linux x64 runtime evidence is terminal, and its paired Linux ARM64 compile-only
check now installs the required cross compiler. `G21-P8-B5` remains open until
that corrected hosted matrix reruns successfully.
