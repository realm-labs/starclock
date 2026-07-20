# Eidolon Compilation

`G01-P5-B3` extends the pure build compiler with exact E0-E6 selection and a
complete six-rank definition set for every build-catalog character. These are
build-domain values only; combat receives the resulting canonical ability,
rule-bundle and modifier bindings in `ResolvedCombatantSpec`.

## Rank and ordering contract

`EidolonLevel` accepts only E0 through E6. Catalog construction requires one
Eidolon definition at every rank E1 through E6, rejects duplicate definition
IDs and wrong-form sets, and canonicalizes input rows by rank. Patches retain
their authored order within a rank.

Selecting E0 applies no Eidolon patches. Selecting En applies every rank from
E1 through En exactly once, after selected Trace patches and before effective
ability-level resolution. Workbook or constructor insertion order cannot
change compilation.

## Executable patch subset

The shared closed patch language now lowers these generic binding changes:

- add or replace an ability;
- add or remove a rule bundle;
- add a modifier;
- adjust an ability family's signed bonus and cap.

Replacement targets must exist when their patch is applied. Adds must be new,
removals must be present, and replacement output cannot already be active.
Explicit remove-then-add and replacement chains are legal because authored
sequence defines them; unspecified last-write-wins behavior is rejected.

Catalog validation checks every E1-E6 patch reference against the immutable
combat catalog and simulates the full rank sequence to reject fixed conflicts
before compilation. Compilation repeats conflict checks against the exact
selected Trace workspace, so a Trace/Eidolon collision returns no partial
combatant. Combined Trace and Eidolon level adjustments are checked against the
complete effective-level table without clamping.

The Sora transport union also reserves resource, state-slot, tag and
phase-program patches. They are not approximated here: each requires a matching
generic combat-domain output before executable lowering. Production content
remains authored through Excel/Sora and coverage remains 0/283 `DataReady`.

## Evidence

The focused fixture proves E0, prefix E3 and E6 compilation, reversed rank-row
input, E1 rule replacement, E2 ability replacement, cumulative E3/E5 level
adjustments, E4 modifier addition, E6 ability addition, incomplete/foreign rank
sets, fixed replacement conflicts and an exact Trace/Eidolon collision.

Run the focused gate with:

```text
cargo test -p starclock-build --all-targets --all-features
```
