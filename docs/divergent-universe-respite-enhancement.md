# Fixed Respite Blessing enhancement

## Current executable boundary

`respite_room_compiler(workbench, policy)` builds an optional enhancement menu
only at an exact current fixed Respite context. It checks the source layout,
area/layer/position join and selected Workbench's released `BuffEnhance`
membership. An altered context, unknown Workbench or transformation-only
Workbench rejects. Optional offered [Blessing overwrite](divergent-universe-respite-reforge.md)
and [Equation overwrite](divergent-universe-respite-equation-reforge.md) can share
this room in either attachment order; remaining Respite services are incomplete.

The source identifies Respite as a non-levelable room offering Equation reset,
Blessing reset, enhancement and purchase. The Workbench function identifies
Heat as enhancement currency and resets Heat at each Workbench. These facts do
not disclose an NPC placement selector, an initial Heat allowance or a price.
No automatic HP, Energy, technique-point recovery or revival is inferred.

The compiler uses the existing immutable production catalogs, shared Activity
offers and typed operations. It introduces no mode state machine, RNG stream,
handler, global slot, content identity or Excel reader. Each optional overwrite
adds four disjoint caller-owned declarations through shared logical/physical scopes.
The selected Workbench and
numeric policy are explicit host configuration inputs to the existing
accepted-price boundary, not new factual rows or original selector evidence.
Production static price/placement authoring and default-profile admission
remain pending; this does not bypass their Excel/Sora ownership.

## Explicit replaceable policy

Accuracy is
`VersionedProjectPolicyExplicitWorkbenchAdmissionHeatAllowanceAndFlatPrice`.

- The caller selects one current enhancement-capable Workbench and supplies
  a nonnegative Heat allowance and a positive flat Heat price as checked
  integers bounded by `i64`. Zero allowance is legal and disables enhancement;
  without an eligible optional service it offers only Leave.
- Entry runs the ordinary Curio domain-entry prefix once, initializes the
  service and replaces its previous Heat with the selected allowance.
  Workbench identity retains its existing physical-node scope: each menu
  reasserts the selected immutable Workbench after the generic node reset.
  Pagination and upgrade returns never initialize the allowance again.
- Menus partition all 414 stable identities into four pages of at most 128
  enhancement choices, below the shared 256-option limit. Each page offers its
  owned affordable base-level Blessings. Navigation exposes another page only
  when that page has a legal enhancement, with its condition required again
  during traversal. All pages remain reachable; no identity is truncated.
  An active Blessing offer or dirty Equation progress disables enhancement.
  Availability is required again inside each accepted option, not merely
  trusted from a previously published menu.
- An upgrade changes only the selected identity's level to two, debits Heat,
  increments the existing enhancement-function receipt and republishes the
  menu in one transaction. The ownership identity and Path counts are
  unchanged; no Equation progress refresh, expansion grant or RNG draw occurs.
- Leave is always available. It clears Workbench identity and remaining Heat
  before entering the next position. Optional services need not be purchased
  to leave. A failed destination entry rolls these changes back with traversal.
- The mathematical enhancement bound is one upgrade per catalog identity.
  Each page allows 415 visits and each repeat/navigation edge 414 traversals;
  redundant navigation can exhaust that compiler resource budget and reject
  without mutation. Leave remains available through a dedicated single-visit
  automatic exit, independent of page budgets. These are explicit compiler
  bounds, not observed original-game visit caps. Arbitrary trusted inventory
  rewrites are not room-offered commands.

The flat price avoids inventing rarity prices, discounts or escalation rules.
The explicit allowance avoids silently giving zero-cost enhancements or
claiming a recovered original budget. No tutorial, neighbouring NPC ID or
other Universe mode supplies missing admission. Confidence in numeric parity
is deliberately absent. Replace these inputs with reviewed production rows
when released evidence supplies the Respite Workbench selector, initial Heat
and exact prices/modifiers; replace the allowance model independently if the
published rule instead describes accumulated Heat and a capacity.

## Composition and identity

