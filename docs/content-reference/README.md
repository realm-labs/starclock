# Version 4.4 Combat Content Reference Pack

## Purpose

This pack closes the gap between compact mechanic profiles and future Excel/Sora
authoring. Goal 01 must not research identities, copy arbitrary web tables, or
invent a different content model while it is implementing the combat runtime.
It starts from this reviewed baseline instead.

The pack is a transcription and review artifact, not runtime configuration. Its
stable Starclock keys are descriptive and independent from source-project IDs.
Source IDs remain only as provenance locators so a reviewer can find the exact
released row that supports a fact.

## Current baseline

The generated Version 4.4 reference contains:

- 88 character combat forms in the frozen executable reference baseline;
- 583 character ability families, including level curves and pre-battle skills;
- 1,618 Trace nodes/level records;
- exactly 528 Eidolons, six per released combat form;
- 165 released Light Cones with promotion and S1-S5 data;
- 613 mechanically distinct enemy templates;
- 2,591 enemy variants with exact multipliers, weaknesses and resistances;
- 3,611 enemy abilities;
- 1,471 deduplicated ordinary Mainline/Calyx/Farm encounter compositions.

The exact generated counts and confidence labels live in
`content-reference/v4.4/coverage.json`. The pack digest lives in
`content-reference/v4.4/pack-index.json`.

Representative [content review fixtures](review-fixtures.md) turn the normalized
facts into semantic invariants for the first runtime golden tests.

## Evidence layers

No public publisher source exposes a complete executable combat specification.
The reference pack therefore keeps three evidence layers separate:

1. **Released structured facts** supply statistics, level curves, numeric
   parameters, weaknesses, resistances, source skill sequences, target metadata,
   ability entry names and source configuration paths.
2. **Project mechanic contracts** supply independently worded character loops,
   ownership, lifecycle and engine-boundary interpretations.
3. **Observation fixtures** later resolve hidden ordering, snapshotting,
   retargeting and exceptional AI behavior that released tables do not prove.

A source text hash proves which released description was reviewed without
committing that description. The pinned local source cache remains ignored. A
reviewer can reproduce it using `tools/content-reference/fetch-sources.ps1`.

## Source baseline

| Source | Pinned revision | Use | Limitation |
|---|---|---|---|
| `Dimbreath/turnbasedgamedata` | `fd978d6ef09f941fba644c731ab54abd6f7c3568` | Released 4.4 character, Light Cone, enemy, stage, AI and ability configuration facts | Community-maintained release-data transcription; no license grant is assumed, so raw files and prose are not redistributed. |
| `Mar-7th/StarRailRes` | `7b349e39ee0f6f3bf814567995829b99c95e7a93` | Released 4.3 structured cross-check and licensed-collaboration fallback for Saber and Archer | Community resource index under its repository license; not an official API. |
| Starclock character profiles | Goal-document revision | Original character behavior and engine-contract summaries | High-level behavior only; exact values come from the structured evidence layer. |

Saber and Archer use the pinned 4.3 release index because their collaboration
records are not present in the pinned 4.4 release dump. This is explicit
`ExactPreviousRelease`, not an approximation or leak. Rin Tohsaka and Gilgamesh
released after this pack was frozen and are not enabled in its 88-form baseline;
their release status does not retroactively change the pack's generated counts.

## What is and is not copied

Committed generated records contain factual numbers, names, categories,
relationships, source paths and hashes. They do not contain art, audio, models,
story text, icons, raw ability programs or bulk ability descriptions.

The operation-type summaries are derived inventories such as “apply modifier,”
“deal damage,” “summon” or “advance action.” They help an implementer select the
correct Rule IR primitive, but are not executable source code and do not define
an undocumented order by themselves.

## Relationship to Goal 01

Goal 01 consumes this pack through a controlled promotion step:

```text
pinned released evidence
        |
        v
normalized reference pack + mechanic contracts
        |
        v
reviewed transcription rows
        |
        v
Excel workbooks --Sora--> validated Starclock catalogs
```

Goal 01 may improve a fact when an observation fixture or stronger public source
proves a difference. It must preserve the old evidence, record the decision and
update the pack/catalog digests. It may not silently replace a value because a
different website is easier to transcribe.

## Gold and Gears Candidate reference

