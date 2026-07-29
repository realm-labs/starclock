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
