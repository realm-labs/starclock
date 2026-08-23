# Repository state

Starclock maintains only the current source, data, behavior and test outputs.
Git history is the only historical record.

## Input lifetime

- Old replay files, serialized state, API envelopes and generated configuration
  are not supported after the current implementation changes.
- The repository has no migration, legacy-decoder or historical-evidence
  requirement.
- Canonical hashes and current goldens may change intentionally and are replaced
  atomically with the implementation that produces them.
- Dependency and toolchain pins remain build inputs.
- Game-version labels, source revisions and access dates in gameplay reference
  packs remain factual provenance and are retained.

## Current runtime

- `starclock-combat` owns deterministic single-battle execution.
- `starclock-activity` owns deterministic cross-battle orchestration.
- Standard battle, Memory of Chaos, Pure Fiction, Apocalyptic Shadow, Anomaly
  Arbitration, Standard Universe, Gold and Gears, and Swarm Disaster use the
  shared combat/activity kernels. Galactic Baseballer additionally has a
  stage-local Activity controller and a persistent-shop Activity controller;
  Fate/Star Rail NIGHT currently provides a
  production 4.4 tactical-card catalog, fail-closed loadout validator and shared
  Activity graph compiler. Neither event mode claims complete battle-effect
  parity.
- Currency Wars has a production 4.4 catalog and shared-Activity controller for
  26 routes/493 nodes, Standard and Overclock progression, economy, roster,
  builds, Bonds, all 834 investment lifecycles, services, encounters, immutable
  battle assembly, battle settlement and transition replay. All 2,367 mechanic
  programs now have terminal typed dispositions. Exact authored behavior and
  executable `VersionedProjectPolicy` boundaries remain distinguishable. Run,
  replay, adapters, hardening, performance, repository audits and exact
  coverage and fresh local macOS ARM64 clean-checkout acceptance are complete;
  only the hosted Windows x64, Linux x64 and macOS ARM64 native matrix remains.
