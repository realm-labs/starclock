# Standard Simulated Universe Reference-Data Contract

This document defines the pre-runtime reference-data boundary for the permanent
main-world Simulated Universe profile. It complements the generic architecture
in [Universe mode profiles](14-run-core-and-universe-modes.md); it does not claim
that the universe runtime is implemented.

## Frozen snapshot

- game/content snapshot: Version 4.4;
- research access date: 2026-07-22;
- structured released-data baseline: `Dimbreath/turnbasedgamedata` commit
  `fd978d6ef09f941fba644c731ab54abd6f7c3568`;
- identity/translation cross-check: `Mar-7th/StarRailRes` commit
  `7b349e39ee0f6f3bf814567995829b99c95e7a93` where applicable;
- public topology cross-check: nine main Worlds and nine selectable Paths.

The snapshot is historical and reproducible. A later live update creates a new
manifest revision; it does not silently mutate this one.

## Included content

Completeness is defined by frozen manifests for:

1. Worlds, difficulties, areas, maps, layers, rooms and domain kinds;
2. run-entry, path selection, path bans, progression and terminal rules;
3. the nine main-world selectable Paths;
4. Path passives, Resonances and Resonance Formations available in main Worlds;
5. Blessings, rarity, Path, prerequisite/pool rules and enhanced values;
6. Curios, negative/error/special states, charges, replacement and repair;
7. Occurrences, choices, conditions and mechanical outcomes;
8. Cosmic Fragments, reward/reroll/enhance prices and deterministic services;
9. Combat, Elite, Boss, Occurrence, Transaction, Respite and Downloader nodes;
10. shops, revival, character download and blessing enhancement services;
11. battle-affecting Ability Tree nodes and their prerequisite graph;
12. main-world monster groups, exact enemy variants, waves and boss pools;
13. every battle-visible rule contribution and cross-battle state slot;
14. bilingual names/original summaries and row-level provenance.

Shared content introduced by an expansion is included only when the frozen
main-world pool can actually offer it. DLC-only dice, countdowns, Cognition,
Scepters, Equations, Weighted Curios, mode-only occurrences and resonance
interplays are excluded even when they reuse a base table.

## Excluded content

- Swarm Disaster, Gold and Gears, Unknowable Domain and Divergent Universe
  profile mechanics;
- historical Planar Infinity and other temporary event variants;
- weekly/account/first-clear rewards, Stellar Jade, achievements and index
  collection payouts;
- Planar Ornament item definitions and account inventory extraction;
- overworld movement, dialogue presentation, assets, audio and minigame input;
- runtime implementation of `starclock-mode-universe`.

Reward and dialogue rows may be retained as provenance locators when necessary
to prove a mechanical choice. They do not become authored runtime content.

## Evidence policy

Evidence priority follows [Sources and confidence](sources.md). The released
structured source supplies exact IDs, numeric parameters, relationships and
TextMap hashes. Public game/Wiki pages cross-check player-visible membership and
meaning. No single community page is a completeness oracle.

Each normalized fact records:

```text
source_record_id
source_repository_or_url
source_revision_or_access_date
source_relative_path_or_page
source_row_locator
evidence_sha256
quality
mechanism_quality
note
```

Allowed quality labels are `ExactStructured`, `ExactPublicText`, `Observed`,
`ApproximateFromReleasedText` and `ProjectPolicy`. Approximation is field-level.
An unresolved hidden rule receives a deterministic proposed policy and a
replacement condition; it is never presented as observed game behavior.

Long descriptions, source programs and assets are not committed. English and
Simplified Chinese descriptions are independently summarized while exact
names, factual numbers and relationships are preserved.

## Normalized reference families

The staging pack under `content-reference/standard-universe-v1/` uses stable
Starclock IDs and canonical JSON only as a reproducible research/bootstrap
artifact:

```text
manifest.json
coverage.json
pack-index.json
sources.json
worlds.json
world-difficulties.json
domains.json
paths.json
resonances.json
blessings.json
curios.json
occurrences.json
services.json
ability-tree.json
encounter-pools.json
mechanic-rules.json
review-fixtures.json
```

The exact record envelope, per-family fields, canonical scalar grammar and
fixture shape are normative in
[Standard Simulated Universe normalized-data design](24-standard-universe-normalized-data.md)
and its machine-readable `schema.json`.

Source-project IDs remain provenance locators and are never runtime identities.
All authoritative decimals are canonical strings. Arrays whose order is not
semantic are sorted by stable ID before hashing.

## Excel and Sora promotion

Production authoring uses `config/data/Universe.xlsx` plus focused Activity and
Rule bindings. Python `openpyxl` is the approved Goal 03 workbook authoring and
inspection adapter. The script regenerates a complete initial workbook from the
normalized pack and never patches a designer-edited workbook.

Sora 0.3.0 remains the only schema validation, code-generation and export
authority. Runtime code must never load the normalized JSON pack or `.xlsx`
directly. Goal 03 prepares generated readers and validates bundle rows, but does
not implement universe runtime lowering.

The prepared Universe tables use the isolated `config/universe-project.toml`
project and `config/universe-generated/` output root. Its `config.sora` is an
authoring/review artifact, not the runtime `config/generated/config.sora`.
Keeping the bundles separate preserves the Goal 01/02 catalog digest and makes
it impossible for non-empty Universe rows to enter `starclock-data` before a
reviewed domain lowering exists. The standalone
`tools/universe-bundle-loader` compiles the isolated generated readers and
proves binary decoding without exposing those rows to a runtime crate.
[Goal 04](goals/04-standard-universe-runtime.md) must deliberately introduce
the domain conversion and compatibility migration before consuming this
staging bundle.

Workbook rows retain stable IDs, bilingual summaries, coverage state and exact
evidence references. Editable categories use data validation. Headers are
frozen, tables have filters/frozen panes, exact numeric text remains text, and
all sheets receive an `openpyxl` load/formula/error check plus a visual render
inspection before acceptance.

## Completeness gates

An entry is complete only when:

- its stable identity belongs to the frozen denominator;
- required bilingual fields and provenance resolve;
- numeric vectors and pool membership are present or explicitly approximate;
- every referenced Path, rule, encounter, service and state resolves;
- main-world versus DLC-only membership is proven;
- its Excel row validates through Sora and regenerates without drift;
- a semantic fixture covers every distinct shared mechanic family.

The terminal report requires 100% manifest accounting, zero enabled incomplete
rows and no unresolved blocking research case. `DataReady` means usable authoring
data; it does not claim that the future runtime executes the mechanic.

## Initial public cross-checks

- [Simulated Universe overview](https://honkai-star-rail.fandom.com/wiki/Simulated_Universe)
- [Main Worlds](https://honkai-star-rail.fandom.com/wiki/Simulated_Universe/Worlds)
- [Paths, Blessings and Resonances](https://honkai-star-rail.fandom.com/wiki/Simulated_Universe/Paths)
- [Curios](https://honkai-star-rail.fandom.com/wiki/Simulated_Universe/Curio)

These maintained community pages are discovery/cross-check sources, not a
substitute for the pinned structured rows and row-level evidence hashes.
