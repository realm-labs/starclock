# Goal 07 Preservation Partition S04

`G07-P2-M02-S04` completes ten content records, ten mechanic-rule
records, one production semantic fixture and eight native-handler reviews.
It covers both released levels of Burst and Concentration, Path Resonance:
Preservation and all three Preservation Resonance Formations. No native
handler is admitted.

## Authoritative authoring boundary

The formal source remains the openpyxl-authored workbook set:

- `Universe.xlsx` contains Blessing levels, Resonance definitions and exact
  parameters;
- `UniverseBindings.xlsx` binds every assigned row to its released mechanic
  source;
- `UniverseEvidence.xlsx` retains the critical-family fixture, audit rows and
  provenance.

`tools/goal07/author-path-partition.py` rejects formulas and spreadsheet error
cells, proves exact partition ownership, and compares the assigned workbook
rows with the committed Sora 0.3.0 debug and production exports. Runtime
materialization consumes the validated `.sora` bundle rather than staging
JSON.

## Exact mechanics

All ratios use six-decimal fixed point. A character is shielded when their
current effective Shield is positive.

### Construct: Burst (`612056`)

```text
shielded character CRIT DMG:
  L1 +30%
  L2 +45%
```

The modifier is dynamic. Losing the last effective Shield removes the bonus
without mutating the authored Blessing state.

### Construct: Concentration (`612057`)

```text
shielded character CRIT Rate:
  L1 +16%
  L2 +24%
```

Critical checks use the labeled CRIT RNG stream. Per-target and shared-hit
policies retain stable target order and reuse one sample for every operation
belonging to the same authored hit. `Never` consumes no CRIT draw.

### Path Resonance: Preservation (`612020`)

The action spends exactly 100 Path Resonance Energy and deals Physical damage
to every enemy:

```text
base damage = 2.50 * sum(current effective Shield of all allies)
```

The Ability Tree Resonance-damage ratio, when present, multiplies this base.
The program evaluates all ally Shields from one transactional battle-query
snapshot and emits ordinary Additional damage through the shared resolver.

### Zero-Dimensional Reinforcement (`612021`)

Preservation Resonance becomes a forced critical hit:

```text
CRIT multiplier =
    1
  + 0.50 fixed Resonance CRIT DMG
  + 0.15 * number of currently shielded allies
```

The fixed 50% Resonance CRIT DMG is authored parameter 2. It does not inherit
the first player's CRIT DMG, so Burst cannot leak from the carrier character
into the Path action. With four 16,000 Shields the frozen vectors are 160,000
without the Formation and 336,000 with it.

### Eutectic Reaction (`612022`)

After Preservation Resonance resolves, every ally receives:

```text
Shield = 0.01 * that ally's current MaxHP
duration = 2 owner turns
Amber = one shield-overflow guard for the same two-turn window
```

When one incoming damage instance exceeds the character's positive effective
Shield, Amber consumes itself, caps that instance at the Shield capacity and
therefore nullifies the overflow. Applying the Formation again replaces the
two-turn Amber effect and its one-shot guard. Ordinary, DoT, Break and Super
Break application paths share the same guard.

### Isomorphous Reaction (`612023`)

```text
battle entry: Path Resonance Energy +40
every positive ally Shield gain: Path Resonance Energy +3
```

Both updates use the checked team-resource service and clamp at the configured
100-point maximum. Shield loss and zero-delta replacement do not regenerate
Energy.

## Generic core additions

This partition adds no source-ID branch. It extends shared behavior in three
places:

- authoritative ability programs now receive the same immutable battle and
  resource query snapshot as triggered Rule IR;
- ordinary hit definitions execute deterministic `PerTarget`, `Shared` and
  `Never` critical policies from live `CritRate` and `CritDamage`;
- effect runtime definitions may carry the generic one-shot
  `ShieldOverflowOnce` damage guard.

`ExecutableResonance` now owns an ordered selector collection, allowing one
action program to target enemies while aggregating allied Shield state.
The earlier Hunt Resonance implementation moved to its own responsibility
module without changing its behavior.

## Production verification

The production integration tests prove:

- a frozen CRIT seed produces exact 97.5 damage from a 50-damage hit with
  Burst L2 and the default 50% CRIT DMG;
- exact 160,000 base and 336,000 forced-critical Resonance vectors;
- Isomorphous battle entry regenerates exactly 40 Energy;
- Eutectic emits four exact 1,000 Shields for four 100,000-HP allies and
  attaches four runtime Amber effects;
- the generic effect-template materializer preserves its damage-guard
  contract.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M02-S04.json`.