- The active Currency Wars runtime foundation freezes 19,250 source
  obligations, 2,367 mechanic programs, 28 semantic fixture families and 12
  policy gaps. Generated exact-once dispositions currently classify 726
  evidence-only obligations as excluded, 2,260 source obligations as
  metadata-only and 16,264 as integrated. All 19,250 obligations are terminal,
  alongside all 529
  executable Activity/Rule-IR programs and all 1,838 structurally audited
  metadata-only programs are terminal. The generated
  execution ledger contains 43 bounded mechanic partitions (11 Activity and 32
  battle). The release state is
  `RuntimeCoverageCompletePendingNativeRelease`. The frozen
  Goal 21 coverage contract contains 97 complete-run targets covering
  all routes, difficulties, Gambits and focal roles, plus exact target fixtures
  for all 834 investments, 152 Bond levels, 683 Bond contributions, encounter
  axes, mechanic partitions, semantic families and policies. It also freezes
  the first Standard vertical slice, nine-component fresh replay identity,
  eight performance workload shapes and three native runtime runners. All 97
  legal matrix runs execute real battles and verify fresh replay, and every
  assigned axis fixture has production behavior evidence. Goal 21 foundation
  is complete through `G21-P0-B6` and runtime execution through `G21-P8-B4`;
  generated verification
  scaffolding binds all batches to owners, prerequisites,
  package-scoped commands, assigned denominator counts and terminal evidence.
  `G21-P1-B1` validates all 111 private generated tables and 78,607 rows before
  returning a catalog candidate, binding schema fingerprint, schema-lock
  digest, exact bundle digest and stable table-inventory content digest.
  `G21-P1-B2` now lowers and validates the complete flow catalog: one profile,
  four modules, two entries, two Gambits, 135 finish conditions, one area group,
  26 routes, 75 layers, five rooms, five domain compositions, 493 nodes and
  stage-flow rows, 97 difficulties and 56 rank-progression rows. Empty
  carry/reset sets remain explicit in lowered data; the later executable
  P3-B1 policy declares cross-Plane behavior separately. The generated row
  surface remains private. `G21-P1-B3` now lowers the complete economy
  catalog: one run currency and economy rule, 10 offer levels, five typed
  buy/sell price tiers, 10 Experience/team-size levels, three position
  definitions, 295 star states (266 role and 29 servant states), 189
  three-copy combination rules, three lifecycle rules, one Squad HP boundary,
  two action-value limits and two battle-result projections. Unknown policy
  text is rejected instead of being inferred, and roster-role star identity is
  kept distinct from servant star identity. `G21-P1-B4` now lowers all 77
  role/build mappings and references, twelve pinned Build source files, two Build
  substitution policies, 520 equipment definitions, 417 off-field conversions,
  4,784 Character Empowerments (4,652 role-position rows and 132 servant-skill
  rows), 341 battle overrides, 49 Bonds, 152 Bond levels and all 683 Bond
  contributions. The Bond catalog preserves the distinction
  between the 152 contributions explicitly listed by Bond definitions and 501
  auxiliary trait-effect/MazeBuff contribution rows instead of inventing
  parent links. The
  exact-once ledger now records 13,111 catalog obligations as lowered, 726
  evidence-only obligations as excluded with proof, and 5,413 later catalog
  obligations as pending. `G21-P1-B5` lowers all 834 investment definitions
  with their authored effect and lifecycle payloads plus 1,392 supporting rows
  for MazeBuff bindings, season membership, formula/blessing empty closure,
  occurrences/variants/choices, workbenches, services and offer candidates.
  Occurrence choices retain whether they point to a variant or directly to an
  occurrence, and those references are validated as a closed graph. This is
  catalog evidence only; the existing false runtime-binding flag remains until
  the assigned P5 execution batches. The ledger now records 15,218 exact
  catalog obligations, 726 exclusions and 3,306 pending obligations.
  `G21-P1-B6` lowers 25 encounter groups, 861 explicit source obligations,
  five formation waves, 306 enemy slots, 721 enemy affixes and 10 bounded boss
  pools. It also joins every one of the 2,367 mechanic-rule rows to exactly one
  pinned source path/digest and preserves its scope, trigger, ordered operation
  payload, lifecycle and current lowering disposition. Exactly 85 reviewed
  Activity progression rows now carry `runtime_lowered = true`; unresolved
  programs retain `false`. Phase 1 is
  therefore catalog-complete: all 18,524 runtime-owned source obligations are
  exactly lowered and all 726 evidence-only obligations are excluded with
  proof; no source catalog obligation remains pending. Remaining mechanic
  execution is pending in its generated P5-P6 partitions. `G21-P2-B1` now
  verifies all
  2,367 mechanism source records against the pinned Version 4.4 revision and
  inventories all 529 executable programs without treating the 1,838 audited
  metadata-only records as executable; every program now has a terminal
  disposition. The generated inventory covers 435
  configuration types across 2,541 structural variants, 176 expression shapes
  including 163 distinct postfix byte sequences and ten distinct opcode bytes,
  785 selector shapes, 131 trigger shapes, 198 state shapes and 18 lifecycle
  shapes. Every shape maps to an existing shared primitive, a named missing
  capability or a non-authoritative presentation boundary. `G21-P2-B2` adds a
  six-operator typed Activity comparison, four bounded collection/state reads,
  and atomic replacement operations for one counter entry, ordered-set member,
  inventory count and modifier stack count. Their definition-time type checks,
  runtime bounds, canonical ordering, events and rollback behavior are covered
  by the shared Activity transaction suite. Every Activity source shape now has
  sufficient generic primitives; source-program lowering remains assigned to
  its generated execution partition and earns no execution credit here. The
  `G21-P2-B3` adds a stable first-occurrence selector-union input, persistent
  effect-owned HP floors computed from maximum HP with explicit ceiling, and
  transactional Skill Point-cap mutation with clamping and ordinary resource
  events. Existing typed trigger phases, event filters, conditions, state
  slots, effects, operations and battle-result metric projections cover the
  remaining Combat shapes compositionally; source opcodes are not copied into
  the core. Pure formation/performance operations remain non-authoritative.
  `G21-P2-B4` adds catalog-owned generic Build contributions with stable IDs,
  exact source attribution, Any/Form/Path applicability and typed Build patches.
  Build specs select contribution IDs; compilation applies them in canonical
  order, rejects unknown, inapplicable or conflicting selections, and includes
  both definitions and selections in current digests. Owned/trial shapes remain
  ordinary character definitions and named presets, while dynamic battle state
  stays in Rule IR. The ten Version 4.4 postfix byte semantics are now the sole
  named shared-capability gap and remain explicitly unresolved rather than
  inferred from historical independent analysis. `G21-P2-B5` executes four
  shared capability probes, audits 206 Activity/Build/Combat/Rules Rust files
  for mode-ID branches, confirms zero admitted native handlers and freezes all
  43 generated partitions covering 2,367 programs with deterministic digests.
  Its configuration-program `VersionedProjectPolicy` affects 156 expression
  shapes in 194 programs: production must lower reviewed high-level source
  structure directly to typed IR, must not interpret raw postfix bytes, and
  cannot claim observed opcode parity. All affected programs now have terminal
  typed high-level lowerings; the replacement trigger remains released evidence
  for all ten opcode semantics. `G21-P3-B1` now executes
  Standard and Overclock entry checks, all 26 route topologies, 49 explicit
  Plane transitions, six settlement conditions and three production fixture
  families. Its terminal evidence covers 50 source obligations: 29 exact
  executable records and 21 audited metadata exclusions. Released data does
  not expose a route-to-Gambit selector or a per-slot Plane carry/reset table,
  so two executable VersionedProjectPolicy rows make those choices explicit:
  both Gambits use the complete route set, run and participant state carry,
  node-scoped offers reset at `NodeStart`, and a new run starts fresh. Phase 3
  continues with `G21-P3-B2`, which executes all 114 released battle-penalty
  rules through one typed boundary compiler: 89 finite action-value clocks and
  25 unlimited rules. Battle results project progress and remaining action
  value rather than caller-computed Squad HP loss. Victory precedes a
  same-boundary timeout; otherwise the executable VersionedProjectPolicy adds
  base loss, ceiling-rounded uncleared-progress loss and any threshold-failure
  extra, clamps Squad HP at zero, and atomically continues or fails through the
  automatic checkpoint. Two production fixture families cover malformed-result
  rollback, victory, timeout recovery and zero-HP run failure. `G21-P3-B3`
  executes the five-slot finite-pool shop, stable weighted offers, lock/carry,
  purchase, sale, direct Experience, battle income and pre-reward interest.
  Its generated audit terminalizes 29 exact source rows plus 25 metadata-only
  rows and proves that failed paid refreshes restore Gold, state and RNG.
  `G21-P3-B4` executes all 77 roles, 266 role-owned star states and 189
  three-copy transitions, including post-combination capacity checks, sale and
  maximum-star shop teardown. The 2,121 shared star/battle rows were retained
  intact for P4-B6. `G21-P3-B5` executes all ten team-level deployment caps,
  Front 1/4 and Back 6/9 boundaries, off-position eligibility, battle-entry
  Front minimum and same-boundary synthesis reconciliation. Five exact capacity
  constants are terminal; P4-B6 retains the ten shared team-level rows in the
  battle contribution snapshot. P5-B6 now consumes the separate 100-unit
  non-shop service overflow constant while ordinary shop/deployment mutations
  retain the nine-unit waiting-area cap. `G21-P3-B6` runs frozen seed 21000501
  through the production
  Standard route 100: paid refresh/purchase, deployment/Bond reconciliation,
  23 authored nodes, 20 validated battle handoffs/results, one recoverable
  non-victory checkpoint, three Supply decisions, all three Planes and terminal
  SSS settlement. The battle specifications remain explicit boundary stubs, so
  production build/enemy assembly, Combat execution and replay receive no early
  credit. `G21-P4-B1` now resolves every one of the 77 roles through its
  explicit `SpecialAvatarID` to a world-level 6 trial build, then joins the
  shared character and Light Cone by their source locators. It compiles exact
  progression, abilities, traces, Eidolon, Light Cone, relic main/sub
  aggregates and statically declared relic-set properties into immutable
  generic combat values. Owned builds are supplied as caller snapshots and
  selected field-wise without Activity/combat account queries or account
  mutation. The dynamic relic-set `AbilityName` and parameters remain retained
  for their P6 battle-program owners. `G21-P4-B2` now lowers all 148 usable
  equipment definitions into typed eligibility, dress-rule and category data;
  enforces three ordinary slots plus the released independent one-implant
  limit; and commits inventory replacement, unequip and role-sale teardown as
  atomic Activity operations. Role-only and trait-only rules use explicit
  roster relationships. All 252 backend-rank conversions join through the
  role's authored `BackendRankList` and apply cumulatively through selected
  Eidolon, while all 165 signature Light Cone conversions join the shared
  Light Cone identity and select exactly the equipped superimposition. Both
  conversion kinds activate only for Back roles. Selected equipment definitions
  and their static contribution inputs now cross the P4-B6 immutable boundary;
  battle-program effects are executed by their terminal P6 partitions, while
  upgrade/crafting rows are executed by P5-B6. `G21-P4-B3` now derives Character
  Empowerment as an immutable
  deployment snapshot rather than duplicate Activity state. Each matching
  deployed role resolves its exact Front/Back display and current role-star
  execution-skill IDs; 154 display rows, 4,052 Front skill rows and 446 Back
  skill rows are closed over all 266 role-star states. Empty-position
  relocation is one atomic Activity boundary, rejected relocation preserves
  state, and moving off-position or undeploying tears down the contribution on
  the next snapshot. Back execution skill lists remain distinct from shorter
  display-only lists. P4-B6 now resolves each stable shared Ability to its final
  effective level; battle operation execution remains owned by P6.
  `G21-P4-B4` now resolves all
  33 main Bonds, 16 subtraits and 152 authored levels from one immutable
  post-mutation snapshot. Subtrait selectors are typed as deployed-role,
  equipped-equipment, granted-front-trait, default-module or exact-module
  rules; module selection is automatic while roster/equipment selectors use an
  explicit Activity command. Parent deactivation or selector loss tears down
  the child level and contribution in the same transaction. The 187 direct
  layer properties are closed over 16 fixed-point property kinds and carry
  resolved role targets into the snapshot. `G21-P4-B5` resolves all 341 battle
  overrides: automatic Front Techniques, defeat-Energy scaling, lethal
  rescue/countdown reduction, Back battle events, Front special resources,
  global modifiers, Rank edits, summon replacements and Cyrene provider edits.
  Its only unresolved observed value is the lethal-rescue HP amount; the
  executable policy restores maximum HP and keeps the replacement trigger
  explicit. `G21-P4-B6` materializes one immutable battle contribution snapshot
  containing selected builds and effective Ability levels, all role/servant
  star data and 1,596 Rank attachments, team-level properties, equipment, 683
  Bond contributions, investments, seven influence rules, 254 exact
  contribution parameters and the resolved overrides. Snapshot identity binds
  Activity definition/config/state plus dynamic selections and the exact
  configuration digest; no Activity catalog lookup occurs after the boundary.
  `G21-P5-B1` now closes 334 Augments, 334 exact season memberships,
  seven selected Enhancements, ten remarks and three module-ban joins as a
  typed catalog. Explicit mode-program boundaries generate stable three-card
  offers from the Reward RNG after filtering by season, Plane, quality,
  Gambit, module and owned IDs. Selection clears the offer atomically;
  replacement names the old stable ID explicitly and never infers a category
  relationship. Selected Enhancements require an active authored Bond trait
  effect, derive `MaxStar` from the authoritative roster, charge configured
  Gold and cross the immutable contribution boundary. Public evidence does
  not establish automatic node scheduling, so the current executable policy
  leaves offer timing with explicit node/mode programs. `G21-P5-B2` additionally
  closes 84 Portal buffs, 376 Orbs, two Projections, 13 permanent Talents,
  40 season Talents, 11 maze-buff rows, four Orb displays, seven Portal remarks,
  83 Portal season memberships and two Portal module bans. Portal eligibility
  checks season, Gambit and module; Projection eligibility checks the owned
  role; both Talent graphs enforce authored prerequisites atomically. Released
  structured rows provide Talent costs but no currency key, so the executable
  boundary requires caller-confirmed payment and does not infer Gold.
  `G21-P5-B3` executes the Activity lifecycle for all 834 investment identities:
  stable ordering, eligibility, stacking/replacement and contribution refresh
  are atomic, while their battle-effect programs are executed by the terminal
  P6 source partitions.
  `G21-P5-B4` preserves the source-proven empty Blessing and Formula families
  as zero and carries the seven exact maze-buff Enhancement rows to battle
  assembly without inventing identities. `G21-P5-B5` validates 167
  Occurrences, 150 variants and 90 choices, resolves released costs and ordered
  outcomes, and keeps external progress and tutorial-program execution at
  explicit typed boundaries. `G21-P5-B6` executes 165 items, seven consumables,
  nine managed functions, 43 special goods, 811 direct rewards, 110 weighted
  reward pools, 57 recipes, 37 upgrades, ten forge definitions and 14 typed
  service constants. Empty gamble, curse-chest, Hex and Curio families remain
  explicitly empty; no Universe content is imported. Shop poems permit one
  purchase per node, the five Cyrene three-star poems are non-shop activations,
  and service role grants use the separately bounded 100-unit overflow policy.
  Phase 5 fixed batches are complete through `G21-P5-B6`. `G21-P5-A01` audits
  its 64 assigned `TutorialTask` programs as presentation-only metadata rather
  than installing UI waits or input locks in Activity. The authoring generator
  proves the closed operation vocabulary from exact source files, records
  ordered-shape digests and zero authoritative operations, and production Sora
  lowering exposes typed audit records. Across the complete tutorial family,
  76 programs contain 683 configuration-type occurrences and 168 tutorial-op
  occurrences; the remaining twelve are terminal metadata members of
  `G21-P5-A02`. That partition also audits the exact GridFight console world-
  prop graph: its 46 configuration nodes only switch prop presentation,
  interaction buttons, sound and the entrance UI, so the graph lowers to the
  typed `WorldPropAndUiEntry` metadata category instead of creating a second
  Activity entry state machine. `G21-P5-A03` and `G21-P5-A04` then lower all
  five `GridFightExpertRestrict` rows and all 80
  `GridFightSeasonExpScore` rows. Role cost tiers are filtered from shop
  candidates by Standard/Overclock run position. Current chapter and section
  are stored in Activity slots, so exact season score/experience projection is
  still queryable after completion or failure. Authored percentage values use
  an explicit `/100` formula boundary and remain exact fixed-point values; no
  unproved integer payout rounding is applied. `G21-P5-A05` through
  `G21-P5-A07` then classify 95 character overrides: 51 exact role, servant or
  summon bindings enter immutable contribution snapshots and 44 unreachable
  files remain metadata-only; all 459 decoder-layout programs are audited.
  `G21-P5-A08` adds the remaining bound Silver Wolf override and, with
  `G21-P5-A09`, executes two module role bans, the exact 77-role season pool,
  32 season/trait pools and 77 reference scores while auditing ten localized
  role labels. `G21-P5-A10` and `G21-P5-A11` classify NPC, world-prop/entity and
  animation audio/effect rows as presentation metadata. Their review proves
  that `GlobalTaskListTemplate_GridFight` is battle-owned because it contains
  wave/alive predicates, combat targets and modifier applications; the scope
  denominator is corrected to 520 Activity and 1,847 battle programs without
  changing the 2,367 total. All 249 executable Activity programs are terminal.
  `G21-P6-B1` terminally executes all 939 assigned source obligations: 25 Camp
  rows, five FormationWave rows, 160 GridFightMonster rows, 146 EliteGroup
  rows and 603 EnemyDifficulty rows. Production battle assembly now treats
  released StageConfig enemies as the level/wave/formation skeleton and fills
  those slots from the selected Camp or BossPool with stable GridFightMonster
  identities. It preloads all 2,400 monster/Stage-level combat inputs, applies
  exact per-monster star scaling and exact chapter/difficulty scaling, binds
  the selected roster into the bounded assembly-cache identity, and validates
  every assembled `BattleSpec` by constructing combat state. Exact reachability
  records 152 Camp-reachable monsters, eight current Camp-unreachable monsters,
  eight referenced EliteGroup definitions and 138 current unreachable
  EliteGroup definitions. Boss identity, Camp roster draw, enemy-star mapping
  and FormationWave selection remain explicit replaceable project policies.
  `G21-P6-B2` terminally executes the next 414 source obligations: 51 Affix
  definitions, 67 Affix MazeBuff rows and 296 stage, difficulty/rank and Action
  Value inputs. All 51 Affix semantics compile before battle creation into
  typed stat scaling, Activity-boundary changes or generic combat modifiers,
  selectors, effects, triggers and operations; no Affix content ID appears in
  shared resolver branches. Enervation's under-equipped owner modifier is
  inherited by subsequently created memosprites. Time Assassin spawning and
  otherwise unproved equal-value tie breaks remain explicit replaceable
  project policies. `G21-P6-B3` then terminally binds 1,340 integrated and 88
  excluded assembly rows into immutable contribution snapshots and a bounded
  assembly cache. Front `EnergyBar`/`MaxSP` properties project to personal
  current/maximum Energy rather than team Skill Points; selected C04 abilities
  declare a bounded persistent `assist-use` team resource before battle
  creation. Finite Action Value battles alone install lethal rescue, lost
  results project zero remaining Action Value, and Energy Disappearance spends
  at most the attacker's current Energy. `G21-P6-B4` executes 1,122 node,
  Stage, route and bonus rows
  through atomic battle-result settlement, while `G21-P6-B5` proves stale and
  rejected assembly/settlement rollback plus fresh transition replay.
  `G21-P6-M01` through `M09` execute 263 reviewed high-level battle programs as
  explicit `VersionedProjectPolicy` Rule IR and terminally audit 285
  presentation-only programs. `M10` through `M13` add 17 exact enemy, AI and
  global-task-template programs and 147 metadata-only programs. Finally,
  `M14` through `M32` sequentially bind exact-once receipts for 1,135
  metadata-only records: 93 skill-description modifications, 997 skill-icon
  routes and 45 resource-preload files, all with zero authoritative operations.
  All 32 battle mechanic partitions and all 43 mechanic partitions overall are
  terminal. `G21-P7-B1` adds a deterministic `starclock-ai` controller that
  consumes only offered Activity and battle commands, executes catalog enemy
  AI, preserves exact command/event/state-hash traces and completes real
  seven-battle Standard and Overclock runs. A same-seed Standard pair produces
  identical reports. The production run also closed generic keyed Toughness-
  layer create/remove execution and transitive nested-program selector
  resolution. `G21-P7-B2` adds `currency-wars coverage` and production
  Standard/Overclock CLI runs with canonical replay export. The replay records
  Activity decisions, nested battle commands, states and events, and fresh
  verification reconstructs immutable Sora-backed inputs before requiring
  exact byte equality. The initial four consumed components remain a strict
  subset of the frozen nine-component P7-B5 target, which also owns
  first-divergence diagnostics. Existing validation and route inspection are
  covered by the same adapter regression suite. `G21-P7-B3` adds bounded
  Currency Wars Agent manifests and sessions over the shared Activity registry.
  The public surface exposes 26 route summaries, 97 difficulty summaries,
  configuration/content digests, player-visible Activity state and opaque
  current legal actions; it does not expose generated rows, debug state or the
  private combat catalog. Encounter and preparation remain distinct actions,
  with preparation settling exactly one real nested battle through the shared
  deterministic controller. `G21-P7-B4` exposes the same registry session via
  MCP mode `currency-wars`, with bounded manifest/rules resources, existing
  Activity OAuth scopes, exact tenant/principal ownership, response-loss
  idempotency and explicit close-as-cancel behavior. MCP cancellation followed
  by retry cannot duplicate a commit. Activity event cursors page accepted-
  action summaries at 256 entries from an 8,192-entry retained window and
  reject future or expired cursors. MCP owns no second runtime state. Runtime
  `G21-P7-B5` binds all nine replay components and first-divergence reporting,
  while `G21-P7-B6` executes all 97 legal matrix entries through real nested
  battles and fresh replay without concession; a battle fault fails the
  matrix, while an ordinary failed run remains a lawful terminal result.
  `G21-P8-B1` and `B2` close hardening and eight
  measured workloads; `B3` passes dependency, architecture, generated-data,
  provenance and workbook audits; `B4` closes all source/program/fixture/policy
  dispositions. Runtime execution is complete through `G21-P8-B4`; the local
  macOS ARM64 clean-checkout half of `G21-P8-B5` passes on the current tree,
  while the hosted Windows x64, Linux x64 and macOS ARM64 native matrix remains
  the final release gate.
