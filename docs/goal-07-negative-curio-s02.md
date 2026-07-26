# Goal 07 Negative Curio Partition S02

`G07-P3-M12-S02` executes the second Standard Simulated Universe negative
Curio slice:

- Normal Code (`universe.curio.49`), repairing state;
- Elegant Code (`universe.curio.51`);
- Mysterious Code (`universe.curio.53`);
- Recursive Code (`universe.curio.55`);
- Star Bait (`universe.curio.57`);
- Insect Web (`universe.curio.59`);
- I.O.U. Dispenser (`universe.curio.60`).

The partition owns 16 records, 16 rules and the Skill Point review fixture.
`Universe.xlsx`, `UniverseBindings.xlsx` and `UniverseEvidence.xlsx` remain the
editable authority. The workbooks are recreated through openpyxl, exported by
Sora 0.3.0 and checked against a partition-local golden. Runtime code consumes
validated domain definitions rather than workbook or generated-row types.

## Corrected released Code values

The pinned upstream `RogueMiracleEffect` and display tables distinguish each
repairing/fixed pair. Elegant Code advances a random enemy by 35% while
repairing and advances the acting ally by 25% after repair. Mysterious Code
grants 35% damage to the remaining enemies while repairing and 25% damage to
all allies after repair. The normalized staging rows previously repeated the
35% repairing values for both fixed states; this partition corrects those two
fixed values to 25% before rebuilding the workbooks and Sora bundle.

Normal Code dynamically reads current and maximum HP. Below 50% HP it adds
35% vulnerability for ordinary, DoT, Break, Super Break, additional, joint,
Elation and true damage. Recursive Code spends one additional Skill Point
after Skill use without underflow while repairing; after repair, Basic ATK
recovers one additional Skill Point. All values use exact six-decimal domain
scalars.

## Action order and Parasitized

Star Bait advances the acting character by 10% after every completed action.
Its 20% map movement bonus is a spatial-only effect and therefore has no
projection in the scene-free Activity or combat model.

Insect Web applies one permanent, non-dispellable Parasitized effect to the
highest-ATK living ally at battle entry. The effect grants 50% ATK. At that
ally's turn start it consumes 20% of current HP with a one-HP floor. When the
marked ally is downed, the effect is removed and transferred to one stable
Reward-independent random living ally. Selector ordering, the
`behavior-choice` RNG purpose and the effect's Replace stack policy make
selection and transfer replay deterministic.

These mechanics lower to ordinary selectors, triggers, effects, modifiers,
resource operations and action-gauge operations. Filter selectors are
explicit dependencies of their programs, so trigger evaluation receives the
same canonical selector snapshot as operation evaluation.

## I.O.U. battle settlement

I.O.U. Dispenser is expressed by the generic negative-Curio runtime effect
`SuppressBattleFragmentsThenDoubleCurrent`. While it is owned, post-battle
fragment additions are removed recursively from ordinary and conditional
Activity projections. Existing full-HP Curio fragment rewards use the same
suppression boundary. Every won battle increments a Curio event counter.
After the fifth win, one checked operation adds the current fragment balance
to itself and the ordinary generic destroy-and-count operation removes the
Curio.

The counter, fragment slot and destruction records are run-owned state, so
replay hashes cover every suppressed settlement and the final doubling. No
encounter ID, Curio ID or resolver branch is required.

## Revisions and executable evidence

`standard-universe-entry-v14`, `standard-universe-topology-v14` and
`standard-universe-catalog-v2` identify this configuration and runtime
revision. Combat integration tests execute the fixed Recursive Code Skill
Point result, a real Mysterious Code defeat trigger, Insect Web application
and current-HP drain, and structurally cover its downed transfer. Run tests
execute five won-battle settlements and prove suppression, doubling and
destruction.

All ten native-handler candidates close as `IrSufficient`. The partition adds
no numeric approximation and no native handler.
