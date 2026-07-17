# Character Profiles: A–H

These profiles are compact E0 implementation contracts. Coefficients, level scaling, minor Traces, and Eidolons belong in versioned balance data.

## Acheron — Lightning / Nihility — Released

- **Core loop:** Acheron has no ordinary Energy. Debuff application grants Slashed Dream and places/updates Crimson Knot on enemies; nine Dream points unlock an Ultimate that removes Knots while dealing multi-stage AoE damage and temporarily ignores Weakness Type for Toughness reduction.
- **Engine contract:** Attribute resource credit to debuff events with per-action limits, choose Knot transfer behavior when a marked enemy dies, and model the Ultimate as a locked sequence with target selection between stages. Her team-composition Trace inspects the number of other Nihility allies.

## Aglaea — Lightning / Remembrance — Released

- **Core loop:** Skill summons or heals Garmentmaker. The memosprite attacks independently, gains stacking SPD from its attacks, and launches joint attacks with Aglaea. Ultimate enters an empowered state, refreshes/retains the summon, changes Aglaea's Basic ATK, and ends through a countdown action.
- **Engine contract:** Support an independently scheduled memosprite, joint actions with separate damage ownership, summon SPD stacks, enhanced Basic replacement, state-ending countdown, and summon teardown damage/energy effects.

## Anaxa — Wind / Erudition — Released

- **Core loop:** Attacks implant temporary Weakness Types. An enemy with enough different Weaknesses becomes Qualitative Disclosure, taking increased damage; Skill is a multi-bounce attack, and Ultimate applies all standard Weaknesses before AoE damage. Hitting sufficiently qualified targets can grant an extra Skill without spending Skill Points.
- **Engine contract:** Weaknesses need source and duration, multi-type state must be queryable during an action, and the free Skill is a queued extra action with an explicit once-per-turn gate and deterministic bounce targeting.

## Archer — Quantum / The Hunt — Released

- **Core loop:** Skill enters a stance in which repeated Skills may be cast on the same turn while Skill Points remain; each cast strengthens subsequent casts. Ending the chain grants Charge, and allies attacking can consume Charge to trigger Archer's follow-up attack. Ultimate deals single-target damage and seeds Charge.
- **Engine contract:** Implement an action-local recast loop, escalating per-chain modifiers, a manual stop/no-SP exit, and charge-gated ally-attack follow-ups. His large maximum Energy and Skill-Point economy are authored data, not scheduler exceptions.

## Argenti — Physical / Erudition — Released

- **Core loop:** Basic and Skill build Energy normally; Talent grants CRIT Rate stacks for each enemy hit. Ultimate has a lower-cost AoE form and a higher-cost form that adds random single-target hits.
- **Engine contract:** An Ultimate may expose two legal costs/programs from one button. Count targets hit, not hit instances, for the Talent, and preserve deterministic bounce/retarget behavior in the full Ultimate.

## Arlan — Lightning / Destruction — Released

- **Core loop:** Skill consumes HP instead of Skill Points and deals single-target damage. Talent increases damage as Arlan's missing-HP ratio rises; Ultimate is Blast damage.
- **Engine contract:** Costs may reduce HP without defeating the payer, and missing-HP scaling must sample at an authored timing point. Skill Point neutrality is an ability property.

## Ashveil — Lightning / The Hunt — Released

- **Core loop:** Skill designates a unique enemy as Bait; while Bait exists, all enemies lose DEF. Other allies attacking Bait restore Ashveil's Energy and can consume Charge for a follow-up that grants Gluttony. Ultimate marks Bait, triggers a free enhanced follow-up, and spends Gluttony in repeated finishing strikes that can continue on a new Bait after a kill.
- **Engine contract:** Provide a unique transferable mark, team aura conditional on mark existence, ally-only attack triggers, charge-free trigger flags, stack-spending loops, and kill-time retargeting during the same follow-up sequence.

## Asta — Fire / Harmony — Released

- **Core loop:** Skill bounces among enemies. Talent gains Charging stacks when different enemies are hit, with extra credit for Fire weakness, and later decays stacks; the stacks provide team ATK. Ultimate gives a timed team SPD buff.
- **Engine contract:** Track per-action unique targets, weakness checks at hit time, owner-turn stack decay, and team auras whose value depends on current stacks.

