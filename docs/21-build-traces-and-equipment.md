# Character Builds, Traces, and Equipment

This document defines how a selected character form, progression choices, Light Cone, relics, and planar ornaments become deterministic combat inputs. It is normative for build validation and compilation. It deliberately excludes account ownership, acquisition, upgrade costs, inventory management, farming, crafting, gacha, and UI.

## Architectural boundary

Progression/equipment content is peripheral to the resolver but not external to the simulation contract:

```text
Excel/Sora -> starclock-data -> CombatCatalog + BuildCatalog
                                      |             |
mode/scenario -> CombatantBuildSpec   |             |
                |                     |             |
                +--------> starclock-build <--------+
                                  |
                             CompiledBuild
                    +-------------+----------------+
                    |                              |
        BuildCompilationReport        ResolvedCombatantSpec
        (peripheral attribution)       (generic combat input)
                                                   |
                                mode/activity binds opaque spec
                                                   |
                                               BattleSpec
                                                   |
                                                   v
                                        starclock-combat resolver
```

Ownership is split as follows:

| Owner | Responsibility |
|---|---|
| `starclock-data` | Convert Sora rows into separate validated `CombatCatalog`, `BuildCatalog`, activity, and mode definitions. |
| `starclock-build` | Own build-specific definitions/specs/policies and compile one exact build into generic combat-domain output. |
| Mode/application layer | Offer/edit `CombatantBuildSpec` values, invoke `starclock-build`, and provide only offered compiled results to activities. |
| `starclock-activity` | Store participant `ResolvedCombatantSpec` values/digests opaquely and enforce whole-loadout replacement/lock boundaries. |
| `starclock-combat` | Define and validate `ResolvedCombatantSpec`, instantiate it, and resolve battle without any build-specific definitions. |
| Account/presentation layer | Ownership, materials, recommendations, editing UI, save synchronization, and display rounding; outside current scope. |

A production `BattleSpec` contains only digest-bound `ResolvedCombatantSpec` values. Production orchestration accepts those values only from a verified `starclock-build` result or frozen compiled preset. The combat crate validates combat-domain integrity but does not prove account/build legality. Synthetic combat specs are allowed for formula tests under an explicit scenario/profile policy and carry a distinct source/spec digest.

Dependency direction is one-way:

```text
starclock-build    -> starclock-combat
starclock-activity -> starclock-combat
mode/application  -> starclock-build + starclock-activity + starclock-combat
```

`starclock-combat` and `starclock-activity` never depend on `starclock-build`. `starclock-build` does not mutate an activity or running battle.

## Definition, selection, and runtime

Keep three layers distinct:

```text
Definition                         Build selection                    Battle runtime
----------                         ---------------                    --------------
CharacterBuildDefinition           CombatantBuildSpec                 UnitState
TraceNodeDefinition       +        exact unlocked Trace IDs   ->      RuleInstanceState
EidolonDefinition                  ability investment levels          ModifierInstance
LightConeDefinition                cone level/promotion/S              resolved abilities
RelicSet/Affix definitions         concrete relic pieces              base contributions
```

Definitions are versioned catalog content. A build selection is immutable input for one battle or activity lock scope. Runtime state contains only instantiated combat state and source-linked rules/modifiers; it does not retain an editable progression tree.

## Build specification

The normalized input shape is:

```rust
pub struct CombatantBuildSpec {
    pub form: UnitDefinitionId,
    pub level: CharacterLevel,
    pub promotion: PromotionStage,
    pub ability_levels: AbilityLevelSelection,
    pub traces: TraceSelection,
    pub eidolon: EidolonLevel,
    pub light_cone: Option<LightConeLoadout>,
    pub relics: Vec<RelicPieceSpec>,
    pub policy: LoadoutValidationPolicyId,
}
```

All collections have explicit maximums and canonical orders. `TraceSelection` is stored as an ordered set of stable `TraceNodeId` values. `RelicPieceSpec` carries a build-local stable piece ID so source attribution does not depend on vector position.

Convenience inputs such as “all legal Traces” or a named build preset are authoring/CLI forms. They must expand to an exact normalized `CombatantBuildSpec` before `starclock-build` compilation and build hashing. Activities and battles never receive an unexpanded convenience form.

`LoadoutValidationPolicy` declares whether the build must be game-legal, may omit progression for a restricted trial, or may use synthetic exact values for formula tests. Production profiles use `GameLegal`. A synthetic build is visibly tagged in results/replays and cannot satisfy production content coverage.

## Character and level definition

