# Agent API owned value boundary

`G02-P1-B2` implements the first `agent-api-v1` Rust values. Every public
authoritative integer is an owned, checked canonical base-ten string; signed
fixed-point fields carry scaled integer values without a JSON float path.
SHA-256 identities are checked lowercase hexadecimal strings. Schema revisions,
Standard scenario IDs and opaque operational IDs reject unknown/noncanonical
input during both constructors and deserialization.

Session IDs, action tokens, event cursors and idempotency keys serialize to their
exact wire values but redact their `Debug` representation. Stable DTOs use
declaration order and `BTreeMap` context so JSON and debug output cannot depend
on randomized container order.

The owned view vocabulary covers battle status/phase, waves, teams, units,
effects and timeline actors. It carries no references into combat stores and no
hidden controller/RNG fields. Projection and collection bounds are implemented
in the next batch.

The error vocabulary freezes every `agent-api-v1` code with explicit retryable
and committed flags plus bounded message/context. Its `Display` output exposes
only the stable code, keeping request/session details out of incidental logs.
