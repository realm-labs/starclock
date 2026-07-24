# Goal 06 Native Matrix and Corruption Hardening

This document is normative for `G06-P4-B2`.

## Local native result

The Windows x64 native hardening runner completed in 45.8 seconds. It executed:

- six dynamic assembly/replay integration tests;
- four Agent replay, parity and concurrent-session tests;
- MCP full-activity parity plus authorized HTTP conformance;
- replay-v3 component and nested identity tests;
- the complete 33-entry Standard Universe release matrix.

The matrix covers 9 worlds and 33 world/difficulty pairs. It completed 166
nested battles, 956 accepted battle commands and 1,954 replay actions. Every
run exports replay v3 and verifies from a fresh runtime factory. The combined
encoded replay size is 1,398,322 bytes.

The canonical matrix stdout SHA-256 is
`69cac2b14abe597717922256c442d5fb1e864b80a5b6a17eb5600a7d1760dff9`.
The ordered final-state digest is
`3825f79b56860791d22c70bbe94f5812742f85715ee020798020d43d8198428f`;
the ordered replay digest is
`67fb1e6ac1c752f2129dac0b71e363f6563a00abcdd45edf422050f33b5c1c99`.

## Corruption and concurrency corpus

The native runner binds these minimum denominators:

- all eight replay-v3 first-divergence boundaries;
- 16 malformed Agent replay cases with live-session inertness;
- 16 equal concurrent sessions sharing immutable factory data;
- CLI baseline, direct Agent and MCP transport parity;
- 33 independently verified matrix replays.

Assembly cache contents and hit/eviction history remain non-authoritative.
Changing cache capacity or scheduling cannot change a matrix state or replay
digest.

## Hosted native contract

The CI matrix executes the same `run-native-hardening.mjs --run` command on:

- Windows x64;
- Linux x64;
- macOS ARM64.

Windows ARM64, Linux ARM64 and macOS x64 remain compile-only profiles and make
no runtime equality claim. Successful hosted jobs retain their machine-readable
profile evidence for 30 days. This repository records the required hosted
contract and local native result; it does not claim a hosted profile ran before
the corresponding CI job succeeds.

Goal 05 evidence remains immutable. Its verifier checks that the current
workflow still executes the Goal 05 gate rather than requiring the entire
workflow file to remain byte-frozen, allowing later goals to add independent
native gates safely.
