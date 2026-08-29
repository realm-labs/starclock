# Currency Wars runtime

Starclock's Currency Wars implementation is a mode profile over
`starclock-activity`; it is not a second Activity or battle state machine.
`starclock-mode-currency-wars` owns mode terminology, catalog validation,
economy/roster formulas and graph compilation. `starclock-data` privately reads
the generated Currency Wars Sora bundle and returns one immutable mode catalog
candidate. Before returning, it decodes all 111 generated tables, validates the
78,607-row inventory and binds the generated schema fingerprint, schema-lock
SHA-256, exact bundle SHA-256 and a stable table-inventory content digest.
Generated row types and table names remain private to `starclock-data`.
Individual fights remain opaque `BattleSpec`/`BattleResult` handoffs owned by
`starclock-combat`.

This document describes the complete current Version 4.4 runtime boundary, not
a claim of exact gameplay parity where explicit project policies remain. The
execution plan is
[`Goal 21 — Complete Currency Wars Runtime`](goals/21-currency-wars-runtime.md).

## Production data

The production authoring surface is the three workbooks under
`config/currency-wars/data/`. The current Sora bundle lowers:

- one profile, four modules, two entries, both Gambits and 135 typed finish
  conditions, including all 114 released battle-penalty rules;
- one area group, 26 routes, 75 Plane layers, five room/domain kinds and 493
  ordered nodes with an exact stage-flow entry per node;
- 97 difficulty records, ten rank boundaries, 23 Plane/section battle bases and
  23 Stage battle bases;
- 77 roster roles, ten offer/team levels and five rarity price rules;
- 49 bond identities: 33 main bonds and 16 source-authored subtrait bonds;
- 834 investment identities across Augments, enhancements, Orbs, Portal buffs,
  Projections and Talents;
- 12 explicit research-gap policies.

Node templates, Stage IDs, node kinds, layer/domain/room references, parameters,
penalty/bonus rules, Gold rewards and next-node references are direct generated
fields. Flow lowering validates profile/module/entry/Gambit/finish references,
the complete area/layer/node/domain closure and all rank-progression variants.
The generated flow table omits its source-side next-flow label, so lowering
derives that label only from the validated generated next-node chain. Its empty
carry/reset arrays remain explicit empty typed rule sets. Runtime execution uses
a separately declared VersionedProjectPolicy: authoritative run slots and
participant battle state carry across nodes and Plane boundaries, node-scoped
shop offers reset at `NodeStart`, and every new run starts from a fresh initial
snapshot. Role rarity and position plus next-level Experience are also direct
generated fields. Runtime lowering does not parse localized summaries or load
JSON/Excel.

Bond execution derives one immutable post-mutation snapshot after each accepted
roster, deployment, equipment or subtrait-selection boundary. Main levels count
distinct deployed authored members. Selected subtraits inherit their parent's
member count, with explicit deployed-role/equipment/granted-trait selectors and
automatic default/exact-module selectors kept as separate typed rules. The
snapshot contains active levels, selected parent-child pairs, exact layer
contribution identities, trait/battle-event IDs and 187 direct properties over
16 fixed-point property kinds with resolved role targets. Rejected selection
does not change Activity state; an inactive parent or selector removes the
child contribution atomically. Dynamic additional-member and granted-trait
producers (including Bond 1014) join the final immutable contribution snapshot.
The snapshot retains all 683 Bond contribution definitions, while MazeBuff and
battle-event program execution remains a P6 responsibility.

The generated P2 capability inventory reopens the exact pinned source records
only as an authoring/audit input. It verifies all 2,367 source-record digests,
then records structural configuration types, expressions, selectors, triggers,
state and lifecycle hooks in
`content-manifests/currency-wars-runtime-v1/capability-inventory.json`. Each
shape has a terminal support mapping: an existing generic primitive, a named
missing capability for P2-B2 through P2-B4, or a non-authoritative presentation
boundary. `ExistingPrimitive` is not execution credit. The Version 4.4 postfix
bytes are retained exactly and remain unresolved until their semantics can be
proved or replaced by an explicitly marked policy; the runtime does not guess
their meaning.

