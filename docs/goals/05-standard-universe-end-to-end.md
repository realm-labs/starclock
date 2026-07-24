# Goal 05 — Standard Universe End-to-End Integration

## Objective

Turn the completed Goal 04 Standard Simulated Universe release from a
deterministic orchestration plus independently executable mechanic evaluators
into a genuinely end-to-end runtime. Production CLI, baseline AI, agent and MCP
workflows must atomically apply noncombat mechanics and execute real
`starclock-combat` battles carrying the selected Path, Blessings, Resonance,
Formations, Curios, Ability Tree, techniques and encounter rules.

The normative design is
[Standard Universe end-to-end runtime integration](../27-standard-universe-end-to-end-integration.md).

## Frozen prerequisites

- Goal 01 through Goal 04 completion commits remain immutable release
  snapshots under `policy/release-snapshots.json`.
- Goal 04 remains the compatibility oracle for its declared reference
  projection/replay revision, not the acceptance oracle for Goal 05 behavior.
- The Version 4.4 Standard Universe Excel/Sora dataset and its documented
  approximation policies remain the content denominator.
- Goal 05 may regenerate workbooks only through the pinned Python `openpyxl`
  authoring tool and Sora 0.3.0 pipeline.

## Terminal outcome

- composed immutable Activity-handler and combat-rule registries replace
  manually assembled mode dispatch;
- physical Activity nodes may share a bounded `DomainVisit` logical scope;
- every offered Occurrence/service/Curio/run effect used by a production run is
  applied atomically through checked Activity operations;
- every structured Standard Universe encounter materializes an executable
  immutable `BattleSpec`;
- production workflows execute `Battle::create/apply` and project the verified
  result back into Activity state;
- Universe battle-visible contributions resolve through normal combat
  operations/events, never by mutating combat state from mode code;
- replay binds consumed component and registry digests and re-verifies real
  nested battles;
- reference Won projection remains test-only and cannot satisfy a production
  acceptance test;
- daily focused acceptance remains within 1–3 minutes on the stable runner;
  the full gate remains an explicit release/checkpoint action.

## Non-goals

- Swarm Disaster, Gold and Gears, Unknowable Domain, Divergent Universe or a
  new universe content snapshot;
- general multi-pending task/ForkJoin support;
- UI, Bevy, 3D navigation, collision, enemy patrol or presentation timing;
- replacing documented Version 4.4 approximation policies with fabricated
  values;
- rewriting Goal 01–04 historical evidence or replay files;
- a dynamic plugin ABI, runtime scripting or filesystem-discovered handlers.

## Architecture invariants

1. `Activity` and `Battle` remain the only authoritative mutation boundaries.
2. Mode handlers return typed operations/contributions and cannot mutate state.
3. `starclock-combat` never depends on `starclock-mode-universe`.
4. A real nested battle consumes an immutable snapshot; battle code never reads
   live Activity state.
5. Costs, effect application, interaction consumption and transition are one
   Activity transaction.
6. Every RNG draw is labeled, counted and replayed.
7. A `DomainVisit` has a stable bounded identity and fresh revisit semantics.
8. Production replay identity covers only consumed components plus artifact
   provenance; normal serialization is never hashed.
9. Generated Sora rows, numeric backends and mode-specific content types do not
   leak into generic public APIs.
10. Handwritten Rust files remain below 1,200 physical lines and new mode
    support does not require central content-ID branches.

## Execution and commit rules

- Execute the earliest unblocked batch; keep one batch `InProgress`.
- Update the status ledger in every batch commit.
- Commit subjects use
  `<type>(<scope>): <batch-id> <imperative summary>`.
- Each phase starts with a vertical integration slice before broad migration.
- Focused checks run per batch. `node tools/repository-check/run.mjs --full`
  runs only at phase/release gates.
- Performance evidence reuses compiled artifacts and existing fixtures.
- New dependencies require an exact pin and license/determinism review.
- Preserve unrelated changes and do not edit frozen Goal 01–04 release
  evidence.

## Delivery phases

### Phase 0 — Contract and failing integration probes

| Batch | Deliverable |
|---|---|
| `G05-P0-B1` | Freeze Goal 05 plan, launch prompt, ledger and normative integration design; record the Goal 04 gaps without changing its historical release. |
| `G05-P0-B2` | Add failing/ignored-until-owned integration probes for real Blessing battle impact, atomic Occurrence/service state changes, fresh domain revisit and component replay divergence; freeze focused performance workloads. |

