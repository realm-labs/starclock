# Gold and Gears V1 Normalized Reference

This directory is the Goal 08 JSON research/staging representation for the
Version 4.4 Gold and Gears reference pack. It is not an authoring surface and
is never loaded by runtime code. The authoritative authoring form is the
isolated Excel workbook set; Sora 0.3.0 owns schema export.

Phase 1 topology files regenerate with:

```text
node tools/gold-and-gears-reference/import-topology.mjs
node tools/gold-and-gears-reference/verify-topology.mjs
```

Phase 1 Cognition, Secret-condition and mode-constant files regenerate with:

```text
node tools/gold-and-gears-reference/import-cognition.mjs
node tools/gold-and-gears-reference/verify-cognition.mjs
```

Phase 1 Custom Dice definition and selected-Path boost files regenerate with:

```text
node tools/gold-and-gears-reference/import-dice-definitions.mjs
node tools/gold-and-gears-reference/verify-dice-definitions.mjs
```

Phase 1 Dice slot, face and filter-tag files regenerate with:

```text
node tools/gold-and-gears-reference/import-dice-faces.mjs
node tools/gold-and-gears-reference/verify-dice-faces.mjs
```

Phase 1 Knowledge binding and deterministic target/order policy files regenerate
with:

```text
node tools/gold-and-gears-reference/import-knowledge.mjs
node tools/gold-and-gears-reference/verify-knowledge.mjs
```

Phase 1 Neural Network graph, cost and effect rows regenerate with:

```text
node tools/gold-and-gears-reference/import-neural-network.mjs
node tools/gold-and-gears-reference/verify-neural-network.mjs
```

Phase 1 Stats/Auxiliary Conundrum composition and effect rows regenerate with:

```text
node tools/gold-and-gears-reference/import-conundrum.mjs
node tools/gold-and-gears-reference/verify-conundrum.mjs
```

Phase 1 Path, shared Resonance, Path Boost, Resonance Extrapolation,
Resonance Interplay and Trailblaze Bonus rows regenerate with:

```text
node tools/gold-and-gears-reference/import-paths.mjs
node tools/gold-and-gears-reference/verify-paths.mjs
```

The inherited shared Blessing pool and both authored levels regenerate with:

```text
node tools/gold-and-gears-reference/import-blessings.mjs
node tools/gold-and-gears-reference/verify-blessings.mjs
```

Gold and Gears Curio identities, mode copies and lifecycle bindings regenerate
with:

```text
node tools/gold-and-gears-reference/import-curios.mjs
node tools/gold-and-gears-reference/verify-curios.mjs
```

Occurrence identities, reachable Gold variants and derived mechanical choice
graphs regenerate with:

```text
node tools/gold-and-gears-reference/import-occurrences.mjs
node tools/gold-and-gears-reference/verify-occurrences.mjs
```

Shared currencies/services and abstract Adventure reward tiers regenerate with:

```text
node tools/gold-and-gears-reference/import-services.mjs
node tools/gold-and-gears-reference/verify-services.mjs
```

Every row carries bilingual mechanical text, explicit ownership and coverage,
and ordered row-level source references. `map-edges.json` is deliberately
`ProjectPolicy`: released chessboard configs contain nodes and coordinates but
no explicit edge relation. Cognition ranges and Secret thresholds remain
`ExactStructured`; their embedded adjustment/clamp/carry/reset order is a
replaceable `ProjectPolicy`, with the released plane-boss evaluation boundary
recorded separately as public evidence. Neural Network rows preserve the
released 40-node prerequisite graph, costs and contributions. The two
indistinguishable Blue-slot upgrade targets and reroll empty-candidate behavior
use named replaceable policies rather than claiming an unpublished engine
mapping. Conundrum track composition, caps and released semantic effects are
exact; unpublished enemy-stat ratios, Berserk timing/stack values, Toughness
ratio and action-advance amount remain one named fail-closed numeric policy.
Shared Path/Resonance identities remain unchanged. Released BattleEvent groups,
Interplay thresholds and Path Boost properties are exact; the unpublished
generic Extrapolation selection/action/polarity controller is a named
fail-closed policy.
