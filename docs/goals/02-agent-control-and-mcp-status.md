# Goal 02 Status — Agent Control API and MCP Adapter

This is the persistent execution ledger for
[Goal 02](02-agent-control-and-mcp.md). Update it in the same commit as every
implementation batch.

## Goal state

| Field | Value |
|---|---|
| Goal ID | `agent-control-mcp-v1` |
| State | `InProgress` |
| Prerequisite | Goal 01 `Complete` at or after `b23f900` |
| Active phase | Phase 4 — Remote Streamable HTTP |
| Next unblocked batch | `G02-P4-B1` |
| Last completed batch | `G02-P3-B5` |
| Last completed commit | This row's containing commit |
| MCP specification baseline | Frozen `2025-11-25` |
| Agent schema revision | Frozen `agent-api-v1` / `1746004f…6725` |
| SDK lock | Official `rmcp 2.2.0` / tag `rmcp-v2.2.0` / Apache-2.0 |
| Standard scenario denominator | Six frozen `scenario.standard-v1.*` production scenarios |
| Blocking condition | None |

Allowed states are `ReadyToStart`, `InProgress`, `Blocked` and `Complete`.
`Blocked` requires that no independent batch can progress and must name the
external evidence or decision required. Phase completion is not goal completion.

## Phase ledger

| Phase | State | Exit evidence |
|---|---|---|
| Phase 0 — Protocol/capability/threat model | `Complete` | Surface audit; MCP/SDK capability lock; `agent-api-v1` schema/budgets; 19-case threat model and fail-closed startup policy |
| Phase 1 — Types and observation | `Complete` | Protocol-neutral responsibility split; exact owned values; bounded player projection/events; private exact-command token table; separately gated marked debug mode; embedded frozen schemas/goldens and seeded property contracts |
| Phase 2 — Authoritative sessions | `Complete` | All six production scenarios finish from public observations/tokens at exact seeded hashes; every replay verifies; bounded owner registry race/expiry/close/quota proofs; strict projection/step/registry/memory baseline |
| Phase 3 — Local MCP | `Complete` | Independent raw JSON-RPC child client discovers seven typed tools/resources/templates/prompt, completes the frozen basic battle in eight external actions, rejects stale/malformed calls inertly, survives advisory cancellation, exports nine-command replay, closes, verifies the same `5021cdd6…1b507ec` hash and shuts down with protocol-only stdout/empty stderr; Inspector fixture and 10-case evidence retained |
| Phase 4 — Remote HTTP | `Pending` | Pending |
| Phase 5 — Hardening/freeze | `Pending` | Pending |

## Batch ledger

Replace `Pending` with `InProgress` only for the one active implementation
batch. A completed row records the containing commit, exact checks and concise
evidence summary.