`CharacterBuildDefinition` is owned by `BuildCatalog` and references the corresponding generic combat form:

```rust
pub struct CharacterBuildDefinition {
    pub form: UnitDefinitionId,
    pub rarity: Rarity,
    pub stat_curve: CharacterStatCurveId,
    pub ability_levels: AbilityLevelTableId,
    pub trace_graph: TraceGraphId,
    pub eidolons: EidolonSetId,
    pub source_records: Vec<SourceRecordId>,
}
```

The matching `UnitDefinition` in `CombatCatalog` owns combat-domain form identity, element/path/tags, base resource definitions, ability IDs, and innate battle rules. `starclock-data` may derive both domain definitions from related Sora rows, but neither catalog embeds the other's definition. The build catalog references combat definitions only by stable IDs and declared compatible digest/revision.

Level/promotion data supplies exact base HP, ATK, DEF, SPD, and any authored build-time base contribution at each supported boundary or through a validated integer/fixed-point curve. The compiler must never interpolate with floating point or infer a missing promotion row.

Alternate combat forms are separate `UnitDefinitionId` values even when they share a person or display name. Form switching that changes the combat definition is an authored transformation or activity build replacement, not an implicit lookup by account character.

## Ability levels

Ability investment and effective level are different values:

```rust
pub struct ResolvedAbilityLevel {
    pub invested: AbilityLevel,
    pub bonus: SignedAbilityLevel,
    pub effective: AbilityLevel,
    pub cap: AbilityLevel,
}
```

`AbilityLevelSelection` records invested levels for each levelled ability family. Trace/Eidolon patches may raise caps, add a level bonus, unlock an ability, or replace a program. Compilation calculates the effective level with checked arithmetic and explicit caps. It never silently clamps an illegal input.

An ability coefficient row is selected only after the effective level is resolved. Basic ATK, Skill, Ultimate, Talent, memosprite/summon ability, and other authored level families remain distinct IDs even when the UI presents related levels together.

## Trace graph

Every battle-relevant major Trace, minor-stat node, ability unlock, and level-cap node is explicit:

```rust
pub struct TraceNodeDefinition {
    pub id: TraceNodeId,
    pub character: UnitDefinitionId,
    pub kind: TraceNodeKind,
    pub prerequisites: Vec<TraceNodeId>,
    pub promotion_requirement: PromotionStage,
    pub patches: Vec<BuildPatch>,
    pub source_records: Vec<SourceRecordId>,
}

pub enum TraceNodeKind {
    MajorPassive,
    MinorStat,
    AbilityUnlock,
    AbilityLevel,
    BasicLevel,
}
```

Unlock currency/material costs and UI coordinates are not combat fields. Prerequisites and promotion requirements are retained because `GameLegal` validation needs them. A profile may explicitly choose a relaxed trial policy, but the resulting policy ID is hashed.

The Trace graph must be acyclic. Exact selection validation requires every selected node to belong to the form and all prerequisites to be selected or supplied by an explicit profile grant. Patch evaluation order is the graph's validated canonical topological order with `TraceNodeId` as the final tie-break; it never depends on workbook row order or the order in which a player unlocked nodes.

Minor-stat nodes produce persistent modifier bindings at their declared stat stage. Major passives normally add rules, state slots, modifiers, or ability patches. Do not add one field to `UnitState` per Trace.

## Eidolons

An Eidolon set contains exactly six ordered definitions for released ordinary forms unless the form schema explicitly declares another policy:

```rust
pub struct EidolonDefinition {
    pub character: UnitDefinitionId,
    pub level: EidolonLevel,
    pub patches: Vec<BuildPatch>,
    pub source_records: Vec<SourceRecordId>,
}
```

Selecting E`n` applies definitions `1..=n` in ascending order. Within one Eidolon, patch sequence is explicit and stable. A patch cannot refer to a field, ability, rule, state slot, or modifier that does not exist at that point in compilation.

Eidolons may add rules, change resource caps, replace/extend programs, add modifier definitions, alter state-slot bounds/defaults, and adjust ability level bonus/caps. They do not execute battle operations during build compilation; an entry effect is represented as a rule triggered at the appropriate battle boundary.

## Build patches

Trace and Eidolon changes use one typed closed patch language:

