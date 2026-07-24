# Standard Universe Integration Coverage

This report separates catalog presence, Activity integration, real battle
execution and factual accuracy for the Goal 05 Standard Simulated Universe
runtime. A row being loadable or evaluable does not by itself mean its combat
mechanic is integrated.

The machine-readable assignments are:

- `content-manifests/standard-universe-end-to-end-v1/integration-dispositions.json`;
- `evidence/standard-universe-end-to-end-v1/coverage/seeded-matrix.json`.

Both artifacts are regenerated and byte-compared by the Goal 05 coverage
tools. The released Goal 04 disposition manifest remains unchanged and is the
source completeness oracle.

## Disposition meanings

| State | Meaning |
|---|---|
| `Integrated` | A production Activity mutation, routing path or executable combat lowering consumes the row. |
| `Metadata` | The row documents or tests behavior but is not runtime content. |
| `Policy` | The row controls deterministic selection or eligibility rather than applying an effect. |
| `RetainedApproximation` | Typed data/evaluator behavior exists, but an exact production lowering, source value or dynamic boundary remains explicitly incomplete. |

The Goal 05 assignment is exact-once:

| Family | Integrated | Metadata | Policy | Retained approximation | Total |
|---|---:|---:|---:|---:|---:|
| Content records | 889 | 0 | 92 | 1,220 | 2,201 |
| Mechanic rules | 3 | 0 | 0 | 783 | 786 |
| Semantic fixtures | 0 | 78 | 0 | 0 | 78 |

The three combat-integrated rule slices are:

- enhanced Abundance Blessing `universe.blessing.612344.level.2`;
- active Curio state `universe.curio.8.state.active`;
- Hunt Resonance `universe.resonance.612420`.

They prove that an owned mode rule can alter real combat events, that a
BattleStarted Curio can execute through Rule IR, and that Resonance is a legal
resource-consuming combat action. Other typed Goal 04 effect evaluators are
not relabeled as integrated combat rules.

## Encounter accuracy

All 173 structured encounter members lower to executable one-wave
`BattleSpec` values, covering 538 enemy slots. All 182 difficulty bindings are
also construction-validated.

Definition and numeric accuracy remain separate:

| Measure | Exact | Approximate | Total |
|---|---:|---:|---:|
| Enemy stable-key definition match | 13 | 73 proxy mappings | 86 |
| Runtime HP/Speed/stat assembly | 0 | 86 under `goal01-executable-enemy-proxy-stats-v1` | 86 |

An exact definition match means that the referenced enemy exists in the Goal
01 combat catalog. It does not promote the Standard Universe level-scaling
assembly to exact. Missing public definitions use deterministic role/rank
proxies and remain visibly approximate.

## Seeded real-run matrix

Seeds `200000..200032` execute one run for each of the 33 constructible
World/difficulty entries and rotate through all nine Path options. The release
matrix records:

- 33 completed runs across nine Worlds;
- 1,749 external Agent actions and 1,904 replay Activity actions;
- 417 atomic external-outcome selections;
- 155 real nested battles;
- 905 accepted combat commands and 905 corresponding battle-state/event
  records;
- nine consumed replay components per run;
- fresh reconstruction and replay-v2 verification for every run.

World/difficulty routes with zero battles remain valid if authored that way;
the verifier never fabricates a combat projection to inflate coverage.
Unknown Worlds, out-of-range difficulties and overflowing seeds fail before
session creation and report `committed=false`.

## Deliberate retained boundaries

The shared production factory begins with the selected Path, empty Blessing
and Curio inventories, and no Ability Tree selection. Its initial battle
assembly therefore declares and materializes zero unowned mechanic rules.

Activity acquisition, costs, inventory/lifecycle changes and graph transition
are authoritative and atomic. However, production battles currently use the
entry-time battle materialization; newly acquired inventory is not
rematerialized into each later `BattleSpec`. This is why the manifest keeps
the remaining 783 combat rules and relevant content records under
`RetainedApproximation` instead of claiming complete dynamic integration.

Similarly, generated shop offers without public concrete price rows, deferred
HP/roster/special Occurrence atoms, and missing enemy definitions or exact
Standard Universe stat curves remain explicit. They are never completed by
inventing source values.

## Verification

Run:

```text
node tools/goal05/verify-seeded-matrix.mjs
node tools/goal05/verify-integration-coverage.mjs
```

The first command re-executes all 33 runs and compares the complete evidence.
The second verifies exact-once assignment of all Goal 04 records, rules and
fixtures and cross-checks the matrix and encounter denominators.

The native hardening gate is:

```text
node tools/goal05/run-native-hardening.mjs --run
```

It combines 16 replay-v2 corruption cases, 16 concurrent shared-factory
sessions, nested-executor rollback, atomic noncombat fixtures and the complete
33-run matrix under a 180-second ceiling. The recorded Windows x64 run
completed in 31.632 seconds. CI executes the same command natively on Windows
x64, Linux x64 and macOS ARM64; only retained successful CI artifacts are
cross-platform execution proof. Windows ARM64, Linux ARM64 and macOS x64
remain compile-only and make no runtime determinism claim.
