# Active battle-event relationships

All five active StageConfig rows carry an explicit `_CreateBattleEvent`
selector:

- the three Knight stages select shared event 30502;
- normal King selects shared event 30503;
- Plight selects shared event 30504.

Every event includes the infinite-summon and Anomaly Arbitration countdown
abilities. Event 30504 also names a hard-boss screen effect; that ability is
classified as presentation-only and no asset is imported. Mechanical event
parameters are preserved as canonical decimal strings.

Events 30502 and 30503 share released bilingual action-bar text: starting from
cycle 3, allies gain a stacking “Middlegame Mayhem” final-damage effect at each
cycle start. Event 30504 has no action-bar TextMap locator. Detailed countdown
program interpretation remains owned by `G13-P2-B3`; this batch freezes only
the exact StageConfig-to-event relationship and event authoring data.
