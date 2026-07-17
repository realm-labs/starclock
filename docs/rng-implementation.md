# Deterministic RNG implementation

Goal 01 batch `G01-P2-B3` implements compatibility revision
`chacha8-rand-0.10.2-intmap-v1`. The revision binds the pinned generator, stream
derivation bytes, raw-word interpretation, project-owned integer mappings and
draw-consumption rules.

## Dependency boundary

`rand = 0.10.2` is built without default features and with only `std` and
`chacha`; thread/system RNG and the optional `unbiased` mapping are not enabled,
and generic distribution helpers are mechanically unused. Only the private
wrapper names `ChaCha8Rng`. `sha2 = 0.11.0` has no default features and is
confined to canonical stream derivation. Public APIs expose only Starclock-owned
types.

A live `DeterministicRng` does not implement `Clone`. Its canonical identity is
the original 32-byte seed plus the number of consumed raw `u64` words. Every
raw word has a zero-based monotonic index and a stable non-zero purpose code.
Counter exhaustion is detected before advancing the generator.

## Canonical stream derivation

The SHA-256 input is the following concatenation. Integers are big-endian and
each text field is prefixed by its big-endian `u16` byte length:

```text
"starclock-rng-stream-v1\0"
rng_algorithm_revision
master_seed: u64
activity_profile_id
activity_instance_id: u64
section: u32
node: u32
attempt: u32
battle_sequence: u32
stream_label
```

Text identities are non-empty printable ASCII with at most 128 bytes. The full
32-byte digest seeds ChaCha8 directly. Profile/instance/location/attempt/battle
coordinates and labels such as `graph`, `spawn` or `battle` therefore isolate
future streams; a live stream is never cloned to create a substream.

## Integer mappings and draw policy

For an exclusive upper bound `n`, unsigned rejection sampling computes
`threshold = (-n mod 2^64) mod n`. Raw words below the threshold are rejected;
the first accepted word maps through `raw mod n`. The counter includes rejected
words and the result records the accepted raw sample plus the rejection count.

Weighted selection first checks a cumulative `u64` total, samples below that
total, then scans candidate weights in authored order. Zero weights remain in
their authored positions but own no interval. Candidate ordering is the
caller's stable domain order and must be included with weights in later event
diagnostics.

The following requests consume no draw:

- zero candidates;
- an empty or all-zero weighted candidate list;
- a weight-total overflow or candidate-count validation failure;
- a direct zero-upper-bound range error.

Changing any mapping, ordering requirement, raw-word width, purpose coding or
draw-consumption rule requires a new RNG algorithm revision and new goldens.
`rng_golden.rs` binds an independently checked SHA-256 seed, eight ChaCha8 raw
words, five range mappings, one weighted result and stream isolation vectors.
