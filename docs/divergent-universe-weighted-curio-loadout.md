# Divergent Universe accepted Weighted Curio loadouts

## Current executable boundary

`weighted_curio_runtime` loads the 17 current Sora-derived Weighted Curio
identities, not ordinary Curios or Grand Miracles. Its accepted-service boundary
`replace_accepted_loadout` atomically replaces the equipped set through shared
Activity operations. Empty selection unequips; unchanged, duplicate, unknown,
over-capacity, stale, foreign or completed requests reject without mutation.
The owner must authenticate a real service offer and bind its declared slot
limit before calling this trusted boundary. This is not a player-facing Forge
menu, automatic room admission or a standalone untrusted equip command.

Slot 39 now records equipped current-catalog keys with count one. Canonical
stable-ID order assigns fixed-width keys; input ordering is immaterial. The
loadout carries across planes in Run state, separately from ordinary Curio
holdings. Equipment changes emit Activity events, consume no currency or RNG,
and reconstruct from fresh production catalogs. Current configuration identity
binds the accepted-loadout policy. There is no historical key or codec contract.
Caller-admitted capacity is bounded to one through three; original level-to-slot
selection and service availability are not inferred from the supplied number.

## Released evidence and policy

Released Version 4.4 revision `fd978d6ef09f941fba644c731ab54abd6f7c3568` of
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
accessed 2026-09-27, supplies `ExcelOutput/RogueTournHex.json` rows with
`TournMode=Tourn3`: HexIDs 1001–1017. SHA-256:
`51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455`.
The actual Tourn3 selector, not the adjacent IDs, establishes membership.
All seventeen referenced MazeBuffs 633401–633417 resolve in the released generic
`ExcelOutput/MazeBuff.json`, despite their absence from `RogueMazeBuff`.
The reference catalog retains its unresolved RogueMazeBuff locator; that is not
absence of released effect data. The separate production
[splash definition](divergent-universe-weighted-curio-splash.md) lowers 633401.
Catalog identity does not implement the remaining programs. Path/element metadata is not used to
invent an eligible randomized offer pool.

`RoguePersonaRoomPreset` row 1017 joins composition type 21 / Reforge at level
one. `RoguePersonaRoomCompType` row type 21 joins English level-description
hash `17567217208297185401`: level one supports Weighted Curios and domain
enhancement; later levels add or increase other services. Type 15 / Escapade
Forge description hash `1726002884355816599` explicitly permits three equipped
Weighted Curios. The file digests and exact field joins are retained in the
[current position contract](divergent-universe-domain-layout.md). TextMap SHA-256:
`afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789`.

