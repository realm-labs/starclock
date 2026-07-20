# Headless CLI contract

Goal 01 batch `G01-P6-B5` completes the first versioned `starclock` command
surface without claiming production content readiness.

## Production bundle and coverage

`config validate` uses the same generated Sora reader and complete immutable
catalog conversion as runtime loading. With no path it validates the production
bundle embedded at build time; `--bundle PATH` validates those exact bytes.
Diagnostic JSON is rejected as a bundle. Results include the exact bundle
SHA-256, version/revision and identity/enabled counts.

`catalog coverage` projects only the frozen Goal 01 denominator categories from
validated Sora identities:

| Category | Current required | Current DataReady |
|---|---:|---:|
| released character combat forms | 88 | 0 |
| released Light Cones | 165 | 0 |
| Standard enemy variants | 17 | 0 |
| Standard encounters | 6 | 0 |
| Standard scenarios | 6 | 0 |
| Standard profile | 1 | 0 |

The optional goal selector accepts `core-combat-v1`; the optional category
filter accepts the canonical hyphenated category name.
Abilities, programs and future supporting identities may exist in a catalog but
do not inflate this frozen denominator. Bootstrap identities remain disabled;
the CLI reports 0/283 DataReady rather than treating metadata presence as
implementation.

## Battle and replay

The existing smoke-only `synthetic-standard-v1` scenario now runs through
`baseline-battle-controller-v1`, using explicit authored ability and target
hints. The header controller digest is SHA-256 over the exact versioned hint
descriptor. Repeated runs produce identical JSON, commands, state hash and
replay bytes. `--replay-out` writes the canonical low-level replay.

`replay verify` decodes the bounded envelope, resolves the compatible synthetic
scenario from the header, reconstructs a fresh battle and compares every
recorded command state. Corruption reports the first command divergence and
uses the replay exit class. Activity replay payload verification is already a
library contract; production Activity/profile resolution follows the B6
Standard manifest import.

Human and JSON modes are projections of the same result. JSON stdout contains
one `starclock-cli-v1` object; errors go to stderr. Exit codes are stable:

| Code | Class |
|---:|---|
| 0 | success |
| 2 | usage |
| 3 | configuration or bundle |
| 4 | replay incompatibility or divergence |
| 5 | invalid scenario |
| 6 | simulation fault |
| 7 | I/O or internal tool failure |
