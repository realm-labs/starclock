# Offered-action binding

`starclock-agent-api` exposes an ordered `OfferedAction` description for each
external command already offered by the authoritative combat decision. It does
not accept an ability, actor, target, cost or damage payload from an agent and
does not construct combat commands from request data.

`OfferedActionSet` retains the corresponding exact `Command` values in a
private table. Tokens are SHA-256-derived opaque identifiers over a domain
separator, the session identity, exact decision identity and canonical action
ordinal. The digest is an identity binding, not an authorization credential;
session ownership and authorization remain independent controls.

The table binds at most 256 commands and rejects empty, mixed-decision,
duplicate, noncanonical and internal battle-start sets. Public action order is
therefore exactly the replay-canonical order supplied by `DecisionPoint`.
Version one describes use-ability, use-interrupt, pass-interrupt and concede
commands. Battle start remains an internal settlement boundary, and the schema
reserves `battle_choice` until the combat command facade provides that exact
command family.

Selection checks the caller's expected decision before token lookup. Forged,
stale and tokens from another session fail without reaching `Battle::apply`.
The selected command remains opaque outside the crate and its debug output is
redacted. A session consumes/replaces the complete table after a successful
commit; that mutation rule is implemented with authoritative sessions in Phase
2.