The shared Activity vocabulary now covers all Activity-side shapes identified
by that inventory. It includes typed equality/ordering comparisons, bounded
counts for counter maps, ordered ID sets, inventories and modifiers, and atomic
replacement of one counter entry, ordered-set member, inventory count or
modifier stack count. These are generic transaction primitives: they validate
against immutable state definitions, preserve canonical collection order,
emit ordinary Activity events and roll back the whole accepted command on a
fault. They do not interpret Currency Wars IDs or execute source programs by
themselves.

The shared Combat vocabulary now also covers every Combat-side shape in the
inventory without introducing a Currency Wars opcode interpreter. A rule
selector may consume the stable first-occurrence union of previously resolved
selectors before applying its normal predicates, ordering, random choice and
cardinality policy. An active effect may impose a persistent maximum-HP ratio
floor; multiple floors compose by maximum, are rounded upward at the HP
boundary, and are removed with the owning effect. Rule IR can also change a
team's Skill Point cap with checked add/subtract/set semantics, clamping the
current value and emitting one typed resource event per affected team. Existing
Rule IR condition, replacement, operation, trigger and informational-event
primitives cover the other authoritative shapes, while battle-result metric
projection carries battle-produced drops and counters back to Activity. This
capability closure still grants no source-program execution credit; those
programs receive execution credit only through their terminal generated P6
partitions, which are now fully executed and audited by exact source row.

The shared Build compiler now accepts catalog-owned `BuildContributionDefinition`
values for equipment, progression and mode contributions. A contribution has a
stable ID, exact source attribution, typed `BuildPatch` operations and explicit
Any/Form/Path applicability. `CombatantBuildSpec` selects only contribution IDs;
the compiler resolves them in canonical ID order, rejects unknown, inapplicable
or conflicting contributions, and binds both the selected IDs and source rows
into build/catalog/combatant identity. Owned and trial character shapes continue
to use ordinary character definitions and named presets, while battle-time
selectors, dynamic state and lifecycle hooks remain in Combat Rule IR. This
closes the Build-side capability inventory without moving account lookup or
Currency Wars IDs into the generic compiler, and grants no P4 source-execution
credit by itself.

P4-B1 additionally resolves all 77 released role-to-trial-build joins through
`GridFightRoleBasicInfo.SpecialAvatarID`. The selected world-level 6
`SpecialAvatar` row provides exact level, promotion, Eidolon, shared character,
shared Light Cone and relic selectors. Relic main/sub aggregates and static set
properties compile into generic HP/ATK/DEF/SPD, effect, CRIT, Break, Energy,
healing and elemental-damage values; their canonical values participate in
build, combatant, battle-input and state identity. Dynamic relic-set abilities
and parameters are retained as typed Currency Wars data for their later P6
battle-program partitions. Account lookup remains caller-owned: Activity and
combat receive only an immutable owned snapshot or the immutable trial minimum,
and neither mutates account state.

The P2 closure executes four shared boundary probes covering Activity
collection replacement and rollback, Combat selector composition/Skill Point
caps/persistent HP floors, and Build contribution selection. It audits every
Rust file under the shared Activity, Build, Combat and Rules roots, including
untracked working-tree files, and finds no Currency Wars branch in shared core.
The production battle/activity native-handler admission count remains zero.
All 43 generated partitions covering 2,367 programs carry deterministic freeze
digests and terminal exact-once receipts. The final disposition is 249 exact
Activity programs, 17 exact battle Rule-IR programs, 263 executable policy
Rule-IR programs and 1,838 proven metadata-only programs.

`G21-P5-A01` closes 64 exact `TutorialTask` source programs as metadata-only.
The source authoring generator recursively rejects any configuration type or
operation outside the reviewed tutorial-presentation vocabulary, then records
per-type counts and an ordered-shape digest. Production Sora lowering exposes
those audits as typed `CurrencyWarsMechanicProgramDisposition::MetadataOnly`
values. It does not translate UI waits, hints, navigation, drag/click guidance,
pause, auto-battle prohibition or input locks into Activity operations. The
complete 76-file tutorial family contributes 683 configuration-type and 168
tutorial-operation occurrences, all with zero authoritative mutations; 64
belong to P5-A01 and twelve are terminal members of P5-A02.

