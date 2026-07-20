# Ability-level parameter runtime boundary

Production ability parameters are keyed by the base ability identity, effective level, and a stable semantic key such as `parameter.01`. Catalog compilation resolves every effective level to the same ability definition selected by the build compiler: level 1 retains the family ID and later levels use the checked variant-ID mapping.

Each resolved ability owns an immutable parameter map in `CombatCatalog`. Rule IR reads that map through `ValueExpr::AbilityParameter`; the current ability comes only from the triggering `RuleOccurrence`, so evaluation cannot accidentally fall back to another level or family. A missing occurrence, reader, ability, or key returns `MissingValue` and faults closed.

This boundary deliberately does not interpret positional parameters. Content authoring must bind a stable parameter key to the typed formula that consumes it. Minimum- and maximum-level production catalog tests prove that the same key resolves to distinct exact scalar values.
