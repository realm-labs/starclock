# Character Profiles: I–R

These profiles are compact E0 implementation contracts. Coefficients, level scaling, minor Traces, and Eidolons belong in versioned balance data.

## Jade — Quantum / Erudition — Released

- **Core loop:** Skill designates another ally as Debt Collector, raises their SPD, and makes each of their attack targets receive Jade-owned additional Quantum damage at an HP cost to the collector. Each enemy hit by Jade or the collector grants Charge; full Charge triggers Jade's AoE follow-up and grants stacking Pawned Asset CRIT DMG. Ultimate deals AoE damage and empowers a limited number of follow-ups.
- **Engine contract:** Link one ally, count targets rather than hit instances, apply nonlethal ally HP costs, emit source-separated additional damage inside another action, and maintain finite enhanced-follow-up charges plus persistent capped stacks.

## Jiaoqiu — Fire / Nihility — Released

- **Core loop:** Attacks apply stacking Ashen Roast, which increases damage taken and counts as Burn. Ultimate equalizes enemy stacks to the current maximum and creates a Zone that raises Ultimate damage taken; enemy turns inside the Zone can gain another stack. Burn ticks from the mark deal Fire DoT.
- **Engine contract:** A debuff can be both vulnerability and DoT, stack equalization is a batch operation, and Zone turn-start triggers need a per-turn application chance/limit.

## Jing Yuan — Lightning / Erudition — Released

- **Core loop:** Lightning-Lord is a linked timeline actor whose Hits Per Action increase when Jing Yuan uses Skill or Ultimate. On its turn it performs random strikes with adjacent splash; SPD rises with hit count. Crowd Control on Jing Yuan prevents the summon from acting.
- **Engine contract:** Add a non-targetable linked actor with dynamic SPD and multi-hit plan, deterministic random targets, wave persistence policy, and an owner-state gate at the summon action.

## Jingliu — Ice / Destruction — Released

- **Core loop:** Skill and Ultimate gain Syzygy. At two stacks Jingliu action-advances into Spectral Transmigration, gains CRIT Rate, and replaces Skill with a no-SP enhanced Blast Skill that consumes Syzygy and drains small amounts of allies' HP to gain ATK. The state ends at zero stacks.
- **Engine contract:** Threshold transition and action advance must be atomic, ability replacement must preserve the pending turn, party HP drain is nonlethal, and ATK gained from drained HP obeys an authored cap/snapshot.

## Kafka — Lightning / Nihility — Released

- **Core loop:** Skill deals Blast damage and immediately detonates DoTs on the primary target. Ultimate attacks all enemies, applies Shock, and detonates their DoTs. Once per turn, an ally Basic ATK prompts Kafka's follow-up, which applies Shock.
- **Engine contract:** Partial/full DoT detonation does not consume duration, each DoT retains original source and stats, and the ally-Basic follow-up has a turn-scoped cooldown.

## Lingsha — Fire / Abundance — Released

- **Core loop:** Skill heals the party and advances Fuyuan, a linked summon. Fuyuan acts to deal AoE damage, heal, and cleanse; its limited action count is refreshed by Skill. When an ally becomes low HP, Fuyuan can perform an emergency follow-up. Ultimate applies Befog, increasing Break damage received, heals, attacks, and advances Fuyuan.
- **Engine contract:** Summon actions carry a remaining-count lifetime, accept action advance, and mix damage/heal/cleanse. Emergency action uses a cooldown and HP-threshold event rather than ordinary summon AV.

## Luka — Physical / Nihility — Released

- **Core loop:** Skill dispels one enemy buff and applies HP-scaled Bleed. Basic/Skill build Fighting Will; at four stacks Basic becomes an enhanced multi-hit attack that detonates Bleed. Ultimate damages one target and increases its damage taken.
- **Engine contract:** Enemy dispel, capped HP-scaling DoT, resource-based Basic replacement, and partial DoT detonation are required.

## Luocha — Imaginary / Abundance — Released

