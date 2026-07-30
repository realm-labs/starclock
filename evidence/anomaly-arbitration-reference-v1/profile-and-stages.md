# Profile and stage evidence

## Released public boundary

The released official gameplay guide identifies Anomaly Arbitration as a
permanent mode with periodically updated stages. It states Equilibrium Level 6
and maximum-star completion of the highest Memory of Chaos, Pure Fiction and
Apocalyptic Shadow stages as participation requirements; those records may
come from different versions. It also identifies three Knight stages, which
may be attempted in any order, and one King stage.

Source: [official gameplay guide](https://www.hoyolab.com/article/41091494),
accessed 2026-07-29 for Version 4.4.

The pinned structured selector closes the active released rotation to group 8,
aliases 801–804 and StageConfig rows 30508011, 30508012, 30508013, 30508021
and 30508022. Exact Chinese and English names come from the referenced TextMap
hashes. Entry IDs are retained only as mechanical locators; account rewards
and presentation remain excluded.

## Explicit uncertainty

Two released public pages render the rotation end as 2026-08-25 and
2026-08-26. The normalized period retains both observations and does not invent
a canonical end instant. A later released structured timestamp replaces this
field-level approximation.

The official guide recommends completing the Knight stages before the normal
King stage but does not state the precise normal-difficulty unlock transition.
The normalized stage uses an independent released
[cross-check](https://honkai-star-rail.fandom.com/wiki/Anomaly_Arbitration) for
the “after all three Knight clears” policy, labels it
`ApproximateFromReleasedText`, and records alternatives and a replacement
condition. The direct Plight alternative and its three-star Knight projection
remain separately backed by the official guide.

No file in this batch lowers these facts into runtime behavior.

## Knight records and team uniqueness

The same released official guide states that the three Knight stages use
different character teams. A successful clear records the team for later
retries. Moving a recorded Light Cone or Relic into another Knight retry
resets the source record; same-team or previously unrecorded equipment instead
allows a retry whose successful result offers an explicit keep-or-replace
choice. Resetting a Knight clears its recorded composition and current result
without deleting Best Battle Records.

Best Battle Records use the highest simultaneous total stars across all three
Knight stages. The detailed star evaluation remains owned by `G13-P1-B6`; this
batch only freezes the separation between mutable current progress and retained
best progress.

Released instructions do not explicitly settle whether alternate Paths/forms
of one character are separate uniqueness identities. The normalized policy
therefore rejects duplicate base character IDs, retains character-plus-Path as
an authored form key, labels that form key `ProjectPolicy`, and supplies a
replacement condition. This is not presented as observed parity.

## King protection and Plight

The official guide says Knight-stage protection greatly enhances the King,
recommends clearing all three Knights to cut their energy transmission, and
states that a direct Plight clear counts as a three-star clear of every Knight
stage. The active structured selector separately closes to normal StageConfig
30508021 and Plight StageConfig 30508022.

Released evidence does not expose numeric protection effects, whether the
protection weakens after each individual Knight clear, or the precise
normal-difficulty unlock predicate. The reference therefore tracks three named
boolean transmissions only for lifecycle audit, records no numeric stacking
claim, and labels per-clear/reset transitions as replaceable policy
boundaries. Normal availability after all three clears remains the same
released-text approximation frozen in `G13-P1-B1`.

The exact Plight projection is limited to three-star result equivalence.
Account rewards stay excluded. Because no Knight battles occurred, synthetic
team snapshots are not fabricated; that fail-closed choice has its own
replacement condition.

## Stage clocks

Pinned structured constants set Knight, normal King and Plight limits to 6, 6
and 2 cycles respectively. The official guide independently states that every
stage combat has its own limit, exceeding it fails the attempt, the first cycle
has increased total action value, the countdown continues across phase changes,
and allies gain an extra combat buff at each cycle start when few cycles
remain.

The public rule is qualitative for first-cycle action value, the low-cycle
threshold and the buff identity/parameters. Those fields remain
`Unavailable`; no assumed 150/200 action value, one/two-cycle threshold or
generic damage buff is authored. Retry creates a fresh stage-attempt clock as a
clearly labeled project boundary. Each unavailable field names its alternatives,
semantic fixture and stronger-evidence replacement condition.

## Arbitral Quadrant

The active alias 804 `BuffList` closes exactly to MazeBuff 3033066, 3033068
and 3033067. Their editable normalized order is numeric, while the offer policy
preserves that selector order. Exact bilingual names, descriptions, canonical
parameters and stage-ability binding keys are retained:

- 3033066, Navigator's Oath / 领航誓言: position-one Skill and Ultimate
  All-Type RES PEN, `0.5`;
- 3033067, Endless Euphoria / 狂欢不息: party All-Type RES PEN `0.2` plus
  Elation-specific `0.2`;
- 3033068, Add Insult to Injury / 落井下石: Follow-Up hits add `0.15`
  damage taken for 2 turns, up to 3 stacks.

The official guide proves that one offered buff is selected before the King
challenge. No-selection rejection and terminal teardown are explicit
attempt-local policies. The fixed ability layout names plugin bodies 0022 and
0023, but the extracted fixed-revision ability list stops at 0021; those two
program bodies remain unresolved without weakening their exact MazeBuff
descriptions or inventing runtime programs. Plugin 0014 resolves in the
extracted ability list.

## Targets, stars and settlement

The active selector closes seven shared BattleTarget rows:

- every Knight uses victory within 4 cycles, victory within 2 cycles and no
  downed characters;
- normal King uses victory within 6, 4 and 2 cycles;
- Plight uses its dedicated victory-within-2-cycles target.

The official guide states that each Knight stage calculates stars independently
at completion and does not combine target results from separate clears. Best
Battle Records retain the maximum total that was active simultaneously across
the three Knights, so current reset/replacement and retained best are separate
projections. The review surface retains three recent periods; structured
locators additionally record 160 retention days and a 14-day expiry warning
without making a runtime wall-clock claim.

`ColorMedalTarget=6` is preserved exactly but not interpreted as either a
six-star threshold or a target ID because no released join defines it. Account
rewards and medal presentation remain excluded.
