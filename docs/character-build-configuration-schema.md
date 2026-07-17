# Character and build configuration schema

Goal 01 batch `G01-P1-B7` freezes the Sora 0.3.0 authoring contracts for
characters, abilities, ordered hit plans, Traces, Eidolons and Light Cones.
These schemas describe identity, progression and build-time composition. They
do not define executable effects: typed selectors, operations, modifiers and
rules belong to `G01-P1-B8`.

## Modules and ownership

- `config/schema/ability.toml` owns ability metadata, resource deltas, effective
  level parameters, ordered phases and ordered hit plans.
- `config/schema/build-patch.toml` owns the closed patch union used by both
  Trace and Eidolon compilation.
- `config/schema/character.toml` owns character base-stat rows, character-local
  resources, ability slots, Trace graphs and E1-E6 definitions.
- `config/schema/equipment.toml` owns Light Cone stat rows, passive
  applicability and S1-S5 parameter values.

Every domain definition ID is a typed reference to `ContentIdentity.id`.
Relations between definitions use Sora references rather than unvalidated
integer foreign keys. Ordered children use explicit positive `sequence` fields
with unique parent/sequence indexes. Decimal facts remain canonical strings and
are converted to checked six-place fixed-point values only at the domain
conversion boundary.

## Ability and hit-plan boundary

An `Ability` declares its kind, target pattern, invalid-target policy, level cap
and optional entry-rule identity. Ordered `AbilityPhase` rows separate entry,
pre-hit, hit, post-hit and resolved boundaries. A phase may point at a program
identity that will be resolved against the typed Rule IR tables introduced in
`G01-P1-B8`.

`HitPlan` and `HitPlanHit` keep hit count, order, target group, damage share,
toughness share and CRIT policy explicit. `AbilityHitPlanBinding` attaches a hit
plan to one ability phase. The golden verifier proves stable ordering and exact
millionth-unit sums for representative damage and Toughness ratios. Catalog
validation must additionally reject a declared count mismatch, a missing phase
or an invalid sum for production rows.

Resource changes identify their resource class, direction and lifecycle timing.
Character-local resources additionally require a stable key. Level-scaled facts
are rows keyed by ability, effective level and parameter key; they are not
floating-point values or partial serialized definitions.

## Characters, Traces and Eidolons

Character progression rows explicitly carry level and promotion with base HP,
ATK, DEF and SPD. Ability bindings distinguish Basic, Skill, Ultimate, Talent,
Technique, enhanced, summon, memosprite and passive slots. A Technique is data,
not an assumption that the Technique was used before the battle; scenario entry
selection remains separate.

Trace nodes are typed self-referential graph rows. Root nodes carry an empty
prerequisite list, while dependent nodes list exact `TraceNode.id` references.
Sora proves reference integrity; build-catalog validation remains responsible
for graph acyclicity, character ownership and canonical topological order.

Released ordinary character forms require exactly six Eidolon definitions with
ranks 1 through 6. Selecting En applies E1 through En in rank order, and each
rank's child patches apply by sequence. The representative verifier rejects
incomplete E1-E6 rank sets and Sora rejects duplicate character/rank rows.

## Closed build-patch language

The `BuildPatch` tagged union has exactly these reviewed variants:

- `AddRule`, `RemoveRule`, `AddModifier`, `AddAbility`, `ReplaceAbility`;
- `PatchAbility`, with an exact ability, phase and replacement program identity;
- `AdjustAbilityLevel`, `AdjustResourceDefinition`, `AdjustStateSlot`;
- `AddTag`.

Patch payloads contain typed references or bounded scalar fields. They contain
no Rust, field-path string, arbitrary JSON or partial serialized definition.
Rule, modifier and program identities are intentionally `ContentIdentity`
references in this batch; `G01-P1-B8` closes them against the corresponding
typed tables without changing the build-patch architecture.

## Light Cones

A Light Cone declares rarity, Path, passive-rule identity and an explicit
applicability policy. Base HP, ATK and DEF are level/promotion rows. Passive
parameters use one row per parameter and superimposition rank. Each scalable
parameter requires S1-S5. A parameter that is explicitly constant across ranks
uses one S1 row with `constant_across_ranks = true`; scalable and constant rows
cannot be mixed for one parameter. A Light Cone is never represented as five
separate item definitions. The build compiler applies base stats independently
from passive eligibility.

## Golden and production boundaries

`config/schema-fixtures/character-build` is disabled synthetic evidence. The
verifier executes Sora check/build, schema lock, Excel-template generation, Rust
codegen, binary and diagnostic exports, compares direct and configured outputs,
rebuilds for drift, checks reference failures and commits only a manifest of the
73 generated byte hashes. Its TOML files are not production content and never
count toward Goal 01 coverage.

Production content remains `.xlsx`-authored. `G01-P1-B10` creates those
workbooks and generated readers, and `G01-P1-B11` converts the generated rows
into validated domain definitions. No runtime JSON path is authorized by this
schema golden.
