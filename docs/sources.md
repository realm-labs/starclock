# Sources and Confidence

## Source policy

HoYoverse explains player-facing concepts in tutorials and ability text but does not publish a complete executable combat specification. HoYoLAB is an official platform, yet user articles hosted there are still community research. Wiki formula pages and theorycrafting guides are therefore secondary sources, even when mature and well tested.

For this project:

- **Verified** means independently consistent sources and/or worked observed results.
- **Observed** means a maintained community source reports the rule, but systematic independent confirmation is not included here.
- **Project policy** means an intentional deterministic choice, not a claim about the original game.

### Evidence priority and upstream admission

Use the strongest available evidence for each fact, in this order:

1. released in-game text or behavior, HoYoWiki, and publisher-authored HoYoLAB
   announcements;
2. a pinned revision of released structured data with an exact source path and
   file hash;
3. reproducible observations, including reviewed KQM Evidence Vault submissions
   and Starclock golden fixtures;
4. independently maintained formula wikis and theorycrafting guides;
5. an explicitly approved project policy that remains labeled as such.

HoYoLAB hosting alone does not make a user article official. A repository mirror
or scraped database derived from the same underlying dump is not an independent
cross-check. Player-profile APIs may confirm displayed builds, but they are not
evidence for hidden combat rules.

An admitted repository source records its remote, exact commit, relative path
and content hash. An admitted web source records its canonical URL or stable
entry/revision identifier, access date, released game version, confidence and a
content hash when the page has no immutable revision. Dynamic pages may support
short paraphrased facts; their bulk text is never vendored.