Goal 08 freezes a separate Version 4.4 Gold and Gears reference package under
`content-reference/gold-and-gears-v1/`. Its 7,913 source obligations are fully
DataReady, while all 16 unpublished evidence boundaries remain explicit,
nonblocking and replaceable. The four isolated authoring workbooks and the
Candidate Sora bundle are review artifacts only: no JSON/Excel runtime path,
runtime lowering, handler registration or playable profile is released.

The normalized Candidate pack digest is
`ea2f3a35807b9a7dae39be2d67fb5de955bfad7852718eb1d3393affed5a5623`;
the isolated Sora review bundle digest is
`97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b`.
Exact counts, evidence and the remaining runtime boundary are recorded in the
[Goal 08 ledger](../goals/08-gold-and-gears-reference-data-status.md).

## Swarm Disaster Candidate reference

Goal 09 freezes a separate Version 4.4 Swarm Disaster reference package under
`content-reference/swarm-disaster-v1/`. Its 6,963 manifest obligations are
fully `DataReady`; 31 unavailable facts remain explicit, nonblocking and
replaceable. The four isolated authoring workbooks and Candidate Sora bundle
are review artifacts only: no JSON/Excel runtime path, runtime lowering,
handler registration or playable profile is released.

The normalized Candidate pack digest is
`82f3ffc444a1cdcd8bcba5a946bee3a3c8d58527b93a1c9d77f285697401b2d8`;
the isolated Sora review bundle digest is
`385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362`.
Exact counts, evidence and the remaining runtime boundary are recorded in the
[Goal 09 ledger](../goals/09-swarm-disaster-reference-data-status.md).

## Unknowable Domain Candidate reference

Goal 10 freezes a separate Version 4.4 Unknowable Domain reference package
under `content-reference/unknowable-domain-v1/`. Its 5,377 source obligations
are fully `DataReady`; all unavailable weights, ordering, timing and fallback
semantics remain explicit, nonblocking and replaceable. The three isolated
authoring workbooks and Candidate Sora bundle are review artifacts only: no
JSON/Excel runtime path, runtime lowering, handler registration or playable
profile is released.

The normalized Candidate pack digest is
`f48f264fb55221e2494156c5ab7911719d703ec47f492c9c0e2d7fd2c8123b28`;
the isolated Sora review bundle digest is
`05114105b6d905c2858865df08d7ab551cb0fb056b3871b959897a4a590451ec`.
Exact counts, evidence and the remaining runtime boundary are recorded in the
[Goal 10 ledger](../goals/10-unknowable-domain-reference-data-status.md).

## Divergent Universe Candidate reference

Goal 11 freezes a separate Version 4.4 Divergent Universe reference package
under `content-reference/divergent-universe-v1/`. Its 6,215 source obligations
are fully `DataReady`; all 25 unavailable facts remain explicit, nonblocking
and replaceable. The three isolated authoring workbooks and Candidate Sora
bundle are review artifacts only: no JSON/Excel runtime path, runtime lowering,
handler registration or playable profile is released.

The normalized Candidate pack digest is
`74234f3f689db6ba897d13865e079a3404ab707d3ddd978d646390e7b50bad02`;
the isolated Sora review bundle digest is
`3221d0965292de6bbbd834338c2ff088821200ea22a4b7e7c65afc996444c5cf`.
Exact counts, evidence and the remaining runtime boundary are recorded in the
[Goal 11 ledger](../goals/11-divergent-universe-reference-data-status.md).

## Currency Wars Candidate reference

Goal 12 freezes a separate Version 4.4 Currency Wars reference package under
`content-reference/currency-wars-v1/`. Its 19,250 obligations resolve as
18,524 eligible `DataReady` rows plus 726 explicit exclusions, with no
unresolved row. The three isolated authoring workbooks and Candidate Sora
bundle are review artifacts only and do not enable a playable profile.

The normalized Candidate pack digest is
`6166401347306cc38f5f0e3eed1a519d25a1f015e88f7782fb0c1bdf2761c2cb`;
the isolated Sora review bundle digest is
`a4569997990727739db74a2d942e6b13a84d2466b0fe3723acb92c7406ae8571`.
Exact counts, evidence and the remaining runtime boundary are recorded in the
[Goal 12 ledger](../goals/12-currency-wars-reference-data-status.md).