`G21-P5-A02` additionally audits
`InitLevelGraph_Prop_Common_GridFightConsole_01.json`. Its closed 18-type,
46-node graph listens for world-prop events, changes prop presentation, enables
an interaction button, plays sound and opens the entrance UI. It never writes
run currency, inventory, roster, route, node or battle state. Production
lowering therefore exposes the enum-backed `WorldPropAndUiEntry` metadata
audit, preserving its exact source and ordered-shape digests without installing
a duplicate Activity entry state machine. All 13 P5-A02 programs are terminal;
P5-A03 and P5-A04 are the next two terminal generated partitions.

`G21-P5-A03` and `G21-P5-A04` execute one responsibility split only by the
64-program partition cap. Five exact `GridFightExpertRestrict` rows define
inclusive role-cost availability thresholds for Standard and Overclock, and
all 80 exact `GridFightSeasonExpScore` rows define optional base weekly score
and experience at a division, score-rule, chapter and section key. Shop
generation excludes a role when its rarity/cost tier is not yet available.
The current chapter and section are committed to bounded Activity slots before
each route-node offer, preserving the last entered position through completion
or failure. `CurrencyWarsRun::progression_projection` combines that position,
the selected Gambit and difficulty with the exact authored row. Difficulty
modifiers are authored percentage values (`100` means 100%), so the formula
applies an explicit `/100` fixed-point boundary. It intentionally returns
exact `Scalar` products and does not guess the game's final integer reward
rounding. All 85 programs are terminal.

`G21-P5-A05` through `G21-P5-A07` audit all 459 decoder-layout descriptors
and classify 95 character-override configuration programs against the exact
Version 4.4 role-star, servant-star and summon-event joins. Fifty-one reachable
programs lower to typed immutable contribution data; 44 unbound character
overrides remain explicit metadata-only audits rather than executable guesses.
The selected role and servant override is included in each role contribution,
and the season-selected summon battle-event overrides are included in the
contribution digest. This covers distinct forms such as both Bronya role IDs,
Aglaea and her servant, and Lingsha's front summon battle event.

`G21-P5-A08` and `G21-P5-A09` add the bound Silver Wolf 999 override and execute
two module role exclusions, one 77-role season pool, 32 season/trait role pools
and all 77 in-game reference scores.
The season/module policy filters initial rosters, shop candidates, random role
rewards and forge candidates. Controller ranking is deterministic: descending
authored reference score, then ascending stable role ID. Six localized role
remarks and four role-tag descriptions are preserved as exact metadata audits;
their text hashes do not mutate Activity state.

`G21-P5-A10` and `G21-P5-A11` close the remaining Activity partitions. The 24
NPC rows, four world entity/prop configurations and eight animation audio/effect
programs contain presentation data only and lower to typed metadata audits with
source, shape and operation-count receipts. During this review,
`GlobalTaskListTemplate_GridFight` was reclassified from Activity to battle:
its released program contains wave/alive predicates, combat target selection
and 11 modifier applications. The corrected frozen denominator is therefore
520 Activity programs and 1,847 battle-visible/boundary programs, with the
2,367 total unchanged. All 249 executable Activity programs are terminal;
`G21-P6-B1` follows this boundary.

`G21-P6-B1` executes all 939 encounter-assigned source rows. The immutable
encounter catalog now distinguishes GridFightMonster IDs, shared enemy stable
keys and StageConfig placeholder IDs instead of crossing their namespaces.
StageConfig supplies level, ordered waves and formation slots; Camp/BossPool
supplies the actual mode enemy candidates; each selected monster supplies its
Star1-4 EliteGroup scaling, and EnemyDifficulty supplies chapter/difficulty
scaling. The battle resource catalog preloads the Cartesian closure of 160
mode enemies and 15 released Stage levels (2,400 inputs). The selected roster,
wave shape, star scaling and difficulty scaling all participate in assembly
cache identity. The exact reachability audit retains eight monsters outside
current Camp pools and 138 EliteGroup definitions outside the current monster
reference closure as validated but unreachable definitions. Selection details
that released data does not prove remain explicit replaceable project policies.
`G21-P6-B2` terminally executes 414 assigned rows: 51 Affix definitions, 67
Affix MazeBuff bindings, 603 normalized difficulty rows and 296 stage/rank
progression inputs. All 51 Affix identities compile to prebattle stat changes,
Activity-boundary Action Value changes or generic battle rules and modifiers.
The battle resolver contains no Affix content-ID branches. Enervation explicitly
marks its under-equipped owner modifier for inheritance by subsequently created
memosprites, while Time Assassin uses a labeled deterministic one-in-four
project-policy draw until released executable evidence identifies its exact
spawn algorithm.

