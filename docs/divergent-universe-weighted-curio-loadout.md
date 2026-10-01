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
definitions enter actual Rule IR; the other fourteen effects still reject.
Unsupported equipment cannot become a digest-only, no-effect battle.
After lawful unequip, the existing real proxy battle pipeline works normally.

Fixtures cover all 17 accepted selections, replacement/unequip, one/three-slot
bounds, canonical order, fresh command reconstruction, rejected requests and
dirty-state rejection in both families. The normal proxy battle after unequip
does not establish a Weighted Curio effect. Separate actual attack probes verify
the lowered splash; they are not Forge admission or a complete public-run gate.
Forge offers, slot-level admission, domain enhancement, the other fourteen effects and encoded equipment-command replay remain
unimplemented. No source obligation, mechanic program or semantic family is
terminalized; genuine Grand Miracle acceptance remains separate and incomplete.
