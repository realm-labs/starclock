# Goal 01 Status — Complete Core Combat and Released Character Content

This file is the persistent execution ledger for
[Goal 01](01-core-combat-and-content.md). The executor must update it in the same
commit as every implementation or content batch.

## Goal state

| Field | Value |
|---|---|
| Goal ID | `core-combat-v1` |
| State | `InProgress` |
| Active phase | Phase 1 — Workspace and reproducible data foundation |
| Next unblocked batch | `G01-P1-B9` |
| Last completed batch | `G01-P1-B8` |
| Last completed commit | This ledger row's containing commit; verify after commit |
| Goal plan baseline | Phase 0 complete; bound to frozen manifests and coverage evidence |
| Content prerequisite | Verified and bound by `G01-P0-B1` |
| Content reference digest | `0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a` |
| Goal manifest digest | `e2188c7844d678253c98d569db017dbad7101541cf502aba4c2eb80c0435bf19` |
| Blocking condition | None |

Allowed goal states are `ReadyToStart`, `InProgress`, `Blocked` and `Complete`.
Use `Blocked` only when no independent batch can progress and record the exact
external decision or evidence required. Phase completion alone never changes the
goal to `Complete`.

## Frozen manifest counters

These totals are populated by `G01-P0-B1` from machine-readable manifests. Do
not manually estimate missing totals.

| Required manifest | Digest | Required | DataReady | Disabled announced | Coverage |
|---|---:|---:|---:|---:|---:|
| Released character combat forms | `d83b6a1621f47ed6964e164b560b317288f1a8a3e53ac0ed3654d5df8ff815e9` | 88 | 0 | 2 announced outside enabled pack | 0% runtime data |
| Released Light Cones | `317afdf6480e58d7396d781b5d23b6377d90b03ca1fc24e62b88ffbb3f4b9994` | 165 | 0 | 0 | 0% runtime data |
| `standard-v1` enemies/variants | `a9fdd427940738a712cbf4b796281113a9df3c2f7465c5f51c76356885eeb84b` | 17 exact variants | 0 | 0 | 0% runtime data |
| `standard-v1` encounters | `a9fdd427940738a712cbf4b796281113a9df3c2f7465c5f51c76356885eeb84b` | 6 | 0 | 0 | 0% runtime data |
| `standard-v1` scenarios | `a9fdd427940738a712cbf4b796281113a9df3c2f7465c5f51c76356885eeb84b` | 6 | 0 | 0 | 0% runtime data |

Disabled announced entries are recorded for audit but are never included in a
required-coverage denominator until public release and a new manifest revision.

## Phase ledger

| Phase | State | Exit evidence |
|---|---|---|
| Phase 0 — Freeze scope and evidence | `Complete` | Frozen manifest `e2188c7844d678253c98d569db017dbad7101541cf502aba4c2eb80c0435bf19`; provenance evidence `e629313eee624ccb124036ec6fd4664df9ca761e392d026ce6f2f7c34a184466`; research evidence `7d0e55631f584c58c72128c18ccc3715cf7f50ab705973815ed244d31979d62e`; initial coverage `05f3539927e298d570a6d128e2b27613f3639620260d16509ab0deb8cda03d69`. All 283 required entries are accounted, zero are `DataReady`, and two announced forms remain disabled outside the denominator. |
| Phase 1 — Workspace and reproducible data foundation | `InProgress` | `G01-P1-B1` established the nine-crate workspace; `G01-P1-B2` pinned the compiler/tool inventory, reviewed four locked registry packages, kept `fixnum` private and added stable IDs; `G01-P1-B3` established the shared-lint and complete local repository gate; `G01-P1-B4` bound and executed Sora 0.3.0 with a complete capability golden; `G01-P1-B5` committed the pinned native/compile-only CI matrix and evidence boundary; `G01-P1-B6` froze shared identity/localization/version/provenance/evidence/decimal schema contracts; `G01-P1-B7` froze character/build/equipment schema contracts; `G01-P1-B8` froze typed Rule IR/effect/modifier contracts; `G01-P1-B9` next. |
| Phase 2 — Deterministic primitives | `Pending` | None |
| Phase 3 — Executable combat vertical slice | `Pending` | None |
| Phase 4 — Complete shared combat kernel | `Pending` | None |
| Phase 5 — Build compiler, Traces, Eidolons and Light Cones | `Pending` | None |
| Phase 6 — Standard orchestration, AI, CLI and replay | `Pending` | None |
| Phase 7 — Complete released content import | `Pending` | None |
| Phase 8 — Hardening and documentation freeze | `Pending` | None |

Allowed batch/phase states are `Pending`, `InProgress`, `Researching`, `Blocked`,
`Complete` and `NotApplicable`. `NotApplicable` requires a decision-log entry and
may not be used for a required acceptance gate.

## Batch ledger

Add one row per concrete batch. Expand the Phase 7 partition families after the
Phase 0 manifests are frozen.