`G21-P6-B3` constructs immutable production `BattleSpec` values from the
current contribution snapshot and a bounded cache. Its exact receipt covers
1,340 integrated and 88 excluded assembly rows. `G21-P6-B4` executes 1,122
node, Stage, route and bonus rows through atomic battle-result projection,
Squad HP/action-value settlement, rewards and next-node transition.
`G21-P6-B5` verifies stale/rejected rollback, cache behavior and component-
addressed fresh reconstruction of transition battles.

Battle assembly keeps personal and team resources distinct. Released front
special `EnergyBar` and `MaxSP` properties project to the selected role's
current and maximum Energy; they do not alter the team's Skill Point pool,
which starts at the declared `3/5` project-policy boundary. If a selected
ability references the C04 `assist-use` key, assembly declares that keyed team
resource as `0/4` with cross-wave persistence before battle creation. Unknown
selected team-resource keys fail assembly instead of becoming implicit state.

The 32 generated battle-program partitions are also terminal. `M01` through
`M09` lower 263 reviewed high-level programs to executable policy Rule IR and
audit 285 presentation-only programs. `M10` through `M13` lower 17 exact enemy
configuration, global/complex AI and global-task-template programs while
auditing 147 metadata-only neighbors. The global-task-template library executes
six exact modifier-selection templates and rejects seven presentation-only
templates at the authoritative boundary. `M14` through `M32` then close 1,135
metadata-only records in order: 93 localized skill-description modifications,
997 skill-sub-icon routes and 45 battle-effect preload files. Each frozen
partition receipt binds its exact source set and recursive field audit and
records zero authoritative operations.

`G21-P7-B1` adds the deterministic baseline controller in `starclock-ai`. It
selects only currently offered Activity decisions and battle legal commands,
uses the exact catalog enemy AI graph, and retains command, event and state-hash
traces for later replay verification. Production tests complete the released
seven-battle route in both Standard and Overclock; a same-seed Standard pair
produces identical complete reports. Those runs also verify generic keyed
Toughness-layer creation/removal and selector resolution across nested program
calls.

`G21-P7-B2` adds the production CLI boundary. `currency-wars coverage` reports
all 19,250 source obligations terminal and zero pending without
inflating them from catalog presence; `currency-wars run` accepts route,
difficulty, Standard/Overclock Gambit and seed, executes the baseline controller
and can export a canonical replay. The replay records accepted Activity and
battle commands, nested-battle boundaries, expected battle states and events,
and expected Activity states. `replay verify` reconstructs the run from fresh
immutable Sora-backed inputs and requires exact replay-byte equality. P7-B5
binds all nine frozen components and reports first divergence across catalog,
Activity, battle assembly, battle commands and settlement. Existing
configuration validation and route inspection remain unchanged.

`G21-P7-B3` adds a protocol-neutral Currency Wars Agent API facade. Its bounded
manifest contains only configuration/content digests, 26 route summaries, 97
difficulty summaries, the two Gambits and the four released baseline fixture
role IDs; it never serializes generated rows, mechanic programs or private
catalog values. Sessions share the existing ownership, lease and quota
registry. Observations are copied exclusively from `ActivityPlayerView`, and
legal choices are state- and boundary-bound opaque tokens. An encounter action
enters the existing preparation boundary; the following preparation action
executes exactly one real nested battle through the deterministic baseline
controller and submits its result through the Currency Wars Activity runtime.
Shop and route actions use the existing runtime commands. Agent-side
component-addressed replay reconstruction uses the shared P7-B5 boundary
rather than leaking the CLI adapter's private encoding.

`G21-P7-B4` exposes that exact registry session through the existing MCP
Activity tools with `mode=currency-wars`; MCP owns no duplicate Activity or
battle state. Two bounded inert resources publish the Currency Wars manifest
and concise rules. Existing Activity create/read/act/close OAuth scopes apply,
and tenant/principal ownership is checked before session data is disclosed.
Exact idempotency keys make a retry after an MCP cancellation notification
return the original response without a second commit. Closing the Activity is
the explicit session-cancellation boundary and releases its quota. Activity
observations accept an event cursor and return accepted-action summaries in
pages of at most 256 from an 8,192-entry retained window; duplicate retries add
no event, while future and expired cursors fail closed. End-to-end MCP coverage
crosses the separate encounter and preparation calls and settles one real
nested Currency Wars battle.