- **Core loop:** Skill heals and gains Abyss Flower. A free emergency Skill triggers when an ally reaches low HP, with a cooldown. At two Flowers a Field is deployed; allies heal themselves and the party when attacking. Ultimate dispels one buff from all enemies, deals AoE damage, and grants a Flower.
- **Engine contract:** Provide threshold-triggered free ability with cooldown, stack-spend field creation, attack-triggered healing, enemy mass dispel, and field duration on Luocha's turns.

## Lynx — Quantum / Abundance — Released

- **Core loop:** Skill increases one ally's Max HP and heals them; if the target is Preservation or Destruction it also raises aggro. Ultimate heals and cleanses the whole party. Talent adds healing-over-time to Skill/Ultimate recipients.
- **Engine contract:** Max-HP modification must declare current-HP adjustment, aggro modifiers are path-conditional, and one party cleanse event removes one eligible debuff per target as authored.

## March 7th (Preservation) — Ice / Preservation — Released

- **Core loop:** Skill shields one ally, cleanses through a Trace, and increases that ally's aggro when above an HP threshold. When a shielded ally is attacked, March counters a limited number of times per turn. Ultimate deals AoE damage with a Freeze chance.
- **Engine contract:** Shields retain source identity, counter charges reset by owner turn, trigger after an attack against any qualifying shield, and aggro depends on current/shield application state.

## March 7th (The Hunt) — Imaginary / The Hunt — Released

- **Core loop:** Skill selects a Shifu ally and changes March's bonuses according to the Shifu's path category. Shifu attacks grant Charge; at seven Charge March takes immediate action and uses an enhanced multi-hit Basic. Ultimate strengthens the next enhanced Basic.
- **Engine contract:** Link/reassign one ally, classify paths into authored groups, count Shifu actions, queue threshold immediate action without disturbing normal AV, and script the enhanced Basic's conditional extra hits.

## Misha — Ice / Destruction — Released

- **Core loop:** Skill gains Hits Per Action for the next Ultimate; every team Skill Point spent adds another hit and restores Misha's Energy. Ultimate bounces among enemies and can Freeze, then resets its hit count.
- **Engine contract:** Observe team SP-spend events with a cap, snapshot/reset the Ultimate hit count at cast, and perform seeded bounce plus per-hit effect application policy.

## Mortenax Blade — Fire / Nihility — Released

- **Core loop:** Basic taunts its target and Skill spends HP for AoE-plus-bounce damage. Ultimate spends HP, applies Balefire Bind (DEF down and vulnerability), and opens an Infinite Fury Zone that increases CRIT, replaces Basic, unlocks Skill, and changes Ultimate. Allied attacks inside the Zone apply Bind and grant Charge; at nine Charge Mortenax gains Energy and a free extra Skill. A countdown ends the Zone, while a killing blow against him ends it immediately.
- **Engine contract:** Implement transformation/ability replacement, battlefield countdown, source-scoped Zone teardown, taunt, HP costs, ally-attack Charge with a 1-HP guard, and a queued cost-free extra Skill. Phase changes/revival must retain or clear Bind by explicit policy.

## Moze — Lightning / The Hunt — Released

- **Core loop:** Skill marks one enemy as Prey and removes Moze from the ordinary action order. Allies attacking Prey deal Moze-owned additional damage and consume Charge; after a set number of charges Moze performs a follow-up. When charges or Prey end, Moze returns to the timeline. Ultimate attacks and immediately triggers the follow-up.
- **Engine contract:** Support a temporarily departed/non-targetable actor that can still emit damage, unique mark lifetime, shared charge consumption, timeline re-entry, and forced follow-up without normal trigger cost.

## Mydei — Imaginary / Destruction — Released

- **Core loop:** Skill consumes HP for Blast damage. Accumulated HP loss builds Charge; at the threshold Mydei restores HP, advances action, and enters Vendetta with increased Max HP and taunt. Vendetta replaces Skill with a stronger no-cost form; reaching the next Charge threshold automatically casts Godslayer Be God, then ends the state. Talent can prevent defeat while charged/state conditions permit.
- **Engine contract:** Tally HP lost across self-cost and incoming damage, perform atomic Max-HP/state changes, enforce forced ability use at thresholds, support taunt and pre-defeat replacement, and define overflow Charge/reset.

