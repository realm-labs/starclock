# Goal 17 Memory Turbulence audit

- MazeBuff / BattleEvent: `3030146` / `30146`
- Ability: `BattleEventAbility_Challenge_Month_46`
- Damage boost: 0.5 for Ultimate and Follow-Up ATK
- Stored-hit gain/cap: 1 per qualifying action / 15
- Cycle-start execution: one random enemy retarget per stored hit
- True-DMG coefficients by source rank branch: 0.12 / 0.02 / 0.012 of target BaseHP
- Program operation kinds: 25
- Frozen obligations: 2/2, each claimed exactly once
- Normalized Turbulence digest: `a1db0c72b6f24cd0aa05ebf1e7d017ecbe7e03e6cea614ae44367d3413d24836`
- Runtime executable rows: 0

Trigger filters, once-per-action accumulation, cap, callback placement, random
retarget location, BaseHP source, rank branches and accumulator reset are exact
program projections. Candidate ordering, RNG label, fixed-point rounding,
empty-target fallback and teardown are explicit ProjectPolicy with replacement
conditions.