```rust
pub enum BuildPatch {
    AddRule(AddRulePatch),
    RemoveRule(RemoveRulePatch),
    AddModifier(AddModifierPatch),
    AddAbility(AddAbilityPatch),
    ReplaceAbility(ReplaceAbilityPatch),
    PatchAbility(PatchAbilityPatch),
    AdjustAbilityLevel(AbilityLevelPatch),
    AdjustResourceDefinition(ResourceDefinitionPatch),
    AdjustStateSlot(StateSlotDefinitionPatch),
    AddTag(AddTagPatch),
}
```

Patch payloads are schema-typed and reference exact stable IDs. No patch contains Rust, a field path string, arbitrary JSON, or a partial serialized definition. Catalog validation checks the entire E0-to-E6 and legal-Trace compilation space for missing targets, conflicts, and illegal ordering where feasible.

When two patches intentionally affect the same target, the target definition declares whether composition is additive, replacement, or ordered program editing. Unspecified last-write-wins behavior is forbidden.

Patches apply to a private build-compilation workspace. They never mutate `CombatCatalog` or `BuildCatalog`. The compiler emits canonical generic resolved ability/resource/rule/modifier values or selects validated combat-definition variants in its `ResolvedCombatantSpec` output.

## Technique representation

A battle-relevant Technique is an ability or entry rule referenced by the character/build and selected scenario entry state. Technique Point consumption, overworld targeting, movement, and encounter initiation UI are outside battle. The resulting entry effect, damage, weakness application, field, or resource change is compiled into the `BattleSpec` entry program with its source retained.

A build does not imply that every Technique is active. The scenario/activity must explicitly bind the Technique use and any legal target/context before battle creation.

## Light Cone definition

```rust
pub struct LightConeDefinition {
    pub id: LightConeId,
    pub rarity: Rarity,
    pub path: Path,
    pub stat_curve: LightConeStatCurveId,
    pub superimposition: SuperimpositionDefinitionId,
    pub passive_rule: RuleBundleId,
    pub applicability: LightConeApplicabilityPolicy,
    pub source_records: Vec<SourceRecordId>,
}

pub struct LightConeLoadout {
    pub definition: LightConeId,
    pub level: LightConeLevel,
    pub promotion: PromotionStage,
    pub superimposition: Superimposition,
}
```

The stat curve provides exact base HP, ATK, and DEF contributions for the selected level/promotion. If future content adds another base contribution, the stat-curve schema must explicitly support it; the compiler does not infer arbitrary stats from a generic map.

Superimposition is a bounded selector over S1-S5 parameter rows for one passive definition. It is not five separate item definitions. Every scalable parameter supplies five values or an explicit validated constant-across-ranks policy.

For the standard compatibility profile, binding a Light Cone is separate from activating its passive: base-stat contributions apply from the equipped cone, while the passive rule requires the declared wearer applicability, normally a matching Path. The applicability policy is data and must be evaluated explicitly; do not reject or activate a cone merely because its `path` field differs without consulting that policy.

Light Cone base HP/ATK/DEF contributions enter the base-stat composition stage defined by the stat pipeline. Passive effects become source-linked rule/modifier instances and remain conditional/dynamic according to their definitions.

## Relic and planar definitions

A relic or planar set is one set definition with one or more piece-count thresholds:

```rust
pub struct RelicSetDefinition {
    pub id: RelicSetId,
    pub family: RelicSetFamily,
    pub allowed_slots: OrderedSet<RelicSlot>,
    pub thresholds: Vec<RelicSetThreshold>,
    pub source_records: Vec<SourceRecordId>,
}

pub struct RelicSetThreshold {
    pub required_pieces: PieceCount,
    pub rule_bundle: RuleBundleId,
}
```

Thresholds are evaluated independently in ascending count, so a four-piece loadout may activate both two- and four-piece rules when the definition says so. Set rules use ordinary Rule IR and modifier definitions; the resolver has no `if relic_set == ...` branches.

Affix definitions declare stat/value domain, legal slots, rarity/tier, main/sub status, curve or roll table, cap, unit, and provenance. Main-affix level curves and sub-affix roll tiers are separate definitions.

## Concrete relic pieces

Combat receives concrete virtual pieces even though account inventory is out of scope:

```rust
pub struct RelicPieceSpec {
    pub id: BuildRelicPieceId,
    pub set: RelicSetId,
    pub slot: RelicSlot,
    pub rarity: Rarity,
    pub level: RelicLevel,
    pub main_affix: MainAffixId,
    pub sub_affixes: Vec<RelicSubAffixSpec>,
}

pub struct RelicSubAffixSpec {
    pub affix: SubAffixId,
    pub magnitude: RelicSubAffixMagnitude,
}

pub enum RelicSubAffixMagnitude {
    Rolls(Vec<SubAffixRollTierId>),
    ExactValidated(StatValue),
}
```