| Batch | State | Commit | Validation/evidence | Result |
|---|---|---|---|---|
| `G02-P0-B1` | `Complete` | This row's containing commit | `node tools/agent-control/verify-surface-audit.mjs`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Frozen seven use cases, six production Standard scenarios, dependency layers, decision ownership, three narrow application seams and forbidden core changes; policy is bound to the Goal 01 production bundle. |
| `G02-P0-B2` | `Complete` | This row's containing commit | `node tools/agent-control/verify-mcp-sdk-lock.mjs`; `cargo test --manifest-path tools/mcp-sdk-capability/Cargo.toml --locked`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Frozen MCP `2025-11-25` and official `rmcp 2.2.0` with exact tag/checksums/features/Apache-2.0 licenses; executable goldens prove stdio, Streamable HTTP, tools/schema/structured output, resources/templates, cancellation and errors, with unsupported assumptions explicit. |
| `G02-P0-B3` | `Complete` | This row's containing commit | `node tools/agent-control/verify-agent-api-v1.mjs`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Frozen observation/action/error schemas, canonical string numerics, default/debug visibility policy, cursor semantics, response/retention/settlement bounds and ordinary/trigger-heavy/error goldens at schema bundle `1746004f…6725`. |
| `G02-P0-B4` | `Complete` | This row's containing commit | `node tools/agent-control/verify-threat-model.mjs`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Frozen 19 threats and controls for ownership, forgery/staleness, payload/replay abuse, prompt/data separation, response loss, races, origins, auth/scopes/tenancy, rate/quota/expiry/cancellation, redaction, stdio, drift, visibility and adapter isolation; three startup profiles include fail-closed non-loopback requirements. |
| `G02-P1-B1` | `Complete` | This row's containing commit | `node tools/workspace/verify-dependencies.mjs`; `cargo test -p starclock-agent-api --all-targets --all-features`; `cargo clippy -p starclock-agent-api --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Added dependency-free, protocol-neutral `starclock-agent-api` with public `schema`, `observation`, `action`, `session` and `error` responsibilities; workspace guard forbids unreviewed dependencies and reverse/protocol coupling. |
| `G02-P1-B2` | `Complete` | This row's containing commit | `node tools/workspace/verify-dependencies.mjs`; `cargo test -p starclock-agent-api --all-targets --all-features`; `cargo clippy -p starclock-agent-api --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Implemented checked revision/scenario/opaque IDs, canonical string integer/fixed-point/hash values, owned battle/team/unit/effect/timeline/status DTOs, 23 stable errors and deterministic serde/debug conversion with secret redaction and ordered context. |
| `G02-P1-B3` | `Complete` | This row's containing commit | `node tools/workspace/verify-dependencies.mjs`; `cargo test -p starclock-agent-api --all-targets --all-features`; `cargo clippy -p starclock-agent-api --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Added an allowlisted stable-boundary projection with canonical team/unit/effect/timeline order, checked exact health conversion, hard collection/event bounds and payload-free cursor pages; negative tests prove default JSON omits enemy AI, automatic ability, rules/modifiers, effect internals, commands and unpublished intent. Existing Goal 01 views were sufficient, so no core query was added. |
| `G02-P1-B4` | `Complete` | This row's containing commit | `node tools/workspace/verify-dependencies.mjs`; `cargo test -p starclock-agent-api --all-targets --all-features`; `cargo clippy -p starclock-agent-api --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Implemented frozen action DTOs and a 256-entry private exact-command table with replay-canonical order, deterministic bounded summaries and SHA-256 opaque tokens scoped by session, decision and ordinal. Tests prove stale, forged, cross-session, mixed-decision, noncanonical and internal-start inputs fail before command selection, while selected-command debug stays redacted. |
| `G02-P1-B5` | `Complete` | This row's containing commit | `node tools/workspace/verify-dependencies.mjs`; `cargo test -p starclock-agent-api --all-targets --all-features`; `cargo clippy -p starclock-agent-api --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Added a separately typed debug projection requiring an explicit in-process capability acknowledgement and always emitting `omniscient_debug` plus `debug_authorized:true`; missing capability fails before projection and negative JSON tests prove the default battle value cannot contain either debug marker. The frozen v1 schema adds no hidden payload, so richer debug facts remain revision-gated. |
| `G02-P1-B6` | `Complete` | This row's containing commit | `node tools/agent-control/verify-agent-api-v1.mjs`; `node tools/workspace/verify-dependencies.mjs`; `cargo test -p starclock-agent-api --all-targets --all-features`; `cargo clippy -p starclock-agent-api --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Embedded and published the exact schema/golden bundle at `1746004f…6725`; added an exact `schema_revision` error field and bounded ordered detail builder; 512-case seeded properties prove all `u64`/`i64` JSON round trips, unknown-revision rejection and insertion-order-independent canonical details, while tests bind implementation limits and stable error output to policy/goldens. Phase 1 complete. |
| `G02-P2-B1` | `Complete` | This row's containing commit | `node tools/workspace/verify-dependencies.mjs`; `cargo test -p starclock-data --all-targets --all-features`; `cargo test -p starclock-cli --all-targets --all-features`; `cargo test -p starclock-agent-api --all-targets --all-features`; `cargo clippy -p starclock-data -p starclock-cli -p starclock-agent-api --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Promoted the frozen Standard-v1 factory from private CLI code to a shared validated `starclock-data` seam and made the CLI delegate to it. `AgentSessionFactory` shares immutable production catalogs and accepts only checked session/scenario IDs, authored-or-exact seed policy and player visibility; each isolated session privately owns one battle, replay identities and empty incremental trace. All six identities/default seeds, explicit-seed reproducibility, session-ID hash inertness and unknown/debug rejection are tested. |
| `G02-P2-B2` | `Complete` | This row's containing commit | `node tools/workspace/verify-dependencies.mjs`; `cargo test -p starclock-agent-api --all-targets --all-features`; `cargo clippy -p starclock-agent-api --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Creation now settles the exact system start command to a player-owned decision; external tokens resolve only to retained commands, and synchronous settlement stops at the next player/terminal boundary under the 4,096-command budget. Team-enemy routing uses the immutable authored graph and isolated deterministic `EnemyController`, while unsupported contextual conditions fail closed. Every accepted external/enemy/system command appends an exact replay command/hash plus ordered controller identity; automatic resolver work remains inside `Battle::apply`. |
| `G02-P2-B3` | `Complete` | This row's containing commit | `node tools/workspace/verify-dependencies.mjs`; `cargo test -p starclock-agent-api --all-targets --all-features`; `cargo clippy -p starclock-agent-api --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Added the frozen typed action request/response and complete next observation, exact session/decision/hash preconditions, full-request idempotency binding and per-session 1,024-entry/512-KiB response cache. Cache insertion follows commit/settlement and canonical serialization before delivery. Simulated response loss returns byte-identical JSON without changing state/replay/controller/RNG; stale hash, forged token, conflicting key reuse and racing-equivalent loser are inert. |
| `G02-P2-B4` | `Complete` | This row's containing commit | `node tools/workspace/verify-dependencies.mjs`; `cargo test -p starclock-agent-api --all-targets --all-features`; `cargo clippy -p starclock-agent-api --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Retains the newest 8,192 payload-free summaries independently of the complete accepted command/hash trace. Observations page at most 256 events strictly after canonical opaque cursors, visibly truncate, reject future/wrong-family cursors and distinguish evicted cursors. Cached action responses include their settlement page; terminal observations have exact status with no decision/actions, and concession tests preserve ordered events plus complete replay/controller facts. |
| `G02-P2-B5` | `Complete` | This row's containing commit | `node tools/workspace/verify-dependencies.mjs`; `cargo test -p starclock-agent-api --all-targets --all-features`; `cargo clippy -p starclock-agent-api --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Exports the unchanged Goal 01 battle envelope with frozen Standard/controller identities and SHA-256 while keeping external/enemy/system attribution in a nonauthoritative sidecar. Verification reconstructs a fresh production battle and checks every accepted command/hash without touching the live session; round-trip, corruption, sidecar-inertness and operational-ID-independence tests pass. |
| `G02-P2-B6` | `Complete` | This row's containing commit | `node tools/workspace/verify-dependencies.mjs`; `cargo test -p starclock-agent-api --all-targets --all-features`; `cargo clippy -p starclock-agent-api --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Added a bounded in-memory owner registry with injected monotonic clock/opaque ID source, serialized per-session lanes, deferred ID allocation, frozen global/tenant/principal quotas, idle/absolute expiry, bounded terminal tombstones and close capacity release. Concurrent same-session races yield one commit plus one stale loser; cross-owner scheduler reordering preserves isolated hashes/replays, and invalid/quota rejection consumes no ID. Session tests were mechanically split below 1,200 LOC. |
| `G02-P2-B7` | `Complete` | This row's containing commit | `cargo test -p starclock-agent-api --test standard_session_loop`; `node tools/agent-control/verify-agent-benchmark.mjs`; `$env:STARCLOCK_BENCH_RUNNER_ID='starclock-bench-win10-i7-10700f-v1'; node tools/agent-control/verify-agent-benchmark.mjs --strict --samples 5`; `node tools/workspace/verify-dependencies.mjs`; `cargo test -p starclock-agent-api --all-targets --all-features`; `cargo clippy -p starclock-agent-api --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | A public-value scripted controller completes all six frozen Standard scenarios in 62 external steps/68 replay commands, matching every Goal 01 terminal hash and fresh replay verification. Added separated release workloads for 1,000 projections, 100 isolated ability steps, 1,000 owned registry projections and 16 resident sessions; the designated Windows x64 five-sample baseline passes reviewed timing/allocation/peak/retained budgets. Phase 2 complete. |
| `G02-P3-B1` | `Complete` | This row's containing commit | `node tools/agent-control/verify-mcp-sdk-lock.mjs`; `node tools/workspace/verify-dependencies.mjs`; `cargo test -p starclock-mcp --all-targets --all-features`; `cargo clippy -p starclock-mcp --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Added the one-way `starclock-mcp -> starclock-agent-api` adapter crate on exact default-off official `rmcp 2.2.0` features/checksums. Initialization freezes MCP `2025-11-25`, server identity and bounded trust instructions without prematurely advertising capabilities. All 23 agent failures map to exact structured tool errors; infrastructure failures are generic data-free JSON-RPC internal errors, and no combat/data/replay dependency or command construction path exists. |
| `G02-P3-B2` | `Complete` | This row's containing commit | `node tools/agent-control/verify-mcp-sdk-lock.mjs`; `node tools/workspace/verify-dependencies.mjs`; `cargo test -p starclock-agent-api --all-targets --all-features`; `cargo test -p starclock-mcp --all-targets --all-features`; `cargo clippy -p starclock-agent-api -p starclock-mcp --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Exposed exactly seven discoverable tools with concrete nested input/output schemas and structured success/error content. All operations delegate to the owned registry or production factory; no command-construction path was added. Listing returns the six exact scenario identities/default seeds, replay export uses lowercase hex with a 64 MiB decoded import cap, and fresh verification succeeds after the originating session is closed. |
| `G02-P3-B3` | `Complete` | This row's containing commit | `node tools/agent-control/verify-mcp-sdk-lock.mjs`; `node tools/workspace/verify-dependencies.mjs`; `cargo test -p starclock-data -p starclock-agent-api -p starclock-mcp --all-targets --all-features`; `cargo clippy -p starclock-data -p starclock-agent-api -p starclock-mcp --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Added two static resources, two exact templates and one fixed argument-free usage prompt. Catalog, scenario and 88-character lookups cross only generated-row-free data/application summaries; every JSON resource is marked inert and capped at 16 KiB, URI input is capped at 256 bytes, pagination cursors fail closed, and tests exclude workbook/Sora/generated/cache/private-command/reasoning markers. No subscription or list-changed capability is advertised. |
| `G02-P3-B4` | `Complete` | This row's containing commit | `node tools/agent-control/verify-mcp-sdk-lock.mjs`; `node tools/workspace/verify-dependencies.mjs`; `cargo test -p starclock-mcp --all-targets --all-features`; `cargo test -p starclock-cli --all-targets --all-features`; `cargo clippy -p starclock-mcp -p starclock-cli --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Added only `starclock mcp serve --transport stdio`: one local owner, validated shared factory, bounded registry, injected monotonic clock and process/start/ordinal session IDs. A 16 KiB newline-frame reader rejects before JSON decode; child tests prove frozen initialization/tool discovery produces only JSON-RPC stdout with empty stderr, while oversized input yields zero stdout and generic stderr without echo. No listener or ambient remote identity exists. |
| `G02-P3-B5` | `Complete` | This row's containing commit | `node tools/agent-control/verify-mcp-stdio-conformance.mjs`; `node tools/agent-control/verify-mcp-sdk-lock.mjs`; `node tools/workspace/verify-dependencies.mjs`; `cargo test -p starclock-cli --test mcp_stdio --all-features`; `cargo test -p starclock-mcp -p starclock-cli --all-targets --all-features`; `cargo clippy -p starclock-mcp -p starclock-cli --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-targets --all-features`; `cargo fmt --all -- --check`; `git diff --check` | Added an independent interactive stdio JSON-RPC client plus Inspector launch and reviewed 10-case evidence fixtures. It freezes discovery schemas/context, completes the basic scenario using only public legal values, proves stale and malformed calls leave the accepted hash unchanged, survives advisory cancellation, exports and verifies the exact replay after close, and requires clean EOF with 24 protocol-only responses and empty stderr. Phase 3 complete. |
| `G02-P4-B1` | `Pending` | — | — | — |
| `G02-P4-B2` | `Pending` | — | — | — |
| `G02-P4-B3` | `Pending` | — | — | — |
| `G02-P4-B4` | `Pending` | — | — | — |
| `G02-P4-B5` | `Pending` | — | — | — |
| `G02-P5-B1` | `Pending` | — | — | — |
| `G02-P5-B2` | `Pending` | — | — | — |
| `G02-P5-B3` | `Pending` | — | — | — |
| `G02-P5-B4` | `Pending` | — | — | — |
| `G02-P5-B5` | `Pending` | — | — | — |
| `G02-P5-B6` | `Pending` | — | — | — |

