# Goal 07 Curio Partition S01

`G07-P3-M11-S01` executes the first eight positive, neutral and special Curios
from the authoritative Standard Simulated Universe workbook:

- Dimension Reduction Dice (`universe.curio.1`);
- Silver Coin of Discord (`universe.curio.102`);
- Family Ties (`universe.curio.104`);
- Black Hole Trap (`universe.curio.106`);
- Interastral Big Lotto (`universe.curio.107`);
- Doctor's Robe (`universe.curio.11`);
- Gossip (`universe.curio.110`);
- Tonic of Efficacious Chaos (`universe.curio.111`).

The partition owns 16 records, 16 rules and five frozen semantic fixtures. Its
authoritative rows remain in `Universe.xlsx`, `UniverseBindings.xlsx` and
`UniverseEvidence.xlsx`; `author-curio-partition.py` compares those openpyxl
rows with the committed Sora 0.3.0 export.

## Runtime model

Curio ownership, state, charges and event tokens stay in Activity state. Only
the immutable combat contribution crosses the battle-assembly boundary.
Destroyed-Curio count is captured alongside the owned Curios, so Family Ties
does not query mutable run state from inside battle resolution.

Dimension Reduction Dice uses conditional random-offer cardinality and a
bounded pending-choice token. While it has a charge, a Blessing reward offers
one fewer candidate but allows one additional selection. Completing the second
selection consumes one charge. The second completed trigger tears down the
Curio and increments the shared destroyed-Curio counter in the same Activity
transaction.

Silver Coin grants half of the pre-acquisition fragment balance. Gossip is one
shared multiplier over positive fragment-gain expressions, so Silver Coin,
Curio grants and the Black Hole Trap settlement cannot drift into separate
rounding or overflow policies. Gossip also bypasses the postcombat Blessing
reward and traverses to the Formation gate through the ordinary graph program.

Black Hole Trap executes after a won nested battle. The settlement counts
returned participants whose current HP equals maximum HP, projects the formal
Curio event, consumes its event token and credits the checked fragment result.

Doctor's Robe materializes full Path Resonance Energy and a 40% Resonance
damage ratio in the battle input. Family Ties installs a 30% source damage
modifier per destroyed Curio. Tonic adds the Technique action tag to the
selected prebattle ability, then applies 200% Technique DamageBoost and a flat
pre-multiplier amount equal to 200% of the actor's maximum HP.

## Interastral Big Lotto evidence boundary

Public evidence describes only a small chance and does not publish a reliable
numeric probability. Starclock therefore does not author a guessed RNG
threshold. `resolve_destructible_lottery` accepts one replayable external
outcome:

- `NoEffect`;
- `Blessing(BlessingId)`;
- `Failure`.

A blessing outcome uses the ordinary validated Blessing acquisition program.
Failure atomically destroys the Curio, increments the destroyed-Curio counter,
sets Technique Points to zero and clears every carried participant's Energy.
The supplied outcome, state hash and resulting Activity events are replay
verifiable. Replacing this external decision with internal RNG requires new
authoritative probability evidence and a revisioned policy.

## Shared primitives added

This partition adds no native handler. It extends shared runtime vocabulary
with:

- checked Activity multiplication and integer division;
- conditional random-offer size and suppression;
- atomic carried-participant Energy assignment;
- a state-hash-checked Activity boundary program;
- an explicit Technique ability tag;
- flat ordinary-damage formula input before multiplicative stages;
- a destroyed-Curio contribution field whose zero value preserves the prior
  digest.

`standard-universe-entry-v7` records the new authoritative Technique Point
slot. The version change intentionally refreshes deterministic entry and run
goldens.

## Executable evidence

The combat integration tests execute Family Ties, Doctor's Robe and both Tonic
terms in real production battles. Curio Activity unit tests execute Silver
Coin, Gossip and both Dice charges through the production transaction
evaluator. Existing nested-battle and run suites execute the after-battle
settlement, reward graph, replay and entry-revision paths.

The unpublished Big Lotto probability remains the sole explicit external
decision in this partition. All released numeric values used by the other
seven Curios are exact workbook parameters.
