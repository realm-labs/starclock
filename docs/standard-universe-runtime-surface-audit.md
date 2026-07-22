# Standard Universe Runtime Surface Audit

## Purpose

This document freezes the implementation surface observed at the start of Goal
04. It is an audit of the committed `285f14f` baseline, not a claim that the
Standard Simulated Universe runtime already exists. Later Goal 04 batches may
replace the limitations listed here, while the audit remains reproducible from
the baseline commit.

The machine-readable contract is
[`policy/goal04-surface-audit.json`](../policy/goal04-surface-audit.json). Run
`node tools/goal04/verify-surface-audit.mjs` to verify the baseline facts and
the frozen prerequisite bundles.

## Frozen baseline

| Surface | Baseline fact | Retained contract | Goal 04 gap owner |
|---|---|---|---|
| Workspace | Eleven crates; no `starclock-mode-universe` | Responsibility-separated crates and no reverse combat dependency | `G04-P1-B1` |
| Activity aggregate | One immutable `ActivitySpec`, one battle node, one battle sequence | `Activity::apply` remains the only run mutation boundary | `G04-P2-B1`–`B6` |
| Activity commands | `StartBattle`, `SubmitBattleResult` | Offered commands carry expected state identity and rejected commands are inert | `G04-P0-B2`, `G04-P2-B3` |
| Activity decisions | Start, submit result, terminal | Resolution returns an explicit next boundary | `G04-P0-B2`, `G04-P2-B3` |
| Battle handoff | Immutable `BattleSpec`, purpose-derived seed and full result identity | Battle owns combat mutation; Activity verifies the returned projection | `G04-P2-B5`–`B6` |
| Activity state | Four scopes and scalar/set slot values | Typed scoped state, explicit reset points and canonical hashing | `G04-P2-B2`, `G04-P2-B4` |
| Activity replay | Exactly one nested start/end pair around the two Activity commands | Versioned canonical records and first-class nested battle boundaries | `G04-P2-B7`, `G04-P5-B2` |
| Core catalog | 82-table Goal 01 production bundle | Its digest and public domain conversion remain unchanged | compatibility invariant |
| Universe catalog | 49-table Goal 03 staging/review bundle only | Generated rows remain private; JSON and Excel are never runtime inputs | `G04-P1-B1`–`B6` |
| Standard profile | Synthetic and one-battle profile construction | Profiles compile data into generic Activity definitions | `G04-P3-B1`–`B7` |
| CLI | Config, coverage, battle, replay and MCP routes | Headless deterministic JSON/human surfaces | `G04-P5-B3` |
| Agent API | Battle-session observation/action facade | Owned observations and opaque offered-action tokens | `G04-P5-B4` |
| MCP | Seven battle-v1 tools | Authorization, tenant/quota, idempotency and stdio/HTTP behavior | `G04-P5-B5` |

## What is already suitable

- `Activity::apply` and `Battle::apply` already establish separate mutation
  authorities.
- A battle handoff is immutable and a returned result is checked against the
  activity, node, attempt, sequence, catalog, specification, seed, projection
  and result digest identities.
- Scope identities, explicit reset points and typed slot values provide a
  usable foundation for run state.
- Canonical activity state hashing and versioned replay records already avoid
  ordinary Serde output as an authoritative codec.
- Agent and MCP clients act through offered actions rather than constructing
  unchecked combat mutations.

These are compatibility seams. Goal 04 extends their cardinality and domain
vocabulary; it does not introduce a second universe-specific run engine.

## Missing runtime behavior

The baseline cannot represent graph traversal, a domain hub, encounter groups,
route choice, services, shops, occurrences, inventories, bounded counter maps,
run RNG streams, multiple battles, carry between battles, full-run replay or a
run controller. It also cannot lower the isolated Universe Sora bundle into
runtime domain definitions.

Consequently, none of the 2,201 Goal 03 DataReady records, 786 mechanic-rule
bindings or 78 semantic fixtures is considered runtime-executable at this
audit point. Data readiness is not runtime coverage.

## Target ownership and dependency direction

```text
Excel authoring -> Sora universe bundle -> private generated readers
                                      -> starclock-mode-universe catalog/compiler
                                      -> generic starclock-activity definitions
external caller -> offered ActivityCommand -> Activity::apply
                                      -> optional immutable BattleHandoff
                                      -> Battle::apply
                                      -> verified BattleResult -> Activity::apply
```

The target has one generic Activity aggregate. `starclock-mode-universe` owns
data lowering and profile compilation, but does not own mutable run state,
canonical RNG, replay or an alternative `apply` method. `starclock-combat`
cannot depend on the mode crate or branch on Universe content IDs.

## Spatial-free encounter conclusion

A domain is a bounded Activity micrograph. Its hub offers stable handles for
available encounters, choices, services, rewards and interactables. Encounter
preparation produces a `BattleHandoff`; completion returns to the graph and
updates only declared projections. Mandatory clear state gates the exit.

Coordinates, collision, patrols, movement and aggro are intentionally absent.
This is sufficient for CLI, service, AI, MCP and future engine adapters to
control the same authoritative simulation.

## Migration order

1. Freeze interface, error/event, codec and revision migrations in `G04-P0-B2`.
2. Assign every frozen content/rule/fixture disposition in `G04-P0-B3`.
3. Add workload, dependency and release scaffolds in `G04-P0-B4`.
4. Lower the private Universe bundle before adding generic graph mutation.
5. Generalize Activity while continuously retaining the one-battle profile.
6. Compile Standard Universe behavior into those generic primitives.
7. Extend replay, CLI, agent and MCP only after the runtime surface is stable.

Every stage preserves the Goal 01/02/03 release contracts. Any intentional
revision change must be explicit and accompanied by migration tests; it must
not be hidden behind regenerated data or convenience exports.
