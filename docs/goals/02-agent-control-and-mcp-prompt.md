# Goal 02 Launch Prompt

Use the prompt below to start or resume persistent execution of
[Goal 02](02-agent-control-and-mcp.md). It requires batch-by-batch execution and
must not stop after planning, scaffolding or the first working MCP tool.

```text
Create and start a persistent goal with this objective:

Implement Starclock Goal 02 completely: a protocol-neutral, deterministic agent
control/session API over the completed Standard battle runtime, plus conformant
local stdio and authorized remote Streamable HTTP MCP adapters. Continue until
every terminal gate in the Goal 02 plan and status ledger is proven. Do not set a
token budget.

Repository execution contract
=============================

1. Before changing code, read these files completely:

   - docs/goals/02-agent-control-and-mcp.md
   - docs/goals/02-agent-control-and-mcp-status.md
   - docs/22-agent-control-and-mcp.md
   - docs/README.md
   - docs/01-core-battle-model.md
   - docs/06-rust-architecture.md
   - docs/08-engineering-standards.md
   - docs/09-determinism-and-numerics.md
   - docs/10-lifecycle-and-resolution.md
   - docs/13-enemy-ai-and-encounters.md
   - docs/16-replay-cli-and-engine-integration.md
   - docs/18-standard-and-challenge-modes.md, but only the Standard and generic
     extension boundaries
   - docs/19-activity-core-and-mode-extension.md
   - docs/20-core-implementation-design.md
   - docs/goal-01-release-contract.md
   - docs/headless-cli-contract.md
   - docs/activity-replay-and-controller-diagnostics.md
   - docs/baseline-controller-execution.md
   - docs/performance-benchmark-baseline.md
   - Cargo.toml and the public modules of starclock-combat, starclock-ai,
     starclock-replay, starclock-data, starclock-build, starclock-activity,
     starclock-mode-standard and starclock-cli

2. Inspect the worktree, recent commits and the full Goal 02 ledger. Preserve
   user changes and never use destructive Git operations.

3. Treat the plan and normative design as contracts. MCP is an adapter over a
   protocol-neutral API. Never add MCP/HTTP/model-provider dependencies to
   domain crates, expose internal BattleState/Sora/fixed-point backend types, or
   create a second combat mutation path.

4. During G02-P0-B2, verify current MCP facts only against official MCP
   specification/SDK sources and executed local fixtures. Freeze an exact SDK
   revision, checksum, features and licenses before production implementation.
   Do not infer API support from an example written for another revision.

Persistent execution loop
=========================

Repeat until the Goal 02 ledger is Complete:

1. Reload the plan and ledger. After context compaction or resumption, reread
   every normative file affected by the next batch.
2. Select the earliest unblocked Pending batch whose dependencies are complete.
   Mark only that batch InProgress and announce its intended outcome/gates.
3. Implement exactly that responsibility. Include its tests, schemas, fixtures,
   dependency policy, security evidence and documentation in the same batch.
4. Run batch-specific tests and universal repository gates. Fix failures rather
   than weakening or deferring a gate to obtain a commit.
5. Update the ledger with exact commands, evidence paths, identities/digests,
   decisions and blockers. Keep phase and Next unblocked batch accurate.
6. Review the diff for scope, dependency direction, public API, deterministic
   behavior, hidden-information leakage, generated drift, secret/log leakage,
   file size and accidental user changes.
7. Commit exactly the completed batch using:

   <type>(<scope>): <batch-id> <concise imperative summary>

   Allowed types are build, chore, ci, data, docs, feat, fix, perf, refactor,
   revert, style and test. Scope is lowercase kebab-case. Batch ID exactly
   matches the ledger, for example:

   feat(agent): G02-P2-B3 enforce idempotent action application
   feat(mcp): G02-P3-B2 expose battle control tools

   After committing, verify the subject matches:

   ^(build|chore|ci|data|docs|feat|fix|perf|refactor|revert|style|test)\([a-z0-9][a-z0-9-]*\): G02-P[0-9]+-B[0-9]+ .+

   Amend only the newly created commit if its subject is invalid. Do not rewrite
   earlier commits, push, publish or open a PR unless separately authorized.
8. Immediately start the next unblocked batch. A commit or phase boundary is
   progress, not a reason to return control or complete the persistent goal.

Implementation rules
====================

- Keep one active implementation batch. Independent read-only research may be
  parallel only when it cannot produce conflicting decisions or writes.
- Agents select a retained exact command from the active DecisionPoint. Never
  accept client-authored damage, costs, selectors, RNG results or equivalent
  reconstructed commands.
- Keep authoritative numeric values exact as scaled integers/canonical decimal
  strings. Never use JSON f32/f64 as an authoritative value.
- PlayerVisible output must be safe by construction. Do not fetch hidden state
  and then rely only on serialization omission. OmniscientDebug is explicit,
  marked, disabled by default and separately scoped remotely.
- One action settles synchronously to the next external decision or terminal
  boundary, under fixed budgets. Do not expose one MCP call per hit, event,
  trigger or internal operation.
- Serialize mutation per session. Use expected decision ID, state hash and
  decision-scoped action token. Rejection must not mutate state, replay or RNG.
- Treat response delivery as separate from commit. Cache the exact committed
  result under an idempotency key so a retry cannot double-apply an action.
- Record accepted player, enemy and automatic commands in replay. Verification
  must not call the external model again and must retain Goal 01 compatibility.
- Operational session IDs, tenant/auth data, clocks, TTLs, metrics and transport
  cancellation never enter battle state, event order, RNG or canonical hashes.
- Remote non-loopback operation must fail closed without the frozen MCP auth and
  origin contracts. Do not invent home-grown bearer semantics or log secrets.
- Bind sessions and idempotency to tenant authority and enforce independent
  catalog/create/observe/act/replay/debug scopes.
- Stdio stdout is protocol-only. Diagnostics and logs use stderr.
- Share immutable catalogs, not mutable sessions. Do not verify live actions by
  replaying growing prefixes; retain incremental state.
- Benchmark combat, projection, JSON/MCP and registry costs separately. MCP is
  not the bulk RL or high-throughput verifier path.
- Do not add provider inference, chain-of-thought storage, universe/challenge
  content, Bevy/UI, accounts, distributed persistence or official-client
  automation.
- Preserve the 1,200-LOC rule, responsibility-based modules and narrow exports.
- If the SDK cannot satisfy a documented capability, record the exact executed
  limitation and make the narrowest reviewed adapter decision. Do not silently
  drop a terminal transport/security requirement.
- If a batch is externally blocked, record it and continue independent work.
  Mark the goal Blocked only when no meaningful in-scope batch can progress.

Completion protocol
===================

When all batches appear complete:

1. Run the complete Goal 02 acceptance suites from an isolated clean checkout.
2. Run all frozen Standard scenarios through the agent loop and verify replay
   plus in-process/stdio/HTTP trace equivalence.
3. Verify stdio framing and remote fail-closed authorization/origin/tenant
   behavior with retained conformance evidence.
4. Verify native Windows/Linux/macOS schema and command/event/hash goldens.
5. Run stable-runner session/projection/serialization/load budgets and retain
   runner identity and inputs.
6. Complete every terminal checklist and completion-record field in the ledger.
7. Commit the completion record as G02-P5-B6 and verify a clean worktree.
8. Mark the persistent goal Complete only after that commit succeeds. Report
   the completion commit, schema digest, SDK/spec lock, scenario totals,
   validation commands, cross-platform evidence and performance report.

Start with the Next unblocked batch in the ledger. Do not respond with another
plan; execute the loop.
```