`G21-P7-B6` executes the bounded 97-entry legal matrix against production
catalogs, offered Activity/battle commands, real nested battles and fresh
replay. The matrix controller cannot concede, treats every battle fault as a
test failure and records ordinary `Failed` runs as lawful terminal gameplay
rather than synthetic success. `G21-P8-B1` hardens malformed, stale, RNG,
empty-pool, overflow,
recursion and replay-corruption boundaries. `G21-P8-B2` freezes eight measured
release workloads. `G21-P8-B3` passes dependency/license, architecture,
generated drift, Sora 0.6.1, 111-sheet workbook visual review, provenance,
native-handler and prior-release isolation audits. `G21-P8-B4` closes all
19,250 sources, 2,367 programs, 28 semantic fixture families and 12 policies;
`G21-P8-B5` passes clean-checkout acceptance and the hosted Windows x64, Linux
x64 and macOS ARM64 release matrix, including all paired compile-only targets.

The remaining configuration-program uncertainty is an executable
`VersionedProjectPolicy`, not a guessed opcode implementation. The pinned data
proves ten opcode bytes, 163 postfix sequences and 153 unresolved expression
shapes affecting 193 programs. Independent public evidence verifies postfix
ordering and byte-0/byte-1 operand references only. Production therefore lowers
reviewed high-level nodes and named dynamic values directly to typed IR; it
never interprets raw `PostfixBase64`, and a partition with an unproved
expression cannot complete. The policy is replaced when released evidence
proves all ten opcode semantics.

## Run model

`CurrencyWarsRunDefinition` binds one entry state, route, difficulty, Gambit,
participant lock and caller-supplied initial roster/deployment. Entry validation
requires player level 21, applies the selected Gambit's unlock, validates the
difficulty against the highest completed Standard rank and checks profile,
season and route membership. Released data does not directly bind routes to a
Gambit, so both Gambits currently use the complete route set under an explicit
VersionedProjectPolicy. Every route node compiles to the shared Activity graph:

- Monster, Camp Monster, Elite Branch and Boss nodes become `Battle` nodes;
- Supply nodes become `Shop` decisions;
- the end of each non-final Plane becomes an explicit route decision that
  traverses into the next Plane;
- a timed-out battle enters an automatic `Checkpoint`, subtracts the
  mode-computed Squad HP loss, then continues when Squad HP remains positive or
  fails at zero;
- a fault enters the ordinary fault terminal; a completed final node enters the
  completed terminal.

The run stores Gold, Experience, team level, Squad HP, last battle metrics,
roster star/count states, deployment positions, active bond levels, current
shop offers and selected investment identities in typed Activity slots. Shop
refresh, purchase/sale, synthesis, deployment, bond recomputation, level-up and
investment selection use typed atomic boundary operations. A rejected command
therefore restores state and RNG together. Settlement maps score through the
authored inclusive B/A/S/SS/SSS intervals and uses the unique unbounded finish
condition when no ranked interval matches.

The frozen seed `21000501` now executes one complete production Standard
Activity route: route 100 at difficulty 10101 crosses 23 authored nodes, 20
validated battle handoffs/results, three Supply decisions, one recoverable
non-victory checkpoint, both Plane transitions and terminal SSS settlement.
The test also performs a paid refresh, purchase and deployment/Bond
reconciliation. This P3-B6 slice deliberately uses caller-supplied boundary
`BattleSpec` values; it does not claim production build/enemy assembly, Combat
command execution, Projection 1508 contribution or replay reconstruction.
Those requirements remain in their assigned P4-P7 batches.

Each active route node automatically refreshes five shop slots unless the shop
is locked. A lock carries the exact remaining slot/role pairs across node entry;
a manual refresh costs the authored two Gold and replaces the locked snapshot.
Candidates are sorted by stable role ID, rarity is sampled with the authored
level weights, and roles are sampled from the finite per-rarity pool using the
authored initial role weight. Duplicate roles are legal. A paid refresh that
cannot fill all five slots rejects atomically and restores Gold and RNG; an
automatic refresh exposes the remaining legal cards or an empty shop. Battle
income applies the authored base Gold and pre-reward interest (`1` per `10`
Gold, capped at `5` in Standard and `0` in Overclock) plus the selected
Gambit's Experience.