Beta, preview-dump, leak, NDA-bound or otherwise unreleased material is forbidden
even when publicly reachable. An unresolved question does not block Goal 01: after
a bounded evidence search, record the gap and adopt an explicit deterministic
`Approximate` or `ProjectPolicy` decision. It must preserve every known mechanic,
state its rationale and replacement conditions, and remain distinguishable from
an observed game fact. The implementation blocker then becomes
`ResolvedProjectPolicy`; the retained observation note is a future correction
gate, not an active Goal 01 blocker.

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
| [HoYoLAB — Combat Mechanic Details](https://www.hoyolab.com/article/17984000) | Ultimates may be requested outside the acting character's turn | Community-authored HoYoLAB guide, reviewed 2026-08-04; corroborated by the independent timing references below. |
| [MiHoYo community wiki — Newbie Guide](https://bbs.mihoyo.com/sr/wiki/content/692/detail?bbs_presentation_style=no_header) | Ultimate insertion during skill preparation and cancellation when the actor becomes unable to act before activation | Community-authored publisher-platform wiki, reviewed 2026-08-04; supports player-facing timing but is not an executable specification. |
| [Icy Veins — Combat Basics](https://www.icy-veins.com/honkai-star-rail/combat-basics) | An Ultimate requested during another action activates after the current action finishes | Independent current guide, reviewed 2026-08-04; used only for the action-boundary baseline, not hidden priority edge cases. |
| [Japanese WikiWiki — Damage Calculation](https://wikiwiki.jp/star-rail/%E3%83%80%E3%83%A1%E3%83%BC%E3%82%B8%E8%A8%88%E7%AE%97%E5%BC%8F) | Current cross-check for damage, stat, Super Break, and incoming damage formulas | Useful independent current-language reference; community-maintained. |
| [HoYoWiki — Honkai: Star Rail](https://wiki.hoyolab.com/pc/hsr/home) | Released character, ability, equipment and enemy descriptions | Publisher-operated released-content reference. Record the stable entry ID, access date, game version and content hash because live pages can change and do not expose a complete executable specification. |
| [KQM Star Rail Library](https://srl.keqingmains.com/) and [Evidence Vault](https://srl.keqingmains.com/evidence) | Reviewed combat-mechanic findings and their submitted observation evidence | Useful behavioral cross-check, not a current full-content or 4.4 numeric authority. When repository evidence is used, pin public revision [`de0e5c0`](https://github.com/KQM-git/SRL/commit/de0e5c09c8dbba9577367ad86e991fe91c4f0e36) reviewed on 2026-07-20 and retain the exact document path/hash. |

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
| [HoYoLAB — Version 4.3 update details](https://www.hoyolab.com/article/45284705) | Released Starward-mode boundary: all three rotating challenges use three teams for three independently restartable stages at the highest difficulty | Primary publisher release note reviewed 2026-08-11. It confirms topology and availability, while exact score/cycle thresholds still come from released structured rows and current in-game transcriptions. |
| [Star Rail Wiki — Forgotten Hall](https://honkai-star-rail.fandom.com/wiki/Forgotten_Hall) and [Icy Veins — Memory of Chaos](https://www.icy-veins.com/honkai-star-rail/memory-of-chaos) | Shared cycle ownership/reset behavior and released Starward Stage 12 availability | Current community cross-checks reviewed 2026-08-11. The exact 45-cycle and 15/30 remaining-cycle rows are bound to the pinned released structured source. |
| [Star Rail Wiki — Pure Fiction](https://honkai-star-rail.fandom.com/wiki/Pure_Fiction) and [Icy Veins — Pure Fiction](https://www.icy-veins.com/honkai-star-rail/pure-fiction) | Released three-node Starward topology, independent retained node scores, 45,000 clear threshold, 60,000/75,000/90,000 objectives and separate 99,000 Prismatic Star | Independent current community cross-checks reviewed 2026-08-11; exact encounter, spawn and buff rows remain pinned structured evidence. |
| [Star Rail Wiki — Apocalyptic Shadow](https://honkai-star-rail.fandom.com/wiki/Apocalyptic_Shadow) and [GamsGo — Version 4.4 Apocalyptic Shadow](https://www.gamsgo.com/blog/hsr-apocalyptic-shadow-teams) | Released three-node Starward topology and current Ruinous Embers/Finality's Axiom descriptions | Community cross-checks reviewed 2026-08-11. GamsGo is used only to corroborate released player-facing text; exact IDs and numeric parameters come from the pinned released data. |
| [HoYoLAB — Anomaly Arbitration guide](https://www.hoyolab.com/article/41091494) | Three arbitrary-order Knight challenges, recorded teams/retries, normal and Plight King routes, Quadrants and player-facing protection behavior | Public released guide reviewed 2026-08-11. It establishes topology and relationships; exact hidden Action Value windows and numeric King-protection effects remain policy-bound. |

## Retained event-mode references

| Source | Used for | Confidence and caveat |
|---|---|---|
| [HoYoLAB — Galactic Baseballer event guide](https://www.hoyolab.com/article/28286762), [HoYoLAB — gameplay guide](https://www.hoyolab.com/article/29125952) and [HoYoWiki — retained event entry](https://wiki.hoyolab.com/pc/hsr/entry/2508?lang=en-us) | Original event's three combat phases, weapon/accessory offers, synthesis, persistent initial-weapon upgrades, progression and score loop | Public released guides, rechecked 2026-08-11. Exact IDs, recipes, 114 shop price steps, 56 Strategies, seven team bonuses, levels and scoring vectors come from the pinned Version 4.4 structured Candidate pack. |
| [HoYoLAB — Demon King guide](https://www.hoyolab.com/article/38894296) and [HoYoLAB — Demon King strategy](https://www.hoyolab.com/article/39751178) | Released Demon King edition, changed equipment relationships and retained Departure boundary | Public released guides. They establish player-facing mode relationships, not complete executable battle programs. |
| [HoYoLAB — Version 4.4 update details](https://www.hoyolab.com/article/45851903) and [GameWith — Fate/Star Rail NIGHT guide](https://gamewith.jp/houkaistarrail/article/show/569826) | Released 4.4 event identity and the player-facing deck/card loop: Rin, Servant and Neutral cards; random hands; magical-energy costs; repeated card plays; explicit end turn; ordinary Ultimate insertion | Primary release notice plus an independent post-release observation reviewed 2026-08-11. Exact card/deck/fight identities come from the pinned structured revision; draw counts, discard/refill order, ability operations and custom-fight-to-`BattleSpec` mapping remain explicit policies. |

## Character catalog references

| Source | Used for | Confidence and caveat |
|---|---|---|
| [HoYoLAB — Version 4.4 update details](https://www.hoyolab.com/article/45851903) | Version boundary; official paths/elements and public core summaries for Himeko • Nova, Rin Tohsaka, and Gilgamesh; July 24 collaboration availability | Primary publisher announcement. It is authoritative for disclosed behavior but is not a full numerical kit specification. |
| [HoYoLAB — Fate/UBW Collaboration Warp details](https://www.hoyolab.com/article_pre/18014398241022940) | Rin Tohsaka and Gilgamesh became playable on 2026-07-24 and remain in the long-term collaboration Warp | Primary publisher event notice. It establishes release state, not complete executable kit data. |
| [Star Rail Wiki — Character List](https://honkai-star-rail.fandom.com/wiki/Character/List) | Public roster and alternate combat-form counting through Version 4.3 | Maintained community index. Its snapshot listed 87 playable forms; the Version 4.4 public catalog count is the explicit inference `87 + Himeko • Nova + Rin Tohsaka + Gilgamesh = 90` from the official notices. The frozen executable reference pack predates promotion of the final two and remains 88. |
| [Mar-7th/StarRailRes](https://github.com/Mar-7th/StarRailRes) | Structured English identity and ability metadata used to cross-check the 87 released forms through Version 4.3 | Community-maintained transcription/resource index, not an official API. Used only as a research aid; extracted assets and bulk verbatim descriptions are not included. |
| [Honey Hunter — Himeko • Nova](https://starrail.honeyhunterworld.com/himeko-nova-character/?lang=EN) | Cross-check of 4.4 ability structure, Assist Skill, field, shared uses, and special resource interactions | Unofficial and version-sensitive. Prefer live in-game text when importing numerical data. |
| [Prydwen — Rin Tohsaka](https://www.prydwen.gg/star-rail/characters/rin-tohsaka) | Released 4.4 cross-check for Gem Energy, automatic enhanced-Skill bounce loop, Ultimate target groups, and Archer joint follow-up | Unofficial released-kit transcription, reviewed 2026-08-05. Exact values require promotion against the pinned released-data and in-game evidence path before becoming executable. |
| [Prydwen — Gilgamesh](https://www.prydwen.gg/star-rail/characters/gilgamesh) | Released 4.4 cross-check for Interest, forced Basic/state transition, random-strike Ultimate, and Saber joint follow-up | Unofficial released-kit transcription, reviewed 2026-08-05. Exact values require promotion against the pinned released-data and in-game evidence path before becoming executable. |
| [HoYoLAB — Acheron guide](https://www.hoyolab.com/article/27358507) | Three controlled Rainblade stages followed by the automatic Stygian Resurge finisher | Community guide hosted on HoYoLAB, not a publisher specification. It supports the segmented-input classification; exact retarget and cancellation behavior remains an observation requirement. |
| [Prydwen — Feixiao](https://www.prydwen.gg/star-rail/characters/feixiao) | One locked duel target, six choices between Boltsunder Blitz and Waraxe Skyward, then an automatic finisher | Maintained community kit transcription. It supports the segmented-option classification; target invalidation still requires a fixture. |
| [KQM — Argenti](https://hsr.keqingmains.com/argenti/) | One Ultimate control exposes 90- and 180-Energy programs | Maintained mechanics guide. It supports a prepared program/cost choice rather than a mid-action segment. |
| [Prydwen — Destruction Trailblazer](https://www.prydwen.gg/star-rail/characters/trailblazer-destruction) | Ultimate chooses single-target Basic-style or Blast Skill-style attack mode | Maintained community kit transcription. It supports a prepared program/target choice before declaration. |
| [HoYoLAB — Imbibitor Lunae guide](https://www.hoyolab.com/article/21312158) | Basic enhancement tier 0–3, SP/Squama substitution, and ordinary Blast Ultimate | Community guide hosted on HoYoLAB. It supports a pre-cast Basic composer and rejects treating his Ultimate as the composer. |

## Pre-implementation content reference sources

| Source | Pinned revision | Used for | Confidence and caveat |
|---|---|---|---|
| [Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata) | `fd978d6ef09f941fba644c731ab54abd6f7c3568` | Released 4.4 character/Light Cone/enemy/stage tables, character target metadata, enemy AI sequences, ability entry points, and operation-type evidence | Community-maintained released-data transcription, not an official API. Raw files remain in an ignored cache; the repository commits only normalized facts, source paths, and hashes. |
| [Mar-7th/StarRailRes](https://github.com/Mar-7th/StarRailRes) | `7b349e39ee0f6f3bf814567995829b99c95e7a93` | Released 4.3 bilingual/structured cross-check and fallback for Saber and Archer collaboration records absent from the pinned 4.4 dump | Community-maintained resource index. Fallback rows are explicitly labeled `ExactPreviousRelease`. |

The generated fact baseline, schema, approximation rules, and pack digest are
documented in [Combat content reference pack](content-reference/README.md).

The character profiles paraphrase behavior into engine semantics. They do not claim exact parity for coefficients, animation hit timing, minor Traces, Eidolons, or undisclosed preview values. Those require a separately licensed/authored data pack and release-version validation.

## Known uncertainties

Do not present these as exact parity until tested against captured game observations:

- internal numeric precision and rounding at every damage/resource boundary;
- universal tie-breaking when multiple actors have exactly equal AV;
- interrupt order when several Ultimates are requested at the same opportunity;
- exact Ultimate activation opportunities inside exceptional multi-action boss sequences and presentation-specific skill-preparation states;
- snapshot rules for every DoT, field, summon, delayed hit, heal, and shield;
- exact trigger ordering for all character-specific passives and boss phase transitions;
- retarget behavior after a target dies during every multi-hit or queued action;
- how attempted versus effective Toughness reduction is consumed by every tally effect;
- current-version limits for RES, mitigation, and unusual negative/overflow stats;
- mode-specific replacements for Energy, cycles, defeat, or wave rules.
- complete Version 4.4 row catalogs and exact values for all enemies, equipment, and permanent universe-mode content; these remain a data-import task until coverage reports mark them `DataReady`.
- exact AI/ability/phase programs for the challenge enemy identities currently
  bound to explicit behavior donors;
- the challenge policies listed in the generated `MocRuntimePolicies`,
  `PfRuntimePolicies` and `ApsRuntimePolicies` tables, including Memory Tierce
  Turbulence attachment, Pure Fiction fixed-damage/order/resource edges and
  Apocalyptic Path/War-Armor/non-Energy-readiness edges.

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
8. when bounded research does not resolve a required behavior, record an
   explicit deterministic approximation or project policy, its alternatives,
   affected tests and replacement conditions, then continue the owning batch.
