# Character Action-Flow Matrix

This document audits player-visible action flow for every public Version 4.4
combat form as of 2026-08-05. It answers a narrower question than the mechanic
profiles: when an action is requested, where may the simulation wait for another
external decision, and which later work is a new action rather than a continuation
of the original one?

The public roster contains 90 released combat forms. The current executable
reference baseline still contains 88; Rin Tohsaka and Gilgamesh remain outside
that baseline until their released data is promoted through the normal
Excel/Sora validation path. A row here is a behavioral requirement, not a claim
that its coefficients or Rule IR are already `DataReady`. Mechanic claims derive
from the compact profiles and the central [source ledger](../sources.md).

## Flow boundaries

The combat core must keep these boundaries distinct:

1. **Queued manual action**: an adapter may buffer an Ultimate click during
   presentation, then submit it when the core exposes the next legal interrupt
   boundary. Selecting that offered Ultimate reserves the prepared action; it has
   not selected targets, paid costs, or started an action.
2. **Prepared action**: the queued request owns the next decision and collects
   an ability variant, enhancement level, target selection, or confirmation.
   Cancellation policy is authored. No `ActionDeclared` event or cost payment has
   occurred yet.
3. **Action frame**: the action has been accepted, declared, and may have resolved
   one or more complete segments. Only a genuinely segmented action may persist
   here while waiting for input between segments.
4. **Post-action continuation**: counters, follow-ups, linked-actor actions,
   transformations, action advance, and extra turns are queued work after the
   original `ActionResolved`. They must not keep its action frame open.

An atomic hit, operation, trigger drain, and committed segment never waits for
external input. Presentation may animate or buffer requests during those units,
but authoritative resolution reaches the next declared boundary first.

## Flow classes

| Code | Meaning | Required state |
|---|---|---|
| `A` | Atomic after the initial target commitment; authored hits, bounces, and branches need no further input. | Ordinary finite `ActionPlan`. |
| `P` | Prepared choice before declaration, such as selecting one of several costs/programs. | `PreparedAction`, then lower the selected finite program. |
| `S` | Segmented action with an external decision between resolved segments. | Persistent `ActionFrame` with a bounded cursor and committed prior inputs. |
| `N` | A completed non-turn-ending action reopens the owning turn's normal decision. | Typed turn continuation; the completed action frame is discarded. |
| `D` | A completed action dispatches one or more child actions. A child may own its own prepared choice or segmented frame. | Ordered child-action queue with explicit source and continuation. |

The continuation column uses `R` for automatic reaction/follow-up/counter,
`X` for action advance or extra turn, `F` for transformation or countdown,
`L` for summon/memosprite/linked actor, and `I` for an ability injected into
another actor. These are not additional external-input classes.

The 90 Ultimate rows divide into 83 atomic flows, two prepared program choices
(Argenti and Physical Trailblazer), two segmented flows (Acheron and Feixiao),
and three child-action dispatchers (Cyrene, Elation Trailblazer, and Yao Guang).
Outside Ultimates, Imbibitor Lunae needs a prepared enhancement composer, while
Archer, Blade, Qingque, and provisionally Sparxie complete a non-turn-ending
action and reopen the current turn. This is why one universal resumable action
frame would be broader than the observed requirement.

## Full roster audit

