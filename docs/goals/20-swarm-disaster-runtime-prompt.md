# Goal 20 Launch Prompt

Run this as a persistent execution goal. The user selected the current clean
worktree; do not create another worktree unless the user later changes that
instruction. Stop and reconcile if concurrent work begins touching the same
runtime files.

```text
Create or resume the persistent goal whose objective is to complete Starclock
Goal 20: implement and release the deterministic Version 4.4 Simulated
Universe: Swarm Disaster runtime from the immutable Goal 09 Candidate Sora
bundle, including complete mechanics, real nested battles, seeded runs, replay,
baseline AI, CLI, agent API and MCP control. Continue batch-by-batch until every
terminal gate is proved. Do not set a token budget.

Read completely before acting:

1. docs/goals/20-swarm-disaster-runtime.md
2. docs/goals/20-swarm-disaster-runtime-status.md
3. docs/goals/09-swarm-disaster-reference-data.md
4. docs/goals/09-swarm-disaster-reference-data-status.md
5. docs/goals/14-gold-and-gears-runtime.md
6. docs/goals/14-gold-and-gears-runtime-status.md
7. docs/goals/07-standard-universe-mechanics-completion-status.md
8. docs/06-rust-architecture.md
9. docs/07-configuration-pipeline.md
10. docs/08-engineering-standards.md
11. docs/09-determinism-and-numerics.md
12. docs/10-lifecycle-and-resolution.md
13. docs/11-rule-ir-and-native-handlers.md
14. docs/12-modifier-and-snapshot-pipeline.md
15. docs/13-enemy-ai-and-encounters.md
16. docs/14-run-core-and-universe-modes.md
17. docs/16-replay-cli-and-engine-integration.md
18. docs/19-activity-core-and-mode-extension.md
19. docs/25-standard-universe-runtime-design.md
20. docs/26-mode-extension-and-evolution.md
21. docs/27-standard-universe-end-to-end-integration.md
22. docs/activity-replay-and-controller-diagnostics.md
23. docs/starclock-agent-integration-contract.md
24. docs/dependency-and-tool-policy.md
25. docs/ci-platform-matrix.md
26. content-manifests/swarm-disaster-v1/README.md
27. evidence/swarm-disaster-reference-v1/release-evidence.json
28. evidence/reference-integration-v1/merged-mode-audit.json

Before the first runtime mutation:

- inspect the worktree, active persistent goal and complete Goal 20 ledger;
- verify Goals 01–09 and Goal 14 completion snapshots and current compatibility;
- verify the merged Candidate integration audit;
- verify Goal 09 bundle and normalized-pack digests match the frozen values;
- prove Goal 09 reference/manifests/workbooks/generated/evidence roots are
  unchanged and protected;
- honor the user's current-worktree choice; do not create another worktree;
- stop and reconcile if another task begins overlapping runtime changes.

Execution loop:

1. Select the earliest unblocked Pending batch and mark only it InProgress.
2. Implement the complete code, lowering, tests, evidence and documentation
   responsibility of that batch.
3. Update the ledger with exact commands, counts, hashes, decisions, policy
   outcomes and blockers. Never record an unexecuted check as passed.
4. Run focused format, Clippy and tests during implementation. Run
   `node tools/repository-check/run.mjs` before each ordinary completion and
   `node tools/repository-check/run.mjs --full` at phase/release checkpoints or
   whenever the quick gate reports deferred inputs.
5. Commit exactly one completed batch using its exact ID, for example:
   feat(swarm-disaster): G20-P2-B2 compile bounded three-plane graphs
6. Continue immediately to the next unblocked batch. Do not stop because one
   table loads, one mechanic works, a seeded run completes, a phase ends or
   context is compacted. Re-read the ledger after continuation.
7. Mark the persistent goal complete only after G20-P8-B4 is committed, the
   completion snapshot is registered and the clean-worktree verifier passes.

Architecture constraints:

- Activity::apply and Battle::apply remain the only authoritative mutation
  boundaries. Do not introduce SwarmDisasterActivity::apply.
- Extend the mode-owned profile/components in starclock-mode-universe. Move a
  genuinely shared primitive to its lowest truthful owner only with focused
  generic tests and an explicit compatibility decision.
- Never add Swarm content IDs or mode IDs to shared Activity/combat resolver
  branches. Use typed operations, Rule IR and statically composed handlers.
- Load config.sora through private generated readers and validated lowering.
  Runtime never reads Excel, normalized JSON or debug JSON and public APIs do
  not expose generated rows.
- Keep the Goal 09 bundle as a separately identified consumed component. Do
  not invalidate Standard, Gold or unrelated-mode replay identities.
- Reuse released shared content by stable identity and digest. Swarm copies own
  only their distinct state, parameters, pools and lifecycle.
- Represent Countdown, Disarray, dice and Communing/progression state as typed
  bounded Activity state with explicit scopes, carry/reset and visibility.
- Use project-owned labeled RNG streams and stable candidates. Specify draw
  consumption for empty candidates; never use generic shuffle, floating
  probability or collection iteration order.
- Every nested battle consumes one immutable Activity snapshot and returns only
  a verified declared projection. Battle cannot read or mutate live Activity.
- Baseline AI, CLI, agent and MCP select only offered commands. Adventure
  accepts only an offered ExternalOutcome and never simulates action physics.

Completeness constraints:

- P0-B2 must generate exact dispositions for all 6,963 obligations, 23 rules,
  23 fixture families and 31 policy boundaries before mechanic work begins.
- A loaded row, evaluator result or reference fixture is not runtime evidence.
  Enabled mechanics must affect production state/events/hashes or receive a
  truthful explicit non-executable disposition.
- Treat all 31 Goal 09 policies as unresolved until a Goal 20 owner records a
  versioned executable policy, proves metadata, replaces it with stronger
  evidence or marks it blocking.
- Never relabel ProjectPolicy, approximation or inference as observed parity.
  No legal Released run may reach an unresolved fail-closed branch.
- Preserve rejection byte identity, independent RNG streams, stable graph and
  option order, checked arithmetic and component-aware replay identity.
- Do not modify immutable Goal 09 artifacts to make runtime tests pass. A real
  authoring defect requires revisioned regeneration, never a direct patch.

Do not mark the goal complete until the P0 matrix covers five difficulties,
all eight Paths/Audience Dice, Disarray boundaries, every fixture and policy
family; all reachable encounters execute real battles; 6,963/23/23 coverage
and all 31 policy boundaries are terminal; replay and CLI/agent/MCP parity,
cross-platform, performance, dependency, architecture, prior-release and full
clean-checkout gates pass; and G20-P8-B4 is committed.
```
