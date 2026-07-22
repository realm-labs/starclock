# Standard Universe Runtime Disposition Register

## Purpose

Goal 03 proves that the Standard Universe reference snapshot is complete and
reviewable. It does not prove that runtime code can execute it. Goal 04 therefore
assigns every frozen record, rule binding and semantic fixture to an exact
implementation batch and runtime mechanism before implementation begins.

The authoritative generated files are:

- `content-manifests/standard-universe-runtime-v1/runtime-dispositions.json`;
- `content-manifests/standard-universe-runtime-v1/partition-manifest.json`.

Regenerate them with
`node tools/goal04/generate-runtime-dispositions.mjs .` and verify with
`node tools/goal04/verify-runtime-dispositions.mjs .`. Editing generated
membership by hand is forbidden.

## Frozen denominator

The register assigns 2,201 content records, 786 mechanic-rule bindings and 78
semantic fixtures to fifteen partitions. Assignment does not mean implemented.
Rows begin as `Planned`; an exact Phase 4 partition changes to `Executable` only
after its runtime tests and semantic fixtures pass and the generated coverage
state is advanced through the audited policy tool. `G04-P4-M01` is the first
closed partition: 42 content records, 42 rules and 10 fixtures are executable.

| Disposition | Content | Rules | Meaning |
|---|---:|---:|---|
| `GenericActivityIr` | 698 | 359 | Generic graph, inventory, option, service or contribution program |
| `BattleRuleIr` | 0 | 0 | Reserved for rules directly lowered into ordinary battle Rule IR |
| `StaticNativeHandler` | 427 | 427 | One of two audited released-binding handlers returns normal typed operations/contributions |
| `DataOnlyMetadata` | 984 | 0 | Immutable topology, identity, pool, graph or encounter definition consumed by generic compilation |
| `ExplicitPolicy` | 92 | 0 | Encounter-pool selection uses its frozen named deterministic policy |

`StaticNativeHandler` is not permission for content-ID branching. The only
assigned handler families are
`universe.native.released-stage-ability-binding` and
`universe.native.released-curio-effect-binding`. Each must be a static registry
entry that consumes validated definitions and returns ordinary operations. A
partition may replace an assignment with equivalent typed IR only through a
reviewed manifest regeneration and fixture evidence.

## Frozen partitions

Counts are `(content / rules / fixtures)` and the JSON manifest lists every
stable ID, not merely these totals.

| Batch | Family | Frozen membership |
|---|---|---:|
| `G04-P4-M01` | Shared Activity operations and Ability Tree | 42 / 42 / 10 |
| `G04-P4-M02` | Preservation | 59 / 58 / 7 |
| `G04-P4-M03` | Remembrance | 59 / 58 / 5 |
| `G04-P4-M04` | Nihility | 59 / 58 / 2 |
| `G04-P4-M05` | Abundance | 59 / 58 / 3 |
| `G04-P4-M06` | The Hunt | 59 / 58 / 2 |
| `G04-P4-M07` | Destruction | 59 / 58 / 1 |
| `G04-P4-M08` | Elation | 59 / 58 / 2 |
| `G04-P4-M09` | Propagation | 59 / 58 / 3 |
| `G04-P4-M10` | Erudition | 59 / 58 / 2 |
| `G04-P4-M11` | Positive, neutral and special Curios | 86 / 86 / 12 |
| `G04-P4-M12` | Negative/Error, repair and replacement Curios | 42 / 42 / 7 |
| `G04-P4-M13` | Occurrences | 447 / 0 / 9 |
| `G04-P4-M14` | Services, roster and interactables | 94 / 94 / 9 |
| `G04-P4-M15` | Encounters, Worlds, difficulty and carry | 959 / 0 / 4 |

Each Path partition contains one Path, its four Resonance/Formation rows,
eighteen Blessings and thirty-six normal/enhanced Blessing levels. The Path row
has no separate rule binding, which explains 59 content versus 58 rules.

Curios with `negative`, `repair` or `replacement` tags and every `Repairing` or
`Fixed` state are assigned to M12. The parent Curio and all its states stay in
one partition. The remaining Curios and Active states are M11.

Structural maps, rooms, encounter groups, Worlds and difficulties are metadata;
encounter pools additionally bind an explicit deterministic selection policy.
They remain executable obligations in M15 even though they do not each require
a custom operation.

## Fixture ownership

A fixture belongs to the partition of every one of its input records. The
generator rejects cross-partition inputs. The 78 fixtures cover Ability Tree
operations, all nine Paths, eighteen Blessing mechanic tags, Curio states/tags,
occurrence outcomes, services and encounter selection/wave policies.

Phase 4 cannot close a partition by merely loading its rows. It must execute
each assigned fixture against lowered runtime domain values and supply an
end-to-end or semantic test for every assigned record/rule disposition. Global
`G04-P4-B1` reruns all 78; `G04-P4-B2` rejects any enabled record whose state is
still planned, missing or reachable only through scattered content-ID logic.

The M01 Ability Tree executor closes all 22 released targets and seven condition
forms into typed enums, executes all ten operation kinds with checked six-place
arithmetic, and retains the source node on every applied effect. Run and battle
values are projected at explicit lifecycle boundaries; node 17's initial
Cosmic Fragments are also materialized into authoritative Activity state.
Unknown targets, conditions, unit combinations or noncanonical selections fail
catalog/runtime validation.

## Change control

- Goal 03 IDs and evidence remain immutable inputs.
- A classification change edits the policy/generator, regenerates both complete
  manifests and updates evidence in one reviewed batch.
- No partition batch may move unrelated IDs to make its local count pass.
- Metadata-only means generic code consumes the definition; it never means the
  record may be ignored.
- An approximation or explicit policy retains its replacement condition until
  stronger public evidence is added through the reference-data process.
