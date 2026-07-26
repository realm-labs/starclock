# Goal 07 Curio Partition S06

`G07-P3-M11-S06` executes three positive Standard Simulated Universe Curios:

- Beacon Coloring Paste (`universe.curio.69`);
- Fortune Glue (`universe.curio.7`);
- The Parchment That Always Eats (`universe.curio.8`).

The partition owns six records and six rules. The definitions, parameters,
rules and provenance remain authoritative in `Universe.xlsx`,
`UniverseBindings.xlsx` and `UniverseEvidence.xlsx`. The openpyxl partition
command verifies those rows against the committed Sora 0.3.0 bundle and emits
the derived partition golden.

## Postcombat Blessing offers

Fortune Glue adds a conditional candidate filter to the generic random-offer
policy. While the Curio is active, every visible option is drawn without
replacement from the eligible three-star Blessing set. Rerolling preserves the
restriction and does not consume the Curio. Selecting an offered Blessing runs
a checked selection prefix in the same Activity transaction: the Curio is
removed, its state and charge are cleared, and the destroyed-Curio counter is
incremented before ordinary Blessing settlement commits.

Beacon Coloring Paste uses an independent Reward RNG purpose after the final
visible subset has been selected. Exactly one visible option is recorded in a
private bounded marker map. A reroll replaces the complete map, so an
unselected marker cannot leak into a later offer. Acquiring the marked
Blessing adds one enhancement level. The shared acquisition expression caps
the final inventory value at level two, including when Warping Compound Eye
also affects the selected one-star Blessing.

The reward compiler contains no Curio-ID branch. The mode compiler translates
the validated Curio contribution into generic candidate-filter, selected-item
marker, selection-prefix and bounded-inventory primitives.

## Battle entry damage

The Parchment That Always Eats lowers to generic Rule IR at
`BattleStarted`. One stable all-enemy selector iterates the present enemy
targets, queries each target's own maximum HP and applies true damage equal to
30% with explicit nearest-ties-even fixed-point rounding. The once-per-battle
trigger is attached to the first player rule owner and contains no native
handler.

The retained upstream parameter list also contains a second value, `5`, which
does not appear in the released public description and is not consumed by the
published 30% formula. It remains preserved in Excel/Sora evidence but is not
assigned guessed runtime semantics.

## Revisions and executable evidence

`standard-universe-entry-v12` and `standard-universe-topology-v12` identify the
new private offer marker and reward-selection semantics. Generic Activity tests
prove deterministic candidate intersection, marker replacement, independent
draws and atomic selection prefixes. Production topology tests prove all 579
reward nodes carry the S06 policies. Combat integration tests execute the exact
30% maximum-HP damage against enemies with different maximum HP values.

All three native-handler reviews close as `IrSufficient`.