| Batch | State | Commit | Validation evidence | Notes |
|---|---|---|---|---|
| `G01-P0-B1` | `Complete` | `3c04cb1` | `node tools/content-reference/verify.mjs content-reference/v4.4`; `node tools/goal-manifest/generate.mjs --check`; `node tools/goal-manifest/verify.mjs`; `cargo fmt --all -- --check`; `cargo test`; `git diff --check`. Frozen evidence: [`manifest-index.json`](../../content-manifests/core-combat-v1/manifest-index.json), [`standard-v1.json`](../../content-manifests/core-combat-v1/standard-v1.json), [`partitions.json`](../../content-manifests/core-combat-v1/partitions.json). | Bound reference pack `0dca8ae5…f5a`; froze goal manifest `e2188c78…f19`, 88 forms, 165 Light Cones, 17 exact enemy variants, 6 encounters and 6 scenarios. |
| `G01-P0-B2` | `Complete` | `6654540` | `node tools/goal-provenance/generate.mjs --check`; `node tools/goal-provenance/verify.mjs`; `node tools/content-reference/verify.mjs content-reference/v4.4`; `node tools/goal-manifest/generate.mjs --check`; `node tools/goal-manifest/verify.mjs`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `git diff --check`. Evidence: [`evidence-index.json`](../../evidence/core-combat-v1/reference-binding/evidence-index.json), [`source-cache-report.json`](../../evidence/core-combat-v1/reference-binding/source-cache-report.json), [`saber-archer-audit.json`](../../evidence/core-combat-v1/reference-binding/saber-archer-audit.json). | Verified both pinned revisions and all 1,811 source hashes; regenerated all 13 pack JSON files exactly; mapped 283 frozen entries to a 3,085-record closure; all 10 explicit approximations retain evidence hashes and no approximation is unbound. |
| `G01-P0-B3` | `Complete` | `63b1fe1` | `node tools/goal-research/generate.mjs --check`; `node tools/goal-research/verify.mjs`; `node tools/content-reference/verify.mjs content-reference/v4.4`; `node tools/goal-manifest/generate.mjs --check`; `node tools/goal-manifest/verify.mjs`; `node tools/goal-provenance/generate.mjs --check`; `node tools/goal-provenance/verify.mjs`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-features`; `git diff --check`. Evidence: [`evidence-index.json`](../../evidence/core-combat-v1/research-register/evidence-index.json), [`research-cases.json`](../../evidence/core-combat-v1/research-register/research-cases.json), [`fixture-specifications.json`](../../evidence/core-combat-v1/research-register/fixture-specifications.json), [`decision-records.json`](../../evidence/core-combat-v1/research-register/decision-records.json). | Named 19 V1a, 8 cross-kit Elation and 10 Himeko Nova cases; assigned every case to a later batch; bound all cases to prepared text hashes and reproducible observation/golden envelopes; no ambiguity was converted into a convenient default. |
| `G01-P0-B4` | `Complete` | `dd2a2ae` | `node tools/content-reference/verify.mjs content-reference/v4.4`; `node tools/goal-manifest/generate.mjs --check`; `node tools/goal-manifest/verify.mjs`; `node tools/goal-provenance/generate.mjs --check`; `node tools/goal-provenance/verify.mjs`; `node tools/goal-research/generate.mjs --check`; `node tools/goal-research/verify.mjs`; `node tools/goal-coverage/generate.mjs --check`; `node tools/goal-coverage/verify.mjs`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-features`; `git diff --check`. Evidence: [`coverage-index.json`](../../evidence/core-combat-v1/coverage/coverage-index.json), [`goal-coverage.json`](../../evidence/core-combat-v1/coverage/goal-coverage.json). | Accounted for 283/283 required entries including the Standard profile; terminal states are 188 `Cataloged`, 85 `Documented`, 10 `Researching`, zero `DataReady` and zero `GoldenVerified`; verified every goal/reference documentation counter and retained two announced forms as disabled audit-only entries. |
| `G01-P1-B1` | `Complete` | `698480c` | `node tools/workspace/verify-dependencies.mjs`; `node tools/content-reference/verify.mjs content-reference/v4.4`; `node tools/goal-manifest/verify.mjs`; `node tools/goal-provenance/verify.mjs`; `node tools/goal-research/verify.mjs`; `node tools/goal-coverage/verify.mjs`; `cargo check --workspace --all-targets --all-features`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-features`; `git diff --check`. Boundary evidence: [`crates/README.md`](../../crates/README.md), [`verify-dependencies.mjs`](../../tools/workspace/verify-dependencies.mjs), [`workspace_boundaries.rs`](../../crates/starclock-cli/tests/workspace_boundaries.rs). | Replaced the placeholder root package with a virtual workspace containing exactly nine in-scope crates; `starclock-combat` has zero dependencies; no external, engine, challenge or universe dependency/crate was introduced. |
| `G01-P1-B2` | `Complete` | `2c0d779` | `node tools/dependency-policy/verify.mjs`; `node tools/workspace/verify-dependencies.mjs`; `node tools/content-reference/verify.mjs content-reference/v4.4`; `node tools/goal-manifest/verify.mjs`; `node tools/goal-provenance/verify.mjs`; `node tools/goal-research/verify.mjs`; `node tools/goal-coverage/verify.mjs`; `cargo check --workspace --all-targets --all-features`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-targets --all-features`; `git diff --check`. Policy evidence: [`dependency-and-tool-policy.json`](../../policy/dependency-and-tool-policy.json), [`dependency-and-tool-policy.md`](../dependency-and-tool-policy.md), [`rust-toolchain.toml`](../../rust-toolchain.toml). | Pinned Rust/Cargo 1.97.0 and Node 24.15.0; reviewed `fixnum =0.9.5` plus all three transitives and measured a 1,161 ms fresh combat check; added non-zero fixed-width definition/runtime IDs and six-decimal `Scalar`/`Ratio` wrappers without exposing `fixnum`. |
| `G01-P1-B3` | `Complete` | `0a2691f` | `node tools/repository-check/run.mjs`; `node tools/repository-check/run.mjs --with-source-cache`; `git diff --check`. Policy evidence: [`repository-checks.json`](../../policy/repository-checks.json), [`generated-drift.json`](../../policy/generated-drift.json), [`repository-check README`](../../tools/repository-check/README.md). | One pinned command verifies the exact dependency/tool inventory, nine-crate dependency direction and shared-lint inheritance, 12 handwritten Rust files against physical-line/facade limits, two explicit reviewed public re-export declarations, eight clean-checkout generated/integrity checks, formatting, Clippy and all-target/all-feature tests. The optional cache gate also reproduced the prepared pack and bound evidence as a ninth check. |
| `G01-P1-B4` | `Complete` | `e4ea293` | `node tools/sora/install.mjs`; `node tools/sora/verify-golden.mjs`; `node tools/repository-check/run.mjs --with-source-cache`; `git diff --check`. Capability evidence: [`capability-lock.json`](../../config/sora-golden/capability-lock.json), [`expected-manifest.json`](../../config/sora-golden/expected-manifest.json), [`sora-toolchain.json`](../../policy/sora-toolchain.json), [`Sora capability lock`](../sora-0.3.0-capability-lock.md). | Bound crates.io archive `90d37310…763a`, annotated tag `fe12080f…aeeb` / commit `4afadfb4…03dc`; proved check/build/schema-lock, template and preview/write sync, primary-key references, a unique index, union/ordered child-table materialization, Rust codegen/reader, `.sora` and `json-debug`. Thirteen stable artifacts digest to `30687554…5f09`; XLSX drift is semantic. Recorded and regression-proved 0.3.0's Windows formatter probe, bare clean-path, unsigned decoder and generated-reader dependency limitations. |
| `G01-P1-B5` | `Complete` | `cc27087` | `node tools/ci/verify-workflow.mjs`; `node tools/ci/write-evidence.mjs --profile windows-x64-native --output .ci-evidence/local-native.json`; `rustup target add aarch64-pc-windows-msvc --toolchain 1.97.0`; `cargo check --workspace --all-targets --all-features --target aarch64-pc-windows-msvc`; `node tools/ci/write-evidence.mjs --profile windows-arm64-compile --output .ci-evidence/local-compile.json`; `node tools/repository-check/run.mjs`; `node tools/repository-check/run.mjs --with-source-cache`; `git diff --check`. Contract evidence: [CI workflow](../../.github/workflows/ci.yml), [`ci-matrix.json`](../../policy/ci-matrix.json), [platform matrix](../ci-platform-matrix.md). | Immutable action SHAs and explicit hosted labels define native Windows x64, Linux x64 and macOS ARM64 jobs that install checksum-bound Sora and invoke the shared repository runner. Paired alternate-CPU jobs run only `cargo check`; evidence records actual host/architecture, target, tools, policy hash, Sora golden digest and an explicit execution flag. Local Windows native evidence-writer smoke and Windows ARM64 compile-only check passed; no unexecuted Linux/macOS runtime claim is made here. |
| `G01-P1-B6` | `Complete` | `29a62ab` | `node tools/config-schema/verify-common.mjs --bless`; `node tools/config-schema/verify-common.mjs`; `node tools/repository-check/run.mjs`; `node tools/repository-check/run.mjs --with-source-cache`; `git diff --check`. Golden evidence: [`expected-manifest.json`](../../config/schema-fixtures/common/expected-manifest.json), [`config-schema.json`](../../policy/config-schema.json), [common schema contract](../common-configuration-schema.md). | Shared Sora tables now own global identity/bilingual metadata/version/release/coverage, source/confidence records, named evidence and ordered fact bindings, plus the config manifest. Positive `i32` transport and string-only canonical-decimal policy are mechanical; integer-only boundary tests include signed `i64` extremes. Sora check/build/schema-lock/codegen/binary/diagnostic exports reproduce 24 byte-golden files at `e8811696…14e8`; six Excel template names and negative localization/reference/uniqueness cases pass. Synthetic TOML rows stay disabled fixture evidence; production `.xlsx` and the generated reader remain owned by `G01-P1-B10`. |
| `G01-P1-B7` | `Complete` | `9a0b62c` | `node tools/config-schema/verify-character-build.mjs --bless`; `node tools/config-schema/verify-character-build.mjs`; `node tools/repository-check/run.mjs --with-source-cache`; `git diff --check`. Golden evidence: [`expected-manifest.json`](../../config/schema-fixtures/character-build/expected-manifest.json), [character/build schema contract](../character-build-configuration-schema.md). | Four schema modules own Ability/HitPlan, Character/Trace/Eidolon, the normative ten-variant closed `BuildPatch`, and Light Cone/S1-S5 facts. Typed references and unique ordered children are mechanical; the fixture proves Trace self-references, Technique binding, two-hit millionth-unit ratios, E1-E6 and scalable S1-S5 completeness, constant-rank policy, and negative missing-reference/duplicate-rank cases. Sora check/build/schema-lock/codegen/binary/diagnostic exports reproduce 73 byte hashes at `7dc7667a…2a2a`. All 16 fixture identities remain disabled `ProjectFixture` rows; Rule IR is deferred to `G01-P1-B8`, production `.xlsx` to B10 and domain validation to B11, with zero coverage promotion. |
| `G01-P1-B8` | `Complete` | This row's containing commit | `node tools/config-schema/verify-rule-ir.mjs --bless`; `node tools/config-schema/verify-rule-ir.mjs`; `node tools/repository-check/run.mjs --with-source-cache`; `git diff --check`. Golden evidence: [`expected-manifest.json`](../../config/schema-fixtures/rule-ir/expected-manifest.json), [typed Rule IR schema contract](../rule-ir-configuration-schema.md). | Six schema modules define Rule/Slot/Trigger/EventFilter, Selector, Expression/Condition, finite Program/Operation, Effect and Modifier/Snapshot families. The composed fixture proves 47 tables, ordered slots/predicates/programs, two normal triggers plus a replacement proposal, `If` and bounded `ForEach`, seven operation rows, cause-role filters, stacking/snapshot bindings and disabled static-handler metadata. Sora rejects missing expression/modifier references and duplicate trigger order; the verifier rejects cross-domain children, cyclic programs/expressions and replacement triggers reaching mutations. Check/build/schema-lock/codegen/binary/diagnostic outputs reproduce 182 byte hashes at `ea2ae5d8…7752`. All 24 identities and the handler remain disabled synthetic evidence; runtime evaluation, production `.xlsx`, domain catalogs and Activity handoff remain `G01-P4-B1`/B10/B11/B9 work, with zero coverage promotion. |
| `G01-P1-B9` | `Pending` | — | — | Enemy/AI/Encounter/Standard and minimum generic Activity handoff schemas/fixtures. |
| `G01-P1-B10` | `Pending` | — | — | Deterministic reference-to-workbook bootstrap, exports, generated readers and conversion boundary. |
| `G01-P1-B11` | `Pending` | — | — | Empty/representative domain catalogs and clean full-pipeline regeneration fixtures. |
| `G01-P2-B1` through `G01-P2-B6` | `Pending` | — | — | Deterministic primitives, replay contract and initial property harness. Expand before Phase 2 starts. |
| `G01-P3-B1` through `G01-P3-B8` | `Pending` | — | — | Synthetic vertical slice, performance baseline and command properties. Expand before Phase 3 starts. |
| `G01-P4-B1` through `G01-P4-B11` | `Pending` | — | — | Shared kernel interleaved with Excel/Sora V1a mechanism probes. Expand before Phase 4 starts. |
| `G01-P5-B1` through `G01-P5-B6` | `Pending` | — | — | Build, Trace, Eidolon and Light Cone compiler. Expand before Phase 5 starts. |
| `G01-P6-B1` through `G01-P6-B6` | `Pending` | — | — | Standard Activity, controllers, replay payloads, CLI and scenarios. Expand before Phase 6 starts. |
| `G01-P7-V1B` | `Pending` | — | — | Promote representative probes into complete production content. |
| `G01-P7-C01` | `Pending` | — | — | 8 forms; frozen membership below. |
| `G01-P7-C02` | `Pending` | — | — | 8 forms; frozen membership below. |
| `G01-P7-C03` | `Pending` | — | — | 8 forms; frozen membership below. |
| `G01-P7-C04` | `Pending` | — | — | 8 forms; frozen membership below. |
| `G01-P7-C05` | `Pending` | — | — | 8 forms; frozen membership below. |
| `G01-P7-C06` | `Pending` | — | — | 8 forms; frozen membership below. |
| `G01-P7-C07` | `Pending` | — | — | 8 forms; frozen membership below. |
| `G01-P7-C08` | `Pending` | — | — | 8 forms; frozen membership below. |
| `G01-P7-C09` | `Pending` | — | — | 8 forms; frozen membership below. |
| `G01-P7-C10` | `Pending` | — | — | 8 forms; frozen membership below. |
| `G01-P7-C11` | `Pending` | — | — | 2 forms; frozen membership below. |
| `G01-P7-L01` | `Pending` | — | — | 16 Light Cones; frozen membership below. |
| `G01-P7-L02` | `Pending` | — | — | 16 Light Cones; frozen membership below. |
| `G01-P7-L03` | `Pending` | — | — | 16 Light Cones; frozen membership below. |
| `G01-P7-L04` | `Pending` | — | — | 16 Light Cones; frozen membership below. |
| `G01-P7-L05` | `Pending` | — | — | 16 Light Cones; frozen membership below. |
| `G01-P7-L06` | `Pending` | — | — | 16 Light Cones; frozen membership below. |
| `G01-P7-L07` | `Pending` | — | — | 16 Light Cones; frozen membership below. |
| `G01-P7-L08` | `Pending` | — | — | 16 Light Cones; frozen membership below. |
| `G01-P7-L09` | `Pending` | — | — | 16 Light Cones; frozen membership below. |
| `G01-P7-L10` | `Pending` | — | — | 16 Light Cones; frozen membership below. |
| `G01-P7-L11` | `Pending` | — | — | 5 Light Cones; frozen membership below. |
| `G01-P7-M01` | `Pending` | — | [`research-cases.json`](../../evidence/core-combat-v1/research-register/research-cases.json) | Resolve and implement the generic Himeko Nova Assist Skill, shared-use counter, Starblazer actor and companion-protocol threshold/forced-use mechanism before `G01-P7-C04`; all 10 approximation fixtures must pass. |
| `G01-P7-M02+` | `Pending` | — | — | Add newly discovered shared primitive or reviewed native-handler prerequisites before their dependent content partitions. |
| `G01-P7-R1` | `Pending` | — | — | Clean catalog and coverage regeneration. |
| `G01-P7-R2` | `Pending` | — | — | Manifest-wide build compilation. |
| `G01-P8-B1` through `G01-P8-B7` | `Pending` | — | — | Audits, established CI matrix, fuzz expansion, performance gate and freeze. Expand before Phase 8 starts. |

