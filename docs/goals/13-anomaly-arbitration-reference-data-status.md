# Goal 13 Status — Anomaly Arbitration Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `anomaly-arbitration-reference-v1` |
| State | `InProgress` |
| Active phase | Phase 1 — Unique mode systems |
| Active batch | `G13-P1-B2` complete pending this commit's publication |
| Next unblocked batch | `G13-P1-B3` after remote verification |
| Snapshot | Version 4.4 / inherited structured-source access 2026-07-22 |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Identity cross-check | `StarRailRes@7b349e39ee0f6f3bf814567995829b99c95e7a93` |
| Planning cache audit | 2026-07-29: both caches clean/detached at pinned commits; origins, commit/blob readability and connectivity verified; execution must reproduce in `G13-P0-B1` |
| Source-cache reproduction | `G13-P0-B1`: both fixed caches materialized twice in an isolated `/tmp` target from the clean connected fixed-commit cache; exact detached HEAD, tree, origin, cleanliness, connectivity and 21 required file receipts verified |
| Starting source oracle | 6 `ChallengePeak*` tables, direct ChallengePeak battle-event ability, indirect gameplay/config closure, CHS/EN TextMaps, StageConfig and transitive shared target/MazeBuff/enemy sources |
| Active-period hypothesis | Group `8`, aliases `801`–`804` and stages `30508011`, `30508012`, `30508013`, `30508021`, `30508022`; not a denominator until `G13-P0-B3` proves the selector chain |
| Focused inventory | 2,745 files: 2,646 inherited Goal 03 files plus 90 focused turnbasedgamedata additions and 9 StarRailRes indexes; SHA-256 `86ec49eef98c3912fa886abd6706eee691106c9b644639d6d55bf4d23a4d2583` |
| Content manifest | 392 active obligations across 28 categories: 76 `AnomalyArbitration`, 316 `Shared`; 106 historical/account/presentation exclusions and 6 generated exact-zero pools; SHA-256 `84d60c5edea6f3210bc706cf21d18d886292cdbff1f2cc4f310b3e18c3146ac4` |
| Content lane | `Experimental`; target reference bundle `Candidate` |
| Workbook adapter | Python `openpyxl`; Sora 0.3.0 remains authoritative |
| Remote | `origin` |
| Branch | `codex/goal13-anomaly-arbitration-reference` |
| Branch base | `b0cd3cb912c9f2ec887c3ae29f79353c4a861643` (`G11-SETUP`; excludes every concurrent worktree's later or uncommitted data) |
| Parallel condition | Separate branch/worktree and six isolated Goal 13 roots while Goals 07 through 12 are active |
| Publication policy | Push and remotely verify each completed batch before starting the next batch |
| Blocking condition | None |

## Phase ledger

| Phase | State | Evidence |
|---|---|---|
| Phase 0 — Scope, sources and contracts | `Complete` | Foundation, 2,745-file inventory, 392-record active manifest, 37-file normalized schema, three-workbook authoring surface, row-level reconciliation and 18-family fixture contract are frozen. |
| Phase 1 — Unique mode systems | `InProgress` | Profile, released eligibility, active period, five StageConfig encounters, legal-order boundary and four terminal outcomes imported; awaiting Knight records/uniqueness, King protection/Plight, clocks, Arbitral Quadrant, stars and settlement. |
| Phase 2 — Content and encounters | `Pending` | Awaiting pool zero/nonzero proofs, targets, traits, battle events, stages, waves, enemies and bosses. |
| Phase 3 — Sora and Excel | `Pending` | Awaiting isolated schemas/readers, three complete workbooks, deterministic exports and visual QA. |
| Phase 4 — Review and freeze | `Pending` | Awaiting ownership audit, semantic fixtures, reconciliation, regeneration and clean-checkout evidence. |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G13-P0-B1` | `Complete` | This row's containing commit | Froze foundation `c85e2cd8…e93d`, Goal 03 commit/tree and preserved bundle digests, two Version 4.4 source revisions/trees, 6 dedicated tables, 18 turnbasedgamedata plus 3 StarRailRes file receipts, 25 batches, Candidate-only scope, Excel/openpyxl/pinned Sora 0.3.0 authority and 6 isolated roots. Checkpoints: Goal 07 remote-merged `4f466df7…0931`; Goal 08 local-only `43b989c4…8d90` (7,913: 7,199/714); Goal 09 remote-backed `9bd2ad28…09c` (6,963: 6,305/658); Goal 10 remote-backed `ce2f6b20…5683` (5,377: 5,243/134); Goal 11 remote-backed inventory `f202c1bd…1779` with no ownership manifest; Goal 12 remote-backed foundation `74cb56a2…159d` with no source/ownership manifest. A bounded fresh GitLab partial clone made no progress and was terminated; no network success is claimed. The isolated copy-on-write reproducer and verifier passed twice/idempotently. Publication contract: `remote=origin`; `branch=codex/goal13-anomaly-arbitration-reference`; push `git push origin HEAD:refs/heads/codex/goal13-anomaly-arbitration-reference`; verify `git rev-parse HEAD` against `git ls-remote --exit-code origin refs/heads/codex/goal13-anomaly-arbitration-reference`, requiring identical full commit IDs before P0-B2. |
| `G13-P0-B2` | `Complete` | `35e10e0ee2bba25c80f37dabf97395333444b393` | Froze 2,745 uniquely sorted file receipts at `86ec49ee…2583`: all 2,646 Goal 03 files, 90 turnbasedgamedata additions and 9 StarRailRes indexes. Closure includes 6 `ChallengePeak` tables, 8 shared table seeds, 5 planning Stage rows, 12 direct and 27 recursively reachable enemy IDs, 26 templates, 74 mechanical config/ability/AI files, 2 reward exclusion locators and 11 bilingual text/index files. Checked-out bytes reproduce their pinned Git blob OIDs; a clean fixed-revision alternate may supply objects missing from a partial clone without lazy fetch or cache mutation. Double generation was byte-identical. Publication: `remote=origin`; `branch=codex/goal13-anomaly-arbitration-reference`; push `git push origin HEAD:refs/heads/codex/goal13-anomaly-arbitration-reference` succeeded; `git rev-parse HEAD`, tracking ref and `git ls-remote --exit-code origin refs/heads/codex/goal13-anomaly-arbitration-reference` all resolved to `35e10e0ee2bba25c80f37dabf97395333444b393`. |
| `G13-P0-B3` | `Complete` | `a6d9ef6c79d966ae312fc36f214932ed044f123a` | Released observations identify Version 4.4 rotation 8 and “尘世卷中” / “Enwreathed by the World”; pinned group row 8 then closes explicitly to aliases 801–804 and five released stages. Froze 392 active obligations across 28 categories (76 `AnomalyArbitration`, 316 `Shared`), including 7 targets, 8 traits, 3 Quadrant options, 3 battle events, 27 enemy variants, 26 templates, 115 skills, 52 statuses and 73 mechanical programs. Separately receipted 77 historical, 13 account-reward, 15 excluded-constant and 1 presentation row/file; six gameplay pool families have generated exact-zero selector proofs. Publication: `remote=origin`; `branch=codex/goal13-anomaly-arbitration-reference`; push `git push origin HEAD:refs/heads/codex/goal13-anomaly-arbitration-reference` succeeded; `git rev-parse HEAD`, tracking ref and `git ls-remote --exit-code origin refs/heads/codex/goal13-anomaly-arbitration-reference` all resolved to `a6d9ef6c79d966ae312fc36f214932ed044f123a`. |
| `G13-P0-B4` | `Complete` | `15c9d067ed47e95a0401978e79e267af66b814d8` | Froze 37 normalized file contracts with bilingual independent summaries, exact ownership/coverage/evidence/mechanism fields, manifest links, ordered source refs, canonical decimals/encoding, field-level approximation and shared reconciliation receipts. Froze exactly three complete no-overwrite openpyxl workbooks (`AnomalyArbitration.xlsx`, `AnomalyArbitrationBindings.xlsx`, `AnomalyArbitrationReview.xlsx`) under isolated Sora 0.3.0 paths and 18 semantic fixture families with explicit battle/cross-battle boundaries. Contract digests: schema `db436a34…a12a`, authoring `66a95988…b850`, fixture `5d523aa8…a5f8`. Publication: `remote=origin`; `branch=codex/goal13-anomaly-arbitration-reference`; push `git push origin HEAD:refs/heads/codex/goal13-anomaly-arbitration-reference`; local, tracking and `git ls-remote --exit-code origin refs/heads/codex/goal13-anomaly-arbitration-reference` all resolved to `15c9d067ed47e95a0401978e79e267af66b814d8`. |
| `G13-P1-B1` | `Complete` | `e8e4eee05195ba3f2429c8f91d87f8b70e8a0d8f` | Imported one profile, one active period, five released stages and four terminal outcomes with 20 exact-once manifest obligations. Eligibility and arbitrary Knight order use the released official guide; entry locators, aliases, StageConfig and bilingual TextMaps use pinned structured rows. The two public end dates are retained without an invented instant, and the normal-King unlock is a field-level released-text approximation with alternatives, fixture owner and replacement condition. Output digests: profiles `fbe85022…e1c2`, periods `c1c7e3e…bacb`, stages `c6ce591a…74d2`, terminal outcomes `171dfa56…193`. Publication: `remote=origin`; `branch=codex/goal13-anomaly-arbitration-reference`; `git push origin HEAD:refs/heads/codex/goal13-anomaly-arbitration-reference` succeeded; local, tracking and `git ls-remote --exit-code origin refs/heads/codex/goal13-anomaly-arbitration-reference` all resolved to `e8e4eee05195ba3f2429c8f91d87f8b70e8a0d8f`. |
| `G13-P1-B2` | `Complete` | This row's containing commit | Imported four participant/uniqueness policies, three Knight team slots, three loadout-record policies and six ordered progress policies. Released official instructions prove successful-clear snapshots, retry eligibility, cross-team equipment invalidation, explicit keep/replace choice, current-record erasure and Best Battle Record retention. All ten P1-B2 manifest obligations are DataReady; multi-Path combat-form identity remains one explicit `ProjectPolicy` field with alternatives and a released-evidence replacement condition. Output digests: participant policies `75df925c…c183`, team slots `e1aa2361…f45a`, loadout records `8dc6e186…18f5`, progress records `ed89f8eb…1792`. Publication contract: `remote=origin`; `branch=codex/goal13-anomaly-arbitration-reference`; push `git push origin HEAD:refs/heads/codex/goal13-anomaly-arbitration-reference`; verify local, tracking and `git ls-remote --exit-code origin refs/heads/codex/goal13-anomaly-arbitration-reference` resolve to the same full commit before P1-B3. |
| `G13-P1-B3` | `Pending` | — | Import King protection, normal/Plight states, shortcut and transition order. |
| `G13-P1-B4` | `Pending` | — | Import clocks, first-cycle window, wave carry, warnings, low-cycle effects and failure/retry. |
| `G13-P1-B5` | `Pending` | — | Import active Arbitral Quadrant offers, selection, parameters and contributions. |
| `G13-P1-B6` | `Pending` | — | Import targets, stars, evaluation, aggregation, settlement and cross-battle projection. |
| `G13-P2-B1` | `Pending` | — | Audit/freeze reachable or exact-zero Blessing, Curio, Occurrence, service, currency and related pools. |
| `G13-P2-B2` | `Pending` | — | Import stage definitions, shared battle targets, objectives and challenge events. |
| `G13-P2-B3` | `Pending` | — | Import enemy traits, King/Plight modifiers, Quadrant MazeBuffs and transitive ability contributions. |
| `G13-P2-B4` | `Pending` | — | Import StageConfig encounters, waves, enemy slots, variants, skills/AI/abilities and bosses. |
| `G13-P2-B5` | `Pending` | — | Generate mechanics, sources, coverage, research gaps, fixtures and pack index. |
| `G13-P3-B1` | `Pending` | — | Add profile/period/stage/participant/record/progress Sora tables. |
| `G13-P3-B2` | `Pending` | — | Add King/Plight, clock, target, objective, aggregation and Quadrant tables. |
| `G13-P3-B3` | `Pending` | — | Add pool, trait, event, encounter, enemy and contribution binding tables. |
| `G13-P3-B4` | `Pending` | — | Add evidence/coverage/reconciliation/fixture tables and isolated locks/templates/readers. |
| `G13-P3-B5` | `Pending` | — | Generate and structurally/semantically verify all three complete `openpyxl` workbooks. |
| `G13-P3-B6` | `Pending` | — | Prove deterministic Sora export/load and visual review of every sheet. |
| `G13-P4-B1` | `Pending` | — | Audit exact-once coverage, period selection, ownership, references, provenance and bilingual fields. |
| `G13-P4-B2` | `Pending` | — | Execute all semantic fixtures and approximation replacement checks. |
| `G13-P4-B3` | `Pending` | — | Reconcile overlap and run full regeneration, drift, reader, dependency and clean-checkout acceptance. |
| `G13-P4-B4` | `Pending` | — | Freeze final documentation, evidence and Candidate reference-bundle identity. |

For a completed batch, the result/evidence cell must record `remote`,
`branch`, full pushed commit ID, exact push command, remote-resolution
verification command and result. A locally committed but remotely unverified
batch remains `InProgress`.

The Goal package setup commit is identified as “this document's containing
commit” to avoid a recursive self-hash. Its remote, branch, exact push command
and remote-resolution result are reported in the setup handoff.
`G13-P0-B1` records the full setup commit and remote verification as immutable
foundation evidence before any data mutation.

## Current batch verification

| Command | Result |
|---|---|
| `tools/anomaly-arbitration-reference/fetch-sources.sh /tmp/starclock-g13-fetch-test.cffiAe /Users/mikai/CLionProjects/starclock/.cache/content-reference` (twice) | Passed; first isolated materialization and second idempotent run both left exact clean/detached revisions with canonical origins and connected Git objects. |
| `node tools/anomaly-arbitration-reference/verify-foundation.mjs --source-cache /tmp/starclock-g13-fetch-test.cffiAe` (twice) | Passed; Goal 03, 21 source receipts, Goal 07–12 committed checkpoints, branch/worktree isolation and 25-batch contract verified. |
| `git diff --check`; plan/status batch-set script; repository-wide local Markdown target script | Passed; 25 batch IDs agree and 634 local links across 301 tracked Markdown files resolve. |
| `node tools/repository-check/run.mjs` under Node 24.15.0 | Passed in 3.0 seconds; no Rust scope selected and two generated/release inputs were deferred. |
| `node tools/repository-check/run.mjs --full --with-source-cache` | Supplemental full gate did not reach Goal 13 or generated-drift checks: existing `tools/goal06/verify-phase0.mjs` rejected the current additive repository `Cargo.lock` as differing from its historical baseline. Goal 13 changes neither file. |
| `node tools/repository-check/verify-generated-drift.mjs --with-source-cache` | Supplemental generated gate passed eight existing pack/manifest/provenance/coverage checks, then stopped because repository-pinned Sora 0.3.0 is not installed; host `sora 0.2.0` is explicitly non-authoritative and Phase 3 owns resolution. |
| `node tools/goal-provenance/generate.mjs --check`; Standard Universe inventory/manifest/bootstrap checks | Direct cache substitutes exposed existing checkout-EOL/current-tree drift: Black Swan source hash mismatch, Standard inventory drift, manifest pass, and `occurrence-choices.json` drift. These historical/shared artifacts are protected; Goal 13 instead verifies exact Git commits/trees and its 21 required blobs. |
| `node tools/anomaly-arbitration-reference/inventory.mjs --source-cache .cache/content-reference --fallback-source-cache /Users/mikai/.codex/worktrees/7c74/starclock/.cache/content-reference` (twice) | Passed; both clean caches resolve to the pinned revisions, lazy fetch remained disabled, every read byte stream matched its pinned Git blob OID and both outputs had SHA-256 `86ec49eef98c3912fa886abd6706eee691106c9b644639d6d55bf4d23a4d2583`. |
| `node tools/anomaly-arbitration-reference/verify-inventory.mjs --source-cache .cache/content-reference --fallback-source-cache /Users/mikai/.codex/worktrees/7c74/starclock/.cache/content-reference` | Passed; exact 2,745-file denominator, Goal 03 receipts, focused family counts, required seed paths, exclusion locators and planning-only selector state verified. |
| P0-B2 `git diff --check`; plan/status batch-set script; repository-wide local Markdown target script; `node tools/repository-check/run.mjs` | Passed; 25 batch IDs agree, 634 local links across 303 Markdown files resolve, no Rust scope was selected and two generated/release inputs remain deferred to the documented full boundary. |
| `node tools/anomaly-arbitration-reference/manifest.mjs --source-cache .cache/content-reference --fallback-source-cache /Users/mikai/.codex/worktrees/7c74/starclock/.cache/content-reference` (twice) | Passed byte-identically; both source caches remained clean/fixed and the manifest SHA-256 was `84d60c5edea6f3210bc706cf21d18d886292cdbff1f2cc4f310b3e18c3146ac4`. |
| `node tools/anomaly-arbitration-reference/verify-manifest.mjs --source-cache .cache/content-reference --fallback-source-cache /Users/mikai/.codex/worktrees/7c74/starclock/.cache/content-reference` | Passed; active group/title chain, 28 category counts, exact-once ownership, 106 exclusions, shared reconciliation fields, 6 exact-zero proofs and counter groups verified. |
| P0-B3 whitespace checks; plan/status batch-set script; repository-wide local Markdown target script; `node tools/repository-check/run.mjs` | Passed; 25 batch IDs agree, 634 local links across 304 Markdown files resolve, no Rust scope was selected and three generated/release inputs remain deferred to the documented full boundary. |
| `node tools/anomaly-arbitration-reference/verify-contracts.mjs` | Passed; 37 normalized files cover all 28 manifest categories, partition exactly once across 3 workbooks, and match all 18 frozen fixture families. Schema `db436a34b66bacc6811bf0b6101d58317691faccd106cf29024f2271cca0a12a`; authoring `66a959881e4877d172b57ea1ff1196a63f878c72ee34991345a51b628bc5b850`; fixtures `5d523aa85c7716cabaea6ce8d95db1f1a4558b4363adda1e44c99246da58a5f8`. |
| P0-B4 `git diff --check`; plan/status batch-set script; repository-wide local Markdown target script; `node tools/repository-check/run.mjs` | Passed; 25 batch IDs agree, 634 local links across 305 Markdown files resolve, no Rust scope was selected and five generated/release inputs remain deferred. |
| Phase 0 checkpoint `node tools/repository-check/run.mjs --full --with-source-cache` | Reconfirmed the documented external blocker: runner, extension, dependency, workflow, release snapshots and Goal 05 release checks passed; the gate then stopped in unchanged `tools/goal06/verify-phase0.mjs` because the additive current `Cargo.lock` differs from Goal 06's immutable historical baseline. Goal 13 changes neither file and no later full-gate step is claimed. |
| `node tools/anomaly-arbitration-reference/import-profile.mjs --source-cache .cache/content-reference --fallback-source-cache /Users/mikai/.codex/worktrees/7c74/starclock/.cache/content-reference` | Generated one profile, one active period, five stages and four terminal outcomes from clean fixed source caches. |
| `node tools/anomaly-arbitration-reference/verify-profile.mjs` | Passed generator drift, normalized-envelope, bilingual summary, row-source, manifest-resolution, exact relationship, explicit-approximation and 20-obligation exact-once checks. Output digests: profiles `fbe85022bae6aacfc779512ae12c560b674120ab4cd242b93c51d5a9d154e1c2`; periods `c1c7e3e5ff6bc7d2db2cd6682c13aa17dca253e0022d470169eecc0b6bd6bacb`; stages `c6ce591ad25a4cd80dac183a99f66dde2f86fdd6e2aefdaf296391f6f55474d2`; terminal outcomes `171dfa56f52a735094092e32922f9786892a647d6baf6a613d820915861d1193`. |
| P1-B1 foundation/inventory/manifest/contracts/profile checks; `git diff --check`; plan/status batch-set script; repository-wide local Markdown target script; `node tools/repository-check/run.mjs` | Passed; fixed-source and all prior Goal 13 contracts remain valid, 25 batch IDs agree, 634 local links across 305 Markdown files resolve, no Rust scope was selected and six generated/release inputs remain deferred to the documented full boundary. |
| `node tools/anomaly-arbitration-reference/import-knight-records.mjs`; `node tools/anomaly-arbitration-reference/verify-knight-records.mjs` | Passed deterministic generation/drift, normalized envelope, bilingual text, source receipt, manifest resolution, explicit approximation, three-slot, uniqueness, invalidation ordering, keep/replace, current/best separation and ten-obligation checks. Digests: participant policies `75df925c209bb4b5dbd89da9ab1fdb13855bb012de2c60042ce393e00318c183`; team slots `e1aa2361245d744e63678b3b5b77d6d4904a4b87fa4c0ddb251d1cc195e6f45a`; loadout records `8dc6e1860e5b56e9289c8e00b74f5a39212d271c0dea95b1c4c80ffe092d18f5`; progress records `ed89f8eb5d8dc54085c6197c4ebe56d20045f5ef5b4e8812f03e6b9876811792`. |
| P1-B2 profile/Knight drift checks; `git diff --check`; plan/status batch-set script; repository-wide local Markdown target script; `node tools/repository-check/run.mjs` | Passed; 25 batch IDs agree, 634 local links across 307 Markdown files resolve, no Rust scope was selected and five generated/release inputs remain deferred to the documented full boundary. |

## Frozen counters

Required counts below come only from the generated P0-B3 manifest. They are
source/semantic obligations, not imported `DataReady` rows. Normalized child
rows may expand in later batches but cannot remove or silently merge an
obligation. Zero denominators carry generated selector-closure proofs.

| Category | Required | Accounted | DataReady | Notes |
|---|---:|---:|---:|---|
| Profile/period/entry/terminal outcomes | 20 | 10 | 10 | Profile, active period, four entry locators and four terminal outcomes are DataReady; ten lifecycle constants remain for later Phase 1 batches. |
| Stages/attempts/current-best progress | 16 | 16 | 16 | Four aliases, five StageConfig parents, Plight difficulty and all six record/current-best lifecycle obligations are DataReady. |
| Participant/team/loadout records | 4 | 4 | 4 | Three expanded Knight slots and character/form, Light Cone and Relic-instance uniqueness policies are DataReady. |
| King protection/Plight transitions | 6 | 0 | 0 | Knight contribution, protection composition/removal, normal/Plight state and direct-clear shortcut. |
| Clocks/wave carry/warnings | 9 | 0 | 0 | First-cycle, three limits, carry, warning, low-cycle, expiry/failure and retry obligations. |
| Arbitral Quadrant offers/options | 3 | 0 | 0 | Exact `BuffList` closure: MazeBuff `3033066`, `3033067`, `3033068`. |
| Targets/stars/aggregation | 12 | 0 | 0 | Seven shared battle targets and five evaluation/current-best aggregation obligations. |
| Enemy traits/MazeBuffs/battle events | 14 | 0 | 0 | Eight normal/Plight traits, three Quadrant MazeBuffs and three stage-selected battle events. |
| Blessings/Curios/Occurrences | 0 | 0 | 0 | Three separate generated exact-zero selector proofs. |
| Services/currencies/other pools | 0 | 0 | 0 | Three separate exact-zero proofs; shop/reward locators remain account-facing `EvidenceOnly`. |
| Encounter groups/waves/enemy slots | 293 | 0 | 0 | 27 variants, 26 templates, 115 skills, 52 statuses and 73 mechanical config programs; five StageConfig parents are counted with stages. |
| Mechanic rules | 18 | 0 | 0 | Non-shrinking semantic rule-family minimum; reference contributions make no runtime claim. |
| Semantic fixtures | 18 | 0 | 0 | Same non-shrinking family minimum; P0-B4 freezes fixture shape. |

## Decisions

| Date | Decision | Rationale |
|---|---|---|
| 2026-07-29 | Create Goal 13 as a complete reference-data package, not a runtime goal. | Anomaly Arbitration research can proceed independently without changing shared challenge, Activity or combat runtime. |
| 2026-07-29 | Use `anomaly-arbitration` as the stable mode slug and reserve Goal 13. | Goal 12 already has an active dedicated branch/worktree; no local Goal 13 package or branch existed during the planning audit. |
| 2026-07-29 | Base the branch on `b0cd3cb9…`, the committed Goal 11 setup package. | The base includes Goal 01–11 indexing while excluding all concurrent worktree changes; Goal 12 remains independently owned. |
| 2026-07-29 | Inherit the pinned Version 4.4 source and identity revisions used by Goals 03 and 08–12. | Shared identity, row ownership and membership comparisons require one reproducible historical boundary. |
| 2026-07-29 | Require `G13-P0-B1` to reproduce both caches even though the planning audit found them clean and readable. | Planning-time availability is not a substitute for batch-owned reproducibility evidence. |
| 2026-07-29 | Accept isolated copy-on-write cache materialization after a bounded fresh GitLab partial clone made no progress. | The clean connected seed has the exact commit trees and required blobs, the clone retains canonical origins, and two reproducer/verifier runs prove idempotence without mutating the shared sparse cache; no fresh-network success is claimed. |
| 2026-07-29 | Freeze only committed Goal 07–12 blobs as P0 reconciliation checkpoints. | Concurrent worktrees changed during the audit; their uncommitted files remain protected and non-authoritative. Goal 08 is informational until remotely backed/merged, and Goal 11/12 ownership manifests must be adopted later if they become available. |
| 2026-07-29 | Treat the six `ChallengePeak*` tables and candidate group/stage IDs only as an inventory seed. | Historical periods, rewards and mechanics share the tables; prefix and ID range do not prove active Version 4.4 membership. |
| 2026-07-29 | Require indirect gameplay/config discovery rather than assume no config exists because `Config/Gameplays` has no direct ChallengePeak path. | StageConfig, BattleEventConfig, MazeBuff and the shared ChallengePeak ability program carry transitive mechanics. |
| 2026-07-29 | Audit Blessing, Curio, Occurrence, service and currency families even when the expected result is empty. | Completeness requires a generated zero proof; absence cannot be inferred from this being a challenge mode. |
| 2026-07-29 | Reconcile shared rows by source path, stable row locator and evidence digest without editing another Goal's artifacts. | Concurrent goals must preserve isolated ledgers and surface conflicts for merge coordination. |
| 2026-07-29 | Exclude presentation and account rewards while retaining mechanical locators. | Keeps the pack implementation-ready and within the project content boundary. |
| 2026-07-29 | Finish at Candidate-quality reference data without a Released runtime claim. | Runtime lowering, shared primitive changes and seeded full runs require a later goal. |
| 2026-07-29 | Require every completed batch commit to be pushed and remotely verified before the next batch begins. | Prevents unpublished local progress from becoming the effective resumable source of truth. |
| 2026-07-29 | Define P0-B2 as a file closure that inherits all 2,646 Goal 03 receipts and adds only focused ChallengePeak, shared-table, encounter, mechanical-config, bilingual and exclusion inputs. | Shared Rogue pools still need explicit zero/nonzero reachability proof, while unrelated challenge tables, layout/editor metadata, animation and audio are outside this mechanical source inventory. |
| 2026-07-29 | Permit a second clean cache at the identical fixed revision as a read-only Git object alternate, with lazy fetch disabled and every byte stream checked against the pinned tree OID. | The caches are partial sparse clones with complementary materialized blobs; the OID check preserves raw-Git identity without mutating another Goal's cache or claiming unavailable network access. |
| 2026-07-29 | Select Version 4.4 group 8 only after released observations identify rotation 8 and its exact bilingual title. | The public identity closes to the pinned title hash and explicit group references; maximum ID, prefix and adjacency remain insufficient. |
| 2026-07-29 | Treat shared source files as symbol/row containers, not wholesale reachable content. | Stage, target, MazeBuff, battle-event, enemy and configuration membership is granted only by explicit active fields or recursive stable-ID references; unrelated Rogue symbols in shared files remain unreachable. |
| 2026-07-29 | Freeze six mechanically relevant content-pool families at exact zero while retaining the shop and reward rows as account-facing exclusions. | No active selector/reference reaches Blessings, Curios, Occurrences, gameplay services, currencies or random pools; each zero has a digest and stronger-evidence replacement condition. |
| 2026-07-29 | Partition 37 normalized file families exactly once across `AnomalyArbitration.xlsx`, `AnomalyArbitrationBindings.xlsx` and `AnomalyArbitrationReview.xlsx`. | Keeps system authoring, content bindings and audit/review concerns distinct while preserving typed stable references and one complete no-overwrite workbook set. |
| 2026-07-29 | Require evidence quality and mechanism quality as independent row fields, plus a manifest record set and ordered source refs. | A precise source identity does not by itself prove exact behavior; ownership, coverage, provenance and mechanism certainty remain separately auditable. |
| 2026-07-29 | Freeze 18 non-shrinking semantic fixture families and require six cases for the empty-pool family. | Every unique lifecycle/contribution has a review obligation, while each zero family independently proves its selector closure and replacement trigger. |
| 2026-07-29 | Retain both released public period end dates and decline to synthesize a timezone-aware instant. | The pinned structured snapshot has no exact boundary offset; adjacent displayed dates are evidence to preserve, not silently reconcile. |
| 2026-07-29 | Record normal King availability after three Knight clears as `ApproximateFromReleasedText`, while keeping direct Plight as an exact official alternative. | The official guide gives a recommendation and exact Plight projection but not the normal unlock transition; an independent released observation supports the selected policy until stronger released evidence appears. |
| 2026-07-29 | Model Light Cone and Relic uniqueness by account-owned equipment instance and separate mutable current progress from retained Best Battle Records. | Released official instructions identify cross-team recorded equipment as the invalidation trigger and state that current resets do not affect the highest simultaneous three-Knight star record. |
| 2026-07-29 | Reject duplicate base character IDs across cleared Knight records and retain character-plus-Path as a visible `ProjectPolicy` form key. | Released text fixes recorded team members but does not explicitly define alternate-Path identity; fail-closed base identity preserves the observed no-overlap rule without claiming the form distinction is exact. |

## Research cases

| ID | State | Question | Owner | Replacement condition |
|---|---|---|---|---|
| `G13-R01` | `Resolved` | Which direct and transitive table, gameplay/config, TextMap, StageConfig, enemy/wave and ability files complete the focused inventory? | P0-B2 | Reopen only if P0-B3 admits a row whose explicit reference escapes the 2,745-file inventory; replacement requires updating the generated closure and another byte-identical double generation. |
| `G13-R02` | `Resolved` | Which released selector proves the active Version 4.4 period and separates it from seven historical groups and other challenge families? | P0-B3 | Reopen only if stronger released evidence contradicts rotation 8/title identity or the pinned group-to-stage reference chain. |
| `G13-R03` | `Resolved` | What are the exact participant, character, Light Cone and Relic identity scopes and the atomic invalidation order when a recorded Knight loadout changes? | P1-B2 | Reopen if released evidence shows equipment identity is template-level rather than instance-level, or demonstrates alternate Paths sharing/separating Knight uniqueness differently; replacement must update the explicit combat-form policy and its fixtures. |
| `G13-R04` | `Open` | How do three Knight-clear states compose King protection, when is protection removed, and what does direct Plight victory project into current and best records? | P1-B3 | Replace with exact transition programs or field-level policies carrying alternatives and lifecycle fixtures. |
| `G13-R05` | `Open` | What are the first-cycle Action Value, stage-local cycle tick, wave carry, low-cycle trigger and expiry ordering semantics? | P1-B4 | Replace with configuration/ability evidence or explicit deterministic policies for each unresolved boundary. |
| `G13-R06` | `Open` | Which Arbitral Quadrant rows are offered in the active period, how are they selected and when do their battle contributions start and end? | P1-B5 / P2-B3 | Replace with selector-backed offers, exact parameters, contribution programs and selection/teardown fixtures. |
| `G13-R07` | `Open` | How are stars, current progress, simultaneous three-Knight best progress and King results evaluated and retained across reset/retry? | P1-B6 | Replace with typed objective/aggregation rows and fixtures for improvement, regression, reset and Plight shortcut. |
| `G13-R08` | `Resolved` | Are any Blessings, Curios, Occurrences, services, currencies or analogous pools mechanically reachable in the active period? | P0-B3 / P2-B1 | Reopen only when a released active selector or transitive reference names a member; P2-B1 imports and re-verifies the six frozen zero proofs. |
| `G13-R09` | `Open` | Which active enemy traits, King/Plight modifiers and shared battle events bind to each stage and difficulty? | P2-B2–B3 | Replace with exact stage/target/MazeBuff/event/ability joins and one fixture per distinct contribution. |
| `G13-R10` | `Open` | Which StageConfig waves, concrete enemy variants, skills, AI, phases and difficulty inputs define every active encounter? | P2-B4 | Replace with complete encounter/enemy dossiers or an explicit nonblocking boundary for unavailable released evidence. |
| `G13-R11` | `Open` | Which hidden ordering, timing, caps, rounding and fallback fields remain unavailable after bounded research? | P2-B5 / P4-B2 | Replace each field with exact/observed evidence or a reviewed approximation/project-policy row with a concrete stronger-evidence trigger. |

## Terminal checklist

- [x] Exact active-period category manifests and denominators are frozen.
- [x] Both pinned caches and the focused
      table/config/TextMap/Stage/ability inventory regenerate deterministically.
- [ ] Complete normalized pack and canonical pack index regenerate without
      drift.
- [ ] All required rows have bilingual summaries and row-level provenance.
- [ ] Ownership, active-period enablement and shared reachability are explicit
      and fail closed.
- [ ] Empty content-pool families have generated selector-closure proofs.
- [ ] Shared classifications reconcile with committed Goal 07–12 facts.
- [ ] All required mechanics are exact or explicitly
      approximate/policy-bound.
- [ ] Knight records/uniqueness, King/Plight, clocks, Arbitral Quadrant,
      objectives and aggregation have complete semantic fixtures.
- [ ] Encounter identities, StageConfig rows, waves, traits and boss bindings
      resolve.
- [ ] Isolated Sora schemas, templates and generated readers validate.
- [ ] All three complete `openpyxl` workbooks pass structural and visual QA.
- [ ] Sora production/debug exports regenerate without drift and load through
      isolated readers.
- [ ] Goal 03 evidence and all other mode/production bundle identities remain
      unchanged.
- [ ] Coverage reports 100% `DataReady` and no blocking research row.
- [ ] Every completed batch commit is reachable from its recorded remote
      branch at the recorded commit ID.
- [ ] Clean-checkout acceptance passes and `G13-P4-B4` is committed and pushed.

## Completion record

| Field | Value |
|---|---|
| Final state | Pending |
| Completion commit | — |
| Remote/branch verification | — |
| Anomaly Arbitration reference bundle | — |
| Workbook semantic digest | — |
| Coverage | Denominators pending `G13-P0-B3` |
| Release evidence | — |
| Remaining required work | Anomaly Arbitration runtime lowering, integration, handlers, controller/API exposure and seeded full challenge runs belong to a later goal. |
