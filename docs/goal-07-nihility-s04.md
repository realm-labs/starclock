# Goal 07 Nihility Partition S04

`G07-P2-M04-S04` completes ten content records, ten mechanic-rule records,
one semantic fixture and eight native-handler reviews. It executes both levels
of Offerings of Deception and Before Sunrise, Nihility Resonance and all three
Nihility Formations. No native handler is admitted.

## Authoritative authoring boundary

The formal source remains the openpyxl-authored workbook set:

- `Universe.xlsx` owns Blessing, level, Resonance and exact parameter rows;
- `UniverseBindings.xlsx` owns the ten mechanic bindings;
- `UniverseEvidence.xlsx` owns provenance, the healing fixture and native
  reviews.

`tools/goal07/author-path-partition.py` reads those production workbooks with
openpyxl, rejects formula/error cells and compares the assigned rows with the
committed Sora 0.3.0 binary and debug exports. Runtime materialization reads
only the validated `.sora` bundle.

## Executed Blessings

### Offerings of Deception (`612256`)

Each enemy DoT damage event heals every living present ally from that ally's
own maximum HP:

```text
L1: 1% MaxHP
L2: 1.5% MaxHP
```

Every DoT event is processed independently. Multiple DoTs in one turn
therefore produce one ordered team-heal pass per DoT.

### Before Sunrise (`612257`)

Each enemy DoT damage event chooses one living present ally through a labeled
stable uniform draw and restores:

```text
L1: 2 Energy
L2: 3 Energy
```

The selection uses the shared battle RNG and appears in replay draw counters.
Energy remains bounded by the target's ordinary maximum.

## Nihility Resonance

Path Resonance costs 100 Resonance Energy and targets all living enemies. Each
target independently receives an 80% base-chance application of:

- Burn for two target turns, dealing 10% of the applier's snapshotted ATK;
- Shock for two target turns, dealing 10% of the applier's snapshotted ATK;
- Bleed for two target turns, dealing 5% target MaxHP capped at twice the
  applier's level-derived Break base damage;
- two Wind Shear stacks for two target turns, each dealing 10% of the
  applier's snapshotted ATK.

The four effects use ordinary resistible-effect resolution and shared DoT
state. Their damage, duration, stack and detonation behavior contains no
Resonance-specific resolver branch.

## Resonance Formations

### The Doubtful Fourfold Root (`612221`)

The Resonance application receives:

```text
+100% base chance
+1 target turn duration
+1 initial stack for stackable statuses
```

The complete upgraded Resonance therefore applies three Wind Shear stacks for
three target turns at 180% base chance.

### Suffering and Sunshine (`612222`)

The Resonance also applies two stacks of Confusion and Devoid for two target
turns at 100% base chance. Fourfold Root upgrades these to three stacks, three
turns and 200% base chance.

When a Confused enemy becomes Weakness Broken, every current DoT on that enemy
is detonated for:

```text
30% * current Confusion stacks
```

The trigger then consumes one Confusion stack. Confusion is capped at five
stacks.

Each Devoid stack reduces the fraction of Toughness restored on ordinary
Weakness Break recovery by 10%, to a minimum of zero. The effect is capped at
five stacks. Three stacks therefore restore 70% of every authored Toughness
layer.

### Outsider (`612223`)

Outsider grants 40 Resonance Energy at battle start. Every enemy DoT damage
event then grants another 2 Resonance Energy. Both use the checked generic
team-resource service and retain its ordinary maximum cap.

## Shared combat additions

This partition adds no content-ID branch to combat core. It extends shared
behavior with:

- a modifier-aware `ToughnessRecovery` stat whose authored base is `1.0`;
- per-layer recovery scaling with checked fixed-point arithmetic;
- effect-stack-backed dynamic modifiers for Devoid;
- no-op-safe modifier stack refresh journal entries;
- non-applicable once-scope handling when an observed event has no required
  action, hit or target context.

`ToughnessRecovery` is appended to the stat enum so existing canonical
discriminants and historical replay hashes do not shift. A missing once-scope
identity skips that trigger candidate; it does not mutate state, consume a
once key or fault the enclosing DoT command.

## Production verification

The production Kafka form and formal Goal 07 materialization prove:

- all ten assigned records and rules materialize from Excel/Sora;
- Fourfold Root produces exact three-turn, three-stack upgraded statuses;
- Kafka DoT damage emits four exact 1,500-point enhanced Offerings heals per
  DoT against the 100,000-MaxHP fixture team;
- enhanced Before Sunrise executes one stable random Energy mutation per DoT;
- Outsider grants exact 40 battle-start and 2 per-DoT Resonance Energy;
- three Confusion stacks detonate every current DoT at 90% and consume one
  stack;
- three Devoid stacks restore exactly 7 of a 10-point Toughness layer;
- all eight exceptional candidates close as `IrSufficient`.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M04-S04.json`.