For a completed row, validation evidence must include commands and a link to a
committed report or fixture when applicable. A commit hash alone is insufficient.

## Required content partitions

### Character partitions

Frozen by `G01-P0-B1` in stable manifest-ID order. `G01-P7-V1B` owns the six
representative probe forms: `character.aglaea`, `character.asta`,
`character.clara`, `character.firefly`, `character.kafka`, and
`character.silver-wolf-lv-999`.

| Batch | Stable manifest IDs |
|---|---|
| `G01-P7-C01` | `character.acheron`, `character.anaxa`, `character.archer`, `character.argenti`, `character.arlan`, `character.ashveil`, `character.aventurine`, `character.bailu` |
| `G01-P7-C02` | `character.black-swan`, `character.blade`, `character.boothill`, `character.bronya`, `character.castorice`, `character.cerydra`, `character.cipher`, `character.cyrene` |
| `G01-P7-C03` | `character.dan-heng`, `character.dan-heng-imbibitor-lunae`, `character.dan-heng-permansor-terrae`, `character.dr-ratio`, `character.evanescia`, `character.evernight`, `character.feixiao`, `character.fu-xuan` |
| `G01-P7-C04` | `character.fugue`, `character.gallagher`, `character.gepard`, `character.guinaifen`, `character.hanya`, `character.herta`, `character.himeko`, `character.himeko-nova` |
| `G01-P7-C05` | `character.hook`, `character.huohuo`, `character.hyacine`, `character.hysilens`, `character.jade`, `character.jiaoqiu`, `character.jing-yuan`, `character.jingliu` |
| `G01-P7-C06` | `character.lingsha`, `character.luka`, `character.luocha`, `character.lynx`, `character.march-7th.preservation`, `character.march-7th.the-hunt`, `character.misha`, `character.mortenax-blade` |
| `G01-P7-C07` | `character.moze`, `character.mydei`, `character.natasha`, `character.pela`, `character.phainon`, `character.qingque`, `character.rappa`, `character.robin` |
| `G01-P7-C08` | `character.ruan-mei`, `character.saber`, `character.sampo`, `character.seele`, `character.serval`, `character.silver-wolf`, `character.sparkle`, `character.sparxie` |
| `G01-P7-C09` | `character.sunday`, `character.sushang`, `character.the-dahlia`, `character.the-herta`, `character.tingyun`, `character.topaz-numby`, `character.trailblazer.destruction`, `character.trailblazer.elation` |
| `G01-P7-C10` | `character.trailblazer.harmony`, `character.trailblazer.preservation`, `character.trailblazer.remembrance`, `character.tribbie`, `character.welt`, `character.xueyi`, `character.yanqing`, `character.yao-guang` |
| `G01-P7-C11` | `character.yukong`, `character.yunli` |