## Frozen identities and budgets

Populate these rows only from committed capability/schema/baseline evidence.

| Identity | Revision/digest | Evidence |
|---|---|---|
| MCP specification | `2025-11-25` | [`mcp-sdk-lock.json`](../../policy/mcp-sdk-lock.json) |
| MCP Rust SDK/toolchain | `rmcp 2.2.0`; Rust `1.97.0`; Apache-2.0 | [`mcp-sdk-capabilities.json`](../../evidence/agent-control-mcp-v1/protocol/mcp-sdk-capabilities.json) |
| Agent schema | `agent-api-v1` / `1746004f…6725` | [`agent-api-v1.json`](../../evidence/agent-control-mcp-v1/schema/agent-api-v1.json) |
| Threat model | `starclock.agent-control-threat-model.v1` / `4080b72e…c45a` | [`threat-model.json`](../../evidence/agent-control-mcp-v1/security/threat-model.json) |
| Standard scenario denominator | 6 scenarios / Goal 01 bundle `abd84f70…0440` | [`agent-control-surfaces.json`](../../policy/agent-control-surfaces.json) |
| Observation/event limits | 256 KiB observation; 256 events/page; 8,192 retained summaries | [`agent-api-v1.json`](../../policy/agent-api-v1.json) |
| Settlement limits | 4,096 commands; 65,536 events; 262,144 operations | [`agent-api-v1.json`](../../policy/agent-api-v1.json) |
| Session/registry limits | 1,024 global / 64 tenant / 16 principal; idle 1,800 s / max 14,400 s | [`agent-control-threat-model.json`](../../policy/agent-control-threat-model.json) |
| Performance workload | `g02-agent-session-baseline-v1` / `e99df5c0…9a09` | [`phase2-baseline-windows-x64.json`](../../evidence/agent-control-mcp-v1/performance/phase2-baseline-windows-x64.json); remote/MCP extension remains `G02-P4-B5` |
| MCP stdio conformance | `starclock.mcp-stdio-conformance-evidence.v1`; 10 cases; `5021cdd6…1b507ec` | [`mcp-stdio-conformance.json`](../../evidence/agent-control-mcp-v1/protocol/mcp-stdio-conformance.json) |

