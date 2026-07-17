# Toughness, Break and Super Break Runtime Boundary

`G01-P4-B4` makes Toughness an authoritative, generic combat subsystem. It
does not encode Firefly or any other character identity in the resolver. The
dedicated Firefly V1a workbook is source-bound executable design evidence and
remains disabled `ProjectFixture` content with zero production coverage.

## Ordered layers and weakness routing

Each resolved combatant may carry an ordered collection of
`ToughnessLayerSpec` values. A layer retains its key, family, maximum/current
raw Toughness, active and locked state, element-eligibility policy,
reducible-while-broken policy, recovery ratio, depletion behavior, optional
Break-element override and Break event-source policy. Reduction selects the
first eligible layer. It clamps to that layer's current value and never spills
overflow into another layer implicitly.

Ordinary, Exo-Toughness, sequential and shared layer identities use this same
state model. Provider-specific routing must therefore be explicit data; an
unverified provider cannot inherit ordinary-bar behavior. A layer may retain
the hit source or replace Break event attribution with a typed provider
`SourceDefinitionId`. Defeat credit remains the applying unit.

Permanent and target-turn-scoped weaknesses are separate canonical state.
Adding a weakness does not alter RES. A timed weakness is available to later
operations in the same hit, counts down at the affected target's turn start,
and emits removal when its authored lifetime ends.

## Fixed-point formulas and lifecycle

All authoritative calculations use checked integer/fixed-point values. Raw
Toughness units are typed separately from HP damage. The reduction calculator
applies base plus additive reduction, reduction increase, capped Weakness Break
Efficiency plus Toughness vulnerability, and the authored ability multiplier.
Events retain attempted and effective reduction for each target.

Layer depletion performs the layer's authored initial Break damage, applies the
universal 2,500 Action Gauge delay, updates global broken state when requested,
and resolves the base Break effect chance. Probability zero and one consume no
RNG. Intermediate probabilities use the battle's purpose-tagged stream. The
seven base element plans retain their duration, damage, stack, delay, speed and
action-skip behavior as separate effect instances. Effect damage, expiration,
speed restoration and layer recovery run at the target's turn boundary. A
lethal turn-start effect settles victory/loss before another actor is selected.

Super Break is a distinct operation and formula family. It requires the target
to be globally broken and consumes the effective per-target reduction recorded
by the preceding Toughness operation in the same hit. Empty or ineligible
layers therefore record a zero sample and emit `SuperBreakSkipped`; attempted
overkill is never substituted for effective reduction.

The canonical state codec includes every layer policy/current value, active and
temporary weaknesses, retained base Break effects and their event source, and
the added sequence state. Read-only views expose layers and active Break
effects without exposing storage or fixed-point implementation types.

## Rule IR and Firefly probe evidence

The closed Rule IR and Sora lowering surface now covers weakness add/remove,
Toughness reduction, initial Break, Super Break and layer create/remove
operations. Production loading still accepts only generated Sora readers and
domain conversion; there is no JSON-direct runtime path.

The isolated Firefly probe exports through pinned Sora 0.3.0 to bundle
`62d6b0efaa55193d7e344469ccd0a0b61d147310a3f1cf9b796f8ec2b37f73ba`.
Its enhanced Skill text is bound at
`a48558d7d5a6f8967142a2d7bf7064e23f55a40ed0f175c6bc3da6377cfcae54`
and Autoreactive Armor at
`25fe591020471c13818909321e8a9731eea1e2b80b96c977d3d1fa9d31d7d568`.
The generated program orders two-target-turn Fire weakness before 90 raw
Toughness reduction and its 50% Super Break branch. Battle goldens prove
effective samples of 50 for the ordinary layer, 40 for the Exo layer while
globally broken, and zero after both layers are empty.

## Validation evidence

The batch-specific evidence is executable through:

- `node tools/config-probes/verify-firefly-damage.mjs`;
- `cargo test -p starclock-combat --test toughness_formula`;
- `cargo test -p starclock-combat --test damage_lifecycle`;
- `cargo test -p starclock-data probe_tests`;
- `node tools/goal-research/generate.mjs --check` and
  `node tools/goal-research/verify.mjs`;
- `node tools/goal-coverage/generate.mjs --check` and
  `node tools/goal-coverage/verify.mjs`.

The universal repository runners, workspace format/lint/test gates and Windows
ARM64 combat/rules compile-only gate remain required. Probe identities are not
production `DataReady` rows and do not change the frozen Version 4.4 coverage
total.