## Aventurine — Imaginary / Preservation — Released

- **Core loop:** Skill grants stackable team shields. Shielded allies gain Effect RES; when shielded allies are attacked, Blind Bet accumulates and triggers a multi-hit follow-up that refreshes shields. Ultimate damages one enemy, applies Unnerved, and grants Blind Bet.
- **Engine contract:** Shields from this source stack up to an authored cap and refresh duration; attack reception is counted per qualifying event, including special extra credit for follow-ups. The follow-up must retarget random living enemies deterministically.

## Bailu — Lightning / Abundance — Released

- **Core loop:** Skill heals one ally and then performs two diminishing random bounce heals. Ultimate heals the party and applies Invigoration; invigorated allies heal reactively when hit. Once per battle, Talent can prevent an ally's defeat and restore HP.
- **Engine contract:** Healing supports random bounce with falloff, hit-reactive charges, maximum-HP modification from Invigoration, and a pre-defeat interception hook with a battle-scoped use counter.

## Black Swan — Wind / Nihility — Released

- **Core loop:** Attacks and allied DoT ticks stack Arcana on enemies. Arcana is a Wind DoT that gains threshold effects, including adjacent damage and DEF ignore, then normally resets after triggering. Skill applies AoE DEF reduction; Ultimate makes Arcana count as all four ordinary DoT types and delays its reset.
- **Engine contract:** DoT stacks need threshold-dependent programs and controlled reset, DoT-tick events must expose source/type, and temporary multi-type classification must not duplicate the same DoT instance.

## Blade — Wind / Destruction — Released

- **Core loop:** Skill consumes HP, enters Hellscape, and replaces Basic ATK with an HP-scaling Blast attack for several turns. HP loss accumulates Charge for an AoE follow-up that heals Blade. Ultimate sets current HP to a fixed ratio and deals damage scaling partly from HP lost since the previous Ultimate.
- **Engine contract:** Support non-turn-ending stance activation, ability replacement, bounded HP-loss tally with reset, self-HP set operations, and once-per-attack Charge credit independent of hit count.

## Boothill — Physical / The Hunt — Released

- **Core loop:** Skill starts Standoff with one enemy, forcing both sides to focus each other and increasing damage received. Defeating or breaking the target grants Pocket Trickshot, which improves enhanced Basic ATK Toughness/damage; the enhanced Basic deals Break damage again to broken targets. Ultimate implants Physical Weakness and delays the target.
- **Engine contract:** Model mutual taunt/duel state, a Basic replacement, break-event and defeat-event stack credit, Weakness implantation, Action delay, and Break-damage replay against already broken targets.

## Bronya — Wind / Harmony — Released

- **Core loop:** Skill dispels one ally's debuff, increases damage, and advances that ally to immediate action. Ultimate buffs team ATK and CRIT DMG. Talent advances Bronya after her Basic ATK.
- **Engine contract:** Dispel, 100% action advance, and self-advance must preserve interrupt priority and buff-duration timing. Skill cannot select Bronya unless an authored exception permits it.

## Castorice — Quantum / Remembrance — Released

- **Core loop:** Allied HP loss builds Newbud instead of conventional Energy. Ultimate consumes the resource to summon Netherwing. The dragon spends party HP to attack, can repeatedly act while enough HP is available, provides a reserve against ally defeat, and deals a final attack when it disappears.
- **Engine contract:** Tally effective party HP loss, schedule an independently acting memosprite, pay costs across multiple allies without invalid negative HP, intercept defeat through summon state, and give disappearance a mandatory resolution program.

## Cerydra — Wind / Harmony — Released

- **Core loop:** Skill designates one ally with Military Merit and grants Cerydra Charge. That ally gains ATK and produces Charge through Basic/Skill use. At six Charge the mark upgrades to Peerage, strengthening Skill damage and RES PEN and enabling Coup de Main, an immediate duplicate/copy of the marked ally's Skill under authored restrictions.
- **Engine contract:** A unique ally link must survive refresh/reassignment; copied Skills need original source, targets, cost suppression, and trigger-credit policy. Charge gain and Peerage transition occur atomically.

## Cipher — Quantum / Nihility — Released

