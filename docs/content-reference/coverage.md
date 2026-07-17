# Content Reference Coverage

## Baseline status

The Version 4.4 reference pack is prepared as a precondition for Goal 01. It is
not a claim that the future Sora workbooks or Rust runtime are implemented.

| Category | Records | Numeric baseline | Mechanic baseline |
|---|---:|---|---|
| Released combat forms | 88 | Complete promotion/resource facts; 86 from released 4.4 data, Saber/Archer from pinned released 4.3 data | Every form has an original Starclock behavior summary and engine contract. |
| Character abilities | 583 | Published level parameter vectors and exposed cost/cooldown/display values | Target/entry/operation evidence where present; released-text inference is explicit where source config is absent. |
| Traces | 1,618 | Published parameters, stat additions and skill-level references | Ability/trigger source names and text digests retained for patch review. |
| Eidolons | 528 | Exactly E1-E6 for all 88 forms | Parameters, skill-level patches, source ability names and evidence digests retained. |
| Light Cones | 165 | Complete promotion segments and ordered S1-S5 parameters | Passive names, source ability names, mechanic tags and evidence digests retained. |
| Enemy templates | 613 | Complete base-stat/Toughness/AI-sequence baseline | Character/AI config locators and normalized ability references retained. |
| Enemy variants | 2,591 | Complete stat multipliers, weaknesses, RES and debuff-resistance rows | Skill, summon, custom-value and AI override relationships retained. |
| Enemy abilities | 3,611 | Complete exposed parameter/cooldown/phase baseline | 3,254 have direct character-config target evidence; 357 special/missing configs are explicitly inferred from released text and remain reviewable by text hash. |
| Ordinary encounter candidates | 1,471 | Complete deduplicated Mainline/Calyx/Farm wave compositions | All enemy variant references resolve. Goal 01 selects the smaller `standard-v1` coverage set. |

Machine-readable counts are authoritative in
`content-reference/v4.4/coverage.json`.

## Accepted approximation boundary

The user-approved baseline permits complete or approximate numeric values while
requiring correct mechanic categories. Accordingly:

- exact released numbers are preserved as canonical decimal strings;
- previous-release collaboration facts are labeled, not silently promoted to
  4.4 origin;
- missing target/config metadata is inferred only from the corresponding
  released mechanic text and labeled `ApproximateFromReleasedText`;
- source text hashes and file hashes make every inference reproducible;
- hidden operation order, snapshotting and retarget edge cases remain named
  observation work during implementation rather than invented facts.

Approximation does not permit a different trigger cardinality, source owner,
target category, lifecycle category or AI policy when the released evidence
already specifies one.

## Known evidence gaps

- Some collaboration/special enemy character configuration files are absent from
  the pinned released dump. Their numerical skill rows remain available; target
  and operation tags use released-text inference.
- A small number of source ability entries live in shared or unavailable files.
  Operation-type inventories may therefore be empty even when the player-facing
  mechanic and numeric row are present.
- Published data does not universally expose semantic hit boundaries, snapshot
  timing, equal-priority reaction order or target invalidation behavior.
- Stage records are encounter candidates, not complete story scripting or
  out-of-scope mode definitions.

These gaps do not authorize arbitrary behavior. The authoring contract requires
an observed fixture or explicit project policy before the affected Excel row is
marked `DataReady`.

## Readiness gate for Goal 01

Goal 01 may start only when:

- `pack-index.json` matches regenerated output;
- the character count is 88 and every form has six Eidolons;
- all Light Cones contain five ordered Superimposition rows;
- every enemy variant resolves to one template and every ordinary encounter
  candidate resolves to enemy variants;
- no record contains an unlabeled approximation;
- the execution prompt binds the pack digest and mandates the authoring contract.

This gate means the implementation begins from prepared facts. It does not mean
all content is already `DataReady`; `DataReady` additionally requires lowering
into reviewed Rule IR/Excel rows and passing behavior fixtures.
