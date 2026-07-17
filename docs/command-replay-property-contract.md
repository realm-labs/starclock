# Phase 3 Command and Replay Property Contract

`G01-P3-B8` extends the fixed-seed property harness across the complete Phase 3
transaction and replay path. These are bounded structural properties over
synthetic definitions, not production-content coverage or a replacement for
the Phase 8 long-sequence/fuzz gate.

## Command sequences

Seed `0x636f6d6d616e6431` runs 256 generated sequences of 1 through 128 steps.
Each step either submits the currently offered deterministic command or forges
a future decision ID. Two independently built battles execute the same stream.

For every accepted command the property compares the complete `Resolution`,
canonical collected state bytes and streaming state hash. For every rejected
command it proves equality of pre/post canonical bytes, hash, RNG draw count
and decision, then proves the independent battle remains byte-identical.
Supported valid steps deliberately exclude Concede so arbitrarily long
generated prefixes remain at a decision boundary.

## Rollback convergence

Seed `0x726f6c6c6261636b` generates every prefix length from 0 through 64. Two
battles first apply the same valid prefix. The next offered command receives a
test-only `Rollback` fault after the resolving-phase mutation in one battle and
after complete command mutation in the other.

Both depths must return the same faulted `Resolution`, complete canonical bytes
and streaming/collecting hash. Fault injection remains private under tests and
does not add a command, runtime switch or production failure path.

## Replay corruption

Seed `0x626174746c652d31` runs 256 mutations over one exact 64-command replay.
The unmodified stream first verifies all command hashes. Generated cases then
cover:

- truncation and trailing bytes;
- configuration identity mutation;
- arbitrary accepted-command payload bytes;
- arbitrary expected-state hash bytes;
- unsupported battle-command payload version;
- unknown record kind; and
- noncanonical record sequence.

Every corrupted replay must fail construction or one-shot verification. Domain
mutations therefore cannot become an accepted alternative command stream, and
the first mismatch continues to use the stable replay error boundary.

All three families retain the harness defaults: ChaCha generation, 256 cases,
4,096 shrink iterations and source-parallel regression persistence. Any
minimized regression file is committed beside its owning crate.