- **Core loop:** Cipher marks the highest-Max-HP enemy as Patron and records a portion of damage allies deal to it. Her follow-up attacks marked targets after ally attacks, while Ultimate deals damage and releases the recorded tally as additional true damage. Skill applies a debuff and damages multiple enemies.
- **Engine contract:** Maintain a transferable Patron mark, record post-mitigation damage with a cap and reset boundary, prevent the release damage from recursively entering its own tally, and support true/additional damage with explicit ownership.

## Clara — Physical / Destruction — Released

- **Core loop:** Svarog counters enemies that attack Clara, marking them. Skill hits all enemies and deals extra damage to marked targets, then clears the marks. Ultimate reduces Clara's damage taken, raises aggro, and grants a limited number of enhanced counters that can trigger when any ally is attacked and deal Blast damage.
- **Engine contract:** Attack-received triggers must identify attacker and protected ally, counters need limited shared charges and priority, and marks are consumed per target after Skill damage resolves.

## Cyrene — Ice / Remembrance — Released

- **Core loop:** Allies with Future grant Recollection when they act. Skill deploys a Zone that adds true damage after allied damage. At maximum Recollection, Ultimate summons Demiurge, activates every teammate's Ultimate, enters a persistent enhanced state, and lets the memosprite grant a character-specific Ode to each Chrysos Heir (or a generic damage buff otherwise).
- **Engine contract:** This is a feature-completeness stress test: party-wide Ultimate readiness/activation, persistent Zone, true-damage echo, memosprite extra turns, per-recipient one-time/persistent programs, and explicit interactions with each supported Chrysos Heir. Odes belong in a registry keyed by combat-form ID, not in the scheduler.

## Dan Heng — Wind / The Hunt — Released

- **Core loop:** Skill is a high-CRIT single-target attack. Talent grants Wind RES PEN when Dan Heng is targeted by an ally ability, with a cooldown; a CRIT from Skill slows the enemy. Ultimate deals extra damage to slowed targets.
- **Engine contract:** Ally-targeted event credit, cooldown in owner turns, conditional debuff on CRIT result, and target-state sampling for Ultimate damage are required.

## Dan Heng • Imbibitor Lunae — Imaginary / Destruction — Released

- **Core loop:** Basic ATK can be enhanced zero to three times before execution by spending Skill Points, changing hit plan from single target to stronger Blast. Ultimate supplies Squama Sacrosancta, a substitute cost for enhancements. Repeated hits build outgoing-damage and CRIT-DMG stacks.
- **Engine contract:** Add a pre-cast enhancement composer, mixed substitute resources with deterministic spending priority, per-hit stack gains, and a final ability identity that reflects the chosen enhancement tier.

## Dan Heng • Permansor Terrae — Physical / Preservation — Released

- **Core loop:** Skill chooses a Bondmate, summons Souldragon, and grants stackable party shields. Souldragon acts independently to cleanse one debuff from all allies and refresh shields. Ultimate attacks all enemies, shields the party, and empowers the dragon's next two actions with a joint Physical/Bondmate-element follow-up.
- **Engine contract:** Support unique ally links, source-stackable shields, independent summon actions, party cleanse, dynamic element copied from Bondmate, and finite enhanced summon actions.

## Dr. Ratio — Imaginary / The Hunt — Released

- **Core loop:** Skill attacks one enemy and may trigger a follow-up; its chance rises with the target's debuff count. Ultimate applies Wiseman's Folly, whose limited charges let allied attacks trigger Ratio's follow-up. Traces grant bonuses from debuff-count thresholds.
- **Engine contract:** Query distinct qualifying debuffs, roll seeded follow-up chance once per Skill, and consume mark charges on ally attack events while preventing Ratio's own generated follow-up from recursive credit.

## Evanescia — Physical / Elation — Released

- **Core loop:** Energy and Certified Banger mirror one another. Reaching authored Energy intervals makes Master Fox perform AoE follow-ups and refund Energy. While Certified Banger is held, Skill, Ultimate, and Fox add Elation damage; the Elation Skill is a large AoE attack that grants more of the resource.
- **Engine contract:** Resource changes need guarded bidirectional mirroring, threshold crossing can fire more than once on a large gain, and follow-up/Elation damage must carry distinct tags. Her 480 maximum Energy is ordinary data.

## Evernight — Ice / Remembrance — Released