- The Version 4.4 challenge bundle contains 13 Memory stages (including one
  three-node Starward stage), five Pure Fiction stages and five Apocalyptic
  stages, plus one five-stage Anomaly Arbitration profile. The first three
  challenge modes support two or three locked teams; all four support battle
  handoff, score/objective projection, canonical Activity state and an owned
  `ActivityDebugView` for adapter-side inspection.
- Anomaly Arbitration owns three locked disjoint Knight teams, one King team,
  arbitrary Knight order/retries, normal-King gating, direct Plight entry,
  Quadrant selection and max-preserving stage records. Its 150/100 cycle
  windows, King-protection effect and fixed runtime-roster seam remain three
  explicit `ProjectPolicy` rows.
- `starclock challenge config validate` lowers all four production profiles
  and compiles the 43 Memory/Pure-Fiction/Apocalyptic encounters over the
  production combat catalog.
  Temporary enemy behavior donors and all unverified mechanics remain visible
  as generated `ProjectPolicy` rows.
- The event production bundle contains two Galactic Baseballer profiles, 13
  stage rows, 102 stage-period candidates, 87 equipment identities, 27
  synthesis recipes, 114 shop price steps, 56 Adventure Strategies, seven
  stage team bonuses, two score rules and six explicit policies. Its
  controller compiles two to four authored period ranks, weighted encounter
  selection, battle handoff, score accumulation, checked
  acquisition/duplicate-upgrade choices and atomic synthesis. The
  separate persistent-shop Activity commits checked purchases atomically and
  projects exact initial-weapon-level and accessory-slot upgrades into later
  stage state; MazeBuff purchases, Strategies and team bonuses remain typed
  identity/parameter data with false Combat bindings. The
  same bundle contains six Fate Case Boards/18 policy-grouped nodes, six owners,
  four deck profiles, seven recommendations, 107 tactical cards, 6/4/15
  story/challenge/map fight locators and 16 explicit policies. `starclock event
  config validate` lowers and validates both catalogs. The legacy 3.4
  `FateHougu`/`FateReiju` and `425001..425008` Stage rows are not treated as the
  4.4 battle surface. All 87 Baseballer equipment effects, 102 shop MazeBuff
  steps, 56 Strategies, seven team bonuses and all 107 Fate card bindings
  remain non-exact until their ability programs and end-to-end battle fixtures
  are promoted.
