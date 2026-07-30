# Active encounters and enemy closure

The active Version 4.4 selector reaches five released StageConfig rows. Each
has two ordered waves. The three Knight stages are level 95, normal King is
level 100 and Plight is level 120. Their 16 explicit slots contain 12 distinct
concrete variants; recursive `SummonIDList` and named summon custom values
close the reachable set to 27 variants and 26 templates.

The enemy dossiers preserve exact bilingual names, template ranks and base
statistics, variant ratios, weaknesses, resistances, explicit summons, skills,
AI/config paths and phase markers. The closure includes 115 exact
MonsterSkill rows. Skill phase lists are retained as source phase markers; no
runtime phase state machine is inferred.

Fifty-two MonsterStatus rows are reachable because their exact
`ModifierName` occurs in one or more of the 73 enabled configuration
programs. Status ownership is therefore recorded at the transitive program
closure instead of being guessed from names or numeric prefixes. The 73
ability-binding rows retain source paths, selectors and digests while
explicitly excluding program bodies and runtime executability.
