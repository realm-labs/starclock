# Content Reference Schema

## Boundary

Reference records are normalized factual inputs. They are deliberately richer
than a compact Markdown profile and deliberately less coupled than an original
client table. They do not have to match future Sora sheet rows one-for-one.

All authoritative decimals are strings. Integer ordinals and counts remain
integers. Every record has a Starclock-owned `id`, source locators and a quality
label. Source IDs are never foreign keys in the future combat runtime.

## Quality labels

| Label | Meaning |
|---|---|
| `ExactStructured` | Directly transcribed from the pinned released 4.4 structured source. |
| `ExactPreviousRelease` | Directly transcribed from the pinned released 4.3 fallback and explicitly versioned. |
| `DerivedWithMissingReferences` | Starclock derived a relationship, but at least one source reference did not resolve. Not used by the current ordinary-encounter pack. |
| `ApproximateFromReleasedText` | Numeric/structural metadata is exact, while a missing configuration field is inferred conservatively from the released player-facing mechanic text. |
| `Observed` | Confirmed by a named battle observation fixture. Added during review, not by the bootstrap generator. |
| `ProjectPolicy` | Intentional Starclock behavior where original behavior is not publicly determinable. |

Approximation is field-level. An exact base stat does not make an inferred target
selector exact. Records therefore also carry `mechanism_quality` where source
configuration is incomplete.

## Shared evidence fields

| Field | Meaning |
|---|---|
| `id` | Descriptive, stable Starclock reference key. |
| `source_*_id` | One or more source-row locators used only for provenance. |
| `source_text.source_hash` | Released text-map key, when available. |
| `source_text.sha256` | Digest of the reviewed released mechanic text. The text itself is not emitted. |
| `source_ability_files` | IDs into `sources.json` for files containing the referenced ability entry. |
| `operation_types` | Sorted inventory of source operation type names reachable in that ability object. |
| `quality` | Overall numeric/relationship quality. |
| `mechanism_quality` | Exact configuration, previous-release text, or explicitly approximate released-text inference. |

`sources.json` maps every source-file ID to repository, relative path and SHA-256.
`manifest.json` pins repository revisions. `pack-index.json` hashes all generated
outputs in deterministic filename order.

## Character records

`characters.json` owns one record per combat form:

- bilingual display identity, path, element and rarity;
- maximum Energy or authored replacement resource cap;
- promotion segments with base/per-level HP, ATK and DEF plus SPD, CRIT and aggro;
- references to abilities, Traces and six Eidolons;
- original Starclock `behavior_summary_en` and `engine_contract_en`;
- every source avatar locator, including both Trailblazer body variants when
  they share one mechanics contract.

Male and female Trailblazer bodies are merged by Path. They do not create two
combat definitions. Alternate Paths and alternate character forms remain
separate records.

## Character ability records

`character-abilities.json` stores one family per semantic ability:

- Basic, Skill, Ultimate, Talent/passive, Technique and enhanced/replacement
  actions;
- target metadata and use type from the released character configuration;
- entry ability name and derived operation-type inventory;
- Energy gain, Skill Point cost, cooldown and delay fields;
- display damage, healing and Toughness components where exposed;
- exact parameter vectors for every published ability level;
- mechanic tags and a source-text digest.

`MazeNormal` represents the ordinary overworld initiation attack. `Maze`
represents the Technique. Goal 01 converts Technique results into pre-battle
handoff operations rather than executing exploration logic inside combat.

Hit timing and snapshot policy are not inferred from animation. Where the
released ability configuration does not prove a semantic hit boundary, the
future Excel row must choose an explicit project/observed policy.

## Trace and Eidolon records

`character-traces.json` stores Trace graph identity, prerequisites, unlock
anchor/type, level-up skill references, stat additions, ability/trigger names and
parameter vectors. Source graph IDs are provenance only.

`character-eidolons.json` stores exactly ranks 1 through 6 per combat form,
including skill-level additions, parameters and native source ability names.
Future build compilation converts these facts into ordered typed patches; battle
state never stores an `EidolonLevel` field.

## Light Cone records

`light-cones.json` stores:

- bilingual identity, Path restriction and rarity;
- promotion HP/ATK/DEF segments;
- passive identity and source ability name;
- complete ordered S1-S5 parameter vectors and property additions;
- mechanic tags and evidence digest.

The generated reference does not decide how a passive lowers into Rule IR. That
decision is reviewed when the Excel passive row is authored, using the source
text digest and the same trigger/selector/snapshot vocabulary as characters.

## Enemy template and variant records

`enemy-templates.json` owns mechanics shared by variants:

- bilingual name and rank;
- base HP, ATK, DEF, SPD, Toughness, CRIT damage and Effect RES;
- Toughness layer count/type and recovery ratio;
- exact character-config and AI-config source paths/hashes;
- authored AI skill sequence and normalized ability references.

`enemy-variants.json` owns encounter-specific differences:

- HP/ATK/DEF/SPD/Toughness multipliers;
- weaknesses, elemental resistances and debuff resistances;
- variant skill/summon lists and ability names;
- AI/sequence overrides and typed custom values.

Variants never mutate a template through display-name matching. Goal 01 selects
one exact variant and compiles the combined template/variant result.

## Enemy ability records

`enemy-abilities.json` stores:

- name, damage type, trigger and attack categories;
- target/use metadata and entry ability;
- damage-on-hit Energy, delay, AI cooldown/initial cooldown and phase list;
- parameter, modifier and status references;
- operation tags, operation-type inventory and evidence digest.

If a licensed/special source configuration is absent, target/operation tags may
be inferred from its released mechanic text and are marked
`ApproximateFromReleasedText`. The text hash makes the inference reviewable.
No missing configuration is silently labeled exact.

## Encounter candidates

`encounters.json` is not a complete story/activity script catalog. It contains
deduplicated released Mainline, Calyx and material-farm wave compositions that
resolve entirely to normalized enemy variants. Goal 01 selects a small
`standard-v1` manifest from these candidates to cover core mechanics.
