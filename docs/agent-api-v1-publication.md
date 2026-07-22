# Agent API v1 publication

`starclock-agent-api::schema` publishes the exact frozen observation, action
and error JSON Schema documents plus their ordinary, trigger-heavy and stale
decision goldens as embedded UTF-8 bytes. Their path-delimited bundle identity
is SHA-256
`1746004f3f73ebbe6fb4cce4b850dd6813a1dc3a8584c3d191903328c0206725`.

The repository verifier validates the documents and goldens structurally,
rejects JSON-number authority, checks visibility and size bounds, and recomputes
the bundle digest. Rust tests independently recompute the same digest from the
published constants and match a constructed stable error to the frozen golden.

Seeded property tests run 512 cases per property and prove:

- every generated `u64` and `i64` survives JSON as an exact canonical string;
- every printable revision other than `agent-api-v1` is rejected;
- error detail output is key-ordered and independent of insertion direction;
- schema cardinality/value bounds and projection constants match policy.

Concrete battle tests additionally cover canonical combat projection, a page
of 257 events truncating at 256, private-field absence, debug gating and action
token ordering/rejection. The property runner uses a fixed ChaCha seed and no
failure-persistence side effects, so failures reproduce from the committed
test input.
