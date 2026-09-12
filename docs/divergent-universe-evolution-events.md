# Divergent Universe Divine Treasures events

The executable `EvolutionEvents` and `EvolutionOptions` worksheets bind the
Green Miracle and Sage's Leaf Robe branches of Divine Treasures II and III to
four events and twelve public choices.
This is a replaceable current-profile policy, not complete implementation of
all Divine Treasures or proof of original room-pool membership.

## Published outcomes

| Event | Choice | Event outcome, separate from Curio acquisition effects |
|---|---|---|
| II | Evolve | Evolve to Awakened; gain 200 fragments |
| II | Request rewards | Keep the treasure; obtain two random Curios |
| II | Sacrifice | Remove the treasure; obtain two random Curios and 150 fragments |
| III | Pay | Spend 100 fragments; evolve to Exalted |
| III | Attempt | 50% chance to evolve; failure retains Awakened |
| III | Sacrifice | Remove the treasure; obtain three random 2–3-star Curios |

Sources 30/31 record the reviewed public detail tables, accessed 2026-09-10:
[Divine Treasures II, revision 95613](https://wiki.biligame.com/sr/index.php?title=%E7%A5%9E%E8%B5%90%E7%8F%8D%E5%AE%9D%EF%BC%88%E5%85%B6%E4%BA%8C%EF%BC%89&oldid=95613)
and [Divine Treasures III, revision 77058](https://wiki.biligame.com/sr/index.php?title=%E7%A5%9E%E8%B5%90%E7%8F%8D%E5%AE%9D%EF%BC%88%E5%85%B6%E4%B8%89%EF%BC%89&oldid=77058).
Fetched HTML SHA-256 values are respectively
`ee8bbf735b0288f50d3859538d5af1bfe9a3caabe3fdca10ec78aa1a7d5f57d9`
and `4a6ef56cb0e117806fa416d85dd5f73b3e1b91a8b907064d25aaebd98d9bd73a`.
The exact frozen handbook/upgrade locators and missing NPC graphs are described
in the [evolution contract](divergent-universe-curio-evolutions.md).

The option values are independent public evidence. Applying them to the current
profile remains policy; no numeric adjacency, shared source table or matching
name is promoted to exact reachability. Community options do not establish the
hidden timing of acquisition effects or original random Curio membership.

## Execution policy

`VersionedProjectPolicyActiveTreasuresAtLayerEntryStableSequence` places II before
the second baseline layer's encounter, and III before the third, in both run
families. Each uses the existing automatic layer-entry node and its logical
scopes; no new Activity state machine or extra physical node is introduced.
Each layer admits at most three distinct treasure owners, sequenced by stable
event key (currently Green Miracle before Sage's Leaf Robe). Eligibility is
re-evaluated after each choice: a reward can enable a later event, but skipped
earlier events are not revisited. Each offered dialogue has a fresh decision ID
on the same node; duplicate dialogue keys are rejected during compilation.
An active predecessor and at least one executable option are required. If no
event qualifies, the initializer proceeds directly to the encounter with no
additional event RNG or player action. These events expose `Service` decisions;
the baseline runner and shared selection replay dispatch them through
`choose_evolution_option`. `offered_evolution_event` exposes immutable definitions
and bilingual option labels, not an authorization to choose hidden options.

The shared generated-choice boundary authenticates definition, graph, current
node, decision, option, state hash and dialogue binding before mutation. Costs
and mandatory successor rewards are rechecked before drawing. Both guaranteed
and chance upgrades require a complete success reward; exhausted Blessing pools
or dirty reward boundaries disable the option, rather than providing a free
failed roll. The generated prefix applies cost, evolution or
inventory exchange, reviewed acquisition effects, then the separate event grant.
The guarded option body completes dialogue/content before opening doors and
offering the next eligible treasure or advancing to the encounter. Raw graph
choices cannot bypass the reward receipt or reuse a preceding dialogue token.
All operations, Reward RNG and subsequent Encounter RNG roll back on failure.

An evolution executes the successor's full immediate grant under its separate
policy. With no global gain modifier, Green Miracle II credits 300 + 200;
III's paid evolution changes the balance by -100 + 600. Sage's Leaf Robe II
grants two Rare Blessings plus 200 event fragments; its paid III grants three
Legendary Blessings and spends 100 fragments. The probability choice
uses one bounded Reward draw, purpose 24001, and succeeds when its zero-based
value is below the authored numerator. Successful Sage evolution then consumes
three distinct Blessing acquisition draws; failure consumes only the chance
draw. Failure is an accepted consumed choice,
retains the old form and does not grant evolution income or offer another try.

Random rewards use stable, uniformly weighted, unowned Curio identities and
canonical source-bound current states without replacement. II's unspecified
rarity uses the explicitly authored 1–3-star policy; III's sacrifice uses the
reported 2–3-star bounds. Ownership includes destroyed states and evolution
aliases. Mandatory acquisition-reward preflight is shared with ordinary Curio
rewards. Insufficient candidate counts disable the option; no smaller reward or
free leave alternative is invented.

Sacrifice snapshots eligibility before removal, keeping the sacrificed identity
out of its immediate replacement pool. Inventory removal and reward insertion
are lowered together, so a subsequent inventory assignment cannot restore the
removed treasure. Acquisition effects and event fragments still use the common
fragment-gain pipeline, including its ordering, independent floor and rollback
rules. Other unimplemented Curio effects remain gaps.

The same ownership alias is now counted by ordinary reward offer predicates and
execution-time candidate checks. A held upgraded Green Miracle cannot make its
base form appear unowned in an offer. Availability sums combine shallow
expression subtrees first, deterministically, to include multi-Path Legendary
Curios within the existing shared expression-depth limit.

Layer/profile binding, active-only eligibility, unspecified rarity, weights,
candidate membership, sacrifice snapshot, grant ordering and probability mapping
are independently replaceable, low-confidence policy fields. Alternatives
include random room placement, weighted rarity, post-removal eligibility and
interleaved effects. Replace each when released graphs or reproducible
current-profile observations establish it. Exact known effects stay separate
from policy and from runtime completeness.

## Scope and verification

Public tests acquire each treasure through the real initial event, execute all
twelve choices across both families, complete three actual battles and verify fresh
selection replay. A bounded public seed corpus requires both probability
branches. Counterfactual balance fixtures cover late overflow, sacrificed
inventory/RNG rollback, disabled unaffordable choices and raw/duplicate choice
rejection. An exhausted-pool fixture checks source-unbound evolution aliases
against both ordinary offer predicates and reward validation. Sage tests exhaust
Rare and Legendary successor pools both before and after offering a choice,
verify no chance draw on rejection, and restore all upgrade/sacrifice draws on
late event-fragment overflow. A controlled two-treasure fixture executes both
dialogues on each shared layer-entry node and rejects old dialogue tokens. Two
fresh factories produce identical state after fourteen actions and three battles;
the injected acquisition is not claimed as natural initial-event selection replay.
Agent tests also execute the two upgrade choices through opaque public tokens,
retry each live-session action idempotently and verify a fresh replay. A separate
counterfactual initial-event overflow regression ensures a rejected reward keeps
the adapter's original offer, observation and unused idempotency key available;
it does not claim the injected balance is a naturally replayable run.

Original Sage's Leaf Robe victory-domain placement, Express Fragment Memorial Plaque, Divine Treasures I's original
producer, original room placement and complete current-profile reachability
remain required work. These partial branches do not terminalize the original event
programs, research gaps or source obligations. See [Goal 22](goals/22-divergent-universe-runtime.md).
