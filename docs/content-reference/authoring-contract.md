# Content Transcription and Authoring Contract

## Mandatory promotion path

No released content row may be authored directly from memory or a single web
page. Use this path:

1. locate the normalized reference record;
2. verify its pack digest and source-file hashes;
3. read the project mechanic contract and the pinned released source evidence;
4. split the mechanic into typed selectors, conditions, expressions, operations,
   triggers, state slots and modifiers;
5. declare every unresolved timing/snapshot/retarget field as `Observed`,
   `Approximate` or `ProjectPolicy` with a reason;
6. add the review fixture before promoting the row to Excel/Sora `DataReady`.

The reference pack is not copied wholesale into one spreadsheet cell. Parameter
vectors become typed level/rank rows; ability phases and operations become child
rows; evidence references remain attached at fact or operation granularity.
The first semantic reviews use the
[content reference review fixtures](review-fixtures.md).

## Character dossier gate

Before a character form can become `DataReady`, its review dossier must account
for:

- all base/promotion statistics and resource caps;
- Basic, Skill, Ultimate, Talent, Technique and every enhanced/replacement action;
- exact level parameters, Energy/SP/HP costs and gains;
- target shape, random ordering, hit/Toughness split and retarget policy;
- effect probability, duration clock, stacking, dispel and teardown;
- trigger event, phase, priority, once-scope and cause/source ownership;
- modifier stage, stacking group, cap and snapshot policy;
- every battle-relevant Trace and its ordered patch;
- E1 through E6 and patch conflict/replacement behavior;
- summon/memosprite/countdown/transformation/presence lifecycle;
- E0 and E6 representative fixtures;
- all approximations and unresolved observations.

Existing character profiles provide the loop and engine contract. They do not
waive any dossier field.

## Enemy dossier gate

Before an enemy template/variant can become `DataReady`, its review dossier must
account for:

- template stats and level/difficulty scaling;
- variant multipliers, weaknesses, RES and control/debuff resistance;
- Toughness layers, recovery and phase carryover;
- every active/passive ability, target selector, parameter and status;
- AI initial state, sequence/candidate rules, cooldowns and deterministic
  fallback;
- charge/telegraph states, consecutive actions and target locks;
- summons, linked actors, shared HP, untargetability and victory contribution;
- boss phase entry/exit and every carry/reset field;
- at least one seeded action-sequence fixture for ordinary enemies and one
  fixture per phase for bosses.

The generated source AI path and skill sequence are the starting evidence. A
generic “enemy uses skills randomly” implementation is forbidden unless the
source evidence explicitly requires a weighted draw and defines its ordering.

## Approximation policy

Approximation is allowed because exact hidden values are not always publicly
observable. It is never implicit.

Every approximate field declares:

- the exact field and value;
- why exact evidence is unavailable;
- the released mechanic facts it preserves;
- a tolerance/range where numeric;
- affected tests;
- replacement conditions when stronger evidence appears.

Safe approximations preserve qualitative behavior. Examples include a hidden
animation hit split whose total multiplier and trigger cardinality are known, or
an undocumented enemy tie-break resolved by stable ID. Unsafe approximations
change a mechanic category, such as turning a once-per-action trigger into
once-per-hit, a snapshot DoT into dynamic stats, or an authored AI sequence into
uniform random choice.

## Source and copyright boundary

- Raw released repositories live only in `.cache/content-reference/`.
- Do not commit source assets, raw ability programs or bulk descriptions.
- Commit independently named facts, canonical numbers, operation semantics and
  short original summaries.
- Preserve repository revision, relative path, file SHA-256 and source-text hash.
- Announced/unreleased records remain disabled and are not approximated into
  playable content.

## Excel mapping rules

Future `.xlsx` tables may improve the normalized structure, but must preserve:

- descriptive project-owned keys;
- canonical decimal strings;
- explicit child-row ordering;
- separate character form, ability, level, phase, hit, Trace patch and Eidolon
  patch concepts;
- separate enemy template, variant, ability, AI state/candidate and phase
  concepts;
- fact-level provenance and approximation labels;
- a deterministic mapping report back to every required reference record.

Sora validation proves structure and references. `starclock-data` validation
proves domain invariants. Golden battle fixtures prove behavior. None replaces
the other two.
