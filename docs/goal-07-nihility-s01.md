# Goal 07 Nihility Partition S01

`G07-P2-M04-S01` completes seventeen content records, sixteen mechanic-rule
records, one production fixture and ten native-handler reviews. It executes
the first six released Nihility Blessings and establishes the shared
**Suspicion** effect. No native handler is admitted.

## Authoritative authoring boundary

The formal source remains the openpyxl-authored workbook set:

- `Universe.xlsx` owns Nihility, six Blessings and their exact level
  parameters;
- `UniverseBindings.xlsx` owns the sixteen mechanic bindings;
- `UniverseEvidence.xlsx` owns provenance, the Nihility fixture and all native
  reviews.

`tools/goal07/author-path-partition.py` rejects formulas/error cells and
compares every assigned workbook row with the committed Sora 0.3.0 production
and debug exports. Runtime materialization reads only the validated `.sora`
bundle.

## Shared Suspicion effect

Suspicion is a non-dispellable enemy debuff with a 99-stack limit:

```text
DoT vulnerability per stack: +1%
ordinary enemy-turn-end decay: -2 stacks
enhanced Funeral: no decay
zero stacks: remove the effect
```

Its DoT vulnerability and Call of the Wilderness modifiers read the same
effect-owned stack slot. Application, refresh, signed stack adjustment,
modifier refresh and removal remain generic combat operations.

## Executed Blessings

### Funeral of Sensory Pursuivant (`612230`)

Each applied DoT damage event adds one Suspicion stack to that enemy. The
enhanced level makes Suspicion persistent by removing its ordinary
two-stack enemy-turn-end decay.

### The Man in the Cover (`612231`)

Applying a new DoT adds three Suspicion stacks. At the enhanced level,
refreshing or positively increasing an existing DoT adds one stack.

### Why Hasn't Everything Already Disappeared? (`612232`)

At each enemy turn start, all DoTs currently on that enemy are detonated at:

```text
L1: 90% of current DoT damage
L2: 135% of current DoT damage
```

The turn-event selector is anchored to the enemy event owner, not an action
primary target.

### Beginning and End (`612240`)

When an enemy with Suspicion is defeated, the exact current stack count is
copied to one random other living enemy at L1 or up to two at L2. Candidates
use stable-ID ordering and the labeled `behavior-choice` RNG stream.

### Café Self-Deceit (`612241`)

At L1, every positive Suspicion application gains one additional stack. At
L2, the newly applied amount is doubled. The rule excludes its own source from
the observed stack events, preventing recursive self-amplification.

### Call of the Wilderness (`612242`)

Each Suspicion stack applies:

```text
L1: -0.3% enemy ATK per stack, capped at -30%
L2: -0.4% enemy ATK and -0.4% Effect RES per stack,
    each capped at -30%
```

The released source decimals `0.0029999998` and `0.0039999997` are
deterministically rounded at Starclock's six-decimal numeric boundary to
`0.003000` and `0.004000`.

## Generic core additions

This partition adds no Nihility or content-ID branch to combat core. Shared
Rule IR now supports:

- expression-backed initial effect stacks;
- signed effect-stack adjustment with exact refresh/removal events;
- aggregate effect-stack queries from an immutable evaluation snapshot;
- typed `StackDelta` event reads;
- excluded-source event filters;
- immediate refresh of stack-backed modifier attachments;
- owner-anchored enemy turn selectors.

Rule value/source accessors were moved into the existing private support
module to retain the 1,200-line source policy without changing public domain
paths or adding a public re-export.

## Production verification

The Kafka production form executes its released Ultimate through the normal
legal-command boundary. The fixture proves:

- all seventeen assigned records and sixteen rules materialize from formal
  Excel/Sora data;
- Kafka Shock creates Suspicion through a real DoT application event;
- ordinary Man in the Cover plus Café creates exactly four stacks, then the
  enemy turn removes exactly two;
- enhanced Café doubles every positive application;
- enhanced Funeral prevents decay;
- all ten exceptional candidates close as `IrSufficient`.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M04-S01.json`.
