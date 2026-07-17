# Excel and Sora Configuration Pipeline

## Decision

Use Excel `.xlsx` workbooks as the editable configuration tables and [realm-labs/sora](https://github.com/realm-labs/sora) as the only schema validation, Rust code generation, and runtime-data export tool.

The authoritative flow is:

```text
Sora schema modules     Excel .xlsx workbooks
        |                       |
        +-----------+-----------+
                    |
              sora check/build
                    |
       +------------+-------------+
       |                          |
generated Rust types/readers   validated bundles
                                  |-- config.sora     production/replay
                                  `-- debug JSON      review/tests only
```

Excel is the balance designer's editing surface. It is **not** a hidden or independent type system. Sora schema modules define field types, enums, unions, keys, references, indexes, defaults, validation rules, and workbook/sheet mappings. Excel headers must be generated or synchronized from those schemas.

The combat core never opens `.xlsx`, parses cells, or depends on Sora's CLI. `starclock-data` loads a validated exported bundle and converts generated records into immutable domain definitions. Bevy and other engine adapters only receive those definitions through the starclock-combat API.

## Pinned toolchain

Sora currently describes itself as early but runnable and recommends pinning the CLI because schema and code-generation semantics may change. Pin exactly one version across developer machines and CI.

Baseline for this document:

```text
sora-cli = 0.3.0
```

Install the pinned crate when a prebuilt binary is not provisioned:

```powershell
cargo install sora-cli --version 0.3.0 --locked
sora --version
```

An upgrade is a deliberate migration. Regenerate and review the schema lock, Excel templates, Rust code, debug JSON, and production bundle together.

## Proposed repository layout

```text
config/
  project.toml
  schema/
    common.toml
    character.toml
    ability.toml
    effect.toml
    equipment.toml
    enemy.toml
    encounter.toml
    rule.toml
    activity.toml
    universe.toml
    challenge.toml
  data/
    Character.xlsx
    Ability.xlsx
    Effect.xlsx
    Equipment.xlsx
    Enemy.xlsx
    Encounter.xlsx
    Rule.xlsx
    Activity.xlsx
    Universe.xlsx
    Challenge.xlsx
  generated/
    schema.lock
    templates/          # generated .xlsx templates; not runtime input
    debug-json/         # deterministic review artifacts
    config.sora         # production runtime bundle
crates/
  starclock-data/
    src/
      generated/        # Sora-generated Rust models/readers
      compile.rs        # generated row -> validated combat/activity definitions
      validate.rs       # cross-table/battle/activity checks Sora cannot express
```

Keep `config/data` and `config/generated/templates` separate. Template generation must never overwrite live balance data.

## Project manifest

Start from a Sora manifest equivalent to:

```toml
package = "starclock_combat_config"
includes = [
  "schema/common.toml",
  "schema/character.toml",
  "schema/ability.toml",
  "schema/effect.toml",
  "schema/equipment.toml",
  "schema/enemy.toml",
  "schema/encounter.toml",
  "schema/rule.toml",
  "schema/activity.toml",
  "schema/universe.toml",
  "schema/challenge.toml",
]

[build]
default_source_format = "xlsx"
data_root = "data"
schema_lock = "generated/schema.lock"
excel_templates = "generated/templates"

[[build.codegen]]
target = "rust"
out = "../crates/starclock-data/src/generated"
format = "required"

[[build.exports]]
format = "binary"
out = "generated/config.sora"

[[build.exports]]
format = "json-debug"
out = "generated/debug-json"
```

Paths are resolved relative to `project.toml`. Confirm paths against the pinned CLI before committing the initial manifest.

## Workbook boundaries

Prefer several focused workbooks with related sheets over one giant workbook. The first data revision should contain these tables:

| Workbook / table | Key fields | Purpose |
|---|---|---|
| `Character.xlsx / Character` | `id`, `element`, `path`, base resources, stat-curve reference, ability references | Stable combat-form identity and loadout. |
| `Character.xlsx / CharacterStat` | character/level composite key, HP, ATK, DEF, SPD | Authored level data or curve samples. |
| `Ability.xlsx / Ability` | `id`, kind, target program, cost, energy gain, phase references, tags | Ability entry point and ordinary metadata. |
| `Ability.xlsx / AbilityPhase` | ability reference, order, operation references | Ordered resolution phases without cell-local scripts. |
| `Ability.xlsx / AbilityOperation` | `id`, operation union, selector, timing, condition | Damage, heal, effect, resource, Toughness, action, or queued-action operation. |
| `Ability.xlsx / HitPlan` | `id`, hit ratios, Toughness ratios, retarget policy | Reusable deterministic multi-hit definition. |
| `Effect.xlsx / Effect` | `id`, category, duration, stacking, dispel class, modifiers | Buff, debuff, DoT, shield, mark, field, or state definition. |
| `Effect.xlsx / Trigger` | `id`, owner effect, event, limit scope, reaction reference | Passive and reactive behavior. |
| `Equipment.xlsx / LightCone` | `id`, path, rarity, stat curve, superimposition references | Light Cone identity, stats, and S1-S5 rule patches. |
| `Equipment.xlsx / RelicSet` | `id`, slot family, piece thresholds, rule references | Relic and planar-set mechanical effects. |
| `Equipment.xlsx / RelicAffix` | `id`, slot legality, level/tier values, stat and roll tables | Main/sub-affix curves without inventory or gacha systems. |
| `Enemy.xlsx / Enemy` | `id`, stats, weaknesses, resistances, Toughness, AI reference | Enemy combat definition. |
| `Enemy.xlsx / EnemyAbility` | `id`, enemy reference, phase, target program, ability program | Enemy and boss action definitions. |
| `Enemy.xlsx / AiState` | `id`, graph reference, entry actions, transitions, priorities | Deterministic authored enemy behavior. |
| `Encounter.xlsx / Encounter` | `id`, team rules, wave references, mode overrides | Battle assembly without presentation data. |
| `Encounter.xlsx / Wave` | encounter reference, order, unit slots | Ordered enemy waves and spawn configuration. |
| `Rule.xlsx / RuleDefinition` | `id`, Battle/Activity domain, source kind/reference, slots, triggers | Reusable character, equipment, enemy, activity, and mode behavior. |
| `Rule.xlsx / Program` | `id`, ordered typed operations | Closed operation programs interpreted by the core. |
| `Activity.xlsx / ActivityDefinition` | `id`, profile, entry node, participant/visit policies | Root deterministic cross-battle workflow. |
| `Activity.xlsx / ActivityNode` | `id`, activity, node kind, section, program/battle references | Battle, choice, reward, shop, roster, external outcome, checkpoint, fork/join, or terminal node. |
| `Activity.xlsx / ActivityEdge` | source/target, condition, priority, once/visit policy | Validated branching and bounded loops. |
| `Activity.xlsx / ActivityProgram` | `id`, owner domain, ordered typed operations | Graph, slot, participant, inventory, clock, metric, objective, and BattleSpec operations. |
| `Activity.xlsx / ActivitySlot` | typed ID, scope, default, bounds, carry/reset | Activity/section/node/attempt state without untyped maps. |
| `Activity.xlsx / ParticipantPolicy` | pools, team slots, eligibility, uniqueness, loadout locks | Trial, borrowed, drafted, banned, fixed, and mutable roster rules. |
| `Activity.xlsx / BattleBinding` | node, encounter/spawn, team, rules, clock/metric projections | Immutable `BattleSpec` input and declared `BattleResult` output. |
| `Activity.xlsx / ActivityClock` | scope, domain, initialization, observation, expiry | Shared cycle/AV/action/turn/wave counters. |
| `Activity.xlsx / MetricObjective` | typed source, aggregation, cap, predicate | Scores and completion/bonus objectives. |
| `Activity.xlsx / SpawnProgram` | pools/groups, ordering/RNG, capacity, refill, escalation, termination | Finite waves, continuous refill, survival, and endless-under-budget spawns. |
| `Universe.xlsx / ModeContent` | `id`, mode family, content kind, activity/rule/program references | Blessings, curios, equations, scepters, components, dice, and other universe profile mechanics. |
| `Challenge.xlsx / ChallengeSeason` | `id`, family, game version, global rules, source references | Active Memory of Chaos, Pure Fiction, and Apocalyptic Shadow snapshot. |
| `Challenge.xlsx / ChallengeStage` | season, order/difficulty, activity section/node references | User-facing stage identity over generic activity nodes. |
| `Challenge.xlsx / ChallengeNodeProfile` | stage, team alias, selectable buff pool, activity bindings | Challenge-specific metadata without duplicate graph/clock/score types. |
| `Challenge.xlsx / ChallengeBuffOption` | season/node applicability, rule/program references | Memory Turbulence, Cacophony, Ruinous Embers, and Finality's Axiom content. |
| `common / SourceRecord` | `id`, URL, access date, version, confidence, evidence digest | Row-level provenance for imported facts. |
| `common / ConfigManifest` | singleton revision row | Game version, data revision, rules compatibility, and notes. |

Use Sora `ref<Table.field>` for cross-table references and secondary unique indexes for natural keys that must remain unique. Prefer child tables and typed structs/unions when an Excel cell would otherwise contain a large JSON or custom mini-language.

## Operation representation

Most abilities should compile from a closed operation union such as:

```text
Damage
ReduceToughness
Heal
ApplyEffect
RemoveEffect
ModifyResource
ModifySkillPoints
AdvanceAction
DelayAction
QueueAction
Summon
Despawn
SetField
```

The schema carries operation-specific typed payloads. A row refers to selectors, conditions, value expressions, and timing points; it does not contain Rust, Lua, or arbitrary formulas.

Exceptional kits may reference a stable `native_handler` key. `starclock-data` must validate the key against a Rust registry during bundle loading. Native handlers emit the same operations/events as table-authored abilities and may not bypass the resolver. Use a native handler only when the reusable operation/trigger model cannot express the behavior.

The exact rule tables and validation boundary are defined in [Rule IR and native handlers](11-rule-ir-and-native-handlers.md). Characters, enemies, equipment, and battle-visible mode effects compile into the battle IR. Cross-battle graph/roster/resource/score behavior compiles into the activity IR in [Activity core and mode extension](19-activity-core-and-mode-extension.md). Do not create a separate operation language per mode.

## Deterministic workbook bootstrap

Bulk transcription into the first Version 4.4 workbook set should be repeatable rather than a manual copy exercise. A future repository tool pins `rust_xlsxwriter = "=0.96.0"` and regenerates complete bootstrap workbooks from normalized, reviewed staging records. It does not edit or merge live designer workbooks.

```text
ignored source cache -> normalized bilingual staging -> validated rows
                                                        |
                                                        v
                                               complete bootstrap .xlsx
                                                        |
                                                        v
                                                   sora check/build
```

The bootstrap tool must sort sheets and rows by stable keys, write canonical decimal strings, include provenance references, and produce identical cell values for identical staging input. Workbook ZIP metadata is not a replay input; after Sora export, deterministic debug JSON and the `config.sora` digest are the review/replay identities.

Sora-generated templates remain the header/schema authority. Bootstrap generation starts from the pinned schema projection or reproduces it through Sora-supported workflows; it must not maintain a competing hand-written column model. After initial import, a designer-edited workbook is never overwritten automatically.

## Excel authoring policy

- Generate new workbooks with `sora excel-template`; do not hand-create schema header rows.
- After schema changes, preview `sora excel-sync` before using `--write`, then review the workbook changes.
- Use stable, nonlocalized IDs in keys and references. Display names and descriptions are optional metadata, never references.
- Store designer-facing ratios as canonical decimal strings (`"0.25"`) or explicitly scaled integers, never as authoritative floating-point cells. `starclock-data` parses them according to [Cross-platform determinism and numeric policy](09-determinism-and-numerics.md).
- Store raw Toughness in the project's chosen integer unit and durations with an explicit clock/phase enum.
- Avoid merged cells, macros, hidden business rules, and spreadsheet formulas in runtime fields. Derived runtime values belong in schema-backed source columns or deterministic compilation code.
- Do not encode a large action program as JSON in one cell. Split it into operation/phase child rows.
- Keep one header/type/rule projection generated by Sora; do not maintain a second hand-written header convention.
- Store `name_en`, `name_zh_cn`, `summary_en`, and `summary_zh_cn` as original project metadata; stable nonlocalized IDs remain the only references.
- Every mechanically meaningful row references at least one `SourceRecord`, or is explicitly labeled as a project policy/test fixture.
- Commit `.xlsx`, schema, schema lock, generated Rust, and deterministic debug JSON according to the repository's generated-file policy. Never manually edit generated Rust or bundles.

## Commands

Normal full build:

```powershell
sora build --project config/project.toml
```

Validation without intentionally modifying source workbooks:

```powershell
sora check --project config/project.toml
```

Create templates and synchronize an existing workbook after a schema change:

```powershell
sora excel-template --project config/project.toml --out config/generated/templates
sora excel-sync --project config/project.toml --data-root config/data
sora excel-sync --project config/project.toml --data-root config/data --write
```

The manifest-driven `sora build` is the canonical command. One-off `gen` and `export` commands are diagnostic tools, not separate production pipelines.

## Load boundary

At application startup:

1. `starclock-data` opens `config.sora` through the Sora-generated Rust reader.
2. It checks the bundle format supported by the generated reader.
3. It checks `rules_revision` against the starclock-combat and starclock-activity compatibility ranges.
4. It resolves generated table references into stable runtime IDs.
5. It parses canonical decimal strings into checked domain fixed-point values and performs domain validation spanning battle operations/events and activity graphs/projections.
6. It constructs an immutable `Arc<SimulationCatalog>` containing combat definitions, activity definitions, and mode-profile indexes.
7. A battle or activity captures the catalog revision and cannot switch data while running.

Do not make core APIs expose generated Sora row types. Convert them at the `starclock-data -> starclock-combat/starclock-activity` boundaries so Sora upgrades cannot leak into rules, profiles, or engine adapters.

## Revision and replay policy

The workbook's `ConfigManifest` should include:

```text
game_version
snapshot_date
data_revision
required_rules_revision
sora_cli_version
numeric_policy_revision
rng_algorithm_revision
state_hash_revision
replay_format_version
coverage_manifest_sha256
```

After export, compute a SHA-256 digest of `config.sora`. Store this digest in the replay header alongside the human-readable data revision. Replay loading rejects a different digest unless an explicit migration supplies the original bundle.

A config change never alters an already running battle or activity. Development hot reload may replace the catalog used for newly created activities/battles only.

## Validation gates

Sora should reject structural problems such as invalid types, missing required fields, duplicate keys/indexes, and unresolved references. `starclock-data` adds domain checks including:

- hit and Toughness ratios have legal sums and nonnegative components;
- all event names, tags, selectors, timing phases, and native handlers are registered;
- trigger graphs obey recursion/event-budget policy;
- every enhanced/replaced ability has a reachable exit transition;
- resources, stacks, shields, and counters have valid caps and consumption rules;
- fields, summons, and marks declare teardown/retarget behavior;
- every released character in the implementation matrix resolves to a complete definition;
- Announced characters are marked disabled until their required fields are complete;
- every enabled public content row has the required bilingual metadata and provenance;
- every entry in the frozen Version 4.4 coverage manifest resolves to exactly one terminal coverage state;
- the manifest digest, config digest, and declared Version 4.4 snapshot agree;
- every challenge clock declares ownership, decrement, boundary reset/carry, expiry, and consuming actor/action kinds;
- every score program is finite, typed, capped/rounded explicitly, and references events or state available in its node result;
- two-node challenge stages have valid disjoint roster policy, encounter/spawn programs, buff pools, objectives, and aggregation;
- every activity graph has one entry, reachable terminal outcomes, valid typed slots, bounded visits/loops, and legal edge conditions;
- participant pools, trial/borrow/draft/ban rules, uniqueness scopes, loadout locks, and carry projections are internally consistent;
- every BattleBinding declares its accepted result metrics/state; a battle cannot return arbitrary activity mutations;
- the same bundle plus seed and command stream reproduces the same battle/activity events and hashes.

## CI gate

CI must use the pinned Sora version and run, in order:

1. verify the exact `sora --version`;
2. `sora check --project config/project.toml`;
3. `sora build --project config/project.toml`;
4. fail if committed generated code, schema lock, or debug JSON differs;
5. run Rust formatting, tests, golden battle/activity replays, profile fixtures, and bundle-load tests;
6. publish `config.sora` with its SHA-256 digest as a versioned artifact when releasing.

This keeps Excel convenient for authoring while preserving deterministic, typed, engine-independent runtime behavior.
