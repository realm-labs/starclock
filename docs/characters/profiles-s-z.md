# Character Profiles: S–Z

These profiles are compact E0 implementation contracts. Coefficients, level scaling, minor Traces, and Eidolons belong in versioned balance data.

## Saber — Wind / Destruction — Released

- **Core loop:** Saber accumulates Core Resonance from allied Ultimate use and Energy changes. Skill deals Blast damage and can consume Resonance for extra actions/attacks; Ultimate uses an unusually large Energy pool for a powerful multi-stage attack. Blessing of the Lake alters Energy flow and supports repeated burst cycles.
- **Engine contract:** Energy-change events must expose effective gain and payer/source, special resource thresholds can queue extra actions, and the Ultimate is a scripted multi-stage program. Keep Fate collaboration joint-attack hooks data-driven for Gilgamesh interactions.

## Sampo — Wind / Nihility — Released

- **Core loop:** Every attack can stack Wind Shear on hit enemies. Skill bounces among enemies. Ultimate deals AoE damage, applies Wind Shear, and increases DoT damage taken.
- **Engine contract:** DoT application can occur per hit but respects a stack cap; seeded bounce targeting and a distinct DoT-vulnerability modifier are required.

## Seele — Quantum / The Hunt — Released

- **Core loop:** Skill increases Seele's SPD. Defeating an enemy grants Resurgence, an immediate extra turn with a damage buff; the extra turn cannot recursively grant another Resurgence. Ultimate enters the buffed state before dealing single-target damage.
- **Engine contract:** Defeat credit schedules a gated extra turn without resetting normal AV, and the no-recursion flag must survive all damage in that extra action.

## Serval — Lightning / Erudition — Released

- **Core loop:** Skill deals Blast damage and applies Shock. Talent adds damage when attacking Shocked enemies. Ultimate deals AoE damage and extends existing Shock duration.
- **Engine contract:** DoT duration extension must preserve stacks/source and target-state additional damage triggers once per attacked target as authored.

## Silver Wolf — Quantum / Nihility — Released

- **Core loop:** Skill may implant a Weakness matching an ally's element, reduces the corresponding RES, and deals damage; it also applies a random Bug through Talent. Basic/Skill/Ultimate can apply Bugs that reduce ATK, DEF, or SPD. Ultimate heavily reduces one enemy's DEF.
- **Engine contract:** Select eligible ally elements with seeded deduplication, distinguish native and implanted Weaknesses, replace/refresh implants by source, and choose Bugs using a deterministic pool that respects active-state rules.

## Silver Wolf LV.999 — Imaginary / Elation — Released

- **Core loop:** Punchline also builds Hidden MMR; MMR increases CRIT Rate and then CRIT DMG beyond 100% CRIT Rate. At 60 MMR, Ultimate transforms her, advances action, and opens a Zone. During transformation, allies spending Skill Points can trigger diminishing-chance Top Loot Boxes that deal Elation damage and either add true damage, restore SP, or grant Punchline. Three enhanced Basics end the form.
- **Engine contract:** Implement transformation, action advance, shared SP-spend triggers with a probability curve/reset, random subprogram selection, CRIT overflow conversion, and a multi-stage enhanced Basic capable of preserving unused hits across enemy entry. Elation Skill can reset the Loot Box chance.

## Sparkle — Quantum / Harmony — Released

- **Core loop:** Talent raises the team's maximum Skill Points and grants stacking team damage whenever an ally spends SP. Skill buffs one ally's CRIT DMG based on Sparkle's CRIT DMG and advances their action. Ultimate restores multiple SP and enhances Talent's damage buff through Cipher.
- **Engine contract:** Team resource cap is mutable, SP spending emits events even inside repeated actions, derived-stat buffs snapshot by policy, and partial action advance preserves queue ordering.

## Sparxie — Fire / Elation — Released

- **Core loop:** Skill starts a livestream, replaces Basic with a Blast version, and repeatedly invokes Engagement Farming to increase its multiplier while randomly granting gifts. Ultimate grants Punchline and deals AoE damage. While Certified Banger is held, enhanced Basic and Ultimate also deal Elation damage; her Elation Skill performs AoE plus twenty bounces and grants Thrill that substitutes for Skill Point costs.
- **Engine contract:** Support a pre-attack repeated subaction loop, random gift table, ability replacement, cost-substitute resource, long deterministic bounce sequence, and separate Elation damage emitted by ordinary abilities.

## Sunday — Imaginary / Harmony — Released

