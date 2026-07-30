# Goal 14 Launch Prompt

Run this as a persistent execution goal. Use a separate branch and worktree
when another task is writing the same checkout or shared runtime files.

```text
Create or resume the persistent goal whose objective is to complete Starclock
Goal 14: implement and release the deterministic Version 4.4 Simulated
Universe: Gold and Gears runtime from the immutable Goal 08 Candidate Sora
bundle, including complete mechanics, real nested battles, seeded runs, replay,
baseline AI, CLI, agent API and MCP control. Continue batch-by-batch until every
terminal gate is proved. Do not set a token budget.

Read completely before acting:

1. docs/goals/14-gold-and-gears-runtime.md
2. docs/goals/14-gold-and-gears-runtime-status.md
3. docs/goals/08-gold-and-gears-reference-data.md
4. docs/goals/08-gold-and-gears-reference-data-status.md
5. docs/goals/07-standard-universe-mechanics-completion-status.md
6. docs/06-rust-architecture.md
7. docs/07-configuration-pipeline.md
8. docs/08-engineering-standards.md
9. docs/09-determinism-and-numerics.md
10. docs/10-lifecycle-and-resolution.md
11. docs/11-rule-ir-and-native-handlers.md
12. docs/12-modifier-and-snapshot-pipeline.md
13. docs/13-enemy-ai-and-encounters.md
14. docs/14-run-core-and-universe-modes.md
15. docs/16-replay-cli-and-engine-integration.md
16. docs/19-activity-core-and-mode-extension.md
17. docs/25-standard-universe-runtime-design.md
18. docs/26-mode-extension-and-evolution.md
19. docs/27-standard-universe-end-to-end-integration.md
20. docs/activity-replay-and-controller-diagnostics.md
21. docs/starclock-agent-integration-contract.md
22. docs/dependency-and-tool-policy.md
23. docs/ci-platform-matrix.md
24. content-manifests/gold-and-gears-v1/README.md
25. evidence/gold-and-gears-reference-v1/release/release-evidence.json
26. evidence/reference-integration-v1/merged-mode-audit.json

Before the first mutation:

- inspect the worktree, active persistent goal and the complete Goal 14 ledger;
- verify Goals 01–08 completion snapshots and current compatibility checks;
- verify the merged Goals 08–13 Candidate integration audit;
- verify the Goal 08 Candidate bundle and normalized-pack digests exactly match
  the Goal 14 prerequisite values;
- prove that Goal 08 reference/manifests/workbooks/generated/evidence roots are
  unchanged and protected;
- if another goal is active on overlapping files, use a separate branch and
  worktree or stop to reconcile the shared-file boundary.

Execution loop:

1. Select the earliest unblocked Pending batch and mark only it InProgress.
2. Implement the complete code, lowering, tests, evidence and documentation
   responsibility of that batch.
3. Update the ledger with exact commands, counts, hashes, decisions, policy
   outcomes and blockers. Never record a check as passed unless it ran.
4. Run focused format, Clippy and tests during implementation. Run
   `node tools/repository-check/run.mjs` before completing every ordinary
   change and `node tools/repository-check/run.mjs --full` at phase/release
   checkpoints or whenever the quick gate reports deferred inputs.
5. Commit exactly one completed batch with its exact ID, for example:
   feat(gold-gears): G14-P2-B2 compile bounded chessboard graphs
6. Immediately continue to the next unblocked batch. Do not stop because one
   table loads, one mechanic works, one seeded run completes, a phase ends or
   context is compacted. Re-read the ledger after continuation.
7. Mark the persistent goal complete only after G14-P8-B4 is committed, the
   completion snapshot is registered and the clean-worktree verifier passes.

Architecture constraints:

- Activity::apply and Battle::apply remain the only authoritative mutation
  boundaries. Do not introduce GoldAndGearsActivity::apply.
- Extend the mode-owned profile/components in starclock-mode-universe. Move a
  genuinely shared primitive to its lowest truthful owner only with focused
  generic tests and a versioned compatibility decision.
- Never add Gold and Gears IDs or mode IDs to shared Activity/combat resolver
  branches. Use typed operations, Rule IR and statically composed bounded
  handlers.
- Load the frozen config.sora through private generated readers and validated
  domain lowering. Runtime must never read Excel, normalized JSON or debug
  JSON, and public APIs must not expose generated rows.
- Keep the Goal 08 bundle as a separately identified consumed component. Do
  not merge physical artifact identity into a monolithic replay dependency or
  invalidate Standard/unrelated mode replays.
- Reuse released shared Path/Blessing/Resonance/content semantics by stable
  identity and digest. Gold mode copies own only their actual distinct state,
  parameters, pools and lifecycle.
- Represent Cognition, dice, Knowledge, Neural and Conundrum as typed bounded
  Activity state with explicit scopes, carry/reset, visibility and canonical
  encoding.
- Use project-owned labeled RNG streams and stable ordered candidates. Specify
  draw consumption for empty candidates; never use generic shuffle, floating
  probability or collection iteration order.
- Every nested battle consumes one immutable current-Activity snapshot and
  returns only a verified declared projection. Battle code cannot read or
  mutate live Activity state.
- Baseline AI, CLI, agent and MCP select only exact offered commands. Adventure
  accepts only an offered ExternalOutcome and does not simulate action physics.

Completeness constraints:

- P0-B2 must generate exact dispositions for all 7,913 source obligations,
  1,224 mechanic rules and 18 semantic fixture families and freeze all nine
  mechanic partitions before partition implementation begins.
- A loaded row, evaluator output or reference fixture is not runtime evidence.
  Every enabled mechanic must produce production state/events/hashes or have a
  truthful explicit non-executable disposition.
- Treat all 16 Goal 08 policy boundaries as unresolved for runtime until their
  Goal 14 owner records a versioned executable policy, proves metadata-only
  status, replaces it with stronger evidence or marks it blocking.
- Never silently relabel ProjectPolicy, approximation or inferred behavior as
  exact parity. No legal Released run may reach an unresolved fail-closed
  branch.
- Preserve rejected-command and rejected-result byte identity, independent RNG
  streams, stable graph/option order, checked fixed-point arithmetic and
  component-aware replay identity.
- Do not modify immutable Goal 08 artifacts to make runtime tests pass. A real
  authoring defect requires an explicit revisioned compatibility decision and
  complete openpyxl/Sora regeneration, never a direct workbook/generated edit.

Do not mark the goal complete until every required difficulty, Path, Custom
Dice, Conundrum boundary, fixture and policy family is covered by the
P0-frozen valid seeded matrix; all reachable encounters execute real battles;
all 7,913/1,224/18 dispositions close; all 16 policy boundaries are terminal;
replay and CLI/agent/MCP parity pass; cross-platform, performance, dependency,
architecture, prior-release and full clean-checkout gates pass; and
G14-P8-B4 is committed.
```
