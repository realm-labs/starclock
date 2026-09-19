# Divergent Universe Equation-expansion rewards

## Current boundary

`EquationExpansionRewards` is an openpyxl-authored production decision table.
Sora exports one typed definition for Tawot state 9074 / effect 2074; the data
compiler validates it against the current reference state, parameter positions,
declared allowance, exact reviewed source keys and explicit policy. It rejects
same-count substitutes such as victory state 9072 and domain-entry state 9075.

The current [Equation transition planner](divergent-universe-equation-grants.md#expansion-transition-boundary)
feeds a bounded reward queue in the originating inventory transaction. Accepted
Blessing acquisitions/replacements/path rewrites, normal battle choices,
occurrence/Curio/victory grants, Equation acquisition/replacement and the vertical
slice use the same executor. This is a partial executable mechanic, not full
public-run parity or terminal source-program coverage. Paid public acquisition
followed by expansion-trigger execution now has encoded three-battle baseline
replays in both families, using only offered choices. The explicit projection-only
`refresh` API still reports derived
transitions; it is not a second reward command.

## Released facts

Decision sources 61–63 bind released Version 4.4 revision
`fd978d6ef09f941fba644c731ab54abd6f7c3568` of
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
accessed 2026-09-12:

- `ExcelOutput/RogueTournMiracle.json`: state 9074, Tourn3, handbook 9068,
  effect 2074; SHA-256
  `176b17030bdf212920d8a59142cf982916349ee986b574b3e2d3bbdeff81c438`.
- `ExcelOutput/RogueMiracleEffect.json`: effect 2074, `ParamList[0]=1`,
  `ParamList[1]=3`; SHA-256
  `fd40b0b35b2735875356681597a532537fb4a98fbcddded501e8d4b4e3eede35`.
- `TextMap/TextMapEN.json`: description hash `1970395831290142958`; SHA-256
  `afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789`.

The text establishes one random Blessing after an Equation expansion and
discard after three triggers. The reference's broad `PassiveWhileOwned`
classification does not establish trigger timing. Hidden distribution, cascade
scheduling and lifecycle edge cases are not released facts established here.

## Authored policy and implementation requirements

`VersionedProjectPolicyActivePreStateUniformUnownedBoundedCascade` declares:

1. Snapshot active ownership and positive remaining allowance before the
   originating command. Acquisition does not reward existing expanded Equations.
   Destruction pauses, repair resumes, and replacement/reacquisition refill.
2. Coalesce the command's final holdings, including Trailblaze Wax grants.
   Queue newly expanded identities in stable order. Enhancement without an
   identity change adds no edge; collapse followed by re-expansion can qualify.
3. Consume one allowance per queued edge, including an exhausted reward pool.
   Uniformly select an unowned current base identity across the policy-selected
   1–3-star pool; enhanced holdings count as owned. Reserve identities in unrelated
   pending offers. Do not inherit battle suppression or sealing-wax offer weights.
4. Use shared Reward RNG purpose 24141. Each granted identity updates recipe
   contributions and appends newly expanded identities to a FIFO queue. The
   remaining allowance bounds the cascade. Discard the holding and all its
   counters on the limiting trigger, before processing further queued work.
   Reward steps compare adjacent expansion sets, so a previously expanded
   Equation collapsed by the originating rewrite can qualify when a reward
   restores it within that same transaction.
5. Ownership, progress, trigger allowance, discard, events and all random draws
   commit in the originating authenticated Activity transaction or all roll back.

These fields have low confidence. Alternatives include weighted rarity,
independent suppression, intermediate-grant triggering, no reward-induced
cascades, or no allowance consumption on an empty pool. The selected policy
preserves finite expansion rewards and deterministic sampling. Released programs
or reproducible current observations replace each field independently.

Mandatory Curio Blessings are sampled as one complete distinct batch and lowered
after stable-order fragment effects. Equation-plus-Wax acquisition feeds its
final Blessing holdings into the reward executor once. Neither attaches rewards
independently to intermediate progress operations. Curio-acquiring/evolving
commands merge the allowance delta into their proposed final holdings before
lowering final counter maps. A simultaneous removal never resurrects the old
holding; other newly acquired Curios survive an expansion-triggered discard.

Execution fixtures cover simultaneous expansions, a reward-induced second
Equation using released recipes, empty/reserved pools, lifecycle pause/repair,
enhancement, collapse/re-expansion, late rollback, activation overflow, mixed
Curio acquisition with limiting discard, and accepted replacement/path rewrite.
Public initial Equation choices in both families and internal Equation offers
are tested with explicitly prepared holdings; those holdings are not a public
starting loadout. Fresh deterministic reconstruction is checked for the controlled
reward vectors. Exhaustive producer/lifecycle combinations and full original
room topology remain separate release requirements.

## Public paid trigger regression

Fixed current-configuration vectors start with an offered Epic Equation, select
the initial Color event's two-Curio reward, and obtain Sage's Leaf Robe and Green
Miracle. Green Miracle's actual acquisition grant funds the 100-fragment purchase
of offered state 9074 at the explicitly admitted level-two Tawot service. Both
treasures then take their public guaranteed upgrades; the two later domain
choices select Elite. Normal Blessing choices prefer an available contribution
to the selected Equation's still-deficient Path.

- Ordinary seed 1315013, third initial Equation: Sage's third-stage acquisition
  grants complete the recipe at its public evolution service.
- Cyclical seed 168745, second initial Equation: the third battle's Sage victory
  grants complete the recipe at verified settlement.

Each vector checks one extra unowned Blessing, one additional Reward draw,
expanded recipe state, activation count one and remaining allowance two before
terminal scope disposal. Other acquisitions/evolutions retain the paid card.
Three real battles and every accepted choice are recorded, encoded, and replayed
against a fresh production fixture with matching final canonical state hash.
Terminal scope disposal clears the inventory; that clearing is not counted as an
expansion or a limiting discard. The explicit ignored seed-discovery test is not
part of the default loop; the fixed regression vectors require no seed search.

This proves the current authored baseline and its versioned policies, not the
original Forge placement, complete room topology, or public three-trigger
exhaustion. Those boundaries are not promoted by these tests.
