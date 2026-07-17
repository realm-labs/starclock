# Property-test harness

Goal 01 batch `G01-P2-B6` establishes the reusable property-testing baseline
that later command, resolver and hardening batches extend. It uses exact
`proptest = 1.11.0` only as a dev-dependency of `starclock-combat` and
`starclock-replay`; production targets and authoritative dependency edges do
not include the harness.

## Reproduction contract

Each property family declares a distinct fixed 64-bit Proptest seed, uses the
ChaCha generator, executes 256 successful cases and permits 4,096 shrink
iterations. `FileFailurePersistence::SourceParallel("proptest-regressions")`
stores a minimized failure beside the owning crate. Generated regression files
are source evidence and must be committed rather than ignored when a failure is
fixed. The fixed seed remains the baseline even when a regression corpus grows.

The current seeds are:

| Family | Seed |
|---|---:|
| Fixed-point arithmetic | `0x6e756d6572696321` |
| RNG range/weight mapping | `0x72616e67652d6d61` |
| Catalog ordering | `0x636174616c6f6721` |
| Canonical primitive codec | `0x636f6465632d7631` |
| Replay round trip | `0x7265706c61792d31` |
| Malformed replay framing | `0x6d616c666f726d31` |
| Arbitrary replay bytes | `0x38cb39cc3ad8389b` |
| Mixed battle command sequences | `0x636f6d6d616e6431` |
| Rollback after valid prefixes | `0x726f6c6c6261636b` |
| Battle replay corruption | `0x626174746c652d31` |

An explicit stress configuration may increase the case count only after
recording the command and seed. A passing random-only run does not replace these
fixed-seed cases or a committed minimized regression.

## Baseline properties

The combat suite proves:

- checked addition/inversion, multiplication commutativity, identity, rounding
  ordering and signed Floor/Ceil duals across bounded fixed-point inputs;
- deterministic range and weighted selection, bounds, positive-weight result,
  raw-draw accounting, and zero-draw empty/overflow rejection;
- identical canonical unit indexes for generated unique definition sets inserted
  in opposing arbitrary orders.

The replay suite proves:

- all canonical primitive families round-trip and collecting/streaming encoders
  feed identical SHA-256 bytes;
- generated closed record kinds and payloads re-encode byte-for-byte after decode;
- truncation, trailing bytes, unknown kinds, invalid sequences and oversized
  lengths are rejected;
- arbitrary byte strings do not escape the total replay decoder through a panic.

`G01-P3-B8` now adds generated mixed valid/forged command streams, byte-exact
rollback convergence after every bounded prefix, deterministic
resolution/hash traces and battle replay envelope/domain corruption. Their
exact contract is recorded in
[Phase 3 command and replay properties](command-replay-property-contract.md).

These remain bounded structural properties, not a content or full-battle
claim. `G01-P8-B3` expands the same harness into long sequences and
coverage-guided corruption/resolver/content cases while preserving every seed
and minimized corpus.
