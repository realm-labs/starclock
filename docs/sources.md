# Sources and Confidence

## Source policy

HoYoverse explains player-facing concepts in tutorials and ability text but does not publish a complete executable combat specification. HoYoLAB is an official platform, yet user articles hosted there are still community research. Wiki formula pages and theorycrafting guides are therefore secondary sources, even when mature and well tested.

For this project:

- **Verified** means independently consistent sources and/or worked observed results.
- **Observed** means a maintained community source reports the rule, but systematic independent confirmation is not included here.
- **Project policy** means an intentional deterministic choice, not a claim about the original game.

## Main references

| Source | Used for | Confidence and caveat |
|---|---|---|
| [Star Rail Wiki — Damage](https://honkai-star-rail.fandom.com/wiki/Damage) | General damage blocks, DEF/RES/vulnerability/mitigation, hit split | High for core formulas; community-maintained and can change. |
| [Star Rail Wiki — Damage RES](https://honkai-star-rail.fandom.com/wiki/Damage_RES) | Effective RES formula, bounds, and current enemy defaults | High for the current community model; modes and enemies override defaults. |
| [Prydwen — Damage Formula](https://www.prydwen.gg/star-rail/guides/damage-formula) | Independent cross-check and worked results; common RES defaults/bounds | Good independent confirmation; some explanatory examples originate from early versions. |
| [Star Rail Wiki — Toughness](https://honkai-star-rail.fandom.com/wiki/Toughness) | Toughness reduction, Break elements, Break damage, level table, Super Break | High for current community model; newer extensions are version-sensitive. |
| [KQM — Speed Guide](https://hsr.keqingmains.com/misc/speed-guide/) | Action Gauge, AV, advance/delay, Break timing | High-quality theorycrafting reference; formula behavior is also consistent with the wiki. |
| [Star Rail Wiki — Speed](https://honkai-star-rail.fandom.com/wiki/Speed) | Turn lifecycle, duration timing, immediate action, cycles, enemy SPD scaling | High for the described baseline; exact ties and special bosses remain incomplete. |
| [Star Rail Wiki — Effect Hit Rate](https://honkai-star-rail.fandom.com/wiki/Effect_Hit_Rate) | Real debuff chance | High for the common formula. |
| [Star Rail Wiki — Skill Point](https://honkai-star-rail.fandom.com/wiki/Skill_Point) | Default start/cap and common gain/spend behavior | High for defaults; character exceptions are numerous. |
| [Star Rail Wiki — Energy](https://honkai-star-rail.fandom.com/wiki/Energy) | Common generation values and ERR behavior | High for defaults; every ability must carry authored values/flags. |
| [Star Rail Wiki — Shield](https://honkai-star-rail.fandom.com/wiki/Shield) | Shield formula, overflow, non-stacking simultaneous depletion | High for ordinary shields; kit-specific stacking overrides exist. |
| [Star Rail Wiki — Outgoing Healing Boost](https://honkai-star-rail.fandom.com/wiki/Outgoing_Healing_Boost) | Healing formula | High for ordinary healing. |
| [Star Rail Wiki — Aggro](https://honkai-star-rail.fandom.com/wiki/Aggro) | Weighted targeting and path weights | Medium; the page explicitly labels aggro terminology/data community-derived. |
| [Star Rail Wiki — Follow-Up Attack](https://honkai-star-rail.fandom.com/wiki/Follow-Up_Attack) | Follow-up/counter semantics and priority | High for baseline; individual kit retarget rules remain authored data. |
| [Star Rail Wiki — Extra Turn](https://honkai-star-rail.fandom.com/wiki/Extra_Turn) | Extra turn priority, no AG movement, duration behavior | High for baseline; special extra actions may differ. |
| [Japanese WikiWiki — Damage Calculation](https://wikiwiki.jp/star-rail/%E3%83%80%E3%83%A1%E3%83%BC%E3%82%B8%E8%A8%88%E7%AE%97%E5%BC%8F) | Current cross-check for damage, stat, Super Break, and incoming damage formulas | Useful independent current-language reference; community-maintained. |

## Configuration tooling references

| Source | Used for | Confidence and caveat |
|---|---|---|
| [realm-labs/sora](https://github.com/realm-labs/sora) | Excel `.xlsx` input, schema validation, Rust code generation, binary/debug exports, manifest-driven build commands, and repository layout | Primary project documentation. Sora is explicitly early-stage, so the project pins the CLI and treats upgrades as migrations. |
| [Sora — Versioning and Compatibility](https://realm-labs.github.io/sora/versioning.html) | CLI pinning, generated-output consistency, schema locks, bundle versions, and upgrade procedure | Primary project documentation. It states that old schema semantics are not retained behind compatibility editions. |
| [Sora v0.3.0 release](https://github.com/realm-labs/sora/releases/tag/v0.3.0) | Initial pinned CLI version for this architecture snapshot | Primary release record. Re-check before an intentional toolchain upgrade; do not float to latest in CI. |
| [`fixnum` 0.9.5](https://docs.rs/fixnum/0.9.5/fixnum/) | Pinned decimal fixed-point implementation, checked arithmetic, explicit rounding operations, wide multiplication/division intermediates, and domain wrapper support | Primary crate documentation. The dependency is hidden behind project domain types; upgrades require numeric-policy and cross-platform replay review. |
| [`rand` 0.10.2](https://docs.rs/rand/0.10.2/rand/) | Pinned ChaCha8 generator exposure and feature selection | Primary crate documentation. Project-owned integer range/weight mapping remains part of the replay revision; generic distributions are not authoritative. |
| [`sha2` 0.11.0](https://docs.rs/sha2/0.11.0/sha2/) | SHA-256 implementation for bundle digests, stream derivation, state hashes, and replay verification | Primary crate documentation. The canonical byte layout is project-owned and independently versioned. |
| [`rust_xlsxwriter` 0.96.0](https://docs.rs/rust_xlsxwriter/0.96.0/rust_xlsxwriter/) | Deterministic bootstrap generation of complete initial `.xlsx` workbooks | Primary crate documentation. It is not the schema authority and does not patch designer-maintained workbooks. |

## Universe-mode references

| Source | Used for | Confidence and caveat |
|---|---|---|
| [HoYoLAB — Swarm Disaster overview](https://www.hoyolab.com/article/21275174) | Permanent availability, map/domains, Audience Dice, Communing Device/Trail, Pathstrider progression, and headline content | Primary publisher announcement for the mode boundary; exact dice, blessing, occurrence, and combat values need row-level evidence. |
| [HoYoLAB — Gold and Gears overview](https://www.hoyolab.com/article/23850968) | Permanent availability, updated blessings/curios/occurrences/resonances, and mode identity | Primary publisher announcement; detailed custom-dice, Cognition, Secrets, and Conundrum tables require public in-game/community transcription. |
| [HoYoLAB — Unknowable Domain overview](https://www.hoyolab.com/article/34422433) | Permanent availability, four Extrapolation Alignments, Scepters, Components, and stages | Primary publisher announcement for system identity; exact activation, slot, synthesis, and effect values require row-level verification. |
| [HoYoLAB — Divergent Universe: Protean Hero update](https://www.hoyolab.com/article/38864547) | Equations/Curios/Weighted Curios/Occurrences, Golden Blood's Boon, Stable Computing Arrays, and Astronomical Division update | Primary publisher announcement for that revision. The Version 4.4 manifest must record subsequent live adjustments instead of assuming this article is a complete current table. |

## Standard and challenge-mode references

| Source | Used for | Confidence and caveat |
|---|---|---|
| [HoYoLAB — Version 1.0 boarding overview](https://www.hoyolab.com/article/17260093) | Official Forgotten Hall distinction between Memory and Memory of Chaos, cycle limits, periodic updates, and cycle-triggered effects | Primary publisher overview for the stable family model. Current stage counts, thresholds, enemies, and turbulence are rotating data. |
| [HoYoLAB — Pure Fiction overview](https://www.hoyolab.com/article/24179984) | Two nodes/two teams, Whimsicality, selectable Cacophony, continuous enemies, limited Cycles, damage/defeat points, and summed node score | Primary publisher announcement. Exact active Version 4.4 spawn, score, clock, threshold, and buff rows require snapshot-specific evidence. |
| [HoYoLAB — Apocalyptic Shadow overview](https://www.hoyolab.com/article_pre/15111) | Two nodes, selectable Finality's Axiom per team, boss traits, Action Value timer, and boss-progress/remaining-AV scoring | Primary publisher announcement. Exact active Version 4.4 boss variants, AV values, thresholds, Ruinous Embers, and Axiom effects require snapshot-specific evidence. |
| [HoYoLAB — Version 2.3 update](https://www.hoyolab.com/article_pre/15243) | Publisher confirmation that Memory of Chaos, Pure Fiction, and Apocalyptic Shadow are the coordinated rotating challenge set | Primary release note; reward amounts and UI/rotation behavior are outside the simulation scope. |

## Character catalog references

| Source | Used for | Confidence and caveat |
|---|---|---|
| [HoYoLAB — Version 4.4 update details](https://www.hoyolab.com/article/45851903) | Version boundary; official paths/elements and public core summaries for Himeko • Nova, Rin Tohsaka, and Gilgamesh; July 24 collaboration availability | Primary publisher announcement. It is authoritative for disclosed behavior but is not a full numerical kit specification. |
| [Star Rail Wiki — Character List](https://honkai-star-rail.fandom.com/wiki/Character/List) | Public roster and alternate combat-form counting through Version 4.3 | Maintained community index. Its snapshot listed 87 playable forms before Himeko • Nova released; the catalog's 88 released count is the explicit inference `87 + Himeko • Nova` from the official 4.4 notice. |
| [Mar-7th/StarRailRes](https://github.com/Mar-7th/StarRailRes) | Structured English identity and ability metadata used to cross-check the 87 released forms through Version 4.3 | Community-maintained transcription/resource index, not an official API. Used only as a research aid; extracted assets and bulk verbatim descriptions are not included. |
| [Honey Hunter — Himeko • Nova](https://starrail.honeyhunterworld.com/himeko-nova-character/?lang=EN) | Cross-check of 4.4 ability structure, Assist Skill, field, shared uses, and special resource interactions | Unofficial and version-sensitive. Prefer live in-game text when importing numerical data. |
| [Honey Hunter — Rin Tohsaka](https://starrail.honeyhunterworld.com/rin-tohsaka-character/?lang=EN) | Preview cross-check for Gem Energy and enhanced-Skill structure | Unofficial preview data. The character is marked Announced and all undisclosed details remain provisional until public availability. |

The character profiles paraphrase behavior into engine semantics. They do not claim exact parity for coefficients, animation hit timing, minor Traces, Eidolons, or undisclosed preview values. Those require a separately licensed/authored data pack and release-version validation.

## Known uncertainties

Do not present these as exact parity until tested against captured game observations:

- internal numeric precision and rounding at every damage/resource boundary;
- universal tie-breaking when multiple actors have exactly equal AV;
- interrupt order when several Ultimates are requested at the same opportunity;
- snapshot rules for every DoT, field, summon, delayed hit, heal, and shield;
- exact trigger ordering for all character-specific passives and boss phase transitions;
- retarget behavior after a target dies during every multi-hit or queued action;
- how attempted versus effective Toughness reduction is consumed by every tally effect;
- current-version limits for RES, mitigation, and unusual negative/overflow stats;
- mode-specific replacements for Energy, cycles, defeat, or wave rules.
- complete Version 4.4 row catalogs and exact values for all enemies, equipment, and permanent universe-mode content; these remain a data-import task until coverage reports mark them `DataReady`.
- active Version 4.4 stage, clock, spawn, score, objective, and rotating-buff rows for Memory of Chaos, Pure Fiction, and Apocalyptic Shadow.

The architecture documents deliberately turn these into explicit authored policies and test points.

## Maintenance checklist

When updating the rules:

1. record the access date and, when identifiable, game version;
2. prefer a player-facing official tutorial or ability description for behavior;
3. require a maintained formula reference for hidden calculations;
4. cross-check surprising values against an independent source or reproducible test;
5. update `rules_revision`, reference constants, and golden tests together;
6. preserve old replay compatibility or explicitly migrate/reject it;
7. mark unresolved conflicts instead of choosing the more convenient number.
