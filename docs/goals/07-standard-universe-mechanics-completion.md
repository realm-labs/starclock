# Goal 07 — Complete Standard Universe Mechanics

## Objective

Complete the public Version 4.4 Standard Simulated Universe mechanics on top of
Goal 06 dynamic per-battle assembly.

Every released Standard Universe content row must have an exact-once runtime
disposition. Every combat rule must lower to executable Rule IR, a reviewed
static native handler or an already executable shared primitive. Noncombat
effects must commit through Activity operations or a declared external
decision. Numeric values may use a named, evidenced approximation when public
data is unavailable; mechanic behavior may not be replaced by a generic proxy.

## Start condition

Goal 07 is blocked until:

- Goal 06 is `Complete`;
- its immutable release snapshot is registered;
- replay v3 and per-battle dynamic assembly pass clean verification; and
- the Goal 06 release contract confirms that current Activity inventory and
  lifecycle state feed every later battle.

Goal 07 must not recreate that foundation in a content partition.

## Frozen content denominator

The starting oracle is the Goal 05 exact-once assignment:

| Family | Denominator | Starting executable/integrated | Starting policy/metadata | Starting retained implementation approximation |
|---|---:|---:|---:|---:|
| Content records | 2,201 | 889 | 92 policy | 1,220 |
| Mechanic rule bindings | 786 | 3 | 0 | 783 |
| Semantic fixtures | 78 | 0 production dispositions | 78 metadata fixtures | 0 |

The inherited Goal 04 mechanism partitions are:

| Milestone | Family | Content | Rules | Fixtures |
|---|---|---:|---:|---:|
| `M01` | Shared Activity and Ability Tree | 42 | 42 | 10 |
| `M02` | Preservation | 59 | 58 | 7 |
| `M03` | Remembrance | 59 | 58 | 5 |
| `M04` | Nihility | 59 | 58 | 2 |
| `M05` | Abundance | 59 | 58 | 3 |
| `M06` | Hunt | 59 | 58 | 2 |
| `M07` | Destruction | 59 | 58 | 1 |
| `M08` | Elation | 59 | 58 | 2 |
| `M09` | Propagation | 59 | 58 | 3 |
| `M10` | Erudition | 59 | 58 | 2 |
| `M11` | Positive, neutral and special Curios | 86 | 86 | 12 |
| `M12` | Negative, error, repair and replacement Curios | 42 | 42 | 7 |
| `M13` | Occurrences | 447 | 0 | 9 |
| `M14` | Services, roster and interactables | 94 | 94 | 9 |
| `M15` | Encounters, Worlds, difficulty and carry | 959 | 0 | 4 |

`G07-P0-B3` must generate and commit the final execution partition manifest
from these exact identities. It expands milestone rows into concrete
`G07-Pn-Mnn-Snn` sub-batches before broad implementation begins.

## Completion dispositions

Goal 07 replaces the ambiguous `RetainedApproximation` state with explicit
runtime and accuracy dimensions.

Runtime disposition:

| State | Meaning |
|---|---|
| `ExecutableRuleIr` | Mechanic executes through validated generic combat or Activity IR. |
| `ExecutableNative` | Mechanic executes through a reviewed deterministic static handler. |
| `ExecutableShared` | Row selects an already executable shared primitive with proven parameters. |
| `SelectionPolicy` | Row only controls deterministic eligibility/selection and has no effect body. |
| `ExternalDecision` | Noncombat outcome is supplied by an explicit command and then applied atomically. |
| `Metadata` | Row is descriptive/test metadata and makes no runtime claim. |

Accuracy disposition:

| State | Meaning |
|---|---|
| `ExactPublic` | Mechanic and numeric inputs match the frozen public evidence. |
| `ApprovedNumericApproximation` | Mechanic is correct; unavailable numeric input uses a named formula/value range with provenance and confidence. |
| `NotApplicable` | The row has no authoritative numeric behavior. |

No enabled rule may finish as metadata, policy or external decision. No row
may use a retained approximation to mean “the implementation is missing.”
Insufficient public evidence for mechanic behavior blocks the affected
partition and must be escalated rather than guessed.

## Terminal outcome

- all 786 rule bindings have executable runtime dispositions;
- all 78 semantic fixtures execute against production definitions and values;
- all 2,201 content rows are consumed exactly once by runtime, selection,
  external-decision or metadata paths;
- every acquired/upgraded/removed mechanic affects the correct subsequent
  battle or Activity boundary through Goal 06 assembly;
- Path passives, Blessings, enhanced levels, Resonances and Formations for all
  nine Paths execute with correct lifecycle, targeting and stacking;
- all 61 Curios and 67 Curio lifecycle states execute, including negative,
  error, repair, replacement and charge behavior;