Supply `CompiledRespiteRoom::fragment()` through
`compile_curio_domain_route`. The single entry, four menu pages and automatic
exit share one logical room, so internal returns and pagination do not consume
Curio lifetimes or reset Heat. The single-visit entry stays small and does not
duplicate the full enhancement menus across Curio lifecycle branches.

The owning profile includes `configuration_digest()` in its payload. This
binds the current source and decision catalogs, exact position namespace,
selected Workbench, allowance and price. Call `validate_definition()` on the
completed immutable graph, then use the existing exact whole-profile battle
and deck bindings. Validation rejects changed programs, missing lifecycle
prefixes, bypass entries/exits, extra random policies and wrong logical scopes.
Whole-profile binding retains ownership of all slot and configuration checks.
Attach the exact validated room list with `bind_position_respite_rooms` to
the already-bound battle profile. Duplicate rooms, mismatched policies, changed
catalogs or a second attachment reject. This immutable capability does not
change identity or live state, and an earlier flow clone gains no authority.

The shared `Service` options contain all payment and level operations.
`choose_respite_service_option` authenticates the whole definition and uses
the existing generated-option transaction with an empty state prefix. Its
rollback includes automatic destination entry, not merely the selected
option. The baseline runner dispatches to this bound capability. Stale, hidden
or no-longer-affordable choices reject; late receipt overflow rolls back the
upgrade, debit, menu traversal and RNG. Raw `GraphActivity::choose_option`
retains the shared low-level accepted-command fault semantics and is not this
mode-executor interface; no new fault behavior is introduced in the engine.

## Released evidence

Pinned repository:
`Dimbreath/turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568`,
released Version 4.4, inspected 2026-09-26. Current factual locators are:

| File | Exact locator | SHA-256 |
|---|---|---|
| `ExcelOutput/RoguePersonaRoomCompType.json` | composition 10, Respite; description hash 184854953851317551 and non-levelable hash 8808349201104714957 | `c0223603c6e5252278dac5224d766f1c4ed60ebe5845b9839b9e3533a8ad76a5` |
| `ExcelOutput/RogueTournWorkbenchFunc.json` | FuncID 1, BuffEnhance; description hash 14227153442758834295 | `b430ce650040a1b2cf5c262f8f69f8e9d15bf6632b81c4947408026e33363078` |
| `ExcelOutput/RogueTournWorkbench.json` | WorkbenchID 101, FuncList [2, 1]; independently selected, not proven Respite placement | `26053803e691fe1fa7be57bf94d1d766ff0cc3e6cebda579fa013b129775d736` |
| `TextMap/TextMapEN.json` | the three description hashes above | `afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789` |
| `TextMap/TextMapCHS.json` | the same hashes; independent project summary: 休整可重置方程与祝福、强化及购买祝福；强化使用调试台热量 | `ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147` |

The [current public domain summary](https://honkai-star-rail.fandom.com/wiki/Divergent_Universe%3A_Arcadian_Chronicles/Domains)
and [current guide](https://game8.co/games/Honkai-Star-Rail/archives/457407),
accessed 2026-09-26, cross-check room services and Heat. Indexed public text was
available; it provides neither immutable version-specific price evidence nor
an archived full-page digest. Exact execution relies on the pinned source,
not an assumed price from those pages or historical Protean Hero guides.

## Verification and remaining scope

`tests/battle_room/respite` fixtures execute the fixed room after two real
proxy battles, perform repeated paid enhancements, leave and complete the
third battle in Ordinary and Cyclical. Separately compiled profiles compare
every boundary's canonical state. An inventory fixture covers all 414 base
identities; it supplies missing identities through a trusted accepted reward
boundary and does not claim the room grants them. Tests also cover empty
allowance, pending/invalid availability, stale/hidden/repeated commands,
late overflow, context changes and exact entry-prefix validation.
Failed next-domain entry also restores Leave's Heat/Workbench changes while
retaining an enhancement committed by an earlier command.

Other reachable rooms remain explicit probes in these controlled profiles.
Original selectors, remaining Respite services, complete source-position
gameplay, default CLI/Agent/MCP topology and encoded profile replay are still
incomplete. No source or mechanic obligation is terminalized by this slice.
