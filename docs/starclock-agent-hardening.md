# Starclock agent hardening corpus

The Goal 02 hardening corpus is retained at
`evidence/agent-control-mcp-v1/security/hardening-corpus.json`. It has a fixed
identity seed and 60 explicit regression cases covering malformed action JSON,
opaque token boundaries, idempotency conflicts, cursor representations,
truncated/trailing/bit-flipped replay bytes, all six production Standard
settlement paths and 16 two-contender mutation races.

`crates/starclock-agent-api/tests/hardening_corpus.rs` executes the artifact
through public protocol-neutral APIs. Decode failures are total. Invalid
tokens and cursors disclose no state. Conflicting idempotency reuse, corrupt
replay verification and the losing side of each race preserve the live state
hash, replay length and RNG draw count. Exactly one racing mutation commits in
every round. All 62 accepted external actions across the six Standard
scenarios remain below the frozen 4,096-command, 65,536-event and
262,144-resolver-operation per-settlement limits.

The retained corpus complements, rather than replaces, fixed-seed property
suites: 512 schema/value cases, 256-case replay codec/framing/arbitrary-byte
properties under three seeds, and 256 battle-replay corruption cases. Replay
properties retain source-parallel regression files when shrinking finds a new
counterexample. The seeds, counts, scenario denominator and settlement budgets
are checked against policy so a passing run cannot silently narrow them.

Run the batch proof with:

```text
cargo test -p starclock-agent-api --test hardening_corpus --all-features
cargo test -p starclock-agent-api --test schema_property_contract --all-features
cargo test -p starclock-replay --test property_contract --test battle_property_contract --all-features
node tools/agent-control/verify-agent-hardening-corpus.mjs
```