- **Core loop:** Skill spends Evernight's HP to summon/heal Evey, buffs allied memosprite CRIT DMG, and gains Memoria. HP loss by either linked unit builds Memoria and CRIT DMG; at the threshold Evey acts immediately. Ultimate creates Darkest Riddle, increasing damage and CC immunity. Evey's enhanced action spends all Memoria and its HP, then disappears.
- **Engine contract:** Link owner/summon HP-change events, implement threshold immediate action, team filters for memosprites, all-resource spend, disappearance hooks, and state-dependent Memoria gain.

## Feixiao — Wind / The Hunt — Released

- **Core loop:** Feixiao's Ultimate uses Flying Aureus rather than ordinary Energy; allied attacks build it. Her Skill advances the next follow-up, while Talent launches a follow-up after an ally attacks. Ultimate performs a selectable multi-strike sequence that ignores Weakness Type and finishes with a stronger hit based on broken state.
- **Engine contract:** Count qualifying ally attacks, cap fractional/whole special resource correctly, enforce once-per-turn follow-up gates, and model the Ultimate as a scripted sequence with broken-state branches.

## Firefly — Fire / Destruction — Released

- **Core loop:** Normal Skill consumes HP and restores Energy. Ultimate enters Complete Combustion, advances Firefly, replaces Basic/Skill, increases SPD and Break performance, and ends at a countdown action. Enhanced Skill implants Fire Weakness, heals, and deals Break/Super Break-oriented damage.
- **Engine contract:** Transformation atomically swaps abilities and stats, inserts a countdown actor, applies Weakness before Toughness damage, and exits cleanly even across waves or defeat/revival.

## Fu Xuan — Quantum / Preservation — Released

- **Core loop:** Skill creates Matrix of Prescience, granting Max HP/CRIT Rate and redirecting most allied incoming damage to Fu Xuan. Ultimate deals AoE damage and restores a charge of her self-heal. Talent reduces party damage and automatically restores Fu Xuan's HP below a threshold while charges remain.
- **Engine contract:** Split incoming damage before HP application without recursively redirecting it, distinguish mitigation from transfer, apply Max-HP changes safely, and resolve threshold self-heal after a damage batch.

## Fugue — Fire / Nihility — Released

- **Core loop:** Skill grants an ally Foxian Prayer; their attacks reduce DEF and, through Fugue, can deal Super Break. Talent gives enemies Exo-Toughness so a second Break can occur. Ultimate deals universal Toughness damage to all enemies.
- **Engine contract:** Add a second Toughness layer with its own break event/reward, source-attributed Super Break, ally-linked on-hit debuff application, and attacks that ignore Weakness Type only for Toughness reduction.

## Gallagher — Fire / Abundance — Released

- **Core loop:** Skill directly heals one ally. Ultimate applies Besotted to all enemies, damages them, and advances Gallagher; it also enhances his next Basic. Allies heal themselves when attacking Besotted enemies, while enhanced Basic reduces one target's ATK and heals the party.
- **Engine contract:** Healing can trigger from attacking a marked enemy with a per-attack limit, Ultimate action advance must queue the enhanced Basic opportunity, and fixed healing values still pass through outgoing/incoming healing modifiers as authored.

## Gepard — Ice / Preservation — Released

- **Core loop:** Skill attacks one enemy with a chance to Freeze. Ultimate gives the party shields. Once per battle, Talent prevents Gepard's defeat and restores HP; a Trace increases his aggro.
- **Engine contract:** Seeded effect application, party shield instances, weighted targeting, and a battle-scoped pre-defeat replacement are sufficient.

## Gilgamesh — Lightning / Destruction — Announced

- **Core loop:** Officially disclosed behavior says teammate actions build **Interest**, provide Gilgamesh Energy, and increase his Ultimate damage. Higher Interest advances his actions and enables frequent attacks; his main burst comes from the Ultimate. A joint follow-up exists when Saber is on the team.
- **Engine contract:** Reserve an Interest resource, ally-action trigger, Energy and Ultimate-damage modifiers, threshold/proportional action advance, and Saber-linked joint follow-up. Exact abilities, caps, costs, hit plans, and trigger limits are open data until the public playable kit is available.

## Guinaifen — Fire / Nihility — Released