## Decision record

| Date | Decision | Rationale/effect |
|---|---|---|
| 2026-07-22 | MCP is an adapter over `starclock-agent-api`, not the combat API or only service protocol. | Keeps deterministic simulation independent from JSON-RPC, transports and model hosts; supports in-process/high-throughput consumers. |
| 2026-07-22 | Agents select opaque offered-action tokens bound to retained exact commands. | Prevents fabricated damage/targets/costs and reuses the authoritative command legality boundary. |
| 2026-07-22 | Default sessions expose player decisions and settle authored enemy/automatic decisions internally. | Keeps tool calls at meaningful turn boundaries and preserves authored enemy behavior. |
| 2026-07-22 | Remote non-loopback service fails closed without authorization and origin policy. | Avoids accidentally publishing an unauthenticated state-mutating MCP server. |
| 2026-07-22 | No model provider or private chain-of-thought is part of Goal 02. | The server is interoperable infrastructure, and replay depends only on accepted commands. |
| 2026-07-22 | Only player-owned decisions are external; system and enemy decisions settle inside the session. | Reuses exact offered commands while keeping authored automation deterministic and replay-visible. |
| 2026-07-22 | Goal 02 may add only narrow application/data composition seams over Goal 01. | Scenario lookup/construction and controller coordination do not authorize combat-rule, lifecycle, RNG, hash or replay redesign. |
| 2026-07-22 | MCP revision `2025-11-25` and official Rust SDK `rmcp 2.2.0` are frozen. | The locked executable fixture proves the used surface; newer SDK capabilities and protocol revisions remain opt-in future work. |
| 2026-07-22 | `agent-api-v1` transports every authoritative number as a canonical integer string. | Avoids JSON floating-point and cross-language safe-integer loss while preserving exact fixed-point backing values. |
| 2026-07-22 | Event summaries are bounded/cursor-paged while complete facts remain replay-owned. | Keeps observations finite without weakening authoritative audit or silently truncating settlement. |
| 2026-07-22 | Non-loopback HTTP startup requires the complete remote security prerequisite set. | Missing TLS/proxy attestation, token validation, exact origins, identity/scopes, proxy trust, quotas/rates/limits or audit sink is a startup error. |
| 2026-07-22 | Expiry/cancellation are checked outside an atomic domain commit. | Operational time and cancellation cannot create platform-dependent partial battle state; idempotent retry resolves delivery ambiguity. |
| 2026-07-22 | Version-one public intent remains absent until explicitly authored as visibility-safe content. | The player projection never infers future intent from enemy AI state, candidates, automatic abilities or retained commands; transformed presence normalizes to public `present`. |
| 2026-07-22 | Offered-action token digests are deterministic identity bindings, not authorization credentials. | Tokens bind session, decision and canonical ordinal to a private exact-command table; independent session ownership/auth checks remain mandatory, and successful commit replaces the whole decision table. |
| 2026-07-22 | Frozen v1 debug mode is a separately gated and marked envelope over the bounded battle schema, without extra hidden fields. | The schema defines the policy/authorization markers but no debug payload. AI/rule/command/seed/RNG internals therefore stay absent; adding them requires a new reviewed schema revision rather than an unversioned extension. |
| 2026-07-22 | Idempotent retries preserve the original response bytes, including its original `idempotent_replay:false` value. | The frozen threat model requires the same bytes after response loss; rewriting the field on a cache hit would make the retry a different response. Full request equality and cache lookup still distinguish retry from conflicting reuse internally. |
| 2026-07-22 | Controller attribution is an export sidecar, not a new canonical replay record. | Goal 01's battle envelope remains the authority for accepted commands and resulting hashes; diagnostics can be discarded or changed without altering replay bytes, identity or verification. |
| 2026-07-22 | Registry creation and each session use separate serialization lanes. | A short creation lane makes quota checks and deferred ID allocation atomic; a per-session lane rechecks action preconditions without blocking unrelated battles. Expiry time is sampled before lane entry and never during a domain commit. |
| 2026-07-22 | Closed and expired sessions retain only a bounded terminal tombstone. | Close/expiry releases active quotas and drops battle state immediately while preserving stable terminal errors for the newest 1,024 identities; older markers may become `unknown_session` rather than creating an unbounded operational store. |
| 2026-07-22 | Agent performance rows separate projection, stepping, registry access and resident memory. | The release-only harness uses the existing pinned allocation counter as a dev dependency; JSON/MCP serialization and transport overhead remain separate later workloads rather than being attributed to the protocol-neutral API. |
| 2026-07-22 | Domain/application failures are MCP tool errors; only routing/decoding/infrastructure failures are JSON-RPC errors. | Independent clients receive the exact frozen agent error in `structuredContent` with `isError:true`; protocol internals stay generic and data-free, matching the SDK's documented failure split. |
| 2026-07-22 | Local MCP replay artifacts use bounded lowercase hexadecimal and standalone verification requires the frozen scenario plus seed policy. | Avoids a new codec dependency, makes the accepted wire alphabet exact and allows verification after session closure; the decoded replay import remains capped at 64 MiB and canonical replay bytes are unchanged. |
| 2026-07-22 | MCP catalog context is a small generated-row-free summary surface and its sole prompt is fixed and authority-neutral. | Static manifest/rules resources plus exact scenario/character templates stay below 16 KiB, mark text as inert and expose no workbook, generated/cache record, long proprietary text or hidden runtime state. The prompt accepts no arguments and cannot grant authorization or alter rules. |
| 2026-07-22 | Local stdio uses a 16 KiB complete JSON-RPC frame bound and owns its runtime/operational identity inside `starclock-mcp`. | Bounds allocation before decode and keeps CLI stdout protocol-only. The tighter transport limit composes with the 64 MiB application replay ceiling; stdio replay calls must satisfy both. Local IDs are uniqueness handles rather than credentials or deterministic inputs. |
| 2026-07-22 | Phase 3 acceptance uses an independent raw JSON-RPC child client; MCP Inspector remains a manual interoperability surface. | The automated proof crosses the production CLI/SDK/stdin/stdout boundary without adapter internals, fixes exact hashes/counts and clean shutdown, and stays deterministic without making a floating Inspector package version part of the conformance authority. |

