# Goal 07 Abundance Partition S04

`G07-P2-M05-S04` completes ten content records, ten mechanic-rule
records, two semantic fixtures and eight native-handler reviews. It executes
both levels of Force Victoire and Empower, Path Resonance: Abundance, and all
three Abundance Resonance Formations. No native handler is admitted.

## Authoritative authoring boundary

The editable source remains the openpyxl-authored workbook set:

- `Universe.xlsx` owns the Blessing, level, Resonance and exact parameter rows;
- `UniverseBindings.xlsx` owns all ten mechanic bindings;
- `UniverseEvidence.xlsx` owns provenance, fixtures and native reviews.

`tools/goal07/author-path-partition.py` loads the workbooks with openpyxl,
rejects formula/error cells, and compares the assigned rows with the Sora 0.3.0
binary and debug exports. Runtime materialization reads only the validated
`.sora` bundle.

The local evidence chain preserves the released Excel-output row, bilingual
description digests and source binding for each record. Public descriptions
were cross-checked on the
[Simulated Universe Paths reference](https://honkai-star-rail.fandom.com/wiki/Simulated_Universe/Paths),
the
[Anatta reference](https://wiki.biligame.com/sr/%E5%9B%9E%E5%93%8D%E6%9E%84%E9%9F%B3%EF%BC%9A%E8%AF%B8%E6%B3%95%E6%97%A0%E6%88%91),
and a
[HoYoLAB Abundance guide](https://www.hoyolab.com/article/18412211),
accessed 2026-07-26. Community prose is corroboration; the hashed structured
rows remain the numeric authoring evidence.

## Healing reactions

Force Victoire (`612356`) observes positive healing received by a character
and installs one refreshable SPD effect for one target turn:

```text
L1: +10% SPD
L2: +15% SPD
```

Empower (`612357`) observes positive healing provided by a character ability:

```text
L1: fixed 30% chance to recover 1 Skill Point
L2: fixed 45% chance to recover 1 Skill Point
```

The trigger uses `OnceScope::Action`. Multi-target or repeated healing within
one action receives one authored chance roll, while separate healing actions
receive independent deterministic draws. Secondary mode healing cannot
recursively qualify as character-ability healing.

## Path Resonance

Path Resonance: Abundance (`612320`) consumes 100 Path Resonance Energy and,
in stable allied formation order:

1. applies `+15% MaxHP` for two target turns;
2. restores HP equal to 50% of that ally's pre-buff MaxHP.

The MaxHP effect is installed before the heal mutation, but the heal expression
is evaluated from the operation-local pre-mutation view. This avoids an
accidental 57.5% heal while preserving the authored event order.

The executable Resonance owns its manual ability plus any formation-provided
auxiliary abilities and timeline actors. These are ordinary catalog
definitions carried into the same immutable battle catalog, not callbacks or
mode-ID branches.

## Resonance Formations

Terminal Nirvana (`612321`) installs one source-matched guard on each active
ally at battle start. The first otherwise-lethal allied damage in the battle:

1. is clamped before `Downed`, leaving the target at 1 HP;
2. consumes every matching team guard instance;
3. sets current Abundance Resonance Energy to zero;
4. queues a no-cost full Abundance Resonance after the current hit.

Only same-definition, same-source team instances are consumed; unrelated
damage guards remain live.

Anicca (`612322`) extends every manual, Terminal and Anatta Resonance:

1. cleanse all negative effects from every active ally;
2. apply one `Subduing Evils` stack for one target turn.

`Subduing Evils` stacks up to five. One stack rejects one incoming Debuff,
Control or DoT before its chance roll, emits the normal resisted fact plus a
generic guarded-effect signal, and restores 10% of that character's MaxHP.
Guaranteed negative effects are also rejected.

Anatta (`612323`) observes the first energy-consuming manual Abundance
Resonance in a battle and installs one persistent recurring Resonance actor in
the action order. Each action automatically executes the same selected
Resonance program at 70% healing effectiveness, so the base heal is 35% MaxHP.
The actor persists across owner defeat/departure and wave changes.

The public and released structured descriptions specify the recurring action
and 30% healing reduction, but do not expose its action-order speed. The
current runtime uses fixed SPD 200 as an explicit numeric approximation. This
value is non-normative and must be replaced without changing the generic
countdown model when a stronger public numeric source is registered.

## Generic core seams

The partition adds two source-agnostic effect policies:

- `EffectDamageGuard::TeamDefeatOnce`;
- `EffectApplicationGuard::NegativeEffectOnce`.

Both are catalog policy attached to an ordinary effect definition. Runtime
state continues to store only stable effect identity, source, target, stacks
and duration, so replay identity remains governed by the locked catalog digest
plus canonical battle state. The negative-effect guard publishes
`NEGATIVE_EFFECT_GUARDED_SIGNAL`; content rules may react to that semantic fact
without inspecting a Blessing or Formation ID.

Executable Path Resonance definitions may also carry auxiliary abilities and
countdown definitions. Battle materialization validates and inserts them into
the normal catalog before battle creation.

## Production verification

Production tests prove:

- all ten assigned rows materialize from Excel/Sora without a native handler;
- enhanced Force Victoire is exactly a one-turn 15% percent-of-base SPD effect;
- enhanced Empower uses fixed 45% chance and `OnceScope::Action`;
- manual Resonance consumes 100 Energy, heals four allies, applies MaxHP,
  cleanses, grants `Subduing Evils`, and installs one Anatta actor;
- the negative-effect guard rejects an incoming guaranteed debuff and consumes
  one stack;
- the team-defeat guard prevents lethal damage before lifecycle settlement and
  consumes its source-matched instances;
- all relevant catalog, combat and replay operations remain deterministic.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M05-S04.json`.
