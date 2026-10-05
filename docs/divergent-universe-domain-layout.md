# Divergent Universe current Persona domain positions

## Current boundary

The [base domain-deck compiler](divergent-universe-domain-deck.md) provides shared
Activity draw/discard programs and nine explicitly selected authored source
decks. The production factory now also compiles these exact positions into a
shared Activity graph, with an explicitly selected deck and either caller-supplied
room programs or the owned native recipe below. The public baseline does not yet
bind its gameplay to this compiler;
automatic mask selection and complete room payloads remain absent. Tests with
room probes do not establish production room execution or terminal source coverage.

`DomainLayout` is an openpyxl-authored Sora table containing all 60 current
Persona layer-position records, joined to the 11 layers used by the current
28 Tourn3 areas. The typed data compiler exposes contiguous per-layer positions
and distinguishes 21 fixed presets from 39 unspecified positions. It rejects
missing layers, historical layer substitutions, position gaps/duplicates,
inconsistent preset kinds/levels and incorrect provenance bindings.

This is configuration readiness, not domain execution. The current public
baseline still uses its explicitly provisional one-battle-per-layer graph.
No unspecified position is silently replaced with a battle, random room or
candidate pool. Deck construction, drawing, pass spending, reusable cards,
level/beacon mutation, fixed-room execution and plane transitions must consume
these positions through the shared Activity graph before full-run readiness.

## Strict position-graph composition

`DivergentUniverseRuntimeFactory::compile_domain_route` consumes the area's
explicit ordered layer list and the current Sora `DomainLayout` records. It
preserves all 5/13/17/20 positions, fixed preset kind and level, and every distinct
card instance in the selected authored deck. At unspecified positions, copies of
the same preset share a room executor but retain separate card options. Fixed
positions neither prepare nor draw a hand. Draw/discard piles carry across plane
transitions without reinitialization; refill and short hands use the existing
explicit base-deck policies, not inferred original RNG behavior.

This compiler requires a deterministic, mode-owned `DomainRoomProgram` for every
reachable preset at every position. It does not supply default handlers, room
completion signals, candidate pools or rewards. Missing content returns
`MissingRoomProgram`. Each fragment has a bounded private node/edge/program
namespace and a single successful continuation to the next exact position.
Fragments can contain multiple execution nodes and failure/abandonment/fault
terminals, but cannot mutate deck-owned slots, cross into another fragment,
skip positions or terminate the run as completed early. Completion guards and
actual content execution belong to the room compiler, not the route compiler.

Runtime room profiles use `compile_curio_domain_route` to additionally lower the
[authored Curio entry lifecycle](divergent-universe-curio-domain-expiry.md) at each
fixed/selected room's single-visit entry. It does not add room payloads. Preparation,
draw staging, unselected alternatives and internal physical nodes do not consume
an allowance or grant entry income. Exact battle/Tawot/event bindings require the
resulting entry program. The raw composition API above intentionally supplies no
Curio lifecycle and is not sufficient for these runtime capabilities.

An [explicit battle-room compiler](divergent-universe-position-battles.md) now
supplies an actual encounter/Battle/reward fragment and validated immutable flow
binding for caller-selected stages and reward domains. It does not admit original
room pools or source-specific Bosses, fill other missing payloads or replace the
default baseline. Complete source-position profile construction/replay remains
incomplete; its real battle tests do not terminalize original room obligations.

An [explicit event-room compiler](divergent-universe-position-occurrences.md)
also supplies the existing authored occurrence choices and rewards at a
caller-selected position. Completion and door publication share the reward
transaction; Leave is separate. Original event/card membership and remaining
room payloads are not inferred, and the default baseline is unchanged.

