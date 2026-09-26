# Divergent Universe logical-layer battles

The `BattleRoutes` workbook table now binds the production Ordinary/Cyclical
baseline to `VersionedProjectPolicyOneBattlePerLayerWithDomainChoices`. Each exact
ordered area layer contains a real proxy battle followed by the provisional
Common/Rare Blessing reward boundary. The first battle is Combat; later layers
offer a public Combat/Elite/Aberration choice before encounter selection. The
default three-layer entries execute three battles and ten accepted public
actions when no reward is suppressed or Treasure event added. Lower-level
single-battle fixtures remain available through their explicit entry selection.

Battle preparation resolves the current graph's unique outgoing Battle node and
its actual section instead of assuming the first layer. Blessing acceptance
recognizes graph-bound victory reward nodes. Each additional battle/reward pair
retains its owning logical layer; changing layer changes the corresponding
logical scopes. The shared participant ledger supplies HP, energy, life and
presence to the next handoff, with the declared clamping policies. There is no
implicit full heal between battles. Accepted results, carry, reward RNG and graph
advance share the existing transaction. A previous battle's result cannot settle
a later pending battle.

Encounter-pool observation also resolves this graph-bound target rather than
interpreting the encounter's physical node ID as a layer ordinal. Encounter and
Battle must share an exact Run/Plane/Room path, with the Plane key matching both
physical sections and the selected area's layer range. The Battle may add a
nested Battle scope. Missing paths, room/plane mismatches, multiple outgoing
Battle edges and foreign state definitions reject before any RNG or assembly
cache access. This removes the initial Tawot encounter's address special case;
its first-plane selection is governed by the same graph/scope validation.

Regression tests readdress first, later and Tawot encounters and their Battle
targets, then execute actual nested battles and fresh command reconstruction in
both run families, with and without nested Battle scopes. The default authored
programs, pool membership and RNG policy are unchanged. These are handoff tests
over the provisional layer route, not original room admission, encoded Persona
profile replay or complete-run release evidence. Production position payload
binding and source-specific encounter/boss selection remain incomplete.

Decision source 18 binds `ExcelOutput/RogueTournArea.json` at released 4.4 revision
`fd978d6ef09f941fba644c731ab54abd6f7c3568` of
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
reviewed 2026-09-09. The file SHA-256 is
`756510177a468130464c27ef382d9ab33a7098dc993022cd8647366cc7c46db8`.
This proves ordered area/layer joins, not room count or battle placement.

The separate `EncounterPools` table binds
`VersionedProjectPolicyFixedFirstUniformLaterLayersWithReplacement`: the first
layer uses authored stage `83002081`, and each later layer independently samples
one of `83002081`, `83002111`, `83002161`, `83002181`, `83002191` from group
`divergent-universe.encounter-group.300202`. Selection uses unit integer weights,
stable stage order and shared `Encounter` RNG purpose `23901`, with replacement
across layers. Only the selected encounter is publicly offered. Automatic
initialization nodes precede later domain choices and retain the same logical
layer. Treasure dialogues, when eligible, precede the domain choice. The domain
Route acceptance and new encounter draw share one graph transaction. Earlier
reward acceptance or suppressed-reward settlement stops at the domain choice
without drawing an encounter.

Assembly validates the offered group/stage before cache lookup; battle start
validates the immutable materialization binding again. The baseline records the
actual selected stage for fresh replay, not its fallback controller constant.
Reading an offer and rejecting stale/hidden choices consume no randomness.

Decision sources 19 and 20 bind `ExcelOutput/RogueMonsterGroup.json`, row
`RogueMonsterGroupID=300202`, and `ExcelOutput/RogueMonster.json`, rows
`RogueMonsterID=3002081,3002111,3002161,3002181,3002191`, respectively, at the
same released revision and access date as above. Their SHA-256 values are
`8abe52c1f9b5fd42eeefaee63ff8bc59c8dd8acc0f18502b5548b353416f2171` and
`52f235fecb6aa99d3fe01d4f1ff8c42ac3b7c8fbd53fb77a7bbdae140e056f32`.
These establish group/monster/stage joins, not normal-room eligibility or an
enabled weekly boss selector.