`Rolls` is preferred because it preserves an auditable exact sum. `ExactValidated` supports imported builds that know only a canonical total; under `GameLegal`, that value must equal a reachable sum under the selected rarity/tier/roll rules. Ambiguous displayed rounded values require an explicit import confidence/provenance note and cannot be silently promoted to exact evidence. `Synthetic` policy may accept an otherwise unreachable exact value for formula fixtures, with the policy recorded in the digest.

Standard loadout policy requires legal/unique slots, legal main affixes, allowed sub-affixes, bounded roll counts, no duplicate sub-affix type on one piece, and valid rarity/level combinations. Mode/trial profiles may select another explicit policy; the compiler never guesses exceptions from mode ID.

Affix contributions become persistent source-linked modifier bindings at their declared stat stages. Set membership is counted from validated equipped pieces in canonical slot then piece-ID order.

## Deterministic compilation

`starclock-build::LoadoutCompiler` receives immutable `BuildCatalog` and `CombatCatalog` references plus one exact `CombatantBuildSpec`, then performs these stages:

```rust
impl LoadoutCompiler {
    pub fn compile(
        &self,
        build_catalog: &BuildCatalog,
        combat_catalog: &CombatCatalog,
        spec: &CombatantBuildSpec,
    ) -> Result<CompiledBuild, BuildCompileError>;
}
```

1. resolve the form and exact loadout policy against the captured catalog revision;
2. validate character level/promotion and load the exact character base-stat row;
3. validate invested ability levels without applying bonuses yet;
4. validate the Trace set and apply Trace patches in canonical graph order;
5. apply Eidolon definitions from E1 through the selected level;
6. resolve ability caps/bonuses/effective levels and bind exact coefficient/program rows;
7. resolve Light Cone base-stat rows and passive applicability/S1-S5 parameters;
8. validate relic pieces, compute exact affix contributions, count set thresholds, and bind set rules;
9. apply profile-declared trial/borrowed or battle-entry build patches in explicit source/priority order;
10. validate the resulting abilities, state slots, resources, rules, modifiers, tags, and source links;
11. canonicalize build input/contributions and compute `CombatantBuildDigest`;
12. construct a generic `ResolvedCombatantSpec` and compute its independent `CombatantSpecDigest`.

Compilation is pure and consumes no battle RNG or runtime IDs. Input collection order does not affect output. Errors are typed with stable IDs and stages; localized item names and spreadsheet coordinates are diagnostic metadata outside canonical hashes.

## Compiled output and combat boundary

The build crate returns both a generic combat value and a peripheral diagnostic/coverage report:

```rust
pub struct CompiledBuild {
    pub combatant: ResolvedCombatantSpec,
    pub report: BuildCompilationReport,
    pub build_digest: CombatantBuildDigest,
}
```

`ResolvedCombatantSpec` is defined by `starclock-combat`, not `starclock-build`:

```rust
pub struct ResolvedCombatantSpec {
    pub form: UnitDefinitionId,
    pub level: UnitLevel,
    pub base_stats: BaseStatContributions,
    pub resources: ResolvedResourceDefinitions,
    pub abilities: ResolvedAbilitySet,
    pub rules: OrderedRuleBindings,
    pub modifiers: OrderedModifierBindings,
    pub entry_programs: OrderedEntryPrograms,
    pub sources: SourceBindings,
    pub digest: CombatantSpecDigest,
}
```

The type contains no Trace, Eidolon, Light Cone, Superimposition, relic, affix, preset, or inventory field. `BaseStatContributions` exposes combined base inputs to stat queries; later-stage and conditional values remain modifier/rule bindings. Conditional effects are never pre-applied to an unconditional “final ATK” field.

`SourceBindings` contains generic stable source IDs, source classes/tags, owner links, and source digests needed by combat attribution. The resolver may filter on registered generic classes/tags but cannot branch on build-system IDs. `BuildCompilationReport` separately maps those generic source IDs back to character, Trace, Eidolon, Light Cone, relic piece/threshold, or profile rows for diagnostics, provenance, and coverage.

`BattleSpec` embeds only canonical `ResolvedCombatantSpec` values. `Battle::create` verifies the combat catalog, spec, combatant-spec digests, referenced abilities/rules/modifiers/programs, value domains, and source bindings. `starclock-build` or replay orchestration separately verifies the BuildCatalog, raw build, policy, compilation report, and build digest. Replays bind both digests where build provenance is available; low-level synthetic battle replays may contain only a synthetic combatant-spec digest.

