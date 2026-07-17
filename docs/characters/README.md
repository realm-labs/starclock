# Public Character Mechanics Catalog

This directory turns every publicly disclosed playable combat form into a compact implementation contract. It is not a build guide and does not reproduce in-game prose. Its purpose is to tell a combat-engine implementer which state, commands, events, selectors, and exceptional timing rules each kit requires.

## Snapshot and coverage

- Snapshot date: **2026-07-17**, Version **4.4**.
- **88 released combat forms** are covered, counting each Trailblazer path and each alternate character form separately.
- **2 officially announced combat forms** are covered from their public kit summaries: Rin Tohsaka and Gilgamesh. Their event availability begins on 2026-07-24.
- Total public coverage: **90 combat forms**.
- Unannounced Version 4.5 characters and leaks are out of scope. Eidolons, battle-relevant Techniques, Light Cones, relics, enemies, and permanent universe-mode modifiers belong to the full Version 4.4 data manifest, but are tracked outside these compact E0 behavioral profiles.

The count is by combat form rather than person. For example, both March 7th paths, all five public Trailblazer paths, and the three Dan Heng forms are distinct entries. Male and female Trailblazer models share one mechanics contract per path and are not double-counted.

## Documents

- [Profile schema](schema.md) defines the E0 kit boundary and the vocabulary used by every profile.
- [Profiles A–H](profiles-a-h.md) covers Acheron through Hysilens.
- [Profiles I–R](profiles-i-r.md) covers Jade through Ruan Mei.
- [Profiles S–Z](profiles-s-z.md) covers Saber through Yunli, including every Trailblazer path.
- [Implementation matrix](implementation-matrix.md) is the auditable 90-row roster and engine-feature checklist.

## What “complete” means here

A profile is complete when it records all behavior that changes the engine model at E0:

- the Basic ATK, Skill, Ultimate, Talent, summon/memosprite, enhanced action, and follow-up loop where applicable;
- unusual resources, caps, transformations, zones, countdowns, target marks, linked units, and team resources;
- action advance, extra-turn, interrupt, retarget, death/revive, HP-consumption, Toughness, DoT, Break, shield, healing, or aggro exceptions;
- the minimum reusable primitives needed to express the kit without character checks in the scheduler or damage formula.

Exact trace-level multipliers, minor-stat nodes, Eidolons, animation hit timing, and localized text are **authored balance data**, not engine rules. Store those values in versioned character definitions. Do not hard-code them into Rust systems. A behavioral profile marked Released is therefore not evidence that its executable coefficient data has been imported.

The full-data acceptance boundary and status vocabulary are defined in [Content data and coverage](../15-content-data-and-coverage.md).

## Status labels

- **Released**: behavior is based on a publicly playable kit as of the snapshot.
- **Announced**: only officially disclosed behavior is normative. Unknown hit counts, multipliers, costs, and trigger details must remain nullable or provisional.
- **Observed extension**: a released kit needs a behavior absent from the initial core milestone, such as memosprites, Elation, or party-wide Ultimate activation.

## Source policy

Roster identity and release status are checked against the official Version 4.4 update notice and the maintained character list. Structured metadata from StarRailRes is used as a transcription aid for released kits through Version 4.3. Public player-facing descriptions are paraphrased into implementation semantics; extracted client assets and long verbatim ability text are intentionally excluded.

See [Sources and confidence](../sources.md) for URLs, provenance, and maintenance rules.
The prepared [Version 4.4 content reference pack](../content-reference/README.md)
adds complete numerical/source rows without changing the compact profile
boundary. Reference readiness is still not Sora/Rule-IR `DataReady`.
