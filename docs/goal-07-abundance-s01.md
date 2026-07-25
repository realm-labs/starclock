# Goal 07 Abundance Partition S01

`G07-P2-M05-S01` completes seventeen content records, sixteen mechanic-rule
records, one semantic fixture and ten native-handler reviews. It executes both
levels of the first five assigned Abundance Blessings. The definition row for
Salvation From Damnation is retained here; its two executable levels belong to
the following partition. No native handler is admitted.

## Authoritative authoring boundary

The formal source remains the openpyxl-authored workbook set:

- `Universe.xlsx` owns the Abundance path, Blessings, levels and exact
  parameters;
- `UniverseBindings.xlsx` owns the sixteen mechanic bindings;
- `UniverseEvidence.xlsx` owns provenance, the production fixture and native
  reviews.

`tools/goal07/author-path-partition.py` reads those production workbooks with
openpyxl, rejects formula/error cells and compares the assigned rows with the
committed Sora 0.3.0 binary and debug exports. Runtime materialization reads
only the validated `.sora` bundle.

## Dewdrop

Perennial Prosperity, Lush Longevity (`612330`) charges Dewdrop by the
effective healing received by each character. Its enhanced level also raises
the eventual rupture damage by 40%.

Mudra of Blessing (`612331`) charges Dewdrop at the start of the owner's turn:

```text
L1: 60% current HP
L2: 70% maximum HP
```

Both generators use one owner-scoped battle slot. Charge is additive and
capped at that owner's maximum HP. The Blessed Many (`612341`) adds 80% charge
efficiency at level 1 or 120% at level 2 while the owner is at full HP; the
same maximum-HP cap still applies.

After the owner resolves an Attack, the complete charge ruptures on one
stable-random member of that action's committed target list. The additional
damage:

- inherits the triggering attack's element;
- cannot CRIT;
- can defeat the target;
- consumes all Dewdrop charge after emitting the rupture signal.

The action target list is retained by `ActionResolved` and committed by the
canonical replay event codec. `RuleSelectorOrigin::EventTargets` restricts the
candidate pool before the labeled `bounce-target` draw. This is a shared Rule
IR capability and contains no Abundance-ID branch in combat core.

## Healing reactions

Sin Dead, Grace Born (`612340`) heals the Dewdrop owner for 20% of the charge
that just ruptured:

```text
L1: maximum 18% MaxHP
L2: minimum 12% MaxHP, maximum 24% MaxHP
```

All Abundance in One Mind (`612332`) shares 30% of effective healing with
every other living present ally. Heals created by the Blessing itself are
source-filtered, so they do not recursively fan out.

At level 2, the same effective-heal amount also grants every ally flat ATK
equal to 15% of that amount. Each target's accumulated value is capped at 80%
of base ATK and expires at the end of the healed owner's next turn. The
implementation uses owner-scoped Rule IR state plus an ordinary dynamic
flat-ATK effect; equipment and character code do not know about the Blessing.

## Deferred definition

Salvation From Damnation (`612342`) contributes its complete bilingual
definition and binding metadata in this partition. Its level records and
executable behavior are assigned to `G07-P2-M05-S02`, so this batch does not
partially activate it.

## Shared combat additions

This partition extends generic combat behavior with:

- scalar reads of current HP and rule-signal payloads;
- event-element additional-damage operations;
- selector exclusion predicates;
- ordered event-target candidate pools;
- stable random selection from a resolved action's committed targets;
- zero-valued effect-stack queries for valid units without that effect.

All arithmetic remains checked fixed point. Missing event element or invalid
subject data faults transactionally. Informational signals retain their
current target in the cause chain, and replay event commitment includes the
resolved action targets.

## Production verification

The production Bailu form and formal Goal 07 materialization prove:

- all five executable Blessing families materialize from Excel/Sora;
- level-2 Mudra charges exactly 70% MaxHP and clears after rupture;
- Sin Dead heals exactly 20% of that charge inside its 12%-24% bounds;
- full-HP level-2 efficiency reaches, but cannot exceed, 100% MaxHP charge;
- rupture damage uses the action element and a labeled random event-target
  selector;
- shared healing does not recurse and its flat-ATK effect reaches all four
  allies;
- all ten exceptional candidates close as `IrSufficient`.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M05-S01.json`.