An [authenticated equipment-room compiler](divergent-universe-weighted-curio-loadout.md#authenticated-source-position-equipment-service)
supplies actual toggle/clear/Leave commands on proven current Reforge cards,
with caller-authored capacity and exact whole-graph binding. Original slot-level
selection, automatic Forge admission and complete Forge services are not inferred.
Exact equipment capabilities now bind to the flow/controller and caller-rebuilt
bound-profile encoded replay. Automatic recipe reconstruction remains pending.

The [fixed Blank-room compiler](divergent-universe-blank-room.md) now executes
the current guide's admitted empty room without intrinsic rewards, retaining
normal Curio entry effects, ordered completion/doors and an independent atomic
Leave. Other guide rooms remain separate payload requirements; this does not
constitute a complete guide or change the default profile.
Its exact compiled capability can also attach through `bind_position_blank_rooms`
for authenticated observation and independent Leave in the same flow/controller.
Missing non-Blank payloads remain requirements, not automatically empty rooms.

An already-bound room profile can attach this complete compiled route through
`bind_position_domain_route` for [source-hand flow dispatch](divergent-universe-domain-deck.md#source-position-flow-dispatch).
The existing controller then drives sampled card choices, battles, events and
services without a test-owned deck executor. This integration does not supply
the remaining room payloads or replace the default baseline.

Preparation, draw and all physical nodes of a selected room share one logical
room instance, nested under its actual plane. Internal transitions preserve
room-bound state, while moving to the next position resets it. Physical Battle
nodes additionally enter a nested battle scope without replacing the room.
Canonical graph, program, random-offer, scope and deck contributions must be validated together by
the owning profile using `GraphActivityDefinition::new`. Its configuration
identity must bind the exact source/decision digests, explicit deck selection,
width policy and gameplay inputs. No additional state machine or RNG is introduced.

Focused tests compile all 28 areas against all nine authored decks and check
exact source positions, fixed presets and instance-to-executor mappings. Shared
Activity tests traverse the guide and both run families' 13/17/20-position
layouts, reconstruct identical commands on a fresh activity, preserve piles
through fixed rooms and plane boundaries, isolate Graph RNG draws, enforce
logical room lifetimes and roll back failed room initialization. Their supplied
two-node room probes test route composition only: they are not room gameplay,
encoded profile replay or full-run release evidence. The current baseline's
encounter/service/boss content still needs production binding before replacing
the three-battle proxy. No source obligation is terminalized by this compiler.

## Explicit native position recipe

`domain_route::profile::PositionProfileRecipe` supplies a closed, production-owned
composition through `DivergentUniverseRuntimeFactory::compile_position_profile`.
It requires a plain mapped battle-route entry, explicit deck/hand width, four
independent battle-role stage/domain selections, bounded battle and Conversion
sequence counts, an Event variant and explicit following variants, authored Shop
stock/Reward selection, a Respite workbench/enhancement policy and equipment
capacity. It accepts no arbitrary room programs, no test probes and no missing-room
fallback. The counts and capacity are caller policies, never inferred from preset
level. The composition accuracy is
`VersionedProjectPolicyExplicitNativeRoomRecipeNoMissingPayloadFallback`; each
room's existing, independently replaceable accuracy contract remains unchanged.

Original automatic deck/mask selection, room-specific stage pools, counts and
capacity selectors remain unavailable. This policy chooses explicit typed inputs
and existing native fragments to make composition executable without inventing
those facts. Alternatives are disabling composition or automatically guessing
defaults; neither supplies observed parity. Original-parity confidence is unproven.
Replace each input policy independently when released selectors/programs or
reproducible observations establish it, updating the native configuration and
fresh-profile/replay tests. No source membership is inferred from successful
composition.

Battle/Boss/Elite/Encounter use actual encounter/Battle/reward fragments; Conversion
uses its explicit loss-tolerant sequence policy. Event/Reward use authored occurrence
transactions; Shop, Coin, Adventure, Respite and fixed Blank use their existing
native compilers. Reforge currently supplies only the equipment service, not the
complete Forge. Current Adventure inputs still use externally supplied results.
Unknown selections, missing rewards and conflicting slot declarations reject the
whole construction. Current level-one/two Coin presets use their exact authored
reward rows, without filtering or downgrading the selected decks. All nine current
decks compile on the controlled Ordinary/Cyclical entries; this is not proof of
original pool or room parity. Missing native payloads in future reviewed inputs
remain typed construction errors rather than automatic replacements.

Composition authenticates every consumed native fragment's configuration digest
in stable entry-node order, exact current decision inputs, deck and width. The
existing graph/profile identity additionally binds programs, scopes and battle/
occurrence payloads; native capability binders authenticate the same immutable
definition. Inputs unused by the selected graph do not create fictitious execution
identity. Repeated identical slot declarations merge; conflicting declarations
fail. The result uses the same shared Activity graph, labeled RNG, flow dispatch,
controller and current replay engine, without a second executor or live-state
rebinding.

Focused native tests reconstruct profiles from fresh current catalogs, account for
all nine decks in both families, compile both guides and the 17/20-position
Conversion layouts, reject changed/unknown/already-attached inputs,
and run actual source rooms through equipment toggle/clear, subsequent real Burn
events and caller-bound encoded replay. Other rooms in this corpus are native
fragments, not completion probes. Stages, rewards and services remain explicit
project-policy selections; these bounded runs are not complete-run release gates.
The recipe is not yet encoded and automatically reconstructed by default replay,
and it does not replace public baseline/adapter admission. Terminal source/program
coverage remains unchanged.

## Admitted released records

The following files belong to released Version 4.4 revision
`fd978d6ef09f941fba644c731ab54abd6f7c3568` of
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
reviewed 2026-09-12. Source 18 already binds the area file; sources 64–66 bind
the three Persona files. Each authored position retains its exact row locator.

| File under `ExcelOutput/` | SHA-256 |
| --- | --- |
| `RogueTournArea.json` | `756510177a468130464c27ef382d9ab33a7098dc993022cd8647366cc7c46db8` |
| `RoguePersonaLayerRoom.json` | `651989d8fd339eec667791495e6ff608418f7d308990d02d941c698203648998` |
| `RoguePersonaRoomPreset.json` | `a4cc8bbd6e3a4db5fecb62b83a0c5de7ad2f4a9820ad5ae9180b30ae33c346b5` |
| `RoguePersonaRoomCompType.json` | `c0223603c6e5252278dac5224d766f1c4ed60ebe5845b9839b9e3533a8ad76a5` |

The reviewed join is area `GLNDIILFKBN[]` → position `CBCHIHEOEGK`;
`EEPIDJJJMAH` orders each layer's positions; optional `BKHDBIFFIKP` joins preset
`LIIPLGLNPGB`; preset `LLICIMBCNPF` joins the published composition enum.
`AAGKEBFHLMC` is interpreted as composition level. Obfuscated field
interpretation remains explicit and replaceable independently of exact values.
Confidence is high in the reproduced values and exhaustive key joins, but the
decoded field roles are reviewed inference, not observed execution. Alternatives
include display-order or preview records rather than traversal instructions;
released named schemas or executable selectors must resolve that distinction.
No RNG, cost or room-execution rule is inferred from this interpretation.
All six selected presets have empty `FJIKMHCJMKH` lists; these records do not
establish rules for adding or removing beacons.

| Current area group | Ordered layers | Position counts | Total |
| --- | --- | --- | ---: |
| Guide 103 / 104 | 103 / 104 respectively | 5 | 5 |
| Formal 401–405; weekly 20401–20405 | 3001, 3002, 3003 | 4 + 5 + 4 | 13 |
| Formal 406–408; weekly 20406–20408 | 3011, 3012, 3013 | 5 + 7 + 5 | 17 |
| Formal 409–413; weekly 20409–20413 | 3021, 3022, 3023 | 6 + 8 + 6 | 20 |

These groups are derived from each area's explicit layer list, not ID-range
membership assumptions. The counts are not a universal thirteen-room rule.
All eleven layers end in preset 1002 / Boss / level 1. Formal/weekly initial
layers start with 1001 / Battle / level 3. Their last planes put
1003 / Respite / level 1 immediately before Boss. Layers 3012 and 3022 place
1004 / Conversion / level 1 at position four. Guide first positions instead
use 9007 / Blank / level 1 and 9008 / Coin / level 1.

## Correction to the old absence inference

The retained `RogueTournLayerRoom` table has no join to these current area
layers. That fact does **not** imply that the current release has no layer
positions: the relevant records are in `RoguePersonaLayerRoom`. It also does
not justify treating legacy Tourn2 rooms as current candidates. Current
position membership, a fixed composition preset, the unspecified-position
selector and actual room effects are separate claims.

The reference generators no longer blanket-exclude `RoguePersona`. Eleven
explicitly reviewed files are retained as structured candidates, and the
`persona_source_obligations` category accounts for their 547 rows. The current
reference manifest therefore contains 6,762 obligations across 51 categories;
all pre-existing 6,215 obligations are preserved. This repairs source accounting,
not runtime promotion. Source-only normalized records now also travel through
the openpyxl/Sora reference package and private Rust admission guard.

The generated [Persona row audit](../content-manifests/divergent-universe-runtime-v1/persona-reachability.json)
accounts for all 547 rows in the eleven inherited Persona tables. Current area
layer joins reach 60 positions, six fixed presets, six composition types and
six exact type/level composition definitions: 78 records in total. The other
469 records remain `PendingSelectorProof`, not excluded and not proven current
runtime candidates. The audit exports only source keys, locators, hashes and
reviewed parent relationships; it does not export source descriptions.

In particular, area field `ILPNADCAIBL` contains 501, 601, 701, 801 and 901,
whereas the style table lacks the first four keys. The single overlapping 901
does not prove a style selector. No style, gift, talent, attribute or constant
is admitted through that shortcut, a matching prefix, or a presentation label.
The six reached composition definitions are further source joins, not an
implementation of their room behavior.

The membership gate passes against the corrected current manifest:

```text
.cache/g22-python/Scripts/python.exe tools/divergent-universe-runtime/persona_reachability.py --check --require-reconciled
```

Removing the prefix alone cannot pass: every row must receive a reviewed
manifest entry or explicit row exclusion, and current-reachable rows cannot be
excluded. Duplicate row accounting is also rejected. This gate establishes
reference accounting only; existing runtime execution gates remain independently
required. The runtime repository-audit verifier also requires normalized
promotion before considering its prior all-pass artifact:

```text
.cache/g22-python/Scripts/python.exe tools/divergent-universe-runtime/persona_reachability.py --check --require-promoted
```

This gate passes: each source row has exactly one normalized coverage record.
The production reference package contains 81 tables and 28,732 rows, including
all 6,762 coverage records. The new table preserves source keys, locators,
row hashes, selector-proof states and parent references; it does not lower
room effects. Its 78 reached records remain `Researched`, the other 469 remain
`Cataloged`, and all 547 carry `Unimplemented`. Rust rejects silent ownership,
coverage or runtime-status promotion and missing rows. DataReady coverage is
6,215/6,762 (91.91%), so reference readiness remains incomplete. The runtime
ledger accounts for all 6,762 obligations exactly once. Persona rows remain
`PendingSourceReview` with `SourceOnly` catalog status, no runtime admission and
unassigned trigger/lifetime/snapshot semantics. The assigned batch and fixture
identify a source-review owner only; executable semantic-family allocation must
follow selector and behavior review. Neither 78 nor 547 is added to terminal
runtime totals by this audit.

Classification-only inventory maintenance uses `inventory.mjs --reclassify`
and `verify-inventory.mjs --reclassify`. This preserves all 2,684 admitted
path/hash/size tuples and verifies pinned revisions, clean caches and selected
Git paths. It does not rehash every source blob. Full inventory reconstruction
currently fails fetching missing blob `36c6f5a6224127ebf8e3096bdc63a3506b53d249`
for `Config/ConfigAbility/BattleEvent/Avatar_RogueBattleevent_Sandworm_S1_Ability.layout.json`
from the promisor remote (`SSL_ERROR_SYSCALL`); that check has not passed. The
twelve source files used by this Persona audit are independently byte-verified
against their admitted SHA-256 hashes, including the area selector table.

Source audit generation and authoring regression checks are reproducible with:

```text
.cache/g22-python/Scripts/python.exe tools/divergent-universe-runtime/persona_reachability.py
.cache/g22-python/Scripts/python.exe tools/divergent-universe-runtime/test_persona_reachability.py
```

The source verifier reproduces every exact-once join without writing raw data:

```text
.cache/g22-python/Scripts/python.exe tools/divergent-universe-runtime/domain_layout_rows.py --verify-source .cache/content-reference/turnbasedgamedata
```

Normal authoring remains self-contained and does not load raw sources at
runtime. The workbook drift check compares a fresh authored workbook/export
against the production bundle. No runtime obligation or mechanic program is
terminalized merely by importing these records.