- Combat resolution stops at stable boundaries between independent actions.
  `Advance` resumes deterministic work; adapters may submit it automatically
  when no ready Ultimate needs to be exposed. If a normal decision already
  coexists with the boundary, `Advance` closes only the Ultimate-insertion
  opportunity and leaves that decision pending.
- A ready Ultimate can be requested at any stable action boundary, including
  another unit's turn. The request creates `PreparedActionState`; a later exact
  commit selects its target and executes it, while cancellation restores the
  suspended continuation without paying resources.
- Trigger-produced actions use an authoritative deterministic queue that may
  survive a stable boundary. Trigger-produced child actions execute through an
  explicit heap-backed frame stack rather than recursive resolver calls.
- Segmented Ultimates use a bounded persistent `ActionFrame` between complete
  segments. It retains one action identity, the suspended continuation, prior
  typed inputs and payment state; no frame may survive action resolution.
- Replay records and verifies only data produced by the current tree.
- `starclock-inspector` captures ID-only owned battle snapshots, including
  stable/prepared boundaries, segmented action frames, queued reactions and
  allocator cursors, plus diffs and optional bounded resolver diagnostics
  without presentation metadata.
- CLI, Agent API and MCP are current adapters over the domain crates.
- `starclock currency-wars config validate` validates the production bundle;
  `starclock currency-wars inspect --route ID` emits ID-only route data for
  debugging and future presentation adapters.