- all Ability Tree effects execute at their declared run/battle boundary;
- every Occurrence/service effect atom is executable or explicitly external;
- shops produce deterministic concrete offers and prices under exact or
  approved numeric policies;
- all 86 referenced enemy variants have mechanism-correct definitions and AI;
- all 173 encounter members and 182 difficulty bindings execute correct
  wave/phase/carry behavior;
- the generic rank-proxy behavior path is removed from released Standard
  Universe profiles;
- CLI, baseline AI, Agent, MCP and replay v3 expose the same mechanics;
- focused partition checks remain within 1–3 minutes and the full release gate
  is explicit.

## Non-goals

- Swarm Disaster, Gold and Gears, Unknowable Domain, Divergent Universe or
  historical expansion-mode revisions;
- Currency Wars;
- UI, game assets, story presentation, 3D navigation or enemy patrols;
- account progression, inventory ownership, gacha or reward payout systems;
- exact reconstruction of hidden server-only values when public evidence is
  unavailable;
- copying long game descriptions or redistributing assets;
- multi-pending Activity tasks or a new child-simulation protocol.

## Architecture invariants

1. `starclock-combat` contains no Path, Blessing, Curio or Universe IDs.
2. Mode content lowers into generic definitions, selectors, modifiers,
   operations and source bindings.
3. Exceptional handlers live in the mode-owned immutable bundle and return
   ordinary operations/events.
4. Character/content IDs never branch inside shared resolvers.
5. A handler cannot mutate `BattleState` or Activity state directly.
6. Each trigger declares event, phase, priority, once-scope and snapshot
   policy; defaults are forbidden where timing changes behavior.
7. Approximate numbers use checked fixed-point values and explicit rounding.
8. Every RNG choice uses a stable ordered candidate set and labeled stream.
9. Excel is authoritative; JSON is staging/debug output and `.sora` is the
   production artifact.
10. Runtime, accuracy, provenance and bilingual completeness are separate
    coverage dimensions.

## Data and research rules

- Edit production workbooks with Python `openpyxl`; do not patch `.xlsx` as a
  ZIP or introduce a second authoring path.
- Export and validate with pinned Sora 0.3.0.
- Record URL, access date, game version, confidence, note/license and evidence
  hash for every new or corrected fact.
- Use public information only; announced but unavailable or leaked mechanics
  cannot enter enabled runtime content.
- Preserve exact factual values when available. Where only approximate
  numbers are possible, record the formula/range, rationale and confidence.
- Mechanic summaries are independently written and bilingual; do not copy
  long descriptions.

## Partition and commit rules

Fixed `Bnn` batches and generated content `Snn` batches are both atomic
commits. The partition generator enforces:

- at most 16 mechanic-rule bindings per combat content sub-batch;
- at most 32 effect-bearing Occurrence/service choices per noncombat
  sub-batch;
- at most 12 enemy variants per enemy sub-batch, or one multi-phase boss;
- at most one newly admitted native handler per sub-batch unless a shared
  handler intentionally covers the entire partition;
- generated level/metadata rows may travel with their owning mechanic;
- every sub-batch owns Excel changes, Sora output, lowering, tests, provenance
  and coverage disposition together.

The generated manifest freezes ordered IDs, dependencies, focused commands and
expected denominators. A partition may be split further, never silently
merged into an oversized commit.

## Delivery phases

### Phase 0 — Freeze truth and execution partitions

| Batch | Deliverable |
|---|---|
| `G07-P0-B1` | Freeze Goal 07 plan, ledger and prompt; verify the Goal 06 snapshot and inherited 2,201/786/78 denominator. |
| `G07-P0-B2` | Audit every retained row into a mechanic family, shared primitive requirement, evidence gap and intended runtime/accuracy disposition. |
| `G07-P0-B3` | Generate the concrete ordered `Snn` execution partition manifest under the batch-size limits and expand the status ledger. |
| `G07-P0-B4` | Resolve or register public-evidence gaps, freeze approximation policies, performance workloads, dependency baseline and release scaffold. |

### Phase 1 — Shared mechanic capability closure

| Batch | Deliverable |
|---|---|
| `G07-P1-B1` | Complete trigger timing, priority, once-scope, cause-chain and boundary semantics required by all partitions. |
| `G07-P1-B2` | Complete selectors, dynamic subjects, target sets, filters and deterministic candidate ordering required by all partitions. |
| `G07-P1-B3` | Complete modifier stages, stacking, caps, snapshots, derived-stat cycle handling and dynamic invalidation. |
| `G07-P1-B4` | Complete state slots, effects, shields, DoT, Freeze/dissociation-like state, resources, charges and lifecycle operations. |
| `G07-P1-B5` | Complete extra/advance/follow-up actions, reaction scheduling, break/Super Break hooks, resonance actions and wave/phase boundaries. |
| `G07-P1-B6` | Freeze native-handler admissions, shared-mechanic golden probes and machine coverage that prevents a typed evaluator from counting as executable behavior. |

