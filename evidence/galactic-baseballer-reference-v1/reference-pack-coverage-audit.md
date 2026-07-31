# Goal 16 Reference Pack and Coverage Audit

`G16-P3-B1` assembles both released profiles into the 40-file normalized
contract and closes every frozen source obligation without reducing the
denominator.

## Final frozen denominator

| State | Rows |
|---|---:|
| DataReady | 2,207 |
| EvidenceOnly | 25 |
| **All frozen obligations** | **2,232** |

`coverage.json` and `reconciliation.json` each contain exactly 2,232 rows.
Every source obligation appears exactly once with its category, stable record
ID, Version 4.4 source path, row locator, evidence SHA-256, evidence quality,
runtime disposition and closure owner.

Every DataReady row points to at least one real normalized record. No
DataReady row is closed by a bare source receipt. The 25 EvidenceOnly rows
have no normalized mechanical owner and remain explicit reward/presentation
exclusions.

All reconciliation rows assert `exact_once=true` and
`inferred_from_name_or_id_range=false`.

## Closure correction: Departure persistent progression

The first assembly audit correctly rejected 154 source-only mechanical
receipts. That exposed previously unnormalized released Departure data rather
than a denominator problem. `G16-P3-B1` therefore added:

| Departure family | Closed rows |
|---|---:|
| Raccoon Gold currency | 1 |
| Store definitions | 14 |
| Store price levels | 54 |
| MazeBuff-linked store levels | 48 |
| Tutorial locators | 20 |
| Store tag definitions | 4 |
| Team-bonus definitions | 8 |

Departure Raccoon Gold uses exact item ID `281019`. The exact enemy income
vector is `5/5/20/200/0`; the chest income vector is `400/1500/2500`; base
and alternate chest-gold vectors are both `50/250/400`; probability and step
vectors are `0.4/0.3/0.3` and `-0.1/0/0.1`. The structured family does not
publish a single maximum-balance value, so `maximum_balance` is explicitly
null with disposition `UnspecifiedInFrozenStructuredFamily`; the
`100/250/500` source vector is retained separately as
`gold_max_level_vector`.

The 14 store definitions expand to 54 exact price levels and cost `68200`
Raccoon Gold in total:

- 12 AddMazeBuff definitions / 48 levels;
- one initial-weapon-level definition / five levels;
- one accessory-slot definition / one level.

Six of the eight released team-bonus definitions have exact authored Departure
stage bindings. The other two remain explicit
`ReleasedDefinitionWithoutAuthoredStageBinding` records; no relationship is
guessed from IDs or binding names.

The shared store-transaction atomicity boundary now names both profiles. It
still validates current level and balance before one atomic commit and remains
a replaceable ProjectPolicy rather than an observed-parity claim.

## Normalized pack

The complete contract contains:

| Family | Rows |
|---|---:|
| Profiles / release boundaries | 2 / 5 |
| Stages / periods | 13 / 113 |
| Weapons / weapon levels / triggers | 55 / 251 / 55 |
| Accessories / levels / bindings | 32 / 128 / 32 |
| Synthesis recipes / inputs | 27 / 54 |
| Candidate pools / policies | 15 / 4 |
| Inventory slots / operations | 8 / 10 |
| Encounters / waves / enemy candidates | 23 / 78 / 1,777 |
| Enemy identities / skills / statuses | 104 / 339 / 10 |
| Score / settlement | 2 / 13 |
| Adventure Strategies | 56 |
| Progression / currencies / store levels / unlocks | 54 / 3 / 114 / 50 |
| Mechanic rules / review fixtures | 26 / 35 |
| Source receipts | 2,634 |
| Approximation boundaries / research gaps | 12 / 12 |

`pack-index.json` lists the other 39 normalized files in the frozen schema
order with exact row counts and canonical SHA-256 digests. Its own row states
the 40-file denominator without creating a recursive self-digest.

## Sources and replacement boundaries

`sources.json` aggregates 2,634 stable source IDs. Each retains:

- repository or URL;
- exact repository revision or access/revision date;
- game version;
- path/page and every row locator;
- SHA-256 values;
- evidence/mechanism quality;
- notes, replacement conditions and normalized consumers.

All source references in all normalized content resolve to one of these
receipts.

The pack contains 12 non-blocking replaceable boundaries: eight foundational
ProjectPolicy records plus the four progression boundaries closed in P2-B3.
Eleven are ProjectPolicy and the released Cosmic Reputation costs remain
visibly `ApproximateFromReleasedText`. Every row has the unavailable fact,
selected policy, at least two rejected alternatives, rationale, affected
fixtures, confidence and replacement condition.

## Semantic coverage

The frozen denominator remains 20 mechanism families. The combined profiles
provide 26 ReferenceOnly rules and 35 concrete review fixtures. Every family
has at least one rule and one fixture, and every rule/fixture records:

- trigger point and state owner;
- preconditions;
- ordered operations;
- concrete input and expected facts;
- evidence links;
- `runtime_executable=false`.

The corrected RuinBot fixtures use the canonical family
`weapon-automatic-action`; no uncontracted alias remains.

## Reproduction

```text
node tools/galactic-baseballer-reference/normalize-departure-progression.mjs \
  --source-cache .cache/galactic-baseballer-source
node tools/galactic-baseballer-reference/normalize-departure-progression.mjs \
  --check --source-cache .cache/galactic-baseballer-source
node tools/galactic-baseballer-reference/assemble-reference-pack.mjs
node tools/galactic-baseballer-reference/assemble-reference-pack.mjs --check
node tools/galactic-baseballer-reference/verify-reference-pack.mjs \
  --source-cache .cache/galactic-baseballer-source
```

The output remains ReferenceOnly/Candidate data. P3-B2 will author the four
complete isolated Excel workbooks from these normalized tables.