### Light Cone partitions

Frozen by `G01-P0-B1` in stable manifest-ID order.

| Batch | Stable manifest IDs |
|---|---|
| `G01-P7-L01` | `light-cone.a-dream-scented-in-wheat`, `light-cone.a-grounded-ascent`, `light-cone.a-secret-vow`, `light-cone.a-star-that-lights-the-night`, `light-cone.a-thankless-coronation`, `light-cone.a-trail-of-bygone-blood`, `light-cone.adversarial`, `light-cone.after-the-charmony-fall`, `light-cone.along-the-passing-shore`, `light-cone.amber`, `light-cone.an-instant-before-a-gaze`, `light-cone.arrows`, `light-cone.baptism-of-pure-thought`, `light-cone.before-dawn`, `light-cone.before-the-tutorial-mission-starts`, `light-cone.boundless-choreo` |
| `G01-P7-L02` | `light-cone.brighter-than-the-sun`, `light-cone.but-the-battle-isnt-over`, `light-cone.carve-the-moon-weave-the-clouds`, `light-cone.chorus`, `light-cone.collapsing-sky`, `light-cone.concert-for-two`, `light-cone.cornucopia`, `light-cone.cruising-in-the-stellar-sea`, `light-cone.dance-at-sunset`, `light-cone.dance-dance-dance`, `light-cone.darting-arrow`, `light-cone.data-bank`, `light-cone.day-one-of-my-new-life`, `light-cone.dazzled-by-a-flowery-world`, `light-cone.defense`, `light-cone.destinys-threads-forewoven` |
| `G01-P7-L03` | `light-cone.dreams-montage`, `light-cone.dreamville-adventure`, `light-cone.earthly-escapade`, `light-cone.echoes-of-the-coffin`, `light-cone.elation-brimming-with-blessings`, `light-cone.epoch-etched-in-golden-blood`, `light-cone.eternal-calculus`, `light-cone.eyes-of-the-prey`, `light-cone.fermata`, `light-cone.final-victor`, `light-cone.fine-fruit`, `light-cone.flame-of-blood-blaze-my-path`, `light-cone.flames-afar`, `light-cone.flickering-stars`, `light-cone.flowing-nightglow`, `light-cone.fly-into-a-pink-tomorrow` |
| `G01-P7-L04` | `light-cone.for-tomorrows-journey`, `light-cone.geniuses-greetings`, `light-cone.geniuses-repose`, `light-cone.good-night-and-sleep-well`, `light-cone.hey-over-here`, `light-cone.hidden-shadow`, `light-cone.holiday-thermae-escapade`, `light-cone.i-am-as-you-behold`, `light-cone.i-shall-be-my-own-sword`, `light-cone.i-venture-forth-to-hunt`, `light-cone.if-time-were-a-flower`, `light-cone.in-pursuit-of-the-wind`, `light-cone.in-the-name-of-the-world`, `light-cone.in-the-night`, `light-cone.incessant-rain`, `light-cone.indelible-promise` |
| `G01-P7-L05` | `light-cone.inherently-unjust-destiny`, `light-cone.into-the-unreachable-veil`, `light-cone.its-showtime`, `light-cone.journey-forever-peaceful`, `light-cone.landaus-choice`, `light-cone.lies-dance-on-the-breeze`, `light-cone.life-should-be-cast-to-flames`, `light-cone.lingering-tear`, `light-cone.long-may-rainbows-adorn-the-sky`, `light-cone.long-road-leads-home`, `light-cone.loop`, `light-cone.make-farewells-more-beautiful`, `light-cone.make-the-world-clamor`, `light-cone.mediation`, `light-cone.memories-of-the-past`, `light-cone.memorys-curtain-never-falls` |
| `G01-P7-L06` | `light-cone.meshing-cogs`, `light-cone.moment-of-victory`, `light-cone.multiplication`, `light-cone.mushy-shroomys-adventures`, `light-cone.mutual-demise`, `light-cone.never-forget-her-flame`, `light-cone.night-of-fright`, `light-cone.night-on-the-milky-way`, `light-cone.ninja-record-sound-hunt`, `light-cone.ninjutsu-inscription-dazzling-evilbreaker`, `light-cone.nowhere-to-run`, `light-cone.on-the-fall-of-an-aeon`, `light-cone.only-silence-remains`, `light-cone.passkey`, `light-cone.past-and-future`, `light-cone.past-self-in-mirror` |
| `G01-P7-L07` | `light-cone.patience-is-all-you-need`, `light-cone.perfect-timing`, `light-cone.pioneering`, `light-cone.planetary-rendezvous`, `light-cone.poised-to-bloom`, `light-cone.post-op-conversation`, `light-cone.quid-pro-quo`, `light-cone.reforged-in-hellfire`, `light-cone.reforged-remembrance`, `light-cone.reminiscence`, `light-cone.resolution-shines-as-pearls-of-sweat`, `light-cone.return-to-darkness`, `light-cone.river-flows-in-spring`, `light-cone.sagacity`, `light-cone.sailing-towards-a-second-life`, `light-cone.scent-alone-stays-true` |
| `G01-P7-L08` | `light-cone.see-you-at-the-end`, `light-cone.shadowburn`, `light-cone.shadowed-by-night`, `light-cone.shared-feeling`, `light-cone.shattered-home`, `light-cone.she-already-shut-her-eyes`, `light-cone.sleep-like-the-dead`, `light-cone.sneering`, `light-cone.solitary-healing`, `light-cone.something-irreplaceable`, `light-cone.subscribe-for-more`, `light-cone.sweat-now-cry-less`, `light-cone.swordplay`, `light-cone.texture-of-memories`, `light-cone.the-birth-of-the-self`, `light-cone.the-day-the-cosmos-fell` |
| `G01-P7-L09` | `light-cone.the-finale-of-a-lie`, `light-cone.the-flower-remembers`, `light-cone.the-forever-victual`, `light-cone.the-great-cosmic-enterprise`, `light-cone.the-hell-where-ideals-burn`, `light-cone.the-moles-welcome-you`, `light-cone.the-seriousness-of-breakfast`, `light-cone.the-storys-next-page`, `light-cone.the-unreachable-side`, `light-cone.this-is-me`, `light-cone.this-love-forever`, `light-cone.those-many-springs`, `light-cone.though-worlds-apart`, `light-cone.thus-burns-the-dawn`, `light-cone.time-waits-for-no-one`, `light-cone.time-woven-into-gold` |
| `G01-P7-L10` | `light-cone.to-evernights-stars`, `light-cone.today-is-another-peaceful-day`, `light-cone.todays-good-luck`, `light-cone.tomorrow-together`, `light-cone.trend-of-the-universal-market`, `light-cone.under-the-blue-sky`, `light-cone.until-the-flowers-bloom-again`, `light-cone.unto-tomorrows-morrow`, `light-cone.victory-in-a-blink`, `light-cone.void`, `light-cone.warmth-shortens-cold-nights`, `light-cone.we-are-wildfire`, `light-cone.we-will-meet-again`, `light-cone.welcome-to-the-cosmic-city`, `light-cone.what-is-real`, `light-cone.when-she-decided-to-see` |
| `G01-P7-L11` | `light-cone.whereabouts-should-dreams-rest`, `light-cone.why-does-the-ocean-sing`, `light-cone.woof-walk-time`, `light-cone.worrisome-blissful`, `light-cone.yet-hope-is-priceless` |