## Natasha — Physical / Abundance — Released

- **Core loop:** Skill heals one ally and applies healing-over-time; a Trace cleanses one debuff. Ultimate heals the party. Talent increases outgoing healing to low-HP allies.
- **Engine contract:** Direct and periodic healing share the standard formula; low-HP bonus samples before each heal and cleanse ordering is authored.

## Pela — Ice / Nihility — Released

- **Core loop:** Skill damages and removes one enemy buff. Ultimate deals AoE damage and applies Exposed, reducing DEF. Talent restores Energy when an attack hits a debuffed enemy, with a per-attack limit.
- **Engine contract:** Enemy dispel, AoE debuff application, and once-per-attack debuffed-target detection are required.

## Phainon — Physical / Destruction — Released

- **Core loop:** Phainon gains Coreflame when allies target him or use Ultimates. At full resource, Ultimate transforms him into Khaslana, removes other allies from the field, creates a Territory, and grants several extra turns with a new ability set. Khaslana accumulates Scourge and supports a defensive counter, multi-target attacks, and a final transformation-ending strike.
- **Engine contract:** This requires party temporary departure, solo targetability, Territory ownership, batched extra turns that do not use ordinary AV, full ability/stat replacement, resource conversion, and guaranteed restoration of allies/state when transformation ends.

## Qingque — Quantum / Erudition — Released

- **Core loop:** Allies' turns draw tiles. Skill spends Skill Points, draws more tiles, and stacks a damage buff without ending Qingque's turn. Four matching tiles replace Basic with an enhanced Blast attack; Ultimate deals AoE damage and grants a matching hand.
- **Engine contract:** Model a seeded hand/multiset, repeated non-turn-ending Skill casts, SP exhaustion, ability replacement, and atomic hand consumption/reset after enhanced Basic.

## Rappa — Imaginary / Erudition — Released

- **Core loop:** Ultimate enters Sealform, grants an extra turn, and replaces Basic with a three-part enhanced attack that deals universal Toughness damage. Enemy Weakness Breaks grant Charge; the final enhanced strike spends Charge to deal extra Break damage and reduce Toughness, including against already broken enemies through Super Break behavior.
- **Engine contract:** Transformation plus extra turn, staged enhanced Basic with finite uses, universal Toughness reduction, break-event Charge, and Break/Super Break damage sourced from the final stage are required.

## Rin Tohsaka — Quantum / Erudition — Announced

- **Core loop:** Officially disclosed behavior says team Skill Point spending or recovery grants **Gem Energy** and increases Rin's CRIT DMG. Her enhanced Skill consumes Gem Energy for a longer, stronger burst. Public preview data also indicates repeated random hits and a team interaction with Archer that triggers a joint follow-up.
- **Engine contract:** Reserve team SP-change listeners, Gem Energy, threshold Skill replacement, a resource-consuming bounce loop, and Archer-linked joint follow-up. Exact caps, SP interaction, hit count, costs, and coefficients remain provisional until the playable kit opens on 2026-07-24.

## Robin — Physical / Harmony — Released

- **Core loop:** Skill grants a timed team damage buff. Ultimate makes every other ally act immediately and puts Robin into Concerto, removing her from normal turns while granting team ATK and adding Robin-owned Physical damage after ally attacks. The state ends when its countdown acts.
- **Engine contract:** Batch 100% action advance with stable ordering, temporary timeline departure, countdown actor, attack-triggered additional damage with no recursive attack credit, and ATK buff derived from Robin's ATK.

## Ruan Mei — Ice / Harmony — Released

- **Core loop:** Skill grants Overtone, increasing party damage and Weakness Break Efficiency. Ultimate creates a field granting RES PEN and applying Thanatoplum Rebloom on attacks; when a marked broken enemy tries to recover, its recovery is delayed and extra Break damage is dealt. Talent raises party SPD and adds Break damage on allied breaks.
- **Engine contract:** Modify Toughness reduction before clamping, intercept Weakness-recovery actions, apply action delay and Break damage without creating a second break event, and time field/buff durations on Ruan Mei's turns.