Add architectural decisions here before implementing a deviation from the goal
or normative design. A decision cannot silently weaken a terminal gate.

## Blockers and research cases

| ID | State | Question/blocker | Owner batch | Resolution/evidence |
|---|---|---|---|---|
| `G02-R01` | `Resolved` | Which exact official Rust MCP SDK revision and features satisfy the frozen stdio/HTTP/schema/cancellation contract? | `G02-P0-B2` | Official `rmcp 2.2.0`, default-off features `client`, `macros`, `server`, `transport-io`, `transport-streamable-http-server`; exact checksums, licenses and passing fixture are committed. |
| `G02-R02` | `Resolved` | Which existing Goal 01 view methods are sufficient for player-visible projection, and which narrow queries are missing? | `G02-P0-B1` / `G02-P1-B3` | Existing immutable battle, unit, effect, team and timeline views provide the complete frozen player-visible allowlist. Integration tests prove canonical projection and hidden-state absence; no combat query was added. |
| `G02-R03` | `Resolved` | Which current decisions are external player decisions versus authored enemy or automatic orchestration boundaries? | `G02-P0-B1` / `G02-P2-B2` | `Team(Player)` is external; `System` and `Team(Enemy)` settle internally from exact offered commands; automatic timeline work drains in `Battle::apply`. |

