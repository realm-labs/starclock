# Goal 06 Identity and Assembly Debt Probes

This document records the mechanically verified Goal 05 starting state for
Goal 06. It is a migration baseline, not a terminal design.

The machine-readable policy is `policy/goal06-debt-probes.json`; run:

```text
node tools/goal06/verify-debt-probes.mjs
```

## Caller-supplied battle identity

`BattleSpec::new` currently accepts a `BattleSpecDigest` value. There are 32
constructor calls across 25 Rust source files, including combat tests, build
and data compilers, Activity tests, Standard mode, replay and Standard
Universe materialization.

The constructor canonicalizes participant order and validates local shape, but
the supplied digest is stored and returned without independently encoding the
battle-visible fields. P1 migrates every caller after combat-core owns the new
codec.

## Entry-time-only Standard Universe assembly

`StandardUniverseRuntimeFactory::load` currently:

1. loads the immutable core and Universe catalogs;
2. builds the default roster;
3. calls `initial_contributions` with the first Path and empty Blessing,
   Curio and Ability Tree inventories;
4. compiles one `Arc<UniverseBattleMaterialization>`;
5. reuses its encounter overlay and combat catalog for every started session.

`StandardUniverseActivity::battle_contributions` can project the current
Activity state, but the production factory does not call it. This exact seam
is the P2 migration target.

## Production surfaces

CLI and Agent construction already share `StandardUniverseRuntimeFactory`.
MCP delegates action, replay export and replay verification to the Agent
registry. Goal 06 must preserve this single authority while replacing the
factory's frozen materialization. Baseline and replay reconstruction must use
the same assembler rather than adding parallel code paths.

## Frozen transition scenarios

| Scenario | Source | Transitions | Expected identity change |
|---|---|---|---|
| Blessing acquire/upgrade | `universe.blessing.612344` | absent → L1 → L2 | combat input + assembly |
| Curio lifecycle | `universe.curio.8` | absent → active → disabled → removed | combat input + assembly |
| Resonance | `universe.resonance.612420` | locked → unlocked → resource consumed | combat input + runtime state |
| Ability Tree | `universe.ability-tree.2` | not selected → selected | combat input + assembly |
| Participant carry | Activity participant carry | full → damaged → Energy changed | combat input only |
| Outer provenance | synthetic component identity | assembly A → assembly B | assembly only |

The first Blessing enhanced level, active Curio state and Hunt Resonance are
the three Goal 05 source-keyed executable rule slices. Goal 06 uses them to
prove dynamic selection; it does not broaden their content implementation.

## Terminal inversion

At Goal 06 release:

- no public combat constructor accepts a caller-provided combat-input digest;
- every production battle uses the current Activity contribution snapshot;
- no combat catalog is rebuilt per battle;
- all five production surfaces share one assembly authority; and
- new production replays bind the v3 combat-input/assembly identity pair.