### Standard battle partitions

Frozen by `G01-P0-B1`. The exact 17 enemy variants and six four-build scenario
bindings are in [`standard-v1.json`](../../content-manifests/core-combat-v1/standard-v1.json).

| Encounter | Required archetype coverage | Seeded scenario |
|---|---|---|
| `encounter.cocoon.0001` | Basic single wave | `scenario.standard-v1.basic-single-wave` |
| `encounter.mainline.0541` | Three waves, summoner/replaceable entity, DoT, revival/return, invalidation, AfterAction advancement | `scenario.standard-v1.multi-wave-dot-revival` |
| `encounter.farmelement.0008` | Elite with adds, crowd control, invalidation | `scenario.standard-v1.elite-control-counter` |
| `encounter.mainline.0276` | Multi-phase boss and crowd control | `scenario.standard-v1.cocolia-phase-change` |
| `encounter.mainline.0755` | Three-layer Toughness routing and boss phases | `scenario.standard-v1.layered-toughness` |
| `encounter.mainline.1253` | Three waves, elites/adds, summoning, CC/DoT, revival/return, self-destruction/invalidation, AfterAction advancement | `scenario.standard-v1.target-invalidation-and-return` |

## Research and blockers

| ID | State | Question or blocker | Evidence required | Owner/batch |
|---|---|---|---|---|
| `G01-R-SABER-ARCHER-SOURCE` | `Complete` | Verified source IDs `1014` and `1015` are absent from pinned 4.4 `AvatarConfig`; their pinned fallback identity, skill/Trace/Eidolon IDs and released-text hashes match. Both retain `ExactPreviousRelease`. | [`saber-archer-audit.json`](../../evidence/core-combat-v1/reference-binding/saber-archer-audit.json) | `G01-P0-B2` |
| `G01-R-HIMEKO-NOVA-APPROX` | `Researching` | Ten named cases now cover every prepared `ApproximateFromReleasedText` ability; exact target/operation/timing remains intentionally unresolved until recorded public observations and executable goldens pass. | [`research-cases.json`](../../evidence/core-combat-v1/research-register/research-cases.json), [`fixture-specifications.json`](../../evidence/core-combat-v1/research-register/fixture-specifications.json) | `G01-P7-M01`, before `G01-P7-C04` |
| `G01-R-ELATION-SEMANTICS` | `Researching` | Eight named cases constrain distinct damage/ability tags, Punchline, Certified Banger, forced Elation Skills, Skill Point observation, shared Aha ownership and generic APIs using four released forms. | [`research-cases.json`](../../evidence/core-combat-v1/research-register/research-cases.json), [`decision-records.json`](../../evidence/core-combat-v1/research-register/decision-records.json) | `G01-P4-B8`; close by `G01-P4-B11` |
| `G01-R-V1A-PROBES` | `Researching` | Nineteen named cases cover every Asta, Kafka, Clara, Firefly and Aglaea ownership/timing/formula constraint and bind each to a Phase 4 owner and replayable fixture envelope. | [`research-cases.json`](../../evidence/core-combat-v1/research-register/research-cases.json), [`fixture-specifications.json`](../../evidence/core-combat-v1/research-register/fixture-specifications.json) | `G01-P4-B2` through `G01-P4-B7`; close by `G01-P4-B11` |

