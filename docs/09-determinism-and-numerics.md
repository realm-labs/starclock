# Determinism and numerics

Starclock guarantees deterministic execution for the current source and data.
It does not promise that old replay bytes, hashes or serialized state remain
readable after an implementation change.

## Authoritative numbers

Authoritative combat and Activity paths use project domain types backed by
fixed-point integers. Floating-point values are not allowed in rules, state,
RNG probability, canonical encoding or hashes.

- Decimal authoring enters through canonical decimal strings.
- Arithmetic is checked; overflow, division by zero and invalid domain
  conversion return typed failures.
- Rounding is named at formula boundaries. Intermediate values are not rounded
  merely for convenience.
- Clamping or saturation occurs only when the mechanic explicitly defines it.
- Collection sizes, counters and serialized integers use fixed-width types.

Dependency pins are build inputs. They do not create a compatibility promise
for outputs produced by an older tree.

## Randomness

All randomness comes from project-owned labeled streams derived from an
explicit seed. Sampling uses stable integer algorithms and canonically ordered
candidates.

- Never use thread, system or wall-clock RNG.
- Never use floating probability draws, generic shuffle or collection
  iteration order.
- A random operation declares whether empty or rejected work consumes a draw.
- Separate graph, reward, shop, spawn and battle streams prevent unrelated
  mechanics from shifting each other's draws.
- Rejected commands restore both state and RNG exactly.

Current golden vectors test stream derivation, raw words, integer range mapping,
weighted selection and draw counts. When the current algorithm intentionally
changes, replace those vectors in the same change; do not retain the old
algorithm behind a revision switch.

## Stable ordering

Authoritative work is ordered by fixed-width domain identity and explicit
semantic priority. `HashMap` and `HashSet` may accelerate lookup, but their
iteration order never selects targets, emits events, consumes RNG or affects a
hash. Filesystem enumeration, addresses, allocation capacity, thread schedules
and platform time are never authoritative inputs.

## Commands and transactions

Battle and Activity mutation enters through accepted commands and typed
operations. A rejected command leaves authoritative bytes and RNG unchanged.
An accepted command commits its ordered events or enters the documented
deterministic fault state. Triggers enqueue bounded reactions instead of
performing recursive arbitrary mutation.

## Current canonical state

Canonical state encoding includes every fact needed to continue current
execution and excludes caches, presentation state, logs, pointers and capacity.
Streaming and collecting encoders must produce identical bytes. State hashes
use SHA-256 over those current canonical bytes.

The codec and hash layout are current implementation details. A change updates
the current tests and goldens atomically. There are no legacy hash branches,
migration tables or historical golden requirements.

## Verification

Use focused Cargo tests while editing. Numeric formulas use table-driven
boundary vectors; RNG and codecs use current golden vectors; rejected command
tests compare exact pre/post bytes. Full workspace tests run in CI, while large
property corpora and seeded matrices are explicit exhaustive checks.
