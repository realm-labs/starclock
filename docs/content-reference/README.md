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

- 88 released character combat forms;
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
`ExactPreviousRelease`, not an approximation or leak. Announced but unavailable
Rin Tohsaka and Gilgamesh are not enabled in the 88-form reference baseline.

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