Decision source 24 additionally records `ExcelOutput/StageConfig.json` at that
revision (SHA-256
`89e7dcd4824b81b080f985930142ddd903db4f31fc41667e898c0022a85ced7f`).
All five selected stages explicitly carry `_IsEliteBattle=1` and
`StageAbility_RogueEliteLevel_Enhance`. Thus source elite classification is known;
the baseline's independently selected proxy domain is a deliberate non-parity policy,
not a claim that the original stages are normal or their classification absent.
Those source markers do not establish normal-room fragment drops.

The one-battle count, later domain choices, checkpoint placement, first-stage
choice and cross-layer candidate/proxy assignment are low-confidence project policy.
Alternatives include multiple rooms and mixed combat/event/service branches.
Released current-profile room graphs or reproducible observations replace these
fields independently. Original room branches, room-specific encounter pools, elite/boss
programs, automatic retry and complete content reachability remain unimplemented;
this policy does not turn the limited baseline into complete gameplay parity.

## Authenticated public domains

The `DomainChoices` sheet declares exactly three stable choices under
`VersionedProjectPolicyExplicitLaterLayerChoices`. Sources 39/40 bind the same
released revision, inspected 2026-09-12: `RoguePersonaRoomCompType.json` rows
3/Battle, 2/Elite and 4/Encounter identify Combat, Elite and Aberration; its
SHA-256 is `c0223603c6e5252278dac5224d766f1c4ed60ebe5845b9839b9e3533a8ad76a5`.
All 28 current Tourn3 areas declare initial Battle, but their ordered layers
have no matching `RogueTournLayerRoom` rows. These facts do not prove later
selectable room membership. No retained Tourn2 room ID is promoted.
The current [Persona layout](divergent-universe-domain-layout.md) instead joins
all eleven current layers through `RoguePersonaLayerRoom`: 60 positions include
21 fixed presets and 39 unspecified positions. The old-table absence is not an
absence of current position records. Those records are now typed configuration;
the provisional baseline below does not yet execute that layout or its selector.
The inspected `ExcelOutput/RogueTournLayerRoom.json` has 103 rows at that
revision; its SHA-256 is
`4adec30dc8f87a4a80e911ed80a4732ce00d7e9b14811457cb4cb9b4b238a006`.
The absence check joins each selected area's `GLNDIILFKBN` values to `LayerID`.

Selection uses the shared generated-choice transaction with ordinary Route
operations. Its authenticated prefix consumes reviewed
[Curio domain allowances](divergent-universe-curio-domain-expiry.md) and sets the
domain marker required by the typed option, before encounter generation. This
prevents direct generic option execution from skipping the entry effects.
A section-scoped label persists
through domain choice, encounter, battle and reward, and resets at the next
layer. This is truthful for the current one-domain-per-layer policy; future
multi-room planes must replace that lifetime explicitly. Starclock keys are
separate from upstream composition IDs. Hidden, stale and duplicate choices
cannot change the label or sample again; no free-form adapter override exists.
Agent/MCP observations display the authored domain names while preserving the
same shared opaque tokens and option bindings.
The selected domain gates the [Sage victory grant](divergent-universe-sage-victory.md).
Every current proxy still uses the same provisional Common/Rare base pool;
original domain-specific base rewards and enemy programs remain pending.
Base fragments now use a separate [fixed-domain credit policy](divergent-universe-battle-fragments.md),
not the source stages' elite markers; the original amounts remain unverified.

Public tests acquire and upgrade Sage, select Elite then Aberration, settle
three real battles and verify fresh selection replay for both families. They
check 0/2/3 immediate rewards before each ordinary offer and domain reset before
each later choice. Controlled tests separately cover all forms and Boss exclusion.

Tests execute each physical battle, verify carried resource values at subsequent
handoffs, reject an earlier result against a later battle, probe a second-layer
loss settlement without reward RNG, and complete fresh
public replay through both families. A fixed seeded corpus exercises all five
later-layer candidates, repeated stages, RNG isolation and rejection of another
candidate before assembly/cache mutation. The workbook/Sora validator reproduces the
current authored bundle. These checks do not terminalize the outstanding
source-program audit in the [complete runtime goal](goals/22-divergent-universe-runtime.md).