An unresolved research case may not be converted into a default implementation
without a documented project-policy decision and regression fixture.

## Decision log

| Date | Decision | Reason |
|---|---|---|
| 2026-07-17 | Goal 01 targets complete core battle plus released character forms, Traces, Techniques, Eidolons and Light Cones. | Establish the first independently playable milestone. |
| 2026-07-17 | Universe families and all three recurring challenge families are excluded. | Prevent activity-specific systems from delaying the core battle milestone. |
| 2026-07-17 | Full relic/planar and public enemy catalogs are excluded; future boundaries remain protected. | They are not part of the requested first content batch. Standard battle uses a frozen representative public-data manifest. |
| 2026-07-17 | Content completeness is manifest-based and requires `DataReady`; behavioral profiles and placeholders do not count. | Make completion auditable and prevent scope inflation. |
| 2026-07-17 | Every batch is committed separately and updates this ledger. | Preserve reviewability, resumption and deterministic progress tracking. |
| 2026-07-17 | Goal 01 binds the prepared Version 4.4 content-reference pack before implementation. | Prevent compact profiles, memory, or ad-hoc websites from becoming the Excel source of truth. |
| 2026-07-17 | Excel workbooks plus pinned Sora output remain the only authoritative production authoring/runtime-data chain; prepared JSON is bootstrap evidence, not a runtime shortcut. | Preserve the formal editable and validated configuration workflow selected for Starclock. |
| 2026-07-17 | Phase 4 interleaves shared-kernel batches with non-production V1a probes compiled from a dedicated Excel/Sora scope. | Make complex released mechanics constrain the Rule IR and lifecycle before bulk import without misreporting partial content as DataReady. |
| 2026-07-17 | Cross-platform CI, property-test scaffolding and performance measurement begin before hardening. | Make Phase 8 consume accumulated evidence instead of creating its prerequisites at the end. |
| 2026-07-17 | Freeze Goal 01 manifest `e2188c7844d678253c98d569db017dbad7101541cf502aba4c2eb80c0435bf19` against reference pack `0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a`. | Establish an immutable machine-readable completeness oracle before workspace or content implementation. |
| 2026-07-17 | Select Silver Wolf LV.999 as the Elation-focused production form in `G01-P7-V1B`. | Reuse the prepared Elation review fixture while Phase 4 still requires at least one additional released form to constrain shared semantics. |
| 2026-07-17 | Freeze six public encounter records, 17 exact variants, and six seeded four-build scenarios as `standard-v1`. | Cover every Goal 01 section 4.4 archetype without claiming or importing the full public enemy catalog. |
| 2026-07-17 | Retain Saber and Archer as `ExactPreviousRelease` records bound to pinned StarRailRes revision `7b349e39ee0f6f3bf814567995829b99c95e7a93`. | Their source IDs are absent from the pinned 4.4 structured AvatarConfig; the audit proves identity and released-text hashes without fabricating 4.4 provenance. |
| 2026-07-17 | Carry the 10 source-bound Himeko Nova mechanism approximations into a named Phase 0 research case. | Evidence hashes validate the released text, but exact operation/target/timing semantics require observations or explicit fixture decisions before production import. |
| 2026-07-17 | Freeze 37 named research cases and reproducible fixture envelopes for V1a, shared Elation and Himeko Nova; retain their state as `Researching`. | Phase 0 must eliminate unnamed ambiguity without pretending that a fixture specification is a completed live observation. |
| 2026-07-17 | Register `G01-P7-M01` as the prerequisite for Himeko Nova's generic Assist Skill/shared-use/companion-protocol mechanism. | The prepared source contains exact numbers/text hashes but no bound configuration operations for ten abilities; the shared primitive and observation goldens must precede production batch `G01-P7-C04`. |
| 2026-07-17 | Freeze initial Goal 01 coverage `05f3539927e298d570a6d128e2b27613f3639620260d16509ab0deb8cda03d69` at 283/283 accounted and 0 `DataReady`. | Prepared reference data, provenance evidence and research fixtures are valuable prerequisites but are not production Excel/Sora runtime content. |
| 2026-07-17 | Replace the root facade with a nine-member virtual Cargo workspace and enforce the exact local graph in both a direct verifier and workspace tests. | Responsibility boundaries must fail mechanically if combat gains peripheral dependencies or an out-of-scope crate appears. |
| 2026-07-17 | Pin Rust/Cargo 1.97.0, Node 24.15.0 and `fixnum =0.9.5`; record every direct/transitive package and tool in the machine-readable dependency policy. | Exact tool/package resolution, license review, deterministic impact and compile cost must be auditable before numeric implementation expands. |
| 2026-07-17 | Make `node tools/repository-check/run.mjs` the single local and future-CI gate, with generated/vendor exclusions and public re-exports allowed only through exact reviewed policy entries. | Prevent local/CI command drift and ensure file-size, visibility, dependency and generated-artifact rules fail mechanically as the workspace grows. |
| 2026-07-17 | Bind `sora-cli 0.3.0` to crates.io archive `90d373102de6a0d7969ebdee51d4ac01ba25c7f2c34661cf581a2c8ead57763a` and golden output digest `3068755483a02de85271dd531c5707a6b8e0e08270a782dd60a34a2b3e965f09`. | Production schema work may rely only on command/type/codegen/export behavior executed by the checksum-bound release. |
| 2026-07-17 | Under Sora 0.3.0, use project paths with a parent, Sora `format = "never"` plus pinned rustfmt, range-validated positive `i32` transport IDs/order fields, and an isolated locked generated-reader fixture. | Executed regression probes show bare clean paths and Windows formatter discovery fail, unsigned Sora Rust fields do not compile, and emitted readers require serde/zstd; domain IDs and production dependency ownership remain protected. |
| 2026-07-17 | Compare generated Excel templates semantically rather than hashing raw `.xlsx` archives. | Sora 0.3.0 workbook ZIP metadata varies across invocations; workbook lists plus read-only no-change synchronization prove schema projection, while schema lock, Rust, diagnostic JSON and `.sora` stay byte-golden. |
| 2026-07-17 | Bind native CI to `windows-2025` x64, `ubuntu-24.04` x64 and `macos-15` ARM64, with paired alternate-CPU `cargo check` profiles and immutable action commits. | The full repository runner and Sora golden execute only on the actual host; compile-only evidence remains mechanically distinct and cannot be promoted into an unexecuted runtime claim. |
| 2026-07-17 | Centralize shared metadata in `ContentIdentity`, bind facts through `SourceRecord`/`EvidenceRecord`, and carry bundle compatibility in singleton `ConfigManifest`. | Later content families can use typed primary-key references while localization and provenance remain non-executable metadata and generated Sora rows stay behind the data boundary. |
| 2026-07-17 | Canonical decimal source fields are strings ending `_decimal`, normalized without redundant zeroes, limited to six fractional digits and checked into signed `i64` millionths without a float conversion. | One machine-readable grammar and integer-only verifier prevent Excel/JSON/locale rounding before Phase 2 domain arithmetic exists. |
| 2026-07-17 | Keep the common-schema TOML rows as disabled `SyntheticFixture` golden input only; defer all production table-source migration and `.xlsx` authoring to `G01-P1-B10`. | B6 can prove Sora structure, references, templates and exporters without creating a non-Excel production runtime shortcut or misreporting partial content. |
| 2026-07-17 | Represent character/build authoring as typed identities plus unique ordered child rows, with one closed Trace/Eidolon patch union and explicit E1-E6 and S1-S5 policies. | Sora can enforce references, bounds and uniqueness now while B8 owns executable Rule IR, B10 owns production Excel, and B11 owns whole-catalog graph/program/completeness validation. |
| 2026-07-17 | Encode Rule IR as referenced relational nodes and closed tagged unions, with finite `If`/bounded-`ForEach` programs, explicit replacement proposals and no recursive cell-local language. | Sora can prove row shape/references while deterministic graph/type/domain validation remains a separately tested catalog-construction responsibility; no JSON/script shortcut or resolver mutation API is introduced. |
| 2026-07-17 | Represent trigger events as a typed event-family union rather than one large enum validation list. | Sora 0.3.0's Excel template writer rejects inline validation lists over Excel's 255-character limit; the family union preserves typed event semantics and the committed golden regression-proves template generation. |

