# Goal 16 Normalization and Authoring Contract Audit

`G16-P0-B4` freezes the machine contracts that all later data, Excel, Sora and
semantic-review batches must obey. The contracts bind the immutable P0-B3
manifest digest
`92bf516ebb2c0baec8df4bbc5ccd435d090181fb4553db0261a4ce49a5b032a4`.

## Normalized reference contract

The normalized schema assigns all 2,232 manifest obligations to 40 typed JSON
file families. These files are research staging and deterministic debug
surfaces only; they are not runtime inputs.

Every normalized row requires:

- a globally unique project-owned stable ID and explicit profile set;
- independent short English and Simplified Chinese mechanical summaries;
- ownership, coverage, evidence and mechanism-quality labels;
- one or more exact manifest record IDs;
- ordered fact-level source references; and
- stable tags.

Exact decimals and upstream 64-bit IDs are strings. Sets sort by stable ID;
stage, candidate, recipe, operation, wave, slot and settlement sequences retain
their declared order. Optional absence is omission, never a second `null`
representation.

## Excel and Sora authority

Four isolated workbooks own the complete authoring surface:

| Workbook | Responsibility |
|---|---|
| `GalacticBaseballerProfiles.xlsx` | profiles, releases, stages, growth, inventory, strategies, progression, currencies and store |
| `GalacticBaseballerArsenal.xlsx` | weapons, accessories, trigger bindings and all synthesis tiers |
| `GalacticBaseballerEncounters.xlsx` | encounters, waves, enemies, skills, statuses, scores and settlement |
| `GalacticBaseballerReview.xlsx` | rules, sources, policies, reconciliation, coverage, fixtures and pack identity |

The 40 normalized files map to these workbooks exactly once. Python
`openpyxl==3.1.5` is the only workbook writer; Sora 0.3.0 is the schema,
code-generation and production-export authority. Generation must create a
complete clean target, reject formulas/errors/unknown columns/references and
prove byte-identical double output.

Every sheet and every schema field column must be rendered and visually
inspected. The isolated reader must later load every generated table and every
row.

## Semantic fixture contract

The 20 non-shrinking mechanism families from P0-B3 are reconciled exactly to
the fixture contract. Each requires at least one ReferenceOnly rule and one
review fixture containing:

- explicit trigger point and state owner;
- typed preconditions and a concrete input;
- nonempty ordered operations;
- typed expected facts;
- source-record and evidence references; and
- evidence and mechanism-quality labels.

Random fixtures use labeled project RNG with integer sampling and stable
candidate IDs. Rejections assert byte-identical authoritative state. A
no-legal-candidate result and simultaneous synthesis/ordinary-upgrade order
must be explicit rather than implied.

## Initial policy boundaries

Eight missing-observation boundaries are registered as `ProjectPolicy`:

- candidate draw weights;
- candidate display order;
- no-legal-candidate behavior;
- simultaneous synthesis versus ordinary upgrade;
- same-boundary weapon trigger ordering;
- target tie-breaking;
- refresh exclusion memory/fallback; and
- intermediate score rounding.

Every record states the unavailable fact, released facts preserved, selected
deterministic policy, at least two rejected alternatives, rationale, affected
fixtures, confidence and a concrete released-evidence replacement condition.
These records are not parity claims. Later research may add policy records but
cannot silently remove or relabel these boundaries as exact.

## Reproduction

```text
node tools/galactic-baseballer-reference/contracts.mjs
node tools/galactic-baseballer-reference/contracts.mjs --check
node tools/galactic-baseballer-reference/verify-contracts.mjs
```

The generator produces:

| Artifact | SHA-256 |
|---|---|
| `normalized-schema.json` | `0b7153f3e146363658a4ad038b68b835c2fc33b232ebca945733a10292f144c3` |
| `authoring-contract.json` | `aaf92c08afa91198ea4a31b1258a54193adf214059cecd1eee287c39f79fe9e5` |
| `fixture-contract.json` | `86e4da97dbfc3ea4fa0c8e59b9ed4c6a4bdd7819e481eaeaa8cff5bcaf97e538` |
| `approximation-register.json` | `b718341d25812adc4f8bfe82d0fc0e24382b7157c80da9f91cded30f6e3f6a2a` |