| # | Combat form | Ultimate flow | Other player-facing flow | Continuation after the action |
|---:|---|---|---|---|
| 1 | Acheron | `S`: choose a target for each Rainblade stage, then resolve the automatic finisher. | Ordinary actions are atomic. | The sequence remains one Ultimate action; Knot transfer on defeat follows authored retarget policy. |
| 2 | Aglaea | `A`: enter the empowered state without further input. | Enhanced Basics remain ordinary target choices on later turns. | `F+L`: Garmentmaker persists and a countdown later ends the state. |
| 3 | Anaxa | `A`: apply Weakness state and resolve AoE damage. | Skill bounce targets are deterministic runtime selection, not player input. | `R`: a qualified hit may queue a free Skill with an explicit gate. |
| 4 | Archer | `A`: choose the initial enemy, deal damage, and seed Charge. | `N`: each Skill cast completes, then the same turn may offer another Skill/target or a manual stop while SP permits. | `R`: later allied attacks may consume Charge for a follow-up. |
| 5 | Argenti | `P`: choose the lower- or higher-cost Ultimate program before declaration; the selected program is atomic. | Ordinary Basic and Skill actions are atomic. | The high-cost random strikes are authored bounce work inside the same action. |
| 6 | Arlan | `A`: choose the primary target and resolve Blast damage. | Skill's nonlethal HP cost is committed atomically. | None. |
| 7 | Ashveil | `A`: choose Bait and resolve the authored Ultimate segment. | Ordinary actions are atomic. | `D+R`: queue the free enhanced follow-up and its bounded Gluttony-spend loop; kill retargeting is automatic. |
| 8 | Asta | `A`: apply the timed team SPD buff. | Skill bounces and unique-target accounting are automatic. | None beyond effect duration. |
| 9 | Aventurine | `A`: choose one enemy, damage it, apply Unnerved, and grant Blind Bet. | Ordinary actions are atomic. | `R`: later attack-received events may queue his follow-up. |
| 10 | Bailu | `A`: heal the party and apply Invigoration. | Skill's bounce heals are deterministic runtime selection. | `R`: reactive heals and defeat interception are independent reactions. |
| 11 | Black Swan | `A`: resolve AoE damage and Arcana classification/reset effects. | Ordinary actions are atomic. | DoT ticks are later effect work, not held action state. |
| 12 | Blade | `A`: choose the primary enemy, set HP, sample the tally, and resolve Blast damage. | `N`: Skill completes without ending the turn, installs Hellscape, and reopens the turn for the replacement Basic. | `F+R`: stance duration and the Charge follow-up outlive their creating actions. |
| 13 | Boothill | `A`: choose one enemy, implant Physical Weakness, damage, and delay it. | Standoff and the replacement Basic are ordinary target decisions on later actions. | Duel state and action delay persist as effects. |
| 14 | Bronya | `A`: apply team buffs. | Skill is one atomic ally-target action followed by that ally's timeline change. | `X`: the selected ally receives immediate action through the scheduler. |
| 15 | Castorice | `A`: consume Newbud and summon Netherwing. | Dragon commands, if controller-selectable, are separate linked-actor turns. | `L+R`: dragon actions, defeat interception, and disappearance attack are queued independently. |
| 16 | Cerydra | `A`: apply the team Ultimate effect. | Skill chooses one ally atomically. | `D+R`: Coup de Main is a copied child Skill with fixed source, cost, and target policy. |
| 17 | Cipher | `A`: release the recorded tally through one finite damage program. | Patron assignment and Skill targeting are atomic. | `R`: later ally attacks may queue Cipher's follow-up. |
| 18 | Clara | `A`: apply self protection, aggro, and finite enhanced-counter charges. | Ordinary actions are atomic. | `R`: future attacks queue counters; no Ultimate action remains open. |
| 19 | Cyrene | `D`: resolve Cyrene's own activation, then dispatch teammate Ultimate child actions in stable order. | Each child receives its own target/variant/segment decisions instead of borrowing Cyrene's action frame. | `F+L+X`: Zone, Demiurge, Odes, and memosprite turns persist independently. |
| 20 | Dan Heng | `A`: choose one slowed or unslowed enemy and resolve conditional damage. | Ordinary actions are atomic. | None. |
| 21 | Dan Heng • Imbibitor Lunae | `A`: Ultimate is an ordinary Blast action that later grants Squama Sacrosancta. | `P`: before a Basic is declared, compose enhancement tier 0–3, target, and SP/Squama payment; cancellation restores the uncommitted choice. | No Ultimate frame is retained; enhanced Basics are later ordinary actions. |
| 22 | Dan Heng • Permansor Terrae | `A`: resolve AoE damage, shields, and dragon enhancement. | Skill atomically chooses Bondmate. | `L+R`: Souldragon acts later and may emit joint follow-ups for finite charges. |
| 23 | Dr. Ratio | `A`: choose one enemy, deal damage, and apply Wiseman's Folly. | Skill is atomic; its seeded follow-up test occurs during resolution. | `R`: allied attacks may consume mark charges and queue Ratio's follow-up. |
| 24 | Evanescia | `A`: resolve the authored AoE and Elation additions. | Ordinary actions are atomic. | `R`: resource threshold crossings may queue Master Fox follow-ups. |
| 25 | Evernight | `A`: create Darkest Riddle and resolve the authored Ultimate effects. | Skill is atomic and may summon or heal Evey. | `L+X+R`: Evey's threshold action and disappearance are independent queued work. |
| 26 | Feixiao | `S`: choose and lock the primary target, then choose Boltsunder Blitz or Waraxe Skyward for each of six complete strikes before the automatic finisher. | Ordinary actions are atomic. | The target commitment remains fixed for the segmented Ultimate; no arbitrary per-strike retarget is offered. |
| 27 | Firefly | `A`: complete transformation entry. | Replacement Basic and Skill are normal decisions during later turns. | `F+X`: advance Firefly and schedule a countdown that ends Complete Combustion. |
| 28 | Fu Xuan | `A`: resolve AoE damage and restore a self-heal charge. | Skill atomically deploys Matrix of Prescience. | Damage transfer and threshold healing are later effect/reaction work. |
| 29 | Fugue | `A`: resolve all-enemy damage and universal Toughness reduction. | Skill atomically chooses the Foxian Prayer ally. | Exo-Toughness and Super Break hooks persist as effects. |
| 30 | Gallagher | `A`: resolve AoE damage, apply Besotted, and enhance the next Basic. | Ordinary actions are atomic. | `X`: Gallagher is advanced; the enhanced Basic is a later normal action. |
| 31 | Gepard | `A`: apply party shields. | Ordinary actions are atomic. | Defeat interception is a later battle-scoped reaction. |
| 32 | Gilgamesh | `A`: resolve AoE damage followed by authored random bounces without player input. | Before Interest Piqued, his turn-start Basic is automatic; afterward the available Skill is an ordinary action. | `R`: Saber/Gilgamesh attack tally may queue a joint follow-up. |
| 33 | Guinaifen | `A`: resolve AoE damage and partial Burn detonations. | Ordinary actions are atomic. | DoT ticks and Firekiss stacking are later effect work. |
| 34 | Hanya | `A`: choose one ally and apply the SPD/ATK buff. | Skill atomically chooses and marks one enemy. | Burden counters and SP refund are later effect reactions. |
| 35 | Herta | `A`: resolve all-enemy damage. | Ordinary actions are atomic. | `R`: each newly crossed HP threshold may queue a follow-up. |
| 36 | Himeko | `A`: resolve all-enemy damage and deterministic defeat-based Energy gains. | Ordinary actions are atomic. | `R`: Weakness Break Charge may queue a follow-up. |
| 37 | Himeko • Nova | `A`: execute a finite scripted multi-attack program without mid-sequence player input. | `I`: allies may receive an Assist ability, but each Assist is a separate ordinary command with caller/source split. | Companion clauses and shared-use updates are effects or child actions. |
| 38 | Hook | `A`: choose one target, deal damage, and prime the next Skill. | Ordinary actions are atomic. | The enhanced-Skill flag is consumed by a later normal action. |
| 39 | Huohuo | `A`: restore team Energy and apply the ATK buff. | Skill atomically chooses its heal center. | Divine Provision heals are later reactions to turn/Ultimate events. |
| 40 | Hyacine | `A`: raise party Max HP, heal, and empower Little Ica. | Skill is atomic. | `L`: summon actions and shared healing tally remain independent. |
| 41 | Hysilens | `A`: deploy the Zone and resolve the initial effect. | Ordinary actions are atomic. | Zone-triggered DoT additions are guarded later reactions. |
| 42 | Jade | `A`: resolve AoE damage and empower finite follow-ups. | Skill atomically chooses Debt Collector. | `R`: Charge threshold queues follow-ups; collector damage is inline additional damage. |
| 43 | Jiaoqiu | `A`: equalize stacks, deal AoE damage, and create the Zone. | Ordinary actions are atomic. | Enemy-turn Zone applications and DoT ticks are later effect work. |
| 44 | Jing Yuan | `A`: resolve AoE damage and increase Lightning-Lord's hit count. | Ordinary actions are atomic. | `L`: Lightning-Lord retains its own timeline turn. |
| 45 | Jingliu | `A`: choose a Blast target, deal damage, and gain Syzygy. | Replacement Skills are ordinary decisions after transformation. | `F+X`: a threshold may atomically enter Spectral Transmigration and advance Jingliu. |
| 46 | Kafka | `A`: resolve AoE damage, Shock, and authored DoT detonations. | Skill is atomic. | `R`: an ally Basic may later queue Kafka's gated follow-up. |
| 47 | Lingsha | `A`: resolve AoE/heal/Befog and advance Fuyuan. | Ordinary actions are atomic. | `L+R`: summon and emergency actions are queued separately. |
| 48 | Luka | `A`: choose one enemy, damage it, and apply vulnerability. | Replacement Basic is selected as an ordinary later action. | Bleed ticks are later effects. |
| 49 | Luocha | `A`: mass-dispel enemies, deal AoE damage, and grant Abyss Flower. | Skill is atomic. | `R`: emergency Skill and Field healing are independently queued reactions. |
| 50 | Lynx | `A`: heal and cleanse the party. | Skill atomically chooses one ally. | Healing-over-time and aggro state persist as effects. |
| 51 | March 7th (Preservation) | `A`: resolve AoE damage and seeded Freeze attempts. | Skill atomically chooses one shield recipient. | `R`: later attacks against shielded allies may queue counters. |
| 52 | March 7th (The Hunt) | `A`: choose one enemy, deal damage, and empower the next enhanced Basic. | Skill atomically chooses or reassigns Shifu. | `X`: Charge threshold queues March's immediate action. |
| 53 | Misha | `A`: snapshot the hit count and execute deterministic bounce selection. | Ordinary actions are atomic. | None after the Ultimate reset. |
| 54 | Mortenax Blade | `A`: pay HP, resolve damage/debuffs, and enter Infinite Fury. | Replacement abilities are normal choices while the Zone exists. | `F+X+R`: countdown exit and threshold free Skill are queued independently. |
| 55 | Moze | `A`: resolve the target attack and force the authored follow-up. | Skill atomically chooses Prey and changes presence. | `D+R`: forced follow-up, ally-triggered damage, and timeline re-entry are separate work. |
| 56 | Mydei | `A`: resolve the authored Ultimate against its committed targets. | Replacement Skills are normal decisions during Vendetta. | `F+X+D`: thresholds enter Vendetta or automatically dispatch Godslayer Be God. |
| 57 | Natasha | `A`: heal the party. | Skill atomically chooses one ally. | Healing-over-time persists as an effect. |
| 58 | Pela | `A`: resolve AoE damage and apply Exposed. | Skill atomically chooses one enemy to dispel. | None beyond effects. |
| 59 | Phainon | `A`: complete transformation entry and party departure. | Each Khaslana ability is selected on a later granted turn; they are not segments of the Ultimate. | `F+X`: Territory owns a bounded extra-turn batch and guaranteed restoration/final strike. |
| 60 | Qingque | `A`: resolve AoE damage and replace the hand with matching tiles. | `N`: each Skill draw completes without ending the turn, then the turn decision reopens for another draw or the current Basic. | The enhanced Basic is a later normal action, not an open Skill frame. |
| 61 | Rappa | `A`: enter Sealform. | Enhanced Basics are later action choices; their internal three-part hit program is automatic. | `F+X`: grant an extra turn and retain finite replacement-Basic uses. |
| 62 | Rin Tohsaka | `A`: choose a primary enemy, resolve primary-plus-other damage, restore SP, and apply vulnerability. | Enhanced Skill automatically spends the authored SP/resource amounts and repeats random bounces to a bound; it requests no per-bounce input. | `R`: Archer linkage may queue a joint follow-up. |
| 63 | Robin | `A`: complete Concerto entry and its Ultimate effects. | Robin has no normal action decision while absent from the timeline. | `F+X`: batch-advance other allies in stable order and end Concerto through a countdown. |
| 64 | Ruan Mei | `A`: create the field and apply team effects. | Ordinary actions are atomic. | Weakness-recovery interception is later effect work. |
| 65 | Saber | `A`: execute the finite multi-stage Ultimate program without external choices between stages. | Ordinary actions are atomic. | `R+X`: resource thresholds and collaboration hooks may queue later actions. |
| 66 | Sampo | `A`: resolve AoE damage, Wind Shear, and DoT vulnerability. | Skill bounce selection is automatic. | DoT ticks are later effects. |
| 67 | Seele | `A`: enter the buffed state and resolve one target attack. | Ordinary actions are atomic. | `X`: qualifying defeat may queue a gated Resurgence extra turn. |
| 68 | Serval | `A`: resolve AoE damage and extend existing Shock durations. | Ordinary actions are atomic. | DoT ticks are later effects. |
| 69 | Silver Wolf | `A`: choose one enemy, resolve damage, and apply DEF reduction. | Skill's eligible-element and Bug choices are seeded runtime selection. | Weakness and Bug state persist as effects. |
| 70 | Silver Wolf LV.999 | `A`: complete transformation and Zone entry. | Replacement Basics and Elation Skill are later normal action choices; the long hit plan is automatic. | `F+X+R`: action advance and Loot Box reactions persist independently. |
| 71 | Sparkle | `A`: restore team SP and enhance Cipher. | Skill atomically chooses one ally and changes its gauge. | `X`: action advance is scheduler work after the Skill. |
| 72 | Sparxie | `A`: resolve AoE damage and grant Punchline. | `N`: livestream/Engagement Farming is a bounded pre-attack repeat loop; whether every repeat/stop is externally selectable remains an observation requirement. | Ability replacement and Thrill persist as state. |
| 73 | Sunday | `A`: choose one ally, restore Energy, and apply The Beatified. | Skill atomically chooses one ally. | `X+L`: Skill may advance both the ally and its linked summon/memosprite. |
| 74 | Sushang | `A`: choose one enemy, deal damage, and apply the authored buffs. | Ordinary actions are atomic. | `X`: Sushang receives immediate action through the scheduler. |
| 75 | The Dahlia | `A`: resolve AoE effects and dynamically implant both partner elements. | Skill creates the Zone and partner link atomically. | `R`: alternating partner attacks may queue Dahlia's follow-up. |
| 76 | The Herta | `A`: redistribute Interpretation, resolve AoE damage, and grant Inspiration. | The enhanced Skill is a later ordinary target decision. | `X`: fully advance The Herta after Ultimate resolution. |
| 77 | Tingyun | `A`: choose one ally, restore Energy, and apply its damage buff. | Skill atomically chooses Benediction's owner. | Benediction additional damage is inline later reaction work. |
| 78 | Topaz & Numby | `A`: empower a finite number of Numby actions. | Skill atomically chooses or transfers Proof of Debt. | `L+X`: Numby acts on its own timeline and accepts authored advances. |
| 79 | Trailblazer (Destruction) | `P`: after requesting the Ultimate, choose single-target Basic-style or Blast Skill-style program and its legal target before declaration. | Ordinary actions are atomic. | None. |
| 80 | Trailblazer (Preservation) | `A`: resolve AoE damage and prime the enhanced Basic. | Ordinary actions are atomic. | Replacement Basic availability persists for a later turn. |
| 81 | Trailblazer (Harmony) | `A`: apply Backup Dancer to the party. | Skill bounce selection is automatic. | Super Break emissions are later inline effect work. |
| 82 | Trailblazer (Remembrance) | `A`: resolve AoE damage and charge Mem. | Skill atomically summons or heals Mem. | `L+X`: threshold Mem action and ally advance are separate queued actions. |
| 83 | Trailblazer (Elation) | `D`: choose one ally and resolve buff/cleanse, then dispatch that ally's forced Elation Skill or timeline advance according to ally type. | The Trailblazer's Elation Skill has automatic random and split targeting. | `X`: non-Elation allies are advanced; forced child-action ownership remains explicit. |
| 84 | Tribbie | `A`: create the Zone and resolve the initial effect. | Ordinary actions are atomic. | `R`: Zone damage and ally-Ultimate follow-up are later reactions. |
| 85 | Welt | `A`: resolve AoE damage, Imprisonment, and action delay. | Skill bounce and seeded debuff attempts are automatic. | Timeline delay persists as effect/scheduler state. |
| 86 | Xueyi | `A`: choose one enemy and resolve universal Toughness reduction plus sampled damage scaling. | Ordinary actions are atomic. | `R`: Karma threshold may queue a deterministic bounce follow-up. |
| 87 | Yanqing | `A`: choose one enemy, snapshot conditional CRIT buffs, and resolve damage. | Ordinary actions are atomic. | Soulsteel and seeded follow-up state persist independently. |
| 88 | Yao Guang | `D`: resolve the team effects, then grant Aha a separate extra turn with fixed contribution. | Elation Skill random targets are automatic. | `X+R`: Aha's turn and Great Boon triggers are separate scheduler/reaction work. |
| 89 | Yukong | `A`: sample Roaring Bowstrings, apply buffs, and attack the chosen enemy. | Ordinary actions are atomic. | Bowstrings consumption is tied to later qualifying turn completions. |
| 90 | Yunli | `A`: apply parry, taunt, and a finite pending-counter window, then resolve the Ultimate. | Ordinary actions are atomic. | `R`: the next qualifying enemy attack or expiry boundary chooses and queues the Intuit counter program. |

