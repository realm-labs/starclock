# Canonical session replay

An agent session exports the unchanged Goal 01 canonical battle replay: one
frozen compatibility header followed by alternating accepted-command and
resulting-state-hash records. The header binds the Standard-v1 rules, catalog,
configuration, numeric policy, RNG, state-hash and agent-controller revisions,
the exact master seed, encounter identity and battle-spec digest.

Operational session IDs, event cursors, idempotency keys and public
observations are deliberately absent. Two sessions with the same scenario,
seed and accepted commands therefore produce byte-identical replay files.

Controller attribution is returned as a nonauthoritative diagnostic sidecar.
Each entry identifies an accepted boundary as `external_player`,
`authored_enemy` or `system_automatic` and records its resulting hash. Changing
or discarding this sidecar cannot change the canonical bytes or their SHA-256
digest; verification trusts only the canonical envelope.

Verification is one-shot and isolated. It reconstructs a fresh production
Standard-v1 battle from the session's frozen scenario and seed, then delegates
record decoding, identity checks, command application and every resulting hash
comparison to `starclock-replay`. It never reuses or mutates the live battle.
Malformed, incompatible, truncated or divergent input returns
`replay_diverged` at the agent boundary.

Tests cover a multi-boundary round trip, exact command count/final hash/phase,
corruption rejection with an unchanged live session, inert diagnostic changes
and byte identity across distinct operational session IDs.