## Terminal acceptance checklist

Change an item to `[x]` only with evidence in this file.

- [x] Required workspace crates compile with enforced dependency direction.
- [x] Pinned dependencies/tools have purpose, license, deterministic-impact and
      compile-cost records; the Sora 0.3.0 golden project proves every relied-on
      command, schema and export capability before production schemas.
- [ ] Fixed-point, RNG, canonical codec and state hashing pass cross-platform
      golden vectors.
- [ ] Canonical hashing streams through the shared encoder, and accepted-command
      transactions reuse bounded battle-local scratch only after legality
      validation while preserving rollback/fault hashes.
- [ ] Core formula, timeline, effect, Toughness, lifecycle and rule suites pass.
- [ ] Asta, Kafka, Clara, Firefly, Aglaea and cross-kit Elation V1a probes pass
      through the production Excel/Sora-to-domain boundary without entering
      production coverage.
- [ ] Standard single-wave, multi-wave, elite and multi-phase boss scenarios run
      from build selection to terminal battle result.
- [ ] CLI configuration validation, coverage, battle run and replay verification
      pass from a clean checkout.
- [ ] Released character combat-form manifest is 100% `DataReady`, including
      abilities, Techniques, Traces and E1-E6.
- [ ] Released Light Cone manifest is 100% `DataReady`, including levels,
      promotions and S1-S5.