- **Core loop:** Skill deals Blast damage and applies Burn; Basic can also Burn through a Trace. Ultimate damages all enemies and immediately triggers existing Burns for a fraction of their normal damage. Enemies taking Burn damage gain stacking Firekiss, increasing damage taken.
- **Engine contract:** Partial DoT detonation must not advance duration, Firekiss credit follows Burn damage events, and stack duration/refresh are source-aware.

## Hanya — Physical / Harmony — Released

- **Core loop:** Skill damages and marks an enemy with Burden. After allies attack a Burdened target enough times, the team recovers a Skill Point and the mark ends; attackers also receive a damage buff. Ultimate grants one ally SPD and ATK based partly on Hanya's SPD.
- **Engine contract:** A target mark needs a shared hit/action counter and one-shot SP refund. Snapshot Hanya's SPD at the authored cast point and define whether multi-hit attacks count once.

## Herta — Ice / Erudition — Released

- **Core loop:** Basic and Skill are direct damage, with Skill gaining damage against high-HP enemies. Whenever an enemy falls to half HP or below, Herta performs an AoE follow-up, potentially once for each newly crossed target.
- **Engine contract:** Detect downward HP-threshold crossings after a damage batch, enqueue the correct number of follow-ups, and avoid retriggering for an enemy that already crossed the threshold.

## Himeko — Fire / Erudition — Released

- **Core loop:** Weakness Breaks grant Charge, with more credit from stronger enemies; at full Charge Himeko launches an AoE follow-up. Skill is Blast damage and Ultimate is AoE damage that restores Energy for defeated enemies.
- **Engine contract:** Break-event credit is per enemy and rank, Charge may overflow only as authored, follow-up timing follows the completed attack, and defeat credit during Ultimate is counted deterministically.

## Himeko • Nova — Fire / Erudition — Released

- **Core loop:** Himeko pilots a mech and establishes Starblazer Visioscape, allowing allies to call a limited-use Assist Skill that counts as Himeko using a Skill. Her Skill restores Assist uses and grants a team damage buff; allies other than Himeko recover Energy after using the Assist. Ultimate initiates a multi-attack mech sequence, with special Trailblaze Companion interactions.
- **Engine contract:** Abilities may be temporarily injected into other actors while preserving Himeko as the action/damage source; track shared uses, team field lifetime, caller-specific Energy refund, universal Toughness reduction for the Assist, and a scripted Ultimate sequence. Exact 4.4 coefficients and Companion clauses remain versioned live data.

## Hook — Fire / Destruction — Released

- **Core loop:** Skill applies Burn and does extra damage to already Burning targets. Ultimate deals single-target damage and enhances the next Skill into stronger Blast damage. Talent adds damage and restores Energy when attacking Burning enemies.
- **Engine contract:** One-use enhanced-ability state, target-state sampling, and additional-damage/Energy triggers once per attack are required.

## Huohuo — Wind / Abundance — Released

- **Core loop:** Skill cleanses and heals a target plus adjacent allies. It grants Divine Provision, which automatically heals allies at turn start or Ultimate use and cleanses limited debuffs. Ultimate restores allies' Energy by a percentage of their maxima and buffs ATK, excluding Huohuo from the Energy grant.
- **Engine contract:** Reactive healing needs trigger filters and per-Skill cleanse charges; percentage Energy restoration uses each recipient's authored maximum and obeys cap/rounding policy.

## Hyacine — Wind / Remembrance — Released

- **Core loop:** Hyacine summons Little Ica and heals the party. Healing performed by Hyacine/Ica accumulates a tally; the memosprite consumes or references it for damage while also providing healing. Ultimate raises party Max HP, heals, and empowers the summon.
- **Engine contract:** Healing events must expose effective and overflow amounts according to the ability's tally rule, owner and memosprite share state, Max-HP changes preserve current-HP policy, and summon actions mix damage and healing.

## Hysilens — Physical / Nihility — Released

- **Core loop:** Allied attacks give Hysilens a chance to apply one of Bleed, Burn, Shock, or Wind Shear. Skill raises enemy damage taken. Ultimate creates a Zone that lowers enemy ATK/DEF and adds a Physical DoT instance whenever enemies receive DoT damage.
- **Engine contract:** Random selection uses seeded weighted choices; DoT-received triggers must be guarded against recursive Zone DoT loops, and Zone teardown removes only its sourced modifiers.
