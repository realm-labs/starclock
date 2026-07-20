# Build Catalog and Compilation Boundary

`G01-P5-B1` establishes `starclock-build` as an upstream pure compiler. It owns
an immutable `BuildCatalog`, exact `CombatantBuildSpec` input, typed validation
report and `LoadoutCompiler`. The only successful combat-facing value is the
existing `starclock_combat::ResolvedCombatantSpec`; no build type is added to
`starclock-combat` or authoritative battle state.

## B1 catalog shape

The first catalog row is deliberately a fixed already-resolved character
boundary: form, supported level, maximum HP, Speed, canonical combat-definition
bindings and a declared nonzero combatant-spec digest. The builder sorts rows by
`UnitDefinitionId`, rejects duplicate forms, and verifies every ability, rule
bundle and modifier against both the immutable `CombatCatalog` and the selected
unit definition. It captures the exact compatible combat revision and digest.

This fixed row is synthetic foundation evidence, not production character
coverage. B2 extends it with exact stat rows, ability curves and Trace patches; B3 adds
Eidolons; B4 adds Light Cones; B5 adds canonical definition/catalog/build/spec
digest encoding, source attribution, named presets and build locks; B6 protects
the relic boundary.
Production Excel/Sora lowering is not bypassed, and coverage remains 0/283
`DataReady`.

## Validation and compilation

Compilation is pure, allocates no battle/runtime ID, consumes no RNG and never
mutates either catalog. Its typed report records these stages in fixed order:

1. exact combat revision and digest compatibility;
2. character-form lookup;
3. exact supported level;
4. current combat-definition reference integrity;
5. construction of the generic resolved combatant.

A failure appends one `Failed` entry at its owning stage and returns no partial
combatant. Success returns five `Passed` entries with the generic value. This
report deliberately contains stable IDs/enums rather than localized names,
spreadsheet coordinates or arbitrary diagnostic strings.

The focused gate is:

```text
cargo test -p starclock-build --all-features
```

It proves canonical catalog ordering, duplicate/cross-catalog rejection, exact
compatibility revalidation, stable failure reports and a successful compilation
whose output exposes only combat-domain fields.
