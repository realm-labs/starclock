# Divergent Universe Service and Adventure Boundary

`G11-P2-B4` closes the remaining service-NPC and Adventure parent obligations
without adding service implementations or action-game simulators.

## Mode service NPCs

The 23 selected `RogueTournNPC` rows are the `RogueNPC_410` entries not
claimed by any current handbook Occurrence. Each publishes an exact graph
path. All 23 paths are absent from
`turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568`, verified by
Git object lookup.

The three separately inventoried gamble/Curio-composition service graphs and
the P1-B8 workbench tables do not contain an explicit join to these NPC IDs.
Consequently, the NPCs remain `UnclassifiedMissingGraph` with empty choices.
Names, adjacent IDs and the existence of another service graph do not prove a
service kind.

## Adventure outcomes

All 32 `RogueTournAdventureRoom` rows are retained. Adventure action gameplay
is excluded; the authoring contract accepts only an already-resolved external
result and applies an abstract settlement.

Twenty-six rows resolve their exact parameter group:

- nine monster-capture score-threshold results;
- eight destructible-object score-threshold results;
- two turntable reward-tier results;
- three laser round-score results; and
- four candy-crash round-threshold results.

The six Wolf Gun rows reference parameter groups 101 or 102. The released
snapshot has no parameter-group table for those IDs.
`RogueWolfGunMiracleTarget` lists potential Curio targets but has no
parameter-group key, so it cannot prove that join. Those six rows retain only
the accepted-external-result boundary and remain fail closed.

All numeric parameter values are canonical decimal strings. No movement,
aiming, timing controls, target placement, presentation or rewards are
simulated or copied into the reference pack.
