# Canonical replay and codec implementation

Goal 01 batch `G01-P2-B4` establishes replay format version 1 and the shared
canonical byte sink. It intentionally reserves command/event/Activity record
kinds without inventing their later domain payloads.

## Shared encoder and hash path

`CanonicalEncode` writes through `Encoder<S>` where `S: CanonicalSink`. The
encoder owns raw, Boolean, little-endian integer, signed `i64`, length-prefixed
byte and UTF-8 string primitives. `Vec<u8>` is the collecting golden/debug sink;
`Sha256Sink` updates the pinned SHA-256 implementation directly. The production
`hash_canonical` path therefore never builds a complete canonical-state byte
vector.

Digest roles are separate newtypes for configuration, entry specs,
definitions, controllers, build catalogs, participant builds and state hashes.
The dependency `sha2` remains private and no dependency digest type crosses the
public boundary.

## Replay format version 1

The exact header and record envelope are recorded in document 16. A validated
header binds all replay-sensitive compatibility revisions, controller identity,
seed and either a battle or activity entry. Low-level synthetic battles contain
no build vocabulary. Build-aware activities bind ordered build digests through
an explicit optional block.

The decoder rejects unknown format/schema/entry/record values, unknown-record
policy, zero entry IDs, invalid presence bytes, noncanonical record sequences,
oversized counts/payloads, truncation and trailing bytes. It performs one full
borrowed validation pass before allocating its bounded record-reference table.
Payloads remain borrowed; command/event decoding later cannot turn this batch
into an unbounded transport allocation.

## Golden evidence

`codec_golden.rs` binds every primitive including signed-minimum and UTF-8
bytes, collecting versus streaming state hash `ee9b6541…b522`, a build-aware
activity replay hash `4822d0f3…8ef5`, low-level battle round-trip and malformed
version/policy/kind/sequence/length/truncation/trailing cases. The same tests run
on each native CI platform; alternate targets are compile-only evidence.
