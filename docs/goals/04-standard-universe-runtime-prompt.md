# Goal 04 Launch Prompt

```text
Create or resume the persistent goal whose objective is to complete Starclock
Goal 04: implement the complete deterministic Version 4.4 main-world Standard
Simulated Universe runtime over starclock-activity and starclock-combat, using
the frozen Goal 03 Excel/Sora data, with full mechanics, seeded headless runs,
replay, baseline AI, CLI, agent API and MCP control. Continue batch-by-batch
until every terminal gate is proved. Do not set a token budget.

Read these files completely before changing code:

- docs/goals/04-standard-universe-runtime.md
- docs/goals/04-standard-universe-runtime-status.md
- docs/25-standard-universe-runtime-design.md
- docs/19-activity-core-and-mode-extension.md
- docs/14-run-core-and-universe-modes.md
- docs/23-standard-simulated-universe-reference.md
- docs/24-standard-universe-normalized-data.md
- docs/06-rust-architecture.md
- docs/07-configuration-pipeline.md
- docs/08-engineering-standards.md
- docs/09-determinism-and-numerics.md
- docs/10-lifecycle-and-resolution.md
- docs/11-rule-ir-and-native-handlers.md
- docs/12-modifier-and-snapshot-pipeline.md
- docs/13-enemy-ai-and-encounters.md
- docs/16-replay-cli-and-engine-integration.md
- docs/20-core-implementation-design.md
- docs/21-build-traces-and-equipment.md
- docs/22-agent-control-and-mcp.md
- docs/activity-one-battle-boundary.md
- docs/activity-replay-and-controller-diagnostics.md
- docs/starclock-agent-integration-contract.md
- docs/dependency-and-tool-policy.md
- docs/ci-platform-matrix.md
- docs/goals/01-core-combat-and-content-status.md
- docs/goals/02-agent-control-and-mcp-status.md
- docs/goals/03-standard-universe-reference-data-status.md
- evidence/standard-universe-reference-v1/release/release-evidence.json

Execution loop:

1. Inspect the worktree, current persistent goal, Goal 04 ledger and all prior
   release contracts. Preserve unrelated user changes.
2. Select only the earliest unblocked Pending batch, mark it InProgress, and
   keep its responsibility and exit gate explicit.
3. Implement domain code, lowering, evidence, tests and documentation required
   by that batch. Update the ledger with exact counts, hashes, commands,
   decisions and remaining research cases.
4. Run focused tests during implementation. Before completing the batch, run
   formatting, linting, source/visibility policy, generated drift relevant to
   the touched artifacts and prior release compatibility. Run full repository
   gates at every phase exit.
5. Commit exactly one completed batch with its exact ID, for example:
   feat(activity): G04-P2-B1 implement bounded activity graphs
6. Immediately continue with the next unblocked batch. Do not stop merely
   because a crate compiles, one World runs, a phase ends, or the context was
   compacted. Re-read the ledger after every continuation.
7. Mark the persistent goal complete only after G04-P6-B4 is committed and the
   release verifier passes from a clean worktree. Report the final goal usage.

Architectural constraints:

- Do not implement coordinates, movement, collision, patrol, aggro radius or a
  3D scene. Compile each domain to a bounded Activity micrograph with stable
  encounter/service/choice/reward/interactable handles and exit gates.
- Activity::apply remains the only run mutation boundary and Battle::apply the
  only battle mutation boundary. Never add UniverseActivity::apply.
- Use private Sora-generated readers and validated lowering. Runtime must not
  read normalized JSON or .xlsx, and public APIs must not expose generated row
  types or the numeric backend.
- If a retained data defect requires workbook regeneration, use the pinned
  Python openpyxl adapter and validate/export through Sora 0.3.0. Never patch
  workbook cells manually or introduce another Excel authoring path.
- Preserve the independently identified Goal 03 universe bundle and compose its
  digest with combat/build catalog identities. Never silently rewrite prior
  release evidence or fabricate missing combat rows.
- Use typed Activity operations and battle Rule IR before native handlers.
  Native handlers are static, audited and return ordinary operations. Never
  scatter universe content-ID branches through activity or combat resolvers.
- Invalid commands and rejected BattleResults must preserve exact state hash
  and RNG counters. Use independent labeled graph, encounter, reward, shop,
  occurrence and battle streams with stable candidate ordering.
- Immutable catalogs are shared; commands may not clone complete catalogs or
  rebuild replay prefixes. Keep incremental service verification O(new work)
  and one-shot replay O(total commands).
- External AI, CLI and MCP select only exact offered commands. ExternalOutcome
  accepts only an offered result ID and does not simulate minigame physics.
- Keep Swarm Disaster, Gold and Gears, Unknowable Domain, Divergent Universe,
  account rewards, story/assets and network save synchronization out of scope.

Completeness is mechanical, not nominal. A loaded Sora row or a stable ID is
not implemented. P0-B3 must freeze all 2,201 content and 786 rule dispositions
and exact P4 partition membership. Every assigned row must become executable,
intentionally metadata-only, or explicitly policy-bound with evidence. All 78
fixtures, nine Worlds, nine Paths, 33 difficulties, complete-run goldens,
cross-platform hashes, performance workloads, Goals 01–03 contracts and the
clean release gate must pass before completion.
```
