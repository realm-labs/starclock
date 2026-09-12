# Divergent Universe Curio battle reactions

The decision workbook's `CurioBattleReactions` table binds Tawot Cards state
9069 to attacked-ally healing of 20% maximum HP and a five-battle discard limit.
This is distinct from state 9068's SPD bonus and states 9070/9079's domain clocks.
It is not an entry heal.

## Released inputs and explicit policy

Sources 46–48 in `DivergentUniverseDecisions.xlsx` pin released 4.4 data from
`Dimbreath/turnbasedgamedata` at
`fd978d6ef09f941fba644c731ab54abd6f7c3568`, accessed 2026-09-12:

| Input | Exact locator |
| --- | --- |
| Mode state | `RogueTournMiracle.json`: 9069, `Tourn3`, handbook 9068, effect 2069 |
| Parameters | `RogueMiracleEffect.json`: 2069, `ParamList[0]=0.2`, `[1]=5` |
| Released English text | `TextMapEN.json`: hash `7916500230569937913` |

The workbook retains exact input digests and validates state/effect/parameter
joins. Magnitude and battle limit are exact. Trigger granularity, applicability,
formula modifiers and terminal counting are not established by those facts.
`VersionedProjectPolicyFirstSurvivingOrdinaryAttackPerTargetAction` owns these
lower-confidence choices:

- An active holding with positive allowance attaches one source-attributed
  run passive at immutable battle assembly. The lowest-formation player anchors
  the rule; its defeat does not disable the run-owned reaction. Original build
  inputs, existing bindings, locked build identity and enemy specs are retained.
- Opposing-side Ability ordinary damage with an action identity triggers at
  shared `AfterEvent`, once per target per action. The first eligible hit heals
  before later hits; a new action receives a fresh allowance. Fully shielded
  hits count. DoT, additional/true/break damage, HP costs and friendly damage
  do not qualify.
- The attacked ally must still be alive and Present, Untargetable, Linked or
  Transformed when evaluated. Reserved/departed/dead targets are excluded.
  Healing never revives a defeated target. The passive does not construct
  summons or linked entities.
- Healing is `floor(current maximum HP × 0.2)`, using the target rather than
  the attacker or anchor. It bypasses outgoing/incoming healing modifiers and
  uses shared HP capping and event attribution. No RNG stream is consumed.
- Verified `Won` and `Lost` results consume one active battle allowance in
  the existing atomic [lifetime settlement stage](divergent-universe-curio-battle-stats.md).
  Faulted, rejected and duplicate results do not consume another allowance.
  Domain choices do not count. Destruction pauses contribution and consumption;
  repair resumes the remainder. Replacement/reacquisition initializes five.
  Exhaustion removes state, charge and activation entries rather than marking
  the Curio destroyed.

Replace each uncertain field independently when released executable evidence
or reproducible observations resolve it. Alternatives include after-action
healing, per-hit healing, roster-only scope, healing boosts and victory-only
counting. The current choices are not asserted as observed game parity.

## Execution and remaining boundaries

Both Ordinary and Cyclical tests use the production Sora catalog, accepted
holding boundary and production battle passive assembly. Controlled opposing
attacks verify multi-hit and next-action behavior, actual HP changes, shields,
dead anchors, linked-presence selectors, excluded damage, lethal hits and fresh
deterministic reconstruction. Destruction, repair, zero allowance and verified
battle-result teardown are covered separately. Five-count operation vectors
are not five public battles.

The generic reward pool still selects the lexicographically first state per
handbook, which is 9068 rather than 9069. The separately admitted
[Tawot service](divergent-universe-tawot-service.md) now provides actual paid
acquisition of 9069, plus same-state repurchase and a real battle-counter test.
Original Forge placement, exact state-pool selection, production summon
construction and full public five-battle exhaustion remain pending. This component does not close an
original mechanic program or source obligation.