## Verification

Focused development and ordinary local completion use
`cargo test -p <package> [filter]` and package-scoped Clippy directly. The full
workspace suite runs in CI and locally only for shared-boundary changes or an
explicit merge check. Current Sora/workbook/data validators run only when their
owned inputs change. Seeded matrices, large property corpora and performance
workloads are explicit exhaustive checks rather than default edit-loop gates.

The current authoring toolchain is checksum-bound Sora 0.6.1. Its capability
golden and Currency Wars project use stable project/view identities and
schema-local table IDs; the Currency Wars workbook metadata, generated reader
and 78,607-row bundle have been regenerated under that contract.

The default test profile minimizes compilation and linking for workspace
crates. Third-party dependencies and the combat hot loop retain light
optimization. Complete gameplay runs are excluded from default adapter tests
because they measure end-to-end simulation rather than local API behavior.

The Rust property/corruption corpus runs explicitly with
`cargo test -p starclock-test-kit --features exhaustive --test exhaustive_suite`.
Adapter corruption, concurrency and TCP load checks run with
`cargo test -p starclock-test-kit --test adapter_suite -- --ignored`.

Complete dynamic Universe replay reconstruction runs with
`cargo test -p starclock-test-kit --test universe_suite dynamic_battle_assembly::dynamic_replay_reconstructs_each_snapshot_and_reports_first_divergence -- --exact --ignored`.

