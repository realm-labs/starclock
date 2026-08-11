# Retained Event Modes

This document defines the current runtime boundary for Anomaly Arbitration,
Legend of the Galactic Baseballer and Fate/Star Rail NIGHT. They reuse the
shared Activity graph and single-battle Combat aggregate. None owns a second
command processor, timeline, RNG, replay format or mutable global registry.

UI, calendar scheduling, account rewards, dialogue and presentation are out of
scope. A content identity being present in the production bundle does not mean
its battle effect is exact: every effect binding carries an explicit accuracy
flag or `ProjectPolicy` boundary.

## Anomaly Arbitration

`starclock-mode-challenge` owns one five-stage profile:

- three Knight stages, selectable and retryable in arbitrary order;
- normal King, offered only after all three Knight clear slots are set;
- Plight King, directly selectable without completing the Knights;
- three King-only Quadrants represented by stable rule-bundle IDs;
- four Activity-locked teams, with character identities disjoint across the
  three Knight teams;
- max-preserving per-stage clear/star records and exact BattleResult handoff.

The Activity graph has an explicit hub route edge to every battle node. Route
selection stores the stage and optional Quadrant, then relocates to the battle
node. Battle victory returns a Knight to the hub or completes a King route;
loss returns to the hub, and a deterministic battle fault reaches the faulted
terminal. The selected Quadrant must already be compiled into the supplied
King `BattleSpec`; the mode never mutates a live battle to inject it.

The production profile records 6/6/6-cycle Knight limits and 2-cycle King
limits. The public evidence establishes first-cycle ordering but not an exact
authoritative Action Value pair, so the current 150/100 AV windows are an
explicit policy. Plight contributes three tracked King-protection sources, but
the runtime does not invent an unverified numeric protection effect.

## Legend of the Galactic Baseballer

`starclock-mode-baseballer` owns edition-specific immutable catalogs and
preparation rules for Departure and Demon King. The production bundle contains
2 profiles, 13 stage records, 102 stage-period candidates, 87 equipment
identities, 27 synthesis recipes, 114 persistent shop price steps, 56 Adventure
Strategies, 7 stage team bonuses, 2 score rules and 6 explicit policies. Those
policies isolate uniform equipment-offer weights, the no-legal-candidate
branch, simultaneous synthesis ordering, shop transaction ordering and the two
unlowered strategy/team-bonus program families; each records unavailable and
known facts, rejected alternatives, rationale, affected tests, confidence and
a replacement condition.

One stage compiles its authored two to four period ranks to the shared Activity
topology. Multiple candidates at one rank use their exact integer selection
weights:

```text
Battle 1 -> equipment choice -> ... -> Battle N
     |                                  |
     +------ shared failed/faulted terminals ------+
```

The mode validates slot limits, profile membership, stage references,
strictly ordered synthesis inputs and acyclic recipe dependencies. Synthesis
validation is also available over a caller-supplied ordered equipment-level
map. Score settlement clamps negative input, applies the final-stage bonus,
caps the result and selects the highest reached C/B/A/S/SS threshold.

The stage controller owns Activity state for equipment levels and unlocked/used
slot counts. New acquisition and duplicate upgrade are separate checked option
transactions; rejected full-slot and maximum-level candidates cannot mutate
state. Eligible recipes settle in that same option transaction: all
prerequisites validate before consumed inputs are cleared, outputs are added,
slot counts are updated and the route advances. It also owns encounter
preparation, battle-result handoff, accumulated score and deterministic reward
offers.

Persistent shop progression is a separate shared Activity aggregate rather
than mutable mode-local account state. Its canonical slots retain the supplied
Raccoon Gold balance and current level of each shop upgrade. A purchase checks
the exact next level and sufficient balance, then commits deduction, level
increment and shop re-entry atomically. Rejected purchases leave canonical
state byte-identical. Snapshots can seed later stage definitions: the 10
`InitWeaponLevel` price steps and 2 `AddAccessorySlot` steps have exact typed
preparation effects. Purchased MazeBuff levels remain visible with their exact
cumulative decimal parameter strings but are not attached to Combat.

The current production surface still stops before claiming complete event
parity. Exact equipment-offer weighting and all weapon/accessory, Adventure
Strategy, team-bonus and shop MazeBuff battle programs remain unlowered.
`runtime_binding_exact = false` identities are inspectable configuration
locators only and must not be attached to a battle as if their effects were
implemented.

## Fate/Star Rail NIGHT

`starclock-mode-fate-night` owns Case Board topology and the Version 4.4
tactical-card configuration boundary. The production bundle contains 6 boards,
18 board-information rows, 6 card owners, 4 deck profiles, 7 recommendations,
107 cards, 6 story fights, 4 challenge fights, 15 map fights and 16 explicit
policies.

Every board validates unique/reachable nodes and edges, then compiles to a
shared Activity graph. Choice and reward edges use `OptionSelected`; battle
success uses `BattleOutcome(Complete)`. The compiler adds deterministic failed
and faulted terminals so every future battle result has a legal exit. Deck
recommendations retain ordered Servant-owned and Neutral card IDs. A checked
loadout accepts only cards owned by the selected deck's Servant, Rin or the
Neutral pool and rejects duplicate, missing and foreign-owner identities.
Before battle assembly it fails closed on the first identity-only card; a false
binding never becomes an executable no-op.

Released post-launch observation establishes that players build a deck from
Rin, Servant and Neutral cards, receive a random hand, spend magical energy to
play multiple cards, explicitly end the turn when no useful play remains and
retain the ordinary Ultimate system. Those facts do not prove initial/draw hand
size, shuffle/refill/discard ordering or the 107 card programs, so the mode does
not own a second card-battle command processor or RNG stream.

Thirteen identity-only battle-event/target obligations and three mode
boundaries are explicit policies rather than parity claims. The mode boundaries
are:

- the 18 released Case Board information rows are grouped by authored order
  into six choice/battle/completed boards until released edge selectors expose
  exact adjacency;
- every card retains exact ID, owner, magical-energy cost, rarity and optional
  ability-program path, while draw/turn ordering and card effects remain
  non-exact until shared Combat lowering exists;
- the 6/4/15 custom fight rows retain battle-event, map-entrance, enemy, buff
  and optional reward-card locators; battle-event IDs are not reinterpreted as
  `EncounterId`, and legacy `425001..425008` FateActivity stages are excluded.

The catalog, graph compiler and loadout validation are executable, but the
current tree does not claim a playable tactical-card battle controller or exact
battle-effect closure.

## Production data and verification

The editable production inputs are
`config/challenge-runtime/data/AnomalyArbitrationRuntime.xlsx` and the two
workbooks under `config/event-runtime/data/`. Sora 0.3.0 validates them and
owns generated Rust readers, schema locks, debug JSON and binary `.sora`
bundles. Runtime code embeds only the generated binary bundles.

The CLI checks both boundaries without starting a battle:

```text
starclock challenge config validate [--json]
starclock event config validate [--json]
```

Mode tests cover locked/direct King routing, Knight roster uniqueness,
data-driven Baseballer period topology, production controller compilation,
checked equipment options, converging and cyclic recipe graphs, score caps,
atomic shop purchases, rejected-purchase state identity, progression-to-stage
seeding, Fate graph lowering, deck/card ownership, fail-closed loadouts and
production denominators. Exact end-to-end battle fixtures are required before
any currently false effect binding can become true.