## Anomaly Arbitration Candidate reference

Goal 13 freezes a separate Version 4.4 Anomaly Arbitration reference package
under `content-reference/anomaly-arbitration-v1/`. Its 392 obligations comprise
76 mode-owned and 316 shared records, all `DataReady`; the normalized pack has
2,103 rows and zero runtime-executable rows. The three isolated authoring
workbooks and Candidate Sora bundle remain review artifacts only.

The normalized pack-index digest is
`923394ff72bddcc86318363e2ef248ee2d47ec05e19b60001aa3f7c1bd7dbdf3`;
the isolated Sora review bundle digest is
`a646b66ad0eae515a624d838ea8574f52c6e40588f88127f281b9cd8c40f89f1`.
Exact counts, evidence and the remaining runtime boundary are recorded in the
[Goal 13 ledger](../goals/13-anomaly-arbitration-reference-data-status.md).

## Pure Fiction Candidate reference

Goal 15 freezes `content-reference/pure-fiction-v1/`: 796/796 DataReady
obligations, 6,014 normalized rows, 606 shared receipts, 25 rules and 18
fixtures. Its three workbooks and 37-table Sora bundle remain review-only.
Exact release evidence is recorded in the
[Goal 15 ledger](../goals/15-pure-fiction-reference-data-status.md).

## Memory of Chaos Candidate reference

Goal 17 freezes `content-reference/memory-of-chaos-v1/`: 477/477 DataReady
obligations, 1,521 normalized rows, 305 shared reconciliation receipts, 29
nonblocking policy boundaries and 18 semantic families. Its three workbooks
and 27-table Sora bundle remain review-only. Exact release evidence is recorded
in the [Goal 17 ledger](../goals/17-memory-of-chaos-reference-data-status.md).

## Apocalyptic Shadow Candidate reference

Goal 18 freezes `content-reference/apocalyptic-shadow-v1/`: 129/129
obligations, 1,246 DataReady normalized rows, 81 shared receipts and 42
fixtures. Its three workbooks and 35-table Sora bundle remain review-only.
Exact release evidence is recorded in the
[Goal 18 ledger](../goals/18-apocalyptic-shadow-reference-data-status.md).

## Fate/Star Rail Night Candidate reference

Goal 19 freezes a separate Version 4.4 Fate/Star Rail Night reference package
under `content-reference/fate-star-rail-night-v1/`. Its 1,904 obligations are
fully accounted: 1,491 are eligible DataReady, 413 are EvidenceOnly and
thirteen exact identities are conservatively policy-bound with released-source
replacement conditions. The normalized pack contains 2,018 records, 1,914
source receipts, six exact-zero selector proofs, 58 semantic fixtures and zero
runtime-executable profiles. Four isolated authoring workbooks and the
48-table Candidate Sora bundle remain review artifacts only.

The complete reference tree digest is
`edfd1fd99eac92b89e78fffbafe2fd9e4f1fcefc7481bafbea583b80c797e68f`;
the isolated Sora review bundle digest is
`f2897da1190ebfe5d6634982382b1bcd5eadcda50b2a050ef1be247b78343336`.
Exact counts, evidence and the remaining runtime boundary are recorded in the
[Goal 19 ledger](../goals/19-fate-star-rail-night-reference-data-status.md).

## Merged Candidate integration

The generated
[`merged-mode-audit.json`](../../evidence/reference-integration-v1/merged-mode-audit.json)
binds the final Goal 08-13 completion commits after merge. It verifies all
46,110 manifest obligations, the complete 15-pair reconciliation chain, zero
factual evidence conflicts and zero runtime-enabled Candidate modes. Historical
release evidence remains unchanged; the integration audit records current-tree
compatibility separately.

The additional
[`high-priority merged audit`](../../evidence/high-priority-reference-integration-v1/merged-mode-audit.json)
binds Goals 15 and 17-19 after merge. It verifies 3,306 obligations and all six
mode pairs, preserves literal provenance while adding canonical upstream-key
comparison, records zero factual conflicts and keeps all four runtime profiles
disabled. Three unqualified cross-mode policy IDs and eight intentional Pure
Fiction materialized-view aliases remain explicit pre-runtime coordination
items rather than silently merged identities.
