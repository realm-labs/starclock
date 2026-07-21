# Headless CLI contract

Goal 01 batch `G01-P6-B5` established the first versioned `starclock` command
surface. Phase 7 subsequently populated its production bundle, coverage and
Standard scenario paths; `G01-P8-B6` freezes the resulting release contract.

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
| released character combat forms | 88 | 88 |
| released Light Cones | 165 | 165 |
| Standard enemy variants | 17 | 17 |
| Standard encounters | 6 | 6 |
| Standard scenarios | 6 | 6 |
| Standard profile | 1 | 1 |

The optional goal selector accepts `core-combat-v1`; the optional category
filter accepts the canonical hyphenated category name.
Abilities, programs and future supporting identities may exist in a catalog but
do not inflate this frozen denominator. The production bundle contains 5,278
enabled identities and is bound to SHA-256
`abd84f70461675337092d12377db53f08b4562114fa90aa0b37ad869e9270440`.
All 283 released character/Light Cone entries are both `DataReady` and
`GoldenVerified`; the Standard/profile categories are separately complete.

## Battle and replay

The retained smoke-only `synthetic-standard-v1` scenario runs through
`baseline-battle-controller-v1`, using explicit authored ability and target
hints. The header controller digest is SHA-256 over the exact versioned hint
descriptor. Repeated runs produce identical JSON, commands, state hash and
replay bytes. `--replay-out` writes the canonical low-level replay.

`replay verify` decodes the bounded envelope, resolves its compatible synthetic
or production Standard scenario, reconstructs a fresh battle and compares every
recorded command state. Corruption reports the first command divergence and
uses the replay exit class. The six frozen `scenario.standard-v1.*` identities
resolve through the production Activity/profile/catalog path; the CLI golden
runs and verifies `scenario.standard-v1.basic-single-wave` at seed 104729.

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