## Activity and lock behavior

`starclock-activity` stores `ResolvedCombatantSpec` values and digests as opaque participant loadouts. `ParticipantPolicy` declares:

- whether a participant is owned, fixed trial, borrowed, drafted, generated, or transformed;
- whether a whole resolved spec may be selected/replaced at a decision boundary;
- the Node/Attempt/Section/Activity scope at which its digest locks;
- uniqueness across team/node/section/activity;
- whether HP/resources persist independently from build replacement;
- which offered replacement IDs are legal at the current decision.

Detailed field editability belongs to a `starclock-build` `BuildEditPolicy` selected by the mode/application. It produces a new exact build and compiled combatant spec before offering a whole-loadout replacement command to `starclock-activity`. Activity commands choose only prevalidated offered IDs; they cannot inject raw build fields or an arbitrary resolved spec.

Changing a build between battles creates new build and combatant-spec digests. A live battle never observes an equipment hot reload. Mode combat modifiers normally remain activity/mode `RuleBundle` inputs rather than pretending to be relic pieces; an actual mode mechanic that replaces equipment invokes `starclock-build` outside the generic activity core and offers the resulting resolved spec.

Trial and borrowed characters do not require account records. Their exact build definitions and compiled presets are content rows with normal provenance and coverage. Account ownership checks happen before or outside activity construction.

## Excel and Sora schema

The normalized workbook boundary includes at least:

| Workbook / table | Key content |
|---|---|
| `Character / Character` | Form identity, path/element/rarity, base resources, stat curve, ability/Trace/Eidolon references. |
| `Character / CharacterStat` | Exact level/promotion base-stat rows or validated curve samples. |
| `Character / TraceNode` | Kind, form, prerequisites, promotion requirement, canonical ID. |
| `Character / TracePatch` | Ordered typed patches owned by one Trace node. |
| `Character / Eidolon` | Form and E1-E6 identity/order. |
| `Character / EidolonPatch` | Ordered typed patches owned by one Eidolon. |
| `Ability / AbilityLevel` | Ability family, effective level, coefficients/program references, caps. |
| `Equipment / LightCone` | Identity, path/rarity, stat curve, passive rule, applicability policy. |
| `Equipment / LightConeStat` | Level/promotion base HP/ATK/DEF rows. |
| `Equipment / LightConeSuperimposition` | Passive parameter values for S1-S5 or validated constant policy. |
| `Equipment / RelicSet` | Set identity/family/allowed slots. |
| `Equipment / RelicSetThreshold` | Piece count and rule-bundle binding. |
| `Equipment / RelicAffix` | Stat, value domain, main/sub kind, legal slots/rarities/tiers. |
| `Equipment / RelicMainAffixValue` | Main-affix value by rarity/tier/level. |
| `Equipment / RelicSubAffixRoll` | Exact sub-affix roll value by rarity/tier/roll tier. |
| `Build / BuildPreset` | Optional scenario/trial/borrowed build identity, form, level/promotion/Eidolon/cone. |
| `Build / BuildTrace` | Exact preset-to-Trace selections. |
| `Build / BuildRelicPiece` | Concrete preset piece identity/set/slot/rarity/level/main affix. |
| `Build / BuildRelicSubAffix` | Piece sub-affix roll tiers or exact validated value. |

Child rows use composite keys and explicit order fields where order is semantic. Do not store a Trace patch list, relic list, or sub-affix list as JSON in one cell. Sora performs structural typing/references; `starclock-data` adds graph, patch, curve, reachable-roll, composition, and full-build validation.

## Catalog organization

`starclock-build` owns a separate immutable catalog:

```rust
pub struct BuildCatalog {
    revision: BuildCatalogRevision,
    digest: BuildCatalogDigest,
    characters: DefinitionTable<UnitDefinitionId, CharacterBuildDefinition>,
    character_stat_curves: DefinitionTable<CharacterStatCurveId, CharacterStatCurve>,
    ability_levels: DefinitionTable<AbilityLevelTableId, AbilityLevelTable>,
    trace_graphs: DefinitionTable<TraceGraphId, TraceGraphDefinition>,
    eidolon_sets: DefinitionTable<EidolonSetId, EidolonSetDefinition>,
    light_cones: DefinitionTable<LightConeId, LightConeDefinition>,
    light_cone_stat_curves: DefinitionTable<LightConeStatCurveId, LightConeStatCurve>,
    light_cone_superimposition: DefinitionTable<SuperimpositionDefinitionId, SuperimpositionDefinition>,
    relic_sets: DefinitionTable<RelicSetId, RelicSetDefinition>,
    relic_affixes: RelicAffixCatalog,
    loadout_policies: DefinitionTable<LoadoutValidationPolicyId, LoadoutValidationPolicy>,
    build_presets: DefinitionTable<BuildPresetId, CombatantBuildSpec>,
}
```

