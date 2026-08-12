# Currency Wars runtime

Starclock's Currency Wars implementation is a mode profile over
`starclock-activity`; it is not a second Activity or battle state machine.
`starclock-mode-currency-wars` owns mode terminology, catalog validation,
economy/roster formulas and graph compilation. `starclock-data` privately reads
the generated Currency Wars Sora bundle and returns one immutable mode catalog.
Individual fights remain opaque `BattleSpec`/`BattleResult` handoffs owned by
`starclock-combat`.

## Production data

The production authoring surface is the three workbooks under
`config/currency-wars/data/`. The current Sora bundle lowers:

- 26 routes and 493 ordered nodes;
- 97 difficulty records and Standard/Overclock score-rule identities;
- 77 roster roles, ten offer/team levels and five rarity price rules;
- 49 bond identities: 33 main bonds and 16 source-authored subtrait bonds;
- 834 investment identities across Augments, enhancements, Orbs, Portal buffs,
  Projections and Talents;
- 12 explicit research-gap policies.

Node templates, Stage IDs, node kinds, parameters, penalty/bonus rules, Gold
rewards and next-node references are direct generated fields. Role rarity and
position plus next-level Experience are also direct generated fields. Runtime
lowering does not parse localized summaries or load JSON/Excel.

## Run model

`CurrencyWarsRunDefinition` binds one route, difficulty, Gambit, participant
lock and caller-supplied initial roster/deployment. Every route node compiles to
the shared Activity graph:

- Monster, Camp Monster, Elite Branch and Boss nodes become `Battle` nodes;
- Supply nodes become `Shop` decisions;
- a lost battle enters an automatic `Checkpoint`, subtracts the projected
  Squad HP loss, then continues when Squad HP remains positive or fails at zero;
- a fault enters the ordinary fault terminal; a completed final node enters the
  completed terminal.

The run stores Gold, Experience, team level, Squad HP, last battle metrics,
roster star/count states, deployment positions, active bond levels, current
shop offers and selected investment identities in typed Activity slots. Shop
refresh, purchase/sale, synthesis, deployment, bond recomputation, level-up and
investment selection use typed atomic boundary operations. A rejected command
therefore restores state and RNG together.

Shop candidates are sorted by stable role ID and sampled with the authored
rarity weights without replacement. Three equal-star copies synthesize in
ascending star order. Maximum-star overflow is retained because no released
overflow reward was verified. Deployment is reconciled after synthesis or
sale before bond levels are recomputed once.

Battle settlement carries exact participant HP/Energy/life/presence and expects
the bounded metrics `currency_wars_squad_hp_loss` and
`currency_wars_action_value_remaining`. Victory adds the node's authored Gold
when present and the selected Gambit's authored Experience. The caller remains
responsible for assembling the selected difficulty, deployed resolved builds,
node mechanic contributions and enemy scaling into `BattleSpec`.

## Explicit policy boundary

The bundle retains these unresolved fields as replaceable `ProjectPolicy`, not
observed parity:

- simultaneous bond recomputation;
- exact Camp boss identity;
- configuration-program lowering;
- cross-route carry/reset behavior;
- Gambit route membership;
- Gold currency source identity;
- investment operation order;
- maximum-star overflow;
- offer sampling order;
- automatic Technique rescue;
- role-to-shared-build selection;
- same-boundary Squad HP/action-value ordering.

Investment selection currently records the exact identity and lifecycle family
only. It deliberately refuses to claim exact battle-effect binding. The
replacement condition and alternatives for every policy are available through
`CurrencyWarsCatalog::policies()`.

## Debug surfaces

`CurrencyWarsRun::player_view` and `debug_view` expose the ordinary owned
Activity views, including IDs and configuration-backed mode state. They do not
dereference presentation data in the simulation core.

The CLI provides:

```text
starclock currency-wars config validate [--json]
starclock currency-wars inspect --route ID [--json]
```

The inspector command prints direct node IDs and their referenced template,
encounter, penalty/bonus, Gold and next-node IDs. A future game UI can resolve
those IDs through its own presentation/catalog adapter without changing battle
performance or authoritative state.