- **Core loop:** Skill advances one ally and their summon/memosprite, increases their damage, and does not consume SP when used on The Beatified under the appropriate state. Ultimate restores Energy and makes one ally The Beatified, granting CRIT DMG based on Sunday's own. Talent grants CRIT Rate to the Skill target.
- **Engine contract:** One command can advance a linked owner and summon, derived-stat buffs need snapshot rules, percentage Max-Energy restoration needs rounding, and unique ally state controls cost overrides.

## Sushang — Physical / The Hunt — Released

- **Core loop:** Skill can trigger extra Sword Stance damage, guaranteed against broken enemies. Ultimate damages one target, advances Sushang to immediate action, buffs ATK, and improves subsequent Sword Stance triggers. Talent grants SPD when any enemy is broken.
- **Engine contract:** Seeded conditional additional damage, broken-state guarantee, 100% self-advance, and battlefield break-state reaction with owner-turn duration are required.

## The Dahlia — Fire / Nihility — Released

- **Core loop:** Skill deploys a Zone that increases party Weakness Break Efficiency and allows Super Break damage even against enemies that are not currently broken. She links with a Dance Partner; either partner attacking enables Super Break, and the other partner's attacks trigger Dahlia's bounce follow-up. Ultimate implants both partners' elements as Weaknesses and reduces enemy DEF.
- **Engine contract:** Generalize Super Break eligibility beyond broken state, link two partners, guard alternating attack triggers from recursion, perform bounce plus conditional Super Break, and apply a set of dynamically sourced Weakness Types.

## The Herta — Ice / Erudition — Released

- **Core loop:** Enemies carry Interpretation stacks, increased by attacks. Skill attacks one target and adjacent targets in repeated stages, scaling with stacks. Ultimate redistributes stacks toward elite targets, deals AoE damage, fully advances Herta, and grants Inspiration; Inspiration enables an enhanced Skill that consumes target stacks for a stronger sequence. Other Erudition allies improve stack generation and team effects.
- **Engine contract:** Per-enemy stacks need transfer/redistribution as an atomic batch, enhanced Skill consumes after sampling, action advance follows Ultimate resolution, and party Path composition modifies triggers.

## Tingyun — Lightning / Harmony — Released

- **Core loop:** Skill grants one ally Benediction, increasing ATK and adding Tingyun-owned Lightning damage to that ally's attacks. Ultimate restores a fixed amount of Energy and increases one ally's damage. Tingyun's Basic adds damage based on the blessed ally's ATK through a Trace.
- **Engine contract:** Unique ally link, derived ATK cap, attack-triggered additional damage with explicit ownership, and flat Energy restoration are required.

## Topaz & Numby — Fire / The Hunt — Released

- **Core loop:** Skill marks one enemy with Proof of Debt, increasing follow-up damage taken. Numby is a linked summon that attacks the marked target; allied follow-up attacks advance Numby. Ultimate empowers a limited number of Numby attacks, and under that state other qualifying ally attacks can also advance it.
- **Engine contract:** Maintain a transferable unique mark, a linked timeline actor, percentage action advance with stable ordering, enhanced-action charges, and retarget on marked-target defeat.

## Trailblazer (Destruction) — Physical / Destruction — Released

- **Core loop:** Skill deals Blast damage. Ultimate chooses between an enhanced single-target Basic-style strike and an enhanced Blast Skill-style strike. Talent stacks ATK when the Trailblazer inflicts Weakness Break.
- **Engine contract:** Ultimate exposes two target/program choices after cost payment, and break-event stacks have an authored duration/cap.

## Trailblazer (Preservation) — Fire / Preservation — Released

- **Core loop:** Attacks and being hit build Magma Will. Skill reduces incoming damage, taunts all enemies, and grants Will. At full Will, Basic becomes an enhanced Blast attack. Every Trailblazer action grants small party shields; Ultimate is AoE and primes the enhanced Basic without consuming Will.
- **Engine contract:** Global taunt uses effect chance, action-completion shields stack/refresh by source, and enhanced Basic has two independent enabling modes with explicit consumption priority.

## Trailblazer (Harmony) — Imaginary / Harmony — Released

- **Core loop:** Skill is a bounce attack. Ultimate grants Backup Dancer, increasing Break Effect and causing allies attacking broken enemies to deal Super Break damage. Talent restores Energy when enemies are Weakness Broken.
- **Engine contract:** Add team-scoped Super Break emission based on Toughness reduction and Break Effect, break-event Energy credit, and bounce targeting.

## Trailblazer (Remembrance) — Ice / Remembrance — Released