Fields remain private in implementation; the shape identifies ownership. Definitions reused directly by battle rules, such as units/abilities/rules/modifiers/programs, remain in `CombatCatalog` and are referenced by stable IDs. `LoadoutCompiler` receives both catalogs and validates their declared compatibility revisions/digests. `starclock-data` constructs and cross-validates both catalogs but neither catalog owns the other.

## `starclock-build` module layout

```text
src/
  lib.rs                 small build-domain facade
  catalog/
    definition.rs        immutable progression/equipment definitions
    builder.rs           validated BuildCatalog construction
    compatibility.rs     CombatCatalog/BuildCatalog revision contract
  spec/                  CombatantBuildSpec, presets, edit policies
  patch/                 typed Trace/Eidolon BuildPatch model
  trace/                 graph validation and canonical ordering
  light_cone/            curves, S1-S5 parameters, applicability
  relic/                 pieces, affixes, roll validation, set thresholds
  compile/
    pipeline.rs          ordered compilation orchestration
    ability.rs           cap/bonus/effective-level resolution
    contribution.rs      generic combat contribution construction
    source.rs            generic SourceBindings + detailed build report
  output.rs              CompiledBuild and BuildCompilationReport
  digest.rs              canonical build/report encoding and hashes
  error.rs               catalog/spec/compile errors
```

The crate may depend on public `starclock-combat` definition/spec types. It must not depend on battle state, resolver internals, `starclock-activity`, modes, Sora-generated rows, Excel readers, Bevy, filesystem state, or account services. Its `lib.rs` does not broadly re-export every progression/equipment type.

## Validation and coverage

Catalog validation requires:

- every released form has complete level/promotion, ability-level, battle Trace, and E1-E6 data;
- Trace graphs are acyclic and patches are valid in canonical application order;
- every Light Cone has complete level/promotion rows, S1-S5 parameters, applicability, rules, and provenance;
- every relic set threshold and affix curve/roll tier is complete for the supported snapshot;
- all definition and source IDs resolve and bilingual summaries/provenance meet content policy;
- build presets compile under their declared policy and store the expected digest;
- no enabled build contribution depends on workbook row/hash-map iteration or floating-point parsing.

Runtime/build validation requires:

- exact Trace prerequisite closure and form ownership;
- legal ability investment, Eidolon, levels, promotion, Light Cone range, and S1-S5 selector;
- legal relic slots, affixes, levels, rolls, uniqueness, and set counts;
- path/applicability rules are evaluated explicitly;
- patch conflicts and missing targets are errors, not ignored changes;
- rebuilding from canonical input produces the same compilation report, `ResolvedCombatantSpec`, build digest, and combatant-spec digest on every platform.

Coverage distinguishes definition completeness from individual build validity. Character, Light Cone, relic-set, and affix manifests must independently reach `DataReady`; one golden maxed build does not prove every node/rank/curve. Representative golden builds must cover E0/minimal Traces, E6/all Traces, matched and mismatched Light Cone applicability, every relic slot/main-affix family, sub-affix roll validation, set thresholds, trial presets, and synthetic-policy rejection by production profiles.

## Required tests

- Trace input order produces the same canonical patch order and build digest;
- missing prerequisites, cross-form nodes, cycles, invalid patch targets, and illegal ability levels are rejected;
- E0-E6 compilation applies patches exactly once and resolves effective levels/caps without silent clamping;
- all Light Cone level/promotion and S1-S5 rows compile, with base stats and passive applicability tested separately;
- relic main curves and sub-affix roll sums match golden values, including unreachable `ExactValidated` rejection;
- unique-slot, duplicate-affix, piece-count threshold, and planar/relic family rules are deterministic;
- compiled reports and generic resolved combatant specs retain linked source attribution through rule/modifier instantiation and battle events;
- named presets expand to the same exact build/digest as their normalized rows;
- activity lock/swap policies create new digests only at authorized boundaries;
- synthetic build values are accepted only under an explicit synthetic policy and cannot enter production coverage;
- canonical build encoding and compilation produce identical digests across supported platforms.
