# Rust Engineering Standards

This document is normative for handwritten Rust code in this repository. Its purpose is to keep the combat engine reviewable, deterministic, and independent of any presentation engine as the character-mechanics surface grows.

The keywords **must**, **must not**, **should**, and **may** express requirement strength.

## File-size rule

- A handwritten `.rs` file **must not exceed 1,200 physical lines** unless splitting it would materially reduce clarity or violate generated/foreign-code constraints.
- Begin evaluating a split around **800 lines**. Do not wait until line 1,200 to design module boundaries.
- Blank lines, comments, inline tests, and conditional code count toward the limit because they still affect navigation and review cost.
- `lib.rs` and `mod.rs` should normally remain below **200 lines** and contain module declarations, carefully selected public API declarations, and little implementation.
- Large test modules should move to `module/tests.rs` or crate-level `tests/` before they force production files over the limit.

Permitted exceptions are:

- Sora-generated Rust under an unmistakable `generated/` directory;
- vendored third-party source that is not maintained as project code;
- a mechanically generated table or match implementation when splitting would make generation or review less reliable;
- a rare cohesive algorithm whose state and invariants become less understandable after a split.

Every handwritten exception must include a short module-level explanation and be noted in review. “It was faster to keep adding code” is not an exception.

## Responsibility-based modules

Each module should have one primary reason to change. Split by behavior and ownership, not by arbitrary line ranges.

Good boundaries include:

```text
damage/
  mod.rs            public module API and orchestration
  context.rs        immutable calculation inputs
  formula.rs        pure formula stages
  modifier.rs       modifier filtering and aggregation
  result.rs         calculation output
  tests.rs          focused module tests
```

Avoid files such as `utils.rs`, `common.rs`, `helpers.rs`, `manager.rs`, or `misc.rs` that accumulate unrelated behavior. A shared abstraction should be named after the domain concept it owns.

Use these separation rules:

- data types describe state and invariants;
- pure calculators transform explicit inputs into results;
- resolvers mutate battle state through commands/operations;
- event collection and reaction scheduling remain separate from mutation logic;
- Sora record conversion belongs to `starclock-data`, not `starclock-combat` or `starclock-activity`;
- deterministic Trace/Eidolon/equipment compilation belongs to `starclock-build`; `starclock-combat` receives only generic resolved combatant specs, while account ownership/inventory remains outside these crates;
- Bevy entity/component mapping belongs to `starclock-bevy`, not combat crates;
- character-specific native behavior belongs behind the same operation/event interfaces as table-authored behavior.

Do not create one file per tiny type when several types form one stable concept. The goal is cohesive modules, not maximum file count.

## Visibility and re-exports

Default to the narrowest visibility:

```text
private -> pub(super) -> pub(crate) -> pub
```

Only items needed by external crate consumers should be `pub`. Cross-module use inside one crate normally requires at most `pub(crate)`.

### `pub use` policy

Do **not** use `pub use` by default.

Allowed cases are limited to:

- a small, deliberate crate-root facade for the stable external API;
- re-exporting a type whose defining module is intentionally private implementation detail;
- compatibility during a documented public-API migration, with a removal plan;
- a macro or generated-code integration that technically requires a re-export.

Rules for allowed re-exports:

- list every item explicitly;
- document why the re-export is part of the public API;
- keep one canonical public path for a type;
- avoid chains where module A re-exports B, then crate root re-exports A;
- never use wildcard public re-exports such as `pub use module::*`;
- do not create a project-wide `prelude` module.

Internal code should import from the defining module, for example `crate::timeline::ActionGauge`, rather than relying on a crate-root re-export. If a `pub use` merely saves consumers a few characters and does not define a meaningful facade, omit it.

## Dependency direction

The dependency rule remains:

```text
Sora-generated records -> starclock-data -> starclock-combat / starclock-build / starclock-activity
starclock-build --------------------------> starclock-combat
starclock-rules --------------------------> starclock-combat / starclock-activity
starclock-activity --------------------------> starclock-combat
starclock-mode-standard --------------------------> starclock-activity / starclock-build / starclock-combat
starclock-mode-challenge -------------------------> starclock-activity / starclock-build / starclock-combat
starclock-mode-universe --------------------------> starclock-activity / starclock-build / starclock-combat
starclock-mode-event -----------------------------> starclock-activity / starclock-build / starclock-combat
starclock-ai / starclock-replay --------------> starclock-combat / starclock-activity
starclock-cli / starclock-bevy -----------------> starclock-combat / starclock-build / starclock-activity / mode crates
```

- `starclock-combat` must not depend on Bevy, Sora CLI crates, spreadsheet readers, filesystem layout, rendering, or platform time.
- `starclock-data` may depend on the Sora-generated runtime reader but converts generated records into the separate domain definitions owned by combat, build, activity, and mode crates.
- `starclock-build` depends on public combat-domain definitions and produces `ResolvedCombatantSpec`; `starclock-combat` never depends on `starclock-build` and never queries a BuildCatalog, inventory, or progression graph.
- `starclock-activity` constructs immutable battle specifications and consumes verified battle results, but it never mutates a live battle, interprets build fields, or owns mode-specific content types. It stores/locks resolved participant specs and digests as opaque domain values.
- Mode crates compose generic activity graphs and combat rules. They must not fork command processing, graph execution, formula, effect, timeline, RNG, replay, or hash implementations.
- `run-core` and `challenge-core` are not target crates. Generic cross-battle behavior belongs to `starclock-activity`; universe/challenge names remain mode-domain concepts.
- `starclock-rules` is a static native-handler registry and cannot depend on presentation, CLI, or mode orchestration.
- Adapter crates may depend on the core; the core must not depend on adapters.
- Avoid cyclic crate or module dependencies. Move the shared domain concept toward the lower-level owner instead of introducing callback glue to hide a cycle.
- New dependencies require a concrete use, license review, and consideration of deterministic behavior and compile-time cost.

