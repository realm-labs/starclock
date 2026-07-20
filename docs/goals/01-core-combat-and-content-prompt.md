# Goal 01 Launch Prompt

Use the prompt below to start or resume persistent execution of
[Goal 01](01-core-combat-and-content.md). It is intentionally explicit: the
executor must continue batch-by-batch and commit-by-commit until the terminal
gates are met, rather than stopping after planning or after one phase.

```text
Create and start a persistent goal with this objective:

Implement Starclock Goal 01 completely: a deterministic, engine-agnostic Rust
core that compiles every released Version 4.4 character combat form with exact
abilities, Techniques, battle-relevant Traces, E1-E6 and released Light Cones
through S5, and can execute and replay complete Standard battle encounters.
Continue execution until every terminal gate in the Goal 01 plan and status
ledger is proven. Do not set a token budget.

Repository execution contract
=============================

1. Before changing code or data, read these files completely:

   - docs/goals/01-core-combat-and-content.md
   - docs/goals/01-core-combat-and-content-status.md
   - docs/README.md
   - docs/01-core-battle-model.md through docs/13-enemy-ai-and-encounters.md
   - docs/15-content-data-and-coverage.md
   - docs/16-replay-cli-and-engine-integration.md
   - the Standard profile and shared boundaries in
     docs/18-standard-and-challenge-modes.md
   - docs/19-activity-core-and-mode-extension.md
   - docs/20-core-implementation-design.md
   - docs/21-build-traces-and-equipment.md
   - docs/reference-data.md
   - docs/sources.md
   - docs/characters/README.md, schema.md, implementation-matrix.md and every
     character profile file
   - docs/content-reference/README.md
   - docs/content-reference/schema.md
   - docs/content-reference/authoring-contract.md
   - docs/content-reference/coverage.md
   - docs/content-reference/review-fixtures.md
   - content-reference/README.md
   - content-reference/v4.4/manifest.json
   - content-reference/v4.4/coverage.json
   - content-reference/v4.4/pack-index.json

   Document 14 and the challenge-specific sections of document 18 describe
   future extension boundaries only. Do not implement them in this goal.

2. Inspect the Git worktree, recent commits, active goal state and the full
   status ledger. Preserve user changes. Never use destructive Git operations.

3. Before selecting `G01-P0-B1`, regenerate or verify the prepared reference
   pack and require digest
   `0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a`.
   Treat the normalized records and authoring contract as the baseline for
   Excel/Sora transcription. Do not rebuild content facts from memory or an
   arbitrary single website.

4. Treat the goal plan and normative documents as implementation contracts.
   Do not replace their architecture with a shortcut, expose fixed-point/Sora
   implementation types, introduce engine dependencies, scatter character-ID
   branches through the resolver, create broad pub-use facades, or exceed the
   documented 1,200-LOC policy without a reviewed exception.

5. The prepared JSON reference pack is evidence and deterministic bootstrap
   input only. Production content must be authored in `.xlsx`, validated and
   exported by pinned Sora 0.3.0, loaded by generated readers and converted into
   domain definitions. Never introduce a JSON-direct runtime path as a milestone
   shortcut.

Persistent execution loop
=========================

Repeat this loop until the Goal 01 status ledger is Complete:

1. Reload the goal plan and status ledger. If conversation context was compacted
   or execution resumed later, reread every normative file affected by the next
   batch before acting.
2. Select the earliest unblocked Pending batch whose dependencies are Complete.
   Mark only that batch InProgress and announce its outcome and validation gate.
   `G01-P0-B1` must replace Phase 7 family placeholders with all concrete
   character and Light Cone partition rows immediately after freezing manifests.
3. Start from the prepared reference record. Research only a recorded conflict,
   approximation replacement or missing observation required for the batch.
   Record source URL, access date, version/confidence/note and evidence hash.
   Do not use leaks, proprietary assets, long copied descriptions or guessed
   values. If sources
   conflict, register a Researching case instead of selecting a convenient value.
   Researching must not stall the goal. If a bounded search cannot resolve a
   required behavior, record an explicit deterministic Approximate or
   ProjectPolicy decision with its evidence gap, selected behavior, rejected
   alternatives, rationale, affected tests, confidence and replacement
   condition. Resolve the implementation blocker as ResolvedProjectPolicy, then
   implement it and continue. Preserve the unresolved observation envelope as a
   future correction note; never present the decision as an observed game fact or
   silently copy behavior from a similar character or similarly named ability.
4. Implement the batch as one responsibility-bounded change. Include code,
   tests, schema/migration, generated output, provenance and documentation that
   belong to the batch. Do not add unrelated future-mode work.
   During Phase 4, compile the named real-character V1a probe owned by the batch
   from its dedicated Excel/Sora probe scope; never count partial probe rows as
   production `DataReady` content.
5. Run the batch-specific and universal gates from the goal plan. Fix failures
   in the same loop; never waive, hide or defer a gate merely to create a commit.
6. Regenerate coverage and update the status ledger with exact commands,
   evidence paths, manifest counts, decisions and blockers.
7. Review the diff for scope, public API, dependency direction, determinism,
   source-file size, generated drift and accidental user changes.
8. Commit exactly this completed batch. Its subject MUST use Conventional
   Commits in this exact structure:

   `<type>(<scope>): <batch-id> <concise imperative summary>`

   The type MUST be one of `build`, `chore`, `ci`, `data`, `docs`, `feat`,
   `fix`, `perf`, `refactor`, `revert`, `style` or `test`. The scope MUST be a
   lowercase kebab-case name for the batch's primary responsibility, chosen
   consistently with recent commits, such as `combat`, `rules`, `build`,
   `characters`, `replay` or `core`. `<batch-id>` MUST exactly match the batch
   being completed. Examples:

   `feat(combat): G01-P4-B5 implement effects DoT and resources`
   `feat(build): G01-P5-B2 implement ability and Trace compilation`
   `data(characters): G01-P7-C03 import character partition 03`

   A bare subject beginning with only the batch ID is invalid. After committing,
   inspect `git log -1 --format=%s` and verify the subject against
   `^(build|chore|ci|data|docs|feat|fix|perf|refactor|revert|style|test)\([a-z0-9][a-z0-9-]*\): G01-P[0-9]+-[A-Z][A-Z0-9-]* .+`.
   If it does not match, amend that newly created commit before proceeding; this
   does not authorize rewriting any earlier commit. Verify the working tree as
   well. Do not push, publish, open a PR or rewrite prior commits unless the user
   separately authorizes it.
9. Immediately start the next unblocked batch. Completing a commit or phase is
   progress, not a reason to return control or mark the persistent goal complete.

Execution rules
===============

- Keep one active implementation batch at a time. Independent read-only research
  may be parallelized only when it cannot cause overlapping writes or inconsistent
  manifest decisions.
- Use the machine-readable frozen manifests as the completeness oracle. IDs,
  compact profiles, schemas, mocks, TODOs, disabled rows and partial E0/S1 data
  do not count as implementation.
- Every Excel/Sora fact must map to the bound content-reference record and retain
  its evidence/approximation label. A stronger source may replace a prepared
  value only through a recorded decision and updated digest.
- Complete the Sora 0.3.0 capability proof before production schema families.
  Use only command names, type syntax, reference behavior, codegen and export
  formats proven by the committed golden project.
- Character content batches are complete through Technique, all battle-relevant
  Traces and E1-E6. Light Cone batches are complete through S5. Land a missing
  generic mechanic batch before the content batch that depends on it.
- Use the Phase 4 V1a probes as mandatory design feedback: Asta constrains
  modifier/buff stacks, Kafka DoT, Clara counters, Firefly HP/transform/Super
  Break, Aglaea memosprites, and at least two released forms constrain shared
  Elation semantics. `G01-P7-V1B` later replaces them with complete content.
- Native handlers are allowed only through the deterministic static registry,
  with an explicit reason the typed rule IR is insufficient and focused tests.
- Authoritative simulation never uses f32/f64. Apply checked fixed-point
  arithmetic, explicit rounding, stable ordering and canonical encoding exactly
  as specified.
- Invalid commands must not mutate authoritative state or consume RNG. Internal
  failures follow the documented rollback/Faulted policy.
- Keep starclock-combat independent from build, data, activity, mode, CLI and
  engines. Builds compile into generic combat-domain input; peripheral ownership
  and equipment concepts never enter battle state.
- Implement only the minimum generic Activity/Standard integration required by
  the plan. Protect the generic scope, typed-slot, participant/build-lock,
  BattleSpec and declared-result-projection seams with architecture tests, but
  do not create empty future-mode semantics. Do not implement Simulated
  Universe, Memory of Chaos, Pure Fiction, Apocalyptic Shadow, UI/Bevy, account
  systems, the full relic/planar dataset or the complete public enemy catalog.
- Establish the CI matrix, property-test harness and benchmark baseline in their
  early batches. Hardening extends accumulated evidence; it must not introduce
  these prerequisites for the first time. Strict performance budgets belong to
  the designated stable runner, while shared CI uses only the documented broad
  smoke ceiling.
- For the baseline transaction, validate legality before preparing reusable
  battle-local scratch, use `clone_from`/state swapping with bounded retained
  capacity, and stream canonical encoding directly into SHA-256. Do not replace
  these semantics with COW, inverse patches or incremental hashes without a
  recorded benchmark and revision/compatibility decision.
- Benchmark both supported verifier profiles: stateful incremental apply and a
  one-shot replay that compares every command hash. Never implement live
  verification by replaying growing prefixes from the beginning, which is
  quadratic over a battle. Share only immutable catalogs across isolated jobs.
- Do not weaken an acceptance gate to fit the current implementation. If a
  normative contradiction is found, add a narrow decision record, update every
  affected document and add a regression fixture in the same atomic batch.
- Prefer safe, informed assumptions inside an existing contract. Ask the user
  only when a choice would materially expand scope, change a public architecture
  boundary, require unavailable authoritative evidence, or perform an external
  action not already authorized.
- When a batch is externally blocked, record the exact blocker and continue any
  independent unblocked batch. Stop only when no meaningful in-scope work can
  progress. Do not label the whole goal Blocked before the persistent-goal
  blocked threshold is genuinely met.
- Provide concise progress updates at least once per minute during long work.
  A final response must be self-contained, but do not use an ordinary final
  response to terminate the loop while required unblocked work remains.

Completion protocol
===================

Do not mark the goal Complete because the workspace compiles, a vertical slice
runs, a phase ends, context is short, or only the content identities exist.

When all batches appear complete:

1. Run the full clean-checkout acceptance suite in Goal 01 section 8.
2. Verify 100% DataReady coverage for the frozen released-character, released
   Light Cone and standard-v1 manifests, with no required Researching or Blocked
   row.
3. Verify cross-platform numeric, RNG, codec, build, battle and replay evidence,
   clearly separating native execution from compile-only CPU targets.
4. Verify the final representative performance report against the reviewed
   stable-runner budgets, including incremental apply, 100/500-command one-shot
   replay, concurrent isolated jobs and peak bytes/job; preserve the benchmark
   inputs and runner identity.
5. Update every terminal checklist item and the completion record in the status
   ledger with committed evidence.
6. Commit the completion record as G01-P8-B7.
7. Mark the persistent goal Complete only after that commit succeeds and the
   final worktree is clean.
8. Report the completion commit, catalog digest, coverage totals, validation
   commands and cross-platform evidence. If the goal system reports final token
   usage, include it.

Start now with the Next unblocked batch recorded in the status ledger. Do not
respond with another plan; execute the loop.
```
