# Goal 12 Launch Prompt

Run this goal in a separate task, branch and git worktree while Goal 07, 08, 09,
10 or 11 is active in another checkout.

```text
Create or resume the persistent goal whose objective is to complete Starclock
Goal 12: prepare the complete Version 4.4 Currency Wars reference pack,
isolated Excel/Sora authoring data, provenance, coverage and semantic review
fixtures before runtime implementation. Continue batch-by-batch until every
terminal gate is proved. Do not set a token budget.

Read completely before acting:

1. docs/goals/12-currency-wars-reference-data.md
2. docs/goals/12-currency-wars-reference-data-status.md
3. docs/goals/03-standard-universe-reference-data-status.md
4. docs/goals/08-gold-and-gears-reference-data.md
5. docs/goals/08-gold-and-gears-reference-data-status.md
6. docs/goals/09-swarm-disaster-reference-data.md
7. docs/goals/09-swarm-disaster-reference-data-status.md
8. docs/goals/10-unknowable-domain-reference-data.md
9. docs/goals/10-unknowable-domain-reference-data-status.md
10. docs/goals/11-divergent-universe-reference-data.md
11. docs/goals/11-divergent-universe-reference-data-status.md
12. docs/14-run-core-and-universe-modes.md
13. docs/15-content-data-and-coverage.md
14. docs/19-activity-core-and-mode-extension.md
15. docs/23-standard-simulated-universe-reference.md
16. docs/24-standard-universe-normalized-data.md
17. docs/26-mode-extension-and-evolution.md
18. docs/07-configuration-pipeline.md
19. docs/08-engineering-standards.md
20. docs/09-determinism-and-numerics.md
21. docs/11-rule-ir-and-native-handlers.md
22. docs/content-reference/README.md
23. docs/content-reference/schema.md
24. docs/content-reference/authoring-contract.md
25. docs/sources.md
26. content-reference/README.md
27. content-manifests/standard-universe-v1/README.md
28. tools/content-reference/README.md

Before the first mutation:

- verify Goal 03 is Complete;
- reproduce the ignored turnbasedgamedata cache at
  fd978d6ef09f941fba644c731ab54abd6f7c3568 and StarRailRes cache at
  7b349e39ee0f6f3bf814567995829b99c95e7a93, then record checkout cleanliness,
  configured origins, commit readability, connectivity and required-file
  hashes;
- inspect the current worktree and every other active worktree/branch;
- inspect the latest committed Goal 08/09/10/11 ownership manifests when
  available and record the exact revisions used for reconciliation;
- prove that this task uses a separate branch/worktree and the six isolated
  Goal 12 artifact roots;
- verify the configured writable remote is origin and the branch is
  codex/goal12-currency-wars-reference, or record an explicitly approved
  replacement before beginning G12-P0-B1;
- resolve the Goal package setup commit from the Goal 12 ledger's containing
  commit and verify the remote branch reaches it;
- do not start if the checkout would share mutable workbooks or generated
  outputs with an active Goal 07, 08, 09, 10 or 11 task.

Execution loop:

The exact batch set, in execution order, is:

`G12-P0-B1`, `G12-P0-B2`, `G12-P0-B3`, `G12-P0-B4`, `G12-P0-B5`,
`G12-P1-B1`, `G12-P1-B2`, `G12-P1-B10`, `G12-P1-B3`, `G12-P1-B4`,
`G12-P1-B5`, `G12-P1-B6`, `G12-P1-B7`, `G12-P1-B8`, `G12-P1-B9`,
`G12-P2-B1`, `G12-P2-B2`, `G12-P2-B3`, `G12-P2-B4`, `G12-P2-B5`,
`G12-P2-B6`, `G12-P3-B1`, `G12-P3-B2`, `G12-P3-B3`, `G12-P3-B4`,
`G12-P3-B5`, `G12-P3-B6`, `G12-P4-B1`, `G12-P4-B2`, `G12-P4-B3` and
`G12-P4-B4`.

1. Select the earliest unblocked Pending batch and mark only it InProgress.
2. Implement its complete source inventory, normalized data, evidence, schema,
   workbook, verification and documentation responsibility.
3. Use the pinned released source first. Record exact repository revisions,
   paths, row locators, URLs, access dates, hashes, confidence and notes.
4. Use the `G12-P0-B5` correction: start source discovery from
   `GuideRogueTab#2` / `GuideRogueData#5`, where Currency Wars is explicitly
   `GuideType = GridFight`; retain all 153 `GridFight` tables, all 984
   GridFight configuration paths, CHS/EN TextMaps, StageConfig and every
   transitively referenced shared build, enemy, wave, battle-event, level and
   ability file. Treat Tourn3/Persona/S3 as superseded Divergent Universe
   evidence unless a GridFight-originating reference proves a shared row.