The same released TextMap, hash `18005246693377615542`, explicitly describes
selection of any Weighted Curio, equipment-only effects and increasing slots
with Forge level. Its values match a bounded
[current Forge tutorial cross-check](https://honkai-star-rail.fandom.com/wiki/Tutorial/Divergent_Universe%3A_Forge),
inspected 2026-09-27. The released text is stronger than that indexed community
copy, but does not supply a menu execution graph or exact per-level slot counts.
The accepted set replacement, duplicate rejection, canonical order and caller
capacity remain `VersionedProjectPolicyAcceptedCurrentCatalogLoadout`. Missing
facts include exact slot selection, menu/replacement timing and unreconstructed
effect execution programs. Released effect parameters/text are available.
Alternatives include random offers or retaining a separate inactive inventory;
the present primitive implements explicit equipment without asserting either.
Replace each policy field independently when released programs or reproducible
current observations establish it. Hidden-timing parity confidence is low.

## Battle rejection and unfinished work

Contribution snapshot construction validates this state before battle assembly.
Unlowered equipped identities return typed `UnsupportedBattleEffect`; dirty
counts, unknown keys or more than three equipped identities reject as invalid
state. The production splash and [Harmony shield](divergent-universe-weighted-curio-shield.md)
and [Automated Experience](divergent-universe-weighted-curio-attack-debuff.md)
and [The Story Presently](divergent-universe-weighted-curio-support-attack.md)
and [Road of Prayers](divergent-universe-weighted-curio-prayer.md)
and [Self-Amusement](divergent-universe-weighted-curio-retaliation.md)
and [Converse of Entropy](divergent-universe-weighted-curio-break-effect.md)
and [Mock Crimson Moon](divergent-universe-weighted-curio-necrosis.md)
and [Sapient Pen](divergent-universe-weighted-curio-elation.md)
and [Genius' Confusion](divergent-universe-weighted-curio-excitation.md)
and [Encouragement for You](divergent-universe-weighted-curio-encouragement.md)
and [Dignity and Passion](divergent-universe-weighted-curio-transfer.md)
and [Walkie-Talkie](divergent-universe-weighted-curio-overflow.md),
[Footstep of Gods](divergent-universe-weighted-curio-footstep.md) and
[Deflagration](divergent-universe-weighted-curio-deflagration.md)
definitions enter actual Rule IR: 15/17 equipment effects are supported;
the other two effects still reject.
Sapient Pen's source operands are separate from its replaceable execution and
shared-meter policies; data loading alone grants no execution credit.
Unsupported equipment cannot become a digest-only, no-effect battle.
After lawful unequip, the existing real proxy battle pipeline works normally.

Fixtures cover all 17 accepted selections, replacement/unequip, one/three-slot
bounds, canonical order, fresh command reconstruction, rejected requests and
dirty-state rejection in both families. The normal proxy battle after unequip
does not establish a Weighted Curio effect. Separate actual attack probes verify
the lowered splash; they are not Forge admission or a complete public-run gate.
Automatic Forge offers, slot-level admission, domain enhancement, the other two effects and encoded equipment-command replay remain
unimplemented. No source obligation, mechanic program or semantic family is
terminalized; genuine Grand Miracle acceptance remains separate and incomplete.

## Authenticated source-position equipment service

`weighted_curio_room_compiler(capacity, slots)` supplies an executable equipment
fragment only for a proven current `Reforge` card context. All nine authored
decks contain such level-one presets. The compiler validates exact current
area/layer/position/preset/kind/level joins; other rooms, historical presets and
upgraded levels reject. It does not infer automatic Forge placement or the
original level-to-slot selector. The caller supplies a one-through-three
capacity independently of the source level.

`VersionedProjectPolicyExplicitReforgeCapacityCanonicalToggle64Changes` adds
these independently replaceable menu rules to the accepted-loadout boundary:

- Offer all 17 identities in canonical stable-ID order, not a random pool or
  the implemented-effect subset. Selecting an equipped identity removes it;
  selecting another adds it when capacity permits. Full-capacity offers retain
  removal and clear but omit additions. Empty clear is not offered. Unsupported
  effects remain selectable and still return `UnsupportedBattleEffect` during
  later battle construction; menu selection does not invent their execution.
- Toggle and clear use the same replacement validator and typed operations as
  the trusted accepted-service API, without currency, RNG or ordinary Curio
  changes. Existing equipment above the bound capacity rejects entry atomically
  rather than silently discarding it.
- Permit 64 equipment changes per logical room, then retain an independent
  Leave that preserves equipment. Budget survives internal menu loops and resets
  at another logical room. The physical-node acceptance gate resets after each
  accepted choice; equipment remains Run-scoped.
- Authenticate the whole immutable graph and actual offered decision/option
  before mutation. Equipment, budget, gate, events and graph movement share the
  shared Activity transaction. Failed next-room entry restores pre-command
  state, pending offer and RNG.

The host supplies two isolated service slots, includes exact declarations
(including the existing production equipped slot), binds each compiled room's
configuration digest into its profile identity and composes through
`compile_curio_domain_route`. Binding requires the exact normal Curio entry
lifecycle, nodes, internal edges, sole exit, slots and logical room path.
Menu-entry bypass, added internal edges, changed programs, room RNG and
interactions reject. Freshly reconstructed identical whole definitions pass;
foreign rooms/graphs do not. The capability owns no mutable state or state machine.

The pinned released evidence above proves current identities and the level-one
Reforge capability, not these execution edges, free-toggle timing, 64-change
budget or per-level capacity. Hidden-timing parity confidence is low. Alternatives
include inactive inventory, paid confirmation, full-set confirmation and random
offers; none is asserted as observed behavior. Replace each policy field
independently when released graphs or reproducible current observations establish
menu transitions, slot selection and replacement timing.

Seven focused command/binding tests cover all 17 selections, capacities 1–3,
canonical order, clear, independent Leave, all nine decks' Reforge construction,
raw/stale/foreign/hidden/dirty rejections, exact budget exhaustion, next-entry
rollback, hostile definitions, no RNG and fresh event/state reconstruction in
both families. Other room payloads are isolated out: these tests do not complete
an original run or terminalize source obligations. Flow/controller dispatch,
encoded equipment-command replay, automatic Forge admission, domain enhancement,
divination and the two remaining effects are separate requirements.