- [ ] `standard-v1` enemy, encounter and scenario manifests are 100% `DataReady`.
- [ ] All required bilingual fields, provenance and evidence hashes validate.
- [ ] Sora/Excel export and generated outputs reproduce without drift.
- [ ] Manifest-wide E0/S1 and E6/S5 build compilation passes.
- [ ] Baseline controller decisions and replay hashes are deterministic.
- [x] Committed Windows/Linux/macOS CI workflows distinguish native execution
      from compile-only CPU coverage and retain golden evidence.
- [ ] Versioned Standard/server-verification workloads satisfy the reviewed
      stable-runner budgets for incremental latency, 100/500-command one-shot
      replay throughput, commands/second/core, concurrent isolated jobs, peak
      bytes/job, allocations, state-copy/hash cost and journal growth.
- [ ] Formatting, clippy, workspace tests, source-size and public-API audits pass.
- [ ] No excluded universe, challenge, UI, account or full relic/enemy dataset is
      claimed as part of Goal 01.
- [ ] Clean-checkout acceptance report is committed and linked here.

## Completion record

| Field | Value |
|---|---|
| Final state | Not complete |
| Completion commit | — |
| Catalog digest | — |
| Clean-checkout report | — |
| Cross-platform report | — |
| Remaining required work | All execution phases |
