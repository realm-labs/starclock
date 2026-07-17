# Rule IR schema golden fixture

This disabled synthetic fixture extends the `G01-P1-B7` character/build fixture
inside `tools/config-schema/verify-rule-ir.mjs`. It proves the Sora 0.3.0
transport shape for typed selectors, expressions, conditions, state slots,
triggers, programs, operations, effects, modifiers and native-handler metadata.

`data-overlay` is intentionally not a production data root. The verifier copies
the prior disabled fixture, appends the identity/evidence fragments, and overlays
the B8 rows in an isolated cache directory. Production authoring remains
`.xlsx`-only in `G01-P1-B10`; domain compilation and whole-graph validation
remain in `G01-P1-B11`.

Run `node tools/config-schema/verify-rule-ir.mjs`. Use `--bless` only for an
intentional reviewed schema/generator revision.