5. Freeze completeness from generated manifests, never estimated Wiki totals
   or raw table sizes. A GridFight/Tourn3 label, table prefix, version suffix,
   ID range, table name or matching display name is not row membership proof.
6. Classify every record as CurrencyWars, Shared, EvidenceOnly or excluded
   named-mode/module ownership. Reachability requires an explicit enabled
   Version 4.4 selector, transitive source reference or inherited stable-ID
   closure.
7. A Blessing, Curio, event or service family may close at zero only when the
   generated manifest proves the complete enabled selector/reference closure
   contains no reachable row. Do not turn an unresolved join into a zero count.
8. Reconcile overlapping facts only by source path, stable row locator and
   evidence digest. Do not copy or edit Goal 08/09/10/11 normalized rows to
   force agreement; record conflicts for the named merge checkpoint.
9. Preserve hidden shop/pool weights, candidate/target ordering, star-combine
   timing, simultaneous Bond changes, Squad-HP/action-value ordering, rounding,
   caps and fallbacks as bounded research cases. If exact evidence remains
   unavailable after a bounded search, record an explicit
   ApproximateFromReleasedText or ProjectPolicy rule with alternatives,
   affected fixtures and a concrete replacement condition.
10. Use Python openpyxl for workbook creation and inspection. Regenerate all
    three complete isolated workbooks into clean targets; never patch a
    designer-edited workbook. Sora 0.3.0 remains schema/codegen/export
    authority.
11. Keep JSON as research/bootstrap/debug output only. Do not add runtime JSON
    or Excel loading.
12. Run focused batch gates and applicable repository/prior-release checks. At
    phase boundaries run deterministic regeneration and isolated Sora reader
    checks. At release run the full clean-checkout gate.
13. Update the ledger with exact commands, counts, digests, decisions,
    research outcomes, replacement conditions, reconciliation receipts and
    publication evidence.
14. Commit exactly one completed batch using the exact batch ID, for example:
    data(currency-wars): G12-P1-B5 import bond definitions
15. Push the commit immediately to the recorded remote branch, verify the
    remote branch resolves to the same full commit ID, and record the remote,
    branch, push/verification commands and result. Do not mark the batch
    Complete or begin another batch while the commit exists only locally.
16. Continue immediately to the next unblocked batch. Do not stop at a source
    inventory, partial normalized pack, workbook export or phase boundary.

Mandatory parallel boundary:

- do not edit Goal 07, 08, 09, 10 or 11 plans, ledgers, manifests, partitions,
  policies, workbooks or evidence;
- do not edit another mode's normalized rows or production workbook rows;
- do not regenerate another mode's generated directory or config/generated;
- do not implement Currency Wars runtime lowering, handlers, controllers, CLI,
  Agent or MCP surfaces;
- do not modify starclock-build, shared Activity/combat semantics or production
  configuration to make a reference row fit;
- when a missing shared runtime capability is discovered, record a later-goal
  prerequisite and continue independent reference work;
- preserve Goal 03 and every current mode/production bundle identity;
- stop and record a reconciliation blocker rather than overwriting concurrent
  work or accepting incompatible ownership/semantic classifications.

Scope boundary:

- include released Version 4.4 Standard and Overclock Gambit; three-Plane Node
  flow; difficulties/ranks and enemy affixes; Squad HP and action-value battle
  limits; roster/shop/Gold Coin/Experience/team-size economy; positions,
  Character Empowerments and Bonds; character costs and star combinations;
  owned/trial build mapping, off-field conversions and equipment; GridFight
  Augments, Projections, Portals, Talents and selector-reachable investment mechanics; reachable
  Blessings/Curios/events/services; and exact encounters, StageConfig waves,
  enemies and bosses;
- exclude story prose, presentation, assets, audio, UI, account/collection/
  rank/weekly/Bond-chain rewards, social/share-code surfaces and every other
  mode;
- retain excluded rows only as bounded provenance needed to prove ownership,
  an enabled selector, unlock, stage boundary or mechanical prerequisite.

Do not mark the goal complete until the corrective G12-P0-B5 manifest has frozen exact enabled-selector
denominators, every required row is DataReady, ownership and references close,
shared facts reconcile with the inspected committed Goal 08/09/10/11
revisions, all distinct mechanic families have semantic fixtures, isolated
Excel/Sora artifacts regenerate and render without drift, generated readers
load every row, no Goal 12 content enters another mode or production bundle,
the clean-checkout release gate passes, G12-P4-B4 is committed and pushed, and
every completed batch commit is reachable from the recorded remote branch at
the recorded commit ID.
```
