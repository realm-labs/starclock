# Goal 01 Status — Complete Core Combat and Released Character Content

This file is the persistent execution ledger for
[Goal 01](01-core-combat-and-content.md). The executor must update it in the same
commit as every implementation or content batch.

## Goal state

| Field | Value |
|---|---|
| Goal ID | `core-combat-v1` |
| State | `InProgress` |
| Active phase | Phase 0 — Freeze scope and evidence |
| Next unblocked batch | `G01-P0-B2` |
| Last completed batch | `G01-P0-B1` |
| Last completed commit | This ledger row's containing commit; verified after commit |
| Goal plan baseline | Bound to frozen manifests; Phase 0 execution in progress |
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
| Phase 0 — Freeze scope and evidence | `InProgress` | `G01-P0-B1` froze and verified manifest `e2188c7844d678253c98d569db017dbad7101541cf502aba4c2eb80c0435bf19`; `G01-P0-B2` next. |
| Phase 1 — Workspace and reproducible data foundation | `Pending` | None |
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
| `G01-P0-B1` | `Complete` | This row's containing commit | `node tools/content-reference/verify.mjs content-reference/v4.4`; `node tools/goal-manifest/generate.mjs --check`; `node tools/goal-manifest/verify.mjs`; `cargo fmt --all -- --check`; `cargo test`; `git diff --check`. Frozen evidence: [`manifest-index.json`](../../content-manifests/core-combat-v1/manifest-index.json), [`standard-v1.json`](../../content-manifests/core-combat-v1/standard-v1.json), [`partitions.json`](../../content-manifests/core-combat-v1/partitions.json). | Bound reference pack `0dca8ae5…f5a`; froze goal manifest `e2188c78…f19`, 88 forms, 165 Light Cones, 17 exact enemy variants, 6 encounters and 6 scenarios. |
| `G01-P0-B2` | `Pending` | — | — | Provenance staging and evidence hashes. |
| `G01-P0-B3` | `Pending` | — | — | Blocking research cases, including Elation and V1a probes. |
| `G01-P0-B4` | `Pending` | — | — | Initial generated coverage. |
| `G01-P1-B1` through `G01-P1-B11` | `Pending` | — | — | Workspace, CI, Sora capability proof, schema families and reproducible pipeline. Expand before Phase 1 starts. |
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
| `G01-P7-Mnn` | `Pending` | — | — | Register known mechanic batches during Phase 0; add newly discovered prerequisites before dependent content. |
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
| `G01-R-SABER-ARCHER-SOURCE` | `Pending` | Verify the pinned 4.3 fallback and 4.4 manifest identity mapping for Saber and Archer without weakening released-content provenance. | Source revisions/hashes, identity mapping and discrepancy report. | `G01-P0-B2` |
| `G01-R-ELATION-SEMANTICS` | `Pending` | Define shared Elation damage, Elation Skill, Punchline, Certified Banger, forced-action and shared-actor/resource semantics from more than one released form. | Cross-kit evidence, decision record and probe fixture specification. | `G01-P0-B3` |
| `G01-R-V1A-PROBES` | `Pending` | Identify every unresolved timing/ownership question needed by the Asta, Kafka, Clara, Firefly and Aglaea mechanism probes. | Named per-mechanic cases with reproducible observation or golden fixture specifications. | `G01-P0-B3` |

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

## Terminal acceptance checklist

Change an item to `[x]` only with evidence in this file.

- [ ] Required workspace crates compile with enforced dependency direction.
- [ ] Pinned dependencies/tools have purpose, license, deterministic-impact and
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
- [ ] Committed Windows/Linux/macOS CI workflows distinguish native execution
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