## Domain modeling

- Prefer domain newtypes such as `UnitId`, `Ratio`, `RawToughness`, and `ActionGauge` over interchangeable primitives.
- Represent states that must not coexist with enums instead of boolean combinations.
- Make illegal states difficult to construct; validate external data at the boundary.
- Use stable IDs in battle state and events rather than references tied to collection addresses.
- Keep authored definitions immutable during a battle. Mutable combat state must not contain hidden pointers into Excel/Sora records.
- Percentage values use ratios (`0.25`), never a mixture of ratios and whole percents.
- Units and rounding policy must be visible in type names, constructors, or field documentation.
- Authoritative combat code uses the pinned fixed-point representation and rules in [Cross-platform determinism and numeric policy](09-determinism-and-numerics.md); raw `f32`/`f64` values are limited to non-authoritative test/reference tools.
- Arithmetic that can overflow or discard precision uses checked named methods with explicit rounding, not bare operators.

Avoid untyped maps for core concepts when the legal keys and formula stages are known. Extensibility should come from typed operation/trigger registries, not `HashMap<String, Value>` throughout the resolver.

## Functions and control flow

- A function should operate at one abstraction level and have a descriptive domain name.
- Prefer early returns for invalid/inapplicable cases over deeply nested conditionals.
- Pass explicit context objects when a calculation needs several related inputs; do not reach into global or thread-local state.
- Keep pure formula code free of state mutation and RNG.
- All randomness goes through the injected battle RNG and records its purpose.
- Authoritative sampling is integer-based; floating probability draws are forbidden.
- Do not hide meaningful state mutation in getters, conversions, `Deref`, or `Drop`.
- Avoid boolean parameters when an enum communicates the policy.
- Extract repeated behavior only when the abstraction has a stable domain meaning; three similar lines are not automatically duplication.

## Error and panic policy

- Invalid commands, targets, configuration, and replay data return typed errors.
- Production code must not use `unwrap()` or `expect()` for recoverable external or battle-state conditions.
- `expect()` is acceptable only for a proven internal invariant and must explain that invariant in its message.
- Indexing that can depend on authored data, target death, or user input must be checked.
- Panics indicate programmer defects, not combat outcomes.
- Tests may use `unwrap()` when failure should abort the test and the surrounding assertion context is clear.

Unsafe Rust is forbidden unless a measured requirement cannot be satisfied safely. An unsafe exception requires a dedicated safe wrapper, documented safety invariants, focused tests, and explicit review.

## Events and mutation

- Battle state changes only through the command/operation resolver.
- Activity state changes only through `ActivityCommand` and typed activity operations; mode profiles/handlers cannot mutate it directly.
- Every meaningful mutation emits or is represented by a domain event.
- Triggers enqueue reactions; they do not recursively mutate arbitrary state.
- Event ordering uses explicit priority and stable sequence IDs, never collection iteration order.
- A trigger states whether it is limited per hit, target, ability, action, turn, wave, or battle.
- Native character handlers may create normal operations/events but may not bypass validation, resource payment, target legality, reaction budgets, or replay tracing.
- Native activity handlers may create normal activity operations/options but may not bypass graph, scope, participant, clock/metric, BattleResult, budget, or replay validation.

## Documentation and comments

- Public APIs require rustdoc that explains invariants and timing semantics, not only field names.
- Formula code should cite the corresponding project document and identify the rules revision it implements.
- Comments explain why a rule or ordering exists. Do not narrate syntax.
- Every workaround has an owner condition for removal, preferably an issue reference.
- Keep examples compilable as doctests where practical.

## Testing standards

- Pure formulas require table-driven unit tests around boundaries and known vectors.
- State machines require transition tests, including invalid transitions.
- Bugs receive a regression test before or with the fix.
- Cross-module public behavior belongs in integration tests.
- Seeded golden tests include the config bundle digest and event/state hash.
- Cross-platform golden fixtures compare the canonical state hash after every command, not only final battle state.
- Generated Sora readers require at least one real bundle-load test; handwritten conversion and validation code requires ordinary unit/integration coverage.
- Tests must not depend on wall-clock time, filesystem enumeration order, thread scheduling, or unseeded randomness.

Test code follows the same responsibility rules. Generated fixtures may be large, but large handwritten fixtures should move to data files or builders.

## Formatting and lint gates

The minimum local and CI gate is:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
```

Once the workspace is created, configure shared lints at the workspace root. At minimum:

- forbid unsafe code unless the project records an approved exception;
- deny unused must-use results and unexpected conditional-compilation values;
- treat Clippy warnings as CI failures;
- do not silence a lint globally when a narrow, explained local allowance is sufficient.

A CI file-size check must fail when a handwritten `.rs` file exceeds 1,200 physical lines. It must exclude only explicit generated/vendor paths, not arbitrary filename patterns.

## Review checklist

Before merging a Rust change, verify:

- no handwritten `.rs` file exceeds 1,200 lines and files approaching 800 lines have a credible scope;
- each changed module has one clear responsibility;
- new public visibility is required by an actual consumer;
- every `pub use` satisfies the limited facade/migration/integration policy;
- no wildcard public re-export or project prelude was introduced;
- configuration logic remains in Excel/Sora and `starclock-data`, not in presentation adapters;
- deterministic ordering, RNG, rounding, and replay effects are covered;
- new failure paths return typed errors rather than panic;
- formatting, Clippy, tests, bundle validation, and golden replays pass as applicable.