Complete Agent API gameplay/replay checks run with
`cargo test -p starclock-agent-api --lib public_offers_complete_real_battles_and_export_fresh_replay -- --ignored`.
Complete CLI gameplay/replay and text/JSON parity checks run with
`cargo test -p starclock-cli --test universe_cli -- --ignored`.

The two current Universe seeded matrices run explicitly with
`cargo test -p starclock-mode-universe seeded_run_tests::frozen_matrix -- --ignored`.

The machine-readable counterpart is `policy/state.json`.

## Identity audit

Retained because they describe external facts or build inputs:

- Cargo package/dependency versions and the Node/Sora/MCP protocol pins;
- game-version labels in gameplay reference packs;
- source repository commits, access dates and evidence digests.

Removed from current runtime surfaces:

- replay v1/v2/v3 modules, alternate decoders and payload-version selectors;
- replay header, component-set and record-payload revision prefixes; the current
  codec keeps only framing magic, semantic type discriminants and bounds;
- `current` forwarding modules and versioned Rust/example filenames;
- Agent API schema selection and `schema_revision` request/response fields;
- CLI schema/Goal identifiers and runtime/release evidence snapshots;
- benchmark and seed-matrix schema/workload/executor revision fields;
- textual component, controller and build revisions duplicated by exact digests;
- Activity codec/RNG/scope/handler revisions duplicated by current structure and digests;
- Combat catalog/rules/numeric/RNG/state-codec revisions and the duplicate
  `BattleSpecDigest` wrapper;
