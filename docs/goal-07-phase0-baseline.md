# Goal 07 Phase 0 Baseline

Phase 0 freezes the complete Goal 07 denominator and the rules under which
missing public numeric evidence may be handled. Verify the baseline with:

```text
node tools/goal07/verify-phase0.mjs
```

## Evidence and external decisions

The retained audit found 52 Occurrence-choice records whose outcome depends on
player input, a presentation-space interaction or a minigame outside the
headless simulation. They are not opaque effects. Each is registered as an
`ExternalDecision` and its generated content partition must provide:

1. an ordered set of legal outcomes;
2. an explicit result command;
3. validation before authoritative mutation;
4. atomic lowering of the chosen cost, effect and transition; and
5. replayed controller input and deterministic reconstruction.

The other audit gaps are implementation work owned by the 104 generated
partitions. If public evidence is insufficient to reproduce a mechanic, the
partition blocks and records the missing evidence rather than guessing.

## Numeric approximation

Seventy-three enemy variants may use
`goal07-public-anchor-level-curve-v1` for missing base numeric fields. This
policy does not approve a generic enemy proxy. It permits only HP, ATK, DEF,
SPD, Effect Hit Rate and Effect RES to be calculated from a recorded public
anchor and the canonical enemy level curve.

Skills, AI, phases, summons, weaknesses, resistance overrides, Toughness and
triggers must remain mechanic-correct. Each approximated field records its
anchor, inputs, curve revision, result, confidence, source and evidence hash in
the production configuration. The candidate remains pending until those
per-field inputs and targeted mechanic fixtures exist.

## Performance and dependency baseline

Six stable workload identities cover focused daily acceptance, complete
catalog loading/lowering, all 786 targeted rule scenarios, all 33 seeded
World/difficulty runs, concurrent sessions and a long-run resource/charge
soak. Focused iteration has a hard 180-second ceiling. Strict elapsed and
allocation budgets are measured and frozen in `G07-P6-B4`, once the complete
dataset exists.

Phase 0 adds no dependency. It reuses the reviewed workspace graph, Python
`openpyxl` authoring path and pinned Sora 0.3.0 exporter. A later dependency
change requires an exact pin, license and deterministic-impact review,
baseline revision and affected cross-platform goldens.

## Release scaffold

The release scaffold contains eight phases, 17 fixed batches and 104 generated
content batches: 121 atomic commits in total. Its terminal denominators are:

- 2,201 content records;
- 786 executable rules;
- 78 production-executed semantic fixtures;
- 86 mechanism-correct enemy variants;
- 173 encounter members;
- 73 named numeric-approximation candidates;
- 52 explicit external-decision records; and
- 33 deterministic World/difficulty runs.

Only `G07-P7-B3` may promote the scaffold to `Released` and register the
immutable `standard-universe-mechanics-complete-v1` snapshot.
