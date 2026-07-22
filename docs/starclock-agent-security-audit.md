# Starclock Agent Security and Architecture Audit

Goal 02 retains a machine-verified audit at
[`security-audit.json`](../evidence/agent-control-mcp-v1/security/security-audit.json).
Run `node tools/agent-control/verify-agent-security-audit.mjs` to reproduce it.
The report is generated from the current Cargo graph, tracked production Rust,
the reviewed dependency/license inventory and the frozen MCP, authorization and
HTTP policies; byte drift is a failure.

## Dependency and API boundary

The graph has one protocol edge: `starclock-mcp` depends on
`starclock-agent-api`. The adapter has no direct combat, data, AI or replay
dependency. The protocol-neutral API depends only on the reviewed Goal 01
composition seams plus serialization and hashing. Domain crates have no MCP,
HTTP, agent API or model-provider dependency, and no provider package is
resolved anywhere in the workspace.

Public declarations in `starclock-agent-api` are scanned for transport and
implementation types. MCP, Axum, Tokio, `BattleState`, exact `Command`, Sora,
the fixed-point backend and private hash/numeric crates cannot cross that API.
The full workspace dependency verifier remains authoritative for exact features
and dependency direction.

## Source, unsafe and license review

All 24 Goal 02 production Rust files inherit the workspace `unsafe_code =
"forbid"` lint and contain zero unsafe syntax. Handwritten modules are limited
to 1,200 physical lines and facades to 200, with no exceptions. The HTTP module
is the only file at or above 95% utilization (1,153 lines) and must be split
before further growth crosses the limit.

The central inventory covers all 136 resolved registry packages. The MCP lock
is the official `rmcp 2.2.0`/`rmcp-macros 2.2.0` pair under Apache-2.0 with exact
workspace lockfile checksums and default-off reviewed features. An unreviewed
package, version, feature, license or checksum fails the repository gate.

## Secrets, output and remote controls

Agent API and MCP production sources contain no logging macros. Opaque session
and action values and the authorization grant have explicit redacted debug
forms; authorization failures are generic, and a retained HTTP regression test
proves an inbound bearer value is not echoed. Stdio stdout is owned directly by
the SDK transport and the CLI reports process failures only on stderr.

The audit binds all eight OAuth scopes and the exact operation matrix, origin
and bearer limits, request/response and worker limits, session quotas,
settlement/event/replay bounds and the explicit loopback-only startup profile.
Starclock exposes no non-loopback listener: adding one requires the complete
TLS/proxy attestation and security-audit profile described by the threat model,
not a relaxation of the current policy.

The audit is static architecture evidence plus executable regression binding.
It does not replace the authorization, tenant, origin, conformance, fuzz,
cross-platform or clean-checkout suites; the universal repository gate runs it
alongside those contracts.