- Combat input/state codec revision sentinels; `SCBI`/`SCBS` framing magic and
  semantic field discriminants remain;
- generated `ConfigManifest` data/rules/numeric/RNG/state/replay revision
  labels and the old Goal coverage digest; the manifest now carries only
  gameplay `game_version`, source `snapshot_date` and pinned
  `sora_cli_version`;
- the production configuration golden registry and its `--bless` path;
  verification now rebuilds directly from the current schema/workbooks and
  compares current generated artifacts;
- the deleted Goal-manifest verifier dependency from production bootstrap;
- four obsolete partition authoring scripts (3,215 lines) and their
  `G01-P7-*` partition ledger; the remaining current ConfigManifest author is
  part of a focused current-workbook adapter;
- Goal schema/id/generated-date metadata from the retained core-combat gameplay
  selection manifests;
- the unused `handler_version` Native Handler column across schema, workbook,
  generated reader and bundle;
- nine Standard Universe path-runtime revision constants, their duplicate
  digest inputs and byte-for-byte digest snapshots;
- the Standard Universe entry revision and its hard-coded core catalog digest;
  composition now requires the current combat and build catalogs to agree;
- the remaining Standard Universe path, blessing, ability, curio, occurrence,
  service, encounter and run revision constants and duplicate digest inputs;
