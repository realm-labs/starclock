# Threshold Protocol and Astronomical Division Boundary

This dossier freezes the `G11-P1-B7` interpretation of Version 4.4 Threshold
Protocols, Astronomical Divisions, Star-Pioneer/Practice Mode and Cognoculi.
It is reference data only.

## Protocols and enemy changes

`RogueTournDivisionEffect` defines Threshold Protocol levels 1–8. Each row
publishes plane-scaled maximum enemy ATK, Max HP and SPD increases; levels 6–8
also publish Max Toughness increases. The normalized programs preserve every
canonical parameter and independently classify the additional mode rule.

Protocol 3 states that First- and Second-Plane bosses change, but does not
identify the replacements. The identity field therefore remains
`ChangedIdentityDeferredToP2B5`; no enemy is selected by name, ID range or
adjacency. Other rule text defines domain, Equation, mask, Grand Miracle,
Berserk, store-price and entry-grant changes without importing reward prose.

## Astronomical Division

`RogueTournDivision` defines levels 1–9. Levels 1–8 bind the same-numbered
Protocol effect; level 9 is a terminal Division with no effect row. Progress
boundaries are retained exactly, including the absence of a numeric progress
field at level 9.

Division hints establish these Cognoculi retention boundaries:

- levels 3–4: Cognoculi do not extinguish;
- levels 5–6: Cognoculi are retained after clearing the First Plane;
- level 7: Cognoculi are retained after clearing the Second Plane;
- levels 1, 2 and 8: no row-level retention hint;
- level 9: terminal Division.

No missing hint is converted into a guessed loss amount or checkpoint.

## Star-Pioneer and Practice modes

Released bilingual Astronomical Division rules state that:

- Star-Pioneer uses the Protocol matching the current Division; successful
  finalization lights Cognoculi, unsuccessful finalization may extinguish
  them, and the Division itself never decreases.
- Practice Mode allows any unlocked Protocol up to the current Division cap
  and changes neither Division nor Cognoculi.

These rules are represented as separate mode rows. Account rewards and
presentation remain excluded.

Any concrete boss replacement, Cognoculi gain/loss quantity or missing
retention checkpoint requires a released selector/transition source before it
can replace the current explicit boundary.
