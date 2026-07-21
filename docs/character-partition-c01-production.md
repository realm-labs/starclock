# Character partition C01 production import

`G01-P7-C01` promotes the first frozen character partition through the
repository-owned Excel/Sora production path. Its membership is Acheron, Anaxa,
Archer, Argenti, Arlan, Ashveil, Aventurine and Bailu.

The deterministic author reads only the prepared Version 4.4 pack with digest
`0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a`.
It emits 8 character definitions, 52 abilities with 1,417 level-parameter
values, 144 battle-relevant Trace nodes and complete E1-E6 sets (48 Eidolons).
The import preserves each prepared record's evidence quality. In particular,
Archer retains `ExactPreviousRelease`/`ExactPreviousReleaseText`; it is not
relabeled as a Version 4.4 observation.

The partition owns internal IDs `30000..39999`. The author replaces that slice
in place, so the V1B author remains independently reproducible after later
partitions land. Source-skill aliases that resolve to one compiled ability are
collapsed before level patches are emitted. Technique unlock nodes do not
incorrectly raise a level-one Technique.

Executable goldens cover all 14 production characters at E0 and complete E6.
C01-specific assertions pin Anaxa's five-hit Wind Skill, Aventurine's seven-hit
DEF-scaling follow-up, and Arlan's Skill-Point-neutral HP-funded Skill alongside
his ordinary Basic ATK gain. Hit shares use exact millionth-unit remainder
allocation; Sora and the domain loader reject non-conserving plans.

The resulting bundle is `77e8ac150bad7b3c81e93e51f01a691a9f9fc94c76756dfceaa06fe0f36218de`.
It contains 1,118 identities, 879 enabled identities, 165 abilities, 108 hit
plans and 14 character forms. Frozen goal coverage is 44/283 overall and 14/88
released character combat forms.