- Standard Universe battle assembly, contribution, materialization, snapshot
  and event-commitment revisions and their byte-for-byte digest snapshots;
- Gold and Gears runtime-coverage, baseline-fixture and seeded-run revisions;
- Swarm Disaster runtime-coverage digest, baseline-fixture revision and
  seeded-run revision;
- twelve Swarm Disaster mechanic-rule runtime revisions and their exact digest
  snapshots; current behavior remains covered by contract and execution tests;
- seven Swarm Disaster entry-policy revision constants and the Communing Trail
  digest snapshot;
- Swarm Disaster content, occurrence, Path, semantic-fixture, service and
  adventure runtime revisions and their fixed digest snapshots;
- Swarm Disaster encounter, enemy-composition, battle-materialization and
  battle-snapshot revisions and their byte-for-byte digest snapshots;
- Swarm Disaster baseline entry/controller and performance-matrix hash-domain
  versions, plus fixed baseline controller digest snapshots;
- seven Gold and Gears mechanic execution/profile revisions and their direct
  fixed digest snapshots;
- Gold and Gears `VersionedProjectPolicy` accuracy naming; current inferred
  rules are now classified simply as `ProjectPolicy`;
- seven Gold and Gears topology/cognition/knowledge/dice policy revision
  constants that were referenced only by self-asserting tests;
- Gold and Gears Conundrum runtime, numeric-policy and combat-modifier
  revisions and the fixed modifier-set digest snapshot;
- Gold and Gears baseline-controller, performance-matrix and battle-cache-key
  hash-domain versions, plus fixed baseline digest snapshots;
- empty deferred relic/planar build fields and their placeholder document.

Mode and generated content modules still contain textual `*_REVISION` domain
labels used inside digest construction. They are not compatibility branches,
but they are redundant current-tree identity and remain cleanup debt. Replace
them with the underlying canonical content/configuration digests. Retain only
semantic type/variant discriminants; remove numeric sentinels whose only
meaning is a codec or payload revision.

The core-combat gameplay reference manifests remain. Their per-row
`implementation_state` labels and `standard-v1` stable IDs still mix workflow
state/version naming into gameplay selections and remain cleanup debt; the
actual character, Light Cone, enemy, encounter and scenario references must be
preserved while that metadata is removed.