### Phase 1 — Generic extension foundations

| Batch | Deliverable |
|---|---|
| `G05-P1-B1` | Add immutable composed Activity handler/executor bundles with stable IDs, schema/revision validation, deterministic order and registry digest. |
| `G05-P1-B2` | Add bounded logical scope definitions/instances and canonical `DomainVisit` ownership over existing physical Activity nodes. |
| `G05-P1-B3` | Add ordered configuration-component identities/root digest and the new replay header/verification scaffold while retaining legacy decode/verify. |

### Phase 2 — Atomic run and noncombat mechanics

| Batch | Deliverable |
|---|---|
| `G05-P2-B1` | Replace opaque external-outcome clearance with validated interaction bindings resolved through the composed registry in one Activity transaction. |
| `G05-P2-B2` | Lower and atomically apply Occurrence choices, costs, outcomes and declared RNG policies; add stale/fault rollback tests. |
| `G05-P2-B3` | Lower and atomically apply services, shops, respite, revival, downloader and currency changes. |
| `G05-P2-B4` | Integrate Curio lifecycle/run effects and Ability Tree boundary projections with the same operation pipeline. |

### Phase 3 — Real nested combat

| Batch | Deliverable |
|---|---|
| `G05-P3-B1` | Add a Universe battle-contribution compiler that maps Path/Blessing/Resonance/Formation/Curio/Ability Tree snapshots to validated combat RuleBundle/modifier bindings. |
| `G05-P3-B2` | Materialize all structured Standard Universe encounter members/waves/difficulty bindings into executable BattleSpecs with explicit exact/approximate coverage. |
| `G05-P3-B3` | Add the production nested Battle executor using `Battle::create/apply`, deterministic battle AI, declared result projection and carry settlement. |
| `G05-P3-B4` | Prove real Blessing, Resonance, Curio, Ability Tree, technique, multi-wave and defeat/carry behavior through end-to-end battle fixtures. |

### Phase 4 — Replay and external surfaces

| Batch | Deliverable |
|---|---|
| `G05-P4-B1` | Record and verify real nested battle command/event/state streams under component/registry replay identity with first-divergence diagnostics. |
| `G05-P4-B2` | Migrate CLI universe run/verify and remove production `verified-reference-projection-v1` settlement. |
| `G05-P4-B3` | Migrate agent sessions and MCP Activity tools without changing authorization, quota or idempotency guarantees. |

### Phase 5 — Coverage and release

| Batch | Deliverable |
|---|---|
| `G05-P5-B1` | Regenerate the 33-entry seeded matrix with real battles and atomic interactions; publish integration coverage separating exact and approximate encounter behavior. |
| `G05-P5-B2` | Run cross-platform determinism, malformed replay, transaction rollback, concurrency and 1–3 minute focused-performance gates. |
| `G05-P5-B3` | Freeze Goal 05 documentation/evidence/release snapshot, pass the full clean-worktree gate and mark the ledger complete. |

## Acceptance

- no production universe runner or session calls `reference_won_result`;
- at least one real battle golden demonstrates a combat hash/event difference
  caused solely by an owned Blessing;
- Resonance activation is a legal combat action/resource transition;
- Occurrence and service selections update authoritative Activity state and
  cannot be acknowledged as complete without applying their effect;
- logical domain state resets on revisit without global source-ID collisions;
- all 173 structured encounter members materialize or fail catalog validation;
- all production nested results are sealed from actual battle projections;
- replay rejects altered component/registry identity and nested command/event
  divergence;
- all enabled Goal 04 mechanic dispositions remain assigned and now report
  `Integrated`, `Metadata`, `Policy` or an explicit retained approximation;
- focused format/lint/tests complete within the daily budget and the final full
  repository gate passes.

## Terminal checklist

- [ ] All 19 batches are committed with ledger evidence.
- [ ] Atomic run/noncombat effects pass.
- [ ] Real nested combat passes for all structured encounters.
- [ ] Production reference settlement is removed.
- [ ] Component-aware replay and external interfaces pass.
- [ ] Seeded/cross-platform/performance evidence passes.
- [ ] Full clean-worktree release verification passes.