## Required command and state model

The audit does not justify a character-specific command enum or a scripting
runtime. It requires a small typed protocol:

```text
ActionBoundary
  -> RequestUltimate(actor, ability)
  -> PreparedAction(action_request_id, actor, ability, origin, continuation)
  -> ActionInput(SelectOption | SelectTargets | SelectCount | Confirm | Cancel)
  -> ActionPlan
  -> optional ActionFrame(cursor, committed_inputs, paid_cost, continuation)
  -> ActionResolved
  -> ordered child/reaction/timeline work
```

`ActionInput` payloads are constrained by immutable catalog definitions. Legal
options are finite, canonically ordered, and offered by the core; a controller
selects an offered value rather than constructing an equivalent payload.
`SelectTargets` carries a typed target-selection specification, not an arbitrary
list of unit IDs. `SelectOption` refers to an authored option ID whose cost,
program, and target policy are known to the catalog.

A finite `SegmentedActionDefinition` is a bounded linear sequence of
`SelectTarget`, `SelectOption`, and `Automatic` steps. It may not contain
callbacks, character IDs, nested segmented actions, unbounded loops, or adapter
state. The runtime frame stores IDs, a cursor, prior commitments, payment state,
and the suspended continuation; configuration lookup and UI presentation remain
outside canonical mutation.