Three equal-star copies synthesize in ascending star order. All 266 role-owned
star states and 189 transitions execute for the 77 production roles: 42 roles
end at star 3 and 35 end at star 4. Capacity is checked after synthesis, so a
purchase into a full bench is accepted only when the new copy immediately
combines into a legal post-purchase roster. Synthesis and sale reconcile old
deployment states before recomputing Bonds in the same Activity boundary.
Once a role reaches its authored maximum star, its remaining current offers
are removed and ordinary refresh no longer offers it. The unreleased behavior
for copies granted by non-shop content remains an explicit
`VersionedProjectPolicy`: preserve the lower-star overflow for an explicit sale
without inventing Gold, another reward or a higher star. Public release guides
cross-check the full-bench merge exception and equal-star combination
([Prydwen](https://www.prydwen.gg/star-rail/guides/currency-wars),
[TapTap](https://www.taptap.cn/moment/731573167151121380), accessed
2026-08-13).

Team levels 1 through 10 permit exactly 1 through 10 deployed roles. The Front
area has an authored minimum of one and maximum of four; the Back area starts
with six slots and may expand to the authored maximum of nine. The waiting area
holds nine ordinary roster units. Deployment mutations validate the current
team-level, Front, Back and waiting-area limits, while battle entry additionally
requires the Front minimum. A role may be placed outside its authored position;
the runtime records its Character Empowerment eligibility as false there. P4-B3
now consumes that eligibility by deriving an immutable deployment snapshot:
matching positions activate the role's exact display and role-star skill
families, while relocation, off-position placement and undeployment refresh or
tear down that snapshot without duplicate Activity state. Battle assembly
selects each stable shared Ability and its final effective level before crossing
the immutable contribution boundary; battle operation execution remains
assigned to P6. This matches
the released public guide's off-position
rule ([Prydwen](https://d2ankz0m1a0dsp.cloudfront.net/star-rail/guides/currency-wars/),
accessed 2026-08-13).

When synthesis consumes deployed copies, roster and deployment are reconciled
inside the same purchase boundary. Released data does not identify which
deployed instance retains the upgraded state, so a `VersionedProjectPolicy`
keeps occupied positions in stable order, places the highest resulting state
in the earliest retained position and removes other consumed positions. The
separate source constant `GridFight_Bench_OverFlow_AvatarNum = 100` applies
only at typed non-shop service role-grant boundaries. Ordinary shop purchases
and deployment mutations retain the nine-unit waiting-area cap. Released data
does not identify the constant's exact call sites, so this scope is a
`VersionedProjectPolicy`: replace it when released executable evidence proves a
narrower or different owner. A service grant above 100 waiting units is
rejected atomically rather than dropping the reward or expanding the roster
without a bound.

Battle settlement carries exact participant HP/Energy/life/presence and expects
the typed metrics `currency_wars_battle_progress` (`Ratio`) and
`currency_wars_action_value_remaining` (`ActionValue`). The caller cannot
submit an already-computed Squad HP loss. Each battle node compiles its exact
penalty-rule reference into one boundary: 89 are finite clocks and 25 are
unlimited. A finite budget is `TotalTurn * 10` action value and expires as
`Lost`; a lost result always projects zero remaining action value. Lethal
rescue exists only for finite Action Value battles and deducts the released
`ratio * 100` action value from the current budget with a floor at zero;
unlimited battles use ordinary defeat. Because released text does not state
the restored HP amount, the executable policy restores maximum HP and records
the replacement condition. Victory adds the node's authored Gold when present
and the selected Gambit's authored Experience. Energy Disappearance reduces
Energy by the lesser of the authored four points and the attacker's current
Energy, preserving strict generic resource-spend semantics without underflow.

The remaining Squad HP composition uncertainty is an executable
`VersionedProjectPolicy`. Victory wins a simultaneous timeout boundary.
Otherwise loss is the configured base plus the ceiling of uncleared progress
times its coefficient, plus the threshold-failure extra when below the
configured threshold; Squad HP then clamps at zero before the automatic
checkpoint continues or terminates the run. Rejected malformed results leave
the Activity hash unchanged. The caller remains responsible for assembling the
selected difficulty, deployed resolved builds, node mechanic contributions and
enemy scaling into `BattleSpec`.

Immediately before that handoff, the mode materializes one immutable
`CurrencyWarsContributionSnapshot`. It owns the exact route, difficulty,
Gambit, node, team-level properties and Squad HP; every deployed role's role and
star definitions, matching servant-star definitions, selected owned/trial
build receipt, effective shared Ability levels, equipment and off-field
conversions; the active Bond snapshot and complete contribution registry;
selected investment definitions; all seven rarity/star influence rules; the
254-entry exact contribution-parameter registry; and the resolved battle
override snapshot. Its identity binds Activity definition/config/state hashes
and dynamic selections. Static definitions are bound by the exact configuration
digest retained in that identity, so battle materialization does not query the
Activity catalog after this boundary. The 230 `GridFightCombinationBonus`
parameter pairs remain an exact read-only registry: no released external key to
team/star rows was found, so consumers may resolve them only from an explicit
authored Bonus ID and never by numeric adjacency.

## Explicit policy boundary

The bundle retains replaceable `ProjectPolicy` boundaries, not observed parity.
The current runtime executes and tests complete-route Gambit membership,
carry/reset behavior, same-boundary Squad HP/action-value ordering, Gold Coin
identity, shop sampling order and maximum-star overflow. The structural
configuration-program policy is also executable across all assigned mechanic
partitions without claiming raw-postfix parity. The remaining policy boundaries
are:

- exact Camp boss identity;
- investment operation order;
- role-to-shared-build selection.

Augments now lower to typed quality, category, chapter, season, module-ban,
lifecycle, remark and exact decimal fields. An explicit mode-program boundary
generates a deterministic three-card offer from stable eligible candidates;
selection and caller-explicit replacement are atomic Activity transactions.
Selected Enhancements are offered only for an active authored Bond trait
effect, with `MaxStar` derived from the current authoritative roster and Gold
cost taken from configuration. Both selected families are retained in the
immutable battle contribution snapshot and its digest. Released evidence does
not establish the automatic node-entry schedule, so the executable policy
keeps offer timing explicit and does not infer replacement from category or ID
adjacency. Portal buffs, Orbs, Projections and permanent/season Talents now
lower into typed immutable catalogs and participate in the owned investment
set or season-Talent set. Portal selection validates season membership, Gambit
availability and module bans; Projection selection validates its required
owned role; Talent selection validates the authored prerequisite graph.
Associated maze buffs and display/remark data are retained as typed values,
but a maze buff is joined to a parent only when the source contains an explicit
effect reference. Talent rows expose an exact numeric cost without a currency
key, so the Activity boundary accepts explicit payment confirmation and does
not infer Gold. The Activity lifecycle now covers all 834 investment
identities with canonical ordering, typed replacement and contribution
refresh; Phase 6 installs their immutable battle contributions and reviewed
battle-visible programs.
The source-proven empty Blessing/Formula families remain empty, while seven
exact maze-buff Enhancement rows cross the immutable contribution boundary.

Occurrence execution validates all 167 Occurrences, 150 variants and 90
choices, retains authored costs and outcome order, and requires explicit typed
boundaries for externally observed progress and tutorial programs. Service
execution covers 165 items, seven consumables, nine managed functions, 43
special goods, 811 direct rewards, 110 weighted reward pools, 57 recipes, 37
upgrades, ten forge definitions and 14 typed constants. Weighted draws use
stable candidates and preserve state and RNG when no legal result exists. The
38 shop poems permit one purchase per node; five Cyrene three-star poems are
non-shop activations. Gamble, curse-chest, Hex and Curio families are
source-proven empty and reject invented Universe content. Special-good effects,
configuration programs and battle-visible item/service effects have terminal
typed execution or audited metadata receipts in their generated partitions. The
replacement condition and alternatives for every policy are available through
`CurrencyWarsCatalog::policies()`.

## Debug surfaces

`CurrencyWarsRun::player_view` and `debug_view` expose the ordinary owned
Activity views, including IDs and configuration-backed mode state. They do not
dereference presentation data in the simulation core.

The CLI provides:

```text
starclock currency-wars config validate [--json]
starclock currency-wars inspect --route ID [--json]
```

The inspector command prints direct node IDs and their referenced template,
encounter, penalty/bonus, Gold and next-node IDs. A future game UI can resolve
those IDs through its own presentation/catalog adapter without changing battle
performance or authoritative state.