- **Core loop:** Skill summons/heals Mem. Mem accumulates Charge from team Energy gains and its own actions; at full Charge it immediately acts and can support one ally, advancing them and adding true damage to their attacks. Ultimate deals AoE damage and charges Mem.
- **Engine contract:** Memosprite timeline, team Energy-change tally, threshold immediate action, unique ally support link, action advance, and true-damage echo with recursion guard are required.

## Trailblazer (Elation) — Lightning / Elation — Released

- **Core loop:** Attacks regenerate Energy and grant Punchline. Skill is AoE and grants Certified Banger; while it is held, Skill also emits Elation damage. Ultimate buffs and cleanses one ally: an Elation ally receives Certified Banger and immediately uses their Elation Skill, while another ally is action-advanced. The Trailblazer's own Elation Skill is random multi-hit damage followed by evenly distributed damage.
- **Engine contract:** Support Punchline/Banger team subsystem, conditional forced Elation Skill versus action advance, ally cleanse, random hit sequence, split damage, and strict forced-action ownership/trigger policy.

## Tribbie — Quantum / Harmony — Released

- **Core loop:** Skill grants the party All-Type RES PEN. Ultimate creates a Zone that increases enemy damage taken and deals additional damage whenever allies attack enemies in the Zone. Talent launches a follow-up after another ally uses an Ultimate, with a trigger limit.
- **Engine contract:** Team aura and enemy-field modifiers have owner-turn durations; Zone additional damage triggers once per qualifying attack and cannot trigger itself; ally Ultimate use queues a gated follow-up.

## Welt — Imaginary / Nihility — Released

- **Core loop:** Skill bounces and can Slow. Ultimate deals AoE damage and Imprisons enemies, delaying actions and reducing SPD. Talent adds damage when hitting slowed enemies.
- **Engine contract:** Seeded per-hit debuff attempts, action delay plus SPD debuff, deterministic bounce, and target-state additional damage are required.

## Xueyi — Quantum / Destruction — Released

- **Core loop:** Allied Toughness reduction grants Karma based on the amount reduced, with limits per action; at full Karma Xueyi performs a bounce follow-up. Ultimate deals universal Toughness damage and gains damage from how much Toughness it reduces. Break Effect converts into ordinary damage bonus through a Trace.
- **Engine contract:** Toughness events expose attempted/effective reduction and source, fractional tally policy is explicit, follow-up threshold overflow is handled, and universal Toughness attacks retain their damage element.

## Yanqing — Ice / The Hunt — Released

- **Core loop:** Skill activates Soulsteel Sync, raising CRIT stats and lowering aggro until Yanqing takes HP damage. While active, attacks can trigger an Ice follow-up and Freeze. Ultimate grants CRIT Rate and, conditionally, CRIT DMG before striking.
- **Engine contract:** Distinguish HP damage from shield damage/attack receipt for stance removal, apply seeded follow-up/debuff chances, and snapshot conditional buffs before Ultimate damage.

## Yao Guang — Physical / Elation — Released

- **Core loop:** Skill deploys a Zone that increases team Elation; Basic/Skill grant Punchline. Ultimate grants Punchline, gives Aha an extra turn using a fixed Punchline contribution, and grants party RES PEN. While Certified Banger is held, allied attacks trigger Great Boon Elation damage, with an extra trigger when the attack spends Skill Points. Her Elation Skill raises enemy damage taken and deals AoE plus random-target damage.
- **Engine contract:** Add Aha as an Elation subsystem actor, Zone-based team stat, per-attack SP-spend inspection, non-action additional Elation damage, extra-turn scheduling, and seeded random target selection.

## Yukong — Imaginary / Harmony — Released

- **Core loop:** Skill grants two Roaring Bowstrings stacks, providing team ATK; one stack is consumed after each ally turn except selected non-turn actions. Ultimate, when Bowstrings is active, also grants team CRIT Rate/DMG before attacking. Basic and a Trace interact with a periodic enhanced Toughness attack.
- **Engine contract:** Buff stacks are consumed by precisely defined turn-completion events, not every action; Ultimate samples the pre-spend state, and a turn counter controls enhanced Basic Toughness.

## Yunli — Physical / Destruction — Released

- **Core loop:** Being attacked restores Energy and triggers an immediate counter against the attacker. Skill heals Yunli and deals Blast damage. Ultimate parries, taunts enemies, and grants a limited window: the next qualifying attack triggers a powerful Intuit counter, with different programs depending on whether an enemy actually attacked before the window expires.
- **Engine contract:** Attack-received counter priority, attacker retarget fallback, taunt, interrupt-window state, and expiry at the next unit turn/action boundary must be explicit. Her Ultimate may exceed normal Energy cap behavior through authored resource rules.