## Consequences for the current baseline

The prepared-action boundary separates Ultimate request from target selection,
declaration, payment, and execution. Ordinary actions still use one complete
atomic `ActionPlan`. The two confirmed segmented families instead declare and
pay one parent action, execute complete segment plans under its `ActionId`, and
persist `ActionFrameState` only while the next typed input is pending.

Non-turn-ending Skill loops reopen a turn decision, while transformations,
counters, summons, child actions, and extra turns remain in their existing
effect/reaction/timeline owners and do not borrow the parent's frame.

## Implementation acceptance

- stale, unoffered, wrong-owner, or invalid action inputs leave state, IDs, RNG,
  resources, and the pending continuation byte-identical;
- cancelling a prepared action follows its authored policy and cannot refund a
  cost that was never committed;
- accepting the final prepared input declares exactly one action and pays each
  cost exactly once;
- an Acheron fixture records three independently offered target commitments and
  one automatic finisher under one action identity;
- a Feixiao fixture records one target commitment, six independently offered
  strike options, and one automatic finisher under one action identity;
- atomic, non-turn-ending, transformation, counter, linked-actor, and extra-turn
  fixtures assert that no segmented action frame survives `ActionResolved`;
- Cyrene and other dispatchers prove stable child order and give every child its
  own decision/action identity;
- canonical codec, replay, state hash, inspector snapshot, and diagnostics expose
  queued/prepared/frame state through IDs and typed cursors without resolving UI
  labels or content records;
- catalog validation rejects empty or oversized flows, noncanonical options,
  missing/self/nested segment references, non-extra-action segment abilities,
  resource-bearing segment templates, and incompatible target or invalidation
  policies.

## Open observation requirements

- exact cancellation points after queueing an Ultimate and after committing its
  first prepared input;
- Acheron's target legality and retarget fallback after each completed stage;
- Feixiao's target invalidation behavior during the locked six-strike sequence;
- stable order and controller prompts when Cyrene activates several teammate
  Ultimates with different target/variant requirements;
- whether every Sparxie Engagement Farming repeat is manually chosen or some
  repeats are automatic;
- buffering and display timing are adapter concerns, but observations must map
  them to the authoritative queue/prepared/segment boundaries above.

These observations refine catalog policy; they do not permit presentation frames
or animation timestamps to enter canonical combat state.
