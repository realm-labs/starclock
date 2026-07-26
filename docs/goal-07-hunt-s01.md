# Goal 07 Hunt Partition S01

`G07-P2-M06-S01` completes seventeen assigned content records, sixteen
mechanic-rule records, two semantic fixtures and ten native-handler reviews.
It executes both released levels of Empyrean Imperium, Radiant Supreme,
Sovereign Skybreaker, Skyward Vendetta and Archery Duel. Adept's Bow is
registered as the assigned definition boundary; its effective-level rows and
executable inheritance behavior belong to the following Hunt partition. No
native handler is admitted.

## Authoritative authoring boundary

The editable source remains the openpyxl-authored workbook set:

- `Universe.xlsx` owns the Hunt Path, Blessing, level and exact parameter rows;
- `UniverseBindings.xlsx` owns all sixteen assigned mechanic bindings;
- `UniverseEvidence.xlsx` owns provenance, fixtures and native reviews.

`tools/goal07/author-path-partition.py` reads these workbooks with openpyxl,
rejects formula and error cells, and compares every assigned row with the Sora
0.3.0 binary and debug exports. Runtime materialization reads only the
validated `.sora` bundle.

The released structured rows remain the numeric source of record. Public
descriptions were cross-checked against the
[Simulated Universe Paths reference](https://honkai-star-rail.fandom.com/wiki/Simulated_Universe/Paths)
and the
[Star Rail Station Blessing catalog](https://starrailstation.com/en/simuniverse/current/blessings),
accessed 2026-07-26. These pages corroborate mechanism prose; the locally
hashed structured rows retain numeric provenance.

## Shared Critical Boost state

Critical Boost is one non-dispellable, battle-lifetime effect per character.
Different Hunt Blessings therefore observe and mutate the same stack count:

```text
per stack: +6% CRIT Rate, +12% CRIT DMG
default cap: 8
enhanced Empyrean Imperium cap: 12
```

Effect attachment slots drive both modifiers through the generic
`RecomputeOnStackChange` policy. Blessing rules never copy Critical Boost into
private per-content counters.

At every allied turn start, the complete existing stack count transfers to
the acting ally before turn-start Blessings add new stacks. When an ally is
attacked, the shared effect is removed from the party. Both transitions use
ordinary effect operations and event filters; there is no detached mode-side
Critical Boost store.

Empyrean Imperium (`612430`) grants one stack at the beginning of each
character turn. Its enhanced level changes the shared catalog cap rather than
installing a second effect.

## Defeat and Break action advance

Radiant Supreme (`612431`) observes a credited enemy defeat:

1. accumulate four stacks, or seven at the enhanced level, in owner-scoped
   battle state;
2. after the current normal turn resets its action gauge, advance that
   character by 100%;
3. at the beginning of the resulting next turn, apply all pending Critical
   Boost stacks and clear the pending state.

The resolver now drains `TurnEnded` rules after the ordinary gauge reset and
before selecting the next timeline actor. This is a general lifecycle
boundary, not a Hunt or Blessing branch. It makes current-actor action advance
produce a normal turn rather than an extra turn, preserving ordinary
turn-duration semantics.

Sovereign Skybreaker (`612432`) uses the same boundary after a character
inflicts Weakness Break. It also prepares a one-action effect for the
character's next Attack:

```text
L1: +50% damage
L2: +75% damage
```

The break action is explicitly excluded: the rule records the break, marks the
bonus ready at that action's `ActionResolved` point, applies it at the next
Attack's `ActionStarted` point, and expires it at `ActionEnd`.

The enhanced level tests the broken target through the generic
`ConditionExpr::EnemyRank` predicate. An Elite or Boss target advances every
ally by 100%; no enemy ID appears in the content lowering or resolver.

## CRIT conversion and sustain

Skyward Vendetta (`612440`) contributes a dynamic CRIT DMG modifier only while
Critical Boost exists:

```text
overflow = max(current CRIT Rate - 100%, 0)
base bonus = overflow / 1% * 3%

L1 bonus = clamp(base bonus, 0%, 150%)
L2 bonus = clamp(base bonus + Critical Boost stacks * 0.2%, 0%, 200%)
```

The released enhanced structured value `0.0019999999` is the source
serialization of the publicly stated `0.2%`. Lowering deterministically rounds
it to the authoritative six-decimal numeric domain as `0.002000`; this is not
floating-point runtime arithmetic.

Archery Duel (`612441`) restores `5% MaxHP × Critical Boost stacks` when a
character turn begins. The enhanced level repeats the same ordinary Heal
operation after that character uses an Ultimate. Healing modifiers,
effective-heal clamping and event attribution remain owned by the shared
sustain pipeline.

## Generic core seams

This partition adds one source-agnostic contextual condition:

- `ConditionExpr::EnemyRank(selector, rank)`.

The immutable battle-query snapshot supplies rank from resolved combatant
state. Catalog validation treats it as a current-state selector dependency,
and modifier snapshots treat it as a predicate without hidden numeric reads.

It also fixes the normative `TurnEnded` dispatch boundary. Rules reacting to a
turn end now settle before the next timeline selection in normal, automatic
and skipped-turn paths. Replay ordering therefore matches the actual state
transition rather than publishing a late reaction after the next decision.

## Production verification

Production tests prove:

- all selected S01 effective levels materialize as generic Rule IR;
- enhanced Empyrean Imperium installs the twelve-stack shared definition;
- one real first turn grants one stack and makes Archery Duel heal exactly 5%
  MaxHP;
- a real one-HP enemy defeat advances the killer into the next normal turn and
  grants exactly seven pending stacks;
- enhanced Sovereign Skybreaker contains the Elite/Boss all-ally branch and
  the next-Attack 75% effect timing;
- enhanced Skyward Vendetta retains the 3%-per-1%, 0.2%-per-stack and 200% cap
  constants in one checked expression;
- combat and replay state remain deterministic after the corrected turn-end
  boundary.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M06-S01.json`.
