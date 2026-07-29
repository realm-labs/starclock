# Currency Wars Version 4.4 Squad Boundary

## Scope

This dossier records the `G12-P1-B2` Squad HP, action-value, battle-result and
run-failure boundary. It is reference data only. It does not lower these rules
into combat, Activity, CLI, Agent or MCP behavior.

## Released facts

The pinned EN/CHS TextMaps publish the following facts independently:

- `TextMapEN.json#1912261755023964838` and the matching CHS locator set initial
  Squad HP to exactly `100` and state that remaining Squad HP determines the
  final match rating.
- `#6971100623138337968` says that victory at Battle, Encounter or Boss Nodes
  leaves Squad HP unchanged; non-victory loses a configured amount; reaching
  zero loses the match.
- `#7693488975416237801` says finite combat Nodes require every enemy to be
  defeated before the action-value limit, otherwise Squad HP is lost, and the
  match concludes at zero. It also says a character's lethal hit is rescued at
  the cost of reducing the remaining combat countdown; `G12-P1-B4` owns the
  detailed battle contribution.
- `#5626677263404827289` identifies a low-difficulty Node whose action value is
  unlimited.
- `#4983101780975847570` proves that content can restore authored Squad HP
  after the final boss battle.
- `#7940111314490605947` proves that content can change Combat- and Boss-Node
  countdowns by separately authored parameters.

Each normalized row carries both locale receipts with the pinned repository
revision, game version, locator, access date and SHA-256 digest. The files do
not copy story or presentation prose.

## Exact and policy-bound fields

The global initial current HP is exact at `100`, the lower terminal boundary is
exact at `0`, victory preservation is exact, and the unlimited low-difficulty
exception is exact. The fixed snapshot does not publish one global finite
action-value number or one global timeout-loss number. The normalized records
therefore use `ConfiguredByNodeOrDifficulty`, allowing later encounter and
content batches to bind exact values without rewriting the lifecycle.

The source says some effects increase or restore Squad HP and Max HP but does
not expose a distinct initialization field for maximum HP. The pack uses the
replaceable policy that initial maximum equals initial current HP (`100`).

The source also does not publish whether a last enemy defeated on the exact
boundary that exhausts action value is victory or timeout. The deterministic
reference policy is:

1. determine victory before timeout loss;
2. project victory preservation or configured non-victory loss;
3. clamp Squad HP to zero; and
4. fail the run at zero, otherwise continue.

The rejected pending-evidence alternatives are timeout precedence and checking
run failure before battle-result projection. This is recorded as
`ProjectPolicy`, not observed parity, and must be replaced by a released state
program or reproducible observation.

Remaining action value is likewise captured for authored finalization
contributions and then discarded rather than carried as a global run resource.
That projection remains policy-bound until a released state/config record
publishes the exact boundary.

## Accounting

The one frozen `squad_hp_action_value_envelopes` obligation is accounted by
`squad-hp-rules.json`. Six derived rows cover two action-value variants, two
battle outcomes and one run-failure rule. Only the exact unlimited-node
exception is `DataReady`; the parent obligation remains `Researched` until
node/difficulty values and same-boundary timing replace the open policies.

## Reproduction

```text
node tools/currency-wars-reference/import-squad-boundary.mjs \
  --source-cache <turnbasedgamedata-repository>
node tools/currency-wars-reference/import-squad-boundary.mjs --check \
  --source-cache <turnbasedgamedata-repository>
node tools/currency-wars-reference/verify-squad-boundary.mjs \
  --source-cache <turnbasedgamedata-repository>
```

The verifier rejects invented global numeric limits/losses, missing bilingual
receipts, runtime-lowered claims, same-boundary policy drift and unexpected
`DataReady` promotion.