Research does not authorize speculative production behavior. Record primary
sources, executed fixtures and exact limitations. Continue independent work when
a case does not block it.

## Terminal checklist

### Agent API and observation

- [x] `starclock-agent-api` is protocol-neutral and responsibility-separated.
- [x] Exact versioned observation/action/error schemas are frozen.
- [x] Player-visible projection passes hidden-information and bound tests.
- [x] Debug projection is separately authorized and visibly marked.
- [x] Offered tokens bind retained exact commands and reject stale/forged use.

### Session and replay

- [x] Creation accepts validated production identities rather than arbitrary
      untrusted specs/programs.
- [x] Player actions settle through bounded enemy/automatic decisions.
- [x] Idempotency, response-loss, race, expiry and close tests pass.
- [x] All frozen Standard scenarios finish through the agent session loop.
- [x] Exported replays verify every accepted external/automatic command/hash.

### MCP and remote service

- [x] MCP protocol and official Rust SDK capability lock are committed.
- [x] All seven tools and bounded resources have schema/conformance evidence.
- [x] Stdio end-to-end play passes with protocol-only stdout.
- [ ] Streamable HTTP end-to-end play matches in-process traces.
- [ ] Non-loopback startup, authorization, scopes, origins, tenant ownership,
      limits and rate policies fail closed and pass adversarial tests.

### Engineering and release

- [ ] Windows/Linux/macOS native golden evidence is retained.
- [ ] Fuzz/property/concurrency/security suites pass with reproducible inputs.
- [ ] Performance budgets cover projection, stepping, serialization and memory.
- [ ] Dependency, license, public API, file-size and secret/log audits pass.
- [ ] Clean-checkout acceptance and generated-drift checks pass.
- [ ] Contracts/examples are frozen and `G02-P5-B6` is committed cleanly.

## Completion record

| Field | Value |
|---|---|
| Completion commit | Pending |
| Agent schema digest | Pending |
| MCP capability/conformance evidence | Pending |
| Standard scenario result | Pending |
| Cross-platform evidence | Pending |
| Performance evidence | Pending |
| Clean-checkout evidence | Pending |