### Phase 2 — Ability Tree and nine Paths

Execute generated sub-batches in this order:

1. `G07-P2-M01-Snn` — shared Activity and Ability Tree;
2. `G07-P2-M02-Snn` — Preservation;
3. `G07-P2-M03-Snn` — Remembrance;
4. `G07-P2-M04-Snn` — Nihility;
5. `G07-P2-M05-Snn` — Abundance;
6. `G07-P2-M06-Snn` — Hunt;
7. `G07-P2-M07-Snn` — Destruction;
8. `G07-P2-M08-Snn` — Elation;
9. `G07-P2-M09-Snn` — Propagation;
10. `G07-P2-M10-Snn` — Erudition.

Each Path milestone closes only when its passive, all Blessing levels,
Resonance, Formations, stacking/upgrade rules and targeted real-battle
fixtures are executable.

### Phase 3 — Curios

1. `G07-P3-M11-Snn` — positive, neutral and special Curios;
2. `G07-P3-M12-Snn` — negative, error, repair and replacement Curios.

Acquisition, active state, charges, disable/repair, replacement, removal,
cross-battle carry and conflict policies must all be tested.

### Phase 4 — Occurrences and services

1. `G07-P4-M13-Snn` — all Occurrence variants and choices;
2. `G07-P4-M14-Snn` — services, shops, respite, roster and interactables.

Every currently deferred HP, roster, reward-selection, battle, special and
shop atom must become an executable operation/task or a declared
`ExternalDecision`. Acknowledging an opaque effect plan is not completion.

### Phase 5 — Enemies, encounters and difficulty

`G07-P5-M15-Snn` partitions cover all referenced enemy definitions, skills,
AI graphs, summons, phases, weaknesses/resistances/toughness, stat curves,
encounter members, waves, difficulty bindings and participant carry.

Approved numeric approximation is allowed when evidence is incomplete.
Generic role/rank proxy behavior is not a released mechanic implementation.

### Phase 6 — Integrated production behavior

| Batch | Deliverable |
|---|---|
| `G07-P6-B1` | Generate targeted mechanic scenarios plus seeded runs covering every executable rule family and every dynamic acquire/upgrade/remove boundary. |
| `G07-P6-B2` | Verify CLI, baseline AI, Agent, MCP and replay-v3 parity from fresh reconstruction with first-divergence corruption tests. |
| `G07-P6-B3` | Harden enemy/player/run AI legality, concurrent sessions, rollback, RNG isolation and long-run resource/charge invariants. |
| `G07-P6-B4` | Freeze focused/full performance, allocation and cache evidence for representative and complete-content workloads. |

### Phase 7 — Audit and release

| Batch | Deliverable |
|---|---|
| `G07-P7-B1` | Run coverage, provenance, bilingual, workbook/Sora drift, dependency/license, native-handler, source-structure and approximation audits. |
| `G07-P7-B2` | Run native cross-platform golden matrices, malformed replay/property tests and the full clean repository gate. |
| `G07-P7-B3` | Freeze Goal 07 documentation/evidence/release contract, commit the final expanded ledger and register the immutable release snapshot. |

## Acceptance

- the P0-generated manifest assigns every inherited record/rule/fixture exactly
  once and all expanded sub-batches are complete;
- 786/786 rules execute through Rule IR, native handler or shared primitive;
- 78/78 semantic fixtures execute against production definitions;
- no enabled runtime row remains `RetainedApproximation`;
- numeric approximations are executable, named and evidenced;
- all nine Path families pass base/enhanced/stacking/resonance fixtures;
- all Curio lifecycle and conflict paths pass;
- no deferred effect atom is silently acknowledged;
- every referenced enemy uses mechanism-correct behavior and deterministic AI;
- all 33 World/difficulty runs complete under replay v3, but route completion
  is not used as a substitute for targeted mechanic coverage;
- adding or removing a mechanic changes only the next eligible assembly and
  battle boundaries;
- malformed commands/data/replays fail without authoritative mutation;
- focused checks stay within budget and final clean full verification passes.

## Terminal checklist

- [ ] Goal 06 immutable release prerequisite passes.
- [ ] The concrete content sub-batch manifest is frozen and fully committed.
- [ ] All 2,201 content records have exact-once runtime and accuracy dispositions.
- [ ] All 786 mechanic rules are executable.
- [ ] All 78 semantic fixtures execute against production values.
- [ ] All Paths, Curios, Ability Tree, Occurrences and services are complete.
- [ ] Enemy, encounter, difficulty and carry behavior is mechanism-correct.
- [ ] CLI, AI, Agent, MCP and replay-v3 parity passes.
- [ ] Determinism, rollback, concurrency, performance and cross-platform gates pass.
- [ ] Full clean-worktree release verification and immutable snapshot registration pass.
