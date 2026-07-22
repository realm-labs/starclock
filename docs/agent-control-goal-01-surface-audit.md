# Agent control Goal 01 surface audit

`G02-P0-B1` freezes the inputs that Goal 02 may build on. The machine-readable
contract is [`policy/agent-control-surfaces.json`](../policy/agent-control-surfaces.json),
and `tools/agent-control/verify-surface-audit.mjs` checks its scenario and bundle
bindings against the released Goal 01 evidence.

## Frozen use cases and denominator

Goal 02 controls one validated, ephemeral Standard session: create it from
production identities and a seed; observe a bounded player decision; apply an
opaque offered action; settle authored non-player work; page events; export and
verify replay; and close or expire the session. The same protocol-neutral
contract must support in-process callers, local stdio MCP, and authorized remote
Streamable HTTP MCP.

The denominator is exactly the six `scenario.standard-v1.*` rows in the Goal 01
manifest. Their stable scenario and encounter identities, numeric production
definition IDs, and frozen default seeds are recorded in the policy. Adding a
scenario is a revisioned contract change; silently testing a subset is not
allowed. The production catalog is the Goal 01 bundle with SHA-256
`abd84f70461675337092d12377db53f08b4562114fa90aa0b37ad869e9270440`.

## Reusable public boundaries

The audit found the authoritative mechanics already exposed at the correct
layers:

- `starclock-combat` owns `Battle`, exact offered `Command` values,
  `DecisionPoint` ownership, atomic `Battle::apply`, immutable `BattleView`,
  typed events/faults, RNG draw count, and canonical state hashes.
- `starclock-ai` selects from offered values and returns auditable baseline or
  authored-enemy diagnostics without mutable battle access.
- `starclock-activity` owns generic activity commands, projections and battle
  handoff; `starclock-mode-standard` owns the ordinary Standard profile and
  scenario vocabulary.
- `starclock-build` compiles validated build identities to generic combat
  inputs. `starclock-data` loads and shares immutable production catalogs and
  resolves numeric Standard descriptors.
- `starclock-replay` owns canonical battle/activity envelopes and verification.

These are composition inputs, not permission to expose their internal types as
the public agent schema.

## Decision ownership

Only `DecisionOwner::Team(TeamSide::Player)` is an external agent boundary.
System decisions are applied by session orchestration using the exact offered
lifecycle command. Enemy decisions are selected from exact offered commands
using authored enemy graph/state data and recorded controller diagnostics.
Automatic timeline work already drains synchronously inside `Battle::apply`.
Terminal and faulted battles have no external decision.

## Proven narrow seams

Goal 01 deliberately leaves three application-layer composition seams: stable
enumeration/resolution of production Standard scenario identities (the current
numeric lookup is insufficient for untrusted string input), construction of the
frozen production battle/activity input (the existing CLI implementation is
private), and coordination/recording of authored enemy decisions. These seams
may be added in `starclock-data`, a responsibility-specific application crate,
or `starclock-agent-api`; they do not justify changing combat formulas,
lifecycle, command legality, canonical replay, or RNG.

## Dependency and change boundary

Dependencies flow upward from deterministic domain crates, through data and the
protocol-neutral agent API, to adapters. `starclock-agent-api` may depend on
Goal 01 libraries but not on MCP, an async runtime, HTTP, or authentication.
`starclock-mcp` may depend on the agent API; no lower crate may depend on either
new crate.

Forbidden changes are enumerated verbatim in the policy. In particular, MCP and
operational session concepts never enter core canonical state; every mutation
continues through `Battle::apply`/`Activity::apply`; untrusted callers never
supply arbitrary Rule IR or `BattleSpec`; and Goal 02 does not grow into model
hosting, durable storage, challenge/universe modes, or prompt/reasoning capture.

