# Gold and Gears Public API Migration

## Decision

The current source revision contracts the pre-1.0 Gold and Gears Rust facade
from 185 re-exported names to 51. This is an intentional compile-time API
migration. It does not change configuration, Activity state, battle state,
RNG, replay bytes, component identities or the immutable Goal 14 release
snapshot.

The stable mode boundary retains:

- `GoldAndGearsRuntimeFactory`, `GoldAndGearsRuntimeInstance`,
  `GoldAndGearsEntry`, `GoldAndGearsCatalogIdentity`,
  `GoldAndGearsCoverage` and `GoldAndGearsControllerIdentity`;
- adapter-facing offered-command, seeded-run and replay types/functions;
- compatibility revision constants that identify executable policies; and
- the explicit baseline-fixture, incremental-run and benchmark modules.

Rule bindings, semantic-fixture probes, cache internals, mechanic execution
plans and other mode implementation types are no longer reachable through the
public `gold_gears_entry` facade. They remain private or crate-visible where
the production runtime still needs them.

## Caller migration

External callers should construct runs through `GoldAndGearsRuntimeFactory`,
submit only offered commands through the generic Activity surface, and use the
published replay helpers or Agent/MCP adapters. Callers that imported a
removed mechanic-internal type must move that logic behind the mode boundary;
no compatibility aliases are provided for implementation-only paths.

Controller component construction now accepts one typed
`GoldAndGearsControllerIdentity` instead of three independent identity
arguments. `GoldAndGearsCoverage` is the stable public name for the existing
runtime coverage summary.

## Enforcement

`tools/repository-check/verify-source-policy.mjs` binds every reviewed public
re-export facade to an exact declaration count, symbol count and canonical
SHA-256 digest. `tools/goal14/verify-runtime-contract.mjs` verifies the exact
six Gold and Gears domain type names against the real Rust source. Any future
addition, removal or replacement requires an explicit policy and migration
update.
