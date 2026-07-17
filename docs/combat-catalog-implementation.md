# Combat catalog implementation

Goal 01 batch `G01-P2-B2` establishes the immutable combat-domain catalog and
identity boundary described by the core implementation design. The catalog is
engine-agnostic, owns no generated Sora rows and is shared as an
`Arc<CombatCatalog>` after construction.

## Identity domains

Definition IDs are non-zero `u32` newtypes. Unit, ability, effect, rule,
program, selector, rule-bundle, modifier, enemy and encounter IDs are mutually
distinct Rust types. Native-handler, rule-state-slot, trigger, hit-plan and
generic source-attribution definitions use the same fixed-width policy without
becoming interchangeable with the catalog families.

Battle-local IDs are non-zero `u64` newtypes for units, timeline actors,
effects, shields, rule and modifier instances, actions, phases, hits,
operations, events, waves and spawn sequences. This batch defines their type
boundary; monotonic allocation, tombstones and canonical runtime stores remain
owned by the battle-state batches that first create instances.

## Immutable storage

`CombatCatalogBuilder` is the only public construction path. It accepts
Starclock-owned domain definitions and produces an immutable shared catalog.
Each definition family is sorted once by its typed ID and stored in a private
boxed table. Lookup is a binary search and iteration is explicit canonical ID
order; insertion order and a hash-map iteration order cannot affect the result.

Reference fields distinguish two ordering policies:

- set-like references must already be strictly ID-ordered and unique;
- authored execution order is preserved for program calls, rule-bundle rules
  and encounter enemy entries.

Later Rule IR and action-program batches extend the payloads of these domain
definitions. They must continue to use this builder and cannot add closures,
generated transport types, mutable cells or content-ID branches to the catalog.

## Construction validation

Construction rejects an empty/malformed revision, a zero configuration digest,
duplicate IDs, noncanonical set references and every missing typed reference.
After reference validation, a stable ID-ordered depth-first traversal rejects
static program cycles and returns the canonical closed cycle path.

`catalog_contract.rs` builds the same complete graph in opposing insertion
orders and proves identical indexes while preserving authored encounter order.
It also covers duplicate, unresolved, noncanonical, cyclic and invalid-identity
failures through stable typed error categories.
